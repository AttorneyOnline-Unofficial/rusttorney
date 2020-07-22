/*use command_derive::*;

#[derive(Debug, Command, PartialEq)]
enum ClientRequest {
    /// Sent to server in the very begin
    #[command(code = "HI")]
    Handshake { hdid: String },
    #[command(code = "HEY")]
    Handshake2(String),
    /// Sent as answer to server's "Ping"
    #[command(code = "PONG")]
    Pong,
}

fn main() {
    let (code, args) = ("HI", vec!["hdid".to_string()]);
    let expected = ClientRequest::Handshake { hdid: "hdid".into() };
    let actual = ClientRequest::from_protocol(code, args.into_iter()).unwrap();
    assert_eq!(actual, expected);

    let (code2, args2) = ("HEY", vec!["hdid2".to_string()]);
    let expected2 = ClientRequest::Handshake2("hdid2".into());
    let actual2 = ClientRequest::from_protocol(code2, args2.into_iter()).unwrap();
    assert_eq!(actual2, expected2);
}*/

fn main() {}
