use std::sync::Arc;

use chrono::{Duration, Utc};
use tonic::{Request, Response, Status};

use rust_hour_common::proto::monitor::monitor_service_server::MonitorService;
use rust_hour_common::proto::monitor::*;

use crate::store::MonitorStore;

pub struct MonitorGrpcService {
    store: Arc<dyn MonitorStore>,
}

impl MonitorGrpcService {
    pub fn new(store: Arc<dyn MonitorStore>) -> Self {
        Self { store }
    }
}

fn endpoint_to_proto(ep: &crate::store::Endpoint) -> EndpointResponse {
    EndpointResponse {
        id: ep.id.to_string(),
        account_id: ep.account_id.to_string(),
        name: ep.name.clone(),
        url: ep.url.clone(),
        method: ep.method.clone(),
        headers_json: ep.headers.to_string(),
        expected_status: ep.expected_status,
        interval_secs: ep.interval_secs,
        timeout_ms: ep.timeout_ms,
        is_active: ep.is_active,
        created_at: ep.created_at.to_rfc3339(),
        updated_at: ep.updated_at.to_rfc3339(),
    }
}

fn check_to_proto(cr: &crate::store::CheckResult) -> CheckResult {
    CheckResult {
        id: cr.id.to_string(),
        endpoint_id: cr.endpoint_id.to_string(),
        status_code: cr.status_code.unwrap_or(0),
        response_time_ms: cr.response_time_ms.unwrap_or(0),
        is_healthy: cr.is_healthy,
        error_message: cr.error_message.clone().unwrap_or_default(),
        checked_at: cr.checked_at.to_rfc3339(),
    }
}

#[tonic::async_trait]
impl MonitorService for MonitorGrpcService {
    async fn create_endpoint(
        &self,
        request: Request<CreateEndpointRequest>,
    ) -> Result<Response<EndpointResponse>, Status> {
        let req = request.into_inner();

        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        if req.url.is_empty() {
            return Err(Status::invalid_argument("URL is required"));
        }

        let headers: serde_json::Value = if req.headers_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&req.headers_json)
                .map_err(|_| Status::invalid_argument("Invalid headers JSON"))?
        };

        let ep = self
            .store
            .create_endpoint(
                account_id,
                &req.name,
                &req.url,
                if req.method.is_empty() { "GET" } else { &req.method },
                &headers,
                if req.expected_status == 0 { 200 } else { req.expected_status },
                if req.interval_secs == 0 { 60 } else { req.interval_secs },
                if req.timeout_ms == 0 { 5000 } else { req.timeout_ms },
            )
            .await
            .map_err(Status::from)?;

        Ok(Response::new(endpoint_to_proto(&ep)))
    }

    async fn list_endpoints(
        &self,
        request: Request<ListEndpointsRequest>,
    ) -> Result<Response<ListEndpointsResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let limit = if req.limit == 0 { 50 } else { req.limit };
        let (endpoints, total) = self
            .store
            .list_endpoints(account_id, limit, req.offset)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(ListEndpointsResponse {
            endpoints: endpoints.iter().map(endpoint_to_proto).collect(),
            total: total as i32,
        }))
    }

    async fn get_endpoint(
        &self,
        request: Request<GetEndpointRequest>,
    ) -> Result<Response<EndpointDetailResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid endpoint UUID"))?;
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let ep = self
            .store
            .get_endpoint(id, account_id)
            .await
            .map_err(Status::from)?;

        let latest_check = self.store.get_latest_check(id).await.map_err(Status::from)?;

        let now = Utc::now();
        let day_ago = now - Duration::hours(24);
        let stats = self
            .store
            .get_uptime_stats(id, day_ago, now)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(EndpointDetailResponse {
            endpoint: Some(endpoint_to_proto(&ep)),
            latest_check: latest_check.as_ref().map(check_to_proto),
            uptime_percent_24h: stats.uptime_pct,
            avg_response_time_ms: stats.avg_response_ms,
        }))
    }

    async fn update_endpoint(
        &self,
        request: Request<UpdateEndpointRequest>,
    ) -> Result<Response<EndpointResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid endpoint UUID"))?;
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let headers = match &req.headers_json {
            Some(h) => Some(
                serde_json::from_str::<serde_json::Value>(h)
                    .map_err(|_| Status::invalid_argument("Invalid headers JSON"))?,
            ),
            None => None,
        };

        let ep = self
            .store
            .update_endpoint(
                id,
                account_id,
                req.name.as_deref(),
                req.url.as_deref(),
                req.method.as_deref(),
                headers.as_ref(),
                req.expected_status,
                req.interval_secs,
                req.timeout_ms,
                req.is_active,
            )
            .await
            .map_err(Status::from)?;

        Ok(Response::new(endpoint_to_proto(&ep)))
    }

    async fn delete_endpoint(
        &self,
        request: Request<DeleteEndpointRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid endpoint UUID"))?;
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        self.store
            .delete_endpoint(id, account_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Endpoint deleted".into(),
        }))
    }

    async fn get_metrics(
        &self,
        request: Request<GetMetricsRequest>,
    ) -> Result<Response<MetricsResponse>, Status> {
        let req = request.into_inner();
        let endpoint_id = req
            .endpoint_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid endpoint_id UUID"))?;
        let _account_id: uuid::Uuid = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let from = chrono::DateTime::parse_from_rfc3339(&req.from)
            .map_err(|_| Status::invalid_argument("Invalid 'from' datetime (use RFC3339)"))?
            .with_timezone(&Utc);

        let to = chrono::DateTime::parse_from_rfc3339(&req.to)
            .map_err(|_| Status::invalid_argument("Invalid 'to' datetime (use RFC3339)"))?
            .with_timezone(&Utc);

        let limit = if req.limit == 0 { 100 } else { req.limit };
        let (results, total) = self
            .store
            .get_check_results(endpoint_id, from, to, limit, req.offset)
            .await
            .map_err(Status::from)?;

        let stats = self
            .store
            .get_uptime_stats(endpoint_id, from, to)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(MetricsResponse {
            results: results.iter().map(check_to_proto).collect(),
            total: total as i32,
            uptime_percent: stats.uptime_pct,
            avg_response_time_ms: stats.avg_response_ms,
        }))
    }

    // --- Saga support ---

    async fn delete_all_endpoints(
        &self,
        request: Request<DeleteAllEndpointsRequest>,
    ) -> Result<Response<DeleteAllEndpointsResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        let count = self
            .store
            .soft_delete_all(account_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(DeleteAllEndpointsResponse {
            success: true,
            deleted_count: count as i32,
        }))
    }

    async fn restore_endpoints(
        &self,
        request: Request<RestoreEndpointsRequest>,
    ) -> Result<Response<SuccessResponse>, Status> {
        let req = request.into_inner();
        let account_id = req
            .account_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid account_id UUID"))?;

        self.store
            .restore_all(account_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(SuccessResponse {
            success: true,
            message: "Endpoints restored".into(),
        }))
    }
}
