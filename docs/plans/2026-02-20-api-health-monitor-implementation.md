# API Health Monitor — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform rust_hour from a monolithic CRUD Q&A API into a microservice-based API Health Monitoring Platform with 7 services, featuring CQRS, Saga pattern, Kafka event streaming, gRPC inter-service communication, and Redis caching.

**Architecture:** Cargo workspace monorepo with 8 crates: `common` (shared library), `gateway` (Axum REST), `auth-service` (gRPC), `monitor-service` (gRPC + probe scheduler, CQRS write side), `monitor-query-service` (gRPC, CQRS read side + Redis cache), `alert-service` (gRPC + Kafka consumer), `saga-orchestrator` (distributed transaction coordination). Each backend service owns its database schema. Gateway is stateless and routes REST requests to gRPC backends.

**Tech Stack:** Rust 2021, Axum 0.8, tonic 0.12, SQLx 0.8, rdkafka (Kafka), reqwest, PASETO, Argon2id, Redis (fred), tokio-cron-scheduler, tracing + OpenTelemetry, testcontainers, mockall.

**Patterns:** CQRS (Monitor write/read separation), Saga (Orchestrator-based account deletion), Event-Driven Architecture (Kafka), API Gateway, Database per Service.

**Reference:** See `docs/plans/2026-02-20-api-health-monitor-design.md` for full design details.

---

## Phase 1: Workspace Foundation

### Task 1.1: Restructure into Cargo Workspace

**Files:**
- Modify: `Cargo.toml` (root — replace package with workspace)
- Create: `crates/common/Cargo.toml`
- Create: `crates/common/src/lib.rs`
- Create: `crates/gateway/Cargo.toml`
- Create: `crates/gateway/src/main.rs`
- Create: `crates/auth-service/Cargo.toml`
- Create: `crates/auth-service/src/main.rs`
- Create: `crates/monitor-service/Cargo.toml`
- Create: `crates/monitor-service/src/main.rs`
- Create: `crates/alert-service/Cargo.toml`
- Create: `crates/alert-service/src/main.rs`
- Create: `crates/monitor-query-service/Cargo.toml`
- Create: `crates/monitor-query-service/src/main.rs`
- Create: `crates/saga-orchestrator/Cargo.toml`
- Create: `crates/saga-orchestrator/src/main.rs`

**Step 1: Replace root Cargo.toml with workspace definition**

```toml
[workspace]
members = [
    "crates/common",
    "crates/gateway",
    "crates/auth-service",
    "crates/monitor-service",
    "crates/monitor-query-service",
    "crates/alert-service",
    "crates/saga-orchestrator",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Son Nguyen Thanh <thanhsondeptrai243@gmail.com>"]
license = "MIT"

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# Web framework (gateway)
axum = { version = "0.8", features = ["macros"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace", "request-id"] }

# gRPC
tonic = "0.12"
tonic-build = "0.12"
prost = "0.13"
prost-types = "0.13"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "migrate"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Auth
rust-argon2 = "1.0"
paseto = "2.0"

# HTTP client (probes)
reqwest = { version = "0.12", features = ["json"] }

# Message queue
rdkafka = { version = "0.37", features = ["cmake-build"] }

# Caching / rate limiting
fred = { version = "10", features = ["subscriber-client"] }

# Scheduling
tokio-cron-scheduler = "0.13"

# Config
config = "0.14"

# Logging / tracing
tracing = { version = "0.1", features = ["log"] }
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1", features = ["v4", "serde"] }

# Misc
rand = "0.8"
regex = "1"
async-trait = "0.1"
thiserror = "2"
anyhow = "1"

# Testing
mockall = "0.13"
```

**Step 2: Create common crate skeleton**

`crates/common/Cargo.toml`:
```toml
[package]
name = "rust-hour-common"
version.workspace = true
edition.workspace = true

[dependencies]
tonic = { workspace = true }
prost = { workspace = true }
prost-types = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
paseto = { workspace = true }
async-trait = { workspace = true }

[build-dependencies]
tonic-build = { workspace = true }
```

`crates/common/src/lib.rs`:
```rust
pub mod auth;
pub mod config;
pub mod error;

// Proto-generated modules will be added after Task 1.2
```

**Step 3: Create service crate skeletons**

Each service crate gets a minimal `Cargo.toml` and `main.rs`.

`crates/gateway/Cargo.toml`:
```toml
[package]
name = "rust-hour-gateway"
version.workspace = true
edition.workspace = true

[[bin]]
name = "gateway"
path = "src/main.rs"

[dependencies]
rust-hour-common = { path = "../common" }
axum = { workspace = true }
axum-extra = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
tonic = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
fred = { workspace = true }
config = { workspace = true }

[dev-dependencies]
mockall = { workspace = true }
reqwest = { workspace = true }
```

`crates/auth-service/Cargo.toml`:
```toml
[package]
name = "rust-hour-auth"
version.workspace = true
edition.workspace = true

[[bin]]
name = "auth-service"
path = "src/main.rs"

[dependencies]
rust-hour-common = { path = "../common" }
tonic = { workspace = true }
tokio = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
paseto = { workspace = true }
rust-argon2 = { workspace = true }
rand = { workspace = true }
regex = { workspace = true }
config = { workspace = true }
async-trait = { workspace = true }

[dev-dependencies]
mockall = { workspace = true }
```

`crates/monitor-service/Cargo.toml`:
```toml
[package]
name = "rust-hour-monitor"
version.workspace = true
edition.workspace = true

[[bin]]
name = "monitor-service"
path = "src/main.rs"

[dependencies]
rust-hour-common = { path = "../common" }
tonic = { workspace = true }
tokio = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
reqwest = { workspace = true }
rdkafka = { workspace = true }
tokio-cron-scheduler = { workspace = true }
config = { workspace = true }
async-trait = { workspace = true }

[dev-dependencies]
mockall = { workspace = true }
```

`crates/alert-service/Cargo.toml`:
```toml
[package]
name = "rust-hour-alert"
version.workspace = true
edition.workspace = true

[[bin]]
name = "alert-service"
path = "src/main.rs"

[dependencies]
rust-hour-common = { path = "../common" }
tonic = { workspace = true }
tokio = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
rdkafka = { workspace = true }
reqwest = { workspace = true }
config = { workspace = true }
async-trait = { workspace = true }

[dev-dependencies]
mockall = { workspace = true }
```

`crates/monitor-query-service/Cargo.toml`:
```toml
[package]
name = "rust-hour-monitor-query"
version.workspace = true
edition.workspace = true

[[bin]]
name = "monitor-query-service"
path = "src/main.rs"

[dependencies]
rust-hour-common = { path = "../common" }
tonic = { workspace = true }
tokio = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
rdkafka = { workspace = true }
fred = { workspace = true }
config = { workspace = true }
async-trait = { workspace = true }

[dev-dependencies]
mockall = { workspace = true }
```

`crates/saga-orchestrator/Cargo.toml`:
```toml
[package]
name = "rust-hour-saga"
version.workspace = true
edition.workspace = true

[[bin]]
name = "saga-orchestrator"
path = "src/main.rs"

[dependencies]
rust-hour-common = { path = "../common" }
tonic = { workspace = true }
tokio = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
rdkafka = { workspace = true }
config = { workspace = true }
async-trait = { workspace = true }

[dev-dependencies]
mockall = { workspace = true }
```

Each `main.rs` starts as:
```rust
#[tokio::main]
async fn main() {
    println!("Service starting...");
}
```

**Step 4: Verify workspace compiles**

Run: `cargo build`
Expected: All 5 crates compile with no errors.

**Step 5: Commit**

```
feat: restructure project as cargo workspace with 5 crates
```

---

### Task 1.2: Define gRPC Proto Files

**Files:**
- Create: `proto/auth.proto`
- Create: `proto/monitor.proto`
- Create: `proto/alert.proto`
- Create: `crates/common/build.rs`
- Modify: `crates/common/src/lib.rs`

**Step 1: Write auth.proto**

`proto/auth.proto`:
```protobuf
syntax = "proto3";
package auth;

service AuthService {
    rpc Register(RegisterRequest) returns (AuthResponse);
    rpc Login(LoginRequest) returns (AuthResponse);
    rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);
    rpc GetAccount(GetAccountRequest) returns (AccountResponse);
    rpc UpdatePassword(UpdatePasswordRequest) returns (SuccessResponse);
}

message RegisterRequest {
    string email = 1;
    string password = 2;
}

message LoginRequest {
    string email = 1;
    string password = 2;
}

message AuthResponse {
    string token = 1;
}

message ValidateTokenRequest {
    string token = 1;
}

message ValidateTokenResponse {
    string account_id = 1;
    string email = 2;
}

message GetAccountRequest {
    string account_id = 1;
}

message AccountResponse {
    string id = 1;
    string email = 2;
    string created_at = 3;
}

message UpdatePasswordRequest {
    string account_id = 1;
    string old_password = 2;
    string new_password = 3;
}

message SuccessResponse {
    bool success = 1;
    string message = 2;
}
```

**Step 2: Write monitor.proto**

`proto/monitor.proto`:
```protobuf
syntax = "proto3";
package monitor;

service MonitorService {
    rpc CreateEndpoint(CreateEndpointRequest) returns (EndpointResponse);
    rpc ListEndpoints(ListEndpointsRequest) returns (ListEndpointsResponse);
    rpc GetEndpoint(GetEndpointRequest) returns (EndpointDetailResponse);
    rpc UpdateEndpoint(UpdateEndpointRequest) returns (EndpointResponse);
    rpc DeleteEndpoint(DeleteEndpointRequest) returns (SuccessResponse);
    rpc GetMetrics(GetMetricsRequest) returns (MetricsResponse);
}

message CreateEndpointRequest {
    string account_id = 1;
    string name = 2;
    string url = 3;
    string method = 4;       // GET, POST, etc.
    string headers_json = 5; // JSON string of headers
    int32 expected_status = 6;
    int32 interval_secs = 7;
    int32 timeout_ms = 8;
}

message EndpointResponse {
    string id = 1;
    string account_id = 2;
    string name = 3;
    string url = 4;
    string method = 5;
    string headers_json = 6;
    int32 expected_status = 7;
    int32 interval_secs = 8;
    int32 timeout_ms = 9;
    bool is_active = 10;
    string created_at = 11;
    string updated_at = 12;
}

message ListEndpointsRequest {
    string account_id = 1;
    int32 limit = 2;
    int32 offset = 3;
}

message ListEndpointsResponse {
    repeated EndpointResponse endpoints = 1;
    int32 total = 2;
}

message GetEndpointRequest {
    string id = 1;
    string account_id = 2;
}

message EndpointDetailResponse {
    EndpointResponse endpoint = 1;
    CheckResult latest_check = 2;
    double uptime_percent_24h = 3;
    double avg_response_time_ms = 4;
}

message UpdateEndpointRequest {
    string id = 1;
    string account_id = 2;
    optional string name = 3;
    optional string url = 4;
    optional string method = 5;
    optional string headers_json = 6;
    optional int32 expected_status = 7;
    optional int32 interval_secs = 8;
    optional int32 timeout_ms = 9;
    optional bool is_active = 10;
}

message DeleteEndpointRequest {
    string id = 1;
    string account_id = 2;
}

message SuccessResponse {
    bool success = 1;
    string message = 2;
}

message GetMetricsRequest {
    string endpoint_id = 1;
    string account_id = 2;
    string from = 3;  // ISO 8601
    string to = 4;    // ISO 8601
    int32 limit = 5;
    int32 offset = 6;
}

message CheckResult {
    string id = 1;
    string endpoint_id = 2;
    int32 status_code = 3;
    int32 response_time_ms = 4;
    bool is_healthy = 5;
    string error_message = 6;
    string checked_at = 7;
}

message MetricsResponse {
    repeated CheckResult results = 1;
    int32 total = 2;
    double uptime_percent = 3;
    double avg_response_time_ms = 4;
}
```

