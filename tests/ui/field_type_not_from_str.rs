use from_env::FromEnv;

struct NotFromStr;

#[derive(FromEnv)]
struct Config {
    field: NotFromStr,
}

fn main() {}
