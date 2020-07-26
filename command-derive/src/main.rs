#![allow(dead_code)]

mod ao_message_handler;
use ao_message_handler::AO2MessageHandler;

use command_derive::*;

#[derive(Debug, Command, PartialEq)]
enum ClientRequest {
    #[command(code = "HI", handle = "handle_handshake")]
    Handshake(String),
    #[command(code = "ID", handle = "handle_client_version")]
    ClientVersion(u32, String, String),
    #[command(code = "CH", handle = "handle_keepalive")]
    KeepAlive(i32),
}

#[derive(Debug, WithStrIter, PartialEq)]
pub struct Nested(i32, i32);

fn main() {}
