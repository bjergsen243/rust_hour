use argon2::{self, Config, Variant, Version};
use rand::Rng;

use rust_hour_common::error::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt: [u8; 32] = rand::thread_rng().gen();
    let config = Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        ..Default::default()
    };

    argon2::hash_encoded(password.as_bytes(), &salt, &config)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    argon2::verify_encoded(hash, password.as_bytes())
        .map_err(|e| AppError::Internal(format!("Password verification failed: {e}")))
}
