pub trait Command {
    fn from_protocol(
        name: String,
        args: impl Iterator<Item = String>,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized;

    fn ident(&self) -> &str;

    fn extract_args(&self) -> Vec<&str>;
}
