pub use typenv_derive::FromEnv;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

mod error;
pub use error::*;

pub mod parser;

pub trait FromEnv: Sized {
    fn from_env() -> Result<Self, FromEnvErrors>;
}
