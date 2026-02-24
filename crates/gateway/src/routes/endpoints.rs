use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::grpc_clients::GrpcClients;
use crate::middleware::auth::AuthUser;
use rust_hour_common::proto::monitor::*;

#[derive(Deserialize)]
pub struct CreateEndpointBody {
    pub name: String,
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: serde_json::Value,
    #[serde(default = "default_expected_status")]
    pub expected_status: i32,
    #[serde(default = "default_interval")]
    pub interval_secs: i32,
    #[serde(default = "default_timeout")]
    pub timeout_ms: i32,
}

fn default_method() -> String { "GET".into() }
fn default_expected_status() -> i32 { 200 }
fn default_interval() -> i32 { 60 }
fn default_timeout() -> i32 { 5000 }

#[derive(Deserialize)]
pub struct UpdateEndpointBody {
    pub name: Option<String>,
    pub url: Option<String>,
    pub method: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub expected_status: Option<i32>,
    pub interval_secs: Option<i32>,
    pub timeout_ms: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

fn default_limit() -> i32 { 50 }

#[derive(Deserialize)]
pub struct MetricsQuery {
    pub from: String,
    pub to: String,
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

#[derive(Serialize)]
pub struct EndpointJson {
    pub id: String,
    pub name: String,
    pub url: String,
    pub method: String,
    pub headers: serde_json::Value,
    pub expected_status: i32,
    pub interval_secs: i32,
    pub timeout_ms: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<EndpointResponse> for EndpointJson {
    fn from(ep: EndpointResponse) -> Self {
        Self {
            id: ep.id,
            name: ep.name,
            url: ep.url,
            method: ep.method,
            headers: serde_json::from_str(&ep.headers_json).unwrap_or(serde_json::json!({})),
            expected_status: ep.expected_status,
            interval_secs: ep.interval_secs,
            timeout_ms: ep.timeout_ms,
            is_active: ep.is_active,
            created_at: ep.created_at,
            updated_at: ep.updated_at,
        }
    }
}

pub async fn create(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Json(body): Json<CreateEndpointBody>,
) -> Result<Json<EndpointJson>, ApiError> {
    let mut monitor = clients.monitor.clone();
    let response = monitor
        .create_endpoint(CreateEndpointRequest {
            account_id: user.account_id,
            name: body.name,
            url: body.url,
            method: body.method,
            headers_json: body.headers.to_string(),
            expected_status: body.expected_status,
            interval_secs: body.interval_secs,
            timeout_ms: body.timeout_ms,
        })
        .await?;

    Ok(Json(response.into_inner().into()))
}

pub async fn list(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut monitor = clients.monitor.clone();
    let response = monitor
        .list_endpoints(ListEndpointsRequest {
            account_id: user.account_id,
            limit: query.limit,
            offset: query.offset,
        })
        .await?;

    let inner = response.into_inner();
    let endpoints: Vec<EndpointJson> = inner.endpoints.into_iter().map(Into::into).collect();

    Ok(Json(serde_json::json!({
        "endpoints": endpoints,
        "total": inner.total,
    })))
}

pub async fn get_one(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut monitor = clients.monitor.clone();
    let response = monitor
        .get_endpoint(GetEndpointRequest {
            id,
            account_id: user.account_id,
        })
        .await?;

    let inner = response.into_inner();
    Ok(Json(serde_json::json!({
        "endpoint": inner.endpoint.map(EndpointJson::from),
        "latest_check": inner.latest_check,
        "uptime_percent_24h": inner.uptime_percent_24h,
        "avg_response_time_ms": inner.avg_response_time_ms,
    })))
}

pub async fn update(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateEndpointBody>,
) -> Result<Json<EndpointJson>, ApiError> {
    let mut monitor = clients.monitor.clone();
    let response = monitor
        .update_endpoint(UpdateEndpointRequest {
            id,
            account_id: user.account_id,
            name: body.name,
            url: body.url,
            method: body.method,
            headers_json: body.headers.map(|h| h.to_string()),
            expected_status: body.expected_status,
            interval_secs: body.interval_secs,
            timeout_ms: body.timeout_ms,
            is_active: body.is_active,
        })
        .await?;

    Ok(Json(response.into_inner().into()))
}

pub async fn remove(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut monitor = clients.monitor.clone();
    monitor
        .delete_endpoint(DeleteEndpointRequest {
            id,
            account_id: user.account_id,
        })
        .await?;

    Ok(Json(serde_json::json!({"message": "Endpoint deleted"})))
}

pub async fn metrics(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Path(id): Path<String>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut monitor = clients.monitor.clone();
    let response = monitor
        .get_metrics(GetMetricsRequest {
            endpoint_id: id,
            account_id: user.account_id,
            from: query.from,
            to: query.to,
            limit: query.limit,
            offset: query.offset,
        })
        .await?;

    let inner = response.into_inner();
    Ok(Json(serde_json::json!({
        "results": inner.results,
        "total": inner.total,
        "uptime_percent": inner.uptime_percent,
        "avg_response_time_ms": inner.avg_response_time_ms,
    })))
}
