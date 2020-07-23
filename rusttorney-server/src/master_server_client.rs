#![allow(unused)]
use crate::config::Config;
use crate::networking::Command;
use anyhow::Error;
use futures::future::BoxFuture;
use serde::export::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::pin::Pin;
use std::task::Context;
use tokio::io::{AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::macros::support::Poll;
use tokio::net::TcpStream;
use tokio::stream::{Stream, StreamExt};

#[derive(Debug, Command)]
pub enum MasterServerCommand {
    #[command(code = "CHECK")]
    Check,
    #[command(code = "PONG")]
    Pong,
    #[command(code = "NOSERV")]
    NOSERV,
    // Other(String), // TODO: discuss why this exists
}

pub trait CommandReader:
    Stream<Item = Result<MasterServerCommand, tokio::io::Error>>
{
}

impl<S: Stream<Item = Result<MasterServerCommand, tokio::io::Error>>>
    CommandReader for S
{
}

pub struct TcpCommandReader {
    reader: ReadHalf<TcpStream>,
}

impl TcpCommandReader {
    pub fn new(reader: ReadHalf<TcpStream>) -> Self {
        Self { reader }
    }
}

impl Stream for TcpCommandReader {
    type Item = Result<MasterServerCommand, tokio::io::Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MasterServerClient<'a, R, W>
where
    R: CommandReader + Unpin,
    W: AsyncWrite + Unpin,
{
    config: &'a Config<'a>,
    software: &'a str,
    reader: R,
    writer: W,
}

#[derive(Debug)]
enum MasterServerClientState {
    WaitCommand,
    WaitPong,
}

impl<'a, R, W> MasterServerClient<'a, R, W>
where
    R: CommandReader + Unpin,
    W: AsyncWrite + Unpin,
{
    pub fn new(
        config: &'a Config<'a>,
        software: &'a str,
        reader: R,
        writer: W,
    ) -> Self {
        MasterServerClient { config, software, reader, writer }
    }

    pub async fn connection_loop(&mut self) -> Result<(), anyhow::Error> {
        let mut state = MasterServerClientState::WaitCommand;
        loop {
            let mes: MasterServerCommand =
                self.reader.next().await.ok_or_else(|| {
                    tokio::io::Error::new(
                        tokio::io::ErrorKind::UnexpectedEof,
                        "unexpected end",
                    )
                })??;
            match mes {
                MasterServerCommand::Check => {
                    self.send_message("PING#%").await?;
                    state = MasterServerClientState::WaitPong;
                }
                MasterServerCommand::Pong => {
                    match state {
                        MasterServerClientState::WaitPong => {
                            state = MasterServerClientState::WaitCommand;
                        }
                        MasterServerClientState::WaitCommand => { /* TODO: log this */
                        }
                    }
                }
                MasterServerCommand::NOSERV => {
                    self.send_message(self.pack_server_info()).await?;
                } // MasterServerCommand::Other(_mes) => { /* TODO: log this */ }
            }
        }
    }

    pub async fn send_message<T: AsRef<str>>(
        &mut self,
        message: T,
    ) -> Result<(), tokio::io::Error> {
        self.writer.write(message.as_ref().as_bytes()).await?;
        self.writer.flush().await?;
        Ok(())
    }

    fn pack_server_info(&self) -> String {
        let cfg = &self.config;
        let port = match cfg.general.websocket_port {
            Some(wsport) => format!("{}&{}", cfg.masterserver.port, wsport),
            _ => format!("{}", cfg.masterserver.port),
        };
        format!(
            "SCC#{}#{}#{}#{}#%",
            port,
            cfg.masterserver.name,
            cfg.masterserver.description,
            self.software
        )
        // TODO: do this with parser struct
        // TODO 2: add needed enum variant and `to_message()` it
    }
}

impl<'a> MasterServerClient<'a, TcpCommandReader, WriteHalf<TcpStream>> {
    pub async fn from_config_with_connect(
        config: &'a Config<'a>,
        software: &'a str,
    ) -> Result<
        MasterServerClient<'a, TcpCommandReader, WriteHalf<TcpStream>>,
        tokio::io::Error,
    > {
        let stream = TcpStream::connect(format!(
            "{}:{}",
            config.masterserver.ip, config.masterserver.port
        ))
        .await?;
        let (reader, writer) = tokio::io::split(stream);
        Ok(Self::new(config, software, TcpCommandReader::new(reader), writer))
    }
}
