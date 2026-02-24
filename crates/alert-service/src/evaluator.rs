use std::sync::Arc;

use serde::Deserialize;

use crate::store::{AlertRule, AlertStore};

#[derive(Debug, Clone, Deserialize)]
pub struct CheckEvent {
    pub endpoint_id: String,
    pub account_id: String,
    pub is_healthy: bool,
    pub status_code: Option<i32>,
    pub response_time_ms: i32,
    pub error_message: Option<String>,
    pub checked_at: String,
}

/// Evaluates alert rules against a check event, returning rules that triggered.
pub async fn evaluate_rules(
    store: &Arc<dyn AlertStore>,
    event: &CheckEvent,
) -> Vec<AlertRule> {
    let endpoint_id = match event.endpoint_id.parse() {
        Ok(id) => id,
        Err(_) => return vec![],
    };

    let rules = match store.get_rules_for_endpoint(endpoint_id).await {
        Ok(rules) => rules,
        Err(_) => return vec![],
    };

    rules
        .into_iter()
        .filter(|rule| match rule.rule_type.as_str() {
            "down" => !event.is_healthy,
            "latency_threshold" => event.response_time_ms > rule.threshold_value,
            "status_mismatch" => !event.is_healthy,
            _ => false,
        })
        .collect()
}
