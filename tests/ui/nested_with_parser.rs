use from_env::FromEnv;

#[derive(FromEnv)]
struct Inner {}

fn parse(_: &str) -> from_env::ParseResult<Inner> {
    Ok(Inner {})
}

#[derive(FromEnv)]
struct Config {
    #[env(nested, parser = parse)]
    inner: Inner,
}

fn main() {}
