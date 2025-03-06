// use super::*;
// use sqlx::postgres::PgPoolOptions;
// use std::env;
// use crate::types::{
//     account::{Account, AccountId, AccountUpdateRequest, AccountUpdatePassword},
//     answer::{Answer, AnswerId, NewAnswer},
//     question::{Question, QuestionId, NewQuestion},
// };

// async fn setup_test_db() -> Store {
//     // Save current environment variables
//     let original_database_url = env::var("DATABASE_URL").ok();
    
//     // Set test environment variables for the default database first
//     let default_db_url = "postgres://postgres:postgres@db:5432/postgres";
    
//     // Create a connection to the default database
//     let pool = PgPoolOptions::new()
//         .max_connections(1)
//         .connect(default_db_url)
//         .await
//         .expect("Failed to connect to default database");
    
//     // Create the test database if it doesn't exist
//     let create_db_query = "CREATE DATABASE rust_hour_test";
//     match sqlx::query(create_db_query).execute(&pool).await {
//         Ok(_) => println!("Created test database"),
//         Err(e) => {
//             if !e.to_string().contains("already exists") {
//                 panic!("Failed to create test database: {}", e);
//             }
//         }
//     };
    
//     // Close the connection to the default database
//     drop(pool);
    
//     // Now connect to the test database
//     let test_db_url = "postgres://postgres:postgres@db:5432/rust_hour_test";
//     env::set_var("DATABASE_URL", test_db_url);
    
//     // Also set individual environment variables for the config
//     env::set_var("DB_USER", "postgres");
//     env::set_var("DB_PASSWORD", "postgres");
//     env::set_var("DB_HOST", "db");
//     env::set_var("DB_PORT", "5432");
//     env::set_var("DB_NAME", "rust_hour_test");
    
//     // Create store
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     let store = Store::new(&database_url).await.expect("Failed to create store");
    
//     // Run migrations on the test database
//     sqlx::migrate!("./migrations")
//         .run(&store.connection)
//         .await
//         .expect("Failed to run migrations");
    
//     // Restore original environment variables
//     match original_database_url {
//         Some(url) => env::set_var("DATABASE_URL", url),
//         None => env::remove_var("DATABASE_URL"),
//     }
    
//     store
// }

// #[tokio::test]
// async fn test_handle_error() {
//     let error = sqlx::Error::RowNotFound;
//     let result: Result<(), Error> = Store::handle_error(Err(error));
//     assert!(matches!(result, Err(Error::DatabaseQueryError(_))));
// }

// #[tokio::test]
// async fn test_check_ownership() {
//     let store = setup_test_db().await;
//     let account_id = AccountId(1);
    
//     // Test with questions table
//     let result = store.check_ownership("questions", 1, &account_id).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_get_questions() {
//     let store = setup_test_db().await;
//     let result = store.get_questions(Some(10), 0).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_add_question() {
//     let store = setup_test_db().await;
//     let new_question = NewQuestion {
//         title: "Test Question".to_string(),
//         content: "Test Content".to_string(),
//         tags: Some(vec!["test".to_string()]),
//     };
//     let account_id = AccountId(1);
    
//     let result = store.add_question(new_question, account_id).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_update_question() {
//     let store = setup_test_db().await;
//     let question = Question {
//         id: QuestionId(1),
//         title: "Updated Title".to_string(),
//         content: "Updated Content".to_string(),
//         tags: Some(vec!["updated".to_string()]),
//     };
//     let account_id = AccountId(1);
    
//     let result = store.update_question(question, 1, account_id).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_delete_question() {
//     let store = setup_test_db().await;
//     let account_id = AccountId(1);
    
//     let result = store.delete_question(1, account_id).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_get_answers() {
//     let store = setup_test_db().await;
//     let result = store.get_answers(1, Some(10), 0).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_add_answer() {
//     let store = setup_test_db().await;
//     let new_answer = NewAnswer {
//         content: "Test Answer".to_string(),
//         question_id: QuestionId(1),
//     };
//     let account_id = AccountId(1);
    
//     let result = store.add_answer(new_answer, account_id).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_update_answer() {
//     let store = setup_test_db().await;
//     let answer = Answer {
//         id: AnswerId(1),
//         content: "Updated Answer".to_string(),
//         question_id: QuestionId(1),
//     };
//     let account_id = AccountId(1);
    
//     let result = store.update_answer(answer, 1, account_id).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_delete_answer() {
//     let store = setup_test_db().await;
//     let account_id = AccountId(1);
    
//     let result = store.delete_answer(1, account_id).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_add_account() {
//     let store = setup_test_db().await;
//     let account = Account {
//         id: None,
//         email: "test@example.com".to_string(),
//         password: "password123".to_string(),
//     };
    
//     let result = store.add_account(account).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_get_account() {
//     let store = setup_test_db().await;
//     let result = store.get_account("test@example.com".to_string()).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_update_account() {
//     let store = setup_test_db().await;
//     let account_update = AccountUpdateRequest {
//         email: "updated@example.com".to_string(),
//     };
//     let account_id = AccountId(1);
    
//     let result = store.update_account(account_id, account_update).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_update_password() {
//     let store = setup_test_db().await;
//     let password_update = AccountUpdatePassword("newpassword123".to_string());
//     let account_id = AccountId(1);
    
//     let result = store.update_password(account_id, password_update).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_get_account_information() {
//     let store = setup_test_db().await;
//     let account_id = AccountId(1);
    
//     let result = store.get_account_information(account_id).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_is_answer_owner() {
//     let store = setup_test_db().await;
//     let account_id = AccountId(1);
    
//     let result = store.is_answer_owner(1, &account_id).await;
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn test_is_question_owner() {
//     let store = setup_test_db().await;
//     let account_id = AccountId(1);
    
//     let result = store.is_question_owner(1, &account_id).await;
//     assert!(result.is_ok());
// } 