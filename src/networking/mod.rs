pub trait Command {
    fn from_protocol(
        name: String,
        args: impl Iterator<Item = String>,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

pub mod codec;
pub mod aocommands;
pub mod master_server_client;
