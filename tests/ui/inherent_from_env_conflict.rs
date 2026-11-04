use from_env::FromEnv;

#[derive(FromEnv)]
struct Config {
    field: String,
}

impl Config {
    fn from_env() -> Self {
        Config { field: String::new() }
    }
}

fn main() {}
