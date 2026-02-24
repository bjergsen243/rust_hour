use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use rust_hour_common::config::{load_config, GatewayConfig};
use rust_hour_common::health::HealthServer;
use rust_hour_common::metrics::init_metrics;

mod error;
mod grpc_clients;
mod middleware;
mod resilience;
mod routes;

#[cfg(test)]
mod error_tests;

use grpc_clients::GrpcClients;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let cfg: GatewayConfig = load_config("GATEWAY");

    // Metrics
    let metrics_handle = init_metrics();

    let clients = GrpcClients::connect(
        &cfg.auth_service_url,
        &cfg.monitor_service_url,
        &cfg.alert_service_url,
        cfg.grpc_timeout_ms,
    )
    .await?;

    let app = Router::new()
        // Auth (public)
        .route("/api/v1/auth/register", post(routes::auth::register))
        .route("/api/v1/auth/login", post(routes::auth::login))
        // Auth (protected)
        .route("/api/v1/auth/me", get(routes::auth::me))
        .route("/api/v1/auth/password", put(routes::auth::update_password))
        // Endpoints (protected)
        .route("/api/v1/endpoints", post(routes::endpoints::create))
        .route("/api/v1/endpoints", get(routes::endpoints::list))
        .route("/api/v1/endpoints/{id}", get(routes::endpoints::get_one))
        .route("/api/v1/endpoints/{id}", put(routes::endpoints::update))
        .route("/api/v1/endpoints/{id}", delete(routes::endpoints::remove))
        .route(
            "/api/v1/endpoints/{id}/metrics",
            get(routes::endpoints::metrics),
        )
        // Alerts (protected)
        .route("/api/v1/alerts/rules", post(routes::alerts::create_rule))
        .route("/api/v1/alerts/rules", get(routes::alerts::list_rules))
        .route(
            "/api/v1/alerts/rules/{id}",
            put(routes::alerts::update_rule),
        )
        .route(
            "/api/v1/alerts/rules/{id}",
            delete(routes::alerts::delete_rule),
        )
        .route(
            "/api/v1/alerts/channels",
            post(routes::alerts::create_channel),
        )
        .route(
            "/api/v1/alerts/channels",
            get(routes::alerts::list_channels),
        )
        .route(
            "/api/v1/alerts/channels/{id}",
            delete(routes::alerts::delete_channel),
        )
        .route("/api/v1/alerts/history", get(routes::alerts::history))
        // Health (on main router for gateway)
        .route("/health/live", get(|| async { "OK" }))
        .route("/health/ready", get(|| async { "OK" }))
        .route(
            "/metrics",
            get({
                let h = metrics_handle;
                move || {
                    let h = h.clone();
                    async move { h.render() }
                }
            }),
        )
        .layer(axum::middleware::from_fn(
            middleware::request_id::request_id_middleware,
        ))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(clients);

    let addr = format!("0.0.0.0:{}", cfg.port);
    tracing::info!("Gateway listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
