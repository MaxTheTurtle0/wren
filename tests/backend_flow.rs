//! The whole backend against local stand-ins for Entra ID and Graph: sign in
//! through the loopback redirect, list chats, read and send messages,
//! restore the session and cache on restart, and sign out.
//!
//! Only Microsoft's servers are simulated; everything else is the real code
//! path (the runtime thread, the worker, PKCE, the token exchange, the
//! Graph client, the SQLite cache).

use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wren::backend::{AuthStatus, Backend, BackendOptions, Command, Event, Waker};
use wren::model::ConversationId;
use wren::paths::AppDirs;
use wren::secrets::{MemoryStore, SecretStore};
use wren::sync::{Resource, Trigger};

const CLIENT: &str = "6731de76-14a6-49ae-97bc-6eba6914391e";
const CHAT: &str = "19:ada-me@unq.gbl.spaces";

fn id_token() -> String {
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
        br#"{"oid":"u-me","tid":"t-1","name":"Alex Rivera","preferred_username":"alex@contoso.com"}"#,
    );
    format!("eyJhbGciOiJub25lIn0.{payload}.")
}

/// Events not yet looked at, so a wait never drops the rest of a batch.
static PENDING: std::sync::Mutex<Vec<Event>> = std::sync::Mutex::new(Vec::new());

/// Waits for the first event `pick` accepts, failing after `seconds`.
/// Events before it are skipped; events after it stay for the next wait.
fn wait_for<T>(backend: &Backend, seconds: u64, mut pick: impl FnMut(Event) -> Option<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        let mut pending = PENDING.lock().unwrap();
        pending.extend(backend.poll());
        while !pending.is_empty() {
            let event = pending.remove(0);
            if std::env::var("WREN_TEST_EVENTS").is_ok() {
                eprintln!("event: {event:?}");
            }
            if let Some(found) = pick(event) {
                return found;
            }
        }
        drop(pending);
        assert!(Instant::now() < deadline, "timed out waiting for an event");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn query_value(url: &str, key: &str) -> String {
    let query = url.split_once('?').map(|(_, q)| q).unwrap_or_default();
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix(&format!("{key}=")))
        .map(|value| urlencoding::decode(value).unwrap().into_owned())
        .unwrap_or_default()
}

async fn mock_microsoft() -> MockServer {
    let server = MockServer::start().await;
    // Entra: the code Wren's loopback received, with the PKCE verifier.
    Mock::given(method("POST"))
        .and(path("/organizations/oauth2/v2.0/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .and(body_string_contains("code=the-code"))
        .and(body_string_contains("code_verifier="))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "access-1",
            "refresh_token": "refresh-1",
            "id_token": id_token(),
            "expires_in": 3600,
            "scope": "User.Read Chat.ReadWrite ChatMessage.Send Team.ReadBasic.All Channel.ReadBasic.All ChannelMessage.Send Presence.Read.All Calendars.Read"
        })))
        .mount(&server)
        .await;
    // Entra: the stored refresh token, after a restart.
    Mock::given(method("POST"))
        .and(path("/organizations/oauth2/v2.0/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains("refresh_token=refresh-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "access-2", "refresh_token": "refresh-2", "expires_in": 3600,
            "scope": "User.Read Chat.ReadWrite ChatMessage.Send"
        })))
        .mount(&server)
        .await;
    // Graph.
    Mock::given(method("GET"))
        .and(path("/v1.0/me/chats"))
        .and(header("authorization", "Bearer access-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": [{
            "id": CHAT, "chatType": "oneOnOne", "lastUpdatedDateTime": "2026-10-11T10:00:00Z",
            "members": [
                {"userId": "u-ada", "displayName": "Ada Lovelace", "email": "ada@contoso.com"},
                {"userId": "u-me", "displayName": "Alex Rivera", "email": "alex@contoso.com"}
            ],
            "lastMessagePreview": {"id": "1700", "createdDateTime": "2026-10-11T10:00:00Z", "messageType": "message",
                "body": {"contentType": "html", "content": "<p>Ship it?</p>"},
                "from": {"user": {"id": "u-ada", "displayName": "Ada Lovelace"}}},
            "viewpoint": {"lastMessageReadDateTime": "2026-10-11T09:00:00Z"}
        }]})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v1.0/chats/{CHAT}/messages")))
        .and(query_param("$orderby", "createdDateTime desc"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": [
                {"id": "1700", "messageType": "message", "createdDateTime": "2026-10-11T10:00:00Z",
                 "lastModifiedDateTime": "2026-10-11T10:00:00Z",
                 "from": {"user": {"id": "u-ada", "displayName": "Ada Lovelace"}},
                 "body": {"contentType": "html", "content": "<p>Ship <b>it</b>?</p>"}},
                {"id": "1690", "messageType": "message", "createdDateTime": "2026-10-11T09:50:00Z",
                 "lastModifiedDateTime": "2026-10-11T09:50:00Z",
                 "from": {"user": {"id": "u-me", "displayName": "Alex Rivera"}},
                 "body": {"contentType": "html", "content": "<p>Build is green</p>"}}
            ]})),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/v1.0/chats/{CHAT}/messages")))
        .and(body_string_contains("<strong>today</strong>"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "1701", "messageType": "message", "createdDateTime": "2026-10-11T10:01:00Z",
            "lastModifiedDateTime": "2026-10-11T10:01:00Z",
            "from": {"user": {"id": "u-me", "displayName": "Alex Rivera"}},
            "body": {"contentType": "html", "content": "<p>Shipping <strong>today</strong></p>"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    // A restored conversation refreshes by modification date, independently
    // of the newest-page request above. No messages changed in this fixture.
    Mock::given(method("GET"))
        .and(path(format!("/v1.0/chats/{CHAT}/messages")))
        .and(query_param("$orderby", "lastModifiedDateTime desc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": []})))
        .mount(&server)
        .await;
    server
}

