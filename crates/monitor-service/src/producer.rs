use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct CheckCompletedEvent {
    pub endpoint_id: String,
    pub account_id: String,
    pub is_healthy: bool,
    pub status_code: Option<i32>,
    pub response_time_ms: i32,
    pub error_message: Option<String>,
    pub checked_at: String,
}

pub fn create_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Failed to create Kafka producer")
}

pub async fn publish_check_result(
    producer: &FutureProducer,
    event: &CheckCompletedEvent,
) -> Result<(), String> {
    let payload = serde_json::to_string(event).map_err(|e| e.to_string())?;
    let key = &event.endpoint_id;

    // Publish to check.results for CQRS read-side projector
    let result = producer
        .send(
            FutureRecord::to("check.results")
                .key(key)
                .payload(&payload),
            Duration::from_secs(5),
        )
        .await;

    match &result {
        Ok(_) => rust_hour_common::metrics::record_kafka_produce("check.results", true),
        Err((e, _)) => {
            rust_hour_common::metrics::record_kafka_produce("check.results", false);
            return Err(format!("Kafka send to check.results failed: {e}"));
        }
    }

    // Publish to monitor.events for alert service evaluation
    let result = producer
        .send(
            FutureRecord::to("monitor.events")
                .key(key)
                .payload(&payload),
            Duration::from_secs(5),
        )
        .await;

    match &result {
        Ok(_) => rust_hour_common::metrics::record_kafka_produce("monitor.events", true),
        Err((e, _)) => {
            rust_hour_common::metrics::record_kafka_produce("monitor.events", false);
            return Err(format!("Kafka send to monitor.events failed: {e}"));
        }
    }

    Ok(())
}
