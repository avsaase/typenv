use typenv::FromEnv;

#[derive(FromEnv)]
struct Config<T> {
    field: T,
}

fn main() {}
