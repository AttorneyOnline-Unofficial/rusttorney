use command_derive::*;

#[derive(Command)]
enum Enum {
    #[command(code = 123)]
    Variant
}

fn main() {}
