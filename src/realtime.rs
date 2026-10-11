//! Push updates through a wren-relay (experimental).
//!
//! When a relay is configured, Wren:
//!
//! 1. opens a WebSocket to `{relay}/ws/{channel}` with a random channel;
//! 2. subscribes to Graph change notifications for all of the signed-in
//!    person's chat messages (`/users/{id}/chats/getAllMessages`, Chat.Read,
//!    delegated), delivered to `{relay}/graph/{channel}`, without resource
//!    data, so only IDs travel through the relay;
//! 3. on each notification whose `clientState` matches, fetches the changed
//!    chat with the usual date-ranged request. This is the pattern
//!    Microsoft's polling rules ask for;
//! 4. renews the subscription before it expires, reauthorizes it when Graph
//!    asks, recreates it after Graph removes it, and deletes it on exit.
//!
//! See docs/realtime.md. Not yet exercised against a live tenant.

use std::time::Duration;

use serde::Deserialize;

/// How long each subscription is asked to last. Renewed well before.
pub const LIFETIME: Duration = Duration::from_secs(55 * 60);
pub const RENEW_EVERY: Duration = Duration::from_secs(40 * 60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PushStatus {
    Off,
    Connecting,
    /// Subscribed; changes arrive as they happen.
    Live,
    /// Not working; retrying. The reason is for the person.
    Failed(String),
}

impl PushStatus {
    pub fn label(&self) -> String {
        match self {
            Self::Off => "Off".into(),
            Self::Connecting => "Connecting…".into(),
            Self::Live => "Live".into(),
            Self::Failed(reason) => format!("Not connected: {reason}"),
        }
    }
}

/// The relay's base URL, checked: `https://host[:port][/path]`, or
/// `http://` on the loopback interface for local testing.
pub fn normalize_relay(url: &str) -> Option<String> {
    let url = url.trim().trim_end_matches('/');
    let rest = url.strip_prefix("https://").or_else(|| {
        url.strip_prefix("http://").filter(|rest| {
            rest.starts_with("127.0.0.1")
                || rest.starts_with("localhost")
                || rest.starts_with("[::1]")
        })
    })?;
    let host = rest.split('/').next().unwrap_or_default();
    (!host.is_empty() && !host.contains('@') && !url.contains('?') && !url.contains('#'))
        .then(|| url.to_owned())
}

pub fn websocket_url(relay: &str, channel: &str) -> String {
    let base = if let Some(rest) = relay.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = relay.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        relay.to_owned()
    };
    format!("{base}/ws/{channel}")
}

pub fn notification_url(relay: &str, channel: &str) -> String {
    format!("{relay}/graph/{channel}")
}

pub fn lifecycle_url(relay: &str, channel: &str) -> String {
    format!("{relay}/lifecycle/{channel}")
}

/// What a notification asks Wren to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    /// A message in this chat was created, edited or deleted.
    Chat(String),
    /// Graph wants the subscription reauthorized (the token behind it is
    /// about to lapse).
    Reauthorize,
    /// Notifications were lost; refresh the chat list.
    Missed,
    /// Graph removed the subscription; make a new one.
    Removed,
}

#[derive(Deserialize)]
struct Batch {
    #[serde(default)]
    value: Vec<Item>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(default)]
    client_state: Option<String>,
    #[serde(default)]
    resource: Option<String>,
    #[serde(default)]
    lifecycle_event: Option<String>,
}

/// Reads a notification batch. Items whose `clientState` is not ours are
/// dropped: the relay, or anyone who guessed the URL, cannot make Wren do
/// more than fetch, but foreign items are still ignored.
pub fn parse(body: &str, client_state: &str) -> Vec<Notice> {
    let Ok(batch) = serde_json::from_str::<Batch>(body) else {
        return Vec::new();
    };
    let mut notices = Vec::new();
    for item in batch.value {
        if item.client_state.as_deref() != Some(client_state) {
            continue;
        }
        let notice = match item.lifecycle_event.as_deref() {
            Some("reauthorizationRequired") => Some(Notice::Reauthorize),
            Some("missed") => Some(Notice::Missed),
            Some("subscriptionRemoved") => Some(Notice::Removed),
            Some(_) => None,
            None => item.resource.as_deref().and_then(chat_of).map(Notice::Chat),
        };
        if let Some(notice) = notice
            && !notices.contains(&notice)
        {
            notices.push(notice);
        }
    }
    notices
}

/// The chat id in a notification's resource path, in either of Graph's
/// spellings: `chats('19:…')/messages('…')` or `chats/19:…/messages/…`.
fn chat_of(resource: &str) -> Option<String> {
    let rest = resource.trim_start_matches('/');
    let rest = rest.strip_prefix("chats")?;
    let id = if let Some(quoted) = rest.strip_prefix("('") {
        quoted.split("')").next()?
    } else {
        rest.strip_prefix('/')?.split('/').next()?
    };
    let id = urlencoding::decode(id).ok()?.into_owned();
    (!id.is_empty()).then_some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_urls_must_be_https_or_loopback() {
        assert_eq!(
            normalize_relay("https://relay.example.com/"),
            Some("https://relay.example.com".into())
        );
        assert_eq!(
            normalize_relay("http://127.0.0.1:8787"),
            Some("http://127.0.0.1:8787".into())
        );
        assert_eq!(normalize_relay("http://relay.example.com"), None);
        assert_eq!(normalize_relay("https://user@relay.example.com"), None);
        assert_eq!(normalize_relay("ftp://x"), None);
        assert_eq!(
            websocket_url("https://r.example/base", "c"),
            "wss://r.example/base/ws/c"
        );
        assert_eq!(
            notification_url("https://r.example", "c"),
            "https://r.example/graph/c"
        );
    }

    #[test]
    fn notifications_name_their_chat() {
        let body = r#"{"value":[
            {"subscriptionId":"s1","clientState":"ours","changeType":"created","resource":"chats('19:abc@thread.v2')/messages('1700')","resourceData":{"id":"1700"}},
            {"subscriptionId":"s1","clientState":"ours","changeType":"updated","resource":"chats('19:abc@thread.v2')/messages('1701')"},
            {"subscriptionId":"s1","clientState":"theirs","changeType":"created","resource":"chats('19:evil')/messages('1')"},
            {"subscriptionId":"s1","clientState":"ours","changeType":"created","resource":"/chats/19%3Adef%40unq.gbl.spaces/messages/9"}
        ]}"#;
        assert_eq!(
            parse(body, "ours"),
            vec![
                Notice::Chat("19:abc@thread.v2".into()),
                Notice::Chat("19:def@unq.gbl.spaces".into())
            ]
        );
    }

    #[test]
    fn lifecycle_events() {
        let body = r#"{"value":[{"subscriptionId":"s1","clientState":"ours","lifecycleEvent":"reauthorizationRequired"},
            {"clientState":"ours","lifecycleEvent":"missed"},{"clientState":"ours","lifecycleEvent":"subscriptionRemoved"}]}"#;
        assert_eq!(
            parse(body, "ours"),
            vec![Notice::Reauthorize, Notice::Missed, Notice::Removed]
        );
        assert!(parse("not json", "ours").is_empty());
    }
}
