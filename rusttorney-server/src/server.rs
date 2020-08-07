use crate::command::{
    CasePreferences, ClientCommand, EvidenceArgs, ServerCommand,
};
use crate::config::Config;

use crate::client_manager::{Client, ClientManager};
use crate::networking::codec::AOMessageCodec;
use crate::networking::database::DbWrapper;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

use crate::music_list::MusicList;
use crate::prompt;
use futures::channel::mpsc;
use futures::channel::oneshot::{channel, Receiver, Sender};
use std::convert::Infallible;
use std::io::{stdin, Read};
use std::net::IpAddr;
use std::ops::Deref;
use std::path::PathBuf;
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_util::codec::{Decoder, Framed};

pub struct AOServer {
    config: Arc<Config>,
    music_list: Arc<MusicList>,
    db: DbWrapper,
    client_manager: Arc<Mutex<ClientManager>>,
}

pub struct AO2MessageHandler {
    pub(crate) socket: Framed<TcpStream, AOMessageCodec>,
    pub(crate) db: DbWrapper,
    pub(crate) client_manager: Arc<Mutex<ClientManager>>,
    pub(crate) ch_tx: mpsc::Sender<()>,
    pub(crate) client: Client,
    pub(crate) software: String,
    pub(crate) version: String,
    pub(crate) config: Arc<Config>,
}

impl AO2MessageHandler {
    pub async fn new(
        mut socket: Framed<TcpStream, AOMessageCodec>,
        db: DbWrapper,
        client_manager: Arc<Mutex<ClientManager>>,
        timeout_tx: Sender<()>,
        ip: IpAddr,
        config: Arc<Config>,
    ) -> Result<Self, anyhow::Error> {
        let (ch_tx, mut ch_rx) = futures::channel::mpsc::channel(1);
        let timeout = config.timeout as u64;

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

        let client =
            client_manager.lock().await.new_client(&mut socket, ip).await?;
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
            config,
        })
    }

    pub(crate) async fn player_count(&self) -> u8 {
        self.client_manager
            .lock()
            .await
            .clients
            .iter()
            .filter(|c| c.char_id != 1)
            .count() as u8
    }

    async fn start_handling(
        &mut self,
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
                        parsed?.handle(self).await?;
                    } else {
                        return Err(anyhow::anyhow!("Client disconnected!"));
                    }
                }
            }
        }
    }
}

impl AOServer {
    pub fn new(config: Arc<Config>, db: DbWrapper) -> anyhow::Result<Self> {
        let playerlimit = config.general.playerlimit;
        let music_list_str = std::fs::read_to_string("./config/music.toml")?;
        let music_list: Arc<MusicList> =
            Arc::new(toml::from_str(&music_list_str)?);
        Ok(Self {
            config,
            db: db.clone(),
            client_manager: Arc::new(Mutex::new(ClientManager::new(
                playerlimit,
                db,
            ))),
            music_list,
        })
    }

    async fn begin_migration(&mut self) -> anyhow::Result<()> {
        const LATEST_VERSION: i32 = 3;

        log::debug!("Getting pool connection for migration...");
        let mut conn = self.db.get().await?;
        let stmt = conn.prepare("SELECT db_version FROM general_info").await;

        let mut current_version = match stmt {
            Err(_) => {
                if !prompt("Begin the migration?") {
                    return Ok(());
                }
                self.migrate().await?
            }
            Ok(stmt) => {
                let row = conn.query_one(&stmt, &[]).await?;
                row.get(0_usize)
            }
        };

        if current_version < LATEST_VERSION {
            if !prompt("Begin the migration?") {
                return Ok(());
            }
            current_version = self.migrate().await?;
        };

        log::info!("Current DB version is: v{}", current_version);
        Ok(())
    }

    async fn migrate(&mut self) -> anyhow::Result<i32> {
        let mut conn = self.db.get().await?;
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
        conn.execute("VACUUM", &[]).await?;
        let row =
            conn.query_one("SELECT db_version FROM general_info", &[]).await?;
        Ok(row.get(0_usize))
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        use futures::StreamExt;

        self.begin_migration().await?;

        log::info!("Starting up the server...");
        let addr = format!(
            "{}:{}",
            self.config.general.host, self.config.general.port
        );
        log::info!("Binding to address: {}", &addr);

        let mut listener = TcpListener::bind(addr).await?;

        loop {
            let db = self.db.clone();
            let config = self.config.clone();
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
                    timeout_tx,
                    c.ip(),
                    config,
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
