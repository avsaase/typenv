use from_env::FromEnv;

#[derive(FromEnv)]
struct Config {
    #[env(default = "1")]
    field: Option<u32>,
}

fn main() {}
