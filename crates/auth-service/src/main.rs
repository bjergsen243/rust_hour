use std::sync::Arc;

use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use rust_hour_common::config::{load_config, AuthServiceConfig};
use rust_hour_common::health::{pg_readiness_check, HealthServer};
use rust_hour_common::metrics::init_metrics;
use rust_hour_common::proto::auth::auth_service_server::AuthServiceServer;

mod grpc;
mod password;
mod store;

#[cfg(test)]
mod grpc_tests;

use grpc::AuthGrpcService;
use store::PgAuthStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let cfg: AuthServiceConfig = load_config("AUTH");

    let pool = sqlx::PgPool::connect(&cfg.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Health + metrics
    let metrics_handle = init_metrics();
    let health_port = cfg.port + 1000;
    let mut health = HealthServer::new();
    health.add_readiness_check("postgres", pg_readiness_check(pool.clone()));
    health.with_metrics(metrics_handle).spawn(health_port);

    let store = Arc::new(PgAuthStore::new(pool));
    let service = AuthGrpcService::new(store, cfg.paseto_key.into_bytes());

    let addr = format!("[::]:{}",cfg.port).parse()?;
    tracing::info!("Auth service listening on {}", addr);

    Server::builder()
        .add_service(AuthServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
