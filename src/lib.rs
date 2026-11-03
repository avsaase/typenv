pub use from_env_derive::FromEnv;

#[path = "private.rs"]
pub mod __private;

mod error;
pub use error::*;

pub mod parser;
pub use parser::{ParseResult, Parser};

pub trait FromEnv: Sized {
    fn from_env() -> Result<Self, FromEnvErrors>;
}
