use std::sync::Arc;

use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use rust_hour_common::config::{load_config, AlertServiceConfig};
use rust_hour_common::health::{pg_readiness_check, HealthServer};
use rust_hour_common::metrics::init_metrics;
use rust_hour_common::proto::alert::alert_service_server::AlertServiceServer;

mod consumer;
mod evaluator;
mod grpc;
mod notifier;
mod store;

#[cfg(test)]
mod evaluator_tests;
#[cfg(test)]
mod grpc_tests;

use consumer::{consume_monitor_events, create_consumer, create_dlq_producer};
use grpc::AlertGrpcService;
use store::PgAlertStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let cfg: AlertServiceConfig = load_config("ALERT");

    let pool = sqlx::PgPool::connect(&cfg.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Health + metrics
    let metrics_handle = init_metrics();
    let health_port = cfg.port + 1000;
    let mut health = HealthServer::new();
    health.add_readiness_check("postgres", pg_readiness_check(pool.clone()));
    health.with_metrics(metrics_handle).spawn(health_port);

    let store = Arc::new(PgAlertStore::new(pool));

    // Spawn Kafka consumer with DLQ support
    let consumer = create_consumer(&cfg.kafka_brokers, "alert-service");
    let dlq_producer = create_dlq_producer(&cfg.kafka_brokers);
    let consumer_store = Arc::clone(&store);
    tokio::spawn(consume_monitor_events(consumer, consumer_store, dlq_producer));

    let service = AlertGrpcService::new(store);
    let addr = format!("[::]:{}", cfg.port).parse()?;
    tracing::info!("Alert service listening on {}", addr);

    Server::builder()
        .add_service(AlertServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
