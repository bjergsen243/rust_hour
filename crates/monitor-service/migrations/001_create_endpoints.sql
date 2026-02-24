CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS endpoints (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL,
    name            VARCHAR(255) NOT NULL,
    url             TEXT NOT NULL,
    method          VARCHAR(10) NOT NULL DEFAULT 'GET',
    headers         JSONB NOT NULL DEFAULT '{}',
    expected_status INT NOT NULL DEFAULT 200,
    interval_secs   INT NOT NULL DEFAULT 60,
    timeout_ms      INT NOT NULL DEFAULT 5000,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    deleted_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_endpoints_account ON endpoints(account_id);
CREATE INDEX idx_endpoints_active ON endpoints(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_endpoints_not_deleted ON endpoints(deleted_at) WHERE deleted_at IS NULL;
