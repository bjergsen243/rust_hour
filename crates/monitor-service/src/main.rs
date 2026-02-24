use std::sync::Arc;

use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use rust_hour_common::config::{load_config, MonitorServiceConfig};
use rust_hour_common::health::{pg_readiness_check, HealthServer};
use rust_hour_common::metrics::init_metrics;
use rust_hour_common::proto::monitor::monitor_service_server::MonitorServiceServer;

mod grpc;
mod probe;
mod producer;
mod scheduler;
mod store;

#[cfg(test)]
mod grpc_tests;

use grpc::MonitorGrpcService;
use producer::create_producer;
use scheduler::run_scheduler;
use store::PgMonitorStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let cfg: MonitorServiceConfig = load_config("MONITOR");

    let pool = sqlx::PgPool::connect(&cfg.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Health + metrics
    let metrics_handle = init_metrics();
    let health_port = cfg.port + 1000;
    let mut health = HealthServer::new();
    health.add_readiness_check("postgres", pg_readiness_check(pool.clone()));
    health.with_metrics(metrics_handle).spawn(health_port);

    let store = Arc::new(PgMonitorStore::new(pool));
    let kafka_producer = create_producer(&cfg.kafka_brokers);
    let http_client = reqwest::Client::new();

    // Spawn the probe scheduler in background
    let scheduler_store = Arc::clone(&store);
    tokio::spawn(run_scheduler(scheduler_store, kafka_producer, http_client));

    let service = MonitorGrpcService::new(store);
    let addr = format!("[::]:{}", cfg.port).parse()?;
    tracing::info!("Monitor service listening on {}", addr);

    Server::builder()
        .add_service(MonitorServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
