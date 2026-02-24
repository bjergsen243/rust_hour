CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS check_results (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id      UUID NOT NULL,
    status_code      INT,
    response_time_ms INT,
    is_healthy       BOOLEAN NOT NULL,
    error_message    TEXT,
    checked_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_check_results_endpoint_time ON check_results(endpoint_id, checked_at DESC);
CREATE INDEX idx_check_results_checked_at ON check_results(checked_at);
