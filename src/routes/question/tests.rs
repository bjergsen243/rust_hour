use mockall::predicate::*;
use mockall::*;
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::http::StatusCode;
use warp::test::request;
use warp::Filter;

use crate::handle_errors;
use crate::types::account::{AccountId, Session};
use crate::types::question::{Question, QuestionId, NewQuestion};
use crate::types::answer::{Answer, AnswerId};
use super::store_trait::StoreTrait;
use crate::types::pagination::Pagination;
use crate::store::Store;

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

/*
#[tokio::test]
async fn test_get_questions_success() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_get_questions()
            .with(eq(None), eq(0))
            .times(1)
            .returning(|_, _| Ok(vec![Question {
                id: QuestionId(1),
                title: "test".to_string(),
                content: "test".to_string(),
                tags: Some(vec!["test".to_string()]),
            }]));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let api = warp::path("questions")
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter)
        .and_then(super::get_questions);
    
    let response = request()
        .method("GET")
        .path("/questions")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_questions_with_pagination() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_get_questions()
            .with(eq(Some(10)), eq(0))
            .times(1)
            .returning(|_, _| Ok(vec![Question {
                id: QuestionId(1),
                title: "test".to_string(),
                content: "test".to_string(),
                tags: Some(vec!["test".to_string()]),
            }]));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let api = warp::path("questions")
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter)
        .and_then(super::get_questions);
    
    let response = request()
        .method("GET")
        .path("/questions?limit=10&offset=0")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_questions_empty_result() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_get_questions()
            .with(eq(None), eq(0))
            .times(1)
            .returning(|_, _| Ok(vec![]));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let api = warp::path("questions")
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter)
        .and_then(super::get_questions);
    
    let response = request()
        .method("GET")
        .path("/questions")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_questions_error() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_get_questions()
            .with(eq(None), eq(0))
            .times(1)
            .returning(|_, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let api = warp::path("questions")
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter)
        .and_then(super::get_questions);
    
    let response = request()
        .method("GET")
        .path("/questions")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_add_question() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_add_question()
            .with(eq(NewQuestion {
                title: "test".to_string(),
                content: "test".to_string(),
                tags: Some(vec!["test".to_string()]),
            }), eq(AccountId(1)))
            .times(1)
            .returning(|question, _| Ok(Question {
                id: QuestionId(1),
                title: question.title,
                content: question.content,
                tags: question.tags,
            }));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let session = create_test_session();
    let api = warp::path("questions")
        .and(warp::post())
        .and(warp::any().map(move || session.clone()))
        .and(store_filter)
        .and(warp::body::json())
        .and_then(super::add_question);
    
    let response = request()
        .method("POST")
        .path("/questions")
        .json(&NewQuestion {
            title: "test".to_string(),
            content: "test".to_string(),
            tags: Some(vec!["test".to_string()]),
        })
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_add_question_error() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_add_question()
            .with(eq(NewQuestion {
                title: "test".to_string(),
                content: "test".to_string(),
                tags: Some(vec!["test".to_string()]),
            }), eq(AccountId(1)))
            .times(1)
            .returning(|_, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let session = create_test_session();
    let api = warp::path("questions")
        .and(warp::post())
        .and(warp::any().map(move || session.clone()))
        .and(store_filter)
        .and(warp::body::json())
        .and_then(super::add_question);
    
    let response = request()
        .method("POST")
        .path("/questions")
        .json(&NewQuestion {
            title: "test".to_string(),
            content: "test".to_string(),
            tags: Some(vec!["test".to_string()]),
        })
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_update_question() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_is_question_owner()
            .with(eq(QuestionId(1)), eq(&AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(true));
        
        mock.expect_update_question()
            .with(eq(Question {
                id: QuestionId(1),
                title: "updated".to_string(),
                content: "updated".to_string(),
                tags: Some(vec!["updated".to_string()]),
            }), eq(QuestionId(1)), eq(AccountId(1)))
            .times(1)
            .returning(|question, _, _| Ok(question));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let session = create_test_session();
    let api = warp::path!("questions" / i32)
        .and(warp::put())
        .and(warp::any().map(move || session.clone()))
        .and(store_filter)
        .and(warp::body::json())
        .map(|id, session, store, question| (QuestionId(id), session, store, question))
        .and_then(|(id, session, store, question)| super::update_question(id, session, store, question));
    
    let response = request()
        .method("PUT")
        .path("/questions/1")
        .json(&Question {
            id: QuestionId(1),
            title: "updated".to_string(),
            content: "updated".to_string(),
            tags: Some(vec!["updated".to_string()]),
        })
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_update_question_unauthorized() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_is_question_owner()
            .with(eq(QuestionId(1)), eq(&AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(false));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let session = create_test_session();
    let api = warp::path!("questions" / i32)
        .and(warp::put())
        .and(warp::any().map(move || session.clone()))
        .and(store_filter)
        .and(warp::body::json())
        .map(|id, session, store, question| (QuestionId(id), session, store, question))
        .and_then(|(id, session, store, question)| super::update_question(id, session, store, question));
    
    let response = request()
        .method("PUT")
        .path("/questions/1")
        .json(&Question {
            id: QuestionId(1),
            title: "updated".to_string(),
            content: "updated".to_string(),
            tags: Some(vec!["updated".to_string()]),
        })
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_question_database_error() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_is_question_owner()
            .with(eq(QuestionId(1)), eq(&AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(true));
        
        mock.expect_update_question()
            .with(eq(Question {
                id: QuestionId(1),
                title: "updated".to_string(),
                content: "updated".to_string(),
                tags: Some(vec!["updated".to_string()]),
            }), eq(QuestionId(1)), eq(AccountId(1)))
            .times(1)
            .returning(|_, _, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let session = create_test_session();
    let api = warp::path!("questions" / i32)
        .and(warp::put())
        .and(warp::any().map(move || session.clone()))
        .and(store_filter)
        .and(warp::body::json())
        .map(|id, session, store, question| (QuestionId(id), session, store, question))
        .and_then(|(id, session, store, question)| super::update_question(id, session, store, question));
    
    let response = request()
        .method("PUT")
        .path("/questions/1")
        .json(&Question {
            id: QuestionId(1),
            title: "updated".to_string(),
            content: "updated".to_string(),
            tags: Some(vec!["updated".to_string()]),
        })
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_delete_question() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_is_question_owner()
            .with(eq(QuestionId(1)), eq(&AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(true));
        
        mock.expect_delete_question()
            .with(eq(QuestionId(1)), eq(AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(true));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let session = create_test_session();
    let api = warp::path!("questions" / i32)
        .and(warp::delete())
        .and(warp::any().map(move || session.clone()))
        .and(store_filter)
        .map(|id, session, store| (QuestionId(id), session, store))
        .and_then(|(id, session, store)| super::delete_question(id, session, store));
    
    let response = request()
        .method("DELETE")
        .path("/questions/1")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_delete_question_unauthorized() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_is_question_owner()
            .with(eq(QuestionId(1)), eq(&AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(false));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let session = create_test_session();
    let api = warp::path!("questions" / i32)
        .and(warp::delete())
        .and(warp::any().map(move || session.clone()))
        .and(store_filter)
        .map(|id, session, store| (QuestionId(id), session, store))
        .and_then(|(id, session, store)| super::delete_question(id, session, store));
    
    let response = request()
        .method("DELETE")
        .path("/questions/1")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_question_database_error() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_is_question_owner()
            .with(eq(QuestionId(1)), eq(&AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(true));
        
        mock.expect_delete_question()
            .with(eq(QuestionId(1)), eq(AccountId(1)))
            .times(1)
            .returning(|_, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let session = create_test_session();
    let api = warp::path!("questions" / i32)
        .and(warp::delete())
        .and(warp::any().map(move || session.clone()))
        .and(store_filter)
        .map(|id, session, store| (QuestionId(id), session, store))
        .and_then(|(id, session, store)| super::delete_question(id, session, store));
    
    let response = request()
        .method("DELETE")
        .path("/questions/1")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_get_answers_success() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_get_answers()
            .with(eq(QuestionId(1)), eq(None), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(vec![Answer {
                id: AnswerId(1),
                content: "test".to_string(),
                question_id: QuestionId(1),
            }]));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let api = warp::path!("questions" / i32 / "answers")
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter)
        .map(|id, params, store| (QuestionId(id), params, store))
        .and_then(|(id, params, store)| super::get_answers(id, params, store));
    
    let response = request()
        .method("GET")
        .path("/questions/1/answers")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_answers_with_pagination() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_get_answers()
            .with(eq(QuestionId(1)), eq(Some(10)), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(vec![Answer {
                id: AnswerId(1),
                content: "test".to_string(),
                question_id: QuestionId(1),
            }]));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let api = warp::path!("questions" / i32 / "answers")
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter)
        .map(|id, params, store| (QuestionId(id), params, store))
        .and_then(|(id, params, store)| super::get_answers(id, params, store));
    
    let response = request()
        .method("GET")
        .path("/questions/1/answers?limit=10&offset=0")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_answers_empty_result() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_get_answers()
            .with(eq(QuestionId(1)), eq(None), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(vec![]));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let api = warp::path!("questions" / i32 / "answers")
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter)
        .map(|id, params, store| (QuestionId(id), params, store))
        .and_then(|(id, params, store)| super::get_answers(id, params, store));
    
    let response = request()
        .method("GET")
        .path("/questions/1/answers")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_answers_error() {
    let mock_store = setup_mock_store();
    {
        let mut mock = mock_store.lock().unwrap();
        mock.expect_get_answers()
            .with(eq(QuestionId(1)), eq(None), eq(0))
            .times(1)
            .returning(|_, _, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
    }
    
    let store_filter = warp::any().map(move || mock_store.lock().unwrap().clone());
    let api = warp::path!("questions" / i32 / "answers")
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter)
        .map(|id, params, store| (QuestionId(id), params, store))
        .and_then(|(id, params, store)| super::get_answers(id, params, store));
    
    let response = request()
        .method("GET")
        .path("/questions/1/answers")
        .reply(&api)
        .await;
    
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
*/

// ... existing code ... 