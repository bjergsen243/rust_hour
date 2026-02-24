use async_trait::async_trait;
use rust_hour_common::error::AppError;

#[async_trait]
pub trait SagaStep: Send + Sync {
    fn name(&self) -> &str;

    /// Execute the forward action
    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError>;

    /// Execute the compensating (rollback) action
    async fn compensate(&self, context: &serde_json::Value) -> Result<(), AppError>;
}