**Step 3: Write alert.proto**

`proto/alert.proto`:
```protobuf
syntax = "proto3";
package alert;

service AlertService {
    rpc CreateRule(CreateRuleRequest) returns (RuleResponse);
    rpc ListRules(ListRulesRequest) returns (ListRulesResponse);
    rpc UpdateRule(UpdateRuleRequest) returns (RuleResponse);
    rpc DeleteRule(DeleteRuleRequest) returns (SuccessResponse);
    rpc CreateChannel(CreateChannelRequest) returns (ChannelResponse);
    rpc ListChannels(ListChannelsRequest) returns (ListChannelsResponse);
    rpc DeleteChannel(DeleteChannelRequest) returns (SuccessResponse);
    rpc GetHistory(GetHistoryRequest) returns (HistoryResponse);
}

message CreateRuleRequest {
    string account_id = 1;
    string endpoint_id = 2;
    string rule_type = 3;         // "down", "latency_threshold", "status_mismatch"
    int32 threshold_value = 4;
    int32 consecutive_failures = 5;
}

message RuleResponse {
    string id = 1;
    string account_id = 2;
    string endpoint_id = 3;
    string rule_type = 4;
    int32 threshold_value = 5;
    int32 consecutive_failures = 6;
    bool is_active = 7;
    string created_at = 8;
}

message ListRulesRequest {
    string account_id = 1;
    int32 limit = 2;
    int32 offset = 3;
}

message ListRulesResponse {
    repeated RuleResponse rules = 1;
    int32 total = 2;
}

message UpdateRuleRequest {
    string id = 1;
    string account_id = 2;
    optional string rule_type = 3;
    optional int32 threshold_value = 4;
    optional int32 consecutive_failures = 5;
    optional bool is_active = 6;
}

message DeleteRuleRequest {
    string id = 1;
    string account_id = 2;
}

message SuccessResponse {
    bool success = 1;
    string message = 2;
}

message CreateChannelRequest {
    string account_id = 1;
    string channel_type = 2;  // "webhook", "email", "slack"
    string config_json = 3;   // JSON: {"url": "..."} or {"email": "..."}
}

message ChannelResponse {
    string id = 1;
    string account_id = 2;
    string channel_type = 3;
    string config_json = 4;
    string created_at = 5;
}

message ListChannelsRequest {
    string account_id = 1;
    int32 limit = 2;
    int32 offset = 3;
}

message ListChannelsResponse {
    repeated ChannelResponse channels = 1;
    int32 total = 2;
}

message DeleteChannelRequest {
    string id = 1;
    string account_id = 2;
}

message GetHistoryRequest {
    string account_id = 1;
    int32 limit = 2;
    int32 offset = 3;
}

message AlertEvent {
    string id = 1;
    string alert_rule_id = 2;
    string channel_id = 3;
    string status = 4;      // "sent", "failed", "acknowledged"
    string message = 5;
    string triggered_at = 6;
}

message HistoryResponse {
    repeated AlertEvent events = 1;
    int32 total = 2;
}
```

**Step 4: Create common build.rs for proto compilation**

`crates/common/build.rs`:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../proto";

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                &format!("{proto_root}/auth.proto"),
                &format!("{proto_root}/monitor.proto"),
                &format!("{proto_root}/alert.proto"),
            ],
            &[proto_root],
        )?;

    Ok(())
}
```

**Step 5: Expose generated proto modules in common/src/lib.rs**

```rust
pub mod auth;
pub mod config;
pub mod error;

pub mod proto {
    pub mod auth {
        tonic::include_proto!("auth");
    }
    pub mod monitor {
        tonic::include_proto!("monitor");
    }
    pub mod alert {
        tonic::include_proto!("alert");
    }
}
```

**Step 6: Verify proto compilation**

Run: `cargo build -p rust-hour-common`
Expected: Compiles successfully, proto types generated.

**Step 7: Commit**

```
feat: add gRPC proto definitions for auth, monitor, and alert services
```

---

### Task 1.3: Common Error Types

**Files:**
- Create: `crates/common/src/error.rs`

**Step 1: Write common error types**

```rust
use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Rate limited")]
    RateLimited,
}

