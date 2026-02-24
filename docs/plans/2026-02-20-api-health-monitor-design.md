# API Health Monitor — Design Document

**Date**: 2026-02-20
**Author**: Generated via brainstorming session
**Status**: Approved

## Overview

Transform **rust_hour** from a CRUD Q&A API into an **API Health Monitoring Platform**. Developers register their API endpoints, and the system continuously monitors uptime, latency, and health — alerting them when things go wrong.

The project showcases microservice architecture in Rust, using gRPC for inter-service communication, Kafka for async event-driven workflows, CQRS for separating write/read paths in the monitoring pipeline, and the Saga pattern for distributed transaction coordination.

## Architecture

### Services

| Service | Responsibility | Port | Database |
|---------|---------------|------|----------|
| **API Gateway** | Public REST API, routing, auth validation, rate limiting | 8080 | None (stateless) |
| **Auth Service** | Registration, login, PASETO tokens, account management | 50051 (gRPC) | PostgreSQL `auth_db` |
| **Monitor Service (Write)** | Endpoint registration, probe scheduling, writes check results to Kafka | 50052 (gRPC) | PostgreSQL `monitor_db` (write store) |
| **Monitor Query Service (Read)** | Serves metrics/dashboard queries from optimized read store | 50054 (gRPC) | PostgreSQL `monitor_db` (materialized views) + Redis cache |
| **Alert Service** | Alert rules, notification dispatch, alert history | 50053 (gRPC) | PostgreSQL `alert_db` |
| **Saga Orchestrator** | Coordinates distributed transactions (account deletion) | Internal | PostgreSQL `saga_db` (saga state log) |

### Data Flow

```
User → Gateway (REST) → Auth Service (gRPC)            → validate token
                       → Monitor Service (gRPC)         → CRUD endpoints, run probes
                       → Monitor Query Service (gRPC)   → read metrics (CQRS read side)
                       → Alert Service (gRPC)           → manage rules

CQRS Write Path:
  Monitor Service → probe execution → Kafka topic "check.results"
                                    → Kafka consumer writes to PostgreSQL check_results table
                                    → Kafka consumer refreshes materialized views

CQRS Read Path:
  Monitor Query Service → reads from materialized views (pre-aggregated uptime, avg latency)
                        → caches hot queries in Redis

Event-Driven Alerting:
  Monitor Service → Kafka topic "monitor.events" → Alert Service consumer
                                                  → evaluates rules → sends notifications

Saga (Account Deletion):
  Gateway DELETE /account → Saga Orchestrator
    Step 1: Auth Service  → mark account "deleting"
    Step 2: Monitor Service → delete all endpoints
    Step 3: Alert Service  → delete all rules/channels
    Step 4: Auth Service  → confirm deletion
    On failure: compensating actions reverse completed steps
```

### Architectural Patterns

| Pattern | Where Applied | Purpose |
|---------|--------------|---------|
| **CQRS** | Monitor Service (write) + Monitor Query Service (read) | Separate high-throughput probe writes from complex metrics queries |
| **Saga (Orchestrator)** | Account deletion across services | Distributed transaction coordination with compensating actions |
| **Event-Driven** | Kafka topics for check results + alerts | Decoupled async communication between services |
| **API Gateway** | Gateway service | Single entry point, auth validation, rate limiting |
| **Database per Service** | Each service owns its DB | Data isolation, independent deployability |

### Inter-Service Communication

- **Synchronous**: gRPC via `tonic` for request/response (gateway ↔ services)
- **Asynchronous**: Kafka for event-driven workflows (monitor → alert, CQRS projections)
- **Saga coordination**: Kafka topics + saga state stored in PostgreSQL

## Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| HTTP Framework | Axum 0.8 | Modern, tower-based, best ecosystem |
| gRPC | tonic 0.12 + prost | De facto Rust gRPC |
| Database | SQLx 0.8 | Compile-time checked queries |
| Message Broker | Kafka (rdkafka) | Partition-based ordering, message replay, consumer groups, scales to millions of events |
| Auth Tokens | PASETO | Better than JWT, proven in current codebase |
| Password Hashing | Argon2id | Industry standard |
| Scheduling | tokio-cron-scheduler | Health check probe scheduling |
| HTTP Client | reqwest | Probe execution |
| Caching | Redis (fred) | Rate limiting, metrics cache |
| Observability | tracing + OpenTelemetry | Distributed tracing |
| Config | config-rs | Unified config from env/files/CLI |
| Testing | mockall + testcontainers | Mocks + real DB integration tests |

