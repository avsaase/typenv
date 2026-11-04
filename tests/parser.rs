//! `#[env(parser = ...)]`.

use typenv::{
    FromEnv,
    parser::{ParseResult, Parser},
};

fn comma_list(s: &str) -> ParseResult<Vec<String>> {
    Ok(s.split(',').map(|item| item.trim().to_owned()).collect())
}

#[test]
fn function_path() {
    #[derive(FromEnv)]
    struct Config {
        #[env(parser = comma_list)]
        parser_hosts: Vec<String>,
    }

    temp_env::with_var("PARSER_HOSTS", Some("a, b,c"), || {
        assert_eq!(Config::from_env().unwrap().parser_hosts, ["a", "b", "c"]);
    });
}

#[test]
fn closure() {
    #[derive(FromEnv)]
    struct Config {
        #[env(parser = |s: &str| Ok(s.len()))]
        parser_closure: usize,
    }

    temp_env::with_var("PARSER_CLOSURE", Some("four"), || {
        assert_eq!(Config::from_env().unwrap().parser_closure, 4);
    });
}

#[test]
fn custom_parser_type() {
    struct Scaled(u64);

    impl Parser<u64> for Scaled {
        fn parse(&self, s: &str) -> ParseResult<u64> {
            Ok(s.parse::<u64>()? * self.0)
        }
    }

    #[derive(FromEnv)]
    struct Config {
        #[env(parser = Scaled(1000))]
        parser_timeout_ms: u64,
    }

    temp_env::with_var("PARSER_TIMEOUT_MS", Some("3"), || {
        assert_eq!(Config::from_env().unwrap().parser_timeout_ms, 3000);
    });
}

#[test]
fn builtin_into() {
    #[derive(Debug, PartialEq)]
    struct Name(String);

    impl From<String> for Name {
        fn from(value: String) -> Self {
            Self(value)
        }
    }

    #[derive(FromEnv)]
    struct Config {
        #[env(parser = typenv::parser::into)]
        parser_name: Name,
    }

    temp_env::with_var("PARSER_NAME", Some("app"), || {
        assert_eq!(Config::from_env().unwrap().parser_name, Name("app".into()));
    });
}

#[test]
fn builtin_try_into() {
    #[derive(Debug, PartialEq)]
    struct NonEmpty(String);

    impl TryFrom<String> for NonEmpty {
        type Error = std::io::Error;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            if value.is_empty() {
                Err(std::io::Error::other("value is empty"))
            } else {
                Ok(Self(value))
            }
        }
    }

    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(parser = typenv::parser::try_into)]
        parser_non_empty: NonEmpty,
    }

    temp_env::with_var("PARSER_NON_EMPTY", Some("x"), || {
        assert_eq!(
            Config::from_env().unwrap().parser_non_empty,
            NonEmpty("x".into())
        );
    });

    temp_env::with_var("PARSER_NON_EMPTY", Some(""), || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `parser_non_empty`: Failed to parse environment variable \
             `PARSER_NON_EMPTY=\"\"`: value is empty\n"
        );
    });
}

#[cfg(feature = "json")]
#[test]
fn builtin_json() {
    #[derive(FromEnv, Debug)]
    struct Config {
        #[env(parser = typenv::parser::json)]
        parser_ports: Vec<u16>,
    }

    temp_env::with_var("PARSER_PORTS", Some("[80, 443]"), || {
        assert_eq!(Config::from_env().unwrap().parser_ports, [80, 443]);
    });

    temp_env::with_var("PARSER_PORTS", Some("[80,"), || {
        let message = Config::from_env().unwrap_err().to_string();
        assert!(
            message.contains("`parser_ports`: Failed to parse environment variable"),
            "{message}"
        );
    });
}

#[test]
fn optional_field_uses_parser_for_inner_type() {
    #[derive(FromEnv)]
    struct Config {
        #[env(parser = comma_list)]
        parser_opt_list: Option<Vec<String>>,
    }

    temp_env::with_var("PARSER_OPT_LIST", Some("x,y"), || {
        assert_eq!(
            Config::from_env().unwrap().parser_opt_list,
            Some(vec!["x".to_owned(), "y".to_owned()])
        );
    });

    temp_env::with_var_unset("PARSER_OPT_LIST", || {
        assert_eq!(Config::from_env().unwrap().parser_opt_list, None);
    });
}

#[test]
fn default_is_parsed_with_custom_parser() {
    #[derive(FromEnv)]
    struct Config {
        #[env(parser = comma_list, default = "one,two")]
        parser_default_list: Vec<String>,
    }

    temp_env::with_var_unset("PARSER_DEFAULT_LIST", || {
        assert_eq!(
            Config::from_env().unwrap().parser_default_list,
            ["one", "two"]
        );
    });
}

#[test]
fn parser_errors_are_reported() {
    fn even(s: &str) -> ParseResult<u32> {
        let n: u32 = s.parse()?;
        if n.is_multiple_of(2) {
            Ok(n)
        } else {
            Err(format!("{n} is not even").into())
        }
    }

    #[derive(FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        #[env(parser = even)]
        parser_even: u32,
    }

    temp_env::with_var("PARSER_EVEN", Some("3"), || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `parser_even`: Failed to parse environment variable \
             `PARSER_EVEN=\"3\"`: 3 is not even\n"
        );
    });
}

