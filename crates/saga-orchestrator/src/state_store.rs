use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use rust_hour_common::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SagaInstance {
    pub id: Uuid,
    pub saga_type: String,
    pub account_id: Uuid,
    pub status: String,
    pub current_step: i32,
    pub context: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SagaStepLog {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub step_number: i32,
    pub step_name: String,
    pub status: String,
    pub command_payload: Option<serde_json::Value>,
    pub reply_payload: Option<serde_json::Value>,
    pub executed_at: Option<DateTime<Utc>>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SagaStateStore: Send + Sync {
    async fn create_saga(
        &self,
        saga_type: &str,
        account_id: Uuid,
    ) -> Result<SagaInstance, AppError>;

    async fn get_saga(&self, id: Uuid) -> Result<SagaInstance, AppError>;

    async fn update_saga_status(
        &self,
        id: Uuid,
        status: &str,
        step: i32,
        context: &serde_json::Value,
    ) -> Result<(), AppError>;

    async fn log_step(
        &self,
        saga_id: Uuid,
        step_number: i32,
        step_name: &str,
        status: &str,
        command: Option<&serde_json::Value>,
        reply: Option<&serde_json::Value>,
    ) -> Result<(), AppError>;

    async fn get_steps(&self, saga_id: Uuid) -> Result<Vec<SagaStepLog>, AppError>;
}

#[derive(Clone)]
pub struct PgSagaStateStore {
    pool: PgPool,
}

impl PgSagaStateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SagaStateStore for PgSagaStateStore {
    async fn create_saga(
        &self,
        saga_type: &str,
        account_id: Uuid,
    ) -> Result<SagaInstance, AppError> {
        let saga = sqlx::query_as::<_, SagaInstance>(
            r#"INSERT INTO saga_instances (saga_type, account_id, context)
               VALUES ($1, $2, $3) RETURNING *"#,
        )
        .bind(saga_type)
        .bind(account_id)
        .bind(serde_json::json!({"account_id": account_id.to_string()}))
        .fetch_one(&self.pool)
        .await?;

        Ok(saga)
    }

    async fn get_saga(&self, id: Uuid) -> Result<SagaInstance, AppError> {
        let saga =
            sqlx::query_as::<_, SagaInstance>("SELECT * FROM saga_instances WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(saga)
    }

    async fn update_saga_status(
        &self,
        id: Uuid,
        status: &str,
        step: i32,
        context: &serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE saga_instances SET status = $2, current_step = $3, context = $4, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .bind(step)
        .bind(context)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn log_step(
        &self,
        saga_id: Uuid,
        step_number: i32,
        step_name: &str,
        status: &str,
        command: Option<&serde_json::Value>,
        reply: Option<&serde_json::Value>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"INSERT INTO saga_step_log (saga_id, step_number, step_name, status, command_payload, reply_payload)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(saga_id)
        .bind(step_number)
        .bind(step_name)
        .bind(status)
        .bind(command)
        .bind(reply)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_steps(&self, saga_id: Uuid) -> Result<Vec<SagaStepLog>, AppError> {
        let steps = sqlx::query_as::<_, SagaStepLog>(
            "SELECT * FROM saga_step_log WHERE saga_id = $1 ORDER BY step_number",
        )
        .bind(saga_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(steps)
    }
}
