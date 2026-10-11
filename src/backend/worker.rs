//! The background side: sign-in, fetches, sends, and the cache.
//!
//! The loop reads commands one at a time and decides; anything that waits
//! on the network or the disk runs as its own task, so a slow request never
//! holds up another. Tasks report back through `Event`s to the interface,
//! and through `Internal` commands to this loop when its own state changes
//! (a finished sign-in, a failed fetch to forget).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::{Semaphore, mpsc, watch};

use super::{AuthStatus, Command, Event, Merge, Waker};
use crate::auth::session::{AccountRecord, Session};
use crate::auth::token::{self, TokenError};
use crate::auth::{self, AuthConfig, CHANNEL_READ_SCOPE, IdClaims, loopback::Loopback};
use crate::graph::{Graph, GraphError};
use crate::model::{Access, ConversationId, Message};
use crate::paths::AppDirs;
use crate::realtime::{self, Notice, PushStatus};
use crate::secrets::SecretStore;
use crate::store::Store;
use crate::sync::{Freshness, Resource, Trigger};

/// How many messages of a conversation the cache hands over at once.
const CACHED_MESSAGES: usize = 200;
/// Photos are fetched again after a week.
const PHOTO_MAX_AGE: i64 = 7 * 24 * 60 * 60 * 1000;
/// Presence of one person is not asked for again within this.
const PRESENCE_REUSE: Duration = Duration::from_secs(60);

/// Sends events and wakes the window for each.
#[derive(Clone)]
struct Outbox {
    events: std::sync::mpsc::Sender<Event>,
    waker: Waker,
}

impl Outbox {
    fn send(&self, event: Event) {
        let _ = self.events.send(event);
        self.waker.wake();
    }

    fn error(&self, context: &str, error: &GraphError) {
        // Throttling and lost connections recover on their own; say so
        // once, briefly.
        let text = match error {
            GraphError::Network(reason) => format!("{context}: offline ({reason})"),
            GraphError::Throttled { .. } => {
                format!("{context}: Microsoft is limiting requests, try again shortly")
            }
            other => format!("{context}: {other}"),
        };
        log::warn!("{text}");
        self.send(Event::Error(text));
    }
}

/// A signed-in account's live pieces.
struct Account {
    record: AccountRecord,
    graph: Arc<Graph>,
    store: Arc<Mutex<Store>>,
}

enum Internal {
    SignedIn(Box<(Session, IdClaims)>),
    SignInFailed(String),
    FetchFailed(Resource),
    /// Confirms the person's own change with a date-ranged refresh.
    Refetch(ConversationId),
    /// Graph said a chat changed.
    Pushed(String),
    /// Graph lost notifications: refresh the chat list.
    PushMissed,
}

struct Worker {
    dirs: AppDirs,
    http: reqwest::Client,
    secrets: Arc<dyn SecretStore>,
    out: Outbox,
    internal: mpsc::UnboundedSender<Internal>,
    config: Option<AuthConfig>,
    account: Option<Account>,
    cancel_sign_in: Option<watch::Sender<bool>>,
    freshness: Freshness,
    presence_seen: HashMap<String, Instant>,
    photos_asked: HashMap<String, Instant>,
    /// Channel paging links, by conversation.
    older_links: Arc<Mutex<HashMap<ConversationId, String>>>,
    downloads: Arc<Semaphore>,
    options: super::BackendOptions,
    relay: Option<String>,
    stop_push: Option<watch::Sender<bool>>,
}

pub(super) async fn run(
    dirs: AppDirs,
    options: super::BackendOptions,
    events: std::sync::mpsc::Sender<Event>,
    mut commands: mpsc::UnboundedReceiver<Command>,
    waker: Waker,
) {
    let (internal_tx, mut internal_rx) = mpsc::unbounded_channel();
    let http = reqwest::Client::builder()
        .user_agent(concat!("Wren/", env!("CARGO_PKG_VERSION")))
        .gzip(true)
        .connect_timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .unwrap_or_default();
    let mut worker = Worker {
        dirs,
        http,
        secrets: Arc::clone(&options.secrets),
        options,
        relay: None,
        stop_push: None,
        out: Outbox { events, waker },
        internal: internal_tx,
        config: None,
        account: None,
        cancel_sign_in: None,
        freshness: Freshness::default(),
        presence_seen: HashMap::new(),
        photos_asked: HashMap::new(),
        older_links: Arc::default(),
        downloads: Arc::new(Semaphore::new(4)),
    };
    loop {
        tokio::select! {
            command = commands.recv() => match command {
                None | Some(Command::Shutdown) => break,
                Some(command) => worker.handle(command).await,
            },
            Some(internal) = internal_rx.recv() => worker.internal(internal).await,
        }
    }
    if let Some(cancel) = worker.cancel_sign_in.take() {
        let _ = cancel.send(true);
    }
    worker.stop_push().await;
}

