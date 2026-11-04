//! `#[env(prefix = "...")]` on structs.

use from_env::FromEnv;

#[test]
fn prefix_is_prepended_to_field_names() {
    #[derive(FromEnv)]
    #[env(prefix = "PREFIX_APP_")]
    struct Config {
        name: String,
        port: u16,
    }

    temp_env::with_vars(
        [
            ("PREFIX_APP_NAME", Some("app")),
            ("PREFIX_APP_PORT", Some("80")),
            ("NAME", Some("wrong")),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!(config.name, "app");
            assert_eq!(config.port, 80);
        },
    );
}

#[test]
fn prefix_is_used_verbatim() {
    #[derive(FromEnv)]
    #[env(prefix = "prefix_Mixed.")]
    struct Config {
        value: String,
    }

    temp_env::with_var("prefix_Mixed.VALUE", Some("verbatim"), || {
        assert_eq!(Config::from_env().unwrap().value, "verbatim");
    });
}

#[test]
fn prefix_is_combined_with_rename() {
    #[derive(FromEnv)]
    #[env(prefix = "PREFIX_RN_")]
    struct Config {
        #[env(rename = "database_url")]
        url: String,
    }

    temp_env::with_vars(
        [
            ("PREFIX_RN_DATABASE_URL", Some("prefixed")),
            ("DATABASE_URL", Some("unprefixed")),
        ],
        || {
            assert_eq!(Config::from_env().unwrap().url, "prefixed");
        },
    );
}

#[test]
fn var_ignores_prefix() {
    #[derive(FromEnv)]
    #[env(prefix = "PREFIX_VAR_")]
    struct Config {
        #[env(var = "PREFIX_EXACT")]
        value: String,
    }

    temp_env::with_vars(
        [
            ("PREFIX_EXACT", Some("exact")),
            ("PREFIX_VAR_PREFIX_EXACT", Some("wrong")),
            ("PREFIX_VAR_VALUE", Some("wrong")),
        ],
        || {
            assert_eq!(Config::from_env().unwrap().value, "exact");
        },
    );
}

#[test]
fn prefix_applies_to_optional_and_default_fields() {
    #[derive(FromEnv)]
    #[env(prefix = "PREFIX_OD_")]
    struct Config {
        optional: Option<u8>,
        #[env(default = "1")]
        defaulted: u8,
    }

    temp_env::with_vars(
        [
            ("PREFIX_OD_OPTIONAL", Some("5")),
            ("PREFIX_OD_DEFAULTED", Some("6")),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!((config.optional, config.defaulted), (Some(5), 6));
        },
    );
}

#[test]
fn prefix_appears_in_missing_var_error() {
    #[derive(FromEnv, Debug)]
    #[env(prefix = "PREFIX_ERR_")]
    struct Config {
        #[allow(dead_code)]
        value: String,
    }

    temp_env::with_var_unset("PREFIX_ERR_VALUE", || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            "Configuration errors:\n  \
             - `value`: Missing required environment variable: `PREFIX_ERR_VALUE`\n"
        );
    });
}

#[test]
fn prefix_is_not_inherited_by_nested_structs() {
    #[derive(FromEnv)]
    #[env(prefix = "PREFIX_PARENT_")]
    struct Parent {
        name: String,
        #[env(nested)]
        child: Child,
        #[env(nested)]
        prefixed_child: PrefixedChild,
    }

    #[derive(FromEnv)]
    struct Child {
        prefix_child_value: String,
    }

    #[derive(FromEnv)]
    #[env(prefix = "PREFIX_OWN_")]
    struct PrefixedChild {
        value: String,
    }

    temp_env::with_vars(
        [
            ("PREFIX_PARENT_NAME", Some("parent")),
            ("PREFIX_CHILD_VALUE", Some("child")),
            ("PREFIX_PARENT_PREFIX_CHILD_VALUE", Some("wrong")),
            ("PREFIX_OWN_VALUE", Some("own")),
            ("PREFIX_PARENT_PREFIX_OWN_VALUE", Some("wrong")),
        ],
        || {
            let parent = Parent::from_env().unwrap();
            assert_eq!(parent.name, "parent");
            assert_eq!(parent.child.prefix_child_value, "child");
            assert_eq!(parent.prefixed_child.value, "own");
        },
    );
}
