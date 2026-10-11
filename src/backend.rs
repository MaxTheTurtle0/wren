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
    Access, Channel, Chat, ConversationId, Meeting, Message, Presence, Profile, SearchHit, Team,
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
    },
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
    /// Something the person should hear about, in plain words.
    Error(String),
}

pub struct Backend {
    commands: mpsc::UnboundedSender<Command>,
    events: std::sync::mpsc::Receiver<Event>,
    thread: Option<std::thread::JoinHandle<()>>,
    detached: bool,
}

impl Backend {
    pub fn spawn(dirs: AppDirs, waker: Waker) -> Self {
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
                runtime.block_on(worker::run(dirs, event_tx, command_rx, waker));
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