impl Worker {
    async fn handle(&mut self, command: Command) {
        match command {
            Command::Start {
                client_id,
                tenant,
                account,
            } => self.start(client_id, tenant, account).await,
            Command::SignIn {
                extra_scopes,
                device_code,
            } => self.sign_in(extra_scopes, device_code),
            Command::CancelSignIn => {
                if let Some(cancel) = self.cancel_sign_in.take() {
                    let _ = cancel.send(true);
                }
            }
            Command::SignOut => self.sign_out().await,
            Command::Fetch { resource, trigger } => self.fetch(resource, trigger),
            Command::LoadOlder(conversation) => self.load_older(conversation),
            Command::LoadReplies { conversation, root } => self.load_replies(conversation, root),
            Command::Send {
                conversation,
                local_id,
                html,
                reply_to,
            } => self.send(conversation, local_id, html, reply_to),
            Command::Edit {
                conversation,
                message,
                html,
            } => self.edit(conversation, message, html),
            Command::Delete {
                conversation,
                message,
            } => self.delete(conversation, message),
            Command::React {
                conversation,
                message,
                emoji,
                on,
            } => self.react(conversation, message, emoji, on),
            Command::MarkRead(chat) => self.mark_read(chat),
            Command::Relay(relay) => {
                self.relay = relay.as_deref().and_then(realtime::normalize_relay);
                if relay.is_some() && self.relay.is_none() {
                    self.out.send(Event::Push(PushStatus::Failed(
                        "the relay must be an https:// address".into(),
                    )));
                }
                self.restart_push().await;
            }
            Command::Presence(users) => self.presence(users),
            Command::Photo(user) => self.photo(user),
            Command::Media(url) => self.media(url),
            Command::Search(query) => self.search(query),
            Command::Shutdown => {}
        }
    }

    async fn internal(&mut self, internal: Internal) {
        match internal {
            Internal::SignedIn(boxed) => {
                let (session, claims) = *boxed;
                self.cancel_sign_in = None;
                self.install(session, claims).await;
            }
            Internal::SignInFailed(reason) => {
                self.cancel_sign_in = None;
                self.out.send(Event::Auth(AuthStatus::Failed(reason)));
            }
            Internal::FetchFailed(resource) => self.freshness.forget(&resource),
            Internal::Refetch(conversation) => {
                self.fetch(Resource::Messages(conversation), Trigger::AfterAction)
            }
            Internal::Pushed(chat) => {
                self.fetch(
                    Resource::Messages(ConversationId::Chat(chat)),
                    Trigger::Push,
                );
                self.fetch(Resource::Chats, Trigger::Push);
            }
            Internal::PushMissed => self.fetch(Resource::Chats, Trigger::Push),
        }
    }

    // -- Accounts ---------------------------------------------------------

    async fn start(&mut self, client_id: String, tenant: String, account: Option<String>) {
        if !crate::settings::is_guid(client_id.trim()) {
            self.config = None;
            self.out.send(Event::Auth(AuthStatus::NotConfigured));
            return;
        }
        let config = AuthConfig {
            authority: self.options.authority.clone(),
            ..AuthConfig::new(&client_id, &tenant)
        };
        let changed = self.config.as_ref() != Some(&config);
        self.config = Some(config.clone());
        if !changed && self.account.is_some() {
            return;
        }
        self.account = None;
        let Some(id) = account else {
            self.out.send(Event::Auth(AuthStatus::SignedOut));
            return;
        };
        let record_path = self.dirs.account(&id).join("account.json");
        let Some(record) = AccountRecord::load(&record_path) else {
            self.out.send(Event::Auth(AuthStatus::SignedOut));
            return;
        };
        self.out.send(Event::Auth(AuthStatus::Restoring));
        // The cache first, so the window has something to show at once.
        let store = match Store::open(&self.dirs.database(&id)) {
            Ok(store) => Arc::new(Mutex::new(store)),
            Err(error) => {
                log::warn!("could not open the cache: {error}");
                match Store::in_memory() {
                    Ok(store) => Arc::new(Mutex::new(store)),
                    Err(_) => return,
                }
            }
        };
        emit_cache(&store, &self.out).await;
        match Session::restore(
            &record,
            config,
            self.http.clone(),
            Arc::clone(&self.secrets),
        )
        .await
        {
            Ok(Some(session)) => {
                let session = Arc::new(session);
                self.out.send(Event::Granted(session.granted()));
                self.out
                    .send(Event::ChannelAccess(channel_access(&session)));
                self.account = Some(Account {
                    graph: Arc::new(Graph::with_base(
                        self.http.clone(),
                        session,
                        &self.options.graph_base,
                    )),
                    record: record.clone(),
                    store,
                });
                self.out.send(Event::Auth(AuthStatus::SignedIn {
                    account: record.id.clone(),
                    name: record.display_name.clone(),
                    username: record.username.clone(),
                }));
                self.restart_push().await;
            }
            Ok(None) => self.out.send(Event::Auth(AuthStatus::Expired {
                username: record.username.clone(),
                reason: "Sign in again to continue.".into(),
            })),
            Err(error) => self.out.send(Event::Auth(AuthStatus::Expired {
                username: record.username.clone(),
                reason: format!("Wren could not read its saved session: {error}."),
            })),
        }
    }

