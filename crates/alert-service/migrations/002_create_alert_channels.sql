CREATE TABLE IF NOT EXISTS alert_channels (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id    UUID NOT NULL,
    channel_type  VARCHAR(50) NOT NULL,
    config        JSONB NOT NULL DEFAULT '{}',
    deleted_at    TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alert_channels_account ON alert_channels(account_id);
