use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::middleware::AuthUser;
use crate::router::{AppState, SyncEvent};

/// SSE endpoint that streams real-time sync notifications to authenticated clients.
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

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
