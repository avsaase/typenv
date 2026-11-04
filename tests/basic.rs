use typenv::FromEnv;

#[test]
fn basic() {
    #[derive(FromEnv)]
    struct Config {
        db_url: String,
    }

    temp_env::with_var(
        "DB_URL",
        Some("postgres://user:password@localhost/db"),
        || {
            let config = Config::from_env().unwrap();
            assert_eq!(config.db_url, "postgres://user:password@localhost/db");
        },
    );
}
