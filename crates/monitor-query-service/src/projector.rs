use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::DateTime;
use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::FutureProducer;
use uuid::Uuid;

use crate::store::QueryStore;
use rust_hour_common::kafka_dlq::send_to_dlq;

const MAX_RETRIES: u32 = 3;
const TOPIC: &str = "check.results";

pub fn create_consumer(brokers: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", "monitor-query-projector")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create Kafka consumer")
}

pub fn create_dlq_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Failed to create DLQ producer")
}

#[derive(serde::Deserialize)]
struct CheckResultEvent {
    endpoint_id: String,
    is_healthy: bool,
    status_code: Option<i32>,
    response_time_ms: i32,
    error_message: Option<String>,
    checked_at: String,
}

/// Consumes check results from Kafka and projects them into the read-side database.
/// Periodically refreshes materialized views for fast dashboard queries.
/// Failed messages are retried 3x with backoff, then sent to DLQ.
pub async fn run_projector(
    consumer: StreamConsumer,
    store: Arc<dyn QueryStore>,
    dlq_producer: FutureProducer,
) {
    consumer
        .subscribe(&[TOPIC])
        .expect("Failed to subscribe to check.results");

    let mut stream = consumer.stream();
    let mut last_refresh = Instant::now();
    let refresh_interval = Duration::from_secs(60);
    let backoffs = [
        Duration::from_millis(100),
        Duration::from_millis(500),
        Duration::from_secs(2),
    ];

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    // Parse
                    let event: CheckResultEvent = match serde_json::from_slice(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            send_to_dlq(
                                &dlq_producer,
                                TOPIC,
                                payload,
                                &format!("Parse error: {e}"),
                                0,
                            )
                            .await;
                            let _ = consumer.commit_message(&message, CommitMode::Async);
                            continue;
                        }
                    };

                    let endpoint_id: Uuid = match event.endpoint_id.parse() {
                        Ok(id) => id,
                        Err(_) => {
                            send_to_dlq(
                                &dlq_producer,
                                TOPIC,
                                payload,
                                "Invalid endpoint_id UUID",
                                0,
                            )
                            .await;
                            let _ = consumer.commit_message(&message, CommitMode::Async);
                            continue;
                        }
                    };

                    let checked_at = DateTime::parse_from_rfc3339(&event.checked_at)
                        .unwrap_or_else(|_| chrono::Utc::now().into())
                        .with_timezone(&chrono::Utc);

                    // Process with retry
                    let mut succeeded = false;
                    for attempt in 0..=MAX_RETRIES {
                        match store
                            .insert_check_result(
                                endpoint_id,
                                event.status_code,
                                event.response_time_ms,
                                event.is_healthy,
                                event.error_message.as_deref(),
                                checked_at,
                            )
                            .await
                        {
                            Ok(()) => {
                                rust_hour_common::metrics::record_kafka_consume(TOPIC, true);
                                succeeded = true;
                                break;
                            }
                            Err(e) => {
                                if attempt < MAX_RETRIES {
                                    let backoff = backoffs[attempt as usize];
                                    tracing::warn!(
                                        "Projection failed (attempt {}/{}): {e}",
                                        attempt + 1,
                                        MAX_RETRIES + 1,
                                    );
                                    tokio::time::sleep(backoff).await;
                                } else {
                                    tracing::error!(
                                        "Projection failed after {} attempts: {e}",
                                        MAX_RETRIES + 1,
                                    );
                                    rust_hour_common::metrics::record_kafka_consume(TOPIC, false);
                                    send_to_dlq(
                                        &dlq_producer,
                                        TOPIC,
                                        payload,
                                        &e.to_string(),
                                        MAX_RETRIES + 1,
                                    )
                                    .await;
                                }
                            }
                        }
                    }

                    // Always commit — DLQ'd messages should not block progress
                    let _ = consumer.commit_message(&message, CommitMode::Async);

                    if !succeeded {
                        continue;
                    }
                }

                // Periodic materialized view refresh
                if last_refresh.elapsed() > refresh_interval {
                    if let Err(e) = store.refresh_materialized_views().await {
                        tracing::error!("Failed to refresh materialized views: {e}");
                    }
                    last_refresh = Instant::now();
                }
            }
            Err(e) => tracing::error!("Kafka consumer error: {e}"),
        }
    }
}