    fn sign_in(&mut self, extra_scopes: Vec<String>, device_code: bool) {
        let Some(config) = self.config.clone() else {
            self.out.send(Event::Auth(AuthStatus::NotConfigured));
            return;
        };
        if let Some(previous) = self.cancel_sign_in.take() {
            let _ = previous.send(true);
        }
        let (cancel, cancelled) = watch::channel(false);
        self.cancel_sign_in = Some(cancel);
        // Keep the scopes already granted when asking for more.
        let mut extra = extra_scopes;
        if let Some(account) = &self.account {
            for scope in auth::OPTIONAL_SCOPES {
                if account.graph.session().has_scope(scope) && !extra.iter().any(|s| s == scope) {
                    extra.push((*scope).to_owned());
                }
            }
        }
        let hint = self.account.as_ref().map(|a| a.record.username.clone());
        let http = self.http.clone();
        let secrets = Arc::clone(&self.secrets);
        let out = self.out.clone();
        let internal = self.internal.clone();
        let open_browser = self.options.open_browser;
        tokio::spawn(async move {
            let result = if device_code {
                device_code_sign_in(&http, &config, &extra, &out, cancelled).await
            } else {
                browser_sign_in(
                    &http,
                    &config,
                    &extra,
                    hint.as_deref(),
                    &out,
                    cancelled,
                    open_browser,
                )
                .await
            };
            let outcome = match result {
                Ok(response) => Session::from_sign_in(config, http, secrets, response)
                    .await
                    .map(|pair| Internal::SignedIn(Box::new(pair)))
                    .unwrap_or_else(|error| Internal::SignInFailed(error.to_string())),
                Err(reason) => Internal::SignInFailed(reason),
            };
            let _ = internal.send(outcome);
        });
    }

    async fn install(&mut self, session: Session, claims: IdClaims) {
        let id = session.account.clone();
        let account_dir = self.dirs.account(&id);
        let _ = crate::paths::create_private_dir(&account_dir);
        let record = AccountRecord {
            id: id.clone(),
            display_name: claims.name.clone().unwrap_or_default(),
            username: claims.preferred_username.clone().unwrap_or_default(),
            tenant_id: claims.tid.clone().unwrap_or_default(),
            granted: session.granted(),
            client_id: session.config.client_id.clone(),
            tenant: session.config.tenant.clone(),
        };
        if let Err(error) = record.save(&account_dir.join("account.json")) {
            log::warn!("could not save the account record: {error}");
        }
        let store = match self.account.take() {
            // Re-consent for more scopes keeps the same cache.
            Some(previous) if previous.record.id == id => previous.store,
            _ => match Store::open(&self.dirs.database(&id)).or_else(|_| Store::in_memory()) {
                Ok(store) => Arc::new(Mutex::new(store)),
                Err(error) => {
                    self.out.send(Event::Auth(AuthStatus::Failed(format!(
                        "Could not open the cache: {error}"
                    ))));
                    return;
                }
            },
        };
        let session = Arc::new(session);
        self.out.send(Event::Granted(session.granted()));
        self.out
            .send(Event::ChannelAccess(channel_access(&session)));
        self.account = Some(Account {
            graph: Arc::new(Graph::with_base(
                self.http.clone(),
                session,
                &self.options.graph_base,
            )),
            record: record.clone(),
            store: Arc::clone(&store),
        });
        self.freshness.clear();
        emit_cache(&store, &self.out).await;
        self.out.send(Event::Auth(AuthStatus::SignedIn {
            account: id,
            name: record.display_name,
            username: record.username,
        }));
        self.restart_push().await;
    }

    // -- Push (experimental) ------------------------------------------------

    /// Stops any running push loop and, when a relay is set and an account is
    /// signed in, starts a new one.
    async fn restart_push(&mut self) {
        self.stop_push().await;
        let (Some(relay), Some(account)) = (self.relay.clone(), &self.account) else {
            self.out.send(Event::Push(PushStatus::Off));
            return;
        };
        let (stop, stopped) = watch::channel(false);
        self.stop_push = Some(stop);
        let graph = Arc::clone(&account.graph);
        let me = account
            .record
            .id
            .split('.')
            .next()
            .unwrap_or_default()
            .to_owned();
        let out = self.out.clone();
        let internal = self.internal.clone();
        tokio::spawn(push_loop(graph, me, relay, out, internal, stopped));
    }

