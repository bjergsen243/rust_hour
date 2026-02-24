use std::sync::Arc;

use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::net::SocketAddr;

/// Per-IP sliding-window rate limiter backed by Redis.
///
/// Uses sorted sets: each request adds timestamp as score,
/// we remove entries older than the window, then count.
#[derive(Clone)]
pub struct RateLimiter {
    redis: Arc<fred::clients::Pool>,
    max_requests: u32,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(redis: fred::clients::Pool, max_requests: u32, window_secs: u64) -> Self {
        Self {
            redis: Arc::new(redis),
            max_requests,
            window_secs,
        }
    }

    async fn check(&self, key: &str) -> Result<(bool, u32), String> {
        use fred::interfaces::SortedSetsInterface;

        let now = chrono::Utc::now().timestamp_millis() as f64;
        let window_start = now - (self.window_secs as f64 * 1000.0);
        let member = format!("{now}:{}", uuid::Uuid::new_v4());

        // Pipeline: remove old entries, add new, count, set TTL
        let _: () = self
            .redis
            .zremrangebyscore(key, f64::NEG_INFINITY, window_start)
            .await
            .map_err(|e| format!("Redis zremrangebyscore: {e}"))?;

        let _: () = self
            .redis
            .zadd(key, None, None, false, false, (now, member.as_str()))
            .await
            .map_err(|e| format!("Redis zadd: {e}"))?;

        let count: u64 = self
            .redis
            .zcard(key)
            .await
            .map_err(|e| format!("Redis zcard: {e}"))?;

        // Set TTL so keys auto-expire
        let _: () = fred::interfaces::KeysInterface::expire(&*self.redis, key, self.window_secs as i64)
            .await
            .map_err(|e| format!("Redis expire: {e}"))?;

        Ok((count <= self.max_requests as u64, self.max_requests.saturating_sub(count as u32)))
    }
}

/// Axum middleware function for rate limiting.
pub async fn rate_limit_middleware<B>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<B>,
    next: Next<B>,
) -> Response
where
    B: Send + 'static,
{
    // Extract rate limiter from request extensions
    let limiter = match request.extensions().get::<RateLimiter>() {
        Some(l) => l.clone(),
        None => return next.run(request).await, // No limiter configured, pass through
    };

    let ip = addr.ip().to_string();
    let key = format!("rate_limit:{ip}");

    match limiter.check(&key).await {
        Ok((true, remaining)) => {
            let mut response = next.run(request).await;
            response.headers_mut().insert(
                "X-RateLimit-Remaining",
                remaining.to_string().parse().unwrap(),
            );
            response
        }
        Ok((false, _)) => {
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(serde_json::json!({ "error": "Rate limit exceeded" })),
            )
                .into_response();
            response
                .headers_mut()
                .insert("Retry-After", "60".parse().unwrap());
            response
        }
        Err(e) => {
            // Redis failure — fail open (allow the request but log)
            tracing::error!("Rate limiter error: {e}");
            next.run(request).await
        }
    }
}
