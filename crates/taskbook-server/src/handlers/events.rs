use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use opentelemetry::metrics::UpDownCounter;
use opentelemetry::{global, KeyValue};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::middleware::AuthUser;
use crate::router::{AppState, SyncEvent};

/// Guard that decrements the SSE active-connections counter on drop.
struct SseConnectionGuard {
    counter: UpDownCounter<i64>,
}

impl SseConnectionGuard {
    fn new() -> Self {
        let meter = global::meter("taskbook-server");
        let counter = meter
            .i64_up_down_counter("sse.active_connections")
            .with_description("Number of active SSE connections")
            .build();
        counter.add(1, &[KeyValue::new("endpoint", "/api/v1/events")]);
        Self { counter }
    }
}

impl Drop for SseConnectionGuard {
    fn drop(&mut self) {
        self.counter
            .add(-1, &[KeyValue::new("endpoint", "/api/v1/events")]);
    }
}

/// Wrapper stream that holds a [`SseConnectionGuard`]. When the client
/// disconnects and the stream is dropped, the counter is automatically
/// decremented.
struct TrackedStream<S> {
    inner: S,
    _guard: SseConnectionGuard,
}

impl<S> Stream for TrackedStream<S>
where
    S: Stream + Unpin,
{
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// SSE endpoint that streams real-time sync notifications to authenticated clients.
#[tracing::instrument(skip(state))]
pub async fn events(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.notifications.subscribe(auth.user_id);

    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(SyncEvent::DataChanged { archived }) => {
            let data = if archived { "archive" } else { "items" };
            Some(Ok(Event::default().event("data_changed").data(data)))
        }
        // Lagged: receiver fell behind — tell the client to do a full refresh.
        Err(_) => Some(Ok(Event::default().event("data_changed").data("items"))),
    });

    let tracked = TrackedStream {
        inner: stream,
        _guard: SseConnectionGuard::new(),
    };

    Sse::new(tracked).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
