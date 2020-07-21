use command_derive::*;

#[derive(Command)]
enum ClientRequest {
    #[command(code = "HI")]
    Handshake {
        hdid: String
    },
    Ping
}

fn main() {}
