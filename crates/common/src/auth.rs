use chrono::{Duration, Utc};
use paseto::tokens::{PasetoBuilder, TimeBackend};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub account_id: String,
    pub email: String,
}

pub fn issue_token(account_id: &str, email: &str, key: &[u8]) -> Result<String, AppError> {
    let now = Utc::now();
    let expiration = now + Duration::hours(24);

    let token = PasetoBuilder::new()
        .set_encryption_key(key)
        .set_expiration(&expiration)
        .set_not_before(&now)
        .set_claim("account_id", serde_json::json!(account_id))
        .set_claim("email", serde_json::json!(email))
        .build()
        .map_err(|e| AppError::Internal(format!("Token creation failed: {e}")))?;

    Ok(token)
}

pub fn validate_token(token: &str, key: &[u8]) -> Result<Claims, AppError> {
    let verified = paseto::tokens::validate_local_token(
        token,
        None,
        key,
        &TimeBackend::Chrono,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {e}")))?;

    let account_id = verified
        .get("account_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Unauthorized("Missing account_id in token".into()))?
        .to_string();

    let email = verified
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Unauthorized("Missing email in token".into()))?
        .to_string();

    Ok(Claims { account_id, email })
}
