#[cfg(test)]
use mockall::predicate::*;
use mockall::*;
use chrono::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use warp::http::StatusCode;

use crate::types::account::{AccountId, Session};
use crate::types::answer::{Answer, AnswerId};
use crate::types::question::{NewQuestion, Question, QuestionId};
use crate::handle_errors;
use super::store_trait::StoreTrait;

mock! {
    #[derive(Debug)]
    Store {}

    #[async_trait::async_trait]
    impl StoreTrait for Store {
        async fn get_questions(&self, limit: Option<i32>, offset: i32) -> Result<Vec<Question>, handle_errors::Error>;
        async fn is_question_owner(&self, question_id: QuestionId, account_id: &AccountId) -> Result<bool, handle_errors::Error>;
        async fn add_question(&self, new_question: NewQuestion, account_id: AccountId) -> Result<Question, handle_errors::Error>;
        async fn update_question(&self, question: Question, id: QuestionId, account_id: AccountId) -> Result<Question, handle_errors::Error>;
        async fn delete_question(&self, id: QuestionId, account_id: AccountId) -> Result<bool, handle_errors::Error>;
        async fn get_answers(&self, question_id: QuestionId, limit: Option<i32>, offset: i32) -> Result<Vec<Answer>, handle_errors::Error>;
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
async fn test_get_questions_success() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    store.expect_get_questions()
        .with(eq(None), eq(0))
        .times(1)
        .returning(|_, _| Ok(vec![Question {
            id: QuestionId(1),
            title: "Test Question".to_string(),
            content: "Test Content".to_string(),
            tags: Some(vec!["test".to_string()]),
        }]));
    
    let params = HashMap::new();
    let result = super::get_questions(params, store).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_question_success() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    let question = Question {
        id: QuestionId(1),
        title: "Updated Title".to_string(),
        content: "Updated Content".to_string(),
        tags: Some(vec!["updated".to_string()]),
    };
    
    store.expect_is_question_owner()
        .with(eq(QuestionId(1)), eq(&AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(true));
        
    store.expect_update_question()
        .with(eq(question.clone()), eq(QuestionId(1)), eq(AccountId(1)))
        .times(1)
        .returning(|q, _, _| Ok(q));
    
    let result = super::update_question(QuestionId(1), session, store, question).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_question_success() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    store.expect_is_question_owner()
        .with(eq(QuestionId(1)), eq(&AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(true));
        
    store.expect_delete_question()
        .with(eq(QuestionId(1)), eq(AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(true));
    
    let result = super::delete_question(QuestionId(1), session, store).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_question_success() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    let new_question = NewQuestion {
        title: "New Question".to_string(),
        content: "New Content".to_string(),
        tags: Some(vec!["new".to_string()]),
    };
    
    store.expect_add_question()
        .with(eq(new_question.clone()), eq(AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(Question {
            id: QuestionId(1),
            title: "New Question".to_string(),
            content: "New Content".to_string(),
            tags: Some(vec!["new".to_string()]),
        }));
    
    let result = super::add_question(session, store, new_question).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_answers_success() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    store.expect_get_answers()
        .with(eq(QuestionId(1)), eq(None), eq(0))
        .times(1)
        .returning(|_, _, _| Ok(vec![Answer {
            id: AnswerId(1),
            content: "Test Answer".to_string(),
            question_id: QuestionId(1),
        }]));
    
    let params = HashMap::new();
    let result = super::get_answers(QuestionId(1), params, store).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_question_unauthorized() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    let question = Question {
        id: QuestionId(1),
        title: "Updated Title".to_string(),
        content: "Updated Content".to_string(),
        tags: Some(vec!["updated".to_string()]),
    };
    
    store.expect_is_question_owner()
        .with(eq(QuestionId(1)), eq(&AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(false));
    
    let result = super::update_question(QuestionId(1), session, store, question).await;
    assert!(result.is_err());
    match result {
        Err(rejection) => {
            let error = rejection.find::<handle_errors::Error>().unwrap();
            assert!(matches!(*error, handle_errors::Error::Unauthorized));
        }
        _ => panic!("Expected unauthorized error"),
    }
}

#[tokio::test]
async fn test_delete_question_unauthorized() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    let session = create_test_session();
    
    store.expect_is_question_owner()
        .with(eq(QuestionId(1)), eq(&AccountId(1)))
        .times(1)
        .returning(|_, _| Ok(false));
    
    let result = super::delete_question(QuestionId(1), session, store).await;
    assert!(result.is_err());
    match result {
        Err(rejection) => {
            let error = rejection.find::<handle_errors::Error>().unwrap();
            assert!(matches!(*error, handle_errors::Error::Unauthorized));
        }
        _ => panic!("Expected unauthorized error"),
    }
}

#[tokio::test]
async fn test_get_questions_with_pagination() {
    let mock_store = setup_mock_store();
    let mut store = mock_store.lock().unwrap().clone();
    
    store.expect_get_questions()
        .with(eq(Some(5)), eq(10))
        .times(1)
        .returning(|_, _| Ok(vec![Question {
            id: QuestionId(1),
            title: "Test Question".to_string(),
            content: "Test Content".to_string(),
            tags: Some(vec!["test".to_string()]),
        }]));
    
    let mut params = HashMap::new();
    params.insert("limit".to_string(), "5".to_string());
    params.insert("offset".to_string(), "10".to_string());
    
    let result = super::get_questions(params, store).await;
    assert!(result.is_ok());
} 