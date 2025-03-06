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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::answer::{Answer, AnswerId, NewAnswer};
    use crate::types::question::QuestionId;
    use crate::types::account::{AccountId, Session};
    use chrono::prelude::*;
    use mockall::predicate::*;
    use mockall::*;
    use std::sync::{Arc, Mutex};
    use warp::http::StatusCode;
    use async_trait::async_trait;
    use crate::routes::answer::{add_answer, update_answer, delete_answer};

    mock! {
        Store {}

        #[async_trait]
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
    async fn test_add_answer_success() {
        let mock_store = setup_mock_store();
        let mut store = mock_store.lock().unwrap().clone();
        let session = create_test_session();
        
        let new_answer = NewAnswer {
            content: "Test answer".to_string(),
            question_id: QuestionId(1),
        };
        
        store.expect_add_answer()
            .with(predicate::function(|a: &NewAnswer| {
                a.content == "Test answer" && a.question_id == QuestionId(1)
            }), eq(AccountId(1)))
            .times(1)
            .returning(|a, _| Ok(Answer {
                id: AnswerId(1),
                content: a.content,
                question_id: a.question_id,
            }));
        
        let result = add_answer(session, store, new_answer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_answer_database_error() {
        let mock_store = setup_mock_store();
        let mut store = mock_store.lock().unwrap().clone();
        let session = create_test_session();
        
        let new_answer = NewAnswer {
            content: "Test answer".to_string(),
            question_id: QuestionId(1),
        };
        
        store.expect_add_answer()
            .with(predicate::always(), predicate::always())
            .times(1)
            .returning(|_, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
        
        let result = add_answer(session, store, new_answer).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_answer_not_owner() {
        let mock_store = setup_mock_store();
        let mut store = mock_store.lock().unwrap().clone();
        let session = create_test_session();
        
        let answer = Answer {
            id: AnswerId(1),
            content: "Updated answer".to_string(),
            question_id: QuestionId(1),
        };
        
        store.expect_is_answer_owner()
            .with(eq(1), eq(&AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(false));
        
        let result = update_answer(1, session, store, answer).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_answer_not_found() {
        let mock_store = setup_mock_store();
        let mut store = mock_store.lock().unwrap().clone();
        let session = create_test_session();
        
        store.expect_is_answer_owner()
            .with(eq(1), eq(&AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(true));
            
        store.expect_delete_answer()
            .with(eq(1), eq(AccountId(1)))
            .times(1)
            .returning(|_, _| Err(handle_errors::Error::DatabaseQueryError(sqlx::Error::RowNotFound)));
        
        let result = delete_answer(1, session, store).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_answer_success() {
        let mock_store = setup_mock_store();
        let mut store = mock_store.lock().unwrap().clone();
        let session = create_test_session();
        
        store.expect_is_answer_owner()
            .with(eq(1), eq(&AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(true));
            
        store.expect_delete_answer()
            .with(eq(1), eq(AccountId(1)))
            .times(1)
            .returning(|_, _| Ok(true));
        
        let result = delete_answer(1, session, store).await;
        assert!(result.is_ok());
    }
}