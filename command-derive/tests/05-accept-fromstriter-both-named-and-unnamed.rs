use command_derive::{FromStrIter, IntoStrIter};

#[derive(FromStrIter, IntoStrIter)]
struct Unnamed(i32, i32);

#[derive(FromStrIter, IntoStrIter)]
struct Named {
    x: f64,
    y: f64,
    named: String
}

fn main() {}
