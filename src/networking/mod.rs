use futures::future::BoxFuture;
use std::borrow::Cow;

pub trait Command {
    fn from_protocol(
        name: String,
        args: impl Iterator<Item = String>,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized;

    fn handle(&self) -> BoxFuture<'static, ()>;
}
