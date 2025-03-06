#[cfg(test)]
use mockall::predicate::*;
use mockall::*;
use chrono::prelude::*;
use std::sync::{Arc, Mutex};

use crate::types::account::{Account, AccountId, Session, AccountUpdateRequest, AccountUpdatePassword, AccountResponse};
use crate::handle_errors;
use super::StoreTrait;

mock! {
    #[derive(Debug)]
    Store {}

    #[async_trait::async_trait]
    impl StoreTrait for Store {
        async fn add_account(&self, account: Account) -> Result<bool, handle_errors::Error>;
        async fn get_account(&self, email: String) -> Result<Account, handle_errors::Error>;
        async fn update_account(&self, account_id: AccountId, account: AccountUpdateRequest) -> Result<AccountResponse, handle_errors::Error>;
        async fn update_password(&self, account_id: AccountId, password: AccountUpdatePassword) -> Result<bool, handle_errors::Error>;
        async fn get_account_information(&self, account_id: AccountId) -> Result<AccountResponse, handle_errors::Error>;
    }

    impl Clone for Store {
        fn clone(&self) -> Self;
    }
}

fn setup_mock_store() -> Arc<Mutex<MockStore>> {
    let mock_store = Arc::new(Mutex::new(MockStore::new()));
    let mut mock = mock_store.lock().unwrap();
    
    mock.expect_clone()
        .returning(|| MockStore::new());
        
    drop(mock);
    mock_store
}

fn create_test_session() -> Session {
    Session {
        account_id: AccountId(1),
        exp: Utc::now() + chrono::Duration::days(1),
        nbf: Utc::now(),
    }
}

