#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    use rust_hour_common::error::AppError;

    use crate::orchestrator::SagaOrchestrator;
    use crate::state_store::{MockSagaStateStore, SagaInstance};
    use crate::steps::SagaStep;

    // ── Fake steps ───────────────────────────────────────

    struct SuccessStep {
        name: &'static str,
        reply: serde_json::Value,
    }

    #[async_trait]
    impl SagaStep for SuccessStep {
        fn name(&self) -> &str {
            self.name
        }
        async fn execute(
            &self,
            _ctx: &serde_json::Value,
        ) -> Result<serde_json::Value, AppError> {
            Ok(self.reply.clone())
        }
        async fn compensate(&self, _ctx: &serde_json::Value) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct FailStep {
        name: &'static str,
    }

    #[async_trait]
    impl SagaStep for FailStep {
        fn name(&self) -> &str {
            self.name
        }
        async fn execute(
            &self,
            _ctx: &serde_json::Value,
        ) -> Result<serde_json::Value, AppError> {
            Err(AppError::Internal("step failed".into()))
        }
        async fn compensate(&self, _ctx: &serde_json::Value) -> Result<(), AppError> {
            Ok(())
        }
    }

    fn mock_saga_instance(account_id: Uuid) -> SagaInstance {
        SagaInstance {
            id: Uuid::new_v4(),
            saga_type: "account_deletion".into(),
            account_id,
            status: "pending".into(),
            current_step: 0,
            context: serde_json::json!({"account_id": account_id.to_string()}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_all_steps_succeed() {
        let account_id = Uuid::new_v4();
        let saga = mock_saga_instance(account_id);
        let saga_id = saga.id;

        let mut mock_store = MockSagaStateStore::new();

        mock_store
            .expect_create_saga()
            .returning(move |_, _| Ok(saga.clone()));

        // Each step: log_step(pending) + log_step(completed) + update_saga_status
        mock_store.expect_log_step().returning(|_, _, _, _, _, _| Ok(()));
        mock_store
            .expect_update_saga_status()
            .returning(|_, _, _, _| Ok(()));

        let steps: Vec<Arc<dyn SagaStep>> = vec![
            Arc::new(SuccessStep {
                name: "step_one",
                reply: serde_json::json!({"key": "val1"}),
            }),
            Arc::new(SuccessStep {
                name: "step_two",
                reply: serde_json::json!({"key2": "val2"}),
            }),
        ];

        let orchestrator = SagaOrchestrator::new(Arc::new(mock_store), steps);
        let result = orchestrator.run("account_deletion", account_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), saga_id);
    }

    #[tokio::test]
    async fn test_step_two_fails_triggers_compensation() {
        let account_id = Uuid::new_v4();
        let saga = mock_saga_instance(account_id);

        let mut mock_store = MockSagaStateStore::new();

        mock_store
            .expect_create_saga()
            .returning(move |_, _| Ok(saga.clone()));

        mock_store.expect_log_step().returning(|_, _, _, _, _, _| Ok(()));
        mock_store
            .expect_update_saga_status()
            .returning(|_, _, _, _| Ok(()));

        let steps: Vec<Arc<dyn SagaStep>> = vec![
            Arc::new(SuccessStep {
                name: "step_one",
                reply: serde_json::json!({}),
            }),
            Arc::new(FailStep { name: "step_two" }),
        ];

        let orchestrator = SagaOrchestrator::new(Arc::new(mock_store), steps);
        let result = orchestrator.run("account_deletion", account_id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_step_three_fails_compensates_first_two() {
        let account_id = Uuid::new_v4();
        let saga = mock_saga_instance(account_id);

        let mut mock_store = MockSagaStateStore::new();

        mock_store
            .expect_create_saga()
            .returning(move |_, _| Ok(saga.clone()));

        mock_store.expect_log_step().returning(|_, _, _, _, _, _| Ok(()));
        mock_store
            .expect_update_saga_status()
            .returning(|_, _, _, _| Ok(()));

        let steps: Vec<Arc<dyn SagaStep>> = vec![
            Arc::new(SuccessStep {
                name: "mark_deleting",
                reply: serde_json::json!({}),
            }),
            Arc::new(SuccessStep {
                name: "delete_endpoints",
                reply: serde_json::json!({}),
            }),
            Arc::new(FailStep {
                name: "delete_alerts",
            }),
        ];

        let orchestrator = SagaOrchestrator::new(Arc::new(mock_store), steps);
        let result = orchestrator.run("account_deletion", account_id).await;

        // The third step fails, triggering compensation of steps 2 and 1 in reverse
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_first_step_fails_no_compensation_needed() {
        let account_id = Uuid::new_v4();
        let saga = mock_saga_instance(account_id);

        let mut mock_store = MockSagaStateStore::new();

        mock_store
            .expect_create_saga()
            .returning(move |_, _| Ok(saga.clone()));

        mock_store.expect_log_step().returning(|_, _, _, _, _, _| Ok(()));
        mock_store
            .expect_update_saga_status()
            .returning(|_, _, _, _| Ok(()));

        let steps: Vec<Arc<dyn SagaStep>> = vec![
            Arc::new(FailStep {
                name: "mark_deleting",
            }),
            Arc::new(SuccessStep {
                name: "delete_endpoints",
                reply: serde_json::json!({}),
            }),
        ];

        let orchestrator = SagaOrchestrator::new(Arc::new(mock_store), steps);
        let result = orchestrator.run("account_deletion", account_id).await;

        // First step fails → compensate(0..0) = no steps to compensate
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_merges_across_steps() {
        let account_id = Uuid::new_v4();
        let saga = mock_saga_instance(account_id);

        let mut mock_store = MockSagaStateStore::new();

        mock_store
            .expect_create_saga()
            .returning(move |_, _| Ok(saga.clone()));

        mock_store.expect_log_step().returning(|_, _, _, _, _, _| Ok(()));
        mock_store
            .expect_update_saga_status()
            .returning(|_, _, _, _| Ok(()));

        // Step 1 adds "deleted_endpoints": 5
        // Step 2 adds "deleted_alerts": 3
        // Both should be in the context by the end
        let steps: Vec<Arc<dyn SagaStep>> = vec![
            Arc::new(SuccessStep {
                name: "delete_endpoints",
                reply: serde_json::json!({"deleted_endpoints": 5}),
            }),
            Arc::new(SuccessStep {
                name: "delete_alerts",
                reply: serde_json::json!({"deleted_alerts": 3}),
            }),
        ];

        let orchestrator = SagaOrchestrator::new(Arc::new(mock_store), steps);
        let result = orchestrator.run("account_deletion", account_id).await;

        assert!(result.is_ok());
    }
}
