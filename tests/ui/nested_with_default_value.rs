use typenv::FromEnv;

#[derive(FromEnv)]
struct Inner {}

#[derive(FromEnv)]
struct Config {
    #[env(nested, default_value = Inner {})]
    inner: Inner,
}

fn main() {}
