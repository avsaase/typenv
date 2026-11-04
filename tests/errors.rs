//! Error accumulation and reporting.

use typenv::FromEnv;

#[test]
fn all_errors_are_reported_in_field_order() {
    #[derive(FromEnv, Debug)]
    #[allow(dead_code)]
    struct Config {
        errors_a: String,
        errors_b: u8,
        errors_c: Option<bool>,
        errors_ok: String,
    }

    temp_env::with_vars(
        [
            ("ERRORS_A", None),
            ("ERRORS_B", Some("-1")),
            ("ERRORS_C", Some("yes")),
            ("ERRORS_OK", Some("fine")),
        ],
        || {
            assert_eq!(
                Config::from_env().unwrap_err().to_string(),
                "Configuration errors:\n  \
                 - `errors_a`: Missing required environment variable: `ERRORS_A`\n  \
                 - `errors_b`: Failed to parse environment variable `ERRORS_B=\"-1\"`: \
                 invalid digit found in string\n  \
                 - `errors_c`: Failed to parse environment variable `ERRORS_C=\"yes\"`: \
                 provided string was not `true` or `false`\n"
            );
        },
    );
}

#[test]
fn errors_implement_std_error() {
    fn assert_error<E: std::error::Error + Send + Sync + 'static>() {}
    assert_error::<typenv::FromEnvErrors>();
    assert_error::<typenv::FromEnvError>();
}

#[cfg(unix)]
#[test]
fn non_unicode_value_is_a_parse_error() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        errors_not_unicode: String,
    }

    temp_env::with_var(
        "ERRORS_NOT_UNICODE",
        Some(OsStr::from_bytes(b"ab\xffcd")),
        || {
            let message = Config::from_env().unwrap_err().to_string();
            assert!(
                message.contains(
                    "`errors_not_unicode`: Failed to parse environment variable \
                     `ERRORS_NOT_UNICODE=\"ab\u{FFFD}cd\"`"
                ),
                "{message}"
            );
        },
    );
}

#[test]
fn single_error_leaves_cause_to_source() {
    use std::error::Error;

    let error = typenv::FromEnvError::ParseError {
        path: "port".into(),
        var: "PORT".into(),
        value: "abc".into(),
        source: "abc".parse::<u8>().unwrap_err().into(),
    };

    assert_eq!(
        error.to_string(),
        "`port`: Failed to parse environment variable `PORT=\"abc\"`"
    );
    assert_eq!(
        error.source().unwrap().to_string(),
        "invalid digit found in string"
    );
}

#[test]
fn error_list_has_no_source() {
    use std::error::Error;

    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        errors_source: u8,
    }

    temp_env::with_var("ERRORS_SOURCE", Some("abc"), || {
        assert!(Config::from_env().unwrap_err().source().is_none());
    });
}

#[test]
fn list_renders_the_full_cause_chain() {
    use std::{error::Error, fmt};

    #[derive(Debug)]
    struct Outer(std::num::ParseIntError);

    impl fmt::Display for Outer {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("not a valid port")
        }
    }

    impl Error for Outer {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.0)
        }
    }

    fn port(s: &str) -> typenv::parser::ParseResult<u16> {
        s.parse().map_err(|e| Outer(e).into())
    }

    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        #[env(parser = port)]
        errors_chain: u16,
    }

    temp_env::with_var("ERRORS_CHAIN", Some("x"), || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `errors_chain`: Failed to parse environment variable `ERRORS_CHAIN=\"x\"`: \
             not a valid port: invalid digit found in string\n"
        );
    });
}
