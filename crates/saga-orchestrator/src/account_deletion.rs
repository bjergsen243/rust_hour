use async_trait::async_trait;
use tonic::transport::Channel;

use crate::steps::SagaStep;
use rust_hour_common::error::AppError;
use rust_hour_common::proto::alert::alert_service_client::AlertServiceClient;
use rust_hour_common::proto::alert::{DeleteAllForAccountRequest, RestoreForAccountRequest};
use rust_hour_common::proto::auth::auth_service_client::AuthServiceClient;
use rust_hour_common::proto::auth::{
    MarkDeletingRequest, PermanentlyDeleteRequest, RevertDeletingRequest,
};
use rust_hour_common::proto::monitor::monitor_service_client::MonitorServiceClient;
use rust_hour_common::proto::monitor::{DeleteAllEndpointsRequest, RestoreEndpointsRequest};

// Step 1: Mark account as "deleting" in auth service
pub struct MarkAccountDeletingStep {
    client: AuthServiceClient<Channel>,
}

impl MarkAccountDeletingStep {
    pub fn new(client: AuthServiceClient<Channel>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SagaStep for MarkAccountDeletingStep {
    fn name(&self) -> &str {
        "mark_account_deleting"
    }

    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        let account_id = context["account_id"]
            .as_str()
            .ok_or_else(|| AppError::Internal("Missing account_id in context".into()))?
            .to_string();

        let mut client = self.client.clone();
        client
            .mark_deleting(MarkDeletingRequest { account_id })
            .await
            .map_err(|e| AppError::Internal(format!("Failed to mark account deleting: {e}")))?;

        Ok(serde_json::json!({"auth_marked": true}))
    }

    async fn compensate(&self, context: &serde_json::Value) -> Result<(), AppError> {
        let account_id = context["account_id"]
            .as_str()
            .ok_or_else(|| AppError::Internal("Missing account_id in context".into()))?
            .to_string();

        let mut client = self.client.clone();
        client
            .revert_deleting(RevertDeletingRequest { account_id })
            .await
            .map_err(|e| AppError::Internal(format!("Failed to revert account status: {e}")))?;

        Ok(())
    }
}

// Step 2: Soft-delete all endpoints in monitor service
pub struct DeleteEndpointsStep {
    client: MonitorServiceClient<Channel>,
}

impl DeleteEndpointsStep {
    pub fn new(client: MonitorServiceClient<Channel>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SagaStep for DeleteEndpointsStep {
    fn name(&self) -> &str {
        "delete_endpoints"
    }

    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        let account_id = context["account_id"]
            .as_str()
            .ok_or_else(|| AppError::Internal("Missing account_id in context".into()))?
            .to_string();

        let mut client = self.client.clone();
        let resp = client
            .delete_all_endpoints(DeleteAllEndpointsRequest { account_id })
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete endpoints: {e}")))?;

        let count = resp.into_inner().deleted_count;
        Ok(serde_json::json!({"endpoints_deleted": count}))
    }

    async fn compensate(&self, context: &serde_json::Value) -> Result<(), AppError> {
        let account_id = context["account_id"]
            .as_str()
            .ok_or_else(|| AppError::Internal("Missing account_id in context".into()))?
            .to_string();

        let mut client = self.client.clone();
        client
            .restore_endpoints(RestoreEndpointsRequest { account_id })
            .await
            .map_err(|e| AppError::Internal(format!("Failed to restore endpoints: {e}")))?;

        Ok(())
    }
}

// Step 3: Soft-delete all alert rules and channels
pub struct DeleteAlertsStep {
    client: AlertServiceClient<Channel>,
}

impl DeleteAlertsStep {
    pub fn new(client: AlertServiceClient<Channel>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SagaStep for DeleteAlertsStep {
    fn name(&self) -> &str {
        "delete_alerts"
    }

    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        let account_id = context["account_id"]
            .as_str()
            .ok_or_else(|| AppError::Internal("Missing account_id in context".into()))?
            .to_string();

        let mut client = self.client.clone();
        let resp = client
            .delete_all_for_account(DeleteAllForAccountRequest { account_id })
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete alerts: {e}")))?;

        let inner = resp.into_inner();
        Ok(serde_json::json!({
            "rules_deleted": inner.deleted_rules,
            "channels_deleted": inner.deleted_channels,
        }))
    }

    async fn compensate(&self, context: &serde_json::Value) -> Result<(), AppError> {
        let account_id = context["account_id"]
            .as_str()
            .ok_or_else(|| AppError::Internal("Missing account_id in context".into()))?
            .to_string();

        let mut client = self.client.clone();
        client
            .restore_for_account(RestoreForAccountRequest { account_id })
            .await
            .map_err(|e| AppError::Internal(format!("Failed to restore alerts: {e}")))?;

        Ok(())
    }
}

// Step 4: Permanently delete account (final, non-compensatable step)
pub struct PermanentlyDeleteAccountStep {
    client: AuthServiceClient<Channel>,
}

impl PermanentlyDeleteAccountStep {
    pub fn new(client: AuthServiceClient<Channel>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SagaStep for PermanentlyDeleteAccountStep {
    fn name(&self) -> &str {
        "permanently_delete_account"
    }

    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, AppError> {
        let account_id = context["account_id"]
            .as_str()
            .ok_or_else(|| AppError::Internal("Missing account_id in context".into()))?
            .to_string();

        let mut client = self.client.clone();
        client
            .permanently_delete(PermanentlyDeleteRequest { account_id })
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to permanently delete account: {e}"))
            })?;

        Ok(serde_json::json!({"account_permanently_deleted": true}))
    }

    async fn compensate(&self, _context: &serde_json::Value) -> Result<(), AppError> {
        Err(AppError::Internal(
            "Cannot compensate permanent deletion — this is the final step".into(),
        ))
    }
}

/// Build the account deletion saga steps with gRPC clients
pub fn build_account_deletion_steps(
    auth_client: AuthServiceClient<Channel>,
    monitor_client: MonitorServiceClient<Channel>,
    alert_client: AlertServiceClient<Channel>,
) -> Vec<std::sync::Arc<dyn SagaStep>> {
    vec![
        std::sync::Arc::new(MarkAccountDeletingStep::new(auth_client.clone())),
        std::sync::Arc::new(DeleteEndpointsStep::new(monitor_client)),
        std::sync::Arc::new(DeleteAlertsStep::new(alert_client)),
        std::sync::Arc::new(PermanentlyDeleteAccountStep::new(auth_client)),
    ]
}
