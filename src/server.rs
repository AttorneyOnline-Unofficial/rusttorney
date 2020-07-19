use crate::{config::Config, networking::Command};
use bytes::{Buf, BufMut, BytesMut};
use futures::stream::SplitSink;
use futures::task::SpawnExt;
use futures::SinkExt;
use sqlx::executor::RefExecutor;
use sqlx::pool::PoolConnection;
use sqlx::prelude::SqliteQueryAs;
use sqlx::{Connect, Connection, Pool, SqliteConnection, SqlitePool};
use std::path::Path;
use std::{
    borrow::Cow,
    char::REPLACEMENT_CHARACTER,
    env,
    fmt::{Debug, Display},
    str::FromStr,
};
use tokio::net::{TcpListener, TcpStream};
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

pub struct AOServer<'a, C: Connect> {
    config: &'a Config<'a>,
    db_pool: Pool<C>,
}

pub struct AO2MessageHandler {
    socket: SplitSink<Framed<TcpStream, AOMessageCodec>, ServerCommand>,
}

impl AO2MessageHandler {
    pub fn new(
        socket: SplitSink<Framed<TcpStream, AOMessageCodec>, ServerCommand>,
    ) -> Self {
        Self { socket }
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

impl<'a> AOServer<'a>
{
    pub fn new(
        config: &'a Config<'a>,
        db_pool: Pool<C>,
    ) -> anyhow::Result<Self> {
        Ok(Self { config, db_pool })
    }

    async fn migrate(&mut self) -> anyhow::Result<()> {
        log::info!("Migrating database...");
        let mut conn = self.db_pool.acquire().await?;
        sqlx::query("PRAGMA foreign_keys = ON").execute(&mut conn).await?;

        if !Path::new(
            &env::var("DATABASE_URL")
                .unwrap_or("sqlite:storage/db.sqlite3".into()),
        )
        .exists()
        {
            let v1_migration = std::fs::read_to_string("migrations/v1.sql")?;
            sqlx::query(&v1_migration).execute(&mut conn).await?;
        }

        for version in 2..=3 {
            self.migrate_to_version(version, &mut conn).await?;
        }
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
            let (socket, c) = listener.accept().await?;
            log::debug!("got incoming connection from: {:?}", &c);

            tokio::spawn(async move {
                let (msg_sink, mut msg_stream) =
                    AOMessageCodec.framed(socket).split();
                let mut handler = AO2MessageHandler::new(msg_sink);

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
