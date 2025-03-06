use rust_hour::{config, run, setup_store};

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();
    let config = config::Config::new().expect("Config can't be set");
    let store = setup_store(&config).await?;
    tracing::info!("Q&A service build ID {}", env!("RUST_WEB_DEV_VERSION"));
    run(config, store).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_server_setup() {
        // Save current environment variables
        let original_database_url = env::var("DATABASE_URL").ok();
        let original_paseto_key = env::var("PASETO_KEY").ok();
        let original_db_password = env::var("DB_PASSWORD").ok();
        
        // Set test environment variables
        env::set_var("DATABASE_URL", "postgres://postgres:password@localhost:5432/rust_hour_test");
        env::set_var("PASETO_KEY", "RANDOM_KEY_ONLY_USED_FOR_TESTS32");
        env::set_var("DB_PASSWORD", "password");
        
        // Test config creation
        let config = config::Config::new();
        assert!(config.is_ok());
        
        // Restore original environment variables
        match original_database_url {
            Some(url) => env::set_var("DATABASE_URL", url),
            None => env::remove_var("DATABASE_URL"),
        }
        match original_paseto_key {
            Some(key) => env::set_var("PASETO_KEY", key),
            None => env::remove_var("PASETO_KEY"),
        }
        match original_db_password {
            Some(pwd) => env::set_var("DB_PASSWORD", pwd),
            None => env::remove_var("DB_PASSWORD"),
        }
    }
}
