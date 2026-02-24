#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use tonic::Request;
    use uuid::Uuid;

    use rust_hour_common::proto::alert::alert_service_server::AlertService;
    use rust_hour_common::proto::alert::*;

    use crate::grpc::AlertGrpcService;
    use crate::store::{AlertChannel, AlertHistoryEntry, AlertRule, MockAlertStore};

    fn mock_rule() -> AlertRule {
        AlertRule {
            id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            endpoint_id: Uuid::new_v4(),
            rule_type: "down".into(),
            threshold_value: 0,
            consecutive_failures: 1,
            is_active: true,
            deleted_at: None,
            created_at: Utc::now(),
        }
    }

    fn mock_channel() -> AlertChannel {
        AlertChannel {
            id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            channel_type: "webhook".into(),
            config: serde_json::json!({"url": "https://hooks.example.com/alert"}),
            deleted_at: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_rule_success() {
        let mut mock = MockAlertStore::new();
        let rule = mock_rule();
        let rule_clone = rule.clone();

        mock.expect_create_rule()
            .returning(move |_, _, _, _, _| Ok(rule_clone.clone()));

        let service = AlertGrpcService::new(Arc::new(mock));
        let request = Request::new(CreateRuleRequest {
            account_id: Uuid::new_v4().to_string(),
            endpoint_id: Uuid::new_v4().to_string(),
            rule_type: "down".into(),
            threshold_value: 0,
            consecutive_failures: 1,
        });

        let response = service.create_rule(request).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().into_inner().rule_type, "down");
    }

    #[tokio::test]
    async fn test_create_rule_empty_type() {
        let mock = MockAlertStore::new();
        let service = AlertGrpcService::new(Arc::new(mock));

        let request = Request::new(CreateRuleRequest {
            account_id: Uuid::new_v4().to_string(),
            endpoint_id: Uuid::new_v4().to_string(),
            rule_type: String::new(),
            threshold_value: 0,
            consecutive_failures: 1,
        });

        let response = service.create_rule(request).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_list_rules_success() {
        let mut mock = MockAlertStore::new();
        let rule = mock_rule();

        mock.expect_list_rules()
            .returning(move |_, _, _| Ok((vec![rule.clone()], 1)));

        let service = AlertGrpcService::new(Arc::new(mock));
        let request = Request::new(ListRulesRequest {
            account_id: Uuid::new_v4().to_string(),
            limit: 50,
            offset: 0,
        });

        let response = service.list_rules(request).await;
        assert!(response.is_ok());
        let inner = response.unwrap().into_inner();
        assert_eq!(inner.rules.len(), 1);
        assert_eq!(inner.total, 1);
    }

    #[tokio::test]
    async fn test_create_channel_success() {
        let mut mock = MockAlertStore::new();
        let channel = mock_channel();
        let channel_clone = channel.clone();

        mock.expect_create_channel()
            .returning(move |_, _, _| Ok(channel_clone.clone()));

        let service = AlertGrpcService::new(Arc::new(mock));
        let request = Request::new(CreateChannelRequest {
            account_id: Uuid::new_v4().to_string(),
            channel_type: "webhook".into(),
            config_json: r#"{"url":"https://hooks.example.com"}"#.into(),
        });

        let response = service.create_channel(request).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().into_inner().channel_type, "webhook");
    }

    #[tokio::test]
    async fn test_delete_rule_success() {
        let mut mock = MockAlertStore::new();
        mock.expect_delete_rule()
            .returning(|_, _| Ok(()));

        let service = AlertGrpcService::new(Arc::new(mock));
        let request = Request::new(DeleteRuleRequest {
            id: Uuid::new_v4().to_string(),
            account_id: Uuid::new_v4().to_string(),
        });

        let response = service.delete_rule(request).await;
        assert!(response.is_ok());
        assert!(response.unwrap().into_inner().success);
    }

    #[tokio::test]
    async fn test_get_history_success() {
        let mut mock = MockAlertStore::new();
        let entry = AlertHistoryEntry {
            id: Uuid::new_v4(),
            alert_rule_id: Uuid::new_v4(),
            channel_id: Uuid::new_v4(),
            status: "sent".into(),
            message: "Alert triggered".into(),
            triggered_at: Utc::now(),
        };

        mock.expect_get_history()
            .returning(move |_, _, _| Ok((vec![entry.clone()], 1)));

        let service = AlertGrpcService::new(Arc::new(mock));
        let request = Request::new(GetHistoryRequest {
            account_id: Uuid::new_v4().to_string(),
            limit: 50,
            offset: 0,
        });

        let response = service.get_history(request).await;
        assert!(response.is_ok());
        let inner = response.unwrap().into_inner();
        assert_eq!(inner.events.len(), 1);
        assert_eq!(inner.events[0].status, "sent");
    }

    #[tokio::test]
    async fn test_soft_delete_all_for_account() {
        let mut mock = MockAlertStore::new();
        mock.expect_soft_delete_all()
            .returning(|_| Ok((3, 2)));

        let service = AlertGrpcService::new(Arc::new(mock));
        let request = Request::new(DeleteAllForAccountRequest {
            account_id: Uuid::new_v4().to_string(),
        });

        let response = service.delete_all_for_account(request).await;
        assert!(response.is_ok());
        let inner = response.unwrap().into_inner();
        assert!(inner.success);
        assert_eq!(inner.deleted_rules, 3);
        assert_eq!(inner.deleted_channels, 2);
    }
}
