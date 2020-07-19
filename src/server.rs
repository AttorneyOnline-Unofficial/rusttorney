use crate::{config::Config, networking::Command};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bytes::{Buf, BufMut, BytesMut};
use futures::stream::SplitSink;
use futures::SinkExt;
use std::{
    borrow::Cow,
    char::REPLACEMENT_CHARACTER,
    fmt::{Debug, Display},
    str::FromStr,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_postgres::NoTls;
use tokio_util::codec::{Decoder, Encoder, Framed};

const MAGIC_SEPARATOR: u8 = b'#';
const MAGIC_END: u8 = b'%';

#[rustfmt::skip]
#[derive(Debug)]
pub enum ClientCommand {
    Handshake(String),                           // HI#<hdid:String>#%
    ClientVersion(u32, String, String),          // ID#<pv:u32>#<software:String>#<version:String>#%
    KeepAlive,                                   // CH
    AskListLengths,                              // askchaa
    AskListCharacters,                           // askchar
    CharacterList(u32),                          // AN#<page:u32>#%
    EvidenceList(u32),                           // AE#<page:u32>#%
    MusicList(u32),                              // AM#<page:u32>#%
    AO2CharacterList,                            // AC#%
    AO2MusicList,                                // AM#%
    AO2Ready,                                    // RD#%
    SelectCharacter(u32, u32, String),           // CC<client_id:u32>#<char_id:u32#<hdid:String>#%
    ICMessage,                                   // MS
    OOCMessage(String, String),                  // CT#<name:String>#<message:String>#%
    PlaySong(u32, u32),                          // MC#<song_name:u32>#<???:u32>#%
    WTCEButtons(String),                         // RT#<type:String>#%
    SetCasePreferences(String, CasePreferences), // SETCASE#<cases:String>#<will_cm:boolean>#<will_def:boolean>#<will_pro:boolean>#<will_judge:boolean>#<will_jury:boolean>#<will_steno:boolean>#%
    CaseAnnounce(String, CasePreferences),       // CASEA
    Penalties(u32, u32),                         // HP#<type:u32>#<new_value:u32>#%
    AddEvidence(EvidenceArgs),                   // PE#<name:String>#<description:String>#<image:String>#%
    DeleteEvidence(u32),                         // DE#<id:u32>#%
    EditEvidence(u32, EvidenceArgs),             // EE#<id:u32>#<name:String>#<description:String>#<image:String>#%
    CallModButton(Option<String>),               // ZZ?#<reason:String>?#%
}

#[rustfmt::skip]
#[derive(Debug)]
pub enum ServerCommand {
    Handshake(String)
}

impl ServerCommand {
    pub fn extract_args(&self) -> Option<Vec<&str>> {
        use ServerCommand::*;

        match self {
            Handshake(str) => Some(vec![str]),
            // _ => None,
        }
    }

    pub fn ident(&self) -> &str {
        use ServerCommand::*;

        match self {
            Handshake(_) => "HI",
        }
    }
}

impl Command for ClientCommand {
    fn from_protocol<'a>(
        name: String,
        mut args: impl Iterator<Item = String>,
    ) -> Result<Self, anyhow::Error> {
        // let args = &mut args;
        let on_err = || {
            anyhow::anyhow!(
                "Amount of arguments for command {} does not match!",
                &name
            )
        };

        fn next<E, T, F>(
            mut args: impl Iterator<Item = String>,
            on_err: F,
        ) -> Result<T, anyhow::Error>
        where
            E: Display,
            T: FromStr<Err = E>,
            F: Fn() -> anyhow::Error,
        {
            args.next()
                .ok_or_else(on_err)
                .map(|s| s.parse::<T>().map_err(|e| anyhow::anyhow!("{}", e)))
                .and_then(std::convert::identity)
        }

        let res = match name.as_str() {
            "HI" => Ok(Self::Handshake(next(&mut args, on_err)?)),
            _ => Err(on_err()),
        };

        if args.next().is_some() {
            return Err(on_err());
        };

        res
    }
}

#[derive(Debug)]
pub struct EvidenceArgs {
    pub name: String,
    pub description: String,
    pub image: String,
}

#[derive(Debug)]
pub struct CasePreferences {
    pub cm: bool,
    pub def: bool,
    pub pro: bool,
    pub judge: bool,
    pub jury: bool,
    pub steno: bool,
}

#[derive(Debug)]
pub struct AOMessage {
    pub command: ClientCommand,
}

pub struct AOMessageCodec;

impl Decoder for AOMessageCodec {
    type Item = ClientCommand;
    type Error = anyhow::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let magic_b = src.iter().position(|&byte| byte == MAGIC_SEPARATOR);
        if let Some(i) = magic_b {
            let cmd = src.split_to(i);

            let cmd_name = ignore_ill_utf8(&cmd);
            src.advance(1);

            let protocol_end = src.iter().rposition(|&b| b == MAGIC_END);

            if let Some(i) = protocol_end {
                let args = src.split_to(i - 2);

                src.clear();

                return Ok(Some(Command::from_protocol(
                    cmd_name,
                    args.as_ref()
                        .split(|&b| b == MAGIC_SEPARATOR)
                        .map(|s| ignore_ill_utf8(s)),
                )?));
            }
        }

        Ok(None)
    }
}

fn ignore_ill_utf8(v: &[u8]) -> String {
    let str = String::from_utf8_lossy(&v);

    match str {
        Cow::Owned(mut own) => {
            own.retain(|c| c != REPLACEMENT_CHARACTER);
            own
        }
        Cow::Borrowed(brw) => brw.to_owned(),
    }
}

pub struct AOServer<'a> {
    config: &'a Config<'a>,
    db_pool: Pool<PostgresConnectionManager<NoTls>>,
}

pub struct AO2MessageHandler {
    socket: SplitSink<Framed<TcpStream, AOMessageCodec>, ServerCommand>,
    db_pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl AO2MessageHandler {
    pub fn new(
        socket: SplitSink<Framed<TcpStream, AOMessageCodec>, ServerCommand>,
        db_pool: Pool<PostgresConnectionManager<NoTls>>,
    ) -> Self {
        Self { socket, db_pool }
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

impl Encoder<ServerCommand> for AOMessageCodec {
    type Error = anyhow::Error;

    fn encode(
        &mut self,
        item: ServerCommand,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let args_len = match item.extract_args() {
            Some(args) => args.iter().fold(0, |i, s| i + s.len() + 1),
            None => 0,
        };
        let ident = item.ident();
        #[rustfmt::skip]
        let reserve_len =
            // 2 - 8
            ident.len() +
            // #
            1 +
            // args_len is every arg + #
            args_len +
            // %
            1;
        dst.reserve(reserve_len);
        dst.put(ident.as_bytes());
        dst.put_u8(b'#');

        if let Some(args) = item.extract_args() {
            for arg in args {
                dst.put(arg.as_bytes());
                dst.put_u8(b'#');
            }
        }

        dst.put_u8(b'%');
        Ok(())
    }
}

impl<'a> AOServer<'a> {
    pub fn new(
        config: &'a Config<'a>,
        db_pool: Pool<PostgresConnectionManager<NoTls>>,
    ) -> anyhow::Result<Self> {
        Ok(Self { config, db_pool })
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
