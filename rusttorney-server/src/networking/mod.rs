pub trait Command: Sized {
    fn from_protocol(
        name: String,
        args: impl Iterator<Item = String>,
    ) -> Result<Self, anyhow::Error>;

    fn ident(&self) -> &str;

    fn extract_args(&self) -> Vec<String>;

    fn to_message<S, I>(&self) -> String {
        let mut res = String::from(self.ident());
        res.push('#');
        self.extract_args().into_iter().for_each(|s| {
            res.push_str(&s);
            res.push('#');
        });
        res.push('%');
        res
    }
}

pub mod codec;
pub mod database;
