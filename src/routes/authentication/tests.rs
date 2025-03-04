use mockall::predicate::*;
use mockall::*;
use chrono::prelude::*;

use crate::types::account::{Account, AccountId, AccountUpdatePassword, AccountUpdateRequest, AccountResponse};
use crate::handle_errors;
use super::StoreTrait;

mock! {
    Store {}

    #[async_trait::async_trait]
    impl StoreTrait for Store {
        async fn add_account(&self, account: Account) -> Result<bool, handle_errors::Error>;
        async fn get_account(&self, email: String) -> Result<Account, handle_errors::Error>;
        async fn update_account(&self, account_id: AccountId, account: AccountUpdateRequest) -> Result<AccountResponse, handle_errors::Error>;
        async fn update_password(&self, account_id: AccountId, password: AccountUpdatePassword) -> Result<bool, handle_errors::Error>;
        async fn get_account_information(&self, account_id: AccountId) -> Result<AccountResponse, handle_errors::Error>;
    }
}

#[tokio::test]
async fn test_register_success() {
    let mut mock_store = MockStore::new();
    let account = Account {
        id: Some(AccountId(1)),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    mock_store
        .expect_add_account()
        .with(always())
        .times(1)
        .returning(|_| Ok(true));

    let result = super::register(mock_store, account).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_login_success() {
    let mut mock_store = MockStore::new();
    let login_account = Account {
        id: None,
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let stored_account = Account {
        id: Some(AccountId(1)),
        email: "test@example.com".to_string(),
        password: super::hash_password("password123".as_bytes()),
    };

    mock_store
        .expect_get_account()
        .with(eq("test@example.com".to_string()))
        .times(1)
        .returning(move |_| Ok(stored_account.clone()));

    let result = super::login(mock_store, login_account).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let mut mock_store = MockStore::new();
    let login_account = Account {
        id: None,
        email: "test@example.com".to_string(),
        password: "wrongpassword".to_string(),
    };

    let stored_account = Account {
        id: Some(AccountId(1)),
        email: "test@example.com".to_string(),
        password: super::hash_password("password123".as_bytes()),
    };

    mock_store
        .expect_get_account()
        .with(eq("test@example.com".to_string()))
        .times(1)
        .returning(move |_| Ok(stored_account.clone()));

    let result = super::login(mock_store, login_account).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_login_wrong_email() {
    let mut mock_store = MockStore::new();
    let login_account = Account {
        id: None,
        email: "wrong@example.com".to_string(),
        password: "password123".to_string(),
    };

    mock_store
        .expect_get_account()
        .with(eq("wrong@example.com".to_string()))
        .times(1)
        .returning(|_| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));

    let result = super::login(mock_store, login_account).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_account() {
    let mut mock_store = MockStore::new();
    let session = super::Session {
        account_id: AccountId(1),
        exp: Utc::now() + chrono::Duration::days(1),
        nbf: Utc::now(),
    };
    let update_request = AccountUpdateRequest {
        email: "newemail@example.com".to_string(),
    };
    let account_response = AccountResponse {
        id: AccountId(1),
        email: "newemail@example.com".to_string(),
    };

    mock_store
        .expect_update_account()
        .with(eq(AccountId(1)), eq(update_request.clone()))
        .times(1)
        .returning(move |_, _| Ok(account_response.clone()));

    let result = super::update_account(session, mock_store, update_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_password() {
    let mut mock_store = MockStore::new();
    let session = super::Session {
        account_id: AccountId(1),
        exp: Utc::now() + chrono::Duration::days(1),
        nbf: Utc::now(),
    };
    let new_password = AccountUpdatePassword("newpassword123".to_string());

    mock_store
        .expect_update_password()
        .with(eq(AccountId(1)), always())
        .times(1)
        .returning(|_, _| Ok(true));

    let result = super::update_password(session, mock_store, new_password).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_account_information() {
    let mut mock_store = MockStore::new();
    let session = super::Session {
        account_id: AccountId(1),
        exp: Utc::now() + chrono::Duration::days(1),
        nbf: Utc::now(),
    };
    let account_response = AccountResponse {
        id: AccountId(1),
        email: "test@example.com".to_string(),
    };

    mock_store
        .expect_get_account_information()
        .with(eq(AccountId(1)))
        .times(1)
        .returning(move |_| Ok(account_response.clone()));

    let result = super::get_account_information(session, mock_store).await;
    assert!(result.is_ok());
} 