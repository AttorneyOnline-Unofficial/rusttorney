use futures::stream::SplitSink;
use futures::SinkExt;
use std::collections::{BinaryHeap, HashSet};

use crate::command::ServerCommand;
use crate::networking::codec::AOMessageCodec;
use crate::networking::database::DbWrapper;
use futures::channel::mpsc;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::codec::Framed;

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub struct Client {
    is_checked: bool,
    pub(crate) hdid: String,
    pub(crate) id: u8,
    pub(crate) char_id: i32,
    // area: AreaManager,
    name: String,
    fake_name: String,
    is_mod: bool,
    pub(crate) ipid: u32,
    // TODO: other fields
}

/// We hash and eq clients only by ipid
impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ipid.hash(state);
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.ipid == other.ipid
    }
}

impl Eq for Client {}

impl Client {
    pub fn new(user_id: u8, ipid: u32) -> Self {
        Self { id: user_id, ipid, ..Default::default() }
    }
}

pub struct ClientManager {
    pub(crate) clients: HashSet<Client>,
    // config: Config<'a>,
    cur_id: BinaryHeap<u8>,
    db: DbWrapper,
}

impl ClientManager {
    pub fn new(playerlimit: u8, db: DbWrapper) -> Self {
        let cur_id = (0..playerlimit).collect();
        Self {
            clients: HashSet::new(),
            // config,
            cur_id,
            db,
        }
    }

    pub async fn new_client(
        &mut self,
        socket: &mut Framed<TcpStream, AOMessageCodec>,
        ip: IpAddr,
    ) -> Result<Client, anyhow::Error> {
        // TODO: GeoIP
        {}

        // TODO: bans
        {}

        let user_id = match self.cur_id.pop() {
            Some(uid) => uid,
            None => {
                socket
                    .send(ServerCommand::BanReason(
                        "This server is full.".into(),
                    ))
                    .await?;
                anyhow::bail!("This server is full!");
            }
        };
        let ipid = self.db.ipid(ip).await? as u32;

        let client = Client::new(user_id, ipid);
        // We have to clone here to store each client in a HashSet
        self.clients.insert(client.clone());

        Ok(client)
    }

    pub fn update_client(&mut self, client: Client) {
        self.clients.replace(client);
    }
}