#[tokio::test]
async fn test_register_success() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    let account = Account {
        id: None,
        email: "test@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    store.expect_add_account()
        .with(predicate::function(|a: &Account| {
            a.email == "test@test.com" && a.password != "password123" // Password should be hashed
        }))
        .times(1)
        .returning(|_| Ok(true));
    
    let result = super::register(store, account).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_login_success() {
    std::env::set_var("PASETO_KEY", "RANDOM_KEY_ONLY_USED_FOR_TESTS32");
    
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    let login = Account {
        id: None,
        email: "test@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    // Generate a valid Argon2 hash for "password123"
    let hashed_password = super::hash_password("password123".as_bytes()).expect("Failed to hash password");
    
    store.expect_get_account()
        .with(eq("test@test.com".to_string()))
        .times(1)
        .returning(move |_| Ok(Account {
            id: Some(AccountId(1)),
            email: "test@test.com".to_string(),
            password: hashed_password.clone(),
        }));
    
    let result = super::login(store, login).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    let login = Account {
        id: None,
        email: "test@test.com".to_string(),
        password: "wrongpassword".to_string(),
    };
    
    // Generate a valid Argon2 hash for "password123"
    let hashed_password = super::hash_password("password123".as_bytes()).expect("Failed to hash password");
    
    store.expect_get_account()
        .with(eq("test@test.com".to_string()))
        .times(1)
        .returning(move |_| Ok(Account {
            id: Some(AccountId(1)),
            email: "test@test.com".to_string(),
            password: hashed_password.clone(),
        }));
    
    let result = super::login(store, login).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_account_success() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    let update_request = AccountUpdateRequest {
        email: "updated@test.com".to_string(),
    };
    
    store.expect_update_account()
        .with(eq(AccountId(1)), eq(update_request.clone()))
        .times(1)
        .returning(|_, _| Ok(AccountResponse {
            id: AccountId(1),
            email: "updated@test.com".to_string(),
        }));
    
    let result = super::update_account(session, store, update_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_password_success() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    let password_update = AccountUpdatePassword("newpassword123".to_string());
    
    store.expect_update_password()
        .with(eq(AccountId(1)), predicate::function(|p: &AccountUpdatePassword| {
            p.0 != "newpassword123" // Password should be hashed
        }))
        .times(1)
        .returning(|_, _| Ok(true));
    
    let result = super::update_password(session, store, password_update).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_account_information_success() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    store.expect_get_account_information()
        .with(eq(AccountId(1)))
        .times(1)
        .returning(|_| Ok(AccountResponse {
            id: AccountId(1),
            email: "test@test.com".to_string(),
        }));
    
    let result = super::get_account_information(session, store).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    let account = Account {
        id: None,
        email: "test@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    store.expect_add_account()
        .with(predicate::always())
        .times(1)
        .returning(|_| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = super::register(store, account).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_login_account_not_found() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    let login = Account {
        id: None,
        email: "nonexistent@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    store.expect_get_account()
        .with(eq("nonexistent@test.com".to_string()))
        .times(1)
        .returning(|_| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = super::login(store, login).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_token_success() {
    std::env::set_var("PASETO_KEY", "RANDOM_KEY_ONLY_USED_FOR_TESTS32");
    let session = create_test_session();
    let token = super::issue_token(session.account_id);
    let result = super::verify_token(token);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_verify_token_invalid() {
    std::env::set_var("PASETO_KEY", "RANDOM_KEY_ONLY_USED_FOR_TESTS32");
    let result = super::verify_token("invalid_token".to_string());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_auth_header_missing() {
    let auth_filter = super::auth();
    let result = warp::test::request()
        .path("/")
        .filter(&auth_filter);
    assert!(result.await.is_err());
}

#[tokio::test]
async fn test_auth_header_invalid() {
    let auth_filter = super::auth();
    let result = warp::test::request()
        .header("Authorization", "invalid_token")
        .path("/")
        .filter(&auth_filter);
    assert!(result.await.is_err());
}

#[tokio::test]
async fn test_auth_header_valid() {
    std::env::set_var("PASETO_KEY", "RANDOM_KEY_ONLY_USED_FOR_TESTS32");
    let session = create_test_session();
    let token = super::issue_token(session.account_id);
    let auth_filter = super::auth();
    
    let result = warp::test::request()
        .header("Authorization", token)
        .path("/")
        .filter(&auth_filter);
    assert!(result.await.is_ok());
}

#[tokio::test]
async fn test_store_trait_add_account_error() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    let account = Account {
        id: None,
        email: "test@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    store.expect_add_account()
        .with(predicate::always())
        .times(1)
        .returning(|_| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = store.add_account(account).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_store_trait_get_account_error() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    store.expect_get_account()
        .with(eq("test@test.com".to_string()))
        .times(1)
        .returning(|_| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = store.get_account("test@test.com".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_store_trait_update_account_error() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    let update_request = AccountUpdateRequest {
        email: "updated@test.com".to_string(),
    };
    
    store.expect_update_account()
        .with(eq(AccountId(1)), eq(update_request.clone()))
        .times(1)
        .returning(|_, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = store.update_account(AccountId(1), update_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_store_trait_update_password_error() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    let password = AccountUpdatePassword("newpassword123".to_string());
    
    store.expect_update_password()
        .with(eq(AccountId(1)), eq(password.clone()))
        .times(1)
        .returning(|_, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = store.update_password(AccountId(1), password).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_store_trait_get_account_information_error() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    store.expect_get_account_information()
        .with(eq(AccountId(1)))
        .times(1)
        .returning(|_| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = store.get_account_information(AccountId(1)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_argon_error() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    let account = Account {
        id: None,
        email: "test@test.com".to_string(),
        password: "".to_string(), // Empty password to trigger Argon2 error
    };
    
    store.expect_add_account()
        .with(predicate::always())
        .times(0); // We expect no calls to add_account because hashing will fail
    
    let result = super::register(store, account).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_account_database_error() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    let update_request = AccountUpdateRequest {
        email: "updated@test.com".to_string(),
    };
    
    store.expect_update_account()
        .with(eq(AccountId(1)), eq(update_request.clone()))
        .times(1)
        .returning(|_, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = super::update_account(session, store, update_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_password_database_error() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    let password_update = AccountUpdatePassword("newpassword123".to_string());
    
    store.expect_update_password()
        .with(eq(AccountId(1)), predicate::function(|p: &AccountUpdatePassword| {
            p.0 != "newpassword123" // Password should be hashed
        }))
        .times(1)
        .returning(|_, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = super::update_password(session, store, password_update).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_account_information_database_error() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    store.expect_get_account_information()
        .with(eq(AccountId(1)))
        .times(1)
        .returning(|_| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    
    let result = super::get_account_information(session, store).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_token_invalid_key() {
    std::env::remove_var("PASETO_KEY");
    let result = super::verify_token("some_token".to_string());
    assert!(matches!(result, Err(handle_errors::Error::EnvironmentError(_))));
} 