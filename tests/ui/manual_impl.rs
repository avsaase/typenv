use typenv::{FromEnv, FromEnvErrors};

struct Config;

impl FromEnv for Config {
    const ENV_VARS: &'static str = "";
    const __NESTED_ENV_VARS: &'static str = "";

    fn from_env() -> Result<Self, FromEnvErrors> {
        Ok(Config)
    }
}

fn main() {}
