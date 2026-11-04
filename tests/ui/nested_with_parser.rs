use typenv::FromEnv;

#[derive(FromEnv)]
struct Inner {}

fn parse(_: &str) -> typenv::ParseResult<Inner> {
    Ok(Inner {})
}

#[derive(FromEnv)]
struct Config {
    #[env(nested, parser = parse)]
    inner: Inner,
}

fn main() {}
