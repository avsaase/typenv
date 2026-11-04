//! Read environment variables into type-safe structs.
//!
//! ## Features
//!
//! - Derive `FromEnv` and load a struct with `Config::from_env()`.
//! - Variable names are the uppercased field names, with an optional struct-level `prefix`.
//! - Support for optional values with an explicit error if parsing fails instead of silently
//!   falling back to `None`.
//! - Stringly and typed defaults.
//! - Parsing using `FromStr` by default, with support for custom parser functions, closures or
//!   type.
//! - Built-in parsers for `From<String>`, `TryFrom<String>` and JSON (behind the `json` feature).
//! - Structs can be nested with the environment variables that each struct will read clear from the
//!   struct definition.
//! - All errors reported at once instead of failing at the first.
//! - Errors include the field path, the variable name and the rejected value.
//!
//! ## Example
//!
//! ```
//! use secrecy::{ExposeSecret, SecretString};
//! use typenv::FromEnv;
//!
//! #[derive(FromEnv)]
//! #[env(prefix = "APP_")]
//! struct Config {
//!     port: u16,                 // APP_PORT
//!     log_level: Option<String>, // APP_LOG_LEVEL, may be unset
//!     #[env(nested)]
//!     database: Database,
//! }
//!
//! #[derive(Debug, FromEnv)]
//! #[env(prefix = "DB_")]
//! struct Database {
//!     url: String, // DB_URL
//!     #[env(parser = typenv::parser::into)]
//!     password: SecretString, // DB_PASSWORD, redacted in `Debug` output
//!     #[env(default_value = 5)]
//!     pool_size: u32, // DB_POOL_SIZE, defaults to 5
//! }
//!
//! # unsafe {
//! std::env::set_var("APP_PORT", "8080");
//! std::env::set_var("DB_URL", "postgres://localhost/app");
//! std::env::set_var("DB_PASSWORD", "hunter2");
//! # std::env::remove_var("APP_LOG_LEVEL");
//! # std::env::remove_var("DB_POOL_SIZE");
//! # }
//!
//! let config = Config::from_env().unwrap();
//! assert_eq!(config.port, 8080);
//! assert_eq!(config.log_level, None);
//! assert_eq!(config.database.url, "postgres://localhost/app");
//! assert_eq!(config.database.password.expose_secret(), "hunter2");
//! assert_eq!(config.database.pool_size, 5);
//! ```

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
