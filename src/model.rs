//! Wren's own types. Views and the store never see Graph's JSON shapes; the
//! Graph layer translates into these in `graph::convert`.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::html::RichText;

/// Unix time in milliseconds. Graph timestamps are converted on arrival so
/// sorting and grouping never parse strings in the frame.
pub type Millis = i64;

/// A Microsoft account signed in to Wren: `<object id>.<tenant id>`, as MSAL
/// names a home account. `demo` for the offline sample data.
pub type AccountId = String;

/// Where a message lives.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConversationId {
    /// A one-on-one, group or meeting chat (`19:...@thread.v2`).
    Chat(String),
    /// A channel of a team. Channel posts are threads: replies carry their
    /// root's id in [`Message::reply_to`].
    Channel { team: String, channel: String },
}

impl ConversationId {
    /// The key the store files messages under.
    pub fn key(&self) -> String {
        match self {
            Self::Chat(id) => format!("chat:{id}"),
            Self::Channel { team, channel } => format!("channel:{team}:{channel}"),
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        if let Some(id) = key.strip_prefix("chat:") {
            return Some(Self::Chat(id.to_owned()));
        }
        let rest = key.strip_prefix("channel:")?;
        // Team ids are GUIDs (no colon); channel ids contain colons.
        let (team, channel) = rest.split_once(':')?;
        Some(Self::Channel {
            team: team.to_owned(),
            channel: channel.to_owned(),
        })
    }

    pub fn chat_id(&self) -> Option<&str> {
        match self {
            Self::Chat(id) => Some(id),
            Self::Channel { .. } => None,
        }
    }

    pub fn is_channel(&self) -> bool {
        matches!(self, Self::Channel { .. })
    }
}

impl fmt::Display for ConversationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key())
    }
}

/// The signed-in person.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub display_name: String,
    pub mail: Option<String>,
    pub user_principal_name: String,
    pub job_title: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatKind {
    #[default]
    OneOnOne,
    Group,
    Meeting,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    pub user_id: String,
    pub display_name: String,
    pub email: Option<String>,
}

/// The newest message of a chat, as the chat list shows it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preview {
    pub message_id: String,
    pub author_id: Option<String>,
    pub author: Option<String>,
    pub text: String,
    pub at: Millis,
    pub deleted: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub kind: ChatKind,
    /// The chat's name, when it has one (group and meeting chats).
    pub topic: Option<String>,
    pub members: Vec<Member>,
    pub preview: Option<Preview>,
    /// When the signed-in person last read the chat (Graph `viewpoint`).
    pub last_read: Option<Millis>,
    pub updated: Millis,
    pub web_url: Option<String>,
    /// A meeting chat's join link.
    pub join_url: Option<String>,
    /// Hidden from the list by the person (Graph `viewpoint.isHidden`).
    pub hidden: bool,
}

impl Chat {
    /// What the list and the header call the chat: its topic, or the other
    /// members' names.
    pub fn title(&self, me: &str) -> String {
        if let Some(topic) = self.topic.as_deref().filter(|t| !t.trim().is_empty()) {
            return topic.to_owned();
        }
        let others: Vec<&str> = self
            .members
            .iter()
            .filter(|member| member.user_id != me)
            .map(|member| member.display_name.as_str())
            .filter(|name| !name.is_empty())
            .collect();
        match others.len() {
            0 => "Just you".to_owned(),
            1..=3 => others.join(", "),
            n => format!("{}, {} and {} others", others[0], others[1], n - 2),
        }
    }

