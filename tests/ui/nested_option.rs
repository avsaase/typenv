use from_env::FromEnv;

#[derive(FromEnv)]
struct Inner {}

#[derive(FromEnv)]
struct Config {
    #[env(nested)]
    inner: Option<Inner>,
}

fn main() {}
