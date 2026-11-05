//! The default parser: `FromStr`, falling back to `TryFrom<String>` (and so `From<String>`).

use std::str::FromStr;

use typenv::FromEnv;

/// Only `From<String>`.
#[derive(Debug, PartialEq)]
struct Name(String);

impl From<String> for Name {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Only `TryFrom<String>`.
#[derive(Debug, PartialEq)]
struct NonEmpty(String);

impl TryFrom<String> for NonEmpty {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err("must not be empty")
        } else {
            Ok(Self(value))
        }
    }
}

/// Both `FromStr` and `From<String>` (so `TryFrom<String>`), with different results.
#[derive(Debug, PartialEq)]
enum Source {
    FromStr,
    FromString,
}

impl FromStr for Source {
    type Err = std::convert::Infallible;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Source::FromStr)
    }
}

impl From<String> for Source {
    fn from(_: String) -> Self {
        Source::FromString
    }
}

#[test]
fn from_string_without_parser() {
    #[derive(FromEnv)]
    struct Config {
        dp_name: Name,
    }

    temp_env::with_var("DP_NAME", Some("app"), || {
        assert_eq!(Config::from_env().unwrap().dp_name, Name("app".to_owned()));
    });
}

#[test]
fn try_from_string_without_parser() {
    #[derive(FromEnv, Debug)]
    struct Config {
        dp_non_empty: NonEmpty,
    }

    temp_env::with_var("DP_NON_EMPTY", Some("x"), || {
        assert_eq!(
            Config::from_env().unwrap().dp_non_empty,
            NonEmpty("x".to_owned())
        );
    });

    temp_env::with_var("DP_NON_EMPTY", Some(""), || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `dp_non_empty`: Failed to parse environment variable `DP_NON_EMPTY=\"\"`: \
             must not be empty\n"
        );
    });
}

#[test]
fn from_str_takes_precedence() {
    #[derive(FromEnv)]
    struct Config {
        dp_source: Source,
    }

    temp_env::with_var("DP_SOURCE", Some("x"), || {
        assert_eq!(Config::from_env().unwrap().dp_source, Source::FromStr);
    });
}

#[test]
fn optional_and_defaulted_fields() {
    #[derive(FromEnv)]
    struct Config {
        dp_opt_name: Option<Name>,
        #[env(default = "fallback")]
        dp_default_non_empty: NonEmpty,
    }

    temp_env::with_vars(
        [("DP_OPT_NAME", Some("opt")), ("DP_DEFAULT_NON_EMPTY", None)],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!(config.dp_opt_name, Some(Name("opt".to_owned())));
            assert_eq!(config.dp_default_non_empty, NonEmpty("fallback".to_owned()));
        },
    );
}

#[test]
fn generic_field_uses_declared_bounds() {
    #[derive(FromEnv)]
    struct Config<T>
    where
        T: TryFrom<String>,
        T::Error: Into<typenv::BoxError>,
    {
        dp_generic: T,
    }

    temp_env::with_var("DP_GENERIC", Some("generic"), || {
        assert_eq!(
            Config::<Name>::from_env().unwrap().dp_generic,
            Name("generic".to_owned())
        );
    });
}
