//! Error types accepted from `FromStr` and `TryFrom<String>` implementations.

use std::{convert::Infallible, str::FromStr};

use from_env::{BoxError, FromEnv};

/// A `FromStr` type whose error is `E`, built from a fixed message.
macro_rules! level_with_error {
    ($name:ident, $err:ty, $make:expr) => {
        #[derive(Debug, PartialEq)]
        struct $name;

        impl FromStr for $name {
            type Err = $err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s == "ok" { Ok($name) } else { Err($make) }
            }
        }
    };
}

level_with_error!(StringErr, String, "bad level".to_owned());
level_with_error!(StrErr, &'static str, "bad level");
level_with_error!(BoxedErr, BoxError, "bad level".into());
level_with_error!(
    StdErr,
    std::io::Error,
    std::io::Error::other("bad level")
);

#[derive(Debug, PartialEq)]
struct Anything(String);

impl FromStr for Anything {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

#[derive(FromEnv, Debug)]
#[allow(dead_code)]
struct Config {
    et_string: StringErr,
    et_str: StrErr,
    et_boxed: BoxedErr,
    et_std: StdErr,
}

#[test]
fn all_error_types_parse_successfully() {
    temp_env::with_vars(
        [
            ("ET_STRING", Some("ok")),
            ("ET_STR", Some("ok")),
            ("ET_BOXED", Some("ok")),
            ("ET_STD", Some("ok")),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!(config.et_string, StringErr);
            assert_eq!(config.et_str, StrErr);
            assert_eq!(config.et_boxed, BoxedErr);
            assert_eq!(config.et_std, StdErr);
        },
    );
}

#[test]
fn all_error_types_report_their_message() {
    temp_env::with_vars(
        [
            ("ET_STRING", Some("x")),
            ("ET_STR", Some("x")),
            ("ET_BOXED", Some("x")),
            ("ET_STD", Some("x")),
        ],
        || {
            assert_eq!(
                Config::from_env().unwrap_err().to_string(),
                "Configuration errors:\n  \
                 - `et_string`: Failed to parse environment variable `ET_STRING=\"x\"`: bad level\n  \
                 - `et_str`: Failed to parse environment variable `ET_STR=\"x\"`: bad level\n  \
                 - `et_boxed`: Failed to parse environment variable `ET_BOXED=\"x\"`: bad level\n  \
                 - `et_std`: Failed to parse environment variable `ET_STD=\"x\"`: bad level\n"
            );
        },
    );
}

#[test]
fn infallible_error() {
    #[derive(FromEnv)]
    struct Config {
        et_infallible: Anything,
    }

    temp_env::with_var("ET_INFALLIBLE", Some("anything"), || {
        assert_eq!(
            Config::from_env().unwrap().et_infallible,
            Anything("anything".into())
        );
    });
}

#[test]
fn try_into_with_string_error() {
    #[derive(Debug, PartialEq)]
    struct NonEmpty(String);

    impl TryFrom<String> for NonEmpty {
        type Error = String;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            if value.is_empty() {
                Err("must not be empty".to_owned())
            } else {
                Ok(Self(value))
            }
        }
    }

    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(parser = from_env::parser::try_into)]
        et_non_empty: NonEmpty,
    }

    temp_env::with_var("ET_NON_EMPTY", Some("x"), || {
        assert_eq!(Config::from_env().unwrap().et_non_empty, NonEmpty("x".into()));
    });

    temp_env::with_var("ET_NON_EMPTY", Some(""), || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `et_non_empty`: Failed to parse environment variable `ET_NON_EMPTY=\"\"`: \
             must not be empty\n"
        );
    });
}

#[test]
fn generic_struct_bound_is_simpler() {
    #[derive(FromEnv)]
    struct Config<T>
    where
        T: FromStr,
        T::Err: Into<BoxError>,
    {
        et_generic: T,
    }

    temp_env::with_var("ET_GENERIC", Some("ok"), || {
        assert_eq!(Config::<StringErr>::from_env().unwrap().et_generic, StringErr);
    });
}
