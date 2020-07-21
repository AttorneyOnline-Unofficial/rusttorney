use command_derive::*;

#[derive(Command)]
enum ClientRequest {
    /// Sent to server in the very begin
    #[command(code = "HI")]
    Hanshake {
        hdid: String
    },
    /// Sent as answer to server's "Ping"
    #[command(code = "PONG")]
    Pong
}

fn main() {}
