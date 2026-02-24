pub mod auth;
pub mod config;
pub mod error;
pub mod health;
pub mod kafka_dlq;
pub mod metrics;

pub mod proto {
    pub mod auth {
        tonic::include_proto!("auth");
    }
    pub mod monitor {
        tonic::include_proto!("monitor");
    }
    pub mod alert {
        tonic::include_proto!("alert");
    }
}