impl From<AppError> for Status {
    fn from(err: AppError) -> Self {
        match err {
            AppError::NotFound(msg) => Status::not_found(msg),
            AppError::Unauthorized(msg) => Status::unauthenticated(msg),
            AppError::Forbidden(msg) => Status::permission_denied(msg),
            AppError::BadRequest(msg) => Status::invalid_argument(msg),
            AppError::Conflict(msg) => Status::already_exists(msg),
            AppError::Internal(msg) => Status::internal(msg),
            AppError::Database(msg) => Status::internal(format!("Database error: {msg}")),
            AppError::RateLimited => Status::resource_exhausted("Rate limit exceeded"),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".into()),
            sqlx::Error::Database(db_err) => {
                if db_err.code().as_deref() == Some("23505") {
                    AppError::Conflict("Resource already exists".into())
                } else {
                    AppError::Database(db_err.to_string())
                }
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}
```

**Step 2: Add `sqlx` to common dependencies**

Add to `crates/common/Cargo.toml` `[dependencies]`:
```toml
sqlx = { workspace = true }
```

**Step 3: Verify compilation**

Run: `cargo build -p rust-hour-common`
Expected: PASS

**Step 4: Commit**

```
feat: add common error types with gRPC and HTTP status mapping
```

---

### Task 1.4: Common Auth Middleware

**Files:**
- Create: `crates/common/src/auth.rs`
- Create: `crates/common/src/config.rs`

**Step 1: Write PASETO token utilities**

`crates/common/src/auth.rs`:
```rust
use chrono::{Duration, Utc};
use paseto::tokens::{PasetoBuilder, TimeBackend};
use paseto::v2::local_paseto;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub account_id: String,
    pub email: String,
}

pub fn issue_token(account_id: &str, email: &str, key: &[u8]) -> Result<String, AppError> {
    let now = Utc::now();
    let expiration = now + Duration::hours(24);

    let token = PasetoBuilder::new()
        .set_encryption_key(key)
        .set_expiration(&expiration)
        .set_not_before(&now)
        .set_claim("account_id", serde_json::json!(account_id))
        .set_claim("email", serde_json::json!(email))
        .build()
        .map_err(|e| AppError::Internal(format!("Token creation failed: {e}")))?;

    Ok(token)
}

pub fn validate_token(token: &str, key: &[u8]) -> Result<Claims, AppError> {
    let verified = paseto::tokens::validate_local_token(
        token,
        None,
        key,
        &TimeBackend::Chrono,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {e}")))?;

    let account_id = verified
        .get("account_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Unauthorized("Missing account_id in token".into()))?
        .to_string();

    let email = verified
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Unauthorized("Missing email in token".into()))?
        .to_string();

    Ok(Claims { account_id, email })
}
```

**Step 2: Write common config types**

`crates/common/src/config.rs`:
```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub name: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.name
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}
```

**Step 3: Verify compilation**

Run: `cargo build -p rust-hour-common`
Expected: PASS

**Step 4: Commit**

```
feat: add common auth (PASETO) and config utilities
```

---

## Phase 2: Auth Service

### Task 2.1: Auth Service — Database Store

**Files:**
- Create: `crates/auth-service/migrations/001_create_accounts.sql`
- Create: `crates/auth-service/src/store.rs`

**Step 1: Write migration**

`crates/auth-service/migrations/001_create_accounts.sql`:
```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS accounts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       VARCHAR(255) UNIQUE NOT NULL,
    password    VARCHAR(255) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_accounts_email ON accounts(email);
```

**Step 2: Write the store**

`crates/auth-service/src/store.rs`:
```rust
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use rust_hour_common::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthStore: Send + Sync {
    async fn create_account(&self, email: &str, password_hash: &str) -> Result<Account, AppError>;
    async fn get_account_by_email(&self, email: &str) -> Result<Account, AppError>;
    async fn get_account_by_id(&self, id: Uuid) -> Result<Account, AppError>;
    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct PgAuthStore {
    pool: PgPool,
}

impl PgAuthStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthStore for PgAuthStore {
    async fn create_account(&self, email: &str, password_hash: &str) -> Result<Account, AppError> {
        let account = sqlx::query_as::<_, Account>(
            "INSERT INTO accounts (email, password) VALUES ($1, $2) RETURNING *"
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(account)
    }

    async fn get_account_by_email(&self, email: &str) -> Result<Account, AppError> {
        let account = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts WHERE email = $1"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(account)
    }

    async fn get_account_by_id(&self, id: Uuid) -> Result<Account, AppError> {
        let account = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(account)
    }

    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE accounts SET password = $1, updated_at = NOW() WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
```

**Step 3: Verify compilation**

Run: `cargo build -p rust-hour-auth`
Expected: PASS

**Step 4: Commit**

```
feat(auth): add database store with account CRUD operations
```

---

### Task 2.2: Auth Service — gRPC Server

**Files:**
- Create: `crates/auth-service/src/grpc.rs`
- Modify: `crates/auth-service/src/main.rs`
- Create: `crates/auth-service/src/password.rs`

**Step 1: Write password hashing utilities**

`crates/auth-service/src/password.rs`:
```rust
use argon2::{self, Config, Variant, Version};
use rand::Rng;

use rust_hour_common::error::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt: [u8; 32] = rand::thread_rng().gen();
    let config = Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        ..Default::default()
    };

    argon2::hash_encoded(password.as_bytes(), &salt, &config)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    argon2::verify_encoded(hash, password.as_bytes())
        .map_err(|e| AppError::Internal(format!("Password verification failed: {e}")))
}
```

**Step 2: Implement gRPC auth service**

`crates/auth-service/src/grpc.rs`:
```rust
use std::sync::Arc;
use tonic::{Request, Response, Status};

use rust_hour_common::auth::{issue_token, validate_token};
use rust_hour_common::proto::auth::auth_service_server::AuthService;
use rust_hour_common::proto::auth::*;

use crate::password::{hash_password, verify_password};
use crate::store::AuthStore;

pub struct AuthGrpcService {
    store: Arc<dyn AuthStore>,
    paseto_key: Vec<u8>,
}

impl AuthGrpcService {
    pub fn new(store: Arc<dyn AuthStore>, paseto_key: Vec<u8>) -> Self {
        Self { store, paseto_key }
    }
}

#[tonic::async_trait]
impl AuthService for AuthGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        // Validate email format
        let email_re = regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
        if !email_re.is_match(&req.email) {
            return Err(Status::invalid_argument("Invalid email format"));
        }

        if req.password.len() < 8 {
            return Err(Status::invalid_argument("Password must be at least 8 characters"));
        }

        let password_hash = hash_password(&req.password).map_err(Status::from)?;
        let account = self.store.create_account(&req.email, &password_hash).await.map_err(Status::from)?;

        let token = issue_token(
            &account.id.to_string(),
            &account.email,
            &self.paseto_key,
        ).map_err(Status::from)?;

        Ok(Response::new(AuthResponse { token }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let account = self.store
            .get_account_by_email(&req.email)
            .await
            .map_err(|_| Status::unauthenticated("Invalid credentials"))?;

        let valid = verify_password(&req.password, &account.password)
            .map_err(Status::from)?;

        if !valid {
            return Err(Status::unauthenticated("Invalid credentials"));
        }

        let token = issue_token(
            &account.id.to_string(),
            &account.email,
            &self.paseto_key,
        ).map_err(Status::from)?;

        Ok(Response::new(AuthResponse { token }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();
        let claims = validate_token(&req.token, &self.paseto_key).map_err(Status::from)?;

        Ok(Response::new(ValidateTokenResponse {
            account_id: claims.account_id,
            email: claims.email,
        }))
    }

    async fn get_account(
        &self,
        request: Request<GetAccountRequest>,
    ) -> Result<Response<AccountResponse>, Status> {
        let req = request.into_inner();
        let id = req.account_id.parse().map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let account = self.store.get_account_by_id(id).await.map_err(Status::from)?;

        Ok(Response::new(AccountResponse {
            id: account.id.to_string(),
            email: account.email,
            created_at: account.created_at.to_rfc3339(),
        }))
    }

    async fn update_password(
        &self,
        request: Request<UpdatePasswordRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        let req = request.into_inner();
        let id = req.account_id.parse().map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        let account = self.store.get_account_by_id(id).await.map_err(Status::from)?;
        let valid = verify_password(&req.old_password, &account.password).map_err(Status::from)?;

        if !valid {
            return Err(Status::unauthenticated("Current password is incorrect"));
        }

        if req.new_password.len() < 8 {
            return Err(Status::invalid_argument("New password must be at least 8 characters"));
        }

        let new_hash = hash_password(&req.new_password).map_err(Status::from)?;
        self.store.update_password(id, &new_hash).await.map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Password updated successfully".into(),
        }))
    }
}
```

**Step 3: Write auth-service main.rs**

```rust
use std::sync::Arc;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use rust_hour_common::proto::auth::auth_service_server::AuthServiceServer;

mod grpc;
mod password;
mod store;

use grpc::AuthGrpcService;
use store::PgAuthStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let database_url = std::env::var("AUTH_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/auth_db".into());

    let paseto_key = std::env::var("PASETO_KEY")
        .expect("PASETO_KEY must be set");

    let pool = sqlx::PgPool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let store = Arc::new(PgAuthStore::new(pool));
    let service = AuthGrpcService::new(store, paseto_key.into_bytes());

    let addr = "[::]:50051".parse()?;
    tracing::info!("Auth service listening on {}", addr);

    Server::builder()
        .add_service(AuthServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
```

**Step 4: Verify compilation**

Run: `cargo build -p rust-hour-auth`
Expected: PASS

**Step 5: Commit**

```
feat(auth): implement gRPC auth service with register, login, token validation
```

---

### Task 2.3: Auth Service — Unit Tests

**Files:**
- Create: `crates/auth-service/src/grpc_tests.rs`

**Step 1: Write tests for auth gRPC handlers using mock store**

`crates/auth-service/src/grpc_tests.rs`:
```rust
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tonic::Request;
    use uuid::Uuid;

    use rust_hour_common::proto::auth::auth_service_server::AuthService;
    use rust_hour_common::proto::auth::*;

    use crate::grpc::AuthGrpcService;
    use crate::store::{Account, MockAuthStore};

    fn test_paseto_key() -> Vec<u8> {
        // 32 bytes for PASETO symmetric key
        b"test-key-that-is-32-bytes-long!!".to_vec()
    }

    fn mock_account() -> Account {
        Account {
            id: Uuid::new_v4(),
            email: "test@example.com".into(),
            password: crate::password::hash_password("password123").unwrap(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_register_success() {
        let mut mock_store = MockAuthStore::new();
        let account = mock_account();
        let account_clone = account.clone();

        mock_store.expect_create_account()
            .returning(move |_, _| Ok(account_clone.clone()));

        let service = AuthGrpcService::new(Arc::new(mock_store), test_paseto_key());

        let request = Request::new(RegisterRequest {
            email: "test@example.com".into(),
            password: "password123".into(),
        });

        let response = service.register(request).await;
        assert!(response.is_ok());
        assert!(!response.unwrap().into_inner().token.is_empty());
    }

    #[tokio::test]
    async fn test_register_invalid_email() {
        let mock_store = MockAuthStore::new();
        let service = AuthGrpcService::new(Arc::new(mock_store), test_paseto_key());

        let request = Request::new(RegisterRequest {
            email: "not-an-email".into(),
            password: "password123".into(),
        });

        let response = service.register(request).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_register_short_password() {
        let mock_store = MockAuthStore::new();
        let service = AuthGrpcService::new(Arc::new(mock_store), test_paseto_key());

        let request = Request::new(RegisterRequest {
            email: "test@example.com".into(),
            password: "short".into(),
        });

        let response = service.register(request).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_login_success() {
        let mut mock_store = MockAuthStore::new();
        let account = mock_account();
        let account_clone = account.clone();

        mock_store.expect_get_account_by_email()
            .returning(move |_| Ok(account_clone.clone()));

        let service = AuthGrpcService::new(Arc::new(mock_store), test_paseto_key());

        let request = Request::new(LoginRequest {
            email: "test@example.com".into(),
            password: "password123".into(),
        });

        let response = service.login(request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let mut mock_store = MockAuthStore::new();
        let account = mock_account();
        let account_clone = account.clone();

        mock_store.expect_get_account_by_email()
            .returning(move |_| Ok(account_clone.clone()));

        let service = AuthGrpcService::new(Arc::new(mock_store), test_paseto_key());

        let request = Request::new(LoginRequest {
            email: "test@example.com".into(),
            password: "wrongpassword".into(),
        });

        let response = service.login(request).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn test_validate_token_roundtrip() {
        let mock_store = MockAuthStore::new();
        let key = test_paseto_key();
        let service = AuthGrpcService::new(Arc::new(mock_store), key.clone());

        // First create a token
        let token = rust_hour_common::auth::issue_token("some-uuid", "test@example.com", &key).unwrap();

        let request = Request::new(ValidateTokenRequest { token });
        let response = service.validate_token(request).await;

        assert!(response.is_ok());
        let resp = response.unwrap().into_inner();
        assert_eq!(resp.account_id, "some-uuid");
        assert_eq!(resp.email, "test@example.com");
    }
}
```

**Step 2: Add module declaration in main.rs or lib.rs**

Add `#[cfg(test)] mod grpc_tests;` to `crates/auth-service/src/main.rs`.

**Step 3: Run tests**

Run: `cargo test -p rust-hour-auth`
Expected: All tests PASS.

**Step 4: Commit**

```
test(auth): add unit tests for auth gRPC handlers
```

---

## Phase 3: Monitor Service

### Task 3.1: Monitor Service — Database Store

**Files:**
- Create: `crates/monitor-service/migrations/001_create_endpoints.sql`
- Create: `crates/monitor-service/migrations/002_create_check_results.sql`
- Create: `crates/monitor-service/src/store.rs`

**Step 1: Write migrations**

`001_create_endpoints.sql`:
```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS endpoints (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL,
    name            VARCHAR(255) NOT NULL,
    url             TEXT NOT NULL,
    method          VARCHAR(10) NOT NULL DEFAULT 'GET',
    headers         JSONB NOT NULL DEFAULT '{}',
    expected_status INT NOT NULL DEFAULT 200,
    interval_secs   INT NOT NULL DEFAULT 60,
    timeout_ms      INT NOT NULL DEFAULT 5000,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_endpoints_account ON endpoints(account_id);
CREATE INDEX idx_endpoints_active ON endpoints(is_active) WHERE is_active = TRUE;
```

`002_create_check_results.sql`:
```sql
CREATE TABLE IF NOT EXISTS check_results (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id      UUID NOT NULL REFERENCES endpoints(id) ON DELETE CASCADE,
    status_code      INT,
    response_time_ms INT,
    is_healthy       BOOLEAN NOT NULL,
    error_message    TEXT,
    checked_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_check_results_endpoint_time ON check_results(endpoint_id, checked_at DESC);
```

**Step 2: Write monitor store with trait + PgPool impl**

Follow the same pattern as `crates/auth-service/src/store.rs`:
- Define `Endpoint` and `CheckResult` structs with `sqlx::FromRow`
- Define `MonitorStore` trait with `#[cfg_attr(test, mockall::automock)]`
- Implement methods: `create_endpoint`, `list_endpoints`, `get_endpoint`, `update_endpoint`, `delete_endpoint`, `insert_check_result`, `get_metrics`, `get_active_endpoints`, `get_latest_check`, `get_uptime_stats`
- Implement `PgMonitorStore` backed by `PgPool`

**Step 3: Verify and commit**

```
feat(monitor): add database store with endpoint and check result operations
```

---

### Task 3.2: Monitor Service — Probe Engine

**Files:**
- Create: `crates/monitor-service/src/probe.rs`
- Create: `crates/monitor-service/src/publisher.rs`

**Step 1: Write the HTTP probe executor**

`crates/monitor-service/src/probe.rs`:
```rust
use std::time::Instant;
use reqwest::Client;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProbeResult {
    pub endpoint_id: String,
    pub status_code: Option<i32>,
    pub response_time_ms: i32,
    pub is_healthy: bool,
    pub error_message: Option<String>,
}

pub async fn execute_probe(
    client: &Client,
    endpoint_id: &str,
    url: &str,
    method: &str,
    headers: &serde_json::Value,
    expected_status: i32,
    timeout_ms: u64,
) -> ProbeResult {
    let start = Instant::now();

    let mut request_builder = match method.to_uppercase().as_str() {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "HEAD" => client.head(url),
        "PATCH" => client.patch(url),
        _ => client.get(url),
    };

    // Apply custom headers
    if let Some(obj) = headers.as_object() {
        for (key, value) in obj {
            if let Some(val) = value.as_str() {
                request_builder = request_builder.header(key.as_str(), val);
            }
        }
    }

    request_builder = request_builder.timeout(std::time::Duration::from_millis(timeout_ms));

    match request_builder.send().await {
        Ok(response) => {
            let elapsed = start.elapsed().as_millis() as i32;
            let status = response.status().as_u16() as i32;
            let is_healthy = status == expected_status;

            ProbeResult {
                endpoint_id: endpoint_id.to_string(),
                status_code: Some(status),
                response_time_ms: elapsed,
                is_healthy,
                error_message: if !is_healthy {
                    Some(format!("Expected status {expected_status}, got {status}"))
                } else {
                    None
                },
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as i32;
            ProbeResult {
                endpoint_id: endpoint_id.to_string(),
                status_code: None,
                response_time_ms: elapsed,
                is_healthy: false,
                error_message: Some(format!("Request failed: {e}")),
            }
        }
    }
}
```

**Step 2: Write Kafka producer**

`crates/monitor-service/src/producer.rs`:
```rust
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct CheckCompletedEvent {
    pub endpoint_id: String,
    pub account_id: String,
    pub is_healthy: bool,
    pub status_code: Option<i32>,
    pub response_time_ms: i32,
    pub error_message: Option<String>,
    pub checked_at: String,
}

pub fn create_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Failed to create Kafka producer")
}

pub async fn publish_check_result(
    producer: &FutureProducer,
    event: &CheckCompletedEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::to_string(event)?;
    let key = &event.endpoint_id; // Partition by endpoint_id for ordering

    producer
        .send(
            FutureRecord::to("check.results")
                .key(key)
                .payload(&payload),
            Duration::from_secs(5),
        )
        .await
        .map_err(|(e, _)| Box::new(e) as Box<dyn std::error::Error>)?;

    // Also publish to monitor.events for alert service
    producer
        .send(
            FutureRecord::to("monitor.events")
                .key(key)
                .payload(&payload),
            Duration::from_secs(5),
        )
        .await
        .map_err(|(e, _)| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(())
}
```

**Step 3: Commit**

```
feat(monitor): add HTTP probe engine and Kafka event producer
```

---

### Task 3.3: Monitor Service — Scheduler

**Files:**
- Create: `crates/monitor-service/src/scheduler.rs`

**Step 1: Write the probe scheduler**

This component fetches active endpoints from the store, schedules them based on `interval_secs`, executes probes, stores results, and publishes events.

```rust
use std::sync::Arc;
use tokio::time::{interval, Duration};
use reqwest::Client;
use chrono::Utc;

use rdkafka::producer::FutureProducer;

use crate::probe::execute_probe;
use crate::producer::{publish_check_result, CheckCompletedEvent};
use crate::store::MonitorStore;

pub async fn run_scheduler(
    store: Arc<dyn MonitorStore>,
    kafka_producer: FutureProducer,
    http_client: Client,
) {
    let mut tick = interval(Duration::from_secs(10)); // Check for work every 10s

    loop {
        tick.tick().await;

        let endpoints = match store.get_active_endpoints().await {
            Ok(eps) => eps,
            Err(e) => {
                tracing::error!("Failed to fetch active endpoints: {e}");
                continue;
            }
        };

        for endpoint in endpoints {
            let producer = kafka_producer.clone();
            let client = http_client.clone();

            tokio::spawn(async move {
                let result = execute_probe(
                    &client,
                    &endpoint.id.to_string(),
                    &endpoint.url,
                    &endpoint.method,
                    &endpoint.headers,
                    endpoint.expected_status,
                    endpoint.timeout_ms as u64,
                ).await;

                // CQRS: Write to Kafka (not directly to DB)
                // The monitor-query-service consumer will persist to DB
                let event = CheckCompletedEvent {
                    endpoint_id: endpoint.id.to_string(),
                    account_id: endpoint.account_id.to_string(),
                    is_healthy: result.is_healthy,
                    status_code: result.status_code,
                    response_time_ms: result.response_time_ms,
                    error_message: result.error_message,
                    checked_at: Utc::now().to_rfc3339(),
                };

                if let Err(e) = publish_check_result(&producer, &event).await {
                    tracing::error!("Failed to publish check result to Kafka: {e}");
                }
            });
        }
    }
}
```

**Note:** A production scheduler would track per-endpoint intervals. For v1 this simpler approach polls all active endpoints every 10 seconds. The endpoint's `interval_secs` can be respected in a future iteration with `tokio-cron-scheduler`.

**Step 2: Commit**

```
feat(monitor): add probe scheduling loop
```

---

### Task 3.4: Monitor Service — gRPC Server & Main

**Files:**
- Create: `crates/monitor-service/src/grpc.rs`
- Modify: `crates/monitor-service/src/main.rs`

**Step 1: Implement MonitorService gRPC trait**

Follow same pattern as auth-service grpc.rs:
- Accept `Arc<dyn MonitorStore>` in constructor
- Implement all methods from `monitor.proto`
- Parse UUIDs, validate ownership via `account_id`
- Return proper gRPC status codes

**Step 2: Write main.rs**

Same pattern as auth-service:
- Initialize tracing
- Connect to `MONITOR_DATABASE_URL`
- Run migrations
- Create Kafka producer (`KAFKA_BROKERS` env var)
- Spawn scheduler task (with Kafka producer)
- Start gRPC server on `[::]:50052`

**Note:** With CQRS, the monitor-service does NOT write check results to the database directly. It writes them to Kafka. The `monitor-query-service` consumes from Kafka and persists to DB. The monitor-service only uses its DB for endpoint CRUD (metadata).

**Step 3: Commit**

```
feat(monitor): implement gRPC server and service main entrypoint
```

---

### Task 3.5: Monitor Service — Unit Tests

**Files:**
- Create: `crates/monitor-service/src/probe_tests.rs`
- Create: `crates/monitor-service/src/grpc_tests.rs`

**Step 1: Write probe tests** (mock HTTP server with `httpmock` or test against known URLs)

**Step 2: Write gRPC handler tests** (same pattern as auth tests — mock store, test success/error paths)

**Step 3: Commit**

```
test(monitor): add unit tests for probe engine and gRPC handlers
```

---

## Phase 4: Alert Service

### Task 4.1: Alert Service — Database Store

**Files:**
- Create: `crates/alert-service/migrations/001_create_alert_rules.sql`
- Create: `crates/alert-service/migrations/002_create_alert_channels.sql`
- Create: `crates/alert-service/migrations/003_create_alert_history.sql`
- Create: `crates/alert-service/src/store.rs`

**Step 1: Write migrations** (use schemas from design doc)

**Step 2: Write AlertStore trait + PgAlertStore**

Methods: `create_rule`, `list_rules`, `update_rule`, `delete_rule`, `create_channel`, `list_channels`, `delete_channel`, `insert_alert_event`, `get_history`, `get_rules_for_endpoint`

Follow same pattern as auth/monitor stores.

**Step 3: Commit**

```
feat(alert): add database store with alert rules, channels, and history
```

---

### Task 4.2: Alert Service — Rule Evaluator & NATS Subscriber

**Files:**
- Create: `crates/alert-service/src/evaluator.rs`
- Create: `crates/alert-service/src/subscriber.rs`
- Create: `crates/alert-service/src/notifier.rs`

**Step 1: Write the rule evaluator**

`crates/alert-service/src/evaluator.rs`:
```rust
use crate::store::{AlertRule, AlertStore};
use std::sync::Arc;

pub struct CheckEvent {
    pub endpoint_id: String,
    pub account_id: String,
    pub is_healthy: bool,
    pub status_code: Option<i32>,
    pub response_time_ms: i32,
}

pub async fn evaluate_rules(
    store: &Arc<dyn AlertStore>,
    event: &CheckEvent,
) -> Vec<AlertRule> {
    let endpoint_id = match event.endpoint_id.parse() {
        Ok(id) => id,
        Err(_) => return vec![],
    };

    let rules = match store.get_rules_for_endpoint(endpoint_id).await {
        Ok(rules) => rules,
        Err(_) => return vec![],
    };

    rules.into_iter().filter(|rule| {
        match rule.rule_type.as_str() {
            "down" => !event.is_healthy,
            "latency_threshold" => event.response_time_ms > rule.threshold_value,
            "status_mismatch" => !event.is_healthy,
            _ => false,
        }
    }).collect()
}
```

**Step 2: Write Kafka consumer**

`crates/alert-service/src/consumer.rs`:
```rust
use std::sync::Arc;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use futures::StreamExt;

use crate::evaluator::{evaluate_rules, CheckEvent};
use crate::notifier::send_notifications;
use crate::store::AlertStore;

pub fn create_consumer(brokers: &str, group_id: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Failed to create Kafka consumer")
}

pub async fn consume_monitor_events(
    consumer: StreamConsumer,
    store: Arc<dyn AlertStore>,
) {
    consumer.subscribe(&["monitor.events"]).expect("Failed to subscribe to topic");

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    let event: CheckEvent = match serde_json::from_slice(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::error!("Failed to parse check event: {e}");
                            consumer.commit_message(&message, CommitMode::Async).unwrap();
                            continue;
                        }
                    };

                    let triggered_rules = evaluate_rules(&store, &event).await;

                    if !triggered_rules.is_empty() {
                        if let Err(e) = send_notifications(&store, &event, &triggered_rules).await {
                            tracing::error!("Failed to send notifications: {e}");
                        }
                    }

                    consumer.commit_message(&message, CommitMode::Async).unwrap();
                }
            }
            Err(e) => tracing::error!("Kafka consumer error: {e}"),
        }
    }
}
```

**Step 3: Write notifier** (webhook dispatcher)

`crates/alert-service/src/notifier.rs`:
```rust
use std::sync::Arc;
use reqwest::Client;

use crate::evaluator::CheckEvent;
use crate::store::{AlertRule, AlertStore};
use rust_hour_common::error::AppError;

static HTTP_CLIENT: std::sync::LazyLock<Client> = std::sync::LazyLock::new(Client::new);

pub async fn send_notifications(
    store: &Arc<dyn AlertStore>,
    event: &CheckEvent,
    rules: &[AlertRule],
) -> Result<(), AppError> {
    let account_id = event.account_id.parse()
        .map_err(|_| AppError::Internal("Invalid account_id".into()))?;

    let channels = store.list_channels(account_id, 100, 0).await?;

    for rule in rules {
        for channel in &channels {
            let message = format!(
                "Alert: endpoint {} triggered rule '{}' (response_time: {}ms, healthy: {})",
                event.endpoint_id, rule.rule_type, event.response_time_ms, event.is_healthy
            );

            match channel.channel_type.as_str() {
                "webhook" => {
                    if let Some(url) = channel.config.get("url").and_then(|v| v.as_str()) {
                        let payload = serde_json::json!({
                            "text": message,
                            "endpoint_id": event.endpoint_id,
                            "rule_type": rule.rule_type,
                            "is_healthy": event.is_healthy,
                        });

                        if let Err(e) = HTTP_CLIENT.post(url).json(&payload).send().await {
                            tracing::error!("Webhook delivery failed: {e}");
                        }
                    }
                }
                // Future: "email", "slack"
                _ => {
                    tracing::warn!("Unsupported channel type: {}", channel.channel_type);
                }
            }

            // Record in history
            let _ = store.insert_alert_event(
                rule.id,
                channel.id,
                "sent",
                &message,
            ).await;
        }
    }

    Ok(())
}
```

**Step 4: Commit**

```
feat(alert): add rule evaluator, Kafka consumer, and webhook notifier
```

---

### Task 4.3: Alert Service — gRPC Server & Main

**Files:**
- Create: `crates/alert-service/src/grpc.rs`
- Modify: `crates/alert-service/src/main.rs`

**Step 1: Implement AlertService gRPC trait** (same pattern as auth/monitor)

**Step 2: Write main.rs**
- Initialize tracing
- Connect to `ALERT_DATABASE_URL`
- Run migrations
- Create Kafka consumer (`KAFKA_BROKERS`, consumer group `alert-service`)
- Spawn Kafka consumer task
- Start gRPC server on `[::]:50053`

**Step 3: Commit**

```
feat(alert): implement gRPC server and service main entrypoint
```

---

### Task 4.4: Alert Service — Unit Tests

**Files:**
- Create: `crates/alert-service/src/evaluator_tests.rs`

**Step 1: Test rule evaluation logic**

- Test "down" rule triggers when `is_healthy = false`
- Test "latency_threshold" rule triggers when response_time > threshold
- Test rules don't trigger on healthy checks
- Test multiple rules for same endpoint

**Step 2: Commit**

```
test(alert): add unit tests for rule evaluator
```

---

## Phase 5: CQRS — Monitor Query Service (Read Side)

### Task 5.1: Monitor Query Service — Kafka Consumer & Projector

**Files:**
- Create: `crates/monitor-query-service/src/projector.rs`
- Create: `crates/monitor-query-service/src/store.rs`
- Create: `crates/monitor-query-service/migrations/001_create_check_results.sql`
- Create: `crates/monitor-query-service/migrations/002_create_materialized_views.sql`

**Step 1: Write migrations**

`001_create_check_results.sql`:
```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS check_results (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id      UUID NOT NULL,
    status_code      INT,
    response_time_ms INT,
    is_healthy       BOOLEAN NOT NULL,
    error_message    TEXT,
    checked_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_check_results_endpoint_time ON check_results(endpoint_id, checked_at DESC);
CREATE INDEX idx_check_results_checked_at ON check_results(checked_at);
```

`002_create_materialized_views.sql`:
```sql
CREATE MATERIALIZED VIEW IF NOT EXISTS endpoint_uptime_hourly AS
SELECT
    endpoint_id,
    date_trunc('hour', checked_at) AS hour,
    COUNT(*) AS total_checks,
    COUNT(*) FILTER (WHERE is_healthy) AS healthy_checks,
    ROUND(100.0 * COUNT(*) FILTER (WHERE is_healthy) / COUNT(*), 2) AS uptime_pct
FROM check_results
GROUP BY endpoint_id, date_trunc('hour', checked_at);

CREATE UNIQUE INDEX idx_uptime_hourly_pk ON endpoint_uptime_hourly(endpoint_id, hour);

CREATE MATERIALIZED VIEW IF NOT EXISTS endpoint_latency_hourly AS
SELECT
    endpoint_id,
    date_trunc('hour', checked_at) AS hour,
    AVG(response_time_ms)::INT AS avg_ms,
    PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY response_time_ms)::INT AS p50_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::INT AS p95_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY response_time_ms)::INT AS p99_ms
FROM check_results
GROUP BY endpoint_id, date_trunc('hour', checked_at);

CREATE UNIQUE INDEX idx_latency_hourly_pk ON endpoint_latency_hourly(endpoint_id, hour);
```

**Step 2: Write the Kafka consumer projector**

The projector consumes from `check.results` topic, writes raw data to PostgreSQL, and periodically refreshes materialized views.

`crates/monitor-query-service/src/projector.rs`:
```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use futures::StreamExt;
use uuid::Uuid;
use chrono::DateTime;

use crate::store::QueryStore;

pub fn create_consumer(brokers: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", "monitor-query-projector")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest") // Process all historical data
        .create()
        .expect("Failed to create Kafka consumer")
}

#[derive(serde::Deserialize)]
struct CheckResultEvent {
    endpoint_id: String,
    is_healthy: bool,
    status_code: Option<i32>,
    response_time_ms: i32,
    error_message: Option<String>,
    checked_at: String,
}

pub async fn run_projector(
    consumer: StreamConsumer,
    store: Arc<dyn QueryStore>,
) {
    consumer.subscribe(&["check.results"]).expect("Failed to subscribe");

    let mut stream = consumer.stream();
    let mut last_refresh = Instant::now();
    let refresh_interval = Duration::from_secs(60); // Refresh views every 60s

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    if let Ok(event) = serde_json::from_slice::<CheckResultEvent>(payload) {
                        let endpoint_id: Uuid = match event.endpoint_id.parse() {
                            Ok(id) => id,
                            Err(_) => {
                                consumer.commit_message(&message, CommitMode::Async).unwrap();
                                continue;
                            }
                        };

                        let checked_at = DateTime::parse_from_rfc3339(&event.checked_at)
                            .unwrap_or_else(|_| chrono::Utc::now().into())
                            .with_timezone(&chrono::Utc);

                        if let Err(e) = store.insert_check_result(
                            endpoint_id,
                            event.status_code,
                            event.response_time_ms,
                            event.is_healthy,
                            event.error_message.as_deref(),
                            checked_at,
                        ).await {
                            tracing::error!("Failed to insert check result: {e}");
                        }

                        consumer.commit_message(&message, CommitMode::Async).unwrap();
                    }
                }

                // Periodically refresh materialized views
                if last_refresh.elapsed() > refresh_interval {
                    if let Err(e) = store.refresh_materialized_views().await {
                        tracing::error!("Failed to refresh materialized views: {e}");
                    }
                    last_refresh = Instant::now();
                }
            }
            Err(e) => tracing::error!("Kafka consumer error: {e}"),
        }
    }
}
```

**Step 3: Write the query store**

`crates/monitor-query-service/src/store.rs`:
```rust
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
    pub uptime_pct: f64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LatencyHourly {
    pub endpoint_id: Uuid,
    pub hour: DateTime<Utc>,
    pub avg_ms: i32,
    pub p50_ms: i32,
    pub p95_ms: i32,
    pub p99_ms: i32,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QueryStore: Send + Sync {
    async fn insert_check_result(
        &self, endpoint_id: Uuid, status_code: Option<i32>,
        response_time_ms: i32, is_healthy: bool, error_message: Option<&str>,
        checked_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn get_check_results(
        &self, endpoint_id: Uuid, from: DateTime<Utc>, to: DateTime<Utc>,
        limit: i32, offset: i32,
    ) -> Result<Vec<CheckResult>, AppError>;

    async fn get_uptime_hourly(
        &self, endpoint_id: Uuid, from: DateTime<Utc>, to: DateTime<Utc>,
    ) -> Result<Vec<UptimeHourly>, AppError>;

    async fn get_latency_hourly(
        &self, endpoint_id: Uuid, from: DateTime<Utc>, to: DateTime<Utc>,
    ) -> Result<Vec<LatencyHourly>, AppError>;

    async fn get_latest_check(&self, endpoint_id: Uuid) -> Result<Option<CheckResult>, AppError>;

    async fn refresh_materialized_views(&self) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct PgQueryStore {
    pool: PgPool,
}

impl PgQueryStore {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl QueryStore for PgQueryStore {
    async fn insert_check_result(
        &self, endpoint_id: Uuid, status_code: Option<i32>,
        response_time_ms: i32, is_healthy: bool, error_message: Option<&str>,
        checked_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO check_results (endpoint_id, status_code, response_time_ms, is_healthy, error_message, checked_at)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(endpoint_id).bind(status_code).bind(response_time_ms)
        .bind(is_healthy).bind(error_message).bind(checked_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    async fn get_check_results(
        &self, endpoint_id: Uuid, from: DateTime<Utc>, to: DateTime<Utc>,
        limit: i32, offset: i32,
    ) -> Result<Vec<CheckResult>, AppError> {
        let results = sqlx::query_as::<_, CheckResult>(
            "SELECT * FROM check_results WHERE endpoint_id = $1 AND checked_at BETWEEN $2 AND $3
             ORDER BY checked_at DESC LIMIT $4 OFFSET $5"
        )
        .bind(endpoint_id).bind(from).bind(to).bind(limit).bind(offset)
        .fetch_all(&self.pool).await?;
        Ok(results)
    }

    async fn get_uptime_hourly(
        &self, endpoint_id: Uuid, from: DateTime<Utc>, to: DateTime<Utc>,
    ) -> Result<Vec<UptimeHourly>, AppError> {
        let results = sqlx::query_as::<_, UptimeHourly>(
            "SELECT * FROM endpoint_uptime_hourly WHERE endpoint_id = $1 AND hour BETWEEN $2 AND $3
             ORDER BY hour DESC"
        )
        .bind(endpoint_id).bind(from).bind(to)
        .fetch_all(&self.pool).await?;
        Ok(results)
    }

    async fn get_latency_hourly(
        &self, endpoint_id: Uuid, from: DateTime<Utc>, to: DateTime<Utc>,
    ) -> Result<Vec<LatencyHourly>, AppError> {
        let results = sqlx::query_as::<_, LatencyHourly>(
            "SELECT * FROM endpoint_latency_hourly WHERE endpoint_id = $1 AND hour BETWEEN $2 AND $3
             ORDER BY hour DESC"
        )
        .bind(endpoint_id).bind(from).bind(to)
        .fetch_all(&self.pool).await?;
        Ok(results)
    }

    async fn get_latest_check(&self, endpoint_id: Uuid) -> Result<Option<CheckResult>, AppError> {
        let result = sqlx::query_as::<_, CheckResult>(
            "SELECT * FROM check_results WHERE endpoint_id = $1 ORDER BY checked_at DESC LIMIT 1"
        )
        .bind(endpoint_id)
        .fetch_optional(&self.pool).await?;
        Ok(result)
    }

    async fn refresh_materialized_views(&self) -> Result<(), AppError> {
        sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY endpoint_uptime_hourly")
            .execute(&self.pool).await?;
        sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY endpoint_latency_hourly")
            .execute(&self.pool).await?;
        tracing::info!("Materialized views refreshed");
        Ok(())
    }
}
```

**Step 4: Commit**

```
feat(monitor-query): add CQRS read side with Kafka projector and materialized views
```

---

### Task 5.2: Monitor Query Service — gRPC Server, Redis Cache & Main

**Files:**
- Create: `crates/monitor-query-service/src/grpc.rs`
- Create: `crates/monitor-query-service/src/cache.rs`
- Modify: `crates/monitor-query-service/src/main.rs`

**Step 1: Write Redis cache layer**

`crates/monitor-query-service/src/cache.rs`:
```rust
use fred::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct MetricsCache {
    client: RedisClient,
}

impl MetricsCache {
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = RedisConfig::from_url(redis_url)?;
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await?;
        Ok(Self { client })
    }

    pub async fn get_or_set<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        fetch: F,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        // Try cache first
        let cached: Option<String> = self.client.get(key).await?;
        if let Some(data) = cached {
            if let Ok(value) = serde_json::from_str(&data) {
                return Ok(value);
            }
        }

        // Cache miss — fetch from store
        let value = fetch().await?;
        let serialized = serde_json::to_string(&value)?;
        let _: () = self.client.set(key, serialized, Some(Expiration::EX(ttl.as_secs() as i64)), None, false).await?;
        Ok(value)
    }
}
```

**Step 2: Implement MonitorQueryService gRPC trait**

The gRPC server wraps the query store with Redis caching:
- `GetMetrics` → cache key `metrics:{endpoint_id}:{from}:{to}` (TTL 30s)
- `GetEndpointDetail` → cache key `detail:{endpoint_id}` (TTL 15s)
- Uses materialized views for uptime/latency aggregations

**Step 3: Write main.rs**

- Initialize tracing
- Connect to `MONITOR_DATABASE_URL` (same DB, but reads from materialized views)
- Run migrations
- Connect to Redis
- Create Kafka consumer
- Spawn projector task
- Start gRPC server on `[::]:50054`

**Step 4: Commit**

```
feat(monitor-query): add gRPC server with Redis caching for CQRS read side
```

---

### Task 5.3: Monitor Query Service — Unit Tests

**Files:**
- Create: `crates/monitor-query-service/src/projector_tests.rs`

**Step 1: Test projector logic** — mock store, verify it correctly parses Kafka messages and persists

**Step 2: Test cache layer** — verify cache hit returns cached value, cache miss calls fetch

**Step 3: Commit**

```
test(monitor-query): add unit tests for CQRS projector and cache
```

---

## Phase 6: Saga Orchestrator

### Task 6.1: Saga — State Store

**Files:**
- Create: `crates/saga-orchestrator/migrations/001_create_saga_tables.sql`
- Create: `crates/saga-orchestrator/src/state_store.rs`

**Step 1: Write migration**

```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS saga_instances (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_type   VARCHAR(50) NOT NULL,
    account_id  UUID NOT NULL,
    status      VARCHAR(20) NOT NULL DEFAULT 'running',
    current_step INT NOT NULL DEFAULT 0,
    context     JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS saga_step_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_id         UUID NOT NULL REFERENCES saga_instances(id) ON DELETE CASCADE,
    step_number     INT NOT NULL,
    step_name       VARCHAR(100) NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',
    command_payload JSONB,
    reply_payload   JSONB,
    executed_at     TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_saga_instances_status ON saga_instances(status) WHERE status = 'running';
CREATE INDEX idx_saga_step_log_saga ON saga_step_log(saga_id, step_number);
```

**Step 2: Write SagaStateStore trait + PgSagaStateStore**

```rust
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use rust_hour_common::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SagaInstance {
    pub id: Uuid,
    pub saga_type: String,
    pub account_id: Uuid,
    pub status: String,       // "running", "completed", "compensating", "failed"
    pub current_step: i32,
    pub context: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SagaStepLog {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub step_number: i32,
    pub step_name: String,
    pub status: String,       // "pending", "completed", "failed", "compensated"
    pub command_payload: Option<serde_json::Value>,
    pub reply_payload: Option<serde_json::Value>,
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SagaStateStore: Send + Sync {
    async fn create_saga(&self, saga_type: &str, account_id: Uuid) -> Result<SagaInstance, AppError>;
    async fn get_saga(&self, id: Uuid) -> Result<SagaInstance, AppError>;
    async fn update_saga_status(&self, id: Uuid, status: &str, step: i32, context: &serde_json::Value) -> Result<(), AppError>;
    async fn log_step(&self, saga_id: Uuid, step_number: i32, step_name: &str, status: &str, command: Option<&serde_json::Value>, reply: Option<&serde_json::Value>) -> Result<(), AppError>;
    async fn get_steps(&self, saga_id: Uuid) -> Result<Vec<SagaStepLog>, AppError>;
}

// PgSagaStateStore implementation follows standard SQLx pattern
```

**Step 3: Commit**

```
feat(saga): add saga state store with instance and step tracking
```

---

### Task 6.2: Saga — Orchestrator Engine

**Files:**
- Create: `crates/saga-orchestrator/src/orchestrator.rs`
- Create: `crates/saga-orchestrator/src/steps.rs`

**Step 1: Define the saga step trait and orchestrator**

`crates/saga-orchestrator/src/steps.rs`:
```rust
use async_trait::async_trait;
use serde_json::Value;
use rust_hour_common::error::AppError;

#[async_trait]
pub trait SagaStep: Send + Sync {
    fn name(&self) -> &str;

    /// Execute the forward action
    async fn execute(&self, context: &Value) -> Result<Value, AppError>;

    /// Execute the compensating (rollback) action
    async fn compensate(&self, context: &Value) -> Result<(), AppError>;
}
```

`crates/saga-orchestrator/src/orchestrator.rs`:
```rust
use std::sync::Arc;
use uuid::Uuid;

use crate::state_store::SagaStateStore;
use crate::steps::SagaStep;
use rust_hour_common::error::AppError;

pub struct SagaOrchestrator {
    state_store: Arc<dyn SagaStateStore>,
    steps: Vec<Arc<dyn SagaStep>>,
}

impl SagaOrchestrator {
    pub fn new(state_store: Arc<dyn SagaStateStore>, steps: Vec<Arc<dyn SagaStep>>) -> Self {
        Self { state_store, steps }
    }

    pub async fn run(&self, saga_type: &str, account_id: Uuid) -> Result<(), AppError> {
        let saga = self.state_store.create_saga(saga_type, account_id).await?;
        let mut context = saga.context.clone();

        for (i, step) in self.steps.iter().enumerate() {
            let step_num = i as i32;

            // Log step as pending
            self.state_store.log_step(
                saga.id, step_num, step.name(), "pending", None, None,
            ).await?;

            // Execute step
            match step.execute(&context).await {
                Ok(reply) => {
                    // Merge reply into context
                    if let Some(obj) = reply.as_object() {
                        for (k, v) in obj {
                            context[k] = v.clone();
                        }
                    }

                    self.state_store.log_step(
                        saga.id, step_num, step.name(), "completed",
                        Some(&context), Some(&reply),
                    ).await?;
                    self.state_store.update_saga_status(
                        saga.id, "running", step_num + 1, &context,
                    ).await?;
                }
                Err(e) => {
                    tracing::error!("Saga step '{}' failed: {e}", step.name());
                    self.state_store.log_step(
                        saga.id, step_num, step.name(), "failed", None, None,
                    ).await?;
                    self.state_store.update_saga_status(
                        saga.id, "compensating", step_num, &context,
                    ).await?;

                    // Compensate all completed steps in reverse order
                    self.compensate(saga.id, i, &context).await?;
                    return Err(e);
                }
            }
        }

        self.state_store.update_saga_status(
            saga.id, "completed", self.steps.len() as i32, &context,
        ).await?;

        Ok(())
    }

    async fn compensate(
        &self,
        saga_id: Uuid,
        failed_step: usize,
        context: &serde_json::Value,
    ) -> Result<(), AppError> {
        // Compensate in reverse order, up to (not including) the failed step
        for i in (0..failed_step).rev() {
            let step = &self.steps[i];
            tracing::info!("Compensating step '{}'", step.name());

            match step.compensate(context).await {
                Ok(()) => {
                    self.state_store.log_step(
                        saga_id, i as i32, step.name(), "compensated", None, None,
                    ).await?;
                }
                Err(e) => {
                    tracing::error!("Compensation for '{}' failed: {e}", step.name());
                    self.state_store.log_step(
                        saga_id, i as i32, step.name(), "compensation_failed", None, None,
                    ).await?;
                    self.state_store.update_saga_status(
                        saga_id, "failed", i as i32, context,
                    ).await?;
                    return Err(e);
                }
            }
        }

        self.state_store.update_saga_status(
            saga_id, "compensated", 0, context,
        ).await?;

        Ok(())
    }
}
```

**Step 2: Commit**

```
feat(saga): add orchestrator engine with forward execution and compensation
```

---

### Task 6.3: Saga — Account Deletion Steps

**Files:**
- Create: `crates/saga-orchestrator/src/account_deletion.rs`

**Step 1: Implement the 4 saga steps for account deletion**

Each step makes a gRPC call to the respective service:

```rust
use std::sync::Arc;
use async_trait::async_trait;
use tonic::transport::Channel;

use crate::steps::SagaStep;
use rust_hour_common::error::AppError;
use rust_hour_common::proto::auth::auth_service_client::AuthServiceClient;
use rust_hour_common::proto::monitor::monitor_service_client::MonitorServiceClient;
use rust_hour_common::proto::alert::alert_service_client::AlertServiceClient;

// Step 1: Mark account as "deleting" in auth service
pub struct MarkAccountDeletingStep {
    auth_client: AuthServiceClient<Channel>,
}

#[async_trait]
impl SagaStep for MarkAccountDeletingStep {
    fn name(&self) -> &str { "mark_account_deleting" }

    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        let account_id = context["account_id"].as_str()
            .ok_or(AppError::Internal("Missing account_id".into()))?;
        // Call auth service to mark account as deleting
        // (requires adding a MarkDeleting RPC to auth.proto)
        Ok(serde_json::json!({"auth_marked": true}))
    }

    async fn compensate(&self, context: &serde_json::Value) -> Result<(), AppError> {
        // Revert account status to "active"
        Ok(())
    }
}

// Step 2: Delete all endpoints in monitor service
pub struct DeleteEndpointsStep {
    monitor_client: MonitorServiceClient<Channel>,
}

#[async_trait]
impl SagaStep for DeleteEndpointsStep {
    fn name(&self) -> &str { "delete_endpoints" }

    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        // Call monitor service to soft-delete all endpoints for account
        Ok(serde_json::json!({"endpoints_deleted": true}))
    }

    async fn compensate(&self, context: &serde_json::Value) -> Result<(), AppError> {
        // Restore soft-deleted endpoints
        Ok(())
    }
}

// Step 3: Delete all alert rules and channels
pub struct DeleteAlertsStep {
    alert_client: AlertServiceClient<Channel>,
}

#[async_trait]
impl SagaStep for DeleteAlertsStep {
    fn name(&self) -> &str { "delete_alerts" }

    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        // Call alert service to delete all rules/channels for account
        Ok(serde_json::json!({"alerts_deleted": true}))
    }

    async fn compensate(&self, context: &serde_json::Value) -> Result<(), AppError> {
        // Restore soft-deleted rules/channels
        Ok(())
    }
}

// Step 4: Permanently delete account
pub struct PermanentlyDeleteAccountStep {
    auth_client: AuthServiceClient<Channel>,
}

#[async_trait]
impl SagaStep for PermanentlyDeleteAccountStep {
    fn name(&self) -> &str { "permanently_delete_account" }

    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        // Call auth service to permanently delete account
        Ok(serde_json::json!({"account_deleted": true}))
    }

    async fn compensate(&self, _context: &serde_json::Value) -> Result<(), AppError> {
        // Cannot compensate permanent deletion — this is the last step
        // If we reach compensation here, something is wrong
        Err(AppError::Internal("Cannot compensate permanent deletion".into()))
    }
}
```

**Note:** The actual gRPC calls need corresponding RPCs added to the proto files:
- `auth.proto`: add `MarkDeleting`, `RevertDeleting`, `PermanentlyDelete`
- `monitor.proto`: add `DeleteAllEndpoints`, `RestoreEndpoints`
- `alert.proto`: add `DeleteAllForAccount`, `RestoreForAccount`

**Step 2: Commit**

```
feat(saga): add account deletion saga with 4 steps and compensation logic
```

---

### Task 6.4: Saga — Main & Integration with Gateway

**Files:**
- Modify: `crates/saga-orchestrator/src/main.rs`

**Step 1: Write main.rs**

The saga orchestrator can be:
- A standalone gRPC service that the gateway calls for saga operations
- Or invoked via Kafka commands

For simplicity, expose it as a gRPC service:
- `StartAccountDeletion(account_id) → saga_id`
- `GetSagaStatus(saga_id) → status, steps`

**Step 2: Commit**

```
feat(saga): add saga orchestrator gRPC service and main entrypoint
```

---

### Task 6.5: Saga — Unit Tests

**Files:**
- Create: `crates/saga-orchestrator/src/orchestrator_tests.rs`

**Step 1: Test happy path** — all steps succeed, saga completes

**Step 2: Test compensation** — step 2 fails, verify step 1 is compensated

**Step 3: Test step 3 failure** — verify steps 1 and 2 are compensated in reverse

**Step 4: Commit**

```
test(saga): add unit tests for orchestrator forward execution and compensation
```

---

## Phase 7: API Gateway

### Task 7.1: Gateway — gRPC Clients

**Files:**
- Create: `crates/gateway/src/grpc_clients.rs`

**Step 1: Write gRPC client wrapper**

```rust
use rust_hour_common::proto::auth::auth_service_client::AuthServiceClient;
use rust_hour_common::proto::monitor::monitor_service_client::MonitorServiceClient;
use rust_hour_common::proto::alert::alert_service_client::AlertServiceClient;
use tonic::transport::Channel;

// Note: monitor-query-service uses the same monitor.proto MonitorServiceClient
// but connects to a different URL (port 50054)

#[derive(Clone)]
pub struct GrpcClients {
    pub auth: AuthServiceClient<Channel>,
    pub monitor_write: MonitorServiceClient<Channel>,  // CQRS write side (endpoint CRUD)
    pub monitor_read: MonitorServiceClient<Channel>,   // CQRS read side (metrics queries)
    pub alert: AlertServiceClient<Channel>,
    // Saga orchestrator called via gateway for account deletion
    pub saga: Option<tonic::transport::Channel>,
}

impl GrpcClients {
    pub async fn connect(
        auth_url: &str,
        monitor_write_url: &str,
        monitor_read_url: &str,
        alert_url: &str,
        saga_url: &str,
    ) -> Result<Self, tonic::transport::Error> {
        let auth = AuthServiceClient::connect(auth_url.to_string()).await?;
        let monitor_write = MonitorServiceClient::connect(monitor_write_url.to_string()).await?;
        let monitor_read = MonitorServiceClient::connect(monitor_read_url.to_string()).await?;
        let alert = AlertServiceClient::connect(alert_url.to_string()).await?;
        let saga = tonic::transport::Channel::from_shared(saga_url.to_string())
            .ok()
            .map(|c| futures::executor::block_on(c.connect()).ok())
            .flatten();

        Ok(Self { auth, monitor_write, monitor_read, alert, saga })
    }
}
```

**Step 2: Commit**

```
feat(gateway): add gRPC client connections to backend services
```

---

### Task 7.2: Gateway — Auth Middleware

**Files:**
- Create: `crates/gateway/src/middleware/mod.rs`
- Create: `crates/gateway/src/middleware/auth.rs`

**Step 1: Write Axum auth extractor that calls auth-service ValidateToken**

```rust
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
};

use crate::grpc_clients::GrpcClients;
use rust_hour_common::proto::auth::ValidateTokenRequest;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub account_id: String,
    pub email: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    GrpcClients: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = parts.headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header".into()))?;

        let mut clients = GrpcClients::from_ref(state);
        let response = clients.auth
            .validate_token(ValidateTokenRequest { token: token.to_string() })
            .await
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {e}")))?;

        let inner = response.into_inner();
        Ok(AuthUser {
            account_id: inner.account_id,
            email: inner.email,
        })
    }
}
```

**Note:** This extractor calls the auth service on every authenticated request. This is the microservice pattern — each service validates via the auth service. For optimization, add Redis caching of validated tokens later.

**Step 2: Commit**

```
feat(gateway): add auth middleware that validates tokens via auth-service
```

---

### Task 7.3: Gateway — Route Handlers

**Files:**
- Create: `crates/gateway/src/routes/mod.rs`
- Create: `crates/gateway/src/routes/auth.rs`
- Create: `crates/gateway/src/routes/endpoints.rs`
- Create: `crates/gateway/src/routes/alerts.rs`

**Step 1: Write auth routes**

Each route handler:
1. Extracts request body (JSON)
2. Calls the appropriate gRPC service
3. Maps gRPC response → JSON HTTP response
4. Maps gRPC errors → HTTP status codes

Example pattern for `POST /api/v1/auth/register`:
```rust
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::grpc_clients::GrpcClients;
use crate::error::ApiError;
use rust_hour_common::proto::auth::RegisterRequest;

#[derive(Deserialize)]
pub struct RegisterBody {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

pub async fn register(
    State(mut clients): State<GrpcClients>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let response = clients.auth
        .register(RegisterRequest {
            email: body.email,
            password: body.password,
        })
        .await?;

    Ok(Json(TokenResponse {
        token: response.into_inner().token,
    }))
}
```

**Step 2: Write endpoint routes** (same pattern, uses `AuthUser` extractor for protected routes)

**Step 3: Write alert routes** (same pattern)

**Step 4: Commit**

```
feat(gateway): add REST route handlers for auth, endpoints, and alerts
```

---

### Task 7.4: Gateway — Router & Main

**Files:**
- Create: `crates/gateway/src/error.rs`
- Modify: `crates/gateway/src/main.rs`

**Step 1: Write API error type that maps gRPC status → HTTP response**

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct ApiError(tonic::Status);

impl From<tonic::Status> for ApiError {
    fn from(status: tonic::Status) -> Self {
        ApiError(status)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let http_status = match self.0.code() {
            tonic::Code::NotFound => StatusCode::NOT_FOUND,
            tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
            tonic::Code::AlreadyExists => StatusCode::CONFLICT,
            tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
            tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
            tonic::Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = serde_json::json!({
            "error": self.0.message(),
        });

        (http_status, axum::Json(body)).into_response()
    }
}
```

**Step 2: Write main.rs with Axum router**

```rust
use axum::{routing::{get, post, put, delete}, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod error;
mod grpc_clients;
mod middleware;
mod routes;

use grpc_clients::GrpcClients;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let auth_url = std::env::var("AUTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://[::1]:50051".into());
    let monitor_url = std::env::var("MONITOR_SERVICE_URL")
        .unwrap_or_else(|_| "http://[::1]:50052".into());
    let alert_url = std::env::var("ALERT_SERVICE_URL")
        .unwrap_or_else(|_| "http://[::1]:50053".into());

    let clients = GrpcClients::connect(&auth_url, &monitor_url, &alert_url).await?;

    let app = Router::new()
        // Auth routes (public)
        .route("/api/v1/auth/register", post(routes::auth::register))
        .route("/api/v1/auth/login", post(routes::auth::login))
        // Auth routes (protected)
        .route("/api/v1/auth/me", get(routes::auth::me))
        .route("/api/v1/auth/password", put(routes::auth::update_password))
        // Endpoint routes (protected)
        .route("/api/v1/endpoints", post(routes::endpoints::create))
        .route("/api/v1/endpoints", get(routes::endpoints::list))
        .route("/api/v1/endpoints/{id}", get(routes::endpoints::get_one))
        .route("/api/v1/endpoints/{id}", put(routes::endpoints::update))
        .route("/api/v1/endpoints/{id}", delete(routes::endpoints::remove))
        .route("/api/v1/endpoints/{id}/metrics", get(routes::endpoints::metrics))
        // Alert routes (protected)
        .route("/api/v1/alerts/rules", post(routes::alerts::create_rule))
        .route("/api/v1/alerts/rules", get(routes::alerts::list_rules))
        .route("/api/v1/alerts/rules/{id}", put(routes::alerts::update_rule))
        .route("/api/v1/alerts/rules/{id}", delete(routes::alerts::delete_rule))
        .route("/api/v1/alerts/channels", post(routes::alerts::create_channel))
        .route("/api/v1/alerts/channels", get(routes::alerts::list_channels))
        .route("/api/v1/alerts/channels/{id}", delete(routes::alerts::delete_channel))
        .route("/api/v1/alerts/history", get(routes::alerts::history))
        // Health check
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(clients);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Gateway listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

**Step 3: Commit**

```
feat(gateway): implement Axum router with all REST endpoints
```

---

## Phase 8: Infrastructure & Deployment

### Task 8.1: Docker Setup

**Files:**
- Create: `docker/gateway.Dockerfile`
- Create: `docker/auth.Dockerfile`
- Create: `docker/monitor.Dockerfile`
- Create: `docker/monitor-query.Dockerfile`
- Create: `docker/alert.Dockerfile`
- Create: `docker/saga.Dockerfile`
- Modify: `docker-compose.yml`
- Create: `.env.example` (updated)

**Step 1: Write Dockerfiles** (one per service, multi-stage build)

Each Dockerfile follows this pattern:
```dockerfile
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p <crate-name>

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/<binary-name> /usr/local/bin/
CMD ["<binary-name>"]
```

**Step 2: Write docker-compose.yml**

```yaml
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./docker/init-databases.sql:/docker-entrypoint-initdb.d/init.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5

  redpanda:
    image: redpandadata/redpanda:latest
    command:
      - redpanda start
      - --smp 1
      - --memory 512M
      - --overprovisioned
      - --kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:19092
      - --advertise-kafka-addr internal://redpanda:9092,external://localhost:19092
    ports:
      - "19092:19092"  # Kafka API (external)
      - "18081:18081"  # Schema Registry
      - "18082:18082"  # REST Proxy
    healthcheck:
      test: ["CMD", "rpk", "cluster", "health"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  gateway:
    build:
      context: .
      dockerfile: docker/gateway.Dockerfile
    ports:
      - "8080:8080"
    environment:
      AUTH_SERVICE_URL: "http://auth-service:50051"
      MONITOR_WRITE_SERVICE_URL: "http://monitor-service:50052"
      MONITOR_READ_SERVICE_URL: "http://monitor-query-service:50054"
      ALERT_SERVICE_URL: "http://alert-service:50053"
      SAGA_SERVICE_URL: "http://saga-orchestrator:50055"
      REDIS_URL: "redis://redis:6379"
      PORT: "8080"
    depends_on:
      auth-service:
        condition: service_started
      monitor-service:
        condition: service_started
      monitor-query-service:
        condition: service_started
      alert-service:
        condition: service_started

  auth-service:
    build:
      context: .
      dockerfile: docker/auth.Dockerfile
    environment:
      AUTH_DATABASE_URL: "postgres://postgres:password@postgres:5432/auth_db"
      PASETO_KEY: "${PASETO_KEY}"
    depends_on:
      postgres:
        condition: service_healthy

  monitor-service:
    build:
      context: .
      dockerfile: docker/monitor.Dockerfile
    environment:
      MONITOR_DATABASE_URL: "postgres://postgres:password@postgres:5432/monitor_db"
      KAFKA_BROKERS: "redpanda:9092"
    depends_on:
      postgres:
        condition: service_healthy
      redpanda:
        condition: service_healthy

  monitor-query-service:
    build:
      context: .
      dockerfile: docker/monitor-query.Dockerfile
    environment:
      MONITOR_DATABASE_URL: "postgres://postgres:password@postgres:5432/monitor_db"
      KAFKA_BROKERS: "redpanda:9092"
      REDIS_URL: "redis://redis:6379"
    depends_on:
      postgres:
        condition: service_healthy
      redpanda:
        condition: service_healthy
      redis:
        condition: service_healthy

  alert-service:
    build:
      context: .
      dockerfile: docker/alert.Dockerfile
    environment:
      ALERT_DATABASE_URL: "postgres://postgres:password@postgres:5432/alert_db"
      KAFKA_BROKERS: "redpanda:9092"
    depends_on:
      postgres:
        condition: service_healthy
      redpanda:
        condition: service_healthy

  saga-orchestrator:
    build:
      context: .
      dockerfile: docker/saga.Dockerfile
    environment:
      SAGA_DATABASE_URL: "postgres://postgres:password@postgres:5432/saga_db"
      AUTH_SERVICE_URL: "http://auth-service:50051"
      MONITOR_SERVICE_URL: "http://monitor-service:50052"
      ALERT_SERVICE_URL: "http://alert-service:50053"
    depends_on:
      postgres:
        condition: service_healthy

volumes:
  pgdata:
```

**Step 3: Create database init script**

`docker/init-databases.sql`:
```sql
CREATE DATABASE auth_db;
CREATE DATABASE monitor_db;
CREATE DATABASE alert_db;
CREATE DATABASE saga_db;
```

**Step 4: Update .env.example**

```
PASETO_KEY=your-secure-32-byte-key-here!!!!
AUTH_DATABASE_URL=postgres://postgres:password@localhost:5432/auth_db
MONITOR_DATABASE_URL=postgres://postgres:password@localhost:5432/monitor_db
ALERT_DATABASE_URL=postgres://postgres:password@localhost:5432/alert_db
SAGA_DATABASE_URL=postgres://postgres:password@localhost:5432/saga_db
KAFKA_BROKERS=localhost:19092
REDIS_URL=redis://localhost:6379
PORT=8080
AUTH_SERVICE_URL=http://[::1]:50051
MONITOR_WRITE_SERVICE_URL=http://[::1]:50052
MONITOR_READ_SERVICE_URL=http://[::1]:50054
ALERT_SERVICE_URL=http://[::1]:50053
SAGA_SERVICE_URL=http://[::1]:50055
RUST_LOG=info
```

**Step 5: Commit**

```
feat: add Docker setup with multi-service compose stack
```

---

### Task 8.2: CI/CD Pipeline

**Files:**
- Modify: `.github/workflows/ci.yaml`

**Step 1: Update CI to build and test all workspace crates**

```yaml
name: CI

on: [push, pull_request]

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        crate:
          - rust-hour-common
          - rust-hour-gateway
          - rust-hour-auth
          - rust-hour-monitor
          - rust-hour-monitor-query
          - rust-hour-alert
          - rust-hour-saga

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: password
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Build ${{ matrix.crate }}
        run: cargo build -p ${{ matrix.crate }} --verbose

      - name: Test ${{ matrix.crate }}
        run: cargo test -p ${{ matrix.crate }} --verbose
        env:
          AUTH_DATABASE_URL: postgres://postgres:password@localhost:5432/auth_db
          MONITOR_DATABASE_URL: postgres://postgres:password@localhost:5432/monitor_db
          ALERT_DATABASE_URL: postgres://postgres:password@localhost:5432/alert_db
          PASETO_KEY: "test-key-that-is-32-bytes-long!!"

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --workspace -- -D warnings

  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check
```

**Step 2: Commit**

```
ci: update pipeline for workspace with per-crate matrix builds
```

---

### Task 8.3: Clean Up Old Code

**Files:**
- Remove: `src/` (old monolith source)
- Remove: `handle-errors/` (replaced by `crates/common/src/error.rs`)
- Remove: `migrations/` (replaced by per-service migrations)
- Remove: `build.rs` (no longer needed at root)
- Update: `README.md`

**Step 1: Remove old source files**

After all services are working, remove the old monolith code.

**Step 2: Update README.md** with new architecture, setup instructions, API docs.

**Step 3: Commit**

```
chore: remove legacy monolith code, update README for microservice architecture
```

---

## Phase 9: Integration Testing & Polish

### Task 7.1: End-to-End Test

**Files:**
- Create: `tests/e2e.rs` (workspace-level integration test)

**Step 1: Write an E2E test that:**
1. Starts all services (or connects to running docker compose)
2. Registers a user
3. Logs in
4. Creates an endpoint
5. Waits for a probe to run
6. Checks metrics
7. Creates an alert rule and channel
8. Verifies alert history after a failure

This can use `reqwest` against the gateway.

**Step 2: Commit**

```
test: add end-to-end integration test
```

---

## Execution Order Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1: Foundation | 1.1–1.4 | Workspace, protos, common crate |
| 2: Auth Service | 2.1–2.3 | DB, gRPC server, tests |
| 3: Monitor Service (Write) | 3.1–3.5 | DB, probe engine, Kafka producer, scheduler, gRPC, tests |
| 4: Alert Service | 4.1–4.4 | DB, evaluator, Kafka consumer, gRPC, tests |
| 5: CQRS Read Side | 5.1–5.3 | Kafka projector, materialized views, Redis cache, gRPC, tests |
| 6: Saga Orchestrator | 6.1–6.5 | State store, orchestrator engine, account deletion steps, tests |
| 7: API Gateway | 7.1–7.4 | gRPC clients, auth middleware, routes, Axum router |
| 8: Infrastructure | 8.1–8.3 | Docker (Redpanda/Kafka, Redis, Postgres), CI/CD, cleanup |
| 9: Polish | 9.1 | E2E integration tests |

**Total: ~30 tasks across 9 phases.**

**Architectural Patterns Demonstrated:**
- CQRS (Phases 3 + 5): Write path → Kafka → Read projection with materialized views + Redis cache
- Saga (Phase 6): Orchestrator-based distributed transaction with compensating actions
- Event-Driven (Phases 3, 4, 5): Kafka topics for decoupled service communication
- API Gateway (Phase 7): Single REST entry point routing to gRPC backends
- Database per Service: 4 isolated databases (auth, monitor, alert, saga)

**Dependencies:**
- Phase 1 must complete before all others
- Phase 2 (Auth) before Phase 7 (Gateway auth middleware)
- Phases 2, 3, 4 can be developed in parallel after Phase 1
- Phase 5 (CQRS Read) depends on Phase 3 (Monitor Write) for Kafka topics
- Phase 6 (Saga) depends on Phases 2, 3, 4 for gRPC services
- Phase 7 (Gateway) depends on Phases 2, 3, 4, 5, 6
- Phase 8 can start after Phase 7
- Phase 9 requires everything running
