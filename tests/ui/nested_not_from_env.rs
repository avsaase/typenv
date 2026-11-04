use typenv::FromEnv;

struct Inner;

#[derive(FromEnv)]
struct Config {
    #[env(nested)]
    inner: Inner,
}

fn main() {}
