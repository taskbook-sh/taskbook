use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::http::{Request, Response};
use opentelemetry::metrics::{Counter, Histogram, UpDownCounter};
use opentelemetry::{global, KeyValue};
use tower::{Layer, Service};

/// Tower [`Layer`] that records HTTP request metrics via OpenTelemetry.
///
/// Recorded instruments:
/// - `http.server.request.count` — counter by method, route, status
/// - `http.server.request.duration` — histogram (seconds) by method, route, status
/// - `http.server.active_requests` — up-down counter by method, route
#[derive(Clone)]
pub struct HttpMetricsLayer {
    request_count: Counter<u64>,
    request_duration: Histogram<f64>,
    active_requests: UpDownCounter<i64>,
}

impl HttpMetricsLayer {
    pub fn new() -> Self {
        let meter = global::meter("taskbook-server");

        let request_count = meter
            .u64_counter("http.server.request.count")
            .with_description("Total HTTP requests")
            .build();

        let request_duration = meter
            .f64_histogram("http.server.request.duration")
            .with_description("HTTP request duration in seconds")
            .with_unit("s")
            .build();

        let active_requests = meter
            .i64_up_down_counter("http.server.active_requests")
            .with_description("Number of in-flight HTTP requests")
            .build();

        Self {
            request_count,
            request_duration,
            active_requests,
        }
    }
}

impl<S> Layer<S> for HttpMetricsLayer {
    type Service = HttpMetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpMetricsService {
            inner,
            request_count: self.request_count.clone(),
            request_duration: self.request_duration.clone(),
            active_requests: self.active_requests.clone(),
        }
    }
}

#[derive(Clone)]
pub struct HttpMetricsService<S> {
    inner: S,
    request_count: Counter<u64>,
    request_duration: Histogram<f64>,
    active_requests: UpDownCounter<i64>,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for HttpMetricsService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let method = req.method().to_string();
        let path = normalize_path(req.uri().path());

        let active_attrs = vec![
            KeyValue::new("http.request.method", method.clone()),
            KeyValue::new("http.route", path.clone()),
        ];
        self.active_requests.add(1, &active_attrs);

        let request_count = self.request_count.clone();
        let request_duration = self.request_duration.clone();
        let active_requests = self.active_requests.clone();

        let mut inner = self.inner.clone();
        let start = Instant::now();

        Box::pin(async move {
            let result = inner.call(req).await;

            let elapsed = start.elapsed().as_secs_f64();
            active_requests.add(-1, &active_attrs);

            let status = match &result {
                Ok(resp) => resp.status().as_u16().to_string(),
                Err(_) => "500".to_string(),
            };

            let attrs = vec![
                KeyValue::new("http.request.method", method),
                KeyValue::new("http.route", path),
                KeyValue::new("http.response.status_code", status),
            ];

            request_count.add(1, &attrs);
            request_duration.record(elapsed, &attrs);

            result
        })
    }
}

/// Normalize the request path for use as a metric attribute.
///
/// The current API has no path parameters, so paths are used as-is.
/// This stub exists for future-proofing — add normalization here if
/// parameterised routes (e.g. `/items/:id`) are introduced later.
fn normalize_path(path: &str) -> String {
    path.to_string()
}
