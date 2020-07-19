use crate::{command::AOMessageCodec, config::Config};
use tokio::net::TcpListener;
use tokio_util::codec::Decoder as _;

pub struct AOServer<'a> {
    config: Config<'a>,
}

impl<'a> AOServer<'a> {
    pub fn new(config: Config<'a>) -> anyhow::Result<Self> {
        Ok(Self { config })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        use futures::StreamExt;

        log::info!("Starting up the server...");
        let addr = format!("127.0.0.1:{}", self.config.general.port);
        log::info!("Binding to address: {}", &addr);

        let mut listener = TcpListener::bind(addr).await?;

        loop {
            let (socket, c) = listener.accept().await?;
            log::debug!("got incoming connection from: {:?}", &c);

            let msg_stream = AOMessageCodec.framed(socket);

            tokio::spawn(msg_stream.for_each(move |msg| async move {
                match msg {
                    Ok(msg) => {
                        log::debug!("Got message: {:?}", msg);
                    }
                    Err(err) => log::error!("Got error: {:?}", err),
                }
            }));
        }
    }
}
