use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use rust_hour_common::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AlertRule {
    pub id: Uuid,
    pub account_id: Uuid,
    pub endpoint_id: Uuid,
    pub rule_type: String,
    pub threshold_value: i32,
    pub consecutive_failures: i32,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AlertChannel {
    pub id: Uuid,
    pub account_id: Uuid,
    pub channel_type: String,
    pub config: serde_json::Value,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AlertHistoryEntry {
    pub id: Uuid,
    pub alert_rule_id: Uuid,
    pub channel_id: Uuid,
    pub status: String,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AlertStore: Send + Sync {
    async fn create_rule(
        &self,
        account_id: Uuid,
        endpoint_id: Uuid,
        rule_type: &str,
        threshold_value: i32,
        consecutive_failures: i32,
    ) -> Result<AlertRule, AppError>;

    async fn list_rules(
        &self,
        account_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<AlertRule>, i64), AppError>;

    async fn update_rule(
        &self,
        id: Uuid,
        account_id: Uuid,
        rule_type: Option<&str>,
        threshold_value: Option<i32>,
        consecutive_failures: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<AlertRule, AppError>;

    async fn delete_rule(&self, id: Uuid, account_id: Uuid) -> Result<(), AppError>;

    async fn get_rules_for_endpoint(&self, endpoint_id: Uuid) -> Result<Vec<AlertRule>, AppError>;

    async fn create_channel(
        &self,
        account_id: Uuid,
        channel_type: &str,
        config: &serde_json::Value,
    ) -> Result<AlertChannel, AppError>;

    async fn list_channels(
        &self,
        account_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<AlertChannel>, i64), AppError>;

    async fn delete_channel(&self, id: Uuid, account_id: Uuid) -> Result<(), AppError>;

    async fn insert_alert_event(
        &self,
        rule_id: Uuid,
        channel_id: Uuid,
        status: &str,
        message: &str,
    ) -> Result<AlertHistoryEntry, AppError>;

    async fn get_history(
        &self,
        account_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<AlertHistoryEntry>, i64), AppError>;

    // Saga support
    async fn soft_delete_all(&self, account_id: Uuid) -> Result<(i64, i64), AppError>;
    async fn restore_all(&self, account_id: Uuid) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct PgAlertStore {
    pool: PgPool,
}

impl PgAlertStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AlertStore for PgAlertStore {
    async fn create_rule(
        &self,
        account_id: Uuid,
        endpoint_id: Uuid,
        rule_type: &str,
        threshold_value: i32,
        consecutive_failures: i32,
    ) -> Result<AlertRule, AppError> {
        let rule = sqlx::query_as::<_, AlertRule>(
            r#"INSERT INTO alert_rules (account_id, endpoint_id, rule_type, threshold_value, consecutive_failures)
               VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
        )
        .bind(account_id)
        .bind(endpoint_id)
        .bind(rule_type)
        .bind(threshold_value)
        .bind(consecutive_failures)
        .fetch_one(&self.pool)
        .await?;

        Ok(rule)
    }

    async fn list_rules(
        &self,
        account_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<AlertRule>, i64), AppError> {
        let rules = sqlx::query_as::<_, AlertRule>(
            "SELECT * FROM alert_rules WHERE account_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(account_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM alert_rules WHERE account_id = $1 AND deleted_at IS NULL",
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((rules, count.0))
    }

    async fn update_rule(
        &self,
        id: Uuid,
        account_id: Uuid,
        rule_type: Option<&str>,
        threshold_value: Option<i32>,
        consecutive_failures: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<AlertRule, AppError> {
        let rule = sqlx::query_as::<_, AlertRule>(
            r#"UPDATE alert_rules SET
                rule_type = COALESCE($3, rule_type),
                threshold_value = COALESCE($4, threshold_value),
                consecutive_failures = COALESCE($5, consecutive_failures),
                is_active = COALESCE($6, is_active)
               WHERE id = $1 AND account_id = $2 AND deleted_at IS NULL
               RETURNING *"#,
        )
        .bind(id)
        .bind(account_id)
        .bind(rule_type)
        .bind(threshold_value)
        .bind(consecutive_failures)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await?;

        Ok(rule)
    }

    async fn delete_rule(&self, id: Uuid, account_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            "DELETE FROM alert_rules WHERE id = $1 AND account_id = $2 AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Alert rule not found".into()));
        }
        Ok(())
    }

    async fn get_rules_for_endpoint(&self, endpoint_id: Uuid) -> Result<Vec<AlertRule>, AppError> {
        let rules = sqlx::query_as::<_, AlertRule>(
            "SELECT * FROM alert_rules WHERE endpoint_id = $1 AND is_active = TRUE AND deleted_at IS NULL",
        )
        .bind(endpoint_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rules)
    }

    async fn create_channel(
        &self,
        account_id: Uuid,
        channel_type: &str,
        config: &serde_json::Value,
    ) -> Result<AlertChannel, AppError> {
        let channel = sqlx::query_as::<_, AlertChannel>(
            r#"INSERT INTO alert_channels (account_id, channel_type, config)
               VALUES ($1, $2, $3) RETURNING *"#,
        )
        .bind(account_id)
        .bind(channel_type)
        .bind(config)
        .fetch_one(&self.pool)
        .await?;

        Ok(channel)
    }

    async fn list_channels(
        &self,
        account_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<AlertChannel>, i64), AppError> {
        let channels = sqlx::query_as::<_, AlertChannel>(
            "SELECT * FROM alert_channels WHERE account_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(account_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM alert_channels WHERE account_id = $1 AND deleted_at IS NULL",
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((channels, count.0))
    }

    async fn delete_channel(&self, id: Uuid, account_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            "DELETE FROM alert_channels WHERE id = $1 AND account_id = $2 AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Alert channel not found".into()));
        }
        Ok(())
    }

    async fn insert_alert_event(
        &self,
        rule_id: Uuid,
        channel_id: Uuid,
        status: &str,
        message: &str,
    ) -> Result<AlertHistoryEntry, AppError> {
        let entry = sqlx::query_as::<_, AlertHistoryEntry>(
            r#"INSERT INTO alert_history (alert_rule_id, channel_id, status, message)
               VALUES ($1, $2, $3, $4) RETURNING *"#,
        )
        .bind(rule_id)
        .bind(channel_id)
        .bind(status)
        .bind(message)
        .fetch_one(&self.pool)
        .await?;

        Ok(entry)
    }

    async fn get_history(
        &self,
        account_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<AlertHistoryEntry>, i64), AppError> {
        let entries = sqlx::query_as::<_, AlertHistoryEntry>(
            r#"SELECT ah.* FROM alert_history ah
               JOIN alert_rules ar ON ah.alert_rule_id = ar.id
               WHERE ar.account_id = $1
               ORDER BY ah.triggered_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(account_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM alert_history ah
               JOIN alert_rules ar ON ah.alert_rule_id = ar.id
               WHERE ar.account_id = $1"#,
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((entries, count.0))
    }

    async fn soft_delete_all(&self, account_id: Uuid) -> Result<(i64, i64), AppError> {
        let rules_result = sqlx::query(
            "UPDATE alert_rules SET deleted_at = NOW() WHERE account_id = $1 AND deleted_at IS NULL",
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        let channels_result = sqlx::query(
            "UPDATE alert_channels SET deleted_at = NOW() WHERE account_id = $1 AND deleted_at IS NULL",
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        Ok((
            rules_result.rows_affected() as i64,
            channels_result.rows_affected() as i64,
        ))
    }

    async fn restore_all(&self, account_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE alert_rules SET deleted_at = NULL WHERE account_id = $1 AND deleted_at IS NOT NULL",
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE alert_channels SET deleted_at = NULL WHERE account_id = $1 AND deleted_at IS NOT NULL",
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
