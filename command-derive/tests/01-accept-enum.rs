use command_derive::*;

#[derive(Command)]
enum ClientRequest {
    /// Sent to server in the very begin
    #[command(code = "HI")]
    Handshake {
        hdid: String
    },
    /// Sent as answer to server's "Ping"
    #[command(code = "PONG")]
    Pong
}

/*
impl ::command_derive::Command for ClientRequest {
    fn ident(&self) -> &'static str {
        use ClientRequest::*;

        match self {
            Handshake => "HI",
            Pong => "PONG"
        }
    }

    fn extract_args(&self) -> Vec<String> {
        use ClientRequest::*;

        match self {}
    }
}
*/

fn main() {}
