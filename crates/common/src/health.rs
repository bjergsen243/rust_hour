use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use tokio::sync::RwLock;

type CheckFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>;

/// A health server that exposes `/health/live`, `/health/ready`, and `/metrics`.
///
/// Runs on a separate port from the main service so K8s probes
/// don't interfere with gRPC traffic.
pub struct HealthServer {
    checks: Arc<RwLock<Vec<(String, CheckFn)>>>,
    metrics_handle: Option<metrics_exporter_prometheus::PrometheusHandle>,
}

impl HealthServer {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(Vec::new())),
            metrics_handle: None,
        }
    }

    /// Register a readiness check. The check returns Ok(()) if healthy.
    pub fn add_readiness_check<F, Fut>(&mut self, name: impl Into<String>, check: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        let check_fn: CheckFn = Box::new(move || Box::pin(check()));
        // We need to block to write, but this is only called at startup
        let checks = Arc::clone(&self.checks);
        let name = name.into();
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                checks.write().await.push((name, check_fn));
            });
        });
    }

    /// Set the Prometheus handle for serving `/metrics`.
    pub fn with_metrics(mut self, handle: metrics_exporter_prometheus::PrometheusHandle) -> Self {
        self.metrics_handle = Some(handle);
        self
    }

    /// Spawn the health server on the given port.
    /// Returns a JoinHandle so callers can detect if it crashes.
    pub fn spawn(self, port: u16) -> tokio::task::JoinHandle<()> {
        let checks = self.checks;
        let metrics_handle = self.metrics_handle;

        tokio::spawn(async move {
            let checks_for_ready = Arc::clone(&checks);

            let mut app = Router::new()
                .route("/health/live", get(|| async { StatusCode::OK }))
                .route(
                    "/health/ready",
                    get(move || {
                        let checks = Arc::clone(&checks_for_ready);
                        async move { readiness_handler(checks).await }
                    }),
                );

            if let Some(handle) = metrics_handle {
                app = app.route(
                    "/metrics",
                    get(move || {
                        let h = handle.clone();
                        async move { h.render() }
                    }),
                );
            }

            let addr = format!("0.0.0.0:{port}");
            tracing::info!("Health server listening on {addr}");

            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .expect("Failed to bind health server");

            axum::serve(listener, app)
                .await
                .expect("Health server crashed");
        })
    }
}

async fn readiness_handler(
    checks: Arc<RwLock<Vec<(String, CheckFn)>>>,
) -> impl IntoResponse {
    let checks = checks.read().await;
    let mut results = Vec::new();
    let mut all_ok = true;

    for (name, check) in checks.iter() {
        match check().await {
            Ok(()) => results.push(serde_json::json!({"name": name, "status": "ok"})),
            Err(e) => {
                all_ok = false;
                results.push(serde_json::json!({"name": name, "status": "fail", "error": e}));
            }
        }
    }

    let status = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(serde_json::json!({ "checks": results })))
}

/// Helper: create a DB readiness check for a sqlx PgPool.
pub fn pg_readiness_check(
    pool: sqlx::PgPool,
) -> impl Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync + 'static {
    move || {
        let pool = pool.clone();
        Box::pin(async move {
            sqlx::query("SELECT 1")
                .execute(&pool)
                .await
                .map(|_| ())
                .map_err(|e| format!("postgres: {e}"))
        })
    }
}