#[tokio::test(flavor = "multi_thread")]
async fn sign_in_sync_send_restore_and_sign_out() {
    let server = mock_microsoft().await;
    let dir = tempfile::tempdir().unwrap();
    let dirs = AppDirs::under(dir.path());
    let secrets: Arc<dyn SecretStore> = Arc::new(MemoryStore::default());
    let options = BackendOptions {
        authority: server.uri(),
        graph_base: format!("{}/v1.0", server.uri()),
        secrets: Arc::clone(&secrets),
        open_browser: false,
    };

    // First run: no account yet.
    let backend = Backend::spawn_with(dirs.clone(), Waker::default(), options.clone());
    backend.send(Command::Start {
        client_id: CLIENT.into(),
        tenant: "organizations".into(),
        account: None,
    });
    wait_for(&backend, 5, |e| {
        matches!(e, Event::Auth(AuthStatus::SignedOut)).then_some(())
    });

    // Sign in: play the browser's part by following the redirect.
    backend.send(Command::SignIn {
        extra_scopes: Vec::new(),
        device_code: false,
    });
    let url = wait_for(&backend, 5, |e| match e {
        Event::Auth(AuthStatus::WaitingForBrowser { url }) => Some(url),
        _ => None,
    });
    assert!(url.starts_with(&format!(
        "{}/organizations/oauth2/v2.0/authorize?",
        server.uri()
    )));
    assert_eq!(query_value(&url, "code_challenge_method"), "S256");
    let redirect = query_value(&url, "redirect_uri");
    let state = query_value(&url, "state");
    let port = redirect.trim_start_matches("http://localhost:");
    let landing = reqwest::get(format!(
        "http://127.0.0.1:{port}/?code=the-code&state={state}"
    ))
    .await
    .unwrap()
    .text()
    .await
    .unwrap();
    assert!(landing.contains("signed in"));
    let account = wait_for(&backend, 10, |e| match e {
        Event::Auth(AuthStatus::SignedIn { account, name, .. }) => {
            assert_eq!(name, "Alex Rivera");
            Some(account)
        }
        Event::Auth(AuthStatus::Failed(reason)) => panic!("sign-in failed: {reason}"),
        _ => None,
    });
    assert_eq!(account, "u-me.t-1");
    assert_eq!(
        secrets
            .read(&wren::secrets::refresh_key(&account))
            .unwrap()
            .as_deref(),
        Some("refresh-1")
    );

    // Chats, then the chat's messages, then a send.
    backend.send(Command::Fetch {
        resource: Resource::Chats,
        trigger: Trigger::Start,
    });
    let chats = wait_for(&backend, 10, |e| match e {
        Event::Chats {
            chats,
            cached: false,
        } => Some(chats),
        _ => None,
    });
    assert_eq!(chats.len(), 1);
    assert_eq!(chats[0].title("u-me"), "Ada Lovelace");
    assert!(chats[0].is_unread("u-me"));

    let conversation = ConversationId::Chat(CHAT.into());
    backend.send(Command::Fetch {
        resource: Resource::Messages(conversation.clone()),
        trigger: Trigger::Opened,
    });
    let messages = wait_for(&backend, 10, |e| match e {
        Event::Messages {
            messages,
            cached: false,
            ..
        } => Some(messages),
        _ => None,
    });
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].body.to_plain(), "Ship it?");

    backend.send(Command::Send {
        conversation: conversation.clone(),
        local_id: "local-1".into(),
        html: wren::html::compose("Shipping **today**"),
        reply_to: None,
        mentions: Vec::new(),
        files: Vec::new(),
        images: Vec::new(),
        recipients: Vec::new(),
    });
    let sent = wait_for(&backend, 10, |e| match e {
        Event::Sent {
            local_id, message, ..
        } => Some((local_id, message)),
        Event::SendFailed { reason, .. } => panic!("send failed: {reason}"),
        _ => None,
    });
    assert_eq!(sent.0, "local-1");
    assert_eq!(sent.1.id, "1701");
    drop(backend);
    PENDING.lock().unwrap().clear();

    // Second run: the session comes back from the credential store, and the
    // cache is shown before any request.
    let backend = Backend::spawn_with(dirs.clone(), Waker::default(), options.clone());
    backend.send(Command::Start {
        client_id: CLIENT.into(),
        tenant: "organizations".into(),
        account: Some(account.clone()),
    });
    let cached = wait_for(&backend, 5, |e| match e {
        Event::Chats {
            chats,
            cached: true,
        } => Some(chats),
        _ => None,
    });
    assert_eq!(cached.len(), 1);
    wait_for(&backend, 5, |e| {
        matches!(e, Event::Auth(AuthStatus::SignedIn { .. })).then_some(())
    });
    backend.send(Command::Fetch {
        resource: Resource::Messages(conversation.clone()),
        trigger: Trigger::Opened,
    });
    let cached_messages = wait_for(&backend, 5, |e| match e {
        Event::Messages {
            messages,
            cached: true,
            more: Some(true),
            ..
        } => Some(messages),
        _ => None,
    });
    assert_eq!(cached_messages.len(), 3, "two fetched and one sent");
    // Opening serves the cache before its Graph refresh completes. Wait for
    // that refresh so its requests cannot be mistaken for a cache-only reopen.
    wait_for(&backend, 5, |e| match e {
        Event::Messages {
            conversation: fetched,
            cached: false,
            ..
        } if fetched == conversation => Some(()),
        _ => None,
    });
    // RAM eviction can reopen a conversation without claiming a Graph fetch.
    let requests_before = server.received_requests().await.unwrap().len();
    backend.send(Command::CachedMessages(conversation.clone()));
    let reopened = wait_for(&backend, 5, |e| match e {
        Event::Messages {
            messages,
            cached: true,
            ..
        } => Some(messages),
        _ => None,
    });
    assert_eq!(reopened.len(), 3);
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        requests_before
    );
    let before = cached_messages.iter().map(|m| m.created).min().unwrap();
    Mock::given(method("GET"))
        .and(path(format!("/v1.0/chats/{CHAT}/messages")))
        .and(query_param(
            "$filter",
            format!("createdDateTime lt {}", wren::time::to_iso(before)),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": [{
                "id": "1600", "messageType": "message", "createdDateTime": "2026-10-10T09:00:00Z",
                "body": {"contentType": "text", "content": "Earlier history"}
            }]})),
        )
        .expect(1)
        .with_priority(1)
        .mount(&server)
        .await;
    backend.send(Command::LoadOlder {
        conversation: conversation.clone(),
        before: Some(before),
    });
    let earlier = wait_for(&backend, 5, |e| match e {
        Event::Messages {
            messages,
            merge: wren::backend::Merge::Older,
            more: Some(false),
            cached: false,
            ..
        } => Some(messages),
        _ => None,
    });
    assert_eq!(
        earlier[0].id, "1600",
        "reopened cached chats can load yesterday's predecessors"
    );
    // The refresh token was used and rotated by the date-ranged refresh.
    let deadline = Instant::now() + Duration::from_secs(5);
    while secrets
        .read(&wren::secrets::refresh_key(&account))
        .unwrap()
        .as_deref()
        != Some("refresh-2")
    {
        assert!(
            Instant::now() < deadline,
            "the rotated refresh token was not stored"
        );
        std::thread::sleep(Duration::from_millis(20));
    }

    // Reopening an evicted image uses the account's encoded cache, with no
    // second Graph download. This remains available after RAM eviction.
    Mock::given(method("GET"))
        .and(path("/v1.0/hosted-image"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"image fixture".to_vec()))
        .expect(1)
        .mount(&server)
        .await;
    let media_url = format!("{}/v1.0/hosted-image", server.uri());
    for _ in 0..2 {
        backend.send(Command::Media(media_url.clone()));
        let bytes = wait_for(&backend, 5, |e| match e {
            Event::Media { url, bytes } if url == media_url => Some(bytes),
            _ => None,
        });
        assert_eq!(bytes, Some(b"image fixture".to_vec()));
    }

    // Sign out: the token and the cache are gone.
    backend.send(Command::SignOut);
    wait_for(&backend, 5, |e| {
        matches!(e, Event::Auth(AuthStatus::SignedOut)).then_some(())
    });
    assert_eq!(
        secrets.read(&wren::secrets::refresh_key(&account)).unwrap(),
        None
    );
    assert!(!dirs.account(&account).exists());
}
