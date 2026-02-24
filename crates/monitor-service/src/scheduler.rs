use std::sync::Arc;

use chrono::Utc;
use rdkafka::producer::FutureProducer;
use reqwest::Client;
use tokio::time::{interval, Duration};

use crate::probe::execute_probe;
use crate::producer::{publish_check_result, CheckCompletedEvent};
use crate::store::MonitorStore;

/// Periodically fetches active endpoints and executes probes.
///
/// Each probe runs in its own spawned task for concurrency.
/// Results are written to Kafka (CQRS write side) and also stored
/// locally in the monitor DB for direct queries.
pub async fn run_scheduler(
    store: Arc<dyn MonitorStore>,
    kafka_producer: FutureProducer,
    http_client: Client,
) {
    let mut tick = interval(Duration::from_secs(10));

    loop {
        tick.tick().await;

        let endpoints = match store.get_active_endpoints().await {
            Ok(eps) => eps,
            Err(e) => {
                tracing::error!("Failed to fetch active endpoints: {e}");
                continue;
            }
        };

        for endpoint in endpoints {
            let store = Arc::clone(&store);
            let producer = kafka_producer.clone();
            let client = http_client.clone();

            tokio::spawn(async move {
                let headers_json = endpoint.headers.clone();
                let result = execute_probe(
                    &client,
                    &endpoint.id.to_string(),
                    &endpoint.url,
                    &endpoint.method,
                    &headers_json,
                    endpoint.expected_status,
                    endpoint.timeout_ms as u64,
                )
                .await;

                // Store locally for direct queries
                if let Err(e) = store
                    .insert_check_result(
                        endpoint.id,
                        result.status_code,
                        result.response_time_ms,
                        result.is_healthy,
                        result.error_message.as_deref(),
                    )
                    .await
                {
                    tracing::error!("Failed to store check result: {e}");
                }

                // Publish to Kafka for CQRS read side + alert service
                let event = CheckCompletedEvent {
                    endpoint_id: endpoint.id.to_string(),
                    account_id: endpoint.account_id.to_string(),
                    is_healthy: result.is_healthy,
                    status_code: result.status_code,
                    response_time_ms: result.response_time_ms,
                    error_message: result.error_message,
                    checked_at: Utc::now().to_rfc3339(),
                };

                if let Err(e) = publish_check_result(&producer, &event).await {
                    tracing::error!("Failed to publish check result to Kafka: {e}");
                }
            });
        }
    }
}
