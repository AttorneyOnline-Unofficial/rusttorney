use command_derive::*;

#[derive(Command)]
enum ClientRequest {
    #[command(code = "HI")]
    Hanshake {
        hdid: String
    },
    Ping
}

fn main() {}
