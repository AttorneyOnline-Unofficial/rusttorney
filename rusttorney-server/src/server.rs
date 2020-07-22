use crate::command::{ClientCommand, ServerCommand};
use crate::config::Config;

use crate::client_manager::{Client, ClientManager};
use crate::networking::codec::AOMessageCodec;
use crate::networking::database::DbWrapper;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

use futures::channel::mpsc;
use futures::channel::oneshot::{channel, Receiver, Sender};
use futures::lock::Mutex;
use std::convert::Infallible;
use std::net::IpAddr;
use std::ops::Deref;
use tokio::select;
use tokio::time::Duration;
use tokio_util::codec::{Decoder, Framed};

pub struct AOServer<'a> {
    config: &'a Config<'a>,
    db: DbWrapper,
    client_manager: Arc<Mutex<ClientManager>>,
}

pub struct AO2MessageHandler {
    socket: Framed<TcpStream, AOMessageCodec>,
    db: DbWrapper,
    client_manager: Arc<Mutex<ClientManager>>,
    ch_tx: mpsc::Sender<()>,
    client: Client,
    software: String,
    version: String,
}

impl AO2MessageHandler {
    pub async fn new(
        mut socket: Framed<TcpStream, AOMessageCodec>,
        db: DbWrapper,
        client_manager: Arc<Mutex<ClientManager>>,
        timeout: u64,
        timeout_tx: Sender<()>,
        ip: IpAddr,
    ) -> Result<Self, anyhow::Error> {
        let (ch_tx, mut ch_rx) = futures::channel::mpsc::channel(1);

        tokio::spawn(async move {
            let mut delay =
                tokio::time::delay_for(Duration::from_secs(timeout));

            loop {
                select! {
                    _ = &mut delay => {
                        timeout_tx.send(());
                        break;
                    }
                    next = ch_rx.next() => {
                        // When sender handles are dropped, next will return Ok(None)
                        // So we check is next has actual value, to reset delay
                        // Otherwise that means that sender was dropped (because parent struct was dropped) and so we end this task
                        if next.is_some() {
                            delay = tokio::time::delay_for(Duration::from_millis(timeout));
                        } else {
                            break;
                        }
                    }
                }
            }
        });

        let client = client_manager
            .lock()
            .await
            .new_client(&mut socket, ip.clone())
            .await?;
        log::info!(
            "Client with IPID: {} connected! His ip is: {}",
            &client.ipid,
            ip
        );

        Ok(Self {
            socket,
            db,
            client_manager,
            ch_tx,
            client,
            software: "rusttorney".into(),
            version: "0.0.1".into(),
        })
    }

    async fn handle(
        &mut self,
        command: ClientCommand,
    ) -> Result<(), anyhow::Error> {
        match command {
            ClientCommand::Handshake(hdid) => self.handle_handshake(hdid).await,
            ClientCommand::KeepAlive(_) => self.handle_keepalive().await,
            _ => Ok(()),
        }
    }

    async fn handle_handshake(
        &mut self,
        hdid: String,
    ) -> Result<(), anyhow::Error> {
        self.client.hdid = hdid.clone();
        self.client_manager.lock().await.update_client(self.client.clone());

        self.db.add_hdid(hdid, self.client.ipid).await?;

        self.socket
            .send(ServerCommand::ServerVersion(
                self.client.id,
                self.software.clone(),
                self.version.clone(),
            ))
            .await?;

        self.socket
            .send(ServerCommand::PlayerCount(self.player_count().await, 255))
            .await
    }

    async fn player_count(&self) -> u8 {
        self.client_manager.lock().await.clients.iter().fold(0, |acc, c| {
            if c.char_id != 1 {
                acc + 1
            } else {
                acc
            }
        })
    }

    async fn handle_keepalive(&mut self) -> Result<(), anyhow::Error> {
        self.ch_tx.send(()).await?;
        self.socket.send(ServerCommand::KeepAlive).await
    }

    async fn start_handling(
        mut self,
        mut timeout_rx: Receiver<()>,
    ) -> Result<(), anyhow::Error> {
        // main client connection loop
        loop {
            // run concurrently timeout receiver and decoder, getting messages and handling them
            select! {
                _ = &mut timeout_rx => {
                    return Err(anyhow::anyhow!("Client disconnected because of timeout!"));
                }
                res = self.socket.next() => {
                    if let Some(parsed) = res {
                        self.handle(parsed?).await?;
                    } else {
                        return Err(anyhow::anyhow!("Client disconnected!"));
                    }
                }
            }
        }
    }
}

impl<'a> AOServer<'a> {
    pub fn new(config: &'a Config<'a>, db: DbWrapper) -> anyhow::Result<Self> {
        let playerlimit = config.general.playerlimit;
        Ok(Self {
            config,
            db: db.clone(),
            client_manager: Arc::new(Mutex::new(ClientManager::new(
                playerlimit,
                db,
            ))),
        })
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
            let client_manager = self.client_manager.clone();
            let (socket, c) = listener.accept().await?;
            log::debug!("got incoming connection from: {:?}", &c);
            let timeout = self.config.timeout as u64;

            tokio::spawn(async move {
                let mut framed = AOMessageCodec.framed(socket);
                let (timeout_tx, timeout_rx) = channel();

                // https://github.com/AttorneyOnline/tsuserver3/blob/master/server/network/aoprotocol.py#L135
                framed.send(ServerCommand::Decryptor(34)).await.unwrap();

                let mut handler = AO2MessageHandler::new(
                    framed,
                    db,
                    client_manager,
                    timeout,
                    timeout_tx,
                    c.ip(),
                )
                .await
                .unwrap();

                handler
                    .start_handling(timeout_rx)
                    .await
                    .map_err(|e| log::error!("{}", e));
            });
        }
    }
}
