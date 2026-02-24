use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use crate::grpc_clients::GrpcClients;
use rust_hour_common::proto::auth::ValidateTokenRequest;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub account_id: String,
    pub email: String,
}

impl FromRequestParts<GrpcClients> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GrpcClients,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing authorization header".into(),
            ))?;

        // Support "Bearer <token>" or raw token
        let token = header.strip_prefix("Bearer ").unwrap_or(header);

        let mut auth_client = state.auth.clone();
        let response = auth_client
            .validate_token(ValidateTokenRequest {
                token: token.to_string(),
            })
            .await
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {e}")))?;

        let inner = response.into_inner();
        Ok(AuthUser {
            account_id: inner.account_id,
            email: inner.email,
        })
    }
}
