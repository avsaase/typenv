use typenv::FromEnv;

#[derive(FromEnv)]
struct Config {
    #[env(parser = 42)]
    field: u32,
}

fn main() {}