## Crate Structure

```
rust_hour/
├── Cargo.toml                        # [workspace]
├── proto/
│   ├── auth.proto
│   ├── monitor.proto
│   └── alert.proto
├── crates/
│   ├── common/                       # Shared types, middleware, errors, Kafka helpers
│   │   └── src/ { lib, auth, config, error, proto, kafka }
│   ├── gateway/                      # API Gateway (binary)
│   │   └── src/ { main, routes/, middleware/, grpc_clients }
│   ├── auth-service/                 # Auth Service (binary)
│   │   ├── migrations/
│   │   └── src/ { main, grpc, store, handlers/ }
│   ├── monitor-service/              # Monitor Write Service — CQRS write side (binary)
│   │   ├── migrations/
│   │   └── src/ { main, grpc, store, scheduler, probe, producer }
│   ├── monitor-query-service/        # Monitor Read Service — CQRS read side (binary)
│   │   └── src/ { main, grpc, store, projector, cache }
│   ├── alert-service/                # Alert Service (binary)
│   │   ├── migrations/
│   │   └── src/ { main, grpc, store, consumer, evaluator, notifier }
│   └── saga-orchestrator/            # Saga coordinator for distributed transactions (binary)
│       ├── migrations/
│       └── src/ { main, orchestrator, steps, state_store }
├── docker/
│   ├── gateway.Dockerfile
│   ├── auth.Dockerfile
│   ├── monitor.Dockerfile
│   └── alert.Dockerfile
├── docker-compose.yml
└── .github/workflows/ci.yaml
```

## Database Schemas

### Auth Service (`auth_db`)

```sql
CREATE TABLE accounts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       VARCHAR(255) UNIQUE NOT NULL,
    password    VARCHAR(255) NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    updated_at  TIMESTAMPTZ DEFAULT NOW()
);
```

### Monitor Service (`monitor_db`)

```sql
CREATE TABLE endpoints (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL,
    name            VARCHAR(255) NOT NULL,
    url             TEXT NOT NULL,
    method          VARCHAR(10) DEFAULT 'GET',
    headers         JSONB DEFAULT '{}',
    expected_status INT DEFAULT 200,
    interval_secs   INT DEFAULT 60,
    timeout_ms      INT DEFAULT 5000,
    is_active       BOOLEAN DEFAULT TRUE,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE check_results (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id      UUID NOT NULL REFERENCES endpoints(id) ON DELETE CASCADE,
    status_code      INT,
    response_time_ms INT,
    is_healthy       BOOLEAN NOT NULL,
    error_message    TEXT,
    checked_at       TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_check_results_endpoint_time ON check_results(endpoint_id, checked_at DESC);
```

### Alert Service (`alert_db`)

```sql
CREATE TABLE alert_rules (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id           UUID NOT NULL,
    endpoint_id          UUID NOT NULL,
    rule_type            VARCHAR(50) NOT NULL,
    threshold_value      INT,
    consecutive_failures INT DEFAULT 3,
    is_active            BOOLEAN DEFAULT TRUE,
    created_at           TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE alert_channels (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id   UUID NOT NULL,
    channel_type VARCHAR(50) NOT NULL,
    config       JSONB NOT NULL,
    created_at   TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE alert_history (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_rule_id UUID NOT NULL REFERENCES alert_rules(id),
    channel_id    UUID NOT NULL REFERENCES alert_channels(id),
    status        VARCHAR(20) NOT NULL,
    message       TEXT NOT NULL,
    triggered_at  TIMESTAMPTZ DEFAULT NOW()
);
```

## API Endpoints (Gateway)

### Auth
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/v1/auth/register` | No | Create account |
| POST | `/api/v1/auth/login` | No | Get PASETO token |
| GET | `/api/v1/auth/me` | Yes | Current user info |
| PUT | `/api/v1/auth/password` | Yes | Update password |

### Endpoints (Monitoring)
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/v1/endpoints` | Yes | Register endpoint |
| GET | `/api/v1/endpoints` | Yes | List user's endpoints |
| GET | `/api/v1/endpoints/{id}` | Yes | Endpoint details + status |
| PUT | `/api/v1/endpoints/{id}` | Yes | Update config |
| DELETE | `/api/v1/endpoints/{id}` | Yes | Delete endpoint |
| GET | `/api/v1/endpoints/{id}/metrics` | Yes | Latency/uptime history |

