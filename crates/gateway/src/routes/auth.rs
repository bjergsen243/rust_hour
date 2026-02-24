use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::grpc_clients::GrpcClients;
use crate::middleware::auth::AuthUser;
use rust_hour_common::proto::auth::{
    GetAccountRequest, LoginRequest, RegisterRequest, UpdatePasswordRequest,
};

#[derive(Deserialize)]
pub struct RegisterBody {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdatePasswordBody {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct AccountResponse {
    pub id: String,
    pub email: String,
    pub created_at: String,
}

pub async fn register(
    State(clients): State<GrpcClients>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let mut auth = clients.auth.clone();
    let response = auth
        .register(RegisterRequest {
            email: body.email,
            password: body.password,
        })
        .await?;

    Ok(Json(TokenResponse {
        token: response.into_inner().token,
    }))
}

pub async fn login(
    State(clients): State<GrpcClients>,
    Json(body): Json<LoginBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let mut auth = clients.auth.clone();
    let response = auth
        .login(LoginRequest {
            email: body.email,
            password: body.password,
        })
        .await?;

    Ok(Json(TokenResponse {
        token: response.into_inner().token,
    }))
}

pub async fn me(
    State(clients): State<GrpcClients>,
    user: AuthUser,
) -> Result<Json<AccountResponse>, ApiError> {
    let mut auth = clients.auth.clone();
    let response = auth
        .get_account(GetAccountRequest {
            account_id: user.account_id,
        })
        .await?;

    let inner = response.into_inner();
    Ok(Json(AccountResponse {
        id: inner.id,
        email: inner.email,
        created_at: inner.created_at,
    }))
}

pub async fn update_password(
    State(clients): State<GrpcClients>,
    user: AuthUser,
    Json(body): Json<UpdatePasswordBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut auth = clients.auth.clone();
    auth.update_password(UpdatePasswordRequest {
        account_id: user.account_id,
        old_password: body.old_password,
        new_password: body.new_password,
    })
    .await?;

    Ok(Json(serde_json::json!({"message": "Password updated"})))
}
