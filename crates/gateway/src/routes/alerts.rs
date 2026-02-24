use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::grpc_clients::GrpcClients;
use crate::middleware::auth::AuthUser;
use rust_hour_common::proto::alert::*;

#[derive(Deserialize)]
pub struct CreateRuleBody {
    pub endpoint_id: String,
    pub rule_type: String,
    #[serde(default)]
    pub threshold_value: i32,
    #[serde(default = "default_consecutive")]
    pub consecutive_failures: i32,
}

fn default_consecutive() -> i32 { 1 }

#[derive(Deserialize)]
pub struct UpdateRuleBody {
    pub rule_type: Option<String>,
    pub threshold_value: Option<i32>,
    pub consecutive_failures: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateChannelBody {
    pub channel_type: String,
    pub config: serde_json::Value,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

fn default_limit() -> i32 { 50 }

#[derive(Serialize)]
pub struct RuleJson {
    pub id: String,
    pub endpoint_id: String,
    pub rule_type: String,
    pub threshold_value: i32,
    pub consecutive_failures: i32,
    pub is_active: bool,
    pub created_at: String,
}

impl From<RuleResponse> for RuleJson {
    fn from(r: RuleResponse) -> Self {
        Self {
            id: r.id,
            endpoint_id: r.endpoint_id,
            rule_type: r.rule_type,
            threshold_value: r.threshold_value,
            consecutive_failures: r.consecutive_failures,
            is_active: r.is_active,
            created_at: r.created_at,
        }
    }
}

#[derive(Serialize)]
pub struct ChannelJson {
    pub id: String,
    pub channel_type: String,
    pub config: serde_json::Value,
    pub created_at: String,
}

impl From<ChannelResponse> for ChannelJson {
    fn from(c: ChannelResponse) -> Self {
        Self {
            id: c.id,
            channel_type: c.channel_type,
            config: serde_json::from_str(&c.config_json).unwrap_or(serde_json::json!({})),
            created_at: c.created_at,
        }
    }
}

pub async fn create_rule(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Json(body): Json<CreateRuleBody>,
) -> Result<Json<RuleJson>, ApiError> {
    let mut alert = clients.alert.clone();
    let response = alert
        .create_rule(CreateRuleRequest {
            account_id: user.account_id,
            endpoint_id: body.endpoint_id,
            rule_type: body.rule_type,
            threshold_value: body.threshold_value,
            consecutive_failures: body.consecutive_failures,
        })
        .await?;

    Ok(Json(response.into_inner().into()))
}

pub async fn list_rules(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut alert = clients.alert.clone();
    let response = alert
        .list_rules(ListRulesRequest {
            account_id: user.account_id,
            limit: query.limit,
            offset: query.offset,
        })
        .await?;

    let inner = response.into_inner();
    let rules: Vec<RuleJson> = inner.rules.into_iter().map(Into::into).collect();

    Ok(Json(serde_json::json!({
        "rules": rules,
        "total": inner.total,
    })))
}

pub async fn update_rule(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateRuleBody>,
) -> Result<Json<RuleJson>, ApiError> {
    let mut alert = clients.alert.clone();
    let response = alert
        .update_rule(UpdateRuleRequest {
            id,
            account_id: user.account_id,
            rule_type: body.rule_type,
            threshold_value: body.threshold_value,
            consecutive_failures: body.consecutive_failures,
            is_active: body.is_active,
        })
        .await?;

    Ok(Json(response.into_inner().into()))
}

pub async fn delete_rule(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut alert = clients.alert.clone();
    alert
        .delete_rule(DeleteRuleRequest {
            id,
            account_id: user.account_id,
        })
        .await?;

    Ok(Json(serde_json::json!({"message": "Rule deleted"})))
}

pub async fn create_channel(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Json(body): Json<CreateChannelBody>,
) -> Result<Json<ChannelJson>, ApiError> {
    let mut alert = clients.alert.clone();
    let response = alert
        .create_channel(CreateChannelRequest {
            account_id: user.account_id,
            channel_type: body.channel_type,
            config_json: body.config.to_string(),
        })
        .await?;

    Ok(Json(response.into_inner().into()))
}

pub async fn list_channels(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut alert = clients.alert.clone();
    let response = alert
        .list_channels(ListChannelsRequest {
            account_id: user.account_id,
            limit: query.limit,
            offset: query.offset,
        })
        .await?;

    let inner = response.into_inner();
    let channels: Vec<ChannelJson> = inner.channels.into_iter().map(Into::into).collect();

    Ok(Json(serde_json::json!({
        "channels": channels,
        "total": inner.total,
    })))
}

pub async fn delete_channel(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut alert = clients.alert.clone();
    alert
        .delete_channel(DeleteChannelRequest {
            id,
            account_id: user.account_id,
        })
        .await?;

    Ok(Json(serde_json::json!({"message": "Channel deleted"})))
}

pub async fn history(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut alert = clients.alert.clone();
    let response = alert
        .get_history(GetHistoryRequest {
            account_id: user.account_id,
            limit: query.limit,
            offset: query.offset,
        })
        .await?;

    let inner = response.into_inner();
    Ok(Json(serde_json::json!({
        "events": inner.events,
        "total": inner.total,
    })))
}
