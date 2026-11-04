use typenv::FromEnv;

#[derive(FromEnv)]
struct Config {
    #[env(prefix = "APP_")]
    field: String,
}

fn main() {}
