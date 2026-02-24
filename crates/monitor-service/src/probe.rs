use reqwest::Client;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct ProbeResult {
    pub endpoint_id: String,
    pub status_code: Option<i32>,
    pub response_time_ms: i32,
    pub is_healthy: bool,
    pub error_message: Option<String>,
}

pub async fn execute_probe(
    client: &Client,
    endpoint_id: &str,
    url: &str,
    method: &str,
    headers: &serde_json::Value,
    expected_status: i32,
    timeout_ms: u64,
) -> ProbeResult {
    let start = Instant::now();

    let mut request_builder = match method.to_uppercase().as_str() {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "HEAD" => client.head(url),
        "PATCH" => client.patch(url),
        _ => client.get(url),
    };

    if let Some(obj) = headers.as_object() {
        for (key, value) in obj {
            if let Some(val) = value.as_str() {
                request_builder = request_builder.header(key.as_str(), val);
            }
        }
    }

    request_builder = request_builder.timeout(std::time::Duration::from_millis(timeout_ms));

    match request_builder.send().await {
        Ok(response) => {
            let elapsed = start.elapsed().as_millis() as i32;
            let status = response.status().as_u16() as i32;
            let is_healthy = status == expected_status;

            ProbeResult {
                endpoint_id: endpoint_id.to_string(),
                status_code: Some(status),
                response_time_ms: elapsed,
                is_healthy,
                error_message: if !is_healthy {
                    Some(format!("Expected status {expected_status}, got {status}"))
                } else {
                    None
                },
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as i32;
            ProbeResult {
                endpoint_id: endpoint_id.to_string(),
                status_code: None,
                response_time_ms: elapsed,
                is_healthy: false,
                error_message: Some(format!("Request failed: {e}")),
            }
        }
    }
}
