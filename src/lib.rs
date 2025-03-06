#![warn(clippy::all)]

pub use handle_errors;
use tokio::sync::oneshot::Sender;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Method, Filter, Reply};

pub mod config;
mod routes;
mod store;
pub mod types;

pub struct OneshotHandler {
    pub sender: Sender<i32>,
}

async fn build_routes<T>(store: T) -> impl Filter<Extract = impl Reply> + Clone 
where 
    T: routes::question::store_trait::StoreTrait 
        + routes::answer::store_trait::StoreTrait 
        + routes::authentication::StoreTrait 
        + Clone 
        + Send 
        + Sync 
        + 'static
{
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::question::get_questions);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .map(|id| types::question::QuestionId(id))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .map(|id| types::question::QuestionId(id))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::add_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::answer::add_answer);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::login);

    let update_password = warp::put()
        .and(warp::path("accounts"))
        .and(warp::path("update_password"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::update_password);

    let update_account = warp::put()
        .and(warp::path("accounts"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::update_account);

    let get_account_information = warp::get()
        .and(warp::path("accounts"))
        .and(warp::path("me"))
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::authentication::get_account_information);

    let get_answers = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .map(|id| types::question::QuestionId(id))
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::question::get_answers);

    let update_answer = warp::put()
        .and(warp::path("answers"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::answer::update_answer);

    let delete_answer = warp::delete()
        .and(warp::path("answers"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::answer::delete_answer);

    get_questions
        .or(update_question)
        .or(add_question)
        .or(delete_question)
        .or(add_answer)
        .or(registration)
        .or(login)
        .or(update_password)
        .or(update_account)
        .or(get_account_information)
        .or(get_answers)
        .or(update_answer)
        .or(delete_answer)
        .with(cors)
        .with(warp::trace::request())
        .recover(handle_errors::return_error)
}

pub async fn setup_store(config: &config::Config) -> Result<store::Store, handle_errors::Error> {
    let store = store::Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_user, config.db_password, config.db_host, config.db_port, config.db_name
    ))
    .await
    .map_err(handle_errors::Error::DatabaseQueryError)?;

    sqlx::migrate!()
        .run(&store.clone().connection)
        .await
        .map_err(handle_errors::Error::MigrationError)?;

    let log_filter = format!(
        "handle_errors={},rust_hour={},warp={}",
        config.log_level, config.log_level, config.log_level
    );

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    Ok(store)
}

