//! The generated `ENV_VARS` const.

use typenv::FromEnv;

#[test]
fn one_line_per_variable() {
    #[derive(FromEnv)]
    #[env(prefix = "APP_")]
    #[allow(dead_code)]
    struct Config {
        port: u16,
        log_level: Option<String>,
        #[env(default = "localhost")]
        host: String,
        #[env(default_value = std::time::Duration::from_secs(5))]
        #[env(parser = |s: &str| Ok(std::time::Duration::from_secs(s.parse()?)))]
        timeout: std::time::Duration,
        #[env(rename = "workers")]
        threads: usize,
        #[env(var = "DATABASE_URL")]
        database_url: String,
        #[env(parser = |s: &str| Ok(s.split(',').map(str::to_owned).collect()))]
        tags: Option<Vec<String>>,
    }

    assert_eq!(
        Config::ENV_VARS,
        "APP_PORT\n\
         APP_LOG_LEVEL (optional)\n\
         APP_HOST (default: localhost)\n\
         APP_TIMEOUT (default: std::time::Duration::from_secs(5))\n\
         APP_WORKERS\n\
         DATABASE_URL\n\
         APP_TAGS (optional)\n"
    );
}

#[test]
fn nested_structs_are_included() {
    #[derive(FromEnv)]
    #[env(prefix = "APP_")]
    #[allow(dead_code)]
    struct Config {
        port: u16,
        #[env(nested)]
        database: Database,
        debug: Option<bool>,
    }

    #[derive(FromEnv)]
    #[env(prefix = "DB_")]
    #[allow(dead_code)]
    struct Database {
        url: String,
        #[env(nested)]
        pool: Pool,
    }

    #[derive(FromEnv)]
    #[env(prefix = "POOL_")]
    #[allow(dead_code)]
    struct Pool {
        #[env(default = "5")]
        size: u32,
    }

    // Each struct uses its own prefix, in field order. Only nested variables name their source.
    assert_eq!(
        Config::ENV_VARS,
        "APP_PORT\n\
         DB_URL, from Database::url\n\
         POOL_SIZE (default: 5), from Pool::size\n\
         APP_DEBUG (optional)\n"
    );

    // On its own, a nested struct's variables have no source, but its own children's do.
    assert_eq!(
        Database::ENV_VARS,
        "DB_URL\n\
         POOL_SIZE (default: 5), from Pool::size\n"
    );
}

#[test]
fn generic_structs() {
    #[derive(FromEnv)]
    #[allow(dead_code)]
    struct Inner {
        inner_value: u8,
    }

    #[derive(FromEnv)]
    #[allow(dead_code)]
    struct WithParam<T>
    where
        T: std::str::FromStr + Default,
        T::Err: Into<typenv::BoxError>,
    {
        #[env(default_value = T::default())]
        value: T,
        #[env(nested)]
        inner: Inner,
    }

    #[derive(FromEnv)]
    #[allow(dead_code)]
    struct Outer<T: FromEnv> {
        #[env(nested)]
        generic: T,
        #[env(nested)]
        inner: Inner,
    }

    // A nested type that does not use the struct's generic parameters is included.
    assert_eq!(
        WithParam::<u8>::ENV_VARS,
        "VALUE (default: T::default())\n\
         INNER_VALUE, from Inner::inner_value\n"
    );

    // One that does is referenced instead.
    assert_eq!(
        Outer::<Inner>::ENV_VARS,
        "variables of `T`, from Outer::generic\n\
         INNER_VALUE, from Inner::inner_value\n"
    );
}

#[test]
fn empty_struct() {
    #[derive(FromEnv)]
    struct Empty {}

    assert_eq!(Empty::ENV_VARS, "");
}

#[test]
fn usable_in_const_context_and_through_the_trait() {
    #[derive(FromEnv)]
    #[allow(dead_code)]
    struct Config {
        #[env(nested)]
        inner: Inner,
    }

    #[derive(FromEnv)]
    #[allow(dead_code)]
    struct Inner {
        const_value: u8,
    }

    const DOCS: &str = Config::ENV_VARS;

    fn docs<T: FromEnv>() -> &'static str {
        T::ENV_VARS
    }

    assert_eq!(DOCS, "CONST_VALUE, from Inner::const_value\n");
    assert_eq!(docs::<Config>(), DOCS);
}

#[test]
fn manual_impl_only_needs_env_vars() {
    struct Manual;

    impl FromEnv for Manual {
        const ENV_VARS: &'static str = "MANUAL_VALUE, from Manual\n";

        fn from_env() -> Result<Self, typenv::FromEnvErrors> {
            Ok(Manual)
        }
    }

    #[derive(FromEnv)]
    #[allow(dead_code)]
    struct Config {
        own: u8,
        #[env(nested)]
        manual: Manual,
    }

    // Without `__NESTED_ENV_VARS`, the trait's default uses `ENV_VARS` as written.
    assert_eq!(
        Config::ENV_VARS,
        "OWN\n\
         MANUAL_VALUE, from Manual\n"
    );
}
