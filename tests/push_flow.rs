//! Push updates end to end: a real wren-relay on the loopback interface, the
//! real backend, and a local stand-in for Graph. Graph's part (validating the
//! URL, posting a change notification) is played by the test.

use std::sync::Arc;
use std::time::{Duration, Instant};

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wren::auth::session::AccountRecord;
use wren::backend::{AuthStatus, Backend, BackendOptions, Command, Event, Waker};
use wren::model::ConversationId;
use wren::paths::AppDirs;
use wren::realtime::PushStatus;
use wren::secrets::{MemoryStore, SecretStore};

const CLIENT: &str = "6731de76-14a6-49ae-97bc-6eba6914391e";
const ACCOUNT: &str = "u-me.t-1";
const CHAT: &str = "19:ada-me@unq.gbl.spaces";

static PENDING: std::sync::Mutex<Vec<Event>> = std::sync::Mutex::new(Vec::new());

fn wait_for<T>(backend: &Backend, seconds: u64, mut pick: impl FnMut(Event) -> Option<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        let mut pending = PENDING.lock().unwrap();
        pending.extend(backend.poll());
        while !pending.is_empty() {
            if let Some(found) = pick(pending.remove(0)) {
                return found;
            }
        }
        drop(pending);
        assert!(Instant::now() < deadline, "timed out waiting for an event");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_change_notification_fetches_the_chat() {
    // The relay.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let relay_url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    tokio::spawn(async move {
        axum_serve(listener).await;
    });

    // Graph and Entra.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/organizations/oauth2/v2.0/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "access", "refresh_token": "refresh-2", "expires_in": 3600,
            "scope": "User.Read Chat.ReadWrite ChatMessage.Send"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1.0/subscriptions"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "sub-1", "expirationDateTime": "2026-10-11T13:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1.0/subscriptions/sub-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1.0/me/chats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": []})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v1.0/chats/{CHAT}/messages")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": [
                {"id": "1702", "messageType": "message", "createdDateTime": "2026-10-11T10:05:00Z",
                 "lastModifiedDateTime": "2026-10-11T10:05:00Z",
                 "from": {"user": {"id": "u-ada", "displayName": "Ada Lovelace"}},
                 "body": {"contentType": "text", "content": "Pushed!"}}
            ]})),
        )
        .mount(&server)
        .await;

    // A signed-in account, as a previous run left it.
    let dir = tempfile::tempdir().unwrap();
    let dirs = AppDirs::under(dir.path());
    let secrets: Arc<dyn SecretStore> = Arc::new(MemoryStore::default());
    secrets
        .write(&wren::secrets::refresh_key(ACCOUNT), "refresh-1")
        .unwrap();
    std::fs::create_dir_all(dirs.account(ACCOUNT)).unwrap();
    AccountRecord {
        id: ACCOUNT.into(),
        display_name: "Alex Rivera".into(),
        username: "alex@contoso.com".into(),
        tenant_id: "t-1".into(),
        granted: "User.Read Chat.ReadWrite ChatMessage.Send".into(),
        client_id: CLIENT.into(),
        tenant: "organizations".into(),
    }
    .save(&dirs.account(ACCOUNT).join("account.json"))
    .unwrap();

    let backend = Backend::spawn_with(
        dirs,
        Waker::default(),
        BackendOptions {
            authority: server.uri(),
            graph_base: format!("{}/v1.0", server.uri()),
            secrets,
            open_browser: false,
        },
    );
    backend.send(Command::Start {
        client_id: CLIENT.into(),
        tenant: "organizations".into(),
        account: Some(ACCOUNT.into()),
    });
    wait_for(&backend, 5, |e| {
        matches!(e, Event::Auth(AuthStatus::SignedIn { .. })).then_some(())
    });
    backend.send(Command::Relay(Some(relay_url.clone())));
    wait_for(&backend, 10, |e| match e {
        Event::Push(PushStatus::Live) => Some(()),
        Event::Push(PushStatus::Failed(reason)) => panic!("push failed: {reason}"),
        _ => None,
    });

    // What Wren asked Graph for.
    let requests = server.received_requests().await.unwrap();
    let subscribe = requests
        .iter()
        .find(|r| r.url.path() == "/v1.0/subscriptions")
        .expect("a subscription request");
    let body: serde_json::Value = serde_json::from_slice(&subscribe.body).unwrap();
    assert_eq!(body["resource"], "/users/u-me/chats/getAllMessages");
    assert_eq!(body["includeResourceData"], false);
    let notification_url = body["notificationUrl"].as_str().unwrap().to_owned();
    let client_state = body["clientState"].as_str().unwrap().to_owned();
    assert!(notification_url.starts_with(&format!("{relay_url}/graph/")));

    // Graph's part: validate the URL, then post a change.
    let http = reqwest::Client::new();
    let echoed = http
        .post(format!("{notification_url}?validationToken=check"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(echoed, "check");
    let notification = serde_json::json!({"value": [{
        "subscriptionId": "sub-1", "clientState": client_state, "changeType": "created",
        "resource": format!("chats('{CHAT}')/messages('1702')"),
        "resourceData": {"id": "1702"}
    }]});
    let status = http
        .post(&notification_url)
        .json(&notification)
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, 202);

    let messages = wait_for(&backend, 10, |e| match e {
        Event::Messages {
            conversation,
            messages,
            cached: false,
            ..
        } if conversation == ConversationId::Chat(CHAT.into()) => Some(messages),
        _ => None,
    });
    assert_eq!(messages[0].body.to_plain(), "Pushed!");

    // Turning push off deletes the subscription.
    backend.send(Command::Relay(None));
    wait_for(&backend, 5, |e| {
        matches!(e, Event::Push(PushStatus::Off)).then_some(())
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let requests = server.received_requests().await.unwrap();
        if requests.iter().any(|r| r.method.as_str() == "DELETE") {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the subscription was not deleted"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn axum_serve(listener: tokio::net::TcpListener) {
    let app = wren_relay::router(wren_relay::Relay::default());
    wren_relay::serve(listener, app).await;
}