pub async fn run(config: config::Config, store: store::Store) {
    let routes = build_routes(store).await;
    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use warp::http::StatusCode;
    use warp::test::request;
    use mockall::predicate::*;
    use mockall::*;
    use crate::routes::question::store_trait::StoreTrait as QuestionStoreTrait;
    use crate::routes::answer::store_trait::StoreTrait as AnswerStoreTrait;
    use crate::routes::authentication::StoreTrait as AuthStoreTrait;
    use crate::types::question::{Question, QuestionId, NewQuestion};
    use crate::types::account::{AccountId, Account, AccountUpdateRequest, AccountUpdatePassword, AccountResponse};
    use crate::types::answer::{Answer, AnswerId, NewAnswer};
    use async_trait::async_trait;
    use crate::types::pagination::Pagination;

    mock! {
        #[derive(Debug)]
        Store {}

        #[async_trait]
        impl QuestionStoreTrait for Store {
            async fn get_questions(&self, limit: Option<i32>, offset: i32) -> Result<Vec<Question>, handle_errors::Error>;
            async fn is_question_owner(&self, question_id: QuestionId, account_id: &AccountId) -> Result<bool, handle_errors::Error>;
            async fn add_question(&self, new_question: NewQuestion, account_id: AccountId) -> Result<Question, handle_errors::Error>;
            async fn update_question(&self, question: Question, id: QuestionId, account_id: AccountId) -> Result<Question, handle_errors::Error>;
            async fn delete_question(&self, id: QuestionId, account_id: AccountId) -> Result<bool, handle_errors::Error>;
            async fn get_answers(&self, question_id: QuestionId, limit: Option<i32>, offset: i32) -> Result<Vec<Answer>, handle_errors::Error>;
        }

        #[async_trait]
        impl AnswerStoreTrait for Store {
            async fn add_answer(&self, new_answer: NewAnswer, account_id: AccountId) -> Result<Answer, handle_errors::Error>;
            async fn is_answer_owner(&self, answer_id: i32, account_id: &AccountId) -> Result<bool, handle_errors::Error>;
            async fn update_answer(&self, answer: Answer, id: i32, account_id: AccountId) -> Result<Answer, handle_errors::Error>;
            async fn delete_answer(&self, id: i32, account_id: AccountId) -> Result<bool, handle_errors::Error>;
        }

        #[async_trait]
        impl AuthStoreTrait for Store {
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

    #[derive(Debug, Clone)]
    struct Store;

    #[async_trait::async_trait]
    impl QuestionStoreTrait for Store {
        async fn get_questions(
            &self,
            _limit: Option<i32>,
            _offset: i32,
        ) -> Result<Vec<Question>, handle_errors::Error> {
            Ok(vec![])
        }

        async fn is_question_owner(
            &self,
            _question_id: QuestionId,
            _account_id: &AccountId,
        ) -> Result<bool, handle_errors::Error> {
            Ok(true)
        }

        async fn add_question(
            &self,
            _new_question: NewQuestion,
            _account_id: AccountId,
        ) -> Result<Question, handle_errors::Error> {
            Ok(Question {
                id: QuestionId(1),
                title: "Test Question".to_string(),
                content: "Test Content".to_string(),
                tags: Some(vec!["test".to_string()]),
            })
        }

        async fn update_question(
            &self,
            question: Question,
            _id: QuestionId,
            _account_id: AccountId,
        ) -> Result<Question, handle_errors::Error> {
            Ok(question)
        }

        async fn delete_question(
            &self,
            _id: QuestionId,
            _account_id: AccountId,
        ) -> Result<bool, handle_errors::Error> {
            Ok(true)
        }

        async fn get_answers(
            &self,
            _question_id: QuestionId,
            _limit: Option<i32>,
            _offset: i32,
        ) -> Result<Vec<Answer>, handle_errors::Error> {
            Ok(vec![])
        }
    }

    #[async_trait::async_trait]
    impl AnswerStoreTrait for Store {
        async fn add_answer(
            &self,
            new_answer: NewAnswer,
            _account_id: AccountId,
        ) -> Result<Answer, handle_errors::Error> {
            Ok(Answer {
                id: AnswerId(1),
                content: new_answer.content,
                question_id: new_answer.question_id,
            })
        }

        async fn is_answer_owner(
            &self,
            _answer_id: i32,
            _account_id: &AccountId,
        ) -> Result<bool, handle_errors::Error> {
            Ok(true)
        }

        async fn update_answer(
            &self,
            answer: Answer,
            _id: i32,
            _account_id: AccountId,
        ) -> Result<Answer, handle_errors::Error> {
            Ok(answer)
        }

        async fn delete_answer(
            &self,
            _id: i32,
            _account_id: AccountId,
        ) -> Result<bool, handle_errors::Error> {
            Ok(true)
        }
    }

    #[async_trait::async_trait]
    impl AuthStoreTrait for Store {
        async fn add_account(
            &self,
            _account: Account,
        ) -> Result<bool, handle_errors::Error> {
            Ok(true)
        }

        async fn get_account(
            &self,
            _email: String,
        ) -> Result<Account, handle_errors::Error> {
            Ok(Account {
                id: Some(AccountId(1)),
                email: "test@test.com".to_string(),
                password: "password".to_string(),
            })
        }

        async fn update_account(
            &self,
            _account_id: AccountId,
            _account: AccountUpdateRequest,
        ) -> Result<AccountResponse, handle_errors::Error> {
            Ok(AccountResponse {
                id: AccountId(1),
                email: "updated@test.com".to_string(),
            })
        }

        async fn update_password(
            &self,
            _account_id: AccountId,
            _password: AccountUpdatePassword,
        ) -> Result<bool, handle_errors::Error> {
            Ok(true)
        }

        async fn get_account_information(
            &self,
            _account_id: AccountId,
        ) -> Result<AccountResponse, handle_errors::Error> {
            Ok(AccountResponse {
                id: AccountId(1),
                email: "test@test.com".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn test_build_routes() {
        let store = Store;
        let _routes = build_routes(store).await;
        // If we got here without panicking, the routes were built successfully
    }

    #[tokio::test]
    async fn test_setup_store_invalid_config() {
        let result = setup_store(&Config {
            db_user: "invalid".to_string(),
            db_password: "invalid".to_string(),
            db_host: "invalid".to_string(),
            db_port: 5432,
            db_name: "invalid".to_string(),
            port: 8080,
            log_level: "info".to_string(),
        })
        .await;
        assert!(result.is_err());
    }
}
