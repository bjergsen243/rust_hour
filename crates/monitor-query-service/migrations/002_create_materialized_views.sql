CREATE MATERIALIZED VIEW IF NOT EXISTS endpoint_uptime_hourly AS
SELECT
    endpoint_id,
    date_trunc('hour', checked_at) AS hour,
    COUNT(*) AS total_checks,
    COUNT(*) FILTER (WHERE is_healthy) AS healthy_checks,
    ROUND(100.0 * COUNT(*) FILTER (WHERE is_healthy) / NULLIF(COUNT(*), 0), 2) AS uptime_pct
FROM check_results
GROUP BY endpoint_id, date_trunc('hour', checked_at);

CREATE UNIQUE INDEX idx_uptime_hourly_pk ON endpoint_uptime_hourly(endpoint_id, hour);

CREATE MATERIALIZED VIEW IF NOT EXISTS endpoint_latency_hourly AS
SELECT
    endpoint_id,
    date_trunc('hour', checked_at) AS hour,
    AVG(response_time_ms)::INT AS avg_ms,
    PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY response_time_ms)::INT AS p50_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::INT AS p95_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY response_time_ms)::INT AS p99_ms
FROM check_results
GROUP BY endpoint_id, date_trunc('hour', checked_at);

CREATE UNIQUE INDEX idx_latency_hourly_pk ON endpoint_latency_hourly(endpoint_id, hour);
