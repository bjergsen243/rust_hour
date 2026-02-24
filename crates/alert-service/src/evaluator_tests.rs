#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use crate::evaluator::{evaluate_rules, CheckEvent};
    use crate::store::{AlertRule, MockAlertStore};

    fn make_rule(rule_type: &str, threshold: i32) -> AlertRule {
        AlertRule {
            id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            endpoint_id: Uuid::new_v4(),
            rule_type: rule_type.into(),
            threshold_value: threshold,
            consecutive_failures: 1,
            is_active: true,
            deleted_at: None,
            created_at: Utc::now(),
        }
    }

    fn make_event(endpoint_id: Uuid, is_healthy: bool, response_time_ms: i32) -> CheckEvent {
        CheckEvent {
            endpoint_id: endpoint_id.to_string(),
            account_id: Uuid::new_v4().to_string(),
            is_healthy,
            status_code: if is_healthy { Some(200) } else { Some(500) },
            response_time_ms,
            error_message: None,
            checked_at: Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_down_rule_triggers_on_unhealthy() {
        let endpoint_id = Uuid::new_v4();
        let mut mock = MockAlertStore::new();
        let rule = make_rule("down", 0);

        mock.expect_get_rules_for_endpoint()
            .returning(move |_| Ok(vec![rule.clone()]));

        let store: Arc<dyn crate::store::AlertStore> = Arc::new(mock);
        let event = make_event(endpoint_id, false, 100);

        let triggered = evaluate_rules(&store, &event).await;
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].rule_type, "down");
    }

    #[tokio::test]
    async fn test_down_rule_does_not_trigger_on_healthy() {
        let endpoint_id = Uuid::new_v4();
        let mut mock = MockAlertStore::new();
        let rule = make_rule("down", 0);

        mock.expect_get_rules_for_endpoint()
            .returning(move |_| Ok(vec![rule.clone()]));

        let store: Arc<dyn crate::store::AlertStore> = Arc::new(mock);
        let event = make_event(endpoint_id, true, 100);

        let triggered = evaluate_rules(&store, &event).await;
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn test_latency_threshold_triggers_above_threshold() {
        let endpoint_id = Uuid::new_v4();
        let mut mock = MockAlertStore::new();
        let rule = make_rule("latency_threshold", 500);

        mock.expect_get_rules_for_endpoint()
            .returning(move |_| Ok(vec![rule.clone()]));

        let store: Arc<dyn crate::store::AlertStore> = Arc::new(mock);
        let event = make_event(endpoint_id, true, 800); // 800ms > 500ms threshold

        let triggered = evaluate_rules(&store, &event).await;
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].rule_type, "latency_threshold");
    }

    #[tokio::test]
    async fn test_latency_threshold_does_not_trigger_below() {
        let endpoint_id = Uuid::new_v4();
        let mut mock = MockAlertStore::new();
        let rule = make_rule("latency_threshold", 500);

        mock.expect_get_rules_for_endpoint()
            .returning(move |_| Ok(vec![rule.clone()]));

        let store: Arc<dyn crate::store::AlertStore> = Arc::new(mock);
        let event = make_event(endpoint_id, true, 200); // 200ms < 500ms threshold

        let triggered = evaluate_rules(&store, &event).await;
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_rules_mixed_trigger() {
        let endpoint_id = Uuid::new_v4();
        let mut mock = MockAlertStore::new();
        let down_rule = make_rule("down", 0);
        let latency_rule = make_rule("latency_threshold", 500);

        mock.expect_get_rules_for_endpoint()
            .returning(move |_| Ok(vec![down_rule.clone(), latency_rule.clone()]));

        let store: Arc<dyn crate::store::AlertStore> = Arc::new(mock);
        // Unhealthy + high latency: both should trigger
        let event = make_event(endpoint_id, false, 800);

        let triggered = evaluate_rules(&store, &event).await;
        assert_eq!(triggered.len(), 2);
    }

    #[tokio::test]
    async fn test_invalid_endpoint_id_returns_empty() {
        let mock = MockAlertStore::new();
        let store: Arc<dyn crate::store::AlertStore> = Arc::new(mock);

        let event = CheckEvent {
            endpoint_id: "not-a-uuid".into(),
            account_id: Uuid::new_v4().to_string(),
            is_healthy: false,
            status_code: Some(500),
            response_time_ms: 100,
            error_message: None,
            checked_at: Utc::now().to_rfc3339(),
        };

        let triggered = evaluate_rules(&store, &event).await;
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn test_unknown_rule_type_does_not_trigger() {
        let endpoint_id = Uuid::new_v4();
        let mut mock = MockAlertStore::new();
        let rule = make_rule("unknown_type", 0);

        mock.expect_get_rules_for_endpoint()
            .returning(move |_| Ok(vec![rule.clone()]));

        let store: Arc<dyn crate::store::AlertStore> = Arc::new(mock);
        let event = make_event(endpoint_id, false, 100);

        let triggered = evaluate_rules(&store, &event).await;
        assert!(triggered.is_empty());
    }
}
