use typenv::FromEnv;

#[derive(FromEnv)]
#[env(prefix = 42)]
struct Config {
    field: String,
}

fn main() {}
