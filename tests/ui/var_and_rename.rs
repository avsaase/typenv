use from_env::FromEnv;

#[derive(FromEnv)]
struct Config {
    #[env(var = "A", rename = "b")]
    field: String,
}

fn main() {}
