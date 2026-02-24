use std::sync::Arc;
use uuid::Uuid;

use crate::state_store::SagaStateStore;
use crate::steps::SagaStep;
use rust_hour_common::error::AppError;

pub struct SagaOrchestrator {
    state_store: Arc<dyn SagaStateStore>,
    steps: Vec<Arc<dyn SagaStep>>,
}

impl SagaOrchestrator {
    pub fn new(state_store: Arc<dyn SagaStateStore>, steps: Vec<Arc<dyn SagaStep>>) -> Self {
        Self { state_store, steps }
    }

    pub async fn run(&self, saga_type: &str, account_id: Uuid) -> Result<Uuid, AppError> {
        let saga = self
            .state_store
            .create_saga(saga_type, account_id)
            .await?;
        let mut context = saga.context.clone();

        for (i, step) in self.steps.iter().enumerate() {
            let step_num = i as i32;

            self.state_store
                .log_step(saga.id, step_num, step.name(), "pending", None, None)
                .await?;

            match step.execute(&context).await {
                Ok(reply) => {
                    if let Some(obj) = reply.as_object() {
                        for (k, v) in obj {
                            context[k] = v.clone();
                        }
                    }

                    self.state_store
                        .log_step(
                            saga.id,
                            step_num,
                            step.name(),
                            "completed",
                            Some(&context),
                            Some(&reply),
                        )
                        .await?;

                    self.state_store
                        .update_saga_status(saga.id, "running", step_num + 1, &context)
                        .await?;
                }
                Err(e) => {
                    tracing::error!("Saga step '{}' failed: {e}", step.name());

                    self.state_store
                        .log_step(saga.id, step_num, step.name(), "failed", None, None)
                        .await?;

                    self.state_store
                        .update_saga_status(saga.id, "compensating", step_num, &context)
                        .await?;

                    self.compensate(saga.id, i, &context).await?;
                    return Err(e);
                }
            }
        }

        self.state_store
            .update_saga_status(saga.id, "completed", self.steps.len() as i32, &context)
            .await?;

        Ok(saga.id)
    }

    async fn compensate(
        &self,
        saga_id: Uuid,
        failed_step: usize,
        context: &serde_json::Value,
    ) -> Result<(), AppError> {
        for i in (0..failed_step).rev() {
            let step = &self.steps[i];
            tracing::info!("Compensating step '{}'", step.name());

            match step.compensate(context).await {
                Ok(()) => {
                    self.state_store
                        .log_step(
                            saga_id,
                            i as i32,
                            step.name(),
                            "compensated",
                            None,
                            None,
                        )
                        .await?;
                }
                Err(e) => {
                    tracing::error!("Compensation for '{}' failed: {e}", step.name());
                    self.state_store
                        .log_step(
                            saga_id,
                            i as i32,
                            step.name(),
                            "compensation_failed",
                            None,
                            None,
                        )
                        .await?;
                    self.state_store
                        .update_saga_status(saga_id, "failed", i as i32, context)
                        .await?;
                    return Err(e);
                }
            }
        }

        self.state_store
            .update_saga_status(saga_id, "compensated", 0, context)
            .await?;

        Ok(())
    }
}
