//! The interface's handle to the background runtime.
//!
//! A tokio runtime on its own thread owns sign-in, the Graph client, the
//! cache and every network call (`backend/worker.rs`). The interface sends
//! [`Command`]s and drains [`Event`]s each frame; every event wakes the
//! window through the shell's [`Waker`], so an idle window draws nothing.

mod worker;

use std::time::Duration;

use tokio::sync::mpsc;

use crate::model::{
    Access, Availability, Channel, Chat, ConversationId, Meeting, Member, MentionRef, Message,
    Presence, Profile, SearchHit, Team,
};
use crate::paths::AppDirs;
use crate::sync::{Resource, Trigger};

pub use fastframe_shell::Waker;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthStatus {
    /// No app registration is set up yet.
    NotConfigured,
    SignedOut,
    /// Reading the stored session at start.
    Restoring,
    /// The browser is open on Microsoft's sign-in page.
    WaitingForBrowser {
        url: String,
    },
    /// Device code sign-in: enter `code` at `url` on any device.
    DeviceCode {
        code: String,
        url: String,
    },
    /// Exchanging the code for tokens.
    Finishing,
    SignedIn {
        account: String,
        name: String,
        username: String,
    },
    /// The stored session no longer works; sign in again.
    Expired {
        username: String,
        reason: String,
    },
    Failed(String),
}

impl AuthStatus {
    /// For the log: no URLs, codes or names.
    pub fn log_label(&self) -> &'static str {
        match self {
            Self::NotConfigured => "not configured",
            Self::SignedOut => "signed out",
            Self::Restoring => "restoring",
            Self::WaitingForBrowser { .. } => "waiting for browser",
            Self::DeviceCode { .. } => "device code",
            Self::Finishing => "finishing",
            Self::SignedIn { .. } => "signed in",
            Self::Expired { .. } => "expired",
            Self::Failed(_) => "failed",
        }
    }
}

#[derive(Debug)]
pub enum Command {
    /// The app registration and the account to restore, once at start and
    /// whenever the registration changes.
    Start {
        client_id: String,
        tenant: String,
        account: Option<String>,
    },
    SignIn {
        extra_scopes: Vec<String>,
        device_code: bool,
    },
    CancelSignIn,
    SignOut,
    Fetch {
        resource: Resource,
        trigger: Trigger,
    },
    LoadOlder(ConversationId),
    LoadReplies {
        conversation: ConversationId,
        root: String,
    },
    Send {
        conversation: ConversationId,
        local_id: String,
        html: String,
        /// A chat message to quote, or a channel post to reply under.
        reply_to: Option<String>,
        /// The people `<at id="n">` tags in `html` name, by index.
        mentions: Vec<MentionRef>,
        /// Files to upload to OneDrive and share in the message.
        files: Vec<std::path::PathBuf>,
        /// Who to share the files with (the chat's members' addresses).
        recipients: Vec<String>,
    },
    SearchPeople(String),
    /// A one-on-one chat with one person, or a group chat with several.
    CreateChat {
        others: Vec<String>,
        topic: Option<String>,
    },
    /// Sets (`Some`) or resets (`None`) the person's preferred availability.
    SetAvailability(Option<Availability>),
    SetStatusMessage(String),
    /// The person is using Wren: keep the presence session Available.
    PresenceActive,
    /// Whether Wren keeps a presence session at all.
    PresenceSession(bool),
    /// The browser (and profile) to open Microsoft's sign-in page in, so
    /// Teams on the web shares the session; `None` uses the default browser.
    SignInBrowser(Option<crate::calling::browser::BrowserProfile>),
    /// Finds the chat of the meeting with this join link.
    FindMeetingChat(String),
    Edit {
        conversation: ConversationId,
        message: String,
        html: String,
    },
    Delete {
        conversation: ConversationId,
        message: String,
    },
    React {
        conversation: ConversationId,
        message: String,
        emoji: String,
        on: bool,
    },
    MarkRead(String),
    /// The push relay to use, or none (experimental, docs/realtime.md).
    Relay(Option<String>),
    Presence(Vec<String>),
    Photo(String),
    Media(String),
    Search(String),
    Shutdown,
}

/// How a batch of messages relates to what the interface already holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Merge {
    /// The newest page: replaces what is shown, keeping older loaded pages
    /// only when they connect.
    Latest,
    /// Changes since the last refresh, merged by id.
    Changes,
    /// An older page, added before what is shown.
    Older,
}

