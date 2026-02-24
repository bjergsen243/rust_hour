# Microservice Infrastructure Design

**Date**: 2026-02-24
**Status**: Approved
**Scope**: Config management, health checks, Prometheus metrics, gateway resilience, rate limiting, Kafka DLQ, request correlation

## Decisions

- **Deploy target**: Docker Compose now, K8s-ready structure
- **Observability**: Prometheus metrics + structured logs (no OTLP yet)
- **Resilience**: Gateway-only (retry/timeout/circuit-breaker on gRPC clients)
- **Architecture**: Library-per-concern in `crates/common/`, not a bootstrap framework

## 1. Config Management

Wire up the existing `common::config` module. Each service defines a typed config struct that deserializes from env vars using the `config` crate.

- `common::config::load_config<T: DeserializeOwned>(prefix: &str) -> T` loads from env, validates, panics early on missing values
- Keep existing `DatabaseConfig`, `KafkaConfig`, `RedisConfig`
- Add per-service config structs (e.g., `AuthConfig { database: DatabaseConfig, paseto_key: String }`)
- No more `unwrap_or_else` with hardcoded defaults in service code. Defaults live only in `.env.example` and `docker-compose.yml`

## 2. Health Checks (K8s-ready)

Every service exposes two endpoints via a lightweight Axum health server on a separate port:

| Endpoint | Purpose | Returns 200 when |
|----------|---------|-----------------|
| `/health/live` | Liveness probe | Process is running |
| `/health/ready` | Readiness probe | DB connected, Kafka reachable |

- `common::health::HealthServer` spawns Axum on `{SERVICE_PORT + 1000}`
- Each service registers readiness checks (DB ping, Kafka metadata, Redis ping)
- Gateway uses its existing Axum router for health endpoints
- Docker Compose exposes health ports

## 3. Prometheus Metrics

`common::metrics` module using `metrics` + `metrics-exporter-prometheus`.

### Standard metrics per gRPC service:
- `grpc_requests_total{service, method, status}` — counter
- `grpc_request_duration_seconds{service, method}` — histogram
- `db_query_duration_seconds{service, query}` — histogram

### Gateway-specific:
- `http_requests_total{method, path, status}` — counter
- `http_request_duration_seconds{method, path}` — histogram

### Kafka-specific:
- `kafka_messages_produced_total{topic, status}` — counter
- `kafka_messages_consumed_total{topic, status}` — counter

Metrics exposed at `/metrics` on the health server port.

## 4. Gateway Resilience

All resilience on the gateway's gRPC `Channel` connections:

### Timeout
- 5s per gRPC call (configurable via `GRPC_TIMEOUT_MS`)
- 5s connection timeout

### Retry
- Exponential backoff: 100ms, 200ms, 400ms (max 3 retries)
- Only on transient errors: `Unavailable`, `DeadlineExceeded`
- Only for idempotent RPCs (reads)

### Circuit Breaker
Per-service state machine:
- **Closed**: requests flow through
- **Open**: 5 consecutive failures → reject for 30s
- **Half-Open**: allow 1 probe request after cooldown, reset on success

### Implementation
```
Channel::from_shared(url)
    .timeout(Duration::from_millis(timeout))
    .connect_timeout(Duration::from_secs(5))
```
Then wrap channel with Tower retry and circuit breaker layers.

## 5. Rate Limiting

Token-bucket rate limiting on the gateway using Redis (`fred`):

- **Per-IP**: 100 req/min (sliding window in Redis)
- Returns `429 Too Many Requests` with `Retry-After` header
- Implemented as Axum middleware layer

## 6. Kafka Dead-Letter Queue

When a consumer fails to process a message:

1. Retry 3 times with backoff (100ms, 500ms, 2s)
2. On final failure: publish to `{topic}.dlq` with error metadata
3. Commit original offset

DLQ message format:
```json
{
  "original_topic": "monitor.events",
  "original_payload": "<base64 or raw>",
  "error": "reason",
  "failed_at": "ISO-8601",
  "retry_count": 3
}
```

## 7. Request Correlation

- Gateway generates `x-request-id` (UUID v4) for each request if not present
- Logged in every tracing span via `tower-http::request-id`
- Propagated to gRPC calls via `tonic::metadata`
- Included in error responses

## New Dependencies

```toml
metrics = "0.24"
metrics-exporter-prometheus = "0.16"
```

All other deps already in workspace (`tower`, `fred`, `tower-http` with `request-id`).

## Files Changed/Created

| Location | Action |
|----------|--------|
| `Cargo.toml` | Add metrics deps |
| `crates/common/src/config.rs` | Rewrite with `load_config<T>` |
| `crates/common/src/health.rs` | New — health server builder |
| `crates/common/src/metrics.rs` | New — Prometheus setup |
| `crates/common/src/lib.rs` | Add modules |
| `crates/gateway/src/grpc_clients.rs` | Rewrite — timeout, retry, circuit breaker |
| `crates/gateway/src/middleware/rate_limit.rs` | New — Redis rate limiting |
| `crates/gateway/src/middleware/request_id.rs` | New — x-request-id to gRPC |
| `crates/gateway/src/main.rs` | Wire new layers |
| `crates/{service}/src/main.rs` (x5) | Wire config, health, metrics |
| `crates/alert-service/src/consumer.rs` | Retry + DLQ |
| `crates/monitor-query-service/src/projector.rs` | Retry + DLQ |
| `crates/monitor-service/src/producer.rs` | DLQ helper |
| `docker-compose.yml` | Health ports |
