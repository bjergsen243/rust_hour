use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// Middleware that ensures every request has an `x-request-id` header.
/// If the client doesn't provide one, generates a UUID v4.
/// The ID is also added to the response headers.
pub async fn request_id_middleware<B>(mut request: Request<B>, next: Next<B>) -> Response
where
    B: Send + 'static,
{
    // Use existing header or generate a new one
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Set it on the request (so handlers can read it)
    request.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap(),
    );

    // Store in extensions for downstream use (e.g., gRPC metadata propagation)
    request.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;

    // Echo back in response
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}

/// Extracted request ID, available via `request.extensions()`.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    /// Convert to tonic metadata for gRPC propagation.
    pub fn to_grpc_metadata(&self) -> tonic::metadata::MetadataValue<tonic::metadata::Ascii> {
        self.0.parse().expect("request ID is valid ASCII")
    }
}
