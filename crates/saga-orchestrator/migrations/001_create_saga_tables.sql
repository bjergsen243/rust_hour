CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS saga_instances (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_type   VARCHAR(50) NOT NULL,
    account_id  UUID NOT NULL,
    status      VARCHAR(20) NOT NULL DEFAULT 'running',
    current_step INT NOT NULL DEFAULT 0,
    context     JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS saga_step_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_id         UUID NOT NULL REFERENCES saga_instances(id) ON DELETE CASCADE,
    step_number     INT NOT NULL,
    step_name       VARCHAR(100) NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',
    command_payload JSONB,
    reply_payload   JSONB,
    executed_at     TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_saga_instances_status ON saga_instances(status) WHERE status = 'running';
CREATE INDEX idx_saga_step_log_saga ON saga_step_log(saga_id, step_number);
