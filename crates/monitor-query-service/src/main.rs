use std::sync::Arc;

use tracing_subscriber::EnvFilter;

use rust_hour_common::config::{load_config, MonitorQueryServiceConfig};
use rust_hour_common::health::{pg_readiness_check, HealthServer};
use rust_hour_common::metrics::init_metrics;

mod projector;
mod store;

#[cfg(test)]
mod store_tests;

use projector::{create_consumer, create_dlq_producer, run_projector};
use store::PgQueryStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let cfg: MonitorQueryServiceConfig = load_config("MONITOR_QUERY");

    let pool = sqlx::PgPool::connect(&cfg.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Health + metrics on port 51054
    let metrics_handle = init_metrics();
    let mut health = HealthServer::new();
    health.add_readiness_check("postgres", pg_readiness_check(pool.clone()));
    health.with_metrics(metrics_handle).spawn(51054);

    let store = Arc::new(PgQueryStore::new(pool));
    let consumer = create_consumer(&cfg.kafka_brokers);
    let dlq_producer = create_dlq_producer(&cfg.kafka_brokers);

    tracing::info!("Monitor query service starting — consuming from check.results");

    run_projector(consumer, store, dlq_producer).await;

    Ok(())
}
