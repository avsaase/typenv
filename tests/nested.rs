//! `#[env(nested)]` fields.

use from_env::FromEnv;

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