### Alerts
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/v1/alerts/rules` | Yes | Create alert rule |
| GET | `/api/v1/alerts/rules` | Yes | List rules |
| PUT | `/api/v1/alerts/rules/{id}` | Yes | Update rule |
| DELETE | `/api/v1/alerts/rules/{id}` | Yes | Delete rule |
| POST | `/api/v1/alerts/channels` | Yes | Add notification channel |
| GET | `/api/v1/alerts/channels` | Yes | List channels |
| DELETE | `/api/v1/alerts/channels/{id}` | Yes | Remove channel |
| GET | `/api/v1/alerts/history` | Yes | Alert history (paginated) |

## gRPC Service Definitions

### auth.proto
- `Register(email, password) → token`
- `Login(email, password) → token`
- `ValidateToken(token) → account_id, email`
- `UpdatePassword(account_id, old_password, new_password) → success`
- `GetAccount(account_id) → email, created_at`

### monitor.proto
- `CreateEndpoint(...) → endpoint`
- `ListEndpoints(account_id) → [endpoint]`
- `GetEndpoint(id) → endpoint + latest_status`
- `UpdateEndpoint(id, ...) → endpoint`
- `DeleteEndpoint(id) → success`
- `GetMetrics(endpoint_id, from, to) → [check_result]`

### alert.proto
- `CreateRule(...) → rule`
- `ListRules(account_id) → [rule]`
- `UpdateRule(id, ...) → rule`
- `DeleteRule(id) → success`
- `CreateChannel(...) → channel`
- `ListChannels(account_id) → [channel]`
- `DeleteChannel(id) → success`
- `GetHistory(account_id, pagination) → [alert_event]`

## Kafka Topics

| Topic | Partition Key | Publisher | Consumers | Payload |
|-------|--------------|-----------|-----------|---------|
| `check.results` | `endpoint_id` | Monitor (write) | Monitor Query (projector), Alert Service | `{ endpoint_id, account_id, is_healthy, status_code, response_time_ms, error_message, checked_at }` |
| `monitor.events` | `endpoint_id` | Monitor (write) | Alert Service | `{ event_type, endpoint_id, account_id, details, timestamp }` |
| `saga.commands` | `saga_id` | Saga Orchestrator | Auth, Monitor, Alert | `{ saga_id, step, command, payload }` |
| `saga.replies` | `saga_id` | Auth, Monitor, Alert | Saga Orchestrator | `{ saga_id, step, status, payload }` |

**Partitioning strategy**: Using `endpoint_id` as partition key ensures all check results for the same endpoint land on the same partition — preserving ordering for that endpoint's metrics timeline.

## CQRS Design (Monitor Service)

### Write Side (monitor-service)
- Receives gRPC calls for endpoint CRUD
- Runs scheduled probes
- Writes check results to **Kafka topic `check.results`** (not directly to DB)
- Endpoint metadata (name, url, config) written directly to PostgreSQL

### Read Side (monitor-query-service)
- **Kafka consumer** reads from `check.results` topic
- Writes raw results to `check_results` table
- Refreshes **materialized views** for fast aggregation:
  - `endpoint_uptime_hourly` — uptime percentage per hour
  - `endpoint_latency_hourly` — avg/p50/p95/p99 latency per hour
- Caches hot queries in **Redis** (TTL: 30s for dashboard, 5min for historical)
- Serves gRPC queries: `GetMetrics`, `GetEndpointDetail` (with stats)

### Materialized Views

```sql
CREATE MATERIALIZED VIEW endpoint_uptime_hourly AS
SELECT
    endpoint_id,
    date_trunc('hour', checked_at) AS hour,
    COUNT(*) AS total_checks,
    COUNT(*) FILTER (WHERE is_healthy) AS healthy_checks,
    ROUND(100.0 * COUNT(*) FILTER (WHERE is_healthy) / COUNT(*), 2) AS uptime_pct
FROM check_results
GROUP BY endpoint_id, date_trunc('hour', checked_at);

