use crate::command::ClientCommand;
use crate::networking::Command;
use crate::{command::AOMessageCodec, config::Config};
use anyhow::Error;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use crate::client_manager::ClientManager;
use futures::stream::SplitSink;
use futures::SinkExt;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_postgres::NoTls;
use tokio_util::codec::{Decoder, Framed};

#[rustfmt::skip]
#[derive(Debug)]
pub enum ServerCommand {
    Handshake(String)
}

impl Command for ServerCommand {
    fn from_protocol(
        _name: String,
        _args: impl Iterator<Item = String>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Err(anyhow::anyhow!("Cannot be made from protocol! (Server response)"))
    }

    fn ident(&self) -> &str {
        use ServerCommand::*;

        match self {
            Handshake(_) => "HI",
        }
    }

    fn extract_args(&self) -> Vec<&str> {
        use ServerCommand::*;

        match self {
            Handshake(str) => vec![str],
            // _ => None,
        }
    }
}

pub struct AOServer<'a> {
    config: &'a Config<'a>,
    db_pool: Pool<PostgresConnectionManager<NoTls>>,
    client_manager: Arc<Mutex<ClientManager>>,
}

pub struct AO2MessageHandler {
    socket: SplitSink<Framed<TcpStream, AOMessageCodec>, ServerCommand>,
    db_pool: Pool<PostgresConnectionManager<NoTls>>,
    client_manager: Arc<Mutex<ClientManager>>,
}

impl AO2MessageHandler {
    pub fn new(
        socket: SplitSink<Framed<TcpStream, AOMessageCodec>, ServerCommand>,
        db_pool: Pool<PostgresConnectionManager<NoTls>>,
        client_manager: Arc<Mutex<ClientManager>>,
    ) -> Self {
        Self { socket, db_pool, client_manager }
    }

    pub async fn handle(
        &mut self,
        command: ClientCommand,
    ) -> Result<(), anyhow::Error> {
        match command {
            ClientCommand::Handshake(hdid) => {
                let conn = self.db_pool.get().await?;
                drop(conn);
                log::debug!("Handshake from HDID: {}", hdid);
                self.handle_handshake(hdid).await
            }
            _ => Ok(()),
        }
    }

    pub async fn handle_handshake(
        &mut self,
        _hdid: String,
    ) -> Result<(), anyhow::Error> {
        self.socket.send(ServerCommand::Handshake(1111.to_string())).await?;

        Ok(())
    }
}

impl<'a> AOServer<'a> {
    pub fn new(
        config: &'a Config<'a>,
        db_pool: Pool<PostgresConnectionManager<NoTls>>,
        client_manager: Arc<Mutex<ClientManager>>,
    ) -> anyhow::Result<Self> {
        Ok(Self { config, db_pool, client_manager })
    }

    async fn migrate(&mut self) -> anyhow::Result<()> {
        let mut conn = self.db_pool.get().await?;
        let stmt = conn.prepare("SELECT db_version FROM general_info").await;

        let current_version = match stmt {
            Err(_) => {
                log::info!("Migrating database...");
                let mut migrations = std::fs::read_dir("migrations")?
                    .map(|res| res.map(|e| e.path()))
                    .collect::<Result<Vec<_>, std::io::Error>>()?;

                migrations.sort();

                for migration in migrations {
                    log::debug!("Executing migration: {:?}", &migration);
                    let migration_stmt = std::fs::read_to_string(migration)?;
                    let tx = conn.transaction().await?;
                    tx.batch_execute(&migration_stmt).await?;
                    tx.commit().await?;
                }
                log::info!("Succesfully migrated!");
                log::debug!("GCing the DB...");
                conn.query("VACUUM", &[]).await?;

                let row = conn
                    .query_one("SELECT db_version FROM general_info", &[])
                    .await?;
                row.get::<_, i32>(0_usize)
            }
            Ok(stmt) => {
                let row = conn.query_one(&stmt, &[]).await?;
                row.get::<_, i32>(0_usize)
            }
        };
        log::info!("Current DB version is: v{}", current_version);
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        use futures::StreamExt;

        self.migrate().await?;

        log::info!("Starting up the server...");
        let addr = format!("127.0.0.1:{}", self.config.general.port);
        log::info!("Binding to address: {}", &addr);

        let mut listener = TcpListener::bind(addr).await?;

        loop {
            let db_pool = self.db_pool.clone();
            let (socket, c) = listener.accept().await?;
            log::debug!("got incoming connection from: {:?}", &c);

            tokio::spawn(async move {
                let (msg_sink, mut msg_stream) =
                    AOMessageCodec.framed(socket).split();
                let mut handler = AO2MessageHandler::new(msg_sink, db_pool);

                while let Some(msg_res) = msg_stream.next().await {
                    match msg_res {
                        Ok(msg) => {
                            log::debug!("Got command! {:?}", &msg);
                            handler.handle(msg).await.unwrap();
                        }
                        Err(err) => log::error!("Got error! {:?}", err),
                    }
                }
            });
        }
    }
}
