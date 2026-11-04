use from_env::FromEnv;

#[derive(FromEnv)]
struct Config {
    #[env(prefix = "APP_")]
    field: String,
}

fn main() {}
