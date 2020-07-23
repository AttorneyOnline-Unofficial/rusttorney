use command_derive::FromStrIter;

#[derive(FromStrIter)]
struct Unnamed(i32, i32);

#[derive(FromStrIter)]
struct Named {
    x: f64,
    y: f64,
    named: String
}

fn main() {}
