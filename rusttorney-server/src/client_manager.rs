use futures::stream::SplitSink;
use futures::SinkExt;
use std::collections::{BinaryHeap, HashSet};

use crate::command::ServerCommand;
use crate::networking::codec::AOMessageCodec;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct Client {
    is_checked: bool,
    // socket: FramedWrite,
    hdid: String,
    id: u32,
    char_id: i32,
    // area: AreaManager,
    // server: Server
    name: String,
    fake_name: String,
    is_mod: bool,
    ipid: u32,
    // TODO: other fields
}

pub struct ClientManager {
    clients: HashSet<Client>,
    // config: Config<'a>,
    cur_id: BinaryHeap<u8>,
}

impl ClientManager {
    pub fn new(playerlimit: u8) -> Self {
        let mut cur_id = BinaryHeap::new();
        (0..playerlimit).for_each(|i| cur_id.push(i));
        Self {
            clients: HashSet::new(),
            // config,
            cur_id,
        }
    }

    pub async fn new_client(
        &mut self,
        socket: &mut SplitSink<
            Framed<TcpStream, AOMessageCodec>,
            ServerCommand,
        >,
    ) -> Result<(), anyhow::Error> {
        let user_id = match self.cur_id.pop() {
            Some(uid) => uid,
            None => {
                socket
                    .send(ServerCommand::Handshake(
                        "This server is full!".into(),
                    ))
                    .await?;
                anyhow::bail!("This server is full!");
            }
        };
        socket
            .send(ServerCommand::Handshake(format!(
                "Your user id is: {}",
                user_id
            )))
            .await?;

        Ok(())
    }
}
