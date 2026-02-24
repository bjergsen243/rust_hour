use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use rust_hour_common::error::AppError;

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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UptimeHourly {
    pub endpoint_id: Uuid,
    pub hour: DateTime<Utc>,
    pub total_checks: i64,
    pub healthy_checks: i64,
    pub uptime_pct: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LatencyHourly {
    pub endpoint_id: Uuid,
    pub hour: DateTime<Utc>,
    pub avg_ms: Option<i32>,
    pub p50_ms: Option<i32>,
    pub p95_ms: Option<i32>,
    pub p99_ms: Option<i32>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QueryStore: Send + Sync {
    async fn insert_check_result(
        &self,
        endpoint_id: Uuid,
        status_code: Option<i32>,
        response_time_ms: i32,
        is_healthy: bool,
        error_message: Option<&str>,
        checked_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn get_check_results(
        &self,
        endpoint_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<CheckResult>, i64), AppError>;

    async fn get_latest_check(&self, endpoint_id: Uuid) -> Result<Option<CheckResult>, AppError>;

    async fn get_uptime_stats(
        &self,
        endpoint_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<(f64, f64), AppError>;

    async fn refresh_materialized_views(&self) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct PgQueryStore {
    pool: PgPool,
}

impl PgQueryStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryStore for PgQueryStore {
    async fn insert_check_result(
        &self,
        endpoint_id: Uuid,
        status_code: Option<i32>,
        response_time_ms: i32,
        is_healthy: bool,
        error_message: Option<&str>,
        checked_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"INSERT INTO check_results (endpoint_id, status_code, response_time_ms, is_healthy, error_message, checked_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(endpoint_id)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(is_healthy)
        .bind(error_message)
        .bind(checked_at)
        .execute(&self.pool)
        .await?;

        Ok(())
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
    ) -> Result<(f64, f64), AppError> {
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

        Ok((uptime_pct, row.2))
    }

    async fn refresh_materialized_views(&self) -> Result<(), AppError> {
        sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY endpoint_uptime_hourly")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to refresh uptime view: {e}")))?;

        sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY endpoint_latency_hourly")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to refresh latency view: {e}")))?;

        tracing::info!("Materialized views refreshed");
        Ok(())
    }
}
