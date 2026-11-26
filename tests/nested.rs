//! `#[env(nested)]` fields.

use typenv::FromEnv;

#[derive(FromEnv, Debug)]
struct Database {
    nested_host: String,
    nested_port: u16,
}

#[derive(FromEnv, Debug)]
struct Config {
    nested_name: String,
    #[env(nested)]
    database: Database,
}

#[test]
fn nested_struct_is_loaded() {
    temp_env::with_vars(
        [
            ("NESTED_NAME", Some("app")),
            ("NESTED_HOST", Some("localhost")),
            ("NESTED_PORT", Some("5432")),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!(config.nested_name, "app");
            assert_eq!(config.database.nested_host, "localhost");
            assert_eq!(config.database.nested_port, 5432);
        },
    );
}

#[test]
fn nested_errors_are_prefixed_and_all_reported() {
    temp_env::with_vars(
        [
            ("NESTED_NAME", None),
            ("NESTED_HOST", None),
            ("NESTED_PORT", Some("99999")),
        ],
        || {
            assert_eq!(
                Config::from_env().unwrap_err().to_string(),
                "Configuration errors:\n  \
                 - `nested_name`: Missing required environment variable: `NESTED_NAME`\n  \
                 - `database.nested_host`: Missing required environment variable: `NESTED_HOST`\n  \
                 - `database.nested_port`: Failed to parse environment variable \
                 `NESTED_PORT=\"99999\"`: number too large to fit in target type\n"
            );
        },
    );
}

#[test]
fn deeply_nested_paths() {
    #[derive(FromEnv, Debug)]
    #[allow(dead_code)]
    struct Outer {
        #[env(nested)]
        middle: Middle,
    }

    #[derive(FromEnv, Debug)]
    #[allow(dead_code)]
    struct Middle {
        #[env(nested)]
        inner: Inner,
    }

    #[derive(FromEnv, Debug)]
    struct Inner {
        #[allow(dead_code)]
        nested_deep: String,
    }

    temp_env::with_var_unset("NESTED_DEEP", || {
        assert_eq!(
            Outer::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `middle.inner.nested_deep`: Missing required environment variable: `NESTED_DEEP`\n"
        );
    });
}

#[test]
#[allow(dead_code)]
fn optional_nested_absent_is_none() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(nested)]
        absent_database: Option<AbsentDatabase>,
    }

    #[derive(FromEnv, Debug)]
    struct AbsentDatabase {
        absent_host: String,
        absent_port: u16,
    }

    temp_env::with_vars(
        [("ABSENT_HOST", None::<&str>), ("ABSENT_PORT", None::<&str>)],
        || {
            assert!(Config::from_env().unwrap().absent_database.is_none());
        },
    );
}

#[test]
#[allow(dead_code)]
fn optional_nested_present_is_loaded() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(nested)]
        present_database: Option<PresentDatabase>,
    }

    #[derive(FromEnv, Debug)]
    struct PresentDatabase {
        present_host: String,
        present_port: u16,
    }

    temp_env::with_vars(
        [
            ("PRESENT_HOST", Some("localhost")),
            ("PRESENT_PORT", Some("5432")),
        ],
        || {
            let database = Config::from_env().unwrap().present_database.unwrap();
            assert_eq!(database.present_host, "localhost");
            assert_eq!(database.present_port, 5432);
        },
    );
}

#[test]
#[allow(dead_code)]
fn partially_present_optional_nested_is_an_error() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(nested)]
        partial_database: Option<PartialDatabase>,
    }

    #[derive(FromEnv, Debug)]
    struct PartialDatabase {
        partial_host: String,
        partial_port: u16,
    }

    temp_env::with_vars(
        [("PARTIAL_HOST", Some("localhost")), ("PARTIAL_PORT", None)],
        || {
            assert_eq!(
                Config::from_env().unwrap_err().to_string(),
                "Configuration errors:\n  \
                 - `partial_database.partial_port`: Missing required environment variable: \
                 `PARTIAL_PORT`\n"
            );
        },
    );
}

#[test]
#[allow(dead_code)]
fn present_optional_nested_parse_error_is_an_error() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(nested)]
        invalid_database: Option<InvalidDatabase>,
    }

    #[derive(FromEnv, Debug)]
    struct InvalidDatabase {
        invalid_port: u16,
    }

    temp_env::with_var("INVALID_PORT", Some("99999"), || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `invalid_database.invalid_port`: Failed to parse environment variable \
             `INVALID_PORT=\"99999\"`: number too large to fit in target type\n"
        );
    });
}

#[test]
#[allow(dead_code)]
fn optional_child_of_optional_nested_group() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(nested)]
        grouped: Option<Grouped>,
    }

    #[derive(FromEnv, Debug)]
    struct Grouped {
        grouped_name: String,
        #[env(nested)]
        grouped_extra: Option<Extra>,
    }

    #[derive(FromEnv, Debug)]
    struct Extra {
        grouped_extra_value: u16,
    }

    temp_env::with_vars(
        [("GROUPED_NAME", Some("app")), ("GROUPED_EXTRA_VALUE", None)],
        || {
            let grouped = Config::from_env().unwrap().grouped.unwrap();
            assert_eq!(grouped.grouped_name, "app");
            assert!(grouped.grouped_extra.is_none());
        },
    );
}

#[test]
#[allow(dead_code)]
fn grandchild_variable_makes_optional_group_present() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(nested)]
        recursive_group: Option<RecursiveGroup>,
    }

    #[derive(FromEnv, Debug)]
    struct RecursiveGroup {
        recursive_name: String,
        #[env(nested)]
        recursive_extra: Option<RecursiveExtra>,
    }

    #[derive(FromEnv, Debug)]
    struct RecursiveExtra {
        recursive_extra_value: u16,
    }

    temp_env::with_vars(
        [
            ("RECURSIVE_NAME", None),
            ("RECURSIVE_EXTRA_VALUE", Some("1")),
        ],
        || {
            assert_eq!(
                Config::from_env().unwrap_err().to_string(),
                "Configuration errors:\n  \
                 - `recursive_group.recursive_name`: Missing required environment variable: \
                 `RECURSIVE_NAME`\n"
            );
        },
    );
}

#[cfg(unix)]
#[test]
#[allow(dead_code)]
fn non_unicode_variable_makes_optional_group_present() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(nested)]
        unicode_group: Option<UnicodeGroup>,
    }

    #[derive(FromEnv, Debug)]
    struct UnicodeGroup {
        unicode_value: String,
    }

    temp_env::with_var(
        "UNICODE_VALUE",
        Some(OsStr::from_bytes(b"ab\xffcd")),
        || {
            let message = Config::from_env().unwrap_err().to_string();
            assert!(
                message.contains(
                    "`unicode_group.unicode_value`: Failed to parse environment variable \
                     `UNICODE_VALUE=\"ab\u{FFFD}cd\"`"
                ),
                "{message}"
            );
        },
    );
}

#[test]
fn nested_invalid_default_is_prefixed() {
    #[derive(FromEnv, Debug)]
    struct Outer {
        #[allow(dead_code)]
        #[env(nested)]
        inner: Inner,
    }

    #[derive(FromEnv, Debug)]
    struct Inner {
        #[allow(dead_code)]
        #[env(default = "x")]
        nested_bad_default: u8,
    }

    temp_env::with_var_unset("NESTED_BAD_DEFAULT", || {
        assert_eq!(
            Outer::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `inner.nested_bad_default`: Invalid default value \"x\" for \
             `NESTED_BAD_DEFAULT` (variable is not set): invalid digit found in string\n"
        );
    });
}
