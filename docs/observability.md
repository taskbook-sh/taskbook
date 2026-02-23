# Observability (OpenTelemetry)

The taskbook server supports full OpenTelemetry instrumentation for traces, metrics, and logs, exported via OTLP. When no OTLP endpoint is configured, the server behaves exactly as before — console-only `fmt` logging.

## Enabling OpenTelemetry

Set `OTEL_EXPORTER_OTLP_ENDPOINT` to activate all three signal pipelines (traces, metrics, logs). No other changes are required — the presence of this variable is the implicit on/off switch.

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp-gateway-prod-us-central-0.grafana.net/otlp
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic $(echo -n '<instance-id>:<api-token>' | base64)"

./tb-server
```

## Environment Variables

All standard `OTEL_*` variables are read automatically by the OpenTelemetry SDK:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Yes (enables OTel) | — | OTLP collector URL (e.g. Grafana Cloud gateway) |
| `OTEL_EXPORTER_OTLP_HEADERS` | Yes (for auth) | — | Auth headers, e.g. `Authorization=Basic <base64>` |
| `OTEL_SERVICE_NAME` | No | `taskbook-server` | Service name in all telemetry |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | No | `http/protobuf` | Keep the default for Grafana Cloud |
| `OTEL_RESOURCE_ATTRIBUTES` | No | — | Additional resource attributes, e.g. `deployment.environment=production` |
| `RUST_LOG` | No | `info` | Log level filter (existing, applies to both console and OTel) |

## Signals

### Traces

Every HTTP handler is instrumented with `#[tracing::instrument]`. Each request produces a span containing:

- Handler name (e.g. `register`, `put_items`)
- Relevant fields (`username` on auth endpoints, `item_count` on write endpoints)
- Sensitive fields (request bodies, passwords, state) are excluded

Trace context is propagated using the W3C `traceparent` header.

### Metrics

**HTTP metrics** (recorded by the metrics middleware):

| Metric | Type | Attributes | Description |
|--------|------|------------|-------------|
| `http.server.request.count` | Counter | `http.request.method`, `http.route`, `http.response.status_code` | Total requests |
| `http.server.request.duration` | Histogram (seconds) | `http.request.method`, `http.route`, `http.response.status_code` | Request latency |
| `http.server.active_requests` | UpDownCounter | `http.request.method`, `http.route` | In-flight requests |

**SSE metrics:**

| Metric | Type | Attributes | Description |
|--------|------|------------|-------------|
| `sse.active_connections` | UpDownCounter | `endpoint` | Active SSE connections (auto-decrements on disconnect) |

**Database pool metrics:**

| Metric | Type | Description |
|--------|------|-------------|
| `db.pool.connections` | ObservableGauge | Total connections in the pool |
| `db.pool.idle_connections` | ObservableGauge | Idle connections in the pool |

Metrics are exported every 15 seconds.

### Logs

All `tracing` log events (`tracing::info!`, `tracing::error!`, etc.) are bridged to the OpenTelemetry Logs pipeline and exported alongside traces and metrics. The console `fmt` layer remains active, so you still see logs in stdout.

## Grafana Cloud Setup

### 1. Create a Grafana Cloud account

Sign up at [grafana.com](https://grafana.com) and create a stack.

### 2. Get your OTLP credentials

In your Grafana Cloud stack, go to **Connections > OpenTelemetry (OTLP)** and copy:
- The OTLP endpoint URL
- Your instance ID and API token

### 3. Configure the server

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp-gateway-prod-us-central-0.grafana.net/otlp
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic $(echo -n '123456:glc_xxxxx' | base64)"
export OTEL_SERVICE_NAME=taskbook-server
export OTEL_RESOURCE_ATTRIBUTES="deployment.environment=production"
```

### 4. Import the Grafana dashboard

A pre-built dashboard is included at `dashboards/taskbook-server.json`. Import it via **Dashboards > Import** in Grafana. It provides:

- **Request Rate by Endpoint** — per-route request rate
- **Error Rate** — 4xx/5xx request rate
- **HTTP Status Distribution** — pie chart of status codes
- **Request Duration Percentiles** — p50, p95, p99 latency
- **Duration by Endpoint (p95)** — per-route p95 latency
- **Active SSE Connections** — live SSE connection count
- **Active HTTP Requests** — in-flight request count
- **DB Connection Pool** — total and idle connections over time

## Docker Compose Example

```yaml
services:
  server:
    build:
      context: .
      dockerfile: Dockerfile.server
    environment:
      TB_DB_HOST: postgres
      TB_DB_NAME: taskbook
      TB_DB_USER: taskbook
      TB_DB_PASSWORD: taskbook
      RUST_LOG: info
      OTEL_EXPORTER_OTLP_ENDPOINT: https://otlp-gateway-prod-us-central-0.grafana.net/otlp
      OTEL_EXPORTER_OTLP_HEADERS: "Authorization=Basic <base64-credentials>"
      OTEL_SERVICE_NAME: taskbook-server
      OTEL_RESOURCE_ATTRIBUTES: "deployment.environment=production"
    ports:
      - "8080:8080"
    depends_on:
      postgres:
        condition: service_healthy
```

## Local Development with a Collector

For local development, you can run an OpenTelemetry Collector instead of sending directly to Grafana Cloud:

```bash
# Start a local collector (e.g. via Docker)
docker run -p 4318:4318 otel/opentelemetry-collector-contrib

# Point the server at it
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
./tb-server
```

## Disabling OpenTelemetry

Simply unset `OTEL_EXPORTER_OTLP_ENDPOINT` (or don't set it). The server falls back to console-only logging with no OTel overhead.

## Shutdown Behaviour

The server holds a `TelemetryGuard` that flushes all pending traces, metrics, and logs on graceful shutdown (SIGTERM / Ctrl+C). This ensures no data is lost when the process exits.
