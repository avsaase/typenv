use typenv::FromEnv;

struct NotFromStr;

#[derive(FromEnv)]
struct Config {
    field: NotFromStr,
}

fn main() {}
