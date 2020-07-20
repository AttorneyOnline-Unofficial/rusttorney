#![allow(unused)]
use crate::command::{
    assert_iterator_is_empty, next_arg, to_message, CommandReader,
    TcpCommandReader,
};
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

#[derive(Debug)]
enum MasterServerCommand {
    Check,
    Pong,
    NOSERV,
}

impl Command for MasterServerCommand {
    fn from_protocol(
        name: String,
        mut args: impl Iterator<Item = String>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let on_err = || {
            anyhow::anyhow!(
                "Amount of arguments for command {} does not match!",
                &name
            )
        };
        let args = &mut args;

        let res = match name.as_str() {
            "CHECK" => Ok(Self::Check),
            "PONG" => Ok(Self::Pong),
            "NOSERV" => Ok(Self::NOSERV),
            _ => Err(on_err()),
        }?;
        assert_iterator_is_empty(args).map(|_| res)
    }

    fn handle(&self) -> BoxFuture<'static, ()> {
        unimplemented!()
    }
}

#[derive(Debug)]
struct MasterServerClient<
    'a,
    R: CommandReader<MasterServerCommand> + Unpin,
    W: AsyncWrite + Unpin,
> {
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

impl<
        'a,
        R: CommandReader<MasterServerCommand> + Unpin,
        W: AsyncWrite + Unpin,
    > MasterServerClient<'a, R, W>
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
                    self.send_message(to_message(
                        "PING",
                        std::iter::empty::<String>(),
                    ))
                    .await?;
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
                }
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
        to_message(
            "SCC",
            [
                port.as_str(),
                cfg.masterserver.name,
                cfg.masterserver.description,
                self.software,
            ]
            .iter(),
        )
    }
}

impl<'a>
    MasterServerClient<
        'a,
        TcpCommandReader<MasterServerCommand>,
        WriteHalf<TcpStream>,
    >
{
    pub async fn from_config_with_connect(
        config: &'a Config<'a>,
        software: &'a str,
    ) -> Result<
        MasterServerClient<
            'a,
            TcpCommandReader<MasterServerCommand>,
            WriteHalf<TcpStream>,
        >,
        std::io::Error,
    > {
        let stream = TcpStream::connect(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::from(config.masterserver.ip.parse::<u32>().unwrap()),
            config.masterserver.port,
        )))
        .await?;
        let (reader, writer) = tokio::io::split(stream);
        Ok(Self::new(config, software, TcpCommandReader::new(reader), writer))
    }
}
