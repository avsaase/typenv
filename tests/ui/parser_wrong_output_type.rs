use from_env::FromEnv;

fn parse(s: &str) -> from_env::ParseResult<String> {
    Ok(s.to_owned())
}

#[derive(FromEnv)]
struct Config {
    #[env(parser = parse)]
    field: u32,
}

fn main() {}
