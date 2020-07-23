use command_derive::*;

#[derive(Debug, Command, PartialEq)]
enum ClientRequest {
    /// Sent to server in the very begin
    #[command(code = "HI")]
    Handshake {
        hdid: String
    },
    #[command(code = "HEY")]
    Handshake2(String),
    /// Sent as answer to server's "Ping"
    #[command(code = "PONG")]
    Pong,
    #[command(skip)]
    #[command(code = "ANOTHER")]
    Another(#[command(flatten)] Nested)
}

#[derive(Debug, FromStrIter, PartialEq)]
struct Nested(i32, i32);

/*impl FromStrIter for Nested {
    type Error = ::anyhow::Error;

    fn from_str_iter<I, S>(mut it: I) -> Result<Nested, ::anyhow::Error>
    where
        S: AsRef<str>,
        I: Iterator<Item=S>
    {
        let on_err = || ::anyhow::anyhow!("Not enough args");
        Ok(Nested(
            it.next().ok_or_else(on_err)?.as_ref().parse().map_err(|e| ::anyhow::anyhow!("{}", e))?,
            it.next().ok_or_else(on_err)?.as_ref().parse().map_err(|e| ::anyhow::anyhow!("{}", e))?
        ))
    }
}*/

fn main() {
    assert_eq!(Nested(42, 36), Nested::from_str_iter(vec!["42", "36"].into_iter()).unwrap());
    assert!(Nested::from_str_iter(vec!["42", "kek"].into_iter()).is_err());

    let (code, args) = ("HI", vec!["hdid"]);
    let expected = ClientRequest::Handshake { hdid: "hdid".into() };
    let actual = ClientRequest::from_protocol(code, args.into_iter()).unwrap();
    assert_eq!(actual, expected);

    let (code2, args2) = ("HEY", vec!["hdid2"]);
    let expected2 = ClientRequest::Handshake2("hdid2".into());
    let actual2 = ClientRequest::from_protocol(code2, args2.into_iter()).unwrap();
    assert_eq!(actual2, expected2);
}
