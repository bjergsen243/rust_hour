use std::time::Duration;

use rdkafka::producer::{FutureProducer, FutureRecord};

/// Publish a failed message to the Dead-Letter Queue topic.
///
/// DLQ topic is `{original_topic}.dlq`. The message includes
/// the original payload plus error metadata.
pub async fn send_to_dlq(
    producer: &FutureProducer,
    original_topic: &str,
    original_payload: &[u8],
    error: &str,
    retry_count: u32,
) {
    let dlq_topic = format!("{original_topic}.dlq");

    let dlq_message = serde_json::json!({
        "original_topic": original_topic,
        "original_payload": String::from_utf8_lossy(original_payload),
        "error": error,
        "failed_at": chrono::Utc::now().to_rfc3339(),
        "retry_count": retry_count,
    });

    let payload = serde_json::to_vec(&dlq_message).unwrap_or_default();

    if let Err((e, _)) = producer
        .send(
            FutureRecord::<(), Vec<u8>>::to(&dlq_topic).payload(&payload),
            Duration::from_secs(5),
        )
        .await
    {
        tracing::error!("Failed to send to DLQ topic '{dlq_topic}': {e}");
    } else {
        tracing::warn!(
            "Message sent to DLQ '{dlq_topic}' after {retry_count} retries: {error}"
        );
    }
}

/// Retry a processing function with exponential backoff.
/// On final failure, sends to DLQ.
///
/// Returns Ok(()) if processing succeeded (possibly after retries),
/// or Ok(()) if it failed and was sent to DLQ (we don't block the consumer).
pub async fn process_with_retry<F, Fut>(
    producer: &FutureProducer,
    topic: &str,
    payload: &[u8],
    max_retries: u32,
    mut process: F,
) -> Result<(), ()>
where
    F: FnMut(&[u8]) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let backoffs = [
        Duration::from_millis(100),
        Duration::from_millis(500),
        Duration::from_secs(2),
    ];

    for attempt in 0..=max_retries {
        match process(payload).await {
            Ok(()) => {
                rust_hour_common_metrics_consume(topic, true);
                return Ok(());
            }
            Err(e) => {
                if attempt < max_retries {
                    let backoff = backoffs.get(attempt as usize).copied().unwrap_or(backoffs[2]);
                    tracing::warn!(
                        "Processing failed (attempt {}/{}), retrying in {:?}: {e}",
                        attempt + 1,
                        max_retries + 1,
                        backoff,
                    );
                    tokio::time::sleep(backoff).await;
                } else {
                    tracing::error!(
                        "Processing failed after {} attempts: {e}",
                        max_retries + 1
                    );
                    rust_hour_common_metrics_consume(topic, false);
                    send_to_dlq(producer, topic, payload, &e, max_retries + 1).await;
                    return Ok(()); // Don't block the consumer
                }
            }
        }
    }

    Ok(())
}

fn rust_hour_common_metrics_consume(topic: &str, success: bool) {
    let labels = [
        ("topic", topic.to_string()),
        ("status", if success { "ok" } else { "dlq" }.to_string()),
    ];
    metrics::counter!("kafka_messages_consumed_total", &labels).increment(1);
}
