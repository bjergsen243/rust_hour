use async_trait::async_trait;
use crate::types::account::AccountId;
use crate::types::answer::{Answer, NewAnswer};
use crate::handle_errors;

#[async_trait]
pub trait StoreTrait: Clone {
    async fn add_answer(&self, new_answer: NewAnswer, account_id: AccountId) -> Result<Answer, handle_errors::Error>;
    async fn is_answer_owner(&self, answer_id: i32, account_id: &AccountId) -> Result<bool, handle_errors::Error>;
    async fn update_answer(&self, answer: Answer, id: i32, account_id: AccountId) -> Result<Answer, handle_errors::Error>;
    async fn delete_answer(&self, id: i32, account_id: AccountId) -> Result<bool, handle_errors::Error>;
} 