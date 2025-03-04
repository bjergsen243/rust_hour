use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow},
    Row,
};

use handle_errors::Error;

use crate::types::{
    account::{Account, AccountId, AccountResponse, AccountUpdatePassword, AccountUpdateRequest},
    answer::{Answer, AnswerId, NewAnswer},
    question::{NewQuestion, Question, QuestionId},
};
use crate::routes::answer::store_trait::StoreTrait as AnswerStoreTrait;
use crate::routes::question::store_trait::StoreTrait as QuestionStoreTrait;

/// Represents a persistent storage unit for your application.
///
/// This struct provides a connection pool to a PostgreSQL database (`PgPool`).
/// Use this struct to interact with and manage your application's data.
///
/// # Fields
///
/// * `connection`: A connection pool to a PostgreSQL database.
///
/// # Examples
#[derive(Debug, Clone)]
pub struct Store {
    pub connection: PgPool,
}

impl Store {
    /// Initializes a new `Store` instance with the provided database URL.
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        tracing::warn!("{}", db_url);
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        Ok(Store {
            connection: db_pool,
        })
    }

    /// Helper function to handle database errors consistently
    fn handle_error<T>(result: Result<T, sqlx::Error>) -> Result<T, Error> {
        result.map_err(|e| {
            tracing::event!(tracing::Level::ERROR, "{:?}", e);
            Error::DatabaseQueryError(e)
        })
    }

    /// Helper function to check ownership of a resource
    async fn check_ownership(
        &self,
        table: &str,
        id: i32,
        account_id: &AccountId,
    ) -> Result<bool, Error> {
        let query = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE id = $1 AND account_id = $2)",
            table
        );
        
        Self::handle_error(
            sqlx::query(&query)
                .bind(id)
                .bind(account_id.0)
                .fetch_one(&self.connection)
                .await
                .map(|row: PgRow| row.get(0))
        )
    }

    /// Retrieves a list of questions from the database with optional pagination.
    pub async fn get_questions(
        self,
        limit: Option<i32>,
        offset: i32,
    ) -> Result<Vec<Question>, Error> {
        Self::handle_error(
            sqlx::query("SELECT * from questions LIMIT $1 OFFSET $2")
                .bind(limit)
                .bind(offset)
                .map(|row: PgRow| Question {
                    id: QuestionId(row.get("id")),
                    title: row.get("title"),
                    content: row.get("content"),
                    tags: row.get("tags"),
                })
                .fetch_all(&self.connection)
                .await
        )
    }

    /// Determines whether the given account is the owner of the specified question.
    pub async fn is_question_owner(
        &self,
        question_id: i32,
        account_id: &AccountId,
    ) -> Result<bool, Error> {
        self.check_ownership("questions", question_id, account_id).await
    }

    /// Adds a new question to the database and returns the created question.
    pub async fn add_question(
        self,
        new_question: NewQuestion,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        Self::handle_error(
            sqlx::query(
                "INSERT INTO questions (title, content, tags, account_id) 
                VALUES ($1, $2, $3, $4) 
                RETURNING id, title, content, tags"
            )
            .bind(new_question.title)
            .bind(new_question.content)
            .bind(new_question.tags)
            .bind(account_id.0)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_one(&self.connection)
            .await
        )
    }

    /// Updates an existing question in the database and returns the updated question.
    pub async fn update_question(
        self,
        question: Question,
        id: i32,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        Self::handle_error(
            sqlx::query(
                "UPDATE questions 
                SET title = $1, content = $2, tags = $3
                WHERE id = $4 AND account_id = $5
                RETURNING id, title, content, tags"
            )
            .bind(question.title)
            .bind(question.content)
            .bind(question.tags)
            .bind(id)
            .bind(account_id.0)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_one(&self.connection)
            .await
        )
    }

    /// Delete an existing question in the database
    pub async fn delete_question(self, id: i32, account_id: AccountId) -> Result<bool, Error> {
        Self::handle_error(
            sqlx::query("DELETE FROM questions WHERE id = $1 AND account_id = $2")
                .bind(id)
                .bind(account_id.0)
                .execute(&self.connection)
                .await
                .map(|_| true)
        )
    }

    /// Retrieves a list of answers for a question from the database with optional pagination.
    pub async fn get_answers(
        self,
        question_id: i32,
        limit: Option<i32>,
        offset: i32,
    ) -> Result<Vec<Answer>, Error> {
        Self::handle_error(
            sqlx::query(
                "SELECT * FROM answers 
                WHERE corresponding_question = $1 
                LIMIT $2 OFFSET $3"
            )
            .bind(question_id)
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| Answer {
                id: AnswerId(row.get("id")),
                content: row.get("content"),
                question_id: QuestionId(row.get("corresponding_question")),
            })
            .fetch_all(&self.connection)
            .await
        )
    }

    /// Adds a new account to the database
    pub async fn add_account(self, account: Account) -> Result<bool, Error> {
        Self::handle_error(
            sqlx::query("INSERT INTO accounts (email, password) VALUES ($1, $2)")
                .bind(account.email)
                .bind(account.password)
                .execute(&self.connection)
                .await
                .map(|_| true)
        )
    }

    /// Retrieves an account by email
    pub async fn get_account(self, email: String) -> Result<Account, Error> {
        Self::handle_error(
            sqlx::query("SELECT * from accounts where email = $1")
                .bind(email)
                .map(|row: PgRow| Account {
                    id: Some(AccountId(row.get("id"))),
                    email: row.get("email"),
                    password: row.get("password"),
                })
                .fetch_one(&self.connection)
                .await
        )
    }

    /// Updates email of an existing account
    pub async fn update_account(
        self,
        account_id: AccountId,
        account: AccountUpdateRequest,
    ) -> Result<AccountResponse, Error> {
        Self::handle_error(
            sqlx::query(
                "UPDATE accounts 
                SET email = $1
                WHERE id = $2
                RETURNING email, id"
            )
            .bind(account.email)
            .bind(account_id.0)
            .map(|row: PgRow| AccountResponse {
                email: row.get("email"),
                id: AccountId(row.get("id")),
            })
            .fetch_one(&self.connection)
            .await
        )
    }

    /// Updates password of an existing account
    pub async fn update_password(
        self,
        account_id: AccountId,
        password: AccountUpdatePassword,
    ) -> Result<bool, Error> {
        Self::handle_error(
            sqlx::query("UPDATE accounts SET password = $1 WHERE id = $2")
                .bind(password.0)
                .bind(account_id.0)
                .execute(&self.connection)
                .await
                .map(|_| true)
        )
    }

    /// Retrieves account information
    pub async fn get_account_information(
        self,
        account_id: AccountId,
    ) -> Result<AccountResponse, Error> {
        Self::handle_error(
            sqlx::query("SELECT email, id FROM accounts WHERE id = $1")
                .bind(account_id.0)
                .map(|row: PgRow| AccountResponse {
                    email: row.get("email"),
                    id: AccountId(row.get("id")),
                })
                .fetch_one(&self.connection)
                .await
        )
    }
}

