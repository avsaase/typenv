//! Controlling which environment variable a field reads.

use from_env::FromEnv;

#[test]
fn field_name_is_uppercased() {
    #[derive(FromEnv)]
    struct Config {
        attrs_db_url: String,
    }

    temp_env::with_vars(
        [("ATTRS_DB_URL", Some("upper")), ("attrs_db_url", Some("lower"))],
        || {
            assert_eq!(Config::from_env().unwrap().attrs_db_url, "upper");
        },
    );
}

#[test]
fn var_is_used_verbatim() {
    #[derive(FromEnv)]
    struct Config {
        #[env(var = "attrs_Exact_Name")]
        value: String,
    }

    temp_env::with_var("attrs_Exact_Name", Some("exact"), || {
        assert_eq!(Config::from_env().unwrap().value, "exact");
    });
}

#[test]
fn rename_is_uppercased() {
    #[derive(FromEnv)]
    struct Config {
        #[env(rename = "attrs_renamed")]
        value: String,
    }

    temp_env::with_var("ATTRS_RENAMED", Some("renamed"), || {
        assert_eq!(Config::from_env().unwrap().value, "renamed");
    });
}

#[test]
fn error_path_uses_field_name_not_var() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        #[env(var = "ATTRS_OTHER_NAME")]
        value: String,
    }

    temp_env::with_var_unset("ATTRS_OTHER_NAME", || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `value`: Missing required environment variable: `ATTRS_OTHER_NAME`\n"
        );
    });
}

#[test]
fn raw_identifier_field() {
    #[derive(FromEnv, Debug)]
    struct Config {
        r#type: String,
    }

    temp_env::with_var("TYPE", Some("raw"), || {
        assert_eq!(Config::from_env().unwrap().r#type, "raw");
    });

    temp_env::with_var_unset("TYPE", || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `type`: Missing required environment variable: `TYPE`\n"
        );
    });
}

#[test]
fn var_with_default() {
    #[derive(FromEnv)]
    struct Config {
        #[env(var = "ATTRS_VAR_DEFAULT", default = "fallback")]
        value: String,
    }

    temp_env::with_var_unset("ATTRS_VAR_DEFAULT", || {
        assert_eq!(Config::from_env().unwrap().value, "fallback");
    });
}

#[test]
fn field_named_errors_does_not_clash_with_generated_code() {
    #[derive(FromEnv)]
    struct Config {
        errors: u32,
        attrs_after_errors: u32,
    }

    temp_env::with_vars(
        [("ERRORS", Some("1")), ("ATTRS_AFTER_ERRORS", Some("2"))],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!((config.errors, config.attrs_after_errors), (1, 2));
        },
    );
}
