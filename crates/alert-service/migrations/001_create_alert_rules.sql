CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS alert_rules (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id            UUID NOT NULL,
    endpoint_id           UUID NOT NULL,
    rule_type             VARCHAR(50) NOT NULL,
    threshold_value       INT NOT NULL DEFAULT 0,
    consecutive_failures  INT NOT NULL DEFAULT 1,
    is_active             BOOLEAN NOT NULL DEFAULT TRUE,
    deleted_at            TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alert_rules_account ON alert_rules(account_id);
CREATE INDEX idx_alert_rules_endpoint ON alert_rules(endpoint_id);
