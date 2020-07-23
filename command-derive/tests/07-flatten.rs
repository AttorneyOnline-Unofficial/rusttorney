use command_derive::*;

#[derive(Debug, Command, PartialEq)]
enum ClientRequest {
    /// Sent to server in the very begin
    #[command(code = "HI")]
    Handshake {
        hdid: String
    },
    /// Sent as answer to server's "Ping"
    #[command(code = "PONG")]
    Pong,
    #[command(code = "NEW")]
    NewChar(#[command(flatten)] Char),
    #[command(code = "EDIT")]
    EditChar {
        char_id: i32,
        #[command(flatten)]
        character: Char
    }
}

#[derive(Debug, WithStrIter, PartialEq)]
struct Nested(i32, i32);

#[derive(Debug, WithStrIter, PartialEq)]
struct Char {
    name: String,
    surname: String
}

fn main() {
    assert_eq!(Nested(42, 36), Nested::from_str_iter(vec!["42", "36"].into_iter()).unwrap());
    assert!(Nested::from_str_iter(vec!["42", "kek"].into_iter()).is_err());

    let (code, args) = ("HI", vec!["hdid"]);
    let expected = ClientRequest::Handshake { hdid: "hdid".into() };
    let actual = ClientRequest::from_protocol(code, args.into_iter()).unwrap();
    assert_eq!(actual, expected);

    assert_eq!(
        ClientRequest::NewChar(Char {
            name: "Yes".to_string(),
            surname: "No".to_string()
        }).extract_args(),
        vec!["Yes".to_string(), "No".to_string()]
    );
    assert_eq!(
        ClientRequest::EditChar {
            char_id: -1,
            character: Char {
                name: "Yes".to_string(),
                surname: "No".to_string()
            }
        }.extract_args(),
        vec!["-1".to_string(), "Yes".to_string(), "No".to_string()]
    );
}
