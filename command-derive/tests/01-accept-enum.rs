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
    Pong
}

/*
impl ::command_derive::Command for ClientRequest {
    fn ident(&self) -> &'static str {
        match self {
            ClientRequest::Handshake { .. } => "HI",
            ClientRequest::Handshake2(..) => "HEY",
            ClientRequest::Pong => "PONG"
        }
    }

    fn extract_args(&self) -> Vec<String> {
        match self {
            ClientRequest::Handshake { hdid } => vec![hdid.to_string()],
            ClientRequest::Handshake2(x0) => vec![x0.to_string()],
            ClientRequest::Pong => vec![]
        }
    }

    fn from_protocol<I>(code: &str, args: I) -> Result<Self, ::anyhow::Error> where I: Iterator<Item = String> {
        let mut args = args.map(Ok).chain(::std::iter::from_fn(|| Some(Err(::anyhow::anyhow!("Not enough args")))));

        let res = match code {
            "HI" => ClientRequest::Handshake {
                hdid: args.next().unwrap()?.parse().map_err(|e| ::anyhow::anyhow!("{}", e))?
            },
            "HEY" => ClientRequest::Handshake2(args.next().unwrap()?.parse().map_err(|e| ::anyhow::anyhow!("{}", e))?),
            "PONG" => ClientRequest::Pong,
            code => return Err(::anyhow::anyhow!("Unknown command code: {}", code))
        };
        if args.next().is_some() {
            return Err(::anyhow::anyhow!("Too much args"));
        }
        Ok(res)
    }
}
*/

fn main() {
    let (code, args) = ("HI", vec!["hdid".to_string()]);
    let expected = ClientRequest::Handshake { hdid: "hdid".into() };
    let actual = ClientRequest::from_protocol(code, args.into_iter()).unwrap();
    assert_eq!(actual, expected);

    let (code2, args2) = ("HEY", vec!["hdid2".to_string()]);
    let expected2 = ClientRequest::Handshake2("hdid2".into());
    let actual2 = ClientRequest::from_protocol(code2, args2.into_iter()).unwrap();
    assert_eq!(actual2, expected2);
}