    /// The other person of a one-on-one chat.
    pub fn peer<'a>(&'a self, me: &str) -> Option<&'a Member> {
        if self.kind != ChatKind::OneOnOne {
            return None;
        }
        self.members.iter().find(|member| member.user_id != me)
    }

    /// Whether the newest message arrived after the person last read it and
    /// was written by someone else.
    pub fn is_unread(&self, me: &str) -> bool {
        let Some(preview) = &self.preview else {
            return false;
        };
        if preview.author_id.as_deref() == Some(me) {
            return false;
        }
        self.last_read.is_none_or(|read| preview.at > read)
    }

    /// Sorting key for the list: newest activity first.
    pub fn activity(&self) -> Millis {
        self.preview
            .as_ref()
            .map_or(self.updated, |preview| preview.at.max(self.updated))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub web_url: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelKind {
    #[default]
    Standard,
    Private,
    Shared,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub team_id: String,
    pub name: String,
    pub description: Option<String>,
    pub kind: ChannelKind,
    pub web_url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reaction {
    /// The emoji itself. Graph's legacy names (`like`, `heart`, ...) are
    /// mapped to their emoji on arrival.
    pub emoji: String,
    pub users: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentKind {
    /// A file shared through OneDrive or SharePoint.
    File,
    /// The message this one replies to (Graph `messageReference`).
    Quote {
        message_id: String,
        author: Option<String>,
        preview: String,
    },
    /// An Adaptive Card or other card Wren does not draw.
    Card,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub kind: AttachmentKind,
    pub name: Option<String>,
    pub content_type: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Delivery {
    /// Confirmed by Graph.
    #[default]
    Sent,
    /// Shown optimistically, waiting for Graph.
    Sending,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation: Option<ConversationId>,
    /// A channel reply's root post.
    pub reply_to: Option<String>,
    pub author_id: Option<String>,
    pub author: String,
    pub created: Millis,
    pub modified: Millis,
    pub edited: bool,
    pub deleted: bool,
    /// A system event ("Ada added Grace to the chat"), drawn as a line.
    pub system: bool,
    pub subject: Option<String>,
    pub importance_high: bool,
    pub body: RichText,
    pub reactions: Vec<Reaction>,
    pub attachments: Vec<Attachment>,
    /// Channel posts: replies loaded with the root.
    pub replies: Vec<Message>,
    pub delivery: Delivery,
}

impl Message {
    pub fn is_from(&self, user: &str) -> bool {
        self.author_id.as_deref() == Some(user)
    }

    pub fn quote(&self) -> Option<&Attachment> {
        self.attachments
            .iter()
            .find(|attachment| matches!(attachment.kind, AttachmentKind::Quote { .. }))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Availability {
    Available,
    Busy,
    DoNotDisturb,
    BeRightBack,
    Away,
    Offline,
    #[default]
    Unknown,
}

impl Availability {
    pub fn from_graph(value: &str) -> Self {
        match value {
            "Available" | "AvailableIdle" => Self::Available,
            "Busy" | "BusyIdle" => Self::Busy,
            "DoNotDisturb" => Self::DoNotDisturb,
            "BeRightBack" => Self::BeRightBack,
            "Away" => Self::Away,
            "Offline" => Self::Offline,
            _ => Self::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "Available",
            Self::Busy => "Busy",
            Self::DoNotDisturb => "Do not disturb",
            Self::BeRightBack => "Be right back",
            Self::Away => "Away",
            Self::Offline => "Offline",
            Self::Unknown => "Status unknown",
        }
    }

    /// The value `setUserPreferredPresence` takes, for the ones a person can
    /// choose.
    pub fn graph_preferred(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Available => Some(("Available", "Available")),
            Self::Busy => Some(("Busy", "Busy")),
            Self::DoNotDisturb => Some(("DoNotDisturb", "DoNotDisturb")),
            Self::BeRightBack => Some(("BeRightBack", "BeRightBack")),
            Self::Away => Some(("Away", "Away")),
            Self::Offline => Some(("Offline", "OffWork")),
            Self::Unknown => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presence {
    pub availability: Availability,
    /// Graph's `activity` (`InACall`, `InAMeeting`, `Presenting`, ...).
    pub activity: Option<String>,
    pub status_message: Option<String>,
}

impl Presence {
    pub fn describe(&self) -> String {
        match self.activity.as_deref() {
            Some("InACall") => "In a call".into(),
            Some("InAConferenceCall") => "In a conference call".into(),
            Some("InAMeeting") => "In a meeting".into(),
            Some("Presenting") => "Presenting".into(),
            Some("OutOfOffice") => "Out of office".into(),
            Some("Inactive") => "Inactive".into(),
            Some("Focusing") => "Focusing".into(),
            _ => self.availability.label().into(),
        }
    }
}

/// An upcoming calendar event.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub subject: String,
    pub start: Millis,
    pub end: Millis,
    pub all_day: bool,
    pub location: Option<String>,
    pub organizer: Option<String>,
    /// The Teams join link, for online meetings.
    pub join_url: Option<String>,
    pub web_link: Option<String>,
    pub cancelled: bool,
    pub preview: Option<String>,
}

/// Where a feature stands for the signed-in account, from the scopes the
/// token carries and the answers Graph gave.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Access {
    #[default]
    Unknown,
    Granted,
    /// The scope was not granted; usually tenant admin consent is needed.
    NeedsConsent,
    /// Graph refused (403) even though the scope looked granted.
    Forbidden,
}

/// A person mentioned in a message being written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MentionRef {
    pub user_id: String,
    pub name: String,
}

/// A message found by search.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    pub conversation: ConversationId,
    pub message_id: String,
    pub author: String,
    pub snippet: String,
    pub at: Millis,
}

/// The rail's destinations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Section {
    #[default]
    Chats,
    Teams,
    Calendar,
    Calls,
}

impl Section {
    pub const ALL: [Self; 4] = [Self::Chats, Self::Teams, Self::Calendar, Self::Calls];

    pub fn label(self) -> &'static str {
        match self {
            Self::Chats => "Chat",
            Self::Teams => "Teams",
            Self::Calendar => "Calendar",
            Self::Calls => "Calls",
        }
    }
}

/// What a view asks the app to do. Views push these; `App::apply` runs them
/// after the frame, so no view mutates state it is drawing from.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Navigate(Section),
    OpenConversation(ConversationId),
    /// Opens a conversation scrolled to a message (search, notifications).
    OpenMessage {
        conversation: ConversationId,
        message: String,
    },
    ToggleTeam(String),
    CloseConversation,
    /// Moves the selection in the current list (Alt+Up/Down).
    Step(i32),

    Send {
        conversation: ConversationId,
        text: String,
        reply_to: Option<String>,
        /// People mentioned with `@`, in the order they were picked.
        mentions: Vec<MentionRef>,
        /// Files to upload to OneDrive and share in the message.
        files: Vec<std::path::PathBuf>,
    },
    Edit {
        conversation: ConversationId,
        message: String,
        text: String,
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
    /// Quotes a chat message, or opens a channel thread, in the composer.
    StartReply(String),
    StartEdit(String),
    CancelCompose,
    LoadOlder(ConversationId),
    LoadReplies {
        conversation: ConversationId,
        message: String,
    },
    /// Shows a channel thread's replies inline.
    ToggleThread(String),
    RetrySend {
        conversation: ConversationId,
        local_id: String,
    },
    /// Opens the file picker for the open conversation.
    AttachFiles,
    /// Adds files (picked or dropped) to the open conversation's draft.
    AddFiles(Vec<std::path::PathBuf>),
    /// Removes a file from the draft, by position.
    RemoveFile(usize),
    LoadMedia(String),

    SignIn,
    SignInWithCode,
    CancelSignIn,
    SignOut,
    /// Asks for an extra scope (incremental consent), such as channel
    /// message reading.
    RequestScope(&'static str),
    StartDemo,
    LeaveDemo,
    SaveAppRegistration {
        client_id: String,
        tenant: String,
    },
    /// Sets the signed-in person's availability; `None` resets it.
    SetAvailability(Option<Availability>),
    SetStatusMessage(String),
    OpenStatusMessage,
    /// Whether Wren keeps a presence session (shows you available while it
    /// runs).
    SetPresenceSession(bool),

    SwitchAccount(usize),
    AddAccount,
    /// Abandons an account being added and goes back to the previous one.
    CancelAddAccount,

    OpenNewChat,
    /// Looks people up in the directory for the new-chat dialog or mentions.
    SearchPeople(String),
    CreateChat {
        members: Vec<Member>,
        topic: Option<String>,
    },

    JoinMeeting(String),
    StartCall {
        users: Vec<String>,
        video: bool,
    },
    /// Closes the call bar (the call itself is in Teams) and restores the
    /// window.
    CallDone,
    /// Brings the call window forward.
    ShowCall,
    /// Docks Wren beside the call window (`true`) or restores its size.
    DockWindow(bool),
    /// Looks up the meeting's chat, asking for OnlineMeetings.Read first.
    FindMeetingChat,
    /// Shows the call's join link as a QR code, to join on a phone.
    JoinOnPhone,
    /// Opens Teams on the web once, to sign in there ahead of calls.
    PrepareTeamsWeb,
    SetCallSettings(crate::settings::CallSettings),
    OpenUrl(String),
    CopyText(String),

    OpenSwitcher,
    CloseOverlay,
    SearchMessages(String),
    OpenSettings,
    OpenShortcuts,
    SetTheme(crate::settings::ThemeChoice),
    SetNotifications(bool),
    SetKeepRunning(bool),
    /// The push relay's URL; empty turns push off.
    SetRelay(String),
    TogglePerfOverlay,
    Refresh,
    DismissToast(u64),
    Quit,
}

/// How a message toast looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub text: String,
}

/// Turns Graph's legacy reaction names into emoji. Newer reactions already
/// arrive as the emoji.
pub fn reaction_emoji(reaction: &str) -> String {
    match reaction {
        "like" => "👍".into(),
        "heart" => "❤️".into(),
        "laugh" => "😆".into(),
        "surprised" => "😮".into(),
        "sad" => "😢".into(),
        "angry" => "😡".into(),
        other => other.to_owned(),
    }
}

/// The reactions offered on hover, in Teams' order.
pub const QUICK_REACTIONS: [&str; 6] = ["👍", "❤️", "😆", "😮", "😢", "😡"];

#[cfg(test)]
mod tests {
    use super::*;

    fn member(id: &str, name: &str) -> Member {
        Member {
            user_id: id.into(),
            display_name: name.into(),
            email: None,
        }
    }

    #[test]
    fn conversation_keys_round_trip() {
        let chat = ConversationId::Chat("19:abc@thread.v2".into());
        assert_eq!(ConversationId::from_key(&chat.key()), Some(chat));
        let channel = ConversationId::Channel {
            team: "2f8e2b0c-1111-4a0a-9c55-000000000000".into(),
            channel: "19:general@thread.tacv2".into(),
        };
        assert_eq!(ConversationId::from_key(&channel.key()), Some(channel));
        assert_eq!(ConversationId::from_key("nonsense"), None);
    }

    #[test]
    fn chat_titles_name_the_other_members() {
        let mut chat = Chat {
            members: vec![member("me", "Me"), member("a", "Ada")],
            ..Default::default()
        };
        assert_eq!(chat.title("me"), "Ada");
        chat.members.extend([
            member("b", "Grace"),
            member("c", "Katherine"),
            member("d", "Dorothy"),
        ]);
        assert_eq!(chat.title("me"), "Ada, Grace and 2 others");
        chat.topic = Some("Launch".into());
        assert_eq!(chat.title("me"), "Launch");
    }

    #[test]
    fn unread_ignores_own_messages_and_read_marks() {
        let mut chat = Chat {
            preview: Some(Preview {
                author_id: Some("a".into()),
                at: 2_000,
                ..Default::default()
            }),
            last_read: Some(1_000),
            ..Default::default()
        };
        assert!(chat.is_unread("me"));
        chat.last_read = Some(3_000);
        assert!(!chat.is_unread("me"));
        chat.last_read = None;
        chat.preview.as_mut().unwrap().author_id = Some("me".into());
        assert!(!chat.is_unread("me"));
    }

    #[test]
    fn legacy_reactions_become_emoji() {
        assert_eq!(reaction_emoji("like"), "👍");
        assert_eq!(reaction_emoji("🎉"), "🎉");
    }
}
