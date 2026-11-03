//! Error accumulation and reporting.

use from_env::FromEnv;

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
    assert_error::<from_env::FromEnvErrors>();
    assert_error::<from_env::FromEnvError>();
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
