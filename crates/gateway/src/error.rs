use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct ApiError(pub tonic::Status);

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