#[test]
fn field_type_without_from_str_is_fine_with_parser() {
    struct NoFromStr(String);

    #[derive(FromEnv)]
    struct Config {
        #[env(parser = |s: &str| Ok(NoFromStr(s.to_owned())))]
        parser_no_from_str: NoFromStr,
    }

    temp_env::with_var("PARSER_NO_FROM_STR", Some("ok"), || {
        assert_eq!(Config::from_env().unwrap().parser_no_from_str.0, "ok");
    });
}

#[cfg(feature = "serde")]
mod deserialize {
    use serde::Deserialize;
    use typenv::FromEnv;

    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum LogLevel {
        Debug,
        Info,
        Warn,
    }

    #[test]
    fn enum_with_rename_all() {
        #[derive(FromEnv)]
        struct Config {
            #[env(parser = typenv::parser::deserialize)]
            de_level: LogLevel,
        }

        temp_env::with_var("DE_LEVEL", Some("warn"), || {
            assert_eq!(Config::from_env().unwrap().de_level, LogLevel::Warn);
        });
    }

    #[test]
    fn optional_field_and_default() {
        #[derive(FromEnv)]
        struct Config {
            #[env(parser = typenv::parser::deserialize)]
            de_opt_level: Option<LogLevel>,
            #[env(parser = typenv::parser::deserialize, default = "info")]
            de_default_level: LogLevel,
        }

        temp_env::with_vars(
            [("DE_OPT_LEVEL", Some("debug")), ("DE_DEFAULT_LEVEL", None)],
            || {
                let config = Config::from_env().unwrap();
                assert_eq!(config.de_opt_level, Some(LogLevel::Debug));
                assert_eq!(config.de_default_level, LogLevel::Info);
            },
        );
    }

    #[test]
    fn unknown_variant_is_reported() {
        #[derive(FromEnv, Debug)]
        struct Config {
            #[allow(dead_code)]
            #[env(parser = typenv::parser::deserialize)]
            de_bad_level: LogLevel,
        }

        temp_env::with_var("DE_BAD_LEVEL", Some("trace"), || {
            assert_eq!(
                Config::from_env().unwrap_err().to_string(),
                "Configuration errors:\n  \
                 - `de_bad_level`: Failed to parse environment variable `DE_BAD_LEVEL=\"trace\"`: \
                 unknown variant `trace`, expected one of `debug`, `info`, `warn`\n"
            );
        });
    }

    #[test]
    fn primitives_are_parsed() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Port(u16);

        #[derive(FromEnv)]
        struct Config {
            #[env(parser = typenv::parser::deserialize)]
            de_port: u16,
            #[env(parser = typenv::parser::deserialize)]
            de_enabled: bool,
            #[env(parser = typenv::parser::deserialize)]
            de_ratio: f64,
            #[env(parser = typenv::parser::deserialize)]
            de_newtype: Port,
        }

        temp_env::with_vars(
            [
                ("DE_PORT", Some("8080")),
                ("DE_ENABLED", Some("true")),
                ("DE_RATIO", Some("0.5")),
                ("DE_NEWTYPE", Some("443")),
            ],
            || {
                let config = Config::from_env().unwrap();
                assert_eq!(config.de_port, 8080);
                assert!(config.de_enabled);
                assert_eq!(config.de_ratio, 0.5);
                assert_eq!(config.de_newtype, Port(443));
            },
        );
    }

    #[test]
    fn invalid_primitive_is_reported() {
        #[derive(FromEnv, Debug)]
        struct Config {
            #[allow(dead_code)]
            #[env(parser = typenv::parser::deserialize)]
            de_bad_port: u16,
            #[allow(dead_code)]
            #[env(parser = typenv::parser::deserialize)]
            de_bad_bool: bool,
        }

        temp_env::with_vars(
            [("DE_BAD_PORT", Some("http")), ("DE_BAD_BOOL", Some("yes"))],
            || {
                assert_eq!(
                    Config::from_env().unwrap_err().to_string(),
                    "Configuration errors:\n  \
                     - `de_bad_port`: Failed to parse environment variable `DE_BAD_PORT=\"http\"`: \
                     invalid digit found in string\n  \
                     - `de_bad_bool`: Failed to parse environment variable `DE_BAD_BOOL=\"yes\"`: \
                     provided string was not `true` or `false`\n"
                );
            },
        );
    }

    #[test]
    fn structured_values_are_rejected() {
        // Use `parser::json` for these.
        #[derive(FromEnv, Debug)]
        struct Config {
            #[allow(dead_code)]
            #[env(parser = typenv::parser::deserialize)]
            de_list: Vec<u16>,
        }

        temp_env::with_var("DE_LIST", Some("80,443"), || {
            assert_eq!(
                Config::from_env().unwrap_err().to_string(),
                "Configuration errors:\n  \
                 - `de_list`: Failed to parse environment variable `DE_LIST=\"80,443\"`: \
                 invalid type: string \"80,443\", expected a sequence\n"
            );
        });
    }
}
