//! The 0.2 features end to end through the real backend, against a local
//! stand-in for Graph: people search, a new chat, a message with a mention
//! and a file (upload, share, reference), availability, the presence
//! session, the status message, and finding a meeting's chat from its join
//! link. Each request's shape is checked against what Microsoft documents.

use std::sync::Arc;
use std::time::{Duration, Instant};

use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};
use wren::auth::session::AccountRecord;
use wren::backend::{AuthStatus, Backend, BackendOptions, Command, Event, Waker};
use wren::model::{Availability, ConversationId, MentionRef};
use wren::paths::AppDirs;
use wren::secrets::{MemoryStore, SecretStore};

const CLIENT: &str = "6731de76-14a6-49ae-97bc-6eba6914391e";
const ACCOUNT: &str = "u-me.t-1";
const GRANTED: &str = "User.Read Chat.ReadWrite ChatMessage.Send Chat.Create Files.ReadWrite Presence.ReadWrite OnlineMeetings.Read";
const JOIN: &str = "https://teams.microsoft.com/l/meetup-join/19%3ameeting_abc%40thread.v2/0";

static PENDING: std::sync::Mutex<Vec<Event>> = std::sync::Mutex::new(Vec::new());

fn wait_for<T>(backend: &Backend, seconds: u64, mut pick: impl FnMut(Event) -> Option<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        let mut pending = PENDING.lock().unwrap();
        pending.extend(backend.poll());
        while !pending.is_empty() {
            let event = pending.remove(0);
            if let Event::Error(text) = &event {
                eprintln!("backend error: {text}");
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

async fn requests_to(server: &MockServer, route: &str) -> Vec<Request> {
    server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.url.path() == route)
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn chats_mentions_files_and_status() {
    let server = MockServer::start().await;
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    Mock::given(method("POST"))
        .and(path("/organizations/oauth2/v2.0/token"))
        .respond_with(ok(serde_json::json!({
            "access_token": "access", "refresh_token": "refresh-2", "expires_in": 3600, "scope": GRANTED
        })))
        .mount(&server)
        .await;
    // People search: eventual consistency, name or address.
    Mock::given(method("GET"))
        .and(path("/v1.0/users"))
        .and(header("consistencylevel", "eventual"))
        .and(query_param(
            "$search",
            "\"displayName:barb\" OR \"mail:barb\"",
        ))
        .respond_with(ok(serde_json::json!({"value": [
            {"id": "u-barbara", "displayName": "Barbara Liskov", "mail": "barbara@contoso.com"}
        ]})))
        .mount(&server)
        .await;
    // A one-on-one chat with Barbara; Graph's answer has no members.
    Mock::given(method("POST"))
        .and(path("/v1.0/chats"))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            serde_json::json!({"id": "19:me_barbara@unq.gbl.spaces", "chatType": "oneOnOne"}),
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1.0/chats/19:me_barbara@unq.gbl.spaces"))
        .respond_with(ok(serde_json::json!({
            "id": "19:me_barbara@unq.gbl.spaces", "chatType": "oneOnOne",
            "members": [
                {"userId": "u-me", "displayName": "Alex Rivera", "email": "alex@contoso.com"},
                {"userId": "u-barbara", "displayName": "Barbara Liskov", "email": "barbara@contoso.com"}
            ]
        })))
        .mount(&server)
        .await;
    // The file: upload, read back webDavUrl, share with Barbara.
    Mock::given(method("PUT"))
        .and(path("/v1.0/me/drive/root:/Microsoft%20Teams%20Chat%20Files/plan.pdf:/content"))
        .and(query_param("@microsoft.graph.conflictBehavior", "rename"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "item-1", "name": "plan.pdf", "eTag": "\"{86EB4C8E-D20D-46B9-AD41-23B8868DDA8A},1\""
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1.0/me/drive/items/item-1"))
        .respond_with(ok(serde_json::json!({
            "id": "item-1", "name": "plan.pdf", "eTag": "\"{86EB4C8E-D20D-46B9-AD41-23B8868DDA8A},1\"",
            "webDavUrl": "https://contoso-my.sharepoint.com/personal/alex/Documents/Microsoft%20Teams%20Chat%20Files/plan.pdf"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1.0/me/drive/items/item-1/invite"))
        .and(body_json(serde_json::json!({
            "recipients": [{"email": "barbara@contoso.com"}],
            "roles": ["read"], "requireSignIn": true, "sendInvitation": false
        })))
        .respond_with(ok(serde_json::json!({"value": []})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1.0/chats/19:me_barbara@unq.gbl.spaces/messages"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "1800", "messageType": "message", "createdDateTime": "2026-10-11T12:00:00Z",
            "from": {"user": {"id": "u-me", "displayName": "Alex Rivera"}},
            "body": {"contentType": "html", "content": "<p><at id=\"0\">Barbara Liskov</at> the plan</p>"},
            "attachments": [{"id": "86eb4c8e-d20d-46b9-ad41-23b8868dda8a", "contentType": "reference", "name": "plan.pdf"}]
        })))
        .mount(&server)
        .await;
    for route in [
        "/v1.0/users/u-me/presence/setUserPreferredPresence",
        "/v1.0/users/u-me/presence/setStatusMessage",
        "/v1.0/users/u-me/presence/setPresence",
        "/v1.0/users/u-me/presence/clearPresence",
    ] {
        Mock::given(method("POST"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path("/v1.0/me/presence"))
        .respond_with(ok(serde_json::json!({
            "id": "u-me", "availability": "DoNotDisturb", "activity": "DoNotDisturb",
            "statusMessage": {"message": {"content": "Heads down", "contentType": "text"}}
        })))
        .mount(&server)
        .await;
    // Anything Wren fetches on its own after a send.
    Mock::given(method("GET"))
        .and(path("/v1.0/chats/19:me_barbara@unq.gbl.spaces/messages"))
        .respond_with(ok(serde_json::json!({"value": []})))
        .mount(&server)
        .await;

    // A meeting found by its join link, then its chat.
    Mock::given(method("GET"))
        .and(path("/v1.0/me/onlineMeetings"))
        .and(query_param("$filter", format!("JoinWebUrl eq '{JOIN}'")))
        .respond_with(ok(serde_json::json!({"value": [{
            "id": "meeting-1", "joinWebUrl": JOIN,
            "chatInfo": {"threadId": "19:meeting_abc@thread.v2", "messageId": "0"}
        }]})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1.0/chats/19:meeting_abc@thread.v2"))
        .respond_with(ok(serde_json::json!({
            "id": "19:meeting_abc@thread.v2", "chatType": "meeting", "topic": "Design review",
            "onlineMeetingInfo": {"joinWebUrl": JOIN},
            "members": [{"userId": "u-me", "displayName": "Alex Rivera", "email": "alex@contoso.com"}]
        })))
        .mount(&server)
        .await;

    // A signed-in account with the new permissions granted.
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
        granted: GRANTED.into(),
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

    // Find Barbara and start a chat.
    backend.send(Command::SearchPeople("barb".into()));
    let people = wait_for(&backend, 5, |e| match e {
        Event::People { people, .. } => Some(people),
        _ => None,
    });
    assert_eq!(people[0].display_name, "Barbara Liskov");
    backend.send(Command::CreateChat {
        others: vec![people[0].user_id.clone()],
        topic: None,
    });
    let chat = wait_for(&backend, 5, |e| match e {
        Event::ChatCreated(chat) => Some(*chat),
        _ => None,
    });
    assert_eq!(chat.title("u-me"), "Barbara Liskov");
    let created = &requests_to(&server, "/v1.0/chats").await[0];
    let body: serde_json::Value = serde_json::from_slice(&created.body).unwrap();
    assert_eq!(body["chatType"], "oneOnOne");
    assert!(
        body.get("topic").is_none(),
        "one-on-one chats have no topic"
    );
    assert_eq!(
        body["members"][0]["user@odata.bind"],
        "https://graph.microsoft.com/v1.0/users('u-me')"
    );
    assert_eq!(
        body["members"][1]["user@odata.bind"],
        "https://graph.microsoft.com/v1.0/users('u-barbara')"
    );

    // Mention her and send a file.
    let file = dir.path().join("plan.pdf");
    std::fs::write(&file, b"%PDF-1.7 plan").unwrap();
    let (html, mentions) = wren::html::compose_with_mentions(
        "@Barbara Liskov the plan",
        &[MentionRef {
            user_id: "u-barbara".into(),
            name: "Barbara Liskov".into(),
        }],
    );
    backend.send(Command::Send {
        conversation: ConversationId::Chat(chat.id.clone()),
        local_id: "local-1".into(),
        html,
        reply_to: None,
        mentions,
        files: vec![file],
        images: Vec::new(),
        recipients: vec!["barbara@contoso.com".into()],
    });
    let sent = wait_for(&backend, 10, |e| match e {
        Event::Sent { message, .. } => Some(*message),
        Event::SendFailed { reason, .. } => panic!("send failed: {reason}"),
        _ => None,
    });
    assert_eq!(sent.id, "1800");
    let posted = &requests_to(&server, "/v1.0/chats/19:me_barbara@unq.gbl.spaces/messages")
        .await
        .into_iter()
        .find(|r| r.method.as_str() == "POST")
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&posted.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "body": {"contentType": "html", "content": "<p><at id=\"0\">Barbara Liskov</at> the plan</p><attachment id=\"86eb4c8e-d20d-46b9-ad41-23b8868dda8a\"></attachment>"},
            "mentions": [{"id": 0, "mentionText": "Barbara Liskov", "mentioned": {"user": {"id": "u-barbara", "displayName": "Barbara Liskov", "userIdentityType": "aadUser"}}}],
            "attachments": [{"id": "86eb4c8e-d20d-46b9-ad41-23b8868dda8a", "contentType": "reference",
                "contentUrl": "https://contoso-my.sharepoint.com/personal/alex/Documents/Microsoft%20Teams%20Chat%20Files/plan.pdf",
                "name": "plan.pdf"}]
        })
    );

    // Status: availability, then a message; each reads presence back.
    backend.send(Command::SetAvailability(Some(Availability::DoNotDisturb)));
    let presence = wait_for(&backend, 5, |e| match e {
        Event::MyPresence(presence) => Some(presence),
        _ => None,
    });
    assert_eq!(presence.availability, Availability::DoNotDisturb);
    assert_eq!(presence.status_message.as_deref(), Some("Heads down"));
    let preferred = &requests_to(
        &server,
        "/v1.0/users/u-me/presence/setUserPreferredPresence",
    )
    .await[0];
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&preferred.body).unwrap(),
        serde_json::json!({"availability": "DoNotDisturb", "activity": "DoNotDisturb"})
    );
    backend.send(Command::SetStatusMessage("Heads down".into()));
    wait_for(&backend, 5, |e| {
        matches!(e, Event::MyPresence(_)).then_some(())
    });

    // Joining a meeting whose chat Wren has not seen: look it up.
    backend.send(Command::FindMeetingChat(JOIN.into()));
    let (join_url, chat) = wait_for(&backend, 5, |e| match e {
        Event::MeetingChat { join_url, chat } => Some((join_url, chat)),
        _ => None,
    });
    assert_eq!(join_url, JOIN);
    let chat = chat.expect("the meeting's chat");
    assert_eq!(chat.id, "19:meeting_abc@thread.v2");
    assert_eq!(chat.kind, wren::model::ChatKind::Meeting);
    assert!(wren::calling::same_meeting(
        chat.join_url.as_deref().unwrap(),
        JOIN
    ));

    // Using Wren keeps a presence session; quitting ends it.
    backend.send(Command::PresenceActive);
    backend.send(Command::PresenceActive);
    let deadline = Instant::now() + Duration::from_secs(5);
    while requests_to(&server, "/v1.0/users/u-me/presence/setPresence")
        .await
        .is_empty()
    {
        assert!(Instant::now() < deadline, "no presence session was set");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let sessions = requests_to(&server, "/v1.0/users/u-me/presence/setPresence").await;
    assert_eq!(sessions.len(), 1, "refreshes are at most every few minutes");
    let session: serde_json::Value = serde_json::from_slice(&sessions[0].body).unwrap();
    assert_eq!(
        session["sessionId"], CLIENT,
        "the session id is the app's client id"
    );
    assert_eq!(session["availability"], "Available");
    drop(backend);
    assert_eq!(
        requests_to(&server, "/v1.0/users/u-me/presence/clearPresence")
            .await
            .len(),
        1
    );
}
