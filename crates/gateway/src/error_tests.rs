#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::error::ApiError;

    fn status_of(code: tonic::Code) -> StatusCode {
        let err = ApiError(tonic::Status::new(code, "test"));
        let response = err.into_response();
        response.status()
    }

    #[test]
    fn test_not_found_maps_to_404() {
        assert_eq!(status_of(tonic::Code::NotFound), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_invalid_argument_maps_to_400() {
        assert_eq!(
            status_of(tonic::Code::InvalidArgument),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_already_exists_maps_to_409() {
        assert_eq!(
            status_of(tonic::Code::AlreadyExists),
            StatusCode::CONFLICT
        );
    }

    #[test]
    fn test_unauthenticated_maps_to_401() {
        assert_eq!(
            status_of(tonic::Code::Unauthenticated),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn test_permission_denied_maps_to_403() {
        assert_eq!(
            status_of(tonic::Code::PermissionDenied),
            StatusCode::FORBIDDEN
        );
    }

    #[test]
    fn test_resource_exhausted_maps_to_429() {
        assert_eq!(
            status_of(tonic::Code::ResourceExhausted),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn test_internal_maps_to_500() {
        assert_eq!(
            status_of(tonic::Code::Internal),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_unknown_maps_to_500() {
        assert_eq!(
            status_of(tonic::Code::Unknown),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_response_body_contains_error_message() {
        let err = ApiError(tonic::Status::new(
            tonic::Code::NotFound,
            "Endpoint not found",
        ));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "Endpoint not found");
    }
}
