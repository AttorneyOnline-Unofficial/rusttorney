use command_derive::*;

#[derive(Command)]
enum ClientRequest {
    #[command(code = "HI", handle = "handle_handshake")]
    #[command(code = "HI")]
    Handshake {
        hdid: String
    }
}

fn main() {}
