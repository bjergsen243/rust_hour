#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use tonic::Request;
    use uuid::Uuid;

    use rust_hour_common::proto::monitor::monitor_service_server::MonitorService;
    use rust_hour_common::proto::monitor::*;

    use crate::grpc::MonitorGrpcService;
    use crate::store::{CheckResult, Endpoint, MockMonitorStore, UptimeStats};

    fn mock_endpoint(account_id: Uuid) -> Endpoint {
        Endpoint {
            id: Uuid::new_v4(),
            account_id,
            name: "Test API".into(),
            url: "https://api.example.com/health".into(),
            method: "GET".into(),
            headers: serde_json::json!({}),
            expected_status: 200,
            interval_secs: 60,
            timeout_ms: 5000,
            is_active: true,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn mock_check_result(endpoint_id: Uuid) -> CheckResult {
        CheckResult {
            id: Uuid::new_v4(),
            endpoint_id,
            status_code: Some(200),
            response_time_ms: Some(150),
            is_healthy: true,
            error_message: None,
            checked_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_endpoint_success() {
        let mut mock = MockMonitorStore::new();
        let account_id = Uuid::new_v4();
        let ep = mock_endpoint(account_id);
        let ep_clone = ep.clone();

        mock.expect_create_endpoint()
            .returning(move |_, _, _, _, _, _, _, _| Ok(ep_clone.clone()));

        let service = MonitorGrpcService::new(Arc::new(mock));
        let request = Request::new(CreateEndpointRequest {
            account_id: account_id.to_string(),
            name: "Test API".into(),
            url: "https://api.example.com/health".into(),
            method: "GET".into(),
            headers_json: "{}".into(),
            expected_status: 200,
            interval_secs: 60,
            timeout_ms: 5000,
        });

        let response = service.create_endpoint(request).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().into_inner().name, "Test API");
    }

    #[tokio::test]
    async fn test_create_endpoint_invalid_uuid() {
        let mock = MockMonitorStore::new();
        let service = MonitorGrpcService::new(Arc::new(mock));

        let request = Request::new(CreateEndpointRequest {
            account_id: "not-a-uuid".into(),
            name: "Test".into(),
            url: "https://example.com".into(),
            method: "GET".into(),
            headers_json: String::new(),
            expected_status: 200,
            interval_secs: 60,
            timeout_ms: 5000,
        });

        let response = service.create_endpoint(request).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_create_endpoint_empty_url() {
        let mock = MockMonitorStore::new();
        let service = MonitorGrpcService::new(Arc::new(mock));

        let request = Request::new(CreateEndpointRequest {
            account_id: Uuid::new_v4().to_string(),
            name: "Test".into(),
            url: String::new(),
            method: "GET".into(),
            headers_json: String::new(),
            expected_status: 200,
            interval_secs: 60,
            timeout_ms: 5000,
        });

        let response = service.create_endpoint(request).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_list_endpoints_success() {
        let mut mock = MockMonitorStore::new();
        let account_id = Uuid::new_v4();
        let ep = mock_endpoint(account_id);

        mock.expect_list_endpoints()
            .returning(move |_, _, _| Ok((vec![ep.clone()], 1)));

        let service = MonitorGrpcService::new(Arc::new(mock));
        let request = Request::new(ListEndpointsRequest {
            account_id: account_id.to_string(),
            limit: 50,
            offset: 0,
        });

        let response = service.list_endpoints(request).await;
        assert!(response.is_ok());
        let inner = response.unwrap().into_inner();
        assert_eq!(inner.endpoints.len(), 1);
        assert_eq!(inner.total, 1);
    }

    #[tokio::test]
    async fn test_get_endpoint_with_detail() {
        let mut mock = MockMonitorStore::new();
        let account_id = Uuid::new_v4();
        let ep = mock_endpoint(account_id);
        let endpoint_id = ep.id;
        let ep_clone = ep.clone();
        let cr = mock_check_result(endpoint_id);

        mock.expect_get_endpoint()
            .returning(move |_, _| Ok(ep_clone.clone()));

        let cr_clone = cr.clone();
        mock.expect_get_latest_check()
            .returning(move |_| Ok(Some(cr_clone.clone())));

        mock.expect_get_uptime_stats()
            .returning(|_, _, _| {
                Ok(UptimeStats {
                    total_checks: 100,
                    healthy_checks: 98,
                    uptime_pct: 98.0,
                    avg_response_ms: 150.0,
                })
            });

        let service = MonitorGrpcService::new(Arc::new(mock));
        let request = Request::new(GetEndpointRequest {
            id: endpoint_id.to_string(),
            account_id: account_id.to_string(),
        });

        let response = service.get_endpoint(request).await;
        assert!(response.is_ok());
        let inner = response.unwrap().into_inner();
        assert!(inner.endpoint.is_some());
        assert!(inner.latest_check.is_some());
        assert!((inner.uptime_percent_24h - 98.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_delete_endpoint_success() {
        let mut mock = MockMonitorStore::new();
        mock.expect_delete_endpoint()
            .returning(|_, _| Ok(()));

        let service = MonitorGrpcService::new(Arc::new(mock));
        let request = Request::new(DeleteEndpointRequest {
            id: Uuid::new_v4().to_string(),
            account_id: Uuid::new_v4().to_string(),
        });

        let response = service.delete_endpoint(request).await;
        assert!(response.is_ok());
        assert!(response.unwrap().into_inner().success);
    }

    #[tokio::test]
    async fn test_soft_delete_all_endpoints() {
        let mut mock = MockMonitorStore::new();
        mock.expect_soft_delete_all()
            .returning(|_| Ok(5));

        let service = MonitorGrpcService::new(Arc::new(mock));
        let request = Request::new(DeleteAllEndpointsRequest {
            account_id: Uuid::new_v4().to_string(),
        });

        let response = service.delete_all_endpoints(request).await;
        assert!(response.is_ok());
        let inner = response.unwrap().into_inner();
        assert!(inner.success);
        assert_eq!(inner.deleted_count, 5);
    }

    #[tokio::test]
    async fn test_restore_endpoints() {
        let mut mock = MockMonitorStore::new();
        mock.expect_restore_all()
            .returning(|_| Ok(()));

        let service = MonitorGrpcService::new(Arc::new(mock));
        let request = Request::new(RestoreEndpointsRequest {
            account_id: Uuid::new_v4().to_string(),
        });

        let response = service.restore_endpoints(request).await;
        assert!(response.is_ok());
        assert!(response.unwrap().into_inner().success);
    }
}
