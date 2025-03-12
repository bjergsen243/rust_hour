use argon2::Config;
use chrono::prelude::*;
use rand::Rng;
use std::{env, future};
use warp::Filter;

use crate::store::Store;
use crate::types::account::{
    Account, AccountId, AccountUpdatePassword, AccountUpdateRequest, Session, AccountResponse,
};

#[cfg(test)]
mod tests;

#[async_trait::async_trait]
pub trait StoreTrait {
    async fn add_account(&self, account: Account) -> Result<bool, handle_errors::Error>;
    async fn get_account(&self, email: String) -> Result<Account, handle_errors::Error>;
    async fn update_account(&self, account_id: AccountId, account: AccountUpdateRequest) -> Result<AccountResponse, handle_errors::Error>;
    async fn update_password(&self, account_id: AccountId, password: AccountUpdatePassword) -> Result<bool, handle_errors::Error>;
    async fn get_account_information(&self, account_id: AccountId) -> Result<AccountResponse, handle_errors::Error>;
}

#[async_trait::async_trait]
impl StoreTrait for Store {
    async fn add_account(&self, account: Account) -> Result<bool, handle_errors::Error> {
        self.add_account(account).await
    }

    async fn get_account(&self, email: String) -> Result<Account, handle_errors::Error> {
        self.get_account(email).await
    }

    async fn update_account(&self, account_id: AccountId, account: AccountUpdateRequest) -> Result<AccountResponse, handle_errors::Error> {
        self.update_account(account_id, account).await
    }

    async fn update_password(&self, account_id: AccountId, password: AccountUpdatePassword) -> Result<bool, handle_errors::Error> {
        self.update_password(account_id, password).await
    }

    async fn get_account_information(&self, account_id: AccountId) -> Result<AccountResponse, handle_errors::Error> {
        self.get_account_information(account_id).await
    }
}

/**
 * @Notice Registration
 *
 * @Dev Registers a new account in the database
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `account`: An `Account` struct containing the account information to be registered.
*/
pub async fn register<S: StoreTrait>(store: S, account: Account) -> Result<impl warp::Reply, warp::Rejection> {
    // Hashes the provided password using a secure algorithm.
    let hashed_password = hash_password(account.password.as_bytes())
        .map_err(|e| warp::reject::custom(handle_errors::Error::ArgonLibraryError(e)))?;
    // Creates a new `Account` struct with the hashed password.
    let account = Account {
        id: account.id,
        email: account.email,
        password: hashed_password,
    };
    // Attempts to add the account to the database using the `store` instance.
    match store.add_account(account).await {
        Ok(_) => Ok(warp::reply::json(&"Account added".to_string())),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/**
 * @Notice Log in
 *
 * @Dev Attempts to log in a user by validating their credentials.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `login`: An `Account` struct containing the user's email and password
*/
pub async fn login<S: StoreTrait>(store: S, login: Account) -> Result<impl warp::Reply, warp::Rejection> {
    // Attempts to retrieve the account associated with the provided email.
    match store.get_account(login.email).await {
        Ok(account) => match verify_password(&account.password, login.password.as_bytes()) {
            Ok(verified) => {
                // Verifies the provided password against the stored password hash.
                if verified {
                    // Generates a token if password verification is successful.
                    Ok(warp::reply::json(&issue_token(
                        account.id.expect("id not found"),
                    )))
                } else {
                    // Returns an error if the password is incorrect.
                    Err(warp::reject::custom(handle_errors::Error::WrongPassword))
                }
            }
            // Handles errors during password verification.
            Err(e) => Err(warp::reject::custom(
                handle_errors::Error::ArgonLibraryError(e),
            )),
        },
        // Handles errors during account lookup.
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/**
 * @Notice Update account
 *
 * @Dev Attempts to update a user's email with new email by validating their credentials.
 *
 * @params  `session`: A `Session` struct containing the user's id
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `account`: An `AccountUpdateRequest` struct containing the user's email to update
*/
pub async fn update_account<S: StoreTrait>(
    session: Session,
    store: S,
    account: AccountUpdateRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    match store.update_account(account_id, account).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/**
 * @Notice Update password
 *
 * @Dev Attempts to update a user's password with new password by validating their credentials.
 *
 * @params  `session`: A `Session` struct containing the user's id
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `login`: A `Account` struct containing the user's email and password
*/
pub async fn update_password<S: StoreTrait>(
    session: Session,
    store: S,
    password: AccountUpdatePassword,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    let hashed_password = AccountUpdatePassword(hash_password(password.0.as_bytes())
        .map_err(handle_errors::Error::ArgonLibraryError)?);

    match store.update_password(account_id, hashed_password).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/**
 * @Notice Get account information
 *
 * @Dev Attempts to get a user's information by validating their credentials.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `session`: A `Session` struct containing the user's id
*/
pub async fn get_account_information<S: StoreTrait>(
    session: Session,
    store: S,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    match store.get_account_information(account_id).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

// Hashes a password securely using Argon2id and returns the result as a string.
fn hash_password(password: &[u8]) -> Result<String, argon2::Error> {
    if password.is_empty() {
        return Err(argon2::Error::PwdTooShort);
    }
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config)
}

// Verifies a password against its hash using Argon2id.
fn verify_password(hash: &str, password: &[u8]) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password)
}

// Verifies a provided token and extracts the associated session data
pub fn verify_token(token: String) -> Result<Session, handle_errors::Error> {
    // Retrieve the PASETO key from the environment variable.
    let key = env::var("PASETO_KEY").map_err(handle_errors::Error::EnvironmentError)?;
    // Attempt to validate the token using PASETO's local token validation.
    let token = paseto::tokens::validate_local_token(
        &token,
        None,
        key.as_bytes(),
        &paseto::tokens::TimeBackend::Chrono,
    )
    .map_err(|_| handle_errors::Error::CannotDecryptToken)?;
    // Deserialize the token's payload into a `Session` struct.
    serde_json::from_value::<Session>(token).map_err(|_| handle_errors::Error::CannotDecryptToken)
}

// Generates a PASETO token containing session information.
fn issue_token(account_id: AccountId) -> String {
    let key = env::var("PASETO_KEY").expect("PASETO_KEY must be set");
    let current_date_time = Utc::now();
    let exp = current_date_time + chrono::Duration::days(1);

    let session = Session {
        account_id,
        exp,
        nbf: current_date_time,
    };

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(&Vec::from(key.as_bytes()))
        .set_expiration(&exp)
        .set_not_before(&session.nbf)
        .set_claim("account_id", serde_json::json!(session.account_id))
        .build()
        .expect("Failed to construct paseto token w/ builder!")
}

pub fn auth() -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    // Extract the "Authorization" header from the request.
    warp::header::<String>("Authorization").and_then(|token: String| {
        // Attempt to verify the provided token using the `verify_token` function.
        let token = match verify_token(token) {
            Ok(t) => t,
            Err(_) => return future::ready(Err(warp::reject::reject())),
        };

        future::ready(Ok(token))
    })
}
