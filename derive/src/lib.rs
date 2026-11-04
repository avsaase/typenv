use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod backend;
mod frontend;
mod model;

/// Derives `FromEnv`, loading a struct from environment variables.
///
/// ```
/// use typenv::FromEnv;
///
/// #[derive(FromEnv)]
/// #[env(prefix = "APP_")]
/// struct Config {
///     port: u16,                       // APP_PORT, required
///     log_level: Option<String>,       // APP_LOG_LEVEL, may be unset
///     #[env(default = "localhost")]
///     host: String,                    // APP_HOST, falls back to "localhost"
///     #[env(var = "DATABASE_URL")]
///     database_url: String,            // DATABASE_URL, prefix ignored
/// }
///
/// # temp_env::with_vars(
/// #     [
/// #         ("APP_PORT", Some("8080")),
/// #         ("DATABASE_URL", Some("postgres://localhost/app")),
/// #         ("APP_HOST", None),
/// #     ],
/// #     || {
/// let config = Config::from_env().unwrap();
/// assert_eq!(config.port, 8080);
/// assert_eq!(config.host, "localhost");
/// #     },
/// # );
/// ```
///
/// Only structs with named fields are supported.
///
/// # Variable names
///
/// A field reads the variable named after the field, uppercased: `port` reads `PORT`. The struct's
/// `prefix`, if any, is prepended verbatim. Use `rename` to change the name (it is still uppercased
/// and prefixed), or `var` to give the exact name.
///
/// # Field types
///
/// Values are parsed with [`FromStr`](std::str::FromStr) unless a `parser` is given. The parse
/// error can be any type that converts into `typenv::BoxError`, including `String` and any
/// `std::error::Error`.
///
/// A field of type `Option<T>` is optional: an unset variable gives `None`, while a value that
/// fails to parse is still an error.
///
/// # Struct attributes
///
/// - `#[env(prefix = "APP_")]`: prepended to the variable names of this struct's fields, except
///   those with `var`. Nested structs do not inherit it.
///
/// # Field attributes
///
/// - `#[env(var = "NAME")]`: read exactly this variable. Cannot be combined with `rename`.
/// - `#[env(rename = "name")]`: read this name, uppercased and prefixed, instead of the field name.
/// - `#[env(default = "...")]`: if the variable is unset, parse this string instead.
/// - `#[env(default_value = expr)]`: if the variable is unset, use this value. It is type checked
///   at compile time, only evaluated when needed, and not passed through the parser. Cannot be
///   combined with `default`.
/// - `#[env(parser = expr)]`: parse the value with this `typenv::parser::Parser`, such as a
///   function, a closure, or one of the functions in `typenv::parser`. For `Option<T>` fields it
///   parses the `T`.
/// - `#[env(nested)]`: load the field from its own `FromEnv` implementation. Cannot be combined
///   with other attributes or used on `Option` fields.
///
/// `default` and `default_value` cannot be used on `Option` fields.
///
/// ```
/// use std::time::Duration;
/// use typenv::{FromEnv, parser::ParseResult};
///
/// fn seconds(s: &str) -> ParseResult<Duration> {
///     Ok(Duration::from_secs(s.parse()?))
/// }
///
/// #[derive(FromEnv)]
/// struct Database {
///     #[env(rename = "db_url")]
///     url: String,                                  // DB_URL
///     #[env(parser = seconds, default_value = Duration::from_secs(5))]
///     timeout: Duration,                            // TIMEOUT
/// }
///
/// #[derive(FromEnv)]
/// struct Config {
///     #[env(nested)]
///     database: Database,
/// }
/// # temp_env::with_vars(
/// #     [("DB_URL", Some("postgres://localhost/app")), ("TIMEOUT", None)],
/// #     || {
/// #         let config = Config::from_env().unwrap();
/// #         assert_eq!(config.database.timeout, Duration::from_secs(5));
/// #     },
/// # );
/// ```
///
/// # Errors
///
/// Loading does not stop at the first problem: `from_env` returns a `FromEnvErrors` listing every
/// missing or invalid variable. Errors from nested structs are reported with the field path, such
/// as `database.url`.
#[proc_macro_derive(FromEnv, attributes(env))]
pub fn derive_from_env(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    frontend::parse(&ast)
        .map(|container| backend::expand(&container))
        .unwrap_or_else(|e| e.write_errors())
        .into()
}
