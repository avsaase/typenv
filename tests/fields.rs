//! Required, optional and defaulted fields.

use from_env::FromEnv;

#[test]
fn required_field_is_parsed() {
    #[derive(FromEnv)]
    struct Config {
        fields_port: u16,
        fields_ratio: f64,
        fields_enabled: bool,
    }

    temp_env::with_vars(
        [
            ("FIELDS_PORT", Some("8080")),
            ("FIELDS_RATIO", Some("0.5")),
            ("FIELDS_ENABLED", Some("true")),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!(config.fields_port, 8080);
            assert_eq!(config.fields_ratio, 0.5);
            assert!(config.fields_enabled);
        },
    );
}

#[test]
fn required_field_missing_is_an_error() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        fields_missing: String,
    }

    temp_env::with_var_unset("FIELDS_MISSING", || {
        let error = Config::from_env().unwrap_err();
        assert_eq!(
            error.to_string(),
            "Configuration errors:\n  \
             - `fields_missing`: Missing required environment variable: `FIELDS_MISSING`\n"
        );
    });
}

#[test]
fn empty_value_is_passed_to_the_parser() {
    #[derive(FromEnv)]
    struct Config {
        fields_empty: String,
    }

    temp_env::with_var("FIELDS_EMPTY", Some(""), || {
        assert_eq!(Config::from_env().unwrap().fields_empty, "");
    });
}

#[test]
fn optional_field_present() {
    #[derive(FromEnv)]
    struct Config {
        fields_opt_present: Option<u32>,
    }

    temp_env::with_var("FIELDS_OPT_PRESENT", Some("42"), || {
        assert_eq!(Config::from_env().unwrap().fields_opt_present, Some(42));
    });
}

#[test]
fn optional_field_absent_is_none() {
    #[derive(FromEnv)]
    struct Config {
        fields_opt_absent: Option<u32>,
    }

    temp_env::with_var_unset("FIELDS_OPT_ABSENT", || {
        assert_eq!(Config::from_env().unwrap().fields_opt_absent, None);
    });
}

#[test]
fn optional_field_invalid_is_an_error_not_none() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        fields_opt_invalid: Option<u32>,
    }

    temp_env::with_var("FIELDS_OPT_INVALID", Some("abc"), || {
        let error = Config::from_env().unwrap_err();
        assert_eq!(
            error.to_string(),
            "Configuration errors:\n  \
             - `fields_opt_invalid`: Failed to parse environment variable \
             `FIELDS_OPT_INVALID=\"abc\"`: invalid digit found in string\n"
        );
    });
}

#[test]
fn fully_qualified_option_is_treated_as_optional() {
    #[derive(FromEnv)]
    struct Config {
        fields_qualified: std::option::Option<u32>,
    }

    temp_env::with_var_unset("FIELDS_QUALIFIED", || {
        assert_eq!(Config::from_env().unwrap().fields_qualified, None);
    });
}

#[test]
fn default_used_when_absent() {
    #[derive(FromEnv)]
    struct Config {
        #[env(default = "5432")]
        fields_default_absent: u16,
    }

    temp_env::with_var_unset("FIELDS_DEFAULT_ABSENT", || {
        assert_eq!(Config::from_env().unwrap().fields_default_absent, 5432);
    });
}

#[test]
fn default_overridden_when_present() {
    #[derive(FromEnv)]
    struct Config {
        #[env(default = "5432")]
        fields_default_present: u16,
    }

    temp_env::with_var("FIELDS_DEFAULT_PRESENT", Some("1234"), || {
        assert_eq!(Config::from_env().unwrap().fields_default_present, 1234);
    });
}

#[test]
fn invalid_default_is_reported_as_invalid_default() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        #[env(default = "not a number")]
        fields_bad_default: u16,
    }

    temp_env::with_var_unset("FIELDS_BAD_DEFAULT", || {
        let error = Config::from_env().unwrap_err();
        assert_eq!(
            error.to_string(),
            "Configuration errors:\n  \
             - `fields_bad_default`: Invalid default value \"not a number\" for \
             `FIELDS_BAD_DEFAULT` (variable is not set): invalid digit found in string\n"
        );
    });
}

#[test]
fn invalid_default_is_ignored_when_var_is_set() {
    #[derive(FromEnv)]
    struct Config {
        #[env(default = "not a number")]
        fields_bad_default_set: u16,
    }

    temp_env::with_var("FIELDS_BAD_DEFAULT_SET", Some("1"), || {
        assert_eq!(Config::from_env().unwrap().fields_bad_default_set, 1);
    });
}

#[test]
fn invalid_env_value_with_default_is_a_parse_error() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        #[env(default = "1")]
        fields_default_bad_value: u16,
    }

    temp_env::with_var("FIELDS_DEFAULT_BAD_VALUE", Some("abc"), || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `fields_default_bad_value`: Failed to parse environment variable \
             `FIELDS_DEFAULT_BAD_VALUE=\"abc\"`: invalid digit found in string\n"
        );
    });
}

#[test]
fn empty_struct() {
    #[derive(FromEnv)]
    struct Empty {}

    assert!(Empty::from_env().is_ok());
}

#[test]
fn generic_struct() {
    #[derive(FromEnv)]
    struct Config<T: std::str::FromStr>
    where
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        fields_generic: T,
    }

    temp_env::with_var("FIELDS_GENERIC", Some("7"), || {
        assert_eq!(Config::<i64>::from_env().unwrap().fields_generic, 7);
    });
}
