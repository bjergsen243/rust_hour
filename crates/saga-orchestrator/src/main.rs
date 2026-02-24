use std::sync::Arc;

use tracing_subscriber::EnvFilter;

use rust_hour_common::config::{load_config, SagaServiceConfig};
use rust_hour_common::health::{pg_readiness_check, HealthServer};
use rust_hour_common::metrics::init_metrics;
use rust_hour_common::proto::alert::alert_service_client::AlertServiceClient;
use rust_hour_common::proto::auth::auth_service_client::AuthServiceClient;
use rust_hour_common::proto::monitor::monitor_service_client::MonitorServiceClient;

mod account_deletion;
mod orchestrator;
mod state_store;
mod steps;

#[cfg(test)]
mod orchestrator_tests;

use account_deletion::build_account_deletion_steps;
use orchestrator::SagaOrchestrator;
use state_store::PgSagaStateStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let cfg: SagaServiceConfig = load_config("SAGA");

    let pool = sqlx::PgPool::connect(&cfg.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Health + metrics on port 51055
    let metrics_handle = init_metrics();
    let mut health = HealthServer::new();
    health.add_readiness_check("postgres", pg_readiness_check(pool.clone()));
    health.with_metrics(metrics_handle).spawn(51055);

    let state_store = Arc::new(PgSagaStateStore::new(pool));

    // Connect gRPC clients for saga steps
    let auth_client = AuthServiceClient::connect(cfg.auth_service_url).await?;
    let monitor_client = MonitorServiceClient::connect(cfg.monitor_service_url).await?;
    let alert_client = AlertServiceClient::connect(cfg.alert_service_url).await?;

    let deletion_steps = build_account_deletion_steps(auth_client, monitor_client, alert_client);
    let _orchestrator = SagaOrchestrator::new(Arc::clone(&state_store), deletion_steps);

    tracing::info!("Saga orchestrator ready — account deletion saga registered");

    tokio::signal::ctrl_c().await?;

    Ok(())
}
