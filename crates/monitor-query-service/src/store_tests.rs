#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use crate::store::{MockQueryStore, QueryStore};

    #[tokio::test]
    async fn test_insert_check_result_success() {
        let mut mock = MockQueryStore::new();

        mock.expect_insert_check_result()
            .returning(|_, _, _, _, _, _| Ok(()));

        let store: Arc<dyn QueryStore> = Arc::new(mock);
        let result = store
            .insert_check_result(
                Uuid::new_v4(),
                Some(200),
                150,
                true,
                None,
                Utc::now(),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_latest_check_returns_some() {
        use crate::store::CheckResult;

        let mut mock = MockQueryStore::new();
        let endpoint_id = Uuid::new_v4();

        let check = CheckResult {
            id: Uuid::new_v4(),
            endpoint_id,
            status_code: Some(200),
            response_time_ms: Some(120),
            is_healthy: true,
            error_message: None,
            checked_at: Utc::now(),
        };

        mock.expect_get_latest_check()
            .returning(move |_| Ok(Some(check.clone())));

        let store: Arc<dyn QueryStore> = Arc::new(mock);
        let result = store.get_latest_check(endpoint_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_get_latest_check_returns_none() {
        let mut mock = MockQueryStore::new();

        mock.expect_get_latest_check()
            .returning(|_| Ok(None));

        let store: Arc<dyn QueryStore> = Arc::new(mock);
        let result = store.get_latest_check(Uuid::new_v4()).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_check_results_pagination() {
        use crate::store::CheckResult;

        let mut mock = MockQueryStore::new();
        let endpoint_id = Uuid::new_v4();

        let check = CheckResult {
            id: Uuid::new_v4(),
            endpoint_id,
            status_code: Some(200),
            response_time_ms: Some(100),
            is_healthy: true,
            error_message: None,
            checked_at: Utc::now(),
        };

        mock.expect_get_check_results()
            .returning(move |_, _, _, _, _| Ok((vec![check.clone()], 10)));

        let store: Arc<dyn QueryStore> = Arc::new(mock);
        let from = Utc::now() - chrono::Duration::hours(24);
        let to = Utc::now();
        let result = store.get_check_results(endpoint_id, from, to, 50, 0).await;

        assert!(result.is_ok());
        let (results, total) = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(total, 10);
    }

    #[tokio::test]
    async fn test_get_uptime_stats() {
        let mut mock = MockQueryStore::new();

        mock.expect_get_uptime_stats()
            .returning(|_, _, _| Ok((99.5, 145.0)));

        let store: Arc<dyn QueryStore> = Arc::new(mock);
        let from = Utc::now() - chrono::Duration::hours(24);
        let to = Utc::now();
        let result = store.get_uptime_stats(Uuid::new_v4(), from, to).await;

        assert!(result.is_ok());
        let (uptime, avg_response) = result.unwrap();
        assert!((uptime - 99.5).abs() < 0.1);
        assert!((avg_response - 145.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_refresh_materialized_views() {
        let mut mock = MockQueryStore::new();

        mock.expect_refresh_materialized_views()
            .returning(|| Ok(()));

        let store: Arc<dyn QueryStore> = Arc::new(mock);
        let result = store.refresh_materialized_views().await;

        assert!(result.is_ok());
    }
}
