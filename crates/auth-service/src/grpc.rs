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

        let email_re = regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
        if !email_re.is_match(&req.email) {
            return Err(Status::invalid_argument("Invalid email format"));
        }

        if req.password.len() < 8 {
            return Err(Status::invalid_argument(
                "Password must be at least 8 characters",
            ));
        }

        let password_hash = hash_password(&req.password).map_err(Status::from)?;
        let account = self
            .store
            .create_account(&req.email, &password_hash)
            .await
            .map_err(Status::from)?;

        let token = issue_token(&account.id.to_string(), &account.email, &self.paseto_key)
            .map_err(Status::from)?;

        Ok(Response::new(AuthResponse { token }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let account = self
            .store
            .get_account_by_email(&req.email)
            .await
            .map_err(|_| Status::unauthenticated("Invalid credentials"))?;

        let valid = verify_password(&req.password, &account.password).map_err(Status::from)?;

        if !valid {
            return Err(Status::unauthenticated("Invalid credentials"));
        }

        let token = issue_token(&account.id.to_string(), &account.email, &self.paseto_key)
            .map_err(Status::from)?;

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
        let id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let account = self
            .store
            .get_account_by_id(id)
            .await
            .map_err(Status::from)?;

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
        let id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        let account = self
            .store
            .get_account_by_id(id)
            .await
            .map_err(Status::from)?;
        let valid =
            verify_password(&req.old_password, &account.password).map_err(Status::from)?;

        if !valid {
            return Err(Status::unauthenticated("Current password is incorrect"));
        }

        if req.new_password.len() < 8 {
            return Err(Status::invalid_argument(
                "New password must be at least 8 characters",
            ));
        }

        let new_hash = hash_password(&req.new_password).map_err(Status::from)?;
        self.store
            .update_password(id, &new_hash)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Password updated successfully".into(),
        }))
    }

    // --- Saga support RPCs ---

    async fn mark_deleting(
        &self,
        request: Request<MarkDeletingRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        self.store
            .mark_deleting(id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Account marked as deleting".into(),
        }))
    }

    async fn revert_deleting(
        &self,
        request: Request<RevertDeletingRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        self.store
            .revert_deleting(id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Account deletion reverted".into(),
        }))
    }

    async fn permanently_delete(
        &self,
        request: Request<PermanentlyDeleteRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        self.store
            .permanently_delete(id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Account permanently deleted".into(),
        }))
    }
}
