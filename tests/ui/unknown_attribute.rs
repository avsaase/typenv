use from_env::FromEnv;

#[derive(FromEnv)]
struct Config {
    #[env(unknown = "x")]
    field: String,
}

fn main() {}
