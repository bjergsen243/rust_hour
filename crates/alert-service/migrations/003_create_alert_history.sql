CREATE TABLE IF NOT EXISTS alert_history (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_rule_id  UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    channel_id     UUID NOT NULL REFERENCES alert_channels(id) ON DELETE CASCADE,
    status         VARCHAR(20) NOT NULL DEFAULT 'sent',
    message        TEXT NOT NULL,
    triggered_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alert_history_rule ON alert_history(alert_rule_id);
CREATE INDEX idx_alert_history_triggered ON alert_history(triggered_at DESC);
