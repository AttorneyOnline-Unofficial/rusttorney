use command_derive::*;

#[derive(Debug, Command, PartialEq)]
pub enum ClientCommand {
    #[command(code = "HI")]
    Handshake(String),

    #[command(code = "ID")]
    ClientVersion(u32, String, String),

    #[command(code = "CH")]
    KeepAlive(i32),

    #[command(code = "EE")]
    EditEvidence(u32, #[command(flatten)] EvidenceArgs),
}

#[derive(Debug, PartialEq, WithStrIter)]
pub struct EvidenceArgs {
    pub name: String,
    pub description: String,
    pub image: String,
}

fn main() {
    let (code, args) = ("HI", vec!["hdid"]);
    let expected = ClientCommand::Handshake("hdid".into());
    let actual = ClientCommand::from_protocol(code, args.into_iter()).unwrap();
    assert_eq!(actual, expected);
}
