use std::sync::Arc;

use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::FutureProducer;

use crate::evaluator::{evaluate_rules, CheckEvent};
use crate::notifier::send_notifications;
use crate::store::AlertStore;
use rust_hour_common::kafka_dlq::send_to_dlq;

const MAX_RETRIES: u32 = 3;
const TOPIC: &str = "monitor.events";

pub fn create_consumer(brokers: &str, group_id: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "latest")
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

pub async fn consume_monitor_events(
    consumer: StreamConsumer,
    store: Arc<dyn AlertStore>,
    dlq_producer: FutureProducer,
) {
    consumer
        .subscribe(&[TOPIC])
        .expect("Failed to subscribe to monitor.events topic");

    let mut stream = consumer.stream();
    let backoffs = [
        std::time::Duration::from_millis(100),
        std::time::Duration::from_millis(500),
        std::time::Duration::from_secs(2),
    ];

    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    // Parse the event — deserialization failures go straight to DLQ
                    let event: CheckEvent = match serde_json::from_slice(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::error!("Failed to parse check event: {e}");
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

                    // Process with retry
                    let mut processed = false;
                    for attempt in 0..=MAX_RETRIES {
                        match process_event(&store, &event).await {
                            Ok(()) => {
                                rust_hour_common::metrics::record_kafka_consume(TOPIC, true);
                                processed = true;
                                break;
                            }
                            Err(e) => {
                                if attempt < MAX_RETRIES {
                                    let backoff = backoffs[attempt as usize];
                                    tracing::warn!(
                                        "Alert processing failed (attempt {}/{}): {e}",
                                        attempt + 1,
                                        MAX_RETRIES + 1,
                                    );
                                    tokio::time::sleep(backoff).await;
                                } else {
                                    tracing::error!(
                                        "Alert processing failed after {} attempts: {e}",
                                        MAX_RETRIES + 1,
                                    );
                                    rust_hour_common::metrics::record_kafka_consume(TOPIC, false);
                                    send_to_dlq(
                                        &dlq_producer,
                                        TOPIC,
                                        payload,
                                        &e,
                                        MAX_RETRIES + 1,
                                    )
                                    .await;
                                }
                            }
                        }
                    }

                    if processed {
                        let _ = consumer.commit_message(&message, CommitMode::Async);
                    } else {
                        // DLQ'd — still commit so we don't get stuck
                        let _ = consumer.commit_message(&message, CommitMode::Async);
                    }
                }
            }
            Err(e) => tracing::error!("Kafka consumer error: {e}"),
        }
    }
}

async fn process_event(store: &Arc<dyn AlertStore>, event: &CheckEvent) -> Result<(), String> {
    let triggered_rules = evaluate_rules(store.as_ref(), event).await;

    if !triggered_rules.is_empty() {
        send_notifications(store.as_ref(), event, &triggered_rules)
            .await
            .map_err(|e| format!("Notification failed: {e}"))?;
    }

    Ok(())
}
