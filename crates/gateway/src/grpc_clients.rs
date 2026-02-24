use std::time::Duration;

use rust_hour_common::proto::alert::alert_service_client::AlertServiceClient;
use rust_hour_common::proto::auth::auth_service_client::AuthServiceClient;
use rust_hour_common::proto::monitor::monitor_service_client::MonitorServiceClient;
use tonic::transport::Channel;

use crate::resilience::CircuitBreaker;

#[derive(Clone)]
pub struct GrpcClients {
    pub auth: AuthServiceClient<Channel>,
    pub monitor: MonitorServiceClient<Channel>,
    pub alert: AlertServiceClient<Channel>,
    pub auth_cb: CircuitBreaker,
    pub monitor_cb: CircuitBreaker,
    pub alert_cb: CircuitBreaker,
}

impl GrpcClients {
    pub async fn connect(
        auth_url: &str,
        monitor_url: &str,
        alert_url: &str,
        timeout_ms: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let timeout = Duration::from_millis(timeout_ms);
        let connect_timeout = Duration::from_secs(5);

        let auth_channel = Channel::from_shared(auth_url.to_string())?
            .timeout(timeout)
            .connect_timeout(connect_timeout)
            .connect()
            .await?;

        let monitor_channel = Channel::from_shared(monitor_url.to_string())?
            .timeout(timeout)
            .connect_timeout(connect_timeout)
            .connect()
            .await?;

        let alert_channel = Channel::from_shared(alert_url.to_string())?
            .timeout(timeout)
            .connect_timeout(connect_timeout)
            .connect()
            .await?;

        Ok(Self {
            auth: AuthServiceClient::new(auth_channel),
            monitor: MonitorServiceClient::new(monitor_channel),
            alert: AlertServiceClient::new(alert_channel),
            auth_cb: CircuitBreaker::new("auth"),
            monitor_cb: CircuitBreaker::new("monitor"),
            alert_cb: CircuitBreaker::new("alert"),
        })
    }
}
