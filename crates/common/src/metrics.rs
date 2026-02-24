use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

/// Initialize the Prometheus metrics recorder and return a handle
/// for rendering the `/metrics` endpoint.
///
/// Call once at service startup. After this, any `metrics::counter!()`,
/// `metrics::histogram!()` etc. calls anywhere in the process are captured.
pub fn init_metrics() -> PrometheusHandle {
    let builder = PrometheusBuilder::new();
    builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

/// Record a gRPC request metric (call from tonic interceptors or handlers).
pub fn record_grpc_request(service: &str, method: &str, status: &str, duration_secs: f64) {
    let labels = [
        ("service", service.to_string()),
        ("method", method.to_string()),
        ("status", status.to_string()),
    ];
    metrics::counter!("grpc_requests_total", &labels).increment(1);
    metrics::histogram!("grpc_request_duration_seconds", &labels).record(duration_secs);
}

/// Record an HTTP request metric (call from Axum middleware).
pub fn record_http_request(method: &str, path: &str, status: u16, duration_secs: f64) {
    let labels = [
        ("method", method.to_string()),
        ("path", path.to_string()),
        ("status", status.to_string()),
    ];
    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_request_duration_seconds", &labels).record(duration_secs);
}

/// Record a Kafka produce event.
pub fn record_kafka_produce(topic: &str, success: bool) {
    let labels = [
        ("topic", topic.to_string()),
        ("status", if success { "ok" } else { "error" }.to_string()),
    ];
    metrics::counter!("kafka_messages_produced_total", &labels).increment(1);
}

/// Record a Kafka consume event.
pub fn record_kafka_consume(topic: &str, success: bool) {
    let labels = [
        ("topic", topic.to_string()),
        ("status", if success { "ok" } else { "error" }.to_string()),
    ];
    metrics::counter!("kafka_messages_consumed_total", &labels).increment(1);
}
