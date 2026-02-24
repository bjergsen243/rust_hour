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
        b"test-key-that-is-32-bytes-long!!".to_vec()
    }

    fn mock_account() -> Account {
        Account {
            id: Uuid::new_v4(),
            email: "test@example.com".into(),
            password: crate::password::hash_password("password123").unwrap(),
            status: "active".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_register_success() {
        let mut mock_store = MockAuthStore::new();
        let account = mock_account();
        let account_clone = account.clone();

        mock_store
            .expect_create_account()
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
        assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_login_success() {
        let mut mock_store = MockAuthStore::new();
        let account = mock_account();
        let account_clone = account.clone();

        mock_store
            .expect_get_account_by_email()
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

        mock_store
            .expect_get_account_by_email()
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

        let token =
            rust_hour_common::auth::issue_token("some-uuid", "test@example.com", &key).unwrap();

        let request = Request::new(ValidateTokenRequest { token });
        let response = service.validate_token(request).await;

        assert!(response.is_ok());
        let resp = response.unwrap().into_inner();
        assert_eq!(resp.account_id, "some-uuid");
        assert_eq!(resp.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_get_account_success() {
        let mut mock_store = MockAuthStore::new();
        let account = mock_account();
        let account_id = account.id;
        let account_clone = account.clone();

        mock_store
            .expect_get_account_by_id()
            .returning(move |_| Ok(account_clone.clone()));

        let service = AuthGrpcService::new(Arc::new(mock_store), test_paseto_key());

        let request = Request::new(GetAccountRequest {
            account_id: account_id.to_string(),
        });

        let response = service.get_account(request).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().into_inner().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_mark_deleting_success() {
        let mut mock_store = MockAuthStore::new();
        let account_id = Uuid::new_v4();

        mock_store
            .expect_mark_deleting()
            .returning(|_| Ok(()));

        let service = AuthGrpcService::new(Arc::new(mock_store), test_paseto_key());

        let request = Request::new(MarkDeletingRequest {
            account_id: account_id.to_string(),
        });

        let response = service.mark_deleting(request).await;
        assert!(response.is_ok());
        assert!(response.unwrap().into_inner().success);
    }

    #[tokio::test]
    async fn test_revert_deleting_success() {
        let mut mock_store = MockAuthStore::new();
        let account_id = Uuid::new_v4();

        mock_store
            .expect_revert_deleting()
            .returning(|_| Ok(()));

        let service = AuthGrpcService::new(Arc::new(mock_store), test_paseto_key());

        let request = Request::new(RevertDeletingRequest {
            account_id: account_id.to_string(),
        });

        let response = service.revert_deleting(request).await;
        assert!(response.is_ok());
        assert!(response.unwrap().into_inner().success);
    }
}
