//! The generated inherent `from_env`, callable without importing the trait.

// Deliberately no `use from_env::FromEnv;` in this file.

mod config {
    #[derive(from_env::FromEnv)]
    pub struct Public {
        pub inherent_public: u16,
    }

    #[derive(from_env::FromEnv)]
    pub(crate) struct CrateVisible {
        pub inherent_crate: u16,
    }
}

#[test]
fn callable_without_trait_import() {
    temp_env::with_var("INHERENT_PUBLIC", Some("1"), || {
        assert_eq!(config::Public::from_env().unwrap().inherent_public, 1);
    });
}

#[test]
fn uses_struct_visibility() {
    temp_env::with_var("INHERENT_CRATE", Some("2"), || {
        assert_eq!(config::CrateVisible::from_env().unwrap().inherent_crate, 2);
    });
}

#[test]
fn generic_struct() {
    #[derive(from_env::FromEnv)]
    struct Config<T>
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        inherent_generic: T,
    }

    temp_env::with_var("INHERENT_GENERIC", Some("3"), || {
        assert_eq!(Config::<u8>::from_env().unwrap().inherent_generic, 3);
    });
}

#[test]
fn same_result_as_trait() {
    #[derive(from_env::FromEnv, Debug)]
    struct Config {
        #[allow(dead_code)]
        inherent_same: u16,
    }

    temp_env::with_var("INHERENT_SAME", Some("x"), || {
        assert_eq!(
            Config::from_env().unwrap_err().to_string(),
            <Config as from_env::FromEnv>::from_env()
                .unwrap_err()
                .to_string()
        );
    });
}

#[test]
fn trait_is_still_implemented() {
    fn load<T: from_env::FromEnv>() -> Result<T, from_env::FromEnvErrors> {
        T::from_env()
    }

    #[derive(from_env::FromEnv)]
    struct Config {
        inherent_trait: u16,
    }

    temp_env::with_var("INHERENT_TRAIT", Some("4"), || {
        assert_eq!(load::<Config>().unwrap().inherent_trait, 4);
    });
}
