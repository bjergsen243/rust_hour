use mockall::predicate::*;
use mockall::*;
use chrono::prelude::*;

use crate::types::account::{AccountId, Session};
use crate::types::answer::{Answer, AnswerId, NewAnswer};
use crate::types::question::QuestionId;
use crate::handle_errors;
use super::store_trait::StoreTrait;

mock! {
    Store {}

    #[async_trait::async_trait]
    impl StoreTrait for Store {
        async fn add_answer(&self, new_answer: NewAnswer, account_id: AccountId) -> Result<Answer, handle_errors::Error>;
        async fn is_answer_owner(&self, answer_id: i32, account_id: &AccountId) -> Result<bool, handle_errors::Error>;
        async fn update_answer(&self, answer: Answer, id: i32, account_id: AccountId) -> Result<Answer, handle_errors::Error>;
        async fn delete_answer(&self, id: i32, account_id: AccountId) -> Result<bool, handle_errors::Error>;
    }

    impl Clone for Store {
        fn clone(&self) -> Self;
    }
}

#[tokio::test]
async fn test_add_answer_success() {
    let mut mock_store = MockStore::new();
    let session = Session {
        account_id: AccountId(1),
        exp: Utc::now() + chrono::Duration::days(1),
        nbf: Utc::now(),
    };
    let new_answer = NewAnswer {
        content: "Test answer".to_string(),
        question_id: QuestionId(1),
    };

    let expected_answer = Answer {
        id: AnswerId(1),
        content: "Test answer".to_string(),
        question_id: QuestionId(1),
    };

    mock_store
        .expect_add_answer()
        .with(eq(new_answer.clone()), eq(AccountId(1)))
        .times(1)
        .returning(move |_, _| Ok(expected_answer.clone()));

    let result = super::add_answer(session, mock_store, new_answer).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_answer_success() {
    let mut mock_store = MockStore::new();
    let session = Session {
        account_id: AccountId(1),
        exp: Utc::now() + chrono::Duration::days(1),
        nbf: Utc::now(),
    };
    let answer = Answer {
        id: AnswerId(1),
        content: "Updated answer".to_string(),
        question_id: QuestionId(1),
    };
    let answer_clone = answer.clone();

    mock_store
        .expect_is_answer_owner()
        .with(eq(1), eq(&AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(true));

    mock_store
        .expect_update_answer()
        .with(eq(answer.clone()), eq(1), eq(AccountId(1)))
        .times(1)
        .returning(move |_, _, _| Ok(answer.clone()));

    let result = super::update_answer(1, session, mock_store, answer_clone).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_answer_unauthorized() {
    let mut mock_store = MockStore::new();
    let session = Session {
        account_id: AccountId(1),
        exp: Utc::now() + chrono::Duration::days(1),
        nbf: Utc::now(),
    };
    let answer = Answer {
        id: AnswerId(1),
        content: "Updated answer".to_string(),
        question_id: QuestionId(1),
    };

    mock_store
        .expect_is_answer_owner()
        .with(eq(1), eq(&AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(false));

    let result = super::update_answer(1, session, mock_store, answer).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_answer_success() {
    let mut mock_store = MockStore::new();
    let session = Session {
        account_id: AccountId(1),
        exp: Utc::now() + chrono::Duration::days(1),
        nbf: Utc::now(),
    };

    mock_store
        .expect_is_answer_owner()
        .with(eq(1), eq(&AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(true));

    mock_store
        .expect_delete_answer()
        .with(eq(1), eq(AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(true));

    let result = super::delete_answer(1, session, mock_store).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_answer_unauthorized() {
    let mut mock_store = MockStore::new();
    let session = Session {
        account_id: AccountId(1),
        exp: Utc::now() + chrono::Duration::days(1),
        nbf: Utc::now(),
    };

    mock_store
        .expect_is_answer_owner()
        .with(eq(1), eq(&AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(false));

    let result = super::delete_answer(1, session, mock_store).await;
    assert!(result.is_err());
}