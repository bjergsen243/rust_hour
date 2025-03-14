use clap::Parser;
use std::env;

/// Q&A web service API
#[derive(Parser, Debug, PartialEq)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Which errors we want to log (info, warn or error)
    #[clap(short, long, default_value = "warn")]
    pub log_level: String,
    /// Which PORT the server is listening to
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
    /// Database user
    #[clap(long, default_value = "postgres")]
    pub db_user: String,
    /// Database user
    #[clap(long, default_value = "postgres")]
    pub db_password: String,
    /// URL for the postgres database
    #[clap(long, default_value = "localhost")]
    pub db_host: String,
    /// PORT number for the database connection
    #[clap(long, default_value = "5432")]
    pub db_port: u16,
    /// Database name
    #[clap(long, default_value = "rust_hour")]
    pub db_name: String,
}

impl Config {
    pub fn new() -> Result<Config, handle_errors::Error> {
        let config = Config::parse();

        if env::var("PASETO_KEY").is_err() {
            panic!("PASETO_KEY not set");
        }

        let port = std::env::var("PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(config.port))
            .map_err(handle_errors::Error::ParseError)?;

        let db_user = env::var("DB_USER").unwrap_or(config.db_user.to_owned());
        let db_password = env::var("DB_PASSWORD").unwrap();
        let db_host = env::var("DB_HOST").unwrap_or(config.db_host.to_owned());
        let db_port = env::var("DB_PORT").unwrap_or(config.db_port.to_string());
        let db_name = env::var("DB_NAME").unwrap_or(config.db_name.to_owned());

        Ok(Config {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port: db_port
                .parse::<u16>()
                .map_err(handle_errors::Error::ParseError)?,
            db_name,
        })
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    fn set_env() {
        env::set_var("PASETO_KEY", "RANDOM WORDS WINTER MACINTOSH PC");
        env::set_var("DB_USER", "postgres");
        env::set_var("DB_PASSWORD", "postgres");
        env::set_var("DB_HOST", "localhost");
        env::set_var("DB_PORT", "5432");
        env::set_var("DB_NAME", "rust_hour");
    }

    #[test]
    fn unset_and_set_api_key() {
        let result = std::panic::catch_unwind(|| Config::new());
        assert!(result.is_err());

        set_env();

        let expected = Config {
            log_level: "warn".to_string(),
            port: 8080,
            db_user: "postgres".to_string(),
            db_password: "postgres".to_string(),
            db_host: "localhost".to_string(),
            db_port: 5432,
            db_name: "rust_hour".to_string(),
        };

        let config = Config::new().unwrap();

        assert_eq!(config, expected);
    }
}