    async fn stop_push(&mut self) {
        if let Some(stop) = self.stop_push.take() {
            let _ = stop.send(true);
            // Give the loop a moment to delete its subscription.
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn sign_out(&mut self) {
        self.stop_push().await;
        let Some(account) = self.account.take() else {
            self.out.send(Event::Auth(AuthStatus::SignedOut));
            return;
        };
        account.graph.session().sign_out().await;
        drop(account.store);
        // The cache holds the person's messages: remove it with the session.
        let dir = self.dirs.account(&account.record.id);
        if let Err(error) = tokio::fs::remove_dir_all(&dir).await {
            log::warn!("could not remove the account's cache: {error}");
        }
        self.freshness.clear();
        self.out.send(Event::Auth(AuthStatus::SignedOut));
    }

    // -- Fetching -----------------------------------------------------------

    fn fetch(&mut self, resource: Resource, trigger: Trigger) {
        let Some(account) = &self.account else { return };
        if !self
            .freshness
            .claim(resource.clone(), trigger, Instant::now())
        {
            return;
        }
        let graph = Arc::clone(&account.graph);
        let store = Arc::clone(&account.store);
        let out = self.out.clone();
        let internal = self.internal.clone();
        let tenant = account.record.tenant_id.clone();
        let links = Arc::clone(&self.older_links);
        tokio::spawn(async move {
            out.send(Event::Loading(resource.clone(), true));
            let result = fetch_resource(&graph, &store, &out, &resource, tenant, &links).await;
            out.send(Event::Loading(resource.clone(), false));
            if let Err(error) = result {
                let _ = internal.send(Internal::FetchFailed(resource.clone()));
                match (&resource, &error) {
                    (
                        Resource::Messages(ConversationId::Channel { .. }),
                        GraphError::Forbidden { .. },
                    ) => {
                        out.send(Event::ChannelAccess(Access::Forbidden));
                    }
                    (Resource::Presence | Resource::Calendar, GraphError::Forbidden { .. }) => {
                        log::info!("{resource:?} is not permitted for this account");
                    }
                    _ => out.error(describe(&resource), &error),
                }
                if let GraphError::Auth(TokenError::Rejected(reason)) = &error {
                    out.send(Event::Auth(AuthStatus::Expired {
                        username: String::new(),
                        reason: reason.clone(),
                    }));
                }
            }
        });
    }

    fn load_older(&mut self, conversation: ConversationId) {
        let Some(account) = &self.account else { return };
        let graph = Arc::clone(&account.graph);
        let store = Arc::clone(&account.store);
        let out = self.out.clone();
        let links = Arc::clone(&self.older_links);
        tokio::spawn(async move {
            let resource = Resource::Messages(conversation.clone());
            out.send(Event::Loading(resource.clone(), true));
            let result = async {
                let page = match &conversation {
                    ConversationId::Chat(chat) => {
                        let oldest = {
                            let store = store.lock().unwrap_or_else(|p| p.into_inner());
                            oldest_cached(&store, &conversation)
                        };
                        graph.chat_messages(chat, oldest).await?
                    }
                    ConversationId::Channel { team, channel } => {
                        let link = links
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .get(&conversation)
                            .cloned();
                        let Some(link) = link else {
                            return Ok(Some(false));
                        };
                        let page = graph.channel_messages(team, channel, Some(&link)).await?;
                        remember_link(&links, &conversation, page.older.clone());
                        page
                    }
                };
                let more = page.older.is_some() || page.messages.len() >= crate::graph::api::PAGE;
                save(&store, &conversation, &page.messages).await;
                out.send(Event::Messages {
                    conversation: conversation.clone(),
                    messages: page.messages,
                    merge: Merge::Older,
                    more: Some(more),
                    cached: false,
                });
                Ok::<_, GraphError>(None)
            }
            .await;
            out.send(Event::Loading(resource, false));
            match result {
                Ok(Some(more)) => out.send(Event::Messages {
                    conversation,
                    messages: Vec::new(),
                    merge: Merge::Older,
                    more: Some(more),
                    cached: false,
                }),
                Ok(None) => {}
                Err(error) => out.error("Loading older messages", &error),
            }
        });
    }

    fn load_replies(&mut self, conversation: ConversationId, root: String) {
        let Some(account) = &self.account else { return };
        let ConversationId::Channel { team, channel } = &conversation else {
            return;
        };
        let (team, channel) = (team.clone(), channel.clone());
        let graph = Arc::clone(&account.graph);
        let out = self.out.clone();
        tokio::spawn(async move {
            match graph.channel_replies(&team, &channel, &root).await {
                Ok(replies) => out.send(Event::Replies {
                    conversation,
                    root,
                    replies,
                }),
                Err(error) => out.error("Loading replies", &error),
            }
        });
    }

    // -- Writing ------------------------------------------------------------

    fn send(
        &mut self,
        conversation: ConversationId,
        local_id: String,
        html: String,
        reply_to: Option<String>,
    ) {
        let Some(account) = &self.account else {
            self.out.send(Event::SendFailed {
                conversation,
                local_id,
                reason: "Not signed in".into(),
            });
            return;
        };
        let graph = Arc::clone(&account.graph);
        let store = Arc::clone(&account.store);
        let out = self.out.clone();
        tokio::spawn(async move {
            let result = match &conversation {
                ConversationId::Chat(chat) => match &reply_to {
                    Some(quoted) => graph.reply_with_quote(chat, quoted, &html).await,
                    None => graph.send_chat_message(chat, &html).await,
                },
                ConversationId::Channel { team, channel } => {
                    graph
                        .send_channel_message(team, channel, reply_to.as_deref(), &html)
                        .await
                }
            };
            match result {
                Ok(message) => {
                    if message.reply_to.is_none() {
                        save(&store, &conversation, std::slice::from_ref(&message)).await;
                    }
                    out.send(Event::Sent {
                        conversation,
                        local_id,
                        message: Box::new(message),
                    });
                }
                Err(error) => {
                    log::warn!("send failed: {error}");
                    out.send(Event::SendFailed {
                        conversation,
                        local_id,
                        reason: error.to_string(),
                    });
                }
            }
        });
    }

    fn edit(&mut self, conversation: ConversationId, message: String, html: String) {
        let Some(account) = &self.account else { return };
        let graph = Arc::clone(&account.graph);
        let out = self.out.clone();
        let internal = self.internal.clone();
        tokio::spawn(async move {
            match graph.edit_message(&conversation, &message, &html).await {
                Ok(()) => {
                    let _ = internal.send(Internal::Refetch(conversation));
                }
                Err(error) => out.error("Editing the message", &error),
            }
        });
    }

    fn delete(&mut self, conversation: ConversationId, message: String) {
        let Some(account) = &self.account else { return };
        let graph = Arc::clone(&account.graph);
        let store = Arc::clone(&account.store);
        let out = self.out.clone();
        let me = account
            .record
            .id
            .split('.')
            .next()
            .unwrap_or_default()
            .to_owned();
        tokio::spawn(async move {
            match graph.delete_message(&me, &conversation, &message).await {
                Ok(()) => {
                    {
                        let store = store.lock().unwrap_or_else(|p| p.into_inner());
                        let _ = store.delete_message(&conversation, &message);
                    }
                    out.send(Event::Deleted {
                        conversation,
                        message,
                    });
                }
                Err(error) => out.error("Deleting the message", &error),
            }
        });
    }

    fn react(&mut self, conversation: ConversationId, message: String, emoji: String, on: bool) {
        let Some(account) = &self.account else { return };
        let graph = Arc::clone(&account.graph);
        let out = self.out.clone();
        let internal = self.internal.clone();
        tokio::spawn(async move {
            match graph.react(&conversation, &message, &emoji, on).await {
                Ok(()) => {
                    let _ = internal.send(Internal::Refetch(conversation));
                }
                Err(error) => out.error("Reacting", &error),
            }
        });
    }

    fn mark_read(&mut self, chat: String) {
        let Some(account) = &self.account else { return };
        let graph = Arc::clone(&account.graph);
        let me = account
            .record
            .id
            .split('.')
            .next()
            .unwrap_or_default()
            .to_owned();
        let tenant = account.record.tenant_id.clone();
        tokio::spawn(async move {
            if let Err(error) = graph.mark_chat_read(&chat, &me, &tenant).await {
                log::info!("could not mark a chat read: {error}");
            }
        });
    }

    // -- People and media ---------------------------------------------------

    fn presence(&mut self, users: Vec<String>) {
        let Some(account) = &self.account else { return };
        if !account.graph.session().has_scope("Presence.Read.All") {
            return;
        }
        let now = Instant::now();
        let due: Vec<String> = users
            .into_iter()
            .filter(|user| {
                self.presence_seen
                    .get(user)
                    .is_none_or(|at| now.duration_since(*at) >= PRESENCE_REUSE)
            })
            .collect();
        if due.is_empty() {
            return;
        }
        for user in &due {
            self.presence_seen.insert(user.clone(), now);
        }
        let graph = Arc::clone(&account.graph);
        let out = self.out.clone();
        tokio::spawn(async move {
            match graph.presences(&due).await {
                Ok(presence) => out.send(Event::Presence(presence)),
                Err(error) => log::info!("presence unavailable: {error}"),
            }
        });
    }

    fn photo(&mut self, user: String) {
        let Some(account) = &self.account else { return };
        let now = Instant::now();
        if self
            .photos_asked
            .get(&user)
            .is_some_and(|at| now.duration_since(*at) < Duration::from_secs(600))
        {
            return;
        }
        self.photos_asked.insert(user.clone(), now);
        let graph = Arc::clone(&account.graph);
        let store = Arc::clone(&account.store);
        let out = self.out.clone();
        let downloads = Arc::clone(&self.downloads);
        tokio::spawn(async move {
            let cached = {
                let store = store.lock().unwrap_or_else(|p| p.into_inner());
                store.photo(&user, PHOTO_MAX_AGE)
            };
            if let Some(bytes) = cached {
                out.send(Event::Photo { user, bytes });
                return;
            }
            let Ok(_permit) = downloads.acquire().await else {
                return;
            };
            match graph.photo(&user).await {
                Ok(bytes) => {
                    {
                        let store = store.lock().unwrap_or_else(|p| p.into_inner());
                        let _ = store.save_photo(&user, bytes.as_deref());
                    }
                    out.send(Event::Photo { user, bytes });
                }
                Err(error) => log::debug!("photo unavailable: {error}"),
            }
        });
    }

    fn media(&mut self, url: String) {
        let Some(account) = &self.account else { return };
        if !account.graph.is_graph_url(&url) {
            // Images hosted elsewhere are not fetched with credentials, and
            // not fetched at all: they could track who read the message.
            self.out.send(Event::Media { url, bytes: None });
            return;
        }
        let graph = Arc::clone(&account.graph);
        let out = self.out.clone();
        let downloads = Arc::clone(&self.downloads);
        tokio::spawn(async move {
            let Ok(_permit) = downloads.acquire().await else {
                return;
            };
            let bytes = graph
                .get_bytes(&url, crate::graph::api::MEDIA_LIMIT)
                .await
                .ok();
            out.send(Event::Media { url, bytes });
        });
    }

    fn search(&mut self, query: String) {
        let Some(account) = &self.account else { return };
        let store = Arc::clone(&account.store);
        let graph = Arc::clone(&account.graph);
        let out = self.out.clone();
        tokio::spawn(async move {
            let local = {
                let store = store.lock().unwrap_or_else(|p| p.into_inner());
                store.search(&query, 50).unwrap_or_default()
            };
            out.send(Event::SearchResults {
                query: query.clone(),
                hits: local,
                remote: false,
            });
            if query.trim().chars().count() < 3 {
                return;
            }
            match graph.search_messages(&query).await {
                Ok(hits) => out.send(Event::SearchResults {
                    query,
                    hits,
                    remote: true,
                }),
                Err(error) => log::info!("Microsoft Search unavailable: {error}"),
            }
        });
    }
}

fn channel_access(session: &Session) -> Access {
    if session.has_scope(CHANNEL_READ_SCOPE) {
        Access::Granted
    } else {
        Access::NeedsConsent
    }
}

fn describe(resource: &Resource) -> &'static str {
    match resource {
        Resource::Profile => "Loading your profile",
        Resource::Chats => "Loading chats",
        Resource::Messages(_) => "Loading messages",
        Resource::Teams => "Loading teams",
        Resource::Channels(_) => "Loading channels",
        Resource::Calendar => "Loading the calendar",
        Resource::Presence => "Loading presence",
    }
}

async fn emit_cache(store: &Arc<Mutex<Store>>, out: &Outbox) {
    let store = Arc::clone(store);
    let cached = tokio::task::spawn_blocking(move || {
        let store = store.lock().unwrap_or_else(|p| p.into_inner());
        (
            store.chats().unwrap_or_default(),
            store.teams().unwrap_or_default(),
            store.channels().unwrap_or_default(),
            store.meetings().unwrap_or_default(),
        )
    })
    .await;
    let Ok((chats, teams, channels, meetings)) = cached else {
        return;
    };
    out.send(Event::Chats {
        chats,
        cached: true,
    });
    out.send(Event::Teams {
        teams,
        cached: true,
    });
    out.send(Event::Channels {
        team: None,
        channels,
    });
    out.send(Event::Meetings {
        meetings,
        cached: true,
    });
}

async fn save(store: &Arc<Mutex<Store>>, conversation: &ConversationId, messages: &[Message]) {
    if messages.is_empty() {
        return;
    }
    let store = Arc::clone(store);
    let conversation = conversation.clone();
    let messages = messages.to_vec();
    let _ = tokio::task::spawn_blocking(move || {
        let mut store = store.lock().unwrap_or_else(|p| p.into_inner());
        if let Err(error) = store.save_messages(&conversation, &messages) {
            log::warn!("could not cache messages: {error}");
        }
    })
    .await;
}

fn oldest_cached(store: &Store, conversation: &ConversationId) -> Option<i64> {
    store.oldest(conversation)
}

fn remember_link(
    links: &Arc<Mutex<HashMap<ConversationId, String>>>,
    conversation: &ConversationId,
    link: Option<String>,
) {
    let mut links = links.lock().unwrap_or_else(|p| p.into_inner());
    match link {
        Some(link) => {
            links.insert(conversation.clone(), link);
        }
        None => {
            links.remove(conversation);
        }
    }
}

async fn fetch_resource(
    graph: &Arc<Graph>,
    store: &Arc<Mutex<Store>>,
    out: &Outbox,
    resource: &Resource,
    tenant: String,
    links: &Arc<Mutex<HashMap<ConversationId, String>>>,
) -> Result<(), GraphError> {
    match resource {
        Resource::Profile => {
            let profile = graph.me(Some(tenant)).await?;
            out.send(Event::Profile(Box::new(profile)));
        }
        Resource::Chats => {
            let chats = graph.chats(2).await?;
            {
                let mut store = store.lock().unwrap_or_else(|p| p.into_inner());
                if let Err(error) = store.save_chats(&chats) {
                    log::warn!("could not cache chats: {error}");
                }
            }
            out.send(Event::Chats {
                chats,
                cached: false,
            });
        }
        Resource::Messages(conversation) => {
            // What the cache has, at once.
            let (cached, since) = {
                let store = store.lock().unwrap_or_else(|p| p.into_inner());
                (
                    store
                        .messages(conversation, CACHED_MESSAGES)
                        .unwrap_or_default(),
                    store.last_modified(conversation),
                )
            };
            let had_cache = !cached.is_empty();
            if had_cache {
                out.send(Event::Messages {
                    conversation: conversation.clone(),
                    messages: cached,
                    merge: Merge::Latest,
                    more: None,
                    cached: true,
                });
            }
            match conversation {
                ConversationId::Chat(chat) => {
                    // A date-ranged refresh when the cache knows where it
                    // stopped; the newest page otherwise.
                    if let Some(since) = since.filter(|_| had_cache) {
                        let changes = graph.chat_changes(chat, since - 1_000).await?;
                        if changes.len() < crate::graph::api::PAGE * 4 {
                            save(store, conversation, &changes).await;
                            out.send(Event::Messages {
                                conversation: conversation.clone(),
                                messages: changes,
                                merge: Merge::Changes,
                                more: None,
                                cached: false,
                            });
                            return Ok(());
                        }
                    }
                    let page = graph.chat_messages(chat, None).await?;
                    let more = page.older.is_some();
                    save(store, conversation, &page.messages).await;
                    out.send(Event::Messages {
                        conversation: conversation.clone(),
                        messages: page.messages,
                        merge: Merge::Latest,
                        more: Some(more),
                        cached: false,
                    });
                }
                ConversationId::Channel { team, channel } => {
                    let page = graph.channel_messages(team, channel, None).await?;
                    out.send(Event::ChannelAccess(Access::Granted));
                    remember_link(links, conversation, page.older.clone());
                    save(store, conversation, &page.messages).await;
                    out.send(Event::Messages {
                        conversation: conversation.clone(),
                        messages: page.messages,
                        merge: Merge::Latest,
                        more: Some(page.older.is_some()),
                        cached: false,
                    });
                }
            }
        }
        Resource::Teams => {
            let teams = graph.joined_teams().await?;
            {
                let mut store = store.lock().unwrap_or_else(|p| p.into_inner());
                let _ = store.save_teams(&teams);
            }
            out.send(Event::Teams {
                teams: teams.clone(),
                cached: false,
            });
            // Channels of every team, a few at a time.
            let limit = Arc::new(Semaphore::new(3));
            let mut tasks = Vec::new();
            for team in teams {
                let graph = Arc::clone(graph);
                let store = Arc::clone(store);
                let out = out.clone();
                let limit = Arc::clone(&limit);
                tasks.push(tokio::spawn(async move {
                    let Ok(_permit) = limit.acquire().await else {
                        return;
                    };
                    match graph.channels(&team.id).await {
                        Ok(channels) => {
                            {
                                let mut store = store.lock().unwrap_or_else(|p| p.into_inner());
                                let _ = store.save_channels(&team.id, &channels);
                            }
                            out.send(Event::Channels {
                                team: Some(team.id),
                                channels,
                            });
                        }
                        Err(error) => log::info!("channels unavailable for a team: {error}"),
                    }
                }));
            }
            for task in tasks {
                let _ = task.await;
            }
        }
        Resource::Channels(team) => {
            let channels = graph.channels(team).await?;
            {
                let mut store = store.lock().unwrap_or_else(|p| p.into_inner());
                let _ = store.save_channels(team, &channels);
            }
            out.send(Event::Channels {
                team: Some(team.clone()),
                channels,
            });
        }
        Resource::Calendar => {
            let from = crate::time::start_of_day(crate::time::now());
            let to = from + 14 * 24 * 60 * 60 * 1000;
            let meetings = graph.calendar(from, to).await?;
            {
                let mut store = store.lock().unwrap_or_else(|p| p.into_inner());
                let _ = store.save_meetings(&meetings);
            }
            out.send(Event::Meetings {
                meetings,
                cached: false,
            });
        }
        Resource::Presence => {}
    }
    Ok(())
}

/// Keeps one subscription alive while `stop` stays false: connect to the
/// relay, subscribe, forward notices, renew, and start over after failures
/// with growing pauses.
async fn push_loop(
    graph: Arc<Graph>,
    me: String,
    relay: String,
    out: Outbox,
    internal: mpsc::UnboundedSender<Internal>,
    mut stop: watch::Receiver<bool>,
) {
    use futures_util::{SinkExt, StreamExt};
    let mut pause = Duration::from_secs(5);
    while !*stop.borrow() {
        out.send(Event::Push(PushStatus::Connecting));
        let channel = auth::random_token(32);
        let client_state = auth::random_token(24);
        let failure = 'session: {
            let socket =
                match tokio_tungstenite::connect_async(realtime::websocket_url(&relay, &channel))
                    .await
                {
                    Ok((socket, _)) => socket,
                    Err(_) => break 'session "the relay could not be reached".to_owned(),
                };
            let (mut sink, mut stream) = socket.split();
            let subscription = match graph
                .subscribe_chat_messages(
                    &me,
                    &realtime::notification_url(&relay, &channel),
                    &realtime::lifecycle_url(&relay, &channel),
                    &client_state,
                    realtime::LIFETIME,
                )
                .await
            {
                Ok(subscription) => subscription,
                Err(error) => {
                    break 'session format!("Microsoft Graph refused the subscription ({error})");
                }
            };
            out.send(Event::Push(PushStatus::Live));
            // A change may have happened while connecting.
            let _ = internal.send(Internal::PushMissed);
            pause = Duration::from_secs(5);
            let mut renew = tokio::time::interval(realtime::RENEW_EVERY);
            renew.tick().await;
            let mut keepalive = tokio::time::interval(Duration::from_secs(30));
            keepalive.tick().await;
            let reason = loop {
                tokio::select! {
                    changed = stop.changed() => {
                        if changed.is_err() || *stop.borrow() {
                            break None;
                        }
                    }
                    _ = renew.tick() => {
                        if let Err(error) = graph.renew_subscription(&subscription.id, realtime::LIFETIME).await {
                            break Some(format!("renewing the subscription failed ({error})"));
                        }
                    }
                    _ = keepalive.tick() => {
                        if sink.send(tokio_tungstenite::tungstenite::Message::Ping(Vec::new().into())).await.is_err() {
                            break Some("the relay connection dropped".to_owned());
                        }
                    }
                    message = stream.next() => {
                        let text = match message {
                            Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => text.to_string(),
                            Some(Ok(_)) => continue,
                            Some(Err(_)) | None => break Some("the relay connection dropped".to_owned()),
                        };
                        let mut recreate = false;
                        for notice in realtime::parse(&text, &client_state) {
                            match notice {
                                Notice::Chat(chat) => {
                                    let _ = internal.send(Internal::Pushed(chat));
                                }
                                Notice::Missed => {
                                    let _ = internal.send(Internal::PushMissed);
                                }
                                Notice::Reauthorize => {
                                    if graph.reauthorize_subscription(&subscription.id).await.is_err() {
                                        recreate = true;
                                    }
                                }
                                Notice::Removed => recreate = true,
                            }
                        }
                        if recreate {
                            break Some("Microsoft Graph ended the subscription".to_owned());
                        }
                    }
                }
            };
            // Best effort: a subscription left behind expires within the hour.
            let _ = graph.delete_subscription(&subscription.id).await;
            match reason {
                Some(reason) => reason,
                None => return,
            }
        };
        if *stop.borrow() {
            return;
        }
        log::info!("push: {failure}; retrying in {pause:?}");
        out.send(Event::Push(PushStatus::Failed(failure)));
        tokio::select! {
            _ = tokio::time::sleep(pause) => {}
            _ = stop.changed() => {}
        }
        pause = (pause * 2).min(Duration::from_secs(300));
    }
}

