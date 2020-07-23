use command_derive::*;

#[derive(Debug, Command, PartialEq)]
enum ClientRequest {
    /// Sent to server in the very begin
    #[command(code = "HI")]
    Handshake { hdid: String },
    #[command(code = "HEY")]
    Handshake2(String),
    /// Sent as answer to server's "Ping"
    #[command(code = "PONG")]
    Pong,
}

#[derive(Debug, FromStrIter, IntoStrIter, PartialEq)]
struct Nested(i32, i32);

fn main() {}
