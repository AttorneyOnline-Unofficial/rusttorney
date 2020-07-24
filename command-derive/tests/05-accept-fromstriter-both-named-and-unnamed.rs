use command_derive::WithStrIter;

#[derive(WithStrIter)]
struct Unnamed(i32, i32);

#[derive(WithStrIter)]
struct Named {
    x: f64,
    y: f64,
    named: String
}

fn main() {}
