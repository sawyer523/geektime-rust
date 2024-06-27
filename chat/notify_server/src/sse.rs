use std::{convert::Infallible, time::Duration};
use std::sync::Arc;

use axum::{
    Extension,
    extract::State,
    response::{Sse, sse::Event},
};
use futures::Stream;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use chat_core::User;

use crate::{AppEvent, AppState};

const CHANNEL_CAPACITY: usize = 256;

struct Guard {
    user_id: u64,
    state: AppState,
}

pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // let user_id = user.id as u64;
    let user_id = user.id as u64;
    let users = &state.users;

    let rx = if let Some(tx) = users.get(&user_id) {
        tx.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        state.users.insert(user_id, tx);
        rx
    };

    let guard = Arc::new(Guard {
        user_id,
        state: state.clone(),
    });

    let stream = async_stream::stream! {
        let _guard = guard.clone();
        let mut broadcast_stream = BroadcastStream::new(rx);

        while let Some(result) = broadcast_stream.next().await {
            if let Ok(v) = result {
                let name = match v.as_ref() {
                    AppEvent::NewChat(_) => "NewChat",
                    AppEvent::AddToChat(_) => "AddToChat",
                    AppEvent::UpdateChatName(_) => "UpdateChatName",
                    AppEvent::RemoveFromChat(_) => "RemoveFromChat",
                    AppEvent::NewMessage(_) => "NewMessage",
                };
                let v = serde_json::to_string(&v).expect("failed to serialize event");
                yield Ok(Event::default().data(v).event(name));
            }
        }
        // `_guard` is dropped here when the stream ends
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.state.users.remove(&self.user_id);
        tracing::info!("User {} disconnected, cleaned up state", self.user_id);
    }
}
