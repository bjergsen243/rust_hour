use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tonic::{Code, Status};

// ── Circuit Breaker ──────────────────────────────────────────────

const FAILURE_THRESHOLD: u32 = 5;
const OPEN_DURATION: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Closed,
    Open { since: Instant },
    HalfOpen,
}

struct Inner {
    name: String,
    state: State,
    consecutive_failures: u32,
}

/// A per-service circuit breaker.
///
/// - **Closed**: requests pass through normally.
/// - **Open**: after `FAILURE_THRESHOLD` consecutive failures, reject
///   immediately for `OPEN_DURATION`.
/// - **Half-Open**: after cooldown, allow one probe request.
///   Success → Closed, Failure → Open again.
#[derive(Clone)]
pub struct CircuitBreaker {
    inner: Arc<Mutex<Inner>>,
}

impl CircuitBreaker {
    pub fn new(name: &str) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                name: name.to_string(),
                state: State::Closed,
                consecutive_failures: 0,
            })),
        }
    }

    /// Check if the circuit allows a request. Returns Err if open.
    pub async fn check(&self) -> Result<(), Status> {
        let mut inner = self.inner.lock().await;
        match inner.state {
            State::Closed => Ok(()),
            State::Open { since } => {
                if since.elapsed() >= OPEN_DURATION {
                    inner.state = State::HalfOpen;
                    tracing::info!("Circuit '{}' entering half-open state", inner.name);
                    Ok(())
                } else {
                    Err(Status::unavailable(format!(
                        "Circuit breaker '{}' is open — service temporarily unavailable",
                        inner.name
                    )))
                }
            }
            State::HalfOpen => Ok(()),
        }
    }

    /// Record a successful call — resets the circuit to Closed.
    pub async fn record_success(&self) {
        let mut inner = self.inner.lock().await;
        if inner.state != State::Closed {
            tracing::info!("Circuit '{}' closed after successful probe", inner.name);
        }
        inner.state = State::Closed;
        inner.consecutive_failures = 0;
    }

    /// Record a failed call — may trip the circuit to Open.
    pub async fn record_failure(&self) {
        let mut inner = self.inner.lock().await;
        inner.consecutive_failures += 1;

        if inner.consecutive_failures >= FAILURE_THRESHOLD || inner.state == State::HalfOpen {
            inner.state = State::Open {
                since: Instant::now(),
            };
            tracing::warn!(
                "Circuit '{}' opened after {} consecutive failures",
                inner.name,
                inner.consecutive_failures
            );
        }
    }
}

// ── Retry helper ─────────────────────────────────────────────────

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF: Duration = Duration::from_millis(100);

/// Whether a gRPC status is transient and retryable.
fn is_retryable(status: &Status) -> bool {
    matches!(status.code(), Code::Unavailable | Code::DeadlineExceeded)
}

/// Execute a gRPC call with circuit breaker protection and retry.
///
/// Usage:
/// ```ignore
/// let response = resilient_call(&clients.auth_cb, || {
///     let mut client = clients.auth.clone();
///     let req = tonic::Request::new(payload.clone());
///     async move { client.validate_token(req).await }
/// }).await?;
/// ```
pub async fn resilient_call<F, Fut, T>(
    cb: &CircuitBreaker,
    make_call: F,
) -> Result<tonic::Response<T>, Status>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<tonic::Response<T>, Status>>,
{
    cb.check().await?;

    let mut last_err = None;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let backoff = INITIAL_BACKOFF * 2u32.pow(attempt - 1);
            tokio::time::sleep(backoff).await;
        }

        match make_call().await {
            Ok(response) => {
                cb.record_success().await;
                return Ok(response);
            }
            Err(status) => {
                if is_retryable(&status) && attempt < MAX_RETRIES {
                    tracing::warn!(
                        "gRPC call failed (attempt {}/{}): {}",
                        attempt + 1,
                        MAX_RETRIES + 1,
                        status.message()
                    );
                    last_err = Some(status);
                    continue;
                }
                cb.record_failure().await;
                return Err(status);
            }
        }
    }

    cb.record_failure().await;
    Err(last_err.unwrap_or_else(|| Status::internal("Retry exhausted")))
}
