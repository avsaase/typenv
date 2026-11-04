# typenv

[![crates.io](https://img.shields.io/crates/v/typenv.svg)](https://crates.io/crates/typenv)
[![docs.rs](https://img.shields.io/docsrs/typenv)](https://docs.rs/typenv)
[![CI](https://github.com/avsaase/typenv/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/avsaase/typenv/actions/workflows/ci.yml)

<!-- cargo-reedme: start -->

<!-- cargo-reedme: info-start

    Do not edit this region by hand
    ===============================

    This region was generated from Rust documentation comments by `cargo-reedme` using this command:

        cargo +nightly reedme

    for more info: https://github.com/nik-rev/cargo-reedme

cargo-reedme: info-end -->

Read environment variables into type-safe structs.

### Features

- Derive `FromEnv` and load a struct with `Config::from_env()`.
- Variable names are the uppercased field names, with an optional struct-level `prefix`.
- Support for optional values with an explicit error if parsing fails instead of silently
  falling back to `None`.
- Stringly and typed defaults.
- Parsing using `FromStr` by default, with support for custom parser functions, closures or
  type.
- Built-in parsers for `From<String>`, `TryFrom<String>`, serde (`serde` feature) and JSON
  (`json` feature).
- Structs can be nested with the environment variables that each struct will read clear from the
  struct definition.
- All errors reported at once instead of failing at the first.
- Errors include the field path, the variable name and the rejected value.
- A generated `ENV_VARS` const that lists every variable a struct reads.

### Example

```rust
use secrecy::{ExposeSecret, SecretString};
use typenv::FromEnv;

#[derive(FromEnv)]
#[env(prefix = "APP_")]
struct Config {
    port: u16,                 // APP_PORT
    log_level: Option<String>, // APP_LOG_LEVEL, may be unset
    #[env(nested)]
    database: Database,
}

#[derive(Debug, FromEnv)]
#[env(prefix = "DB_")]
struct Database {
    url: String, // DB_URL
    #[env(parser = typenv::parser::into)]
    password: SecretString, // DB_PASSWORD, redacted in `Debug` output
    #[env(default_value = 5)]
    pool_size: u32, // DB_POOL_SIZE, defaults to 5
}

std::env::set_var("APP_PORT", "8080");
std::env::set_var("DB_URL", "postgres://localhost/app");
std::env::set_var("DB_PASSWORD", "hunter2");

let config = Config::from_env().unwrap();
assert_eq!(config.port, 8080);
assert_eq!(config.log_level, None);
assert_eq!(config.database.url, "postgres://localhost/app");
assert_eq!(config.database.password.expose_secret(), "hunter2");
assert_eq!(config.database.pool_size, 5);
```

<!-- cargo-reedme: end -->
