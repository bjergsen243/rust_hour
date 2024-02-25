use chrono::prelude::*;
use serde::{Deserialize, Serialize};
// Represents a user session with authentication and timing information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    // Expiration time of the session in UTC.
    pub exp: DateTime<Utc>,
    // ID of the associated account.
    pub account_id: AccountId,
    // Time before which the session is not valid in UTC.
    pub nbf: DateTime<Utc>,
}

// Represents a user account with their credentials.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    // Unique identifier of the account (optional for new accounts).
    pub id: Option<AccountId>,
    // Email address of the user.
    pub email: String,
    // Password for the account (stored securely in a production environment).
    pub password: String,
}

// Represents a unique identifier for an account.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountId(pub i32);

// Used for requesting email updates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateRequest {
    // New email address for the account.
    pub email: String,
}

// Used for returning account information in responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    // Email address of the account.
    pub email: String,
    // ID of the account.
    pub id: AccountId,
}

// Used for requesting password updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdatePassword(pub String);
