use opentelemetry::propagation::TextMapCompositePropagator;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{self as sdk, Resource};
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Guard that flushes and shuts down OTel providers on drop.
pub struct TelemetryGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
    logger_provider: SdkLoggerProvider,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Err(e) = self.tracer_provider.shutdown() {
            eprintln!("failed to shut down tracer provider: {e}");
        }
        if let Err(e) = self.meter_provider.shutdown() {
            eprintln!("failed to shut down meter provider: {e}");
        }
        if let Err(e) = self.logger_provider.shutdown() {
            eprintln!("failed to shut down logger provider: {e}");
        }
    }
}

/// Initialise telemetry.
///
/// When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, full OpenTelemetry pipelines
/// (traces, metrics, logs) are configured and exported via OTLP HTTP/protobuf.
/// Otherwise, only console `fmt` logging is enabled (identical to previous
/// behaviour).
///
/// Returns `Some(TelemetryGuard)` when OTel is active — the guard **must** be
/// held until the end of `main` to ensure a clean flush on shutdown.
pub fn init_telemetry() -> Option<TelemetryGuard> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let otel_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

    if let Some(endpoint) = otel_endpoint {
        // --- OTel-enabled path ---

        let service_name =
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "taskbook-server".to_string());

        let resource = Resource::builder()
            .with_attributes([
                KeyValue::new(
                    opentelemetry_semantic_conventions::attribute::SERVICE_NAME,
                    service_name,
                ),
                KeyValue::new(
                    opentelemetry_semantic_conventions::attribute::SERVICE_VERSION,
                    env!("CARGO_PKG_VERSION"),
                ),
            ])
            .build();

        // W3C TraceContext propagator
        let propagator =
            TextMapCompositePropagator::new(vec![Box::new(TraceContextPropagator::new())]);
        global::set_text_map_propagator(propagator);

        // --- Traces ---
        // Do not call .with_endpoint() — the SDK reads OTEL_EXPORTER_OTLP_ENDPOINT
        // and OTEL_EXPORTER_OTLP_HEADERS automatically, appending /v1/traces for HTTP.
        let trace_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()
            .expect("failed to create OTLP trace exporter");

        let tracer_provider = SdkTracerProvider::builder()
            .with_batch_exporter(trace_exporter)
            .with_resource(resource.clone())
            .build();

        let tracer = tracer_provider.tracer("taskbook-server");

        // --- Metrics ---
        let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .build()
            .expect("failed to create OTLP metric exporter");

        let metric_reader = sdk::metrics::PeriodicReader::builder(metric_exporter)
            .with_interval(std::time::Duration::from_secs(15))
            .build();

        let meter_provider = SdkMeterProvider::builder()
            .with_reader(metric_reader)
            .with_resource(resource.clone())
            .build();

        global::set_meter_provider(meter_provider.clone());

        // --- Logs ---
        let log_exporter = opentelemetry_otlp::LogExporter::builder()
            .with_http()
            .build()
            .expect("failed to create OTLP log exporter");

        let logger_provider = SdkLoggerProvider::builder()
            .with_batch_exporter(log_exporter)
            .with_resource(resource)
            .build();

        // Compose subscriber layers
        let fmt_layer = tracing_subscriber::fmt::layer();
        let otel_trace_layer = OpenTelemetryLayer::new(tracer);
        let otel_metrics_layer = MetricsLayer::new(meter_provider.clone());
        let otel_logs_layer = OpenTelemetryTracingBridge::new(&logger_provider);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_trace_layer)
            .with(otel_metrics_layer)
            .with(otel_logs_layer)
            .init();

        tracing::info!("OpenTelemetry enabled — exporting to {endpoint}");

        Some(TelemetryGuard {
            tracer_provider,
            meter_provider,
            logger_provider,
        })
    } else {
        // --- Disabled path (console-only) ---
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();

        None
    }
}

/// Spawn background-observable gauges that report the DB connection pool state.
///
/// Only meaningful when OTel is active, but safe to call regardless — when no
/// meter provider is configured the callbacks are simply never invoked.
pub fn spawn_db_pool_metrics(pool: sqlx::PgPool) {
    let meter = global::meter("taskbook-server");

    let pool_total = pool.clone();
    let _total_gauge = meter
        .u64_observable_gauge("db.pool.connections")
        .with_description("Total connections in the database pool")
        .with_callback(move |observer| {
            observer.observe(pool_total.size() as u64, &[]);
        })
        .build();

    let _idle_gauge = meter
        .u64_observable_gauge("db.pool.idle_connections")
        .with_description("Idle connections in the database pool")
        .with_callback(move |observer| {
            observer.observe(pool.num_idle() as u64, &[]);
        })
        .build();
}
