use crate::config::Config;

use futures::stream::SplitSink;
use futures::SinkExt;

use crate::networking::aocommands::{ClientCommand, ServerCommand};
use crate::networking::codec::AOMessageCodec;
use crate::networking::database::DbWrapper;
use tokio::net::{TcpListener, TcpStream};

use tokio_util::codec::{Decoder, Framed};

pub struct AOServer<'a> {
    config: &'a Config<'a>,
    db: DbWrapper,
}

pub struct AO2MessageHandler {
    socket: SplitSink<Framed<TcpStream, AOMessageCodec>, ServerCommand>,
    #[allow(unused)]
    db: DbWrapper,
}

impl AO2MessageHandler {
    pub fn new(
        socket: SplitSink<Framed<TcpStream, AOMessageCodec>, ServerCommand>,
        db: DbWrapper,
    ) -> Self {
        Self { socket, db }
    }

    pub async fn handle(
        &mut self,
        command: ClientCommand,
    ) -> Result<(), anyhow::Error> {
        match command {
            ClientCommand::Handshake(hdid) => {
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
    pub fn new(config: &'a Config<'a>, db: DbWrapper) -> anyhow::Result<Self> {
        Ok(Self { config, db })
    }

    async fn migrate(&mut self) -> anyhow::Result<()> {
        let mut conn = self.db.get().await?;
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
            let db = self.db.clone();
            let (socket, c) = listener.accept().await?;
            log::debug!("got incoming connection from: {:?}", &c);

            tokio::spawn(async move {
                let (msg_sink, mut msg_stream) =
                    AOMessageCodec.framed(socket).split();
                let mut handler = AO2MessageHandler::new(msg_sink, db);

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