CREATE MATERIALIZED VIEW endpoint_latency_hourly AS
SELECT
    endpoint_id,
    date_trunc('hour', checked_at) AS hour,
    AVG(response_time_ms) AS avg_ms,
    PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY response_time_ms) AS p50_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms) AS p95_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY response_time_ms) AS p99_ms
FROM check_results
GROUP BY endpoint_id, date_trunc('hour', checked_at);
```

### Why CQRS Here?

Without CQRS, every dashboard query runs `AVG()`, `COUNT()`, `PERCENTILE_CONT()` over millions of rows in real-time. With CQRS:
- **Writes are fire-and-forget** to Kafka (microseconds, non-blocking)
- **Reads hit pre-computed views** (milliseconds, cached in Redis)
- Write path and read path scale independently

## Saga Design (Account Deletion)

### Orchestrator-Based Saga

When a user requests account deletion (`DELETE /api/v1/account`):

```
┌───────────────┐
│ Saga          │──── Step 1 ────→ Auth Service: mark account "deleting"
│ Orchestrator  │←─── Reply ─────  { status: "completed" }
│               │
│               │──── Step 2 ────→ Monitor Service: delete all endpoints
│               │←─── Reply ─────  { status: "completed", deleted_count: 5 }
│               │
│               │──── Step 3 ────→ Alert Service: delete all rules & channels
│               │←─── Reply ─────  { status: "completed" }
│               │
│               │──── Step 4 ────→ Auth Service: permanently delete account
│               │←─── Reply ─────  { status: "completed" }
└───────────────┘

On Step 2 failure (compensate):
  → Auth Service: revert account status to "active"

On Step 3 failure (compensate):
  → Monitor Service: restore deleted endpoints (from soft-delete)
  → Auth Service: revert account status to "active"
```

### Saga State Table (`saga_db`)

```sql
CREATE TABLE saga_instances (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_type   VARCHAR(50) NOT NULL,       -- "account_deletion"
    account_id  UUID NOT NULL,
    status      VARCHAR(20) NOT NULL,       -- "running", "completed", "compensating", "failed"
    current_step INT NOT NULL DEFAULT 0,
    context     JSONB NOT NULL DEFAULT '{}', -- accumulated state from steps
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    updated_at  TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE saga_step_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_id         UUID NOT NULL REFERENCES saga_instances(id),
    step_number     INT NOT NULL,
    step_name       VARCHAR(100) NOT NULL,
    status          VARCHAR(20) NOT NULL,   -- "pending", "completed", "failed", "compensated"
    command_payload JSONB,
    reply_payload   JSONB,
    executed_at     TIMESTAMPTZ DEFAULT NOW()
);
```

## Testing Strategy

| Level | Tool | Scope |
|-------|------|-------|
| Unit | mockall | Handler logic, rule evaluation, probe parsing, saga step logic |
| Integration | testcontainers | Real Postgres/Kafka/Redis per test |
| gRPC | tonic test client | Service endpoint tests |
| CQRS | testcontainers | Verify write → Kafka → read projection consistency |
| Saga | mockall + state assertions | Verify saga step ordering, compensation on failure |
| E2E | reqwest → gateway | Full flow: register → create endpoint → verify probes → delete account (saga) |

## Improvements Over Current Codebase

1. UUID primary keys (not serial integers)
2. Proper foreign key constraints in all migrations
3. Email validation with regex before storage
4. Rate limiting on auth endpoints via Redis
5. Configurable connection pool size
6. No sensitive data in logs (DB URLs redacted)
7. API versioning (`/api/v1/`) from the start
8. Pagination with sensible defaults (limit=20, max=100)
9. Config via `config-rs` instead of manual env parsing
10. CI matrix building all crates independently

## Deployment

All services run via `docker compose up`:

```yaml
services:
  postgres:              # Single instance, 4 databases (auth_db, monitor_db, alert_db, saga_db)
  kafka:                 # Kafka broker (via Redpanda for lightweight local dev)
  redis:                 # Rate limiting + CQRS read cache
  gateway:               # port 8080
  auth-service:          # port 50051
  monitor-service:       # port 50052 (CQRS write side)
  monitor-query-service: # port 50054 (CQRS read side)
  alert-service:         # port 50053
  saga-orchestrator:     # internal (coordinates distributed transactions)
```

## Observability

- Structured JSON logging via `tracing`
- Distributed tracing with OpenTelemetry (spans propagated in gRPC metadata)
- Health/readiness endpoints per service
