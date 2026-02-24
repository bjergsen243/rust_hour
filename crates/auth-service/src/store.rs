use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use rust_hour_common::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthStore: Send + Sync {
    async fn create_account(&self, email: &str, password_hash: &str) -> Result<Account, AppError>;
    async fn get_account_by_email(&self, email: &str) -> Result<Account, AppError>;
    async fn get_account_by_id(&self, id: Uuid) -> Result<Account, AppError>;
    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<(), AppError>;
    async fn mark_deleting(&self, id: Uuid) -> Result<(), AppError>;
    async fn revert_deleting(&self, id: Uuid) -> Result<(), AppError>;
    async fn permanently_delete(&self, id: Uuid) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct PgAuthStore {
    pool: PgPool,
}

impl PgAuthStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthStore for PgAuthStore {
    async fn create_account(&self, email: &str, password_hash: &str) -> Result<Account, AppError> {
        let account = sqlx::query_as::<_, Account>(
            "INSERT INTO accounts (email, password) VALUES ($1, $2) RETURNING *",
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(account)
    }

    async fn get_account_by_email(&self, email: &str) -> Result<Account, AppError> {
        let account = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts WHERE email = $1 AND status = 'active'",
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(account)
    }

    async fn get_account_by_id(&self, id: Uuid) -> Result<Account, AppError> {
        let account =
            sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = $1 AND status = 'active'")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(account)
    }

    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE accounts SET password = $1, updated_at = NOW() WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn mark_deleting(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE accounts SET status = 'deleting', updated_at = NOW() WHERE id = $1 AND status = 'active'",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Account not found or not active".into()));
        }
        Ok(())
    }

    async fn revert_deleting(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE accounts SET status = 'active', updated_at = NOW() WHERE id = $1 AND status = 'deleting'",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "Account not found or not in deleting state".into(),
            ));
        }
        Ok(())
    }

    async fn permanently_delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM accounts WHERE id = $1 AND status = 'deleting'")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "Account not found or not in deleting state".into(),
            ));
        }
        Ok(())
    }
}