#[async_trait::async_trait]
impl QuestionStoreTrait for Store {
    async fn get_questions(
        &self,
        limit: Option<i32>,
        offset: i32,
    ) -> Result<Vec<Question>, Error> {
        Self::handle_error(
            sqlx::query(
                "SELECT * FROM questions 
                LIMIT $1 OFFSET $2"
            )
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_all(&self.connection)
            .await
        )
    }

    async fn is_question_owner(
        &self,
        question_id: QuestionId,
        account_id: &AccountId,
    ) -> Result<bool, Error> {
        self.check_ownership("questions", question_id.0, account_id).await
    }

    async fn add_question(
        &self,
        new_question: NewQuestion,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        Self::handle_error(
            sqlx::query(
                "INSERT INTO questions (title, content, tags, account_id) 
                VALUES ($1, $2, $3, $4) 
                RETURNING id, title, content, tags"
            )
            .bind(new_question.title)
            .bind(new_question.content)
            .bind(new_question.tags)
            .bind(account_id.0)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_one(&self.connection)
            .await
        )
    }

    async fn update_question(
        &self,
        question: Question,
        id: QuestionId,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        Self::handle_error(
            sqlx::query(
                "UPDATE questions 
                SET title = $1, content = $2, tags = $3
                WHERE id = $4 AND account_id = $5
                RETURNING id, title, content, tags"
            )
            .bind(question.title)
            .bind(question.content)
            .bind(question.tags)
            .bind(id.0)
            .bind(account_id.0)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_one(&self.connection)
            .await
        )
    }

