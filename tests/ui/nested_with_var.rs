use from_env::FromEnv;

#[derive(FromEnv)]
struct Inner {}

#[derive(FromEnv)]
struct Config {
    #[env(nested, var = "A")]
    inner: Inner,
}

fn main() {}
