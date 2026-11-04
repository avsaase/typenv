//! `#[env(default_value = expr)]`: typed defaults.

use std::time::Duration;

use typenv::{FromEnv, parser::ParseResult};

#[test]
fn used_when_absent() {
    #[derive(FromEnv)]
    struct Config {
        #[env(default_value = 8080)]
        dv_port: u16,
    }

    temp_env::with_var_unset("DV_PORT", || {
        assert_eq!(Config::from_env().unwrap().dv_port, 8080);
    });
}

#[test]
fn env_value_takes_precedence() {
    #[derive(FromEnv)]
    struct Config {
        #[env(default_value = 8080)]
        dv_port_set: u16,
    }

    temp_env::with_var("DV_PORT_SET", Some("9090"), || {
        assert_eq!(Config::from_env().unwrap().dv_port_set, 9090);
    });
}

#[test]
fn invalid_env_value_is_still_a_parse_error() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        #[env(default_value = 8080)]
        dv_port_bad: u16,
    }

    temp_env::with_var("DV_PORT_BAD", Some("abc"), || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `dv_port_bad`: Failed to parse environment variable \
             `DV_PORT_BAD=\"abc\"`: invalid digit found in string\n"
        );
    });
}

#[test]
fn arbitrary_expressions() {
    const DEFAULT_RETRIES: u32 = 3;

    #[derive(FromEnv)]
    struct Config {
        #[env(default_value = DEFAULT_RETRIES)]
        dv_retries: u32,
        #[env(default_value = String::from("localhost"))]
        dv_host: String,
        #[env(default_value = vec!["a".to_owned(), "b".to_owned()], parser = |s: &str| Ok(vec![s.to_owned()]))]
        dv_list: Vec<String>,
        #[env(default_value = DEFAULT_RETRIES * 2 + 1)]
        dv_computed: u32,
    }

    temp_env::with_vars(
        [
            ("DV_RETRIES", None::<&str>),
            ("DV_HOST", None),
            ("DV_LIST", None),
            ("DV_COMPUTED", None),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!(config.dv_retries, 3);
            assert_eq!(config.dv_host, "localhost");
            assert_eq!(config.dv_list, ["a", "b"]);
            assert_eq!(config.dv_computed, 7);
        },
    );
}

#[test]
fn string_literal_is_kept_as_a_literal() {
    #[derive(FromEnv)]
    struct Config {
        #[env(default_value = "localhost", parser = |s: &str| Ok(s.len().to_string().leak() as &str))]
        dv_literal: &'static str,
    }

    temp_env::with_var_unset("DV_LITERAL", || {
        assert_eq!(Config::from_env().unwrap().dv_literal, "localhost");
    });
}

#[test]
fn default_is_not_passed_through_the_parser() {
    fn seconds(s: &str) -> ParseResult<Duration> {
        Ok(Duration::from_secs(s.parse()?))
    }

    #[derive(FromEnv)]
    struct Config {
        #[env(parser = seconds, default_value = Duration::from_millis(1500))]
        dv_timeout: Duration,
    }

    temp_env::with_var_unset("DV_TIMEOUT", || {
        assert_eq!(
            Config::from_env().unwrap().dv_timeout,
            Duration::from_millis(1500)
        );
    });

    temp_env::with_var("DV_TIMEOUT", Some("2"), || {
        assert_eq!(
            Config::from_env().unwrap().dv_timeout,
            Duration::from_secs(2)
        );
    });
}

#[test]
fn default_is_only_evaluated_when_needed() {
    #[derive(FromEnv)]
    struct Config {
        #[env(default_value = panic!("default must not be evaluated"))]
        dv_lazy: u8,
    }

    temp_env::with_var("DV_LAZY", Some("1"), || {
        assert_eq!(Config::from_env().unwrap().dv_lazy, 1);
    });
}

#[test]
fn works_with_prefix_and_rename() {
    #[derive(FromEnv)]
    #[env(prefix = "DV_APP_")]
    struct Config {
        #[env(rename = "listen_port", default_value = 1)]
        port: u16,
    }

    temp_env::with_var("DV_APP_LISTEN_PORT", Some("2"), || {
        assert_eq!(Config::from_env().unwrap().port, 2);
    });

    temp_env::with_var_unset("DV_APP_LISTEN_PORT", || {
        assert_eq!(Config::from_env().unwrap().port, 1);
    });
}

#[test]
fn generic_field() {
    #[derive(FromEnv)]
    struct Config<T>
    where
        T: std::str::FromStr + Default,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        #[env(default_value = T::default())]
        dv_generic: T,
    }

    temp_env::with_var_unset("DV_GENERIC", || {
        assert_eq!(Config::<u64>::from_env().unwrap().dv_generic, 0);
    });
}