    async fn delete_question(&self, id: QuestionId, account_id: AccountId) -> Result<bool, Error> {
        Self::handle_error(
            sqlx::query("DELETE FROM questions WHERE id = $1 AND account_id = $2")
                .bind(id.0)
                .bind(account_id.0)
                .execute(&self.connection)
                .await
                .map(|_| true)
        )
    }

    async fn get_answers(
        &self,
        question_id: QuestionId,
        limit: Option<i32>,
        offset: i32,
    ) -> Result<Vec<Answer>, Error> {
        Self::handle_error(
            sqlx::query(
                "SELECT * FROM answers 
                WHERE corresponding_question = $1 
                LIMIT $2 OFFSET $3"
            )
            .bind(question_id.0)
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| Answer {
                id: AnswerId(row.get("id")),
                content: row.get("content"),
                question_id: QuestionId(row.get("corresponding_question")),
            })
            .fetch_all(&self.connection)
            .await
        )
    }
}

#[async_trait::async_trait]
impl AnswerStoreTrait for Store {
    async fn add_answer(&self, new_answer: NewAnswer, account_id: AccountId) -> Result<Answer, Error> {
        Self::handle_error(
            sqlx::query(
                "INSERT INTO answers (content, corresponding_question, account_id) 
                VALUES ($1, $2, $3) 
                RETURNING id, content, corresponding_question"
            )
            .bind(new_answer.content)
            .bind(new_answer.question_id.0)
            .bind(account_id.0)
            .map(|row: PgRow| Answer {
                id: AnswerId(row.get("id")),
                content: row.get("content"),
                question_id: QuestionId(row.get("corresponding_question")),
            })
            .fetch_one(&self.connection)
            .await
        )
    }

    async fn is_answer_owner(&self, answer_id: i32, account_id: &AccountId) -> Result<bool, Error> {
        self.check_ownership("answers", answer_id, account_id).await
    }

    async fn update_answer(&self, answer: Answer, id: i32, account_id: AccountId) -> Result<Answer, Error> {
        Self::handle_error(
            sqlx::query(
                "UPDATE answers 
                SET content = $1, corresponding_question = $2
                WHERE id = $3 AND account_id = $4
                RETURNING id, content, corresponding_question"
            )
            .bind(answer.content)
            .bind(answer.question_id.0)
            .bind(id)
            .bind(account_id.0)
            .map(|row: PgRow| Answer {
                id: AnswerId(row.get("id")),
                content: row.get("content"),
                question_id: QuestionId(row.get("corresponding_question")),
            })
            .fetch_one(&self.connection)
            .await
        )
    }

    async fn delete_answer(&self, id: i32, account_id: AccountId) -> Result<bool, Error> {
        Self::handle_error(
            sqlx::query("DELETE FROM answers WHERE id = $1 AND account_id = $2")
                .bind(id)
                .bind(account_id.0)
                .execute(&self.connection)
                .await
                .map(|_| true)
        )
    }
}