#[derive(Debug)]
pub enum Event {
    Auth(AuthStatus),
    Profile(Box<Profile>),
    /// The scopes the session holds, space-separated.
    Granted(String),
    Chats {
        chats: Vec<Chat>,
        cached: bool,
    },
    Teams {
        teams: Vec<Team>,
        cached: bool,
    },
    Channels {
        team: Option<String>,
        channels: Vec<Channel>,
    },
    Messages {
        conversation: ConversationId,
        messages: Vec<Message>,
        merge: Merge,
        /// Whether older messages exist beyond these.
        more: Option<bool>,
        cached: bool,
    },
    Replies {
        conversation: ConversationId,
        root: String,
        replies: Vec<Message>,
    },
    Sent {
        conversation: ConversationId,
        local_id: String,
        message: Box<Message>,
    },
    SendFailed {
        conversation: ConversationId,
        local_id: String,
        reason: String,
    },
    Deleted {
        conversation: ConversationId,
        message: String,
    },
    /// Whether channel posts can be read with the granted scopes.
    ChannelAccess(Access),
    Presence(Vec<(String, Presence)>),
    Photo {
        user: String,
        bytes: Option<Vec<u8>>,
    },
    Media {
        url: String,
        bytes: Option<Vec<u8>>,
    },
    Meetings {
        meetings: Vec<Meeting>,
        cached: bool,
    },
    SearchResults {
        query: String,
        hits: Vec<SearchHit>,
        remote: bool,
    },
    /// A fetch started (`true`) or ended.
    Loading(Resource, bool),
    /// The push relay's state.
    Push(crate::realtime::PushStatus),
    /// Directory search results for `query`.
    People {
        query: String,
        people: Vec<Member>,
    },
    ChatCreated(Box<Chat>),
    /// The signed-in person's own presence.
    MyPresence(Presence),
    /// The chat of a meeting, looked up by its join link (`None`: not found).
    MeetingChat {
        join_url: String,
        chat: Option<Box<Chat>>,
    },
    /// Something the person should hear about, in plain words.
    Error(String),
}

/// Where the backend signs in and fetches from, and where it keeps secrets.
/// The defaults are Microsoft's endpoints and the OS credential store; tests
/// point them at local mock servers and an in-memory store.
#[derive(Clone)]
pub struct BackendOptions {
    pub authority: String,
    pub graph_base: String,
    pub secrets: std::sync::Arc<dyn crate::secrets::SecretStore>,
    /// Open the system browser for sign-in (off in tests).
    pub open_browser: bool,
}

impl Default for BackendOptions {
    fn default() -> Self {
        Self {
            authority: crate::auth::DEFAULT_AUTHORITY.to_owned(),
            graph_base: crate::graph::GRAPH_BASE.to_owned(),
            secrets: std::sync::Arc::new(crate::secrets::NativeStore::default()),
            open_browser: true,
        }
    }
}

pub struct Backend {
    commands: mpsc::UnboundedSender<Command>,
    events: std::sync::mpsc::Receiver<Event>,
    thread: Option<std::thread::JoinHandle<()>>,
    detached: bool,
}

impl Backend {
    pub fn spawn(dirs: AppDirs, waker: Waker) -> Self {
        Self::spawn_with(dirs, waker, BackendOptions::default())
    }

    pub fn spawn_with(dirs: AppDirs, waker: Waker, options: BackendOptions) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("wren-runtime")
            .enable_all()
            .build()
            .expect("unable to start the async runtime");
        let thread = std::thread::Builder::new()
            .name("wren-backend".into())
            .spawn(move || {
                runtime.block_on(worker::run(dirs, options, event_tx, command_rx, waker));
                runtime.shutdown_timeout(Duration::from_secs(2));
            })
            .expect("unable to start the backend thread");
        Self {
            commands: command_tx,
            events: event_rx,
            thread: Some(thread),
            detached: false,
        }
    }

    /// A backend with no runtime, for the demo and tests. Commands are
    /// dropped; the returned sender delivers events.
    pub fn detached() -> (Self, std::sync::mpsc::Sender<Event>) {
        let (command_tx, _) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        (
            Self {
                commands: command_tx,
                events: event_rx,
                thread: None,
                detached: true,
            },
            event_tx,
        )
    }

    /// Records commands for a test.
    pub fn recording() -> (Self, mpsc::UnboundedReceiver<Command>) {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (_, event_rx) = std::sync::mpsc::channel();
        (
            Self {
                commands: command_tx,
                events: event_rx,
                thread: None,
                detached: false,
            },
            command_rx,
        )
    }

    pub fn is_detached(&self) -> bool {
        self.detached
    }

    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }

    pub fn poll(&self) -> Vec<Event> {
        self.events.try_iter().collect()
    }

    pub fn shutdown(&mut self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        self.shutdown();
    }
}
