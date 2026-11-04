use typenv::FromEnv;

#[derive(FromEnv)]
struct Config {
    #[env(default_value = "8080")]
    port: u16,
}

fn main() {}
