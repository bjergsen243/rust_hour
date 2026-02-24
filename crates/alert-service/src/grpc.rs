use std::sync::Arc;

use tonic::{Request, Response, Status};

use rust_hour_common::proto::alert::alert_service_server::AlertService;
use rust_hour_common::proto::alert::*;

use crate::store::AlertStore;

pub struct AlertGrpcService {
    store: Arc<dyn AlertStore>,
}

impl AlertGrpcService {
    pub fn new(store: Arc<dyn AlertStore>) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl AlertService for AlertGrpcService {
    async fn create_rule(
        &self,
        request: Request<CreateRuleRequest>,
    ) -> Result<Response<RuleResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;
        let endpoint_id = req
            .endpoint_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid endpoint_id UUID"))?;

        if req.rule_type.is_empty() {
            return Err(Status::invalid_argument("rule_type is required"));
        }

        let rule = self
            .store
            .create_rule(
                account_id,
                endpoint_id,
                &req.rule_type,
                req.threshold_value,
                if req.consecutive_failures == 0 {
                    1
                } else {
                    req.consecutive_failures
                },
            )
            .await
            .map_err(Status::from)?;

        Ok(Response::new(rule_to_proto(&rule)))
    }

    async fn list_rules(
        &self,
        request: Request<ListRulesRequest>,
    ) -> Result<Response<ListRulesResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let limit = if req.limit == 0 { 50 } else { req.limit };
        let (rules, total) = self
            .store
            .list_rules(account_id, limit, req.offset)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(ListRulesResponse {
            rules: rules.iter().map(rule_to_proto).collect(),
            total: total as i32,
        }))
    }

    async fn update_rule(
        &self,
        request: Request<UpdateRuleRequest>,
    ) -> Result<Response<RuleResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid rule UUID"))?;
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let rule = self
            .store
            .update_rule(
                id,
                account_id,
                req.rule_type.as_deref(),
                req.threshold_value,
                req.consecutive_failures,
                req.is_active,
            )
            .await
            .map_err(Status::from)?;

        Ok(Response::new(rule_to_proto(&rule)))
    }

    async fn delete_rule(
        &self,
        request: Request<DeleteRuleRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid rule UUID"))?;
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        self.store
            .delete_rule(id, account_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Rule deleted".into(),
        }))
    }

    async fn create_channel(
        &self,
        request: Request<CreateChannelRequest>,
    ) -> Result<Response<ChannelResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        if req.channel_type.is_empty() {
            return Err(Status::invalid_argument("channel_type is required"));
        }

        let config: serde_json::Value = if req.config_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&req.config_json)
                .map_err(|_| Status::invalid_argument("Invalid config JSON"))?
        };

        let channel = self
            .store
            .create_channel(account_id, &req.channel_type, &config)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(channel_to_proto(&channel)))
    }

    async fn list_channels(
        &self,
        request: Request<ListChannelsRequest>,
    ) -> Result<Response<ListChannelsResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let limit = if req.limit == 0 { 50 } else { req.limit };
        let (channels, total) = self
            .store
            .list_channels(account_id, limit, req.offset)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(ListChannelsResponse {
            channels: channels.iter().map(channel_to_proto).collect(),
            total: total as i32,
        }))
    }

    async fn delete_channel(
        &self,
        request: Request<DeleteChannelRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid channel UUID"))?;
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        self.store
            .delete_channel(id, account_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Channel deleted".into(),
        }))
    }

    async fn get_history(
        &self,
        request: Request<GetHistoryRequest>,
    ) -> Result<Response<HistoryResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let limit = if req.limit == 0 { 50 } else { req.limit };
        let (events, total) = self
            .store
            .get_history(account_id, limit, req.offset)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(HistoryResponse {
            events: events
                .iter()
                .map(|e| AlertEvent {
                    id: e.id.to_string(),
                    alert_rule_id: e.alert_rule_id.to_string(),
                    channel_id: e.channel_id.to_string(),
                    status: e.status.clone(),
                    message: e.message.clone(),
                    triggered_at: e.triggered_at.to_rfc3339(),
                })
                .collect(),
            total: total as i32,
        }))
    }

    // --- Saga support ---

    async fn delete_all_for_account(
        &self,
        request: Request<DeleteAllForAccountRequest>,
    ) -> Result<Response<DeleteAllForAccountResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let (rules, channels) = self
            .store
            .soft_delete_all(account_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(DeleteAllForAccountResponse {
            success: true,
            deleted_rules: rules as i32,
            deleted_channels: channels as i32,
        }))
    }

    async fn restore_for_account(
        &self,
        request: Request<RestoreForAccountRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        self.store
            .restore_all(account_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Alert data restored".into(),
        }))
    }
}

fn rule_to_proto(rule: &crate::store::AlertRule) -> RuleResponse {
    RuleResponse {
        id: rule.id.to_string(),
        account_id: rule.account_id.to_string(),
        endpoint_id: rule.endpoint_id.to_string(),
        rule_type: rule.rule_type.clone(),
        threshold_value: rule.threshold_value,
        consecutive_failures: rule.consecutive_failures,
        is_active: rule.is_active,
        created_at: rule.created_at.to_rfc3339(),
    }
}

fn channel_to_proto(ch: &crate::store::AlertChannel) -> ChannelResponse {
    ChannelResponse {
        id: ch.id.to_string(),
        account_id: ch.account_id.to_string(),
        channel_type: ch.channel_type.clone(),
        config_json: ch.config.to_string(),
        created_at: ch.created_at.to_rfc3339(),
    }
}
