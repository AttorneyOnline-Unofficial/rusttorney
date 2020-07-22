pub use command_derive_impl::*;

pub trait Command: Sized {
    fn from_protocol<I, S>(code: &str, args: I) -> Result<Self, anyhow::Error>
    where
        I: Iterator<Item = S>,
        S: AsRef<str>;

    fn ident(&self) -> &str;

    fn extract_args(&self) -> Vec<String>;

    fn to_message(&self) -> String {
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
