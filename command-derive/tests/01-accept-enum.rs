#![allow(dead_code)]

mod ao_message_handler;
use ao_message_handler::AO2MessageHandler;

use command_derive::*;

#[derive(Debug, Command, PartialEq)]
enum ClientRequest {
    #[command(code = "HI", handle = "handle_handshake")]
    Handshake(String),

    #[command(code = "ID")]
    #[command(handle = "handle_client_version")]
    ClientVersion(u32, String, String),

    #[command(handle = "handle_keepalive")]
    #[command(code = "CH")]
    KeepAlive(i32),

    #[command(code = "_NO", handle = "handle_flattened")]
    Flattened(#[command(flatten)] Nested),
}

#[derive(Debug, WithStrIter, PartialEq)]
pub struct Nested(i32, i32);

fn main() {
    assert_eq!(Nested(42, 36), Nested::from_str_iter(vec!["42", "36"].into_iter()).unwrap());
    assert!(Nested::from_str_iter(vec!["42", "kek"].into_iter()).is_err());

    let (code, args) = ("HI", vec!["hdid"]);
    let expected = ClientRequest::Handshake("hdid".into());
    let actual = ClientRequest::from_protocol(code, args.into_iter()).unwrap();
    assert_eq!(actual, expected);
}
