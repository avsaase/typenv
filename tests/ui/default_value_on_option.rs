use from_env::FromEnv;

#[derive(FromEnv)]
struct Config {
    #[env(default_value = Some(1))]
    field: Option<u32>,
}

fn main() {}
