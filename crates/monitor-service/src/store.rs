use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use rust_hour_common::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Endpoint {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub url: String,
    pub method: String,
    pub headers: serde_json::Value,
    pub expected_status: i32,
    pub interval_secs: i32,
    pub timeout_ms: i32,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CheckResult {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub status_code: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub is_healthy: bool,
    pub error_message: Option<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UptimeStats {
    pub total_checks: i64,
    pub healthy_checks: i64,
    pub uptime_pct: f64,
    pub avg_response_ms: f64,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MonitorStore: Send + Sync {
    async fn create_endpoint(
        &self,
        account_id: Uuid,
        name: &str,
        url: &str,
        method: &str,
        headers: &serde_json::Value,
        expected_status: i32,
        interval_secs: i32,
        timeout_ms: i32,
    ) -> Result<Endpoint, AppError>;

    async fn list_endpoints(
        &self,
        account_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<Endpoint>, i64), AppError>;

    async fn get_endpoint(&self, id: Uuid, account_id: Uuid) -> Result<Endpoint, AppError>;

    async fn update_endpoint(
        &self,
        id: Uuid,
        account_id: Uuid,
        name: Option<&str>,
        url: Option<&str>,
        method: Option<&str>,
        headers: Option<&serde_json::Value>,
        expected_status: Option<i32>,
        interval_secs: Option<i32>,
        timeout_ms: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<Endpoint, AppError>;

    async fn delete_endpoint(&self, id: Uuid, account_id: Uuid) -> Result<(), AppError>;

    async fn get_active_endpoints(&self) -> Result<Vec<Endpoint>, AppError>;

    async fn insert_check_result(
        &self,
        endpoint_id: Uuid,
        status_code: Option<i32>,
        response_time_ms: i32,
        is_healthy: bool,
        error_message: Option<&str>,
    ) -> Result<CheckResult, AppError>;

    async fn get_latest_check(&self, endpoint_id: Uuid) -> Result<Option<CheckResult>, AppError>;

    async fn get_uptime_stats(
        &self,
        endpoint_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<UptimeStats, AppError>;

    async fn get_check_results(
        &self,
        endpoint_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<CheckResult>, i64), AppError>;

    // Saga support
    async fn soft_delete_all(&self, account_id: Uuid) -> Result<i64, AppError>;
    async fn restore_all(&self, account_id: Uuid) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct PgMonitorStore {
    pool: PgPool,
}

impl PgMonitorStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MonitorStore for PgMonitorStore {
    async fn create_endpoint(
        &self,
        account_id: Uuid,
        name: &str,
        url: &str,
        method: &str,
        headers: &serde_json::Value,
        expected_status: i32,
        interval_secs: i32,
        timeout_ms: i32,
    ) -> Result<Endpoint, AppError> {
        let ep = sqlx::query_as::<_, Endpoint>(
            r#"INSERT INTO endpoints (account_id, name, url, method, headers, expected_status, interval_secs, timeout_ms)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#,
        )
        .bind(account_id)
        .bind(name)
        .bind(url)
        .bind(method)
        .bind(headers)
        .bind(expected_status)
        .bind(interval_secs)
        .bind(timeout_ms)
        .fetch_one(&self.pool)
        .await?;

        Ok(ep)
    }

    async fn list_endpoints(
        &self,
        account_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<Endpoint>, i64), AppError> {
        let endpoints = sqlx::query_as::<_, Endpoint>(
            "SELECT * FROM endpoints WHERE account_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(account_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM endpoints WHERE account_id = $1 AND deleted_at IS NULL",
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((endpoints, count.0))
    }

    async fn get_endpoint(&self, id: Uuid, account_id: Uuid) -> Result<Endpoint, AppError> {
        let ep = sqlx::query_as::<_, Endpoint>(
            "SELECT * FROM endpoints WHERE id = $1 AND account_id = $2 AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(account_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ep)
    }

    async fn update_endpoint(
        &self,
        id: Uuid,
        account_id: Uuid,
        name: Option<&str>,
        url: Option<&str>,
        method: Option<&str>,
        headers: Option<&serde_json::Value>,
        expected_status: Option<i32>,
        interval_secs: Option<i32>,
        timeout_ms: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<Endpoint, AppError> {
        let ep = sqlx::query_as::<_, Endpoint>(
            r#"UPDATE endpoints SET
                name = COALESCE($3, name),
                url = COALESCE($4, url),
                method = COALESCE($5, method),
                headers = COALESCE($6, headers),
                expected_status = COALESCE($7, expected_status),
                interval_secs = COALESCE($8, interval_secs),
                timeout_ms = COALESCE($9, timeout_ms),
                is_active = COALESCE($10, is_active),
                updated_at = NOW()
               WHERE id = $1 AND account_id = $2 AND deleted_at IS NULL
               RETURNING *"#,
        )
        .bind(id)
        .bind(account_id)
        .bind(name)
        .bind(url)
        .bind(method)
        .bind(headers)
        .bind(expected_status)
        .bind(interval_secs)
        .bind(timeout_ms)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await?;

        Ok(ep)
    }

    async fn delete_endpoint(&self, id: Uuid, account_id: Uuid) -> Result<(), AppError> {
        let result =
            sqlx::query("DELETE FROM endpoints WHERE id = $1 AND account_id = $2 AND deleted_at IS NULL")
                .bind(id)
                .bind(account_id)
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Endpoint not found".into()));
        }
        Ok(())
    }

    async fn get_active_endpoints(&self) -> Result<Vec<Endpoint>, AppError> {
        let endpoints = sqlx::query_as::<_, Endpoint>(
            "SELECT * FROM endpoints WHERE is_active = TRUE AND deleted_at IS NULL",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(endpoints)
    }

    async fn insert_check_result(
        &self,
        endpoint_id: Uuid,
        status_code: Option<i32>,
        response_time_ms: i32,
        is_healthy: bool,
        error_message: Option<&str>,
    ) -> Result<CheckResult, AppError> {
        let cr = sqlx::query_as::<_, CheckResult>(
            r#"INSERT INTO check_results (endpoint_id, status_code, response_time_ms, is_healthy, error_message)
               VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
        )
        .bind(endpoint_id)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(is_healthy)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(cr)
    }

    async fn get_latest_check(&self, endpoint_id: Uuid) -> Result<Option<CheckResult>, AppError> {
        let cr = sqlx::query_as::<_, CheckResult>(
            "SELECT * FROM check_results WHERE endpoint_id = $1 ORDER BY checked_at DESC LIMIT 1",
        )
        .bind(endpoint_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(cr)
    }

    async fn get_uptime_stats(
        &self,
        endpoint_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<UptimeStats, AppError> {
        let row: (i64, i64, f64) = sqlx::query_as(
            r#"SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE is_healthy),
                COALESCE(AVG(response_time_ms)::FLOAT8, 0)
               FROM check_results
               WHERE endpoint_id = $1 AND checked_at BETWEEN $2 AND $3"#,
        )
        .bind(endpoint_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;

        let uptime_pct = if row.0 > 0 {
            (row.1 as f64 / row.0 as f64) * 100.0
        } else {
            100.0
        };

        Ok(UptimeStats {
            total_checks: row.0,
            healthy_checks: row.1,
            uptime_pct,
            avg_response_ms: row.2,
        })
    }

    async fn get_check_results(
        &self,
        endpoint_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<CheckResult>, i64), AppError> {
        let results = sqlx::query_as::<_, CheckResult>(
            r#"SELECT * FROM check_results
               WHERE endpoint_id = $1 AND checked_at BETWEEN $2 AND $3
               ORDER BY checked_at DESC LIMIT $4 OFFSET $5"#,
        )
        .bind(endpoint_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM check_results
               WHERE endpoint_id = $1 AND checked_at BETWEEN $2 AND $3"#,
        )
        .bind(endpoint_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;

        Ok((results, count.0))
    }

    // Saga support: soft-delete all endpoints for an account
    async fn soft_delete_all(&self, account_id: Uuid) -> Result<i64, AppError> {
        let result = sqlx::query(
            "UPDATE endpoints SET deleted_at = NOW(), updated_at = NOW() WHERE account_id = $1 AND deleted_at IS NULL",
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    // Saga support: restore soft-deleted endpoints for an account
    async fn restore_all(&self, account_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE endpoints SET deleted_at = NULL, updated_at = NOW() WHERE account_id = $1 AND deleted_at IS NOT NULL",
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
