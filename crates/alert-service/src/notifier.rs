use std::sync::Arc;

use reqwest::Client;

use crate::evaluator::CheckEvent;
use crate::store::{AlertRule, AlertStore};
use rust_hour_common::error::AppError;

static HTTP_CLIENT: std::sync::LazyLock<Client> = std::sync::LazyLock::new(Client::new);

pub async fn send_notifications(
    store: &Arc<dyn AlertStore>,
    event: &CheckEvent,
    rules: &[AlertRule],
) -> Result<(), AppError> {
    let account_id = event
        .account_id
        .parse()
        .map_err(|_| AppError::Internal("Invalid account_id".into()))?;

    let (channels, _) = store.list_channels(account_id, 100, 0).await?;

    for rule in rules {
        for channel in &channels {
            let message = format!(
                "Alert: endpoint {} triggered rule '{}' (response_time: {}ms, healthy: {})",
                event.endpoint_id, rule.rule_type, event.response_time_ms, event.is_healthy
            );

            match channel.channel_type.as_str() {
                "webhook" => {
                    if let Some(url) = channel.config.get("url").and_then(|v| v.as_str()) {
                        let payload = serde_json::json!({
                            "text": message,
                            "endpoint_id": event.endpoint_id,
                            "rule_type": rule.rule_type,
                            "is_healthy": event.is_healthy,
                        });

                        if let Err(e) = HTTP_CLIENT.post(url).json(&payload).send().await {
                            tracing::error!("Webhook delivery failed: {e}");
                        }
                    }
                }
                _ => {
                    tracing::warn!("Unsupported channel type: {}", channel.channel_type);
                }
            }

            let _ = store
                .insert_alert_event(rule.id, channel.id, "sent", &message)
                .await;
        }
    }

    Ok(())
}
