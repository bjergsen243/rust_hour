use async_trait::async_trait;
use std::fmt::Debug;
use crate::types::account::AccountId;
use crate::types::question::{Question, NewQuestion, QuestionId};
use crate::types::answer::Answer;
use crate::handle_errors;

#[async_trait]
pub trait StoreTrait: Clone + Debug {
    async fn get_questions(&self, limit: Option<i32>, offset: i32) -> Result<Vec<Question>, handle_errors::Error>;
    async fn is_question_owner(&self, question_id: QuestionId, account_id: &AccountId) -> Result<bool, handle_errors::Error>;
    async fn add_question(&self, new_question: NewQuestion, account_id: AccountId) -> Result<Question, handle_errors::Error>;
    async fn update_question(&self, question: Question, id: QuestionId, account_id: AccountId) -> Result<Question, handle_errors::Error>;
    async fn delete_question(&self, id: QuestionId, account_id: AccountId) -> Result<bool, handle_errors::Error>;
    async fn get_answers(&self, question_id: QuestionId, limit: Option<i32>, offset: i32) -> Result<Vec<Answer>, handle_errors::Error>;
} 