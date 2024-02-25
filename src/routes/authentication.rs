use argon2::Config;
use chrono::prelude::*;
use rand::Rng;
use std::{env, future};
use warp::Filter;

use crate::store::Store;
use crate::types::account::{
    Account, AccountId, AccountUpdatePassword, AccountUpdateRequest, Session,
};

/**
 * @Notice Registration
 *
 * @Dev Registers a new account in the database
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `account`: An `Account` struct containing the account information to be registered.
*/
pub async fn register(store: Store, account: Account) -> Result<impl warp::Reply, warp::Rejection> {
    // Hashes the provided password using a secure algorithm.
    let hashed_password = hash_password(account.password.as_bytes());
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
pub async fn login(store: Store, login: Account) -> Result<impl warp::Reply, warp::Rejection> {
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

// Verifies a provided token and extracts the associated session data
pub fn verify_token(token: String) -> Result<Session, handle_errors::Error> {
    // Retrieve the PASETO key from the environment variable.
    let key = env::var("PASETO_KEY").unwrap();
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

// Hashes a password securely using Argon2id and returns the result as a string.
fn hash_password(password: &[u8]) -> String {
    // Generate a random 32-byte salt for added security.
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    // Use the default Argon2id configuration for secure hashing.
    let config = Config::default();
    // Hash the password with the provided salt and configuration.
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

// Verifies a password by comparing it to a stored hash and returns true if they match.
fn verify_password(hash: &str, password: &[u8]) -> Result<bool, argon2::Error> {
    // Verify the provided password against the stored hash and salt.
    argon2::verify_encoded(hash, password)
}

// This function creates and returns a signed PASETO token containing the account ID.
fn issue_token(account_id: AccountId) -> String {
    // Retrieve the PASETO key securely from the environment variable.
    let key = env::var("PASETO_KEY").unwrap();
    // Set the expiration date of the token to one day in the future.
    let current_date_time = Utc::now();
    let dt = current_date_time + chrono::Duration::days(1);

    // Build the PASETO token using a builder.
    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(&Vec::from(key.as_bytes()))
        .set_expiration(&dt)
        .set_not_before(&Utc::now())
        .set_claim("account_id", serde_json::json!(account_id))
        .build()
        .expect("Failed to construct paseto token w/ builder!")
}

// This function defines a filter for authentication within your Warp application.
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

/**
 * @Notice Update account
 *
 * @Dev Attempts to update a user's email with new email by validating their credentials.
 *
 * @params  `session`: A `Session` struct containing the user's id
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `account`: An `AccountUpdateRequest` struct containing the user's email to update
*/
pub async fn update_account(
    session: Session,
    store: Store,
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
pub async fn update_password(
    session: Session,
    store: Store,
    password: AccountUpdatePassword,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    let hashed_password = AccountUpdatePassword(hash_password(password.0.as_bytes()));

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
pub async fn get_account_information(
    session: Session,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    match store.get_account_information(account_id).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

#[cfg(test)]
mod authentication_tests {
    use super::*;

    #[tokio::test]
    async fn auth_valid() {
        env::set_var("PASETO_KEY", "RANDOM WORDS WINTER MACINTOSH PC");
        let token = issue_token(AccountId(3));

        let filter = auth();

        let res = warp::test::request()
            .header("Authorization", token)
            .filter(&filter);

        assert_eq!(res.await.unwrap().account_id, AccountId(3));
    }

    #[tokio::test]
    async fn auth_invalid() {}

    #[tokio::test]
    async fn register_valid() {}

    #[tokio::test]
    async fn register_missing_email() {}

    #[tokio::test]
    async fn register_missing_password() {}

    #[tokio::test]
    async fn login_valid() {}

    #[tokio::test]
    async fn login_missing_email() {}

    #[tokio::test]
    async fn login_missing_password() {}

    #[tokio::test]
    async fn login_wrong_password() {}

    #[tokio::test]
    async fn login_invalid_email() {}

    #[tokio::test]
    async fn update_account_valid() {}

    #[tokio::test]
    async fn update_account_missing_email() {}

    #[tokio::test]
    async fn update_password_valid() {}

    #[tokio::test]
    async fn update_password_missing_password() {}

    #[tokio::test]
    async fn get_account_information_valid() {}
}
