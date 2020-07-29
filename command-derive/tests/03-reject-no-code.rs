use command_derive::*;

#[derive(Command)]
enum ClientRequest {
    #[command(code = "HI", handle = "handle_handshake")]
    Handshake {
        hdid: String
    },
    Ping
}

fn main() {}
