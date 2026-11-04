use typenv::FromEnv;

#[derive(FromEnv)]
struct Config {
    #[env(default = "1", default_value = 1)]
    field: u32,
}

fn main() {}
