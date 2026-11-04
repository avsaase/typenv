use typenv::FromEnv;

#[derive(FromEnv)]
enum Config {
    A,
}

fn main() {}
