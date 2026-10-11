//! wren-relay: forwards Microsoft Graph change notifications to Wren.
//!
//! Graph can only push change notifications to a public HTTPS endpoint, and a
//! desktop app has none. This relay is that endpoint, small enough for
//! anyone to run behind their own TLS proxy:
//!
//! - Wren opens a WebSocket at `/ws/{channel}`. The channel is a random token
//!   Wren makes for each run.
//! - Wren creates a Graph subscription whose `notificationUrl` is
//!   `/graph/{channel}` and whose `lifecycleNotificationUrl` is
//!   `/lifecycle/{channel}`.
//! - Graph validates each URL with a `validationToken`, which the relay echoes
//!   back. Graph then posts notifications there, and the relay hands each body
//!   to the channel's WebSocket and answers `202 Accepted` at once.
//!
//! Wren subscribes without resource data, so notifications carry only IDs
//! (chat and message), never message content. Wren fetches the content
//! itself from Graph. The relay keeps nothing on disk and forgets a channel
//! when its WebSocket closes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use tokio::sync::mpsc;

/// Graph notification batches are small; anything bigger is not Graph.
const BODY_LIMIT: usize = 512 * 1024;
/// How many notifications may wait for a slow client before new ones drop.
const QUEUE: usize = 256;

#[derive(Clone, Default)]
pub struct Relay {
    channels: Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>,
}

impl Relay {
    pub fn connected(&self) -> usize {
        self.channels
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .len()
    }
}

/// The relay's routes.
pub fn router(relay: Relay) -> Router {
    Router::new()
        .route("/", get(|| async { "wren-relay ok\n" }))
        .route("/ws/{channel}", get(connect))
        .route("/graph/{channel}", post(notification))
        .route("/lifecycle/{channel}", post(notification))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(relay)
}

/// Channels are Wren's random tokens: long, URL-safe, nothing else.
fn valid_channel(channel: &str) -> bool {
    (32..=128).contains(&channel.len())
        && channel
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

async fn connect(
    ws: WebSocketUpgrade,
    Path(channel): Path<String>,
    State(relay): State<Relay>,
) -> Response {
    if !valid_channel(&channel) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    ws.on_upgrade(move |socket| serve_client(socket, channel, relay))
}

async fn serve_client(mut socket: WebSocket, channel: String, relay: Relay) {
    let (sender, mut receiver) = mpsc::channel::<String>(QUEUE);
    relay
        .channels
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(channel.clone(), sender.clone());
    log::info!("client connected ({} connected)", relay.connected());
    loop {
        tokio::select! {
            forwarded = receiver.recv() => {
                let Some(body) = forwarded else { break };
                if socket.send(Message::Text(body.into())).await.is_err() {
                    break;
                }
            }
            incoming = socket.recv() => match incoming {
                // Clients only ping; anything else is ignored.
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                Some(Ok(_)) => {}
            },
        }
    }
    let mut channels = relay.channels.lock().unwrap_or_else(|p| p.into_inner());
    // A reconnect may already have replaced this client; keep the new one.
    if channels
        .get(&channel)
        .is_some_and(|current| current.same_channel(&sender))
    {
        channels.remove(&channel);
    }
    log::info!("client disconnected ({} connected)", channels.len());
}

#[derive(Deserialize)]
struct Validation {
    #[serde(rename = "validationToken")]
    validation_token: Option<String>,
}

async fn notification(
    Path(channel): Path<String>,
    Query(query): Query<Validation>,
    State(relay): State<Relay>,
    body: String,
) -> Response {
    if !valid_channel(&channel) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    // Graph checks a new notification URL by posting a token that must come
    // back as plain text within ten seconds.
    if let Some(token) = query.validation_token {
        return ([(header::CONTENT_TYPE, "text/plain")], token).into_response();
    }
    let sender = relay
        .channels
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&channel)
        .cloned();
    match sender {
        Some(sender) => {
            if sender.try_send(body).is_err() {
                log::warn!("a client is not keeping up; dropped a notification");
            }
        }
        None => log::info!("notification for a channel with no client; dropped"),
    }
    // Graph must hear back quickly whatever happens, or it slows down and
    // eventually drops the subscription.
    StatusCode::ACCEPTED.into_response()
}

/// Serves `app` on `listener` until the process ends.
pub async fn serve(listener: tokio::net::TcpListener, app: Router) {
    if let Err(error) = axum::serve(listener, app).await {
        log::error!("relay stopped: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    const CHANNEL: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    async fn start() -> (String, Relay) {
        let relay = Relay::default();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = router(relay.clone());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (format!("127.0.0.1:{}", address.port()), relay)
    }

    #[tokio::test]
    async fn validation_tokens_are_echoed_as_text() {
        let (host, _) = start().await;
        let response = reqwest::Client::new()
            .post(format!("http://{host}/graph/{CHANNEL}?validationToken=Validation%3A%20Testing%20client%20application"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["content-type"], "text/plain");
        assert_eq!(
            response.text().await.unwrap(),
            "Validation: Testing client application"
        );
    }

    #[tokio::test]
    async fn notifications_reach_the_channel_websocket() {
        let (host, relay) = start().await;
        let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{host}/ws/{CHANNEL}"))
            .await
            .unwrap();
        // Wait until the relay has registered the channel.
        while relay.connected() == 0 {
            tokio::task::yield_now().await;
        }
        let body = r#"{"value":[{"clientState":"s","resource":"chats('19:a')/messages('1')"}]}"#;
        let status = reqwest::Client::new()
            .post(format!("http://{host}/graph/{CHANNEL}"))
            .body(body)
            .send()
            .await
            .unwrap()
            .status();
        assert_eq!(status, 202);
        let message = socket.next().await.unwrap().unwrap();
        assert_eq!(message.into_text().unwrap().as_str(), body);
    }

    #[tokio::test]
    async fn bad_channels_and_unknown_channels() {
        let (host, _) = start().await;
        let client = reqwest::Client::new();
        let short = client
            .post(format!("http://{host}/graph/short"))
            .body("{}")
            .send()
            .await
            .unwrap();
        assert_eq!(short.status(), 400);
        // Nobody listening: still accepted, so Graph keeps the subscription.
        let unknown = client
            .post(format!("http://{host}/lifecycle/{CHANNEL}"))
            .body("{}")
            .send()
            .await
            .unwrap();
        assert_eq!(unknown.status(), 202);
    }
}