async fn browser_sign_in(
    http: &reqwest::Client,
    config: &AuthConfig,
    extra: &[String],
    hint: Option<&str>,
    out: &Outbox,
    cancelled: watch::Receiver<bool>,
    open_browser: bool,
) -> Result<token::TokenResponse, String> {
    let loopback = Loopback::bind().await.map_err(|e| e.to_string())?;
    let flow = auth::begin_browser(config, loopback.port(), extra, hint, !extra.is_empty());
    out.send(Event::Auth(AuthStatus::WaitingForBrowser {
        url: flow.url.clone(),
    }));
    if open_browser && let Err(error) = open::that_detached(&flow.url) {
        log::warn!("could not open the browser: {error}");
    }
    let code = loopback
        .wait_for_code(&flow.state, cancelled)
        .await
        .map_err(|e| e.to_string())?;
    out.send(Event::Auth(AuthStatus::Finishing));
    token::exchange_code(
        http,
        config,
        &code,
        &flow.verifier,
        &flow.redirect_uri,
        &flow.scopes,
    )
    .await
    .map_err(|e| e.to_string())
}

async fn device_code_sign_in(
    http: &reqwest::Client,
    config: &AuthConfig,
    extra: &[String],
    out: &Outbox,
    cancelled: watch::Receiver<bool>,
) -> Result<token::TokenResponse, String> {
    let scopes = auth::scope_string(extra);
    let code = token::start_device_code(http, config, &scopes)
        .await
        .map_err(|e| e.to_string())?;
    out.send(Event::Auth(AuthStatus::DeviceCode {
        code: code.user_code.clone(),
        url: code.verification_uri.clone(),
    }));
    let response = token::poll_device_code(http, config, &code, cancelled)
        .await
        .map_err(|e| e.to_string())?;
    out.send(Event::Auth(AuthStatus::Finishing));
    Ok(response)
}
