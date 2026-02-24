use config::{Config, Environment};
use serde::de::DeserializeOwned;

/// Load a typed config from environment variables.
///
/// Uses the given prefix to namespace env vars. Double underscores (`__`)
/// map to nested struct fields.
///
/// Example: prefix "AUTH", struct field `database_url` → reads `AUTH_DATABASE_URL`.
///
/// Panics on missing or invalid values — fail fast at startup.
pub fn load_config<T: DeserializeOwned>(prefix: &str) -> T {
    Config::builder()
        .add_source(
            Environment::with_prefix(prefix)
                .separator("__")
                .try_parsing(true),
        )
        .build()
        .unwrap_or_else(|e| panic!("Failed to build config with prefix '{prefix}': {e}"))
        .try_deserialize()
        .unwrap_or_else(|e| panic!("Failed to deserialize config with prefix '{prefix}': {e}"))
}

// ── Per-service config structs ───────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthServiceConfig {
    pub database_url: String,
    pub paseto_key: String,
    #[serde(default = "default_grpc_port")]
    pub port: u16,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MonitorServiceConfig {
    pub database_url: String,
    pub kafka_brokers: String,
    #[serde(default = "default_grpc_port")]
    pub port: u16,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AlertServiceConfig {
    pub database_url: String,
    pub kafka_brokers: String,
    #[serde(default = "default_grpc_port")]
    pub port: u16,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MonitorQueryServiceConfig {
    pub database_url: String,
    pub kafka_brokers: String,
    #[serde(default)]
    pub redis_url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SagaServiceConfig {
    pub database_url: String,
    pub auth_service_url: String,
    pub monitor_service_url: String,
    pub alert_service_url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GatewayConfig {
    pub auth_service_url: String,
    pub monitor_service_url: String,
    pub alert_service_url: String,
    #[serde(default)]
    pub redis_url: Option<String>,
    #[serde(default = "default_gateway_port")]
    pub port: u16,
    #[serde(default = "default_grpc_timeout_ms")]
    pub grpc_timeout_ms: u64,
    #[serde(default = "default_rate_limit_per_min")]
    pub rate_limit_per_min: u32,
}

fn default_grpc_port() -> u16 {
    50051
}
fn default_gateway_port() -> u16 {
    8080
}
fn default_grpc_timeout_ms() -> u64 {
    5000
}
fn default_rate_limit_per_min() -> u32 {
    100
}
