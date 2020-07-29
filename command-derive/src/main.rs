#![allow(dead_code)]

mod ao_message_handler;
use ao_message_handler::AO2MessageHandler;

use command_derive::*;

#[derive(Debug, Command, PartialEq)]
#[command(handler = "AO2MessageHandler<'a>")]
pub enum ClientCommand {
    #[command(code = "HI", handle = "handle_handshake")]
    Handshake(String),

    #[command(code = "ID", handle = "handle_client_version")]
    ClientVersion(u32, String, String),

    #[command(code = "CH", handle = "handle_keepalive")]
    KeepAlive(i32),

    #[command(code = "EE", handle = "handle_edit_evidence")]
    EditEvidence(u32, #[command(flatten)] EvidenceArgs),
}

#[derive(Debug, PartialEq, WithStrIter)]
pub struct EvidenceArgs {
    pub name: String,
    pub description: String,
    pub image: String,
}

fn main() {}
