use typenv::FromEnv;

fn parse(s: &str) -> typenv::parser::ParseResult<String> {
    Ok(s.to_owned())
}

#[derive(FromEnv)]
struct Config {
    #[env(parser = parse)]
    field: u32,
}

fn main() {}
