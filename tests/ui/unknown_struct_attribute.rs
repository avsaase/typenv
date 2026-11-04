use from_env::FromEnv;

#[derive(FromEnv)]
#[env(unknown = "x")]
struct Config {
    field: String,
}

fn main() {}
