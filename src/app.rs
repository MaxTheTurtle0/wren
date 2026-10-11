//! Application state and the one place it changes.
//!
//! Views in `ui/` read [`App`] and push [`Action`]s; [`App::apply`] runs
//! them after the frame. Backend [`Event`]s are applied at the start of each
//! frame. Nothing here waits on the network.
//!
//! Several Microsoft accounts can be signed in at once. Each [`Account`] has
//! its own backend (runtime thread, Graph session, cache, credentials) and
//! its own view state; [`App`] dereferences to the account on screen, so
//! views draw `app.chats` without knowing there are others. Events from an
//! account that is not on screen are applied with `hidden` set: they update
//! that account, notify, and never touch what the window shows.

use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::backend::{AuthStatus, Backend, Command, Event, Merge, Waker};
use crate::html;
use crate::model::{
    Access, Action, Availability, Channel, Chat, ChatKind, ConversationId, Delivery, Meeting,
    Member, MentionRef, Message, Presence, Profile, Reaction, SearchHit, Section, Team, Toast,
    ToastKind,
};
use crate::paths::AppDirs;
use crate::settings::Settings;
use crate::sync::{Resource, Trigger};
use crate::theme::Palette;

mod calls;
pub use calls::Docked;

/// Messages and view state of one conversation.
#[derive(Debug, Default)]
pub struct Conversation {
    /// Oldest first.
    pub messages: Vec<Message>,
    /// Older messages exist beyond the loaded ones.
    pub more: bool,
    /// Graph answered at least once (not only the cache).
    pub fresh: bool,
    /// Measured row heights, by message id (virtualized list).
    pub heights: HashMap<String, f32>,
    /// Channel threads shown expanded.
    pub expanded: HashSet<String>,
}

impl Conversation {
    pub fn find(&self, id: &str) -> Option<&Message> {
        self.messages.iter().find(|m| m.id == id).or_else(|| {
            self.messages
                .iter()
                .flat_map(|m| m.replies.iter())
                .find(|m| m.id == id)
        })
    }

    fn find_mut(&mut self, id: &str) -> Option<&mut Message> {
        if let Some(index) = self.messages.iter().position(|m| m.id == id) {
            return self.messages.get_mut(index);
        }
        self.messages
            .iter_mut()
            .flat_map(|m| m.replies.iter_mut())
            .find(|m| m.id == id)
    }

    /// Merges messages by id, keeping messages still being sent.
    pub fn merge(&mut self, incoming: Vec<Message>, merge: Merge) {
        match merge {
            Merge::Latest if self.messages.iter().all(|m| m.delivery == Delivery::Sent) => {
                // Keep older loaded messages when the new page connects to
                // them (overlaps by id).
                let oldest_new = incoming.iter().map(|m| m.created).min();
                let overlaps = incoming
                    .iter()
                    .any(|m| self.messages.iter().any(|o| o.id == m.id));
                let mut keep: Vec<Message> = if overlaps {
                    self.messages
                        .drain(..)
                        .filter(|m| oldest_new.is_some_and(|oldest| m.created < oldest))
                        .collect()
                } else {
                    Vec::new()
                };
                keep.extend(incoming);
                self.messages = keep;
            }
            _ => {
                for message in incoming {
                    if let Some(reply_to) = message.reply_to.clone()
                        && message
                            .conversation
                            .as_ref()
                            .is_some_and(ConversationId::is_channel)
                        && let Some(root) = self.messages.iter_mut().find(|m| m.id == reply_to)
                    {
                        match root.replies.iter_mut().find(|r| r.id == message.id) {
                            Some(existing) => *existing = message,
                            None => root.replies.push(message),
                        }
                        continue;
                    }
                    match self.messages.iter_mut().find(|m| m.id == message.id) {
                        Some(existing) => {
                            let replies = std::mem::take(&mut existing.replies);
                            *existing = message;
                            if existing.replies.is_empty() {
                                existing.replies = replies;
                            }
                        }
                        None => self.messages.push(message),
                    }
                }
            }
        }
        self.messages.sort_by(|a, b| {
            // Unsent messages stay at the end.
            (a.delivery != Delivery::Sent, a.created, &a.id).cmp(&(
                b.delivery != Delivery::Sent,
                b.created,
                &b.id,
            ))
        });
        self.messages.dedup_by(|a, b| a.id == b.id);
    }
}

/// What the composer of one conversation holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Draft {
    pub text: String,
    /// The chat message being quoted, or the channel post being replied to.
    pub reply_to: Option<String>,
    /// The message being edited.
    pub editing: Option<String>,
    /// People picked with `@`, in order.
    pub mentions: Vec<MentionRef>,
    /// Files to send with the message.
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Overlay {
    Switcher,
    Settings,
    Shortcuts,
    NewChat,
    StatusMessage,
    /// A feature needs a permission the account has not granted yet.
    Consent(&'static str),
    /// A join link as a QR code, to join on a phone.
    JoinOnPhone(String),
}

#[derive(Debug, Default)]
pub struct Switcher {
    pub query: String,
    pub selected: usize,
    pub hits: Vec<SearchHit>,
    pub remote_hits: Vec<SearchHit>,
    pub searched: String,
}

/// The new-chat dialog.
#[derive(Debug, Default)]
pub struct NewChat {
    pub query: String,
    pub picked: Vec<Member>,
    pub topic: String,
    pub selected: usize,
}

/// Directory search results, shared by the new-chat dialog and mentions.
#[derive(Debug, Default)]
pub struct People {
    pub query: String,
    pub results: Vec<Member>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Image {
    Loading,
    Ready(String),
    Missing,
}

/// A send Graph has not confirmed, kept for Retry.
#[derive(Clone, Debug)]
struct PendingSend {
    html: String,
    reply_to: Option<String>,
    mentions: Vec<MentionRef>,
    files: Vec<PathBuf>,
    recipients: Vec<String>,
}

/// Frame timing for the performance overlay.
#[derive(Debug, Default)]
pub struct Perf {
    pub cpu: Vec<f32>,
    pub frames: u64,
    pub started: Option<Instant>,
    pub first_frame: Option<Duration>,
}

impl Perf {
    pub fn record(&mut self, cpu: Option<f32>) {
        self.frames += 1;
        if let Some(cpu) = cpu {
            if self.cpu.len() >= 120 {
                self.cpu.remove(0);
            }
            self.cpu.push(cpu);
        }
    }

    pub fn average_ms(&self) -> f32 {
        if self.cpu.is_empty() {
            return 0.0;
        }
        self.cpu.iter().sum::<f32>() / self.cpu.len() as f32 * 1000.0
    }
}

/// One Microsoft account: its backend and everything Wren shows for it.
pub struct Account {
    pub backend: Backend,
    /// `oid.tid`, once signed in or restored.
    pub id: Option<String>,
    pub auth: AuthStatus,
    pub profile: Option<Profile>,
    pub granted: String,
    pub channel_access: Access,

    pub chats: Vec<Chat>,
    pub teams: Vec<Team>,
    pub channels: HashMap<String, Vec<Channel>>,
    pub conversations: HashMap<ConversationId, Conversation>,
    pub presence: HashMap<String, Presence>,
    /// The signed-in person's own presence.
    pub my_presence: Option<Presence>,
    pub meetings: Vec<Meeting>,
    pub photos: HashMap<String, Image>,
    pub media: HashMap<String, Image>,
    pub loading: HashSet<Resource>,

    pub open: Option<ConversationId>,
    pub expanded_teams: HashSet<String>,
    pub selected_meeting: Option<String>,
    pub chat_filter: String,
    pub drafts: HashMap<ConversationId, Draft>,
    pub push: crate::realtime::PushStatus,

    /// The browser profile folder for this account's calls, when each
    /// account has its own (`settings.calls.profile_per_account`).
    pub browser_profile: String,

    pending_sends: HashMap<String, PendingSend>,
    next_local: u64,
    reminded: HashSet<String>,
}

impl Account {
    pub fn new(backend: Backend) -> Self {
        Self {
            backend,
            id: None,
            auth: AuthStatus::Restoring,
            profile: None,
            granted: String::new(),
            channel_access: Access::Unknown,
            chats: Vec::new(),
            teams: Vec::new(),
            channels: HashMap::new(),
            conversations: HashMap::new(),
            presence: HashMap::new(),
            my_presence: None,
            meetings: Vec::new(),
            photos: HashMap::new(),
            media: HashMap::new(),
            loading: HashSet::new(),
            open: None,
            expanded_teams: HashSet::new(),
            selected_meeting: None,
            chat_filter: String::new(),
            drafts: HashMap::new(),
            push: crate::realtime::PushStatus::Off,
            browser_profile: format!("Wren {}", crate::auth::random_token(4)),
            pending_sends: HashMap::new(),
            next_local: 0,
            reminded: HashSet::new(),
        }
    }

    /// The signed-in person's id (Graph user id).
    pub fn me(&self) -> &str {
        if let Some(profile) = &self.profile {
            return &profile.id;
        }
        match &self.auth {
            AuthStatus::SignedIn { account, .. } => account.split('.').next().unwrap_or_default(),
            _ => self
                .id
                .as_deref()
                .and_then(|id| id.split('.').next())
                .unwrap_or_default(),
        }
    }

    /// The account's name for menus and notifications.
    pub fn label(&self) -> String {
        if let Some(profile) = &self.profile
            && !profile.display_name.is_empty()
        {
            return profile.display_name.clone();
        }
        match &self.auth {
            AuthStatus::SignedIn { name, username, .. } if !name.is_empty() => {
                let _ = username;
                name.clone()
            }
            AuthStatus::SignedIn { username, .. } | AuthStatus::Expired { username, .. } => {
                username.clone()
            }
            _ => "New account".into(),
        }
    }

    /// The account's sign-in address.
    pub fn username(&self) -> String {
        match &self.auth {
            AuthStatus::SignedIn { username, .. } | AuthStatus::Expired { username, .. } => {
                username.clone()
            }
            _ => self
                .profile
                .as_ref()
                .map(|p| p.user_principal_name.clone())
                .unwrap_or_default(),
        }
    }

    pub fn is_signed_in_here(&self) -> bool {
        matches!(self.auth, AuthStatus::SignedIn { .. })
    }

    pub fn chat(&self, id: &str) -> Option<&Chat> {
        self.chats.iter().find(|chat| chat.id == id)
    }

    pub fn channel(&self, team: &str, channel: &str) -> Option<&Channel> {
        self.channels.get(team)?.iter().find(|c| c.id == channel)
    }

    pub fn team(&self, team: &str) -> Option<&Team> {
        self.teams.iter().find(|t| t.id == team)
    }

    pub fn conversation(&self, id: &ConversationId) -> Option<&Conversation> {
        self.conversations.get(id)
    }

    pub fn unread_chats(&self) -> usize {
        let me = self.me();
        self.chats
            .iter()
            .filter(|chat| !chat.hidden && chat.is_unread(me))
            .count()
    }

    /// A conversation's title for headers, notifications and search.
    pub fn title_of(&self, conversation: &ConversationId) -> String {
        match conversation {
            ConversationId::Chat(id) => self
                .chat(id)
                .map(|chat| chat.title(self.me()))
                .unwrap_or_else(|| "Chat".into()),
            ConversationId::Channel { team, channel } => {
                let team_name = self.team(team).map(|t| t.name.as_str()).unwrap_or("Team");
                let channel_name = self
                    .channel(team, channel)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Channel");
                format!("{team_name} › {channel_name}")
            }
        }
    }

    /// Everyone in the account's chats except the person, by name: the
    /// local directory for mentions and new chats.
    pub fn directory(&self) -> Vec<Member> {
        let me = self.me();
        let mut people: Vec<Member> = Vec::new();
        for member in self.chats.iter().flat_map(|chat| chat.members.iter()) {
            if member.user_id != me
                && !member.display_name.is_empty()
                && !people.iter().any(|p| p.user_id == member.user_id)
            {
                people.push(member.clone());
            }
        }
        people.sort_by_key(|p| p.display_name.to_lowercase());
        people
    }

    fn clear_data(&mut self) {
        self.profile = None;
        self.granted.clear();
        self.chats.clear();
        self.teams.clear();
        self.channels.clear();
        self.conversations.clear();
        self.presence.clear();
        self.my_presence = None;
        self.meetings.clear();
        self.photos.clear();
        self.media.clear();
        self.drafts.clear();
        self.open = None;
    }
}

pub struct App {
    pub dirs: AppDirs,
    pub settings: Settings,
    settings_dirty: bool,
    pub waker: Waker,
    pub demo: bool,

    pub accounts: Vec<Account>,
    pub active: usize,
    /// Events being applied belong to an account that is not on screen.
    hidden: bool,
    /// The account shown before "Add account", to go back to.
    adding_from: Option<usize>,
    /// Account slots to drop after this round of events (duplicates).
    retire: Vec<usize>,

    pub section: Section,
    pub overlay: Option<Overlay>,
    pub switcher: Switcher,
    pub new_chat: NewChat,
    pub people: People,
    /// A message to scroll to once its conversation is drawn.
    pub jump_to: Option<String>,
    pub toasts: Vec<Toast>,
    next_toast: u64,
    pub actions: Vec<Action>,

    /// Forms.
    pub form_client_id: String,
    pub form_tenant: String,
    pub form_relay: String,
    pub form_status: String,

    pub palette: Palette,
    applied_palette: Option<Palette>,
    pub window_focused: bool,
    pub window_hidden: bool,
    pub quit_requested: bool,
    pub wants_show: bool,
    pub perf: Perf,
    pub tray: Option<fastframe_tray::Tray>,
    pub requests: Option<std::sync::mpsc::Receiver<String>>,
    pub focus_composer: bool,
    /// Frames left in which the open history scrolls to its newest message.
    pub scroll_to_end: u8,
    /// When the person last touched the window (egui time), for presence.
    last_input: Option<f64>,
    /// Opens the account menu on the next frame (screenshots, tests).
    pub open_account_menu: bool,

    /// The call or meeting the call bar follows.
    pub call: Option<crate::calling::session::CallSession>,
    /// Wren's window before it moved beside a call window.
    pub docked: Option<Docked>,
    /// Edge and Chrome, found on first use.
    browsers: Option<Vec<crate::calling::browser::Browser>>,
}

impl Deref for App {
    type Target = Account;

    fn deref(&self) -> &Account {
        &self.accounts[self.active]
    }
}

impl DerefMut for App {
    fn deref_mut(&mut self) -> &mut Account {
        &mut self.accounts[self.active]
    }
}

impl App {
    pub fn new(dirs: AppDirs, settings: Settings, waker: Waker, demo: bool) -> Self {
        let mut app = Self::empty(dirs, settings, waker, demo);
        if demo {
            app.accounts.push(Account::new(Backend::detached().0));
            crate::demo::populate(&mut app);
        } else {
            let ids = app.settings.accounts.clone();
            if ids.is_empty() {
                app.add_slot(None);
            }
            for id in &ids {
                app.add_slot(Some(id.clone()));
            }
            app.active = app
                .settings
                .account
                .as_ref()
                .and_then(|active| ids.iter().position(|id| id == active))
                .unwrap_or(0);
        }
        app
    }

    /// An app with one account slot on `backend` (tests).
    pub fn with_backend(
        dirs: AppDirs,
        settings: Settings,
        waker: Waker,
        backend: Backend,
        demo: bool,
    ) -> Self {
        let mut app = Self::empty(dirs, settings, waker, demo);
        app.accounts.push(Account::new(backend));
        app
    }

    fn empty(dirs: AppDirs, settings: Settings, waker: Waker, demo: bool) -> Self {
        let palette = crate::theme::resolve(&settings.theme, true);
        Self {
            form_client_id: settings.client_id.clone(),
            form_tenant: settings.tenant.clone(),
            form_relay: settings.relay_url.clone().unwrap_or_default(),
            form_status: String::new(),
            dirs,
            settings,
            settings_dirty: false,
            waker,
            demo,
            accounts: Vec::new(),
            active: 0,
            hidden: false,
            adding_from: None,
            retire: Vec::new(),
            section: Section::Chats,
            overlay: None,
            switcher: Switcher::default(),
            new_chat: NewChat::default(),
            people: People::default(),
            jump_to: None,
            toasts: Vec::new(),
            next_toast: 0,
            actions: Vec::new(),
            palette,
            applied_palette: None,
            window_focused: true,
            window_hidden: false,
            quit_requested: false,
            wants_show: false,
            perf: Perf::default(),
            tray: None,
            requests: None,
            focus_composer: false,
            scroll_to_end: 4,
            last_input: None,
            open_account_menu: false,
            call: None,
            docked: None,
            browsers: None,
        }
    }

    /// Starts a backend for an account (or a new sign-in when `None`) and
    /// adds its slot.
    fn add_slot(&mut self, id: Option<String>) -> usize {
        let backend = Backend::spawn(self.dirs.clone(), self.waker.clone());
        let mut account = Account::new(backend);
        if let Some(profile) = id
            .as_ref()
            .and_then(|id| self.settings.calls.profiles.get(id))
        {
            account.browser_profile = profile.clone();
        }
        account.id = id;
        self.accounts.push(account);
        let index = self.accounts.len() - 1;
        self.start_account(index);
        self.sync_call_browser();
        index
    }

    fn start_account(&mut self, index: usize) {
        let configured = self.settings.has_app_registration();
        let start = Command::Start {
            client_id: self.settings.client_id.clone(),
            tenant: self.settings.tenant.clone(),
            account: self.accounts[index].id.clone(),
        };
        let relay = Command::Relay(self.settings.relay_url.clone());
        let session = Command::PresenceSession(self.settings.presence_session);
        let account = &mut self.accounts[index];
        account.auth = if configured {
            AuthStatus::Restoring
        } else {
            AuthStatus::NotConfigured
        };
        account.backend.send(start);
        account.backend.send(relay);
        account.backend.send(session);
    }

    /// Stops an account's backend without blocking the window: it finishes
    /// what it was doing (a sign-out, say) on its own thread.
    fn retire_slot(&mut self, index: usize) {
        if index >= self.accounts.len() {
            return;
        }
        let account = self.accounts.remove(index);
        let _ = std::thread::Builder::new()
            .name("wren-retire".into())
            .spawn(move || drop(account));
        if self.active >= self.accounts.len() {
            self.active = self.accounts.len().saturating_sub(1);
        } else if self.active > index {
            self.active -= 1;
        }
    }

    pub fn is_signed_in(&self) -> bool {
        self.demo || matches!(self.auth, AuthStatus::SignedIn { .. })
    }

    /// The account the person was in before "Add account", while adding.
    pub fn adding_from(&self) -> Option<&Account> {
        self.adding_from.and_then(|index| self.accounts.get(index))
    }

    /// Whether the main interface (rather than the sign-in screen) shows.
    pub fn shows_workspace(&self) -> bool {
        self.is_signed_in()
            || (matches!(
                self.auth,
                AuthStatus::Restoring | AuthStatus::Expired { .. }
            ) && !self.chats.is_empty())
    }

    /// Unread chats in accounts other than the one on screen.
    pub fn other_unread(&self) -> usize {
        self.accounts
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != self.active)
            .map(|(_, account)| account.unread_chats())
            .sum()
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.demo || crate::auth::has_scope(&self.granted, scope)
    }

    /// Whether the account holds `scope`; otherwise asks for consent.
    fn ensure_scope(&mut self, scope: &'static str) -> bool {
        if self.has_scope(scope) {
            return true;
        }
        self.overlay = Some(Overlay::Consent(scope));
        false
    }

    pub fn toast(&mut self, kind: ToastKind, text: impl Into<String>) {
        let mut text = text.into();
        if self.hidden && self.accounts.len() > 1 {
            text = format!("{}: {text}", self.label());
        }
        if self.toasts.iter().any(|t| t.text == text) {
            return;
        }
        self.next_toast += 1;
        self.toasts.push(Toast {
            id: self.next_toast,
            kind,
            text,
        });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    fn fetch(&self, resource: Resource, trigger: Trigger) {
        if !self.demo {
            self.backend.send(Command::Fetch { resource, trigger });
        }
    }

    /// Asks for whatever the current view shows.
    fn fetch_view(&self, trigger: Trigger) {
        match self.section {
            Section::Chats => self.fetch(Resource::Chats, trigger),
            Section::Teams => self.fetch(Resource::Teams, trigger),
            Section::Calendar => self.fetch(Resource::Calendar, trigger),
            Section::Calls => {}
        }
        if let Some(open) = &self.open {
            self.fetch(Resource::Messages(open.clone()), trigger);
        }
    }

    pub fn mark_settings_dirty(&mut self) {
        self.settings_dirty = true;
    }

    pub fn save_settings(&mut self) {
        if self.demo || !self.settings_dirty {
            return;
        }
        self.settings_dirty = false;
        if let Err(error) = self.settings.save(&self.dirs.settings_file()) {
            log::warn!("could not save settings: {error}");
        }
    }

    /// Records which accounts exist and which one shows, for the next start.
    fn remember_accounts(&mut self) {
        let ids: Vec<String> = self.accounts.iter().filter_map(|a| a.id.clone()).collect();
        let active = self.id.clone();
        if self.settings.accounts != ids || self.settings.account != active {
            self.settings.accounts = ids;
            self.settings.account = active;
            self.mark_settings_dirty();
        }
    }

    // -- Frame -----------------------------------------------------------------

    /// Work every frame does, with or without a window: backend events of
    /// every account, tray clicks, reminders.
    pub fn background(&mut self, ctx: &egui::Context) {
        let shown = self.active;
        for index in 0..self.accounts.len() {
            let events = self.accounts[index].backend.poll();
            if events.is_empty() {
                continue;
            }
            self.active = index;
            self.hidden = index != shown;
            for event in events {
                self.handle(event, ctx);
            }
        }
        self.active = shown.min(self.accounts.len().saturating_sub(1));
        self.hidden = false;
        self.retire_duplicates();
        self.handle_tray();
        self.handle_requests();
        self.remind(ctx);
        self.apply_actions(ctx);
        self.save_settings();
    }

    fn retire_duplicates(&mut self) {
        let mut retire = std::mem::take(&mut self.retire);
        retire.sort_unstable();
        retire.dedup();
        for index in retire.into_iter().rev() {
            self.retire_slot(index);
        }
    }

    /// Tracks focus and input; coming back to the window refreshes what it
    /// shows, and using it keeps the presence session Available.
    pub fn observe_window(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if focused && !self.window_focused {
            self.fetch_view(Trigger::Focused);
            self.fetch(Resource::MyPresence, Trigger::Focused);
            if let Some(ConversationId::Chat(chat)) = self.open.clone() {
                self.mark_read(&chat);
            }
        }
        self.window_focused = focused;
        let active_input = ctx.input(|i| {
            i.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Key { .. }
                        | egui::Event::Text(_)
                        | egui::Event::PointerButton { .. }
                        | egui::Event::MouseWheel { .. }
                )
            })
        });
        if focused && active_input {
            let now = ctx.input(|i| i.time);
            // Presence needs a refresh only every few minutes.
            if self.last_input.is_none_or(|last| now - last > 60.0) && !self.demo {
                for account in &self.accounts {
                    account.backend.send(Command::PresenceActive);
                }
            }
            self.last_input = Some(now);
        }
        let system_dark = ctx.input(|i| i.raw.system_theme) != Some(egui::Theme::Light);
        self.palette = crate::theme::resolve(&self.settings.theme, system_dark);
        if self.applied_palette != Some(self.palette) {
            crate::theme::apply(ctx, &self.palette);
            self.applied_palette = Some(self.palette);
        }
        let zoom = self.settings.zoom;
        if (ctx.zoom_factor() - zoom).abs() > f32::EPSILON {
            ctx.set_zoom_factor(zoom);
        }
        self.tick_call(ctx);
    }

    fn handle_tray(&mut self) {
        let Some(tray) = &self.tray else { return };
        for event in tray.events() {
            match event {
                fastframe_tray::Event::Toggle
                | fastframe_tray::Event::Show
                | fastframe_tray::Event::Menu("show") => {
                    self.wants_show = true;
                }
                fastframe_tray::Event::Menu("quit") => self.quit_requested = true,
                fastframe_tray::Event::Menu(_) => {}
            }
        }
    }

    fn handle_requests(&mut self) {
        let Some(requests) = &self.requests else {
            return;
        };
        for request in requests.try_iter().collect::<Vec<_>>() {
            if request == "show" {
                self.wants_show = true;
            }
        }
    }

    /// Meeting reminders from the cached calendars: no network involved.
    fn remind(&mut self, ctx: &egui::Context) {
        if !self.settings.notifications {
            return;
        }
        let now = crate::time::now();
        let lead = 5 * 60 * 1000;
        let mut next: Option<i64> = None;
        let demo = self.demo;
        for account in &mut self.accounts {
            let mut due = Vec::new();
            for meeting in &account.meetings {
                if meeting.cancelled || meeting.all_day || account.reminded.contains(&meeting.id) {
                    continue;
                }
                let at = meeting.start - lead;
                if at <= now && meeting.start > now - 60_000 {
                    due.push(meeting.clone());
                } else if at > now {
                    next = Some(next.map_or(at, |n| n.min(at)));
                }
            }
            for meeting in due {
                account.reminded.insert(meeting.id.clone());
                if !demo {
                    crate::notify::show(
                        &meeting.subject,
                        &format!("Starts {}", crate::time::until_label(meeting.start, now)),
                    );
                }
            }
        }
        if let Some(next) = next {
            ctx.request_repaint_after(Duration::from_millis(
                (next - now).clamp(1_000, 3_600_000) as u64
            ));
        }
    }

    // -- Events ------------------------------------------------------------------

    /// Applies one event to the account at `self.active`.
    pub fn handle(&mut self, event: Event, ctx: &egui::Context) {
        match event {
            Event::Auth(status) => self.on_auth(status),
            Event::Profile(profile) => self.profile = Some(*profile),
            Event::Granted(granted) => self.granted = granted,
            Event::ChannelAccess(access) => self.channel_access = access,
            Event::Chats { chats, cached } => self.on_chats(chats, cached),
            Event::Teams { teams, .. } => {
                if self.expanded_teams.is_empty()
                    && let Some(first) = teams.first()
                {
                    let first = first.id.clone();
                    self.expanded_teams.insert(first);
                }
                self.teams = teams;
            }
            Event::Channels { team, channels } => match team {
                Some(team) => {
                    self.channels.insert(team, channels);
                }
                None => {
                    self.channels.clear();
                    for channel in channels {
                        self.channels
                            .entry(channel.team_id.clone())
                            .or_default()
                            .push(channel);
                    }
                }
            },
            Event::Messages {
                conversation,
                messages,
                merge,
                more,
                cached,
            } => self.on_messages(conversation, messages, merge, more, cached),
            Event::Replies {
                conversation,
                root,
                replies,
            } => {
                let entry = self.conversations.entry(conversation).or_default();
                if let Some(root) = entry.messages.iter_mut().find(|m| m.id == root) {
                    root.replies = replies;
                }
            }
            Event::Sent {
                conversation,
                local_id,
                message,
            } => self.on_sent(conversation, local_id, *message),
            Event::SendFailed {
                conversation,
                local_id,
                reason,
            } => {
                if let Some(entry) = self.conversations.get_mut(&conversation)
                    && let Some(message) = entry.find_mut(&local_id)
                {
                    message.delivery = Delivery::Failed;
                }
                self.toast(ToastKind::Error, format!("Not sent: {reason}"));
            }
            Event::Deleted {
                conversation,
                message,
            } => {
                if let Some(entry) = self.conversations.get_mut(&conversation)
                    && let Some(message) = entry.find_mut(&message)
                {
                    message.deleted = true;
                    message.body = html::RichText::default();
                }
            }
            Event::Presence(list) => self.presence.extend(list),
            Event::MyPresence(presence) => {
                let me = self.me().to_owned();
                if !me.is_empty() {
                    self.presence.insert(me, presence.clone());
                }
                self.observe_call(presence.activity.as_deref());
                self.my_presence = Some(presence);
            }
            Event::MeetingChat { join_url, chat } => self.on_meeting_chat(join_url, chat),
            Event::Photo { user, bytes } => {
                let state = match bytes
                    .and_then(|bytes| register_image(ctx, &format!("photo-{user}"), bytes))
                {
                    Some(uri) => Image::Ready(uri),
                    None => Image::Missing,
                };
                self.photos.insert(user, state);
            }
            Event::Media { url, bytes } => {
                let key = format!("media-{:016x}", fnv(&url));
                let state = match bytes.and_then(|bytes| register_image(ctx, &key, bytes)) {
                    Some(uri) => Image::Ready(uri),
                    None => Image::Missing,
                };
                self.media.insert(url, state);
            }
            Event::Meetings { meetings, .. } => self.meetings = meetings,
            Event::SearchResults {
                query,
                hits,
                remote,
            } => {
                if !self.hidden && query == self.switcher.searched {
                    if remote {
                        self.switcher.remote_hits = hits;
                    } else {
                        self.switcher.hits = hits;
                    }
                }
            }
            Event::People { query, people } => {
                if !self.hidden && query == self.people.query {
                    self.people.results = people;
                }
            }
            Event::ChatCreated(chat) => {
                let chat = *chat;
                let id = chat.id.clone();
                self.chats.retain(|c| c.id != id);
                self.chats.insert(0, chat);
                if !self.hidden {
                    self.open_conversation(ConversationId::Chat(id));
                }
            }
            Event::Loading(resource, on) => {
                if on {
                    self.loading.insert(resource);
                } else {
                    self.loading.remove(&resource);
                }
            }
            Event::Error(text) => self.toast(ToastKind::Error, text),
            Event::Push(status) => self.push = status,
        }
    }

    fn on_auth(&mut self, status: AuthStatus) {
        log::info!("account: {}", status.log_label());
        let became_signed_in = matches!(status, AuthStatus::SignedIn { .. })
            && !matches!(self.auth, AuthStatus::SignedIn { .. });
        match &status {
            AuthStatus::SignedIn { account, .. } => {
                // The same account signed in twice (through "Add account"):
                // keep the slot that had it and show that one.
                if let Some(existing) =
                    self.accounts.iter().enumerate().position(|(index, a)| {
                        index != self.active && a.id.as_deref() == Some(account)
                    })
                {
                    self.retire.push(self.active);
                    self.adding_from = None;
                    self.toast(ToastKind::Info, "That account is already signed in.");
                    self.actions
                        .push(Action::SwitchAccount(if existing > self.active {
                            existing - 1
                        } else {
                            existing
                        }));
                    return;
                }
                self.id = Some(account.clone());
                self.remember_browser_profile();
                if !self.hidden {
                    self.adding_from = None;
                }
            }
            AuthStatus::SignedOut => {
                self.clear_data();
            }
            AuthStatus::Expired { username, reason } => {
                // Keep the name the account had when Graph does not repeat it.
                if username.is_empty()
                    && let AuthStatus::SignedIn {
                        username: known, ..
                    } = &self.auth
                {
                    let known = known.clone();
                    let reason = reason.clone();
                    self.auth = AuthStatus::Expired {
                        username: known,
                        reason,
                    };
                    return;
                }
            }
            _ => {}
        }
        self.auth = status;
        if became_signed_in {
            self.remember_accounts_for_slot();
            for resource in [
                Resource::Profile,
                Resource::Chats,
                Resource::Teams,
                Resource::Calendar,
                Resource::MyPresence,
            ] {
                self.fetch(resource, Trigger::Start);
            }
            if let Some(open) = &self.open {
                self.fetch(Resource::Messages(open.clone()), Trigger::Start);
            }
        }
    }

    /// `remember_accounts` from inside event handling, where the active
    /// index may point at a hidden account.
    fn remember_accounts_for_slot(&mut self) {
        let ids: Vec<String> = self.accounts.iter().filter_map(|a| a.id.clone()).collect();
        if self.settings.accounts != ids {
            self.settings.accounts = ids;
            self.mark_settings_dirty();
        }
        if !self.hidden && self.settings.account != self.id {
            self.settings.account = self.id.clone();
            self.mark_settings_dirty();
        }
    }

    fn on_chats(&mut self, mut chats: Vec<Chat>, cached: bool) {
        chats.sort_by_key(|chat| std::cmp::Reverse(chat.activity()));
        let me = self.me().to_owned();
        if !cached && !self.chats.is_empty() {
            let mut notices = Vec::new();
            for chat in &chats {
                let Some(preview) = &chat.preview else {
                    continue;
                };
                let previous = self.chat(&chat.id).and_then(|c| c.preview.as_ref());
                let new = previous
                    .is_none_or(|p| p.message_id != preview.message_id && preview.at > p.at);
                let looking = !self.hidden
                    && self.window_focused
                    && self.open == Some(ConversationId::Chat(chat.id.clone()));
                if new
                    && !looking
                    && preview.author_id.as_deref() != Some(me.as_str())
                    && !preview.deleted
                {
                    notices.push((
                        chat.title(&me),
                        preview.author.clone(),
                        preview.text.clone(),
                    ));
                }
            }
            for (title, author, text) in notices {
                self.notify_message(&title, author.as_deref(), &text);
            }
        }
        // Read state Wren set locally survives a cached list.
        for chat in &mut chats {
            if let Some(known) = self.chats.iter().find(|c| c.id == chat.id)
                && known.last_read > chat.last_read
            {
                chat.last_read = known.last_read;
            }
        }
        self.chats = chats;
    }

    fn notify_message(&self, title: &str, author: Option<&str>, text: &str) {
        if !self.settings.notifications || self.demo {
            return;
        }
        let body = match author {
            Some(author) if !title.contains(author) => format!("{author}: {text}"),
            _ => text.to_owned(),
        };
        // With several accounts, say which one the message is for.
        let title = if self.accounts.len() > 1 {
            format!("{title} · {}", self.username())
        } else {
            title.to_owned()
        };
        crate::notify::show(&title, &body);
    }

    fn on_messages(
        &mut self,
        conversation: ConversationId,
        messages: Vec<Message>,
        merge: Merge,
        more: Option<bool>,
        cached: bool,
    ) {
        let me = self.me().to_owned();
        let open = !self.hidden && self.open.as_ref() == Some(&conversation);
        let focused = self.window_focused;
        let entry = self.conversations.entry(conversation.clone()).or_default();
        let newest_before = entry
            .messages
            .iter()
            .filter(|m| m.delivery == Delivery::Sent)
            .map(|m| m.created)
            .max();
        let incoming_new: Vec<(String, String)> = messages
            .iter()
            .filter(|m| !m.system && !m.deleted && !m.is_from(&me))
            .filter(|m| newest_before.is_some_and(|newest| m.created > newest))
            .map(|m| (m.author.clone(), m.body.summary(140)))
            .collect();
        entry.merge(messages, merge);
        if let Some(more) = more {
            entry.more = more;
        }
        if !cached {
            entry.fresh = true;
        }
        if !cached && !(open && focused) {
            let title = self.title_of(&conversation);
            for (author, text) in incoming_new.iter().take(3) {
                self.notify_message(&title, Some(author), text);
            }
        }
        if open
            && !cached
            && let ConversationId::Chat(chat) = &conversation
        {
            self.mark_read(&chat.clone());
        }
        // Keep the chat list's preview in step with what the chat shows.
        if let ConversationId::Chat(chat_id) = &conversation
            && let Some(last) = self
                .conversations
                .get(&conversation)
                .and_then(|c| c.messages.last())
                .cloned()
            && let Some(chat) = self.chats.iter_mut().find(|c| &c.id == chat_id)
            && chat.preview.as_ref().is_none_or(|p| p.at < last.created)
            && last.delivery == Delivery::Sent
        {
            chat.preview = Some(crate::model::Preview {
                message_id: last.id.clone(),
                author_id: last.author_id.clone(),
                author: Some(last.author.clone()),
                text: preview_text(&last),
                at: last.created,
                deleted: last.deleted,
            });
            self.chats
                .sort_by_key(|chat| std::cmp::Reverse(chat.activity()));
        }
    }

    fn on_sent(&mut self, conversation: ConversationId, local_id: String, message: Message) {
        self.pending_sends.remove(&local_id);
        let entry = self.conversations.entry(conversation.clone()).or_default();
        if let Some(reply_to) = &message.reply_to
            && conversation.is_channel()
        {
            if let Some(root) = entry.messages.iter_mut().find(|m| &m.id == reply_to) {
                root.replies.retain(|r| r.id != local_id);
                root.replies.push(message);
            }
            return;
        }
        entry.messages.retain(|m| m.id != local_id);
        entry.merge(vec![message], Merge::Changes);
        self.fetch(Resource::Messages(conversation), Trigger::AfterAction);
    }

    fn mark_read(&mut self, chat: &str) {
        if self.hidden {
            return;
        }
        let me = self.me().to_owned();
        let demo = self.demo;
        let Some(entry) = self.chats.iter_mut().find(|c| c.id == chat) else {
            return;
        };
        if !entry.is_unread(&me) {
            return;
        }
        entry.last_read = Some(crate::time::now());
        if !demo {
            self.backend.send(Command::MarkRead(chat.to_owned()));
        }
    }

    // -- Actions -------------------------------------------------------------------

    pub fn apply_actions(&mut self, ctx: &egui::Context) {
        let mut rounds = 0;
        while !self.actions.is_empty() && rounds < 8 {
            rounds += 1;
            for action in std::mem::take(&mut self.actions) {
                self.apply(action, ctx);
            }
        }
    }

    pub fn apply(&mut self, action: Action, ctx: &egui::Context) {
        match action {
            Action::Navigate(section) => {
                if self.section != section {
                    self.section = section;
                    self.open = match (section, &self.open) {
                        (Section::Chats, Some(open @ ConversationId::Chat(_))) => {
                            Some(open.clone())
                        }
                        (Section::Teams, Some(open @ ConversationId::Channel { .. })) => {
                            Some(open.clone())
                        }
                        _ => None,
                    };
                }
                self.fetch_view(Trigger::Opened);
            }
            Action::OpenConversation(conversation) => self.open_conversation(conversation),
            Action::OpenMessage {
                conversation,
                message,
            } => {
                self.open_conversation(conversation);
                self.jump_to = Some(message);
                self.overlay = None;
            }
            Action::ToggleTeam(team) => {
                if !self.expanded_teams.remove(&team) {
                    self.expanded_teams.insert(team.clone());
                    if self.channels.get(&team).is_none_or(Vec::is_empty) {
                        self.fetch(Resource::Channels(team), Trigger::Opened);
                    }
                }
            }
            Action::CloseConversation => self.open = None,
            Action::Step(delta) => self.step(delta),
            Action::Send {
                conversation,
                text,
                reply_to,
                mentions,
                files,
            } => self.send(conversation, text, reply_to, mentions, files),
            Action::Edit {
                conversation,
                message,
                text,
            } => {
                let html = html::compose(&text);
                if let Some(entry) = self.conversations.get_mut(&conversation)
                    && let Some(found) = entry.find_mut(&message)
                {
                    found.body = html::parse(&html);
                    found.edited = true;
                }
                if !self.demo {
                    self.backend.send(Command::Edit {
                        conversation: conversation.clone(),
                        message,
                        html,
                    });
                }
                self.drafts.remove(&conversation);
            }
            Action::Delete {
                conversation,
                message,
            } => {
                if let Some(entry) = self.conversations.get_mut(&conversation)
                    && let Some(found) = entry.find_mut(&message)
                {
                    found.deleted = true;
                    found.body = html::RichText::default();
                }
                if !self.demo {
                    self.backend.send(Command::Delete {
                        conversation,
                        message,
                    });
                }
            }
            Action::React {
                conversation,
                message,
                emoji,
                on,
            } => {
                let me = self.me().to_owned();
                if let Some(entry) = self.conversations.get_mut(&conversation)
                    && let Some(found) = entry.find_mut(&message)
                {
                    toggle_reaction(&mut found.reactions, &emoji, &me, on);
                }
                if !self.demo {
                    self.backend.send(Command::React {
                        conversation,
                        message,
                        emoji,
                        on,
                    });
                }
            }
            Action::StartReply(message) => {
                if let Some(open) = self.open.clone() {
                    let draft = self.drafts.entry(open).or_default();
                    draft.reply_to = Some(message);
                    draft.editing = None;
                    self.focus_composer = true;
                }
            }
            Action::StartEdit(message) => {
                if let Some(open) = self.open.clone() {
                    let text = self
                        .conversations
                        .get(&open)
                        .and_then(|c| c.find(&message))
                        .map(|m| m.body.to_plain())
                        .unwrap_or_default();
                    self.drafts.insert(
                        open,
                        Draft {
                            text,
                            editing: Some(message),
                            ..Default::default()
                        },
                    );
                    self.focus_composer = true;
                }
            }
            Action::CancelCompose => {
                if let Some(open) = self.open.clone()
                    && let Some(draft) = self.drafts.get_mut(&open)
                {
                    if draft.editing.take().is_some() {
                        draft.text.clear();
                    }
                    draft.reply_to = None;
                }
            }
            Action::LoadOlder(conversation) => {
                if !self.demo {
                    self.backend.send(Command::LoadOlder(conversation));
                } else if let Some(entry) = self.conversations.get_mut(&conversation) {
                    entry.more = false;
                }
            }
            Action::LoadReplies {
                conversation,
                message,
            } => {
                if !self.demo {
                    self.backend.send(Command::LoadReplies {
                        conversation,
                        root: message,
                    });
                }
            }
            Action::ToggleThread(root) => {
                if let Some(open) = self.open.clone() {
                    let entry = self.conversations.entry(open).or_default();
                    if !entry.expanded.remove(&root) {
                        entry.expanded.insert(root);
                    }
                }
            }
            Action::RetrySend {
                conversation,
                local_id,
            } => {
                if let Some(pending) = self.pending_sends.get(&local_id).cloned() {
                    if let Some(entry) = self.conversations.get_mut(&conversation)
                        && let Some(found) = entry.find_mut(&local_id)
                    {
                        found.delivery = Delivery::Sending;
                    }
                    self.backend.send(Command::Send {
                        conversation,
                        local_id,
                        html: pending.html,
                        reply_to: pending.reply_to,
                        mentions: pending.mentions,
                        files: pending.files,
                        recipients: pending.recipients,
                    });
                }
            }
            Action::AttachFiles => {
                if self.open.is_none() || !self.ensure_scope(crate::auth::FILES_SCOPE) {
                    return;
                }
                // The platform's own picker; it is modal, like a dialog.
                if let Some(paths) = rfd::FileDialog::new()
                    .set_title("Attach files")
                    .pick_files()
                {
                    self.actions.push(Action::AddFiles(paths));
                }
            }
            Action::AddFiles(paths) => self.add_files(paths),
            Action::RemoveFile(index) => {
                if let Some(open) = self.open.clone()
                    && let Some(draft) = self.drafts.get_mut(&open)
                    && index < draft.files.len()
                {
                    draft.files.remove(index);
                }
            }
            Action::LoadMedia(url) => {
                if !self.media.contains_key(&url) {
                    let demo = self.demo;
                    self.media.insert(url.clone(), Image::Loading);
                    if demo {
                        self.media.insert(url, Image::Missing);
                    } else {
                        self.backend.send(Command::Media(url));
                    }
                }
            }
            Action::SignIn => self.backend.send(Command::SignIn {
                extra_scopes: Vec::new(),
                device_code: false,
            }),
            Action::SignInWithCode => self.backend.send(Command::SignIn {
                extra_scopes: Vec::new(),
                device_code: true,
            }),
            Action::CancelSignIn => {
                self.backend.send(Command::CancelSignIn);
                self.auth = AuthStatus::SignedOut;
            }
            Action::SignOut => self.sign_out(),
            Action::RequestScope(scope) => {
                self.overlay = None;
                if !self.demo {
                    self.backend.send(Command::SignIn {
                        extra_scopes: vec![scope.to_owned()],
                        device_code: false,
                    });
                    self.toast(
                        ToastKind::Info,
                        "Approve the permission in your browser, then try again.",
                    );
                }
            }
            Action::StartDemo => self.switch_demo(true),
            Action::LeaveDemo => self.switch_demo(false),
            Action::SaveAppRegistration { client_id, tenant } => {
                let client_id = client_id.trim().to_owned();
                if !crate::settings::is_guid(&client_id) {
                    self.toast(
                        ToastKind::Error,
                        "The application (client) ID is a GUID like 00000000-0000-0000-0000-000000000000.",
                    );
                    return;
                }
                self.settings.client_id = client_id;
                self.settings.tenant = match tenant.trim() {
                    "" => "organizations".into(),
                    other => other.to_owned(),
                };
                self.mark_settings_dirty();
                self.save_settings();
                if !self.demo {
                    for index in 0..self.accounts.len() {
                        self.start_account(index);
                    }
                }
            }
            Action::SetAvailability(availability) => {
                if !self.ensure_scope(crate::auth::PRESENCE_WRITE_SCOPE) {
                    return;
                }
                // Shown at once; Graph's answer replaces it.
                let shown = Presence {
                    availability: availability.unwrap_or(Availability::Available),
                    activity: None,
                    status_message: self
                        .my_presence
                        .as_ref()
                        .and_then(|p| p.status_message.clone()),
                };
                let me = self.me().to_owned();
                self.presence.insert(me, shown.clone());
                self.my_presence = Some(shown);
                if !self.demo {
                    self.backend.send(Command::SetAvailability(availability));
                }
            }
            Action::OpenStatusMessage => {
                if !self.ensure_scope(crate::auth::PRESENCE_WRITE_SCOPE) {
                    return;
                }
                self.form_status = self
                    .my_presence
                    .as_ref()
                    .and_then(|p| p.status_message.clone())
                    .unwrap_or_default();
                self.overlay = Some(Overlay::StatusMessage);
            }
            Action::SetStatusMessage(text) => {
                self.overlay = None;
                let text = text.trim().to_owned();
                let mut presence = self.my_presence.clone().unwrap_or_default();
                presence.status_message = (!text.is_empty()).then(|| text.clone());
                let me = self.me().to_owned();
                self.presence.insert(me, presence.clone());
                self.my_presence = Some(presence);
                if !self.demo {
                    self.backend.send(Command::SetStatusMessage(text));
                }
            }
            Action::SetPresenceSession(on) => {
                self.settings.presence_session = on;
                self.mark_settings_dirty();
                for account in &self.accounts {
                    account.backend.send(Command::PresenceSession(on));
                }
            }
            Action::SwitchAccount(index) => {
                if index < self.accounts.len() && index != self.active {
                    self.active = index;
                    self.overlay = None;
                    self.switcher = Switcher::default();
                    self.scroll_to_end = 4;
                    self.remember_accounts();
                    self.fetch_view(Trigger::Focused);
                    self.fetch(Resource::MyPresence, Trigger::Focused);
                }
            }
            Action::AddAccount => {
                if self.demo {
                    self.toast(
                        ToastKind::Info,
                        "The demo has its accounts already: switch between them in this menu.",
                    );
                    return;
                }
                self.overlay = None;
                let from = self.active;
                let index = self.add_slot(None);
                self.adding_from = Some(from);
                self.active = index;
                if self.settings.has_app_registration() {
                    // Straight to Microsoft's account picker.
                    self.backend.send(Command::SignIn {
                        extra_scopes: Vec::new(),
                        device_code: false,
                    });
                }
            }
            Action::CancelAddAccount => {
                if let Some(from) = self.adding_from.take() {
                    self.backend.send(Command::CancelSignIn);
                    let adding = self.active;
                    self.retire_slot(adding);
                    self.active = if from > adding { from - 1 } else { from }
                        .min(self.accounts.len().saturating_sub(1));
                }
            }
            Action::OpenNewChat => {
                if !self.ensure_scope(crate::auth::CHAT_CREATE_SCOPE) {
                    return;
                }
                self.new_chat = NewChat::default();
                self.people = People::default();
                self.overlay = Some(Overlay::NewChat);
            }
            Action::SearchPeople(query) => {
                self.people.query = query.clone();
                self.people.results.clear();
                if query.trim().chars().count() < 2 {
                    return;
                }
                if self.demo {
                    self.people.results = crate::demo::search_people(self, &query);
                } else {
                    self.backend.send(Command::SearchPeople(query));
                }
            }
            Action::CreateChat { members, topic } => self.create_chat(members, topic),
            Action::JoinMeeting(url) => self.join_meeting(url, ctx),
            Action::StartCall { users, video } => self.start_call(users, video, ctx),
            Action::CallDone => self.end_call(ctx),
            Action::ShowCall => self.show_call(),
            Action::DockWindow(true) => {
                self.dock(ctx);
            }
            Action::DockWindow(false) => self.undock(ctx),
            Action::FindMeetingChat => self.find_meeting_chat(),
            Action::JoinOnPhone => self.join_on_phone(),
            Action::PrepareTeamsWeb => self.prepare_teams_web(),
            Action::SetCallSettings(calls) => self.set_call_settings(calls),
            Action::OpenUrl(url) => {
                if url.starts_with("https://")
                    || url.starts_with("http://")
                    || url.starts_with("mailto:")
                {
                    open_external(&url);
                }
            }
            Action::CopyText(text) => {
                ctx.copy_text(text);
                self.toast(ToastKind::Info, "Copied");
            }
            Action::OpenSwitcher => {
                self.switcher = Switcher::default();
                self.overlay = Some(Overlay::Switcher);
            }
            Action::CloseOverlay => self.overlay = None,
            Action::SearchMessages(query) => {
                self.switcher.searched = query.clone();
                self.switcher.hits.clear();
                self.switcher.remote_hits.clear();
                if query.trim().is_empty() {
                    return;
                }
                if self.demo {
                    self.switcher.hits = self.local_search(&query);
                } else {
                    self.backend.send(Command::Search(query));
                }
            }
            Action::OpenSettings => self.overlay = Some(Overlay::Settings),
            Action::OpenShortcuts => self.overlay = Some(Overlay::Shortcuts),
            Action::SetTheme(choice) => {
                self.settings.theme = choice;
                self.mark_settings_dirty();
            }
            Action::SetNotifications(on) => {
                self.settings.notifications = on;
                self.mark_settings_dirty();
            }
            Action::SetKeepRunning(on) => {
                self.settings.keep_running = on;
                self.mark_settings_dirty();
                // The tray is the way back to a closed window: make it now.
                if on && self.tray.is_none() && !self.demo {
                    self.tray = crate::platform::tray(&self.waker);
                    if let Some(tray) = &mut self.tray {
                        tray.attach();
                    }
                }
            }
            Action::SetRelay(url) => {
                let url = url.trim().to_owned();
                if !url.is_empty() && crate::realtime::normalize_relay(&url).is_none() {
                    self.toast(ToastKind::Error, "The relay must be an https:// address.");
                    return;
                }
                self.settings.relay_url = (!url.is_empty()).then_some(url);
                self.mark_settings_dirty();
                if !self.demo {
                    for account in &self.accounts {
                        account
                            .backend
                            .send(Command::Relay(self.settings.relay_url.clone()));
                    }
                }
            }
            Action::TogglePerfOverlay => {
                self.settings.perf_overlay = !self.settings.perf_overlay;
                self.mark_settings_dirty();
            }
            Action::Refresh => {
                self.fetch_view(Trigger::Refresh);
                if self.demo {
                    self.toast(ToastKind::Info, "The demo has no account to refresh from.");
                }
            }
            Action::DismissToast(id) => self.toasts.retain(|t| t.id != id),
            Action::Quit => {
                self.quit_requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn open_conversation(&mut self, conversation: ConversationId) {
        self.section = match conversation {
            ConversationId::Chat(_) => Section::Chats,
            ConversationId::Channel { .. } => Section::Teams,
        };
        if let ConversationId::Channel { team, .. } = &conversation {
            self.expanded_teams.insert(team.clone());
        }
        self.open = Some(conversation.clone());
        self.focus_composer = true;
        self.scroll_to_end = 4;
        self.fetch(Resource::Messages(conversation.clone()), Trigger::Opened);
        if let ConversationId::Chat(chat) = &conversation {
            self.mark_read(chat);
            let me = self.me().to_owned();
            let members: Vec<String> = self
                .chat(chat)
                .map(|c| {
                    c.members
                        .iter()
                        .filter(|m| m.user_id != me)
                        .map(|m| m.user_id.clone())
                        .take(20)
                        .collect()
                })
                .unwrap_or_default();
            if !members.is_empty() && !self.demo {
                self.backend.send(Command::Presence(members));
            }
        }
    }

    fn step(&mut self, delta: i32) {
        let me = self.me().to_owned();
        let list: Vec<ConversationId> = match self.section {
            Section::Chats => self
                .chats
                .iter()
                .filter(|c| !c.hidden)
                .filter(|c| crate::ui::chat_list::matches_filter(c, &self.chat_filter, &me))
                .map(|c| ConversationId::Chat(c.id.clone()))
                .collect(),
            Section::Teams => self
                .teams
                .iter()
                .flat_map(|t| self.channels.get(&t.id).into_iter().flatten())
                .map(|c| ConversationId::Channel {
                    team: c.team_id.clone(),
                    channel: c.id.clone(),
                })
                .collect(),
            _ => return,
        };
        if list.is_empty() {
            return;
        }
        let current = self
            .open
            .as_ref()
            .and_then(|open| list.iter().position(|c| c == open));
        let next = match current {
            Some(index) => (index as i32 + delta).clamp(0, list.len() as i32 - 1) as usize,
            None => 0,
        };
        self.open_conversation(list[next].clone());
    }

    fn add_files(&mut self, paths: Vec<PathBuf>) {
        let Some(open) = self.open.clone() else {
            self.toast(ToastKind::Info, "Open a chat or channel to send files to.");
            return;
        };
        if !self.ensure_scope(crate::auth::FILES_SCOPE) {
            return;
        }
        let mut refused = Vec::new();
        let draft = self.drafts.entry(open).or_default();
        for path in paths {
            let size = std::fs::metadata(&path).map(|m| (m.is_file(), m.len()));
            match size {
                Ok((true, size)) if size <= crate::graph::files::MAX_FILE => {
                    if !draft.files.contains(&path) {
                        draft.files.push(path);
                    }
                }
                _ => refused.push(
                    path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                ),
            }
        }
        self.focus_composer = true;
        if !refused.is_empty() {
            self.toast(
                ToastKind::Error,
                format!(
                    "Not attached (folders and files over 1 GB are not sent): {}",
                    refused.join(", ")
                ),
            );
        }
    }

    fn send(
        &mut self,
        conversation: ConversationId,
        text: String,
        reply_to: Option<String>,
        mentions: Vec<MentionRef>,
        files: Vec<PathBuf>,
    ) {
        let text = text.trim_end().to_owned();
        if text.trim().is_empty() && files.is_empty() {
            return;
        }
        let (html, mentions) = html::compose_with_mentions(&text, &mentions);
        self.next_local += 1;
        let local_id = format!("local-{}", self.next_local);
        let me = self.me().to_owned();
        let author = self
            .profile
            .as_ref()
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| "You".into());
        let channel_reply = conversation.is_channel() && reply_to.is_some();
        let demo = self.demo;
        let mut message = Message {
            id: local_id.clone(),
            conversation: Some(conversation.clone()),
            reply_to: if channel_reply {
                reply_to.clone()
            } else {
                None
            },
            author_id: Some(me.clone()),
            author,
            created: crate::time::now(),
            modified: crate::time::now(),
            body: html::parse(&html),
            delivery: if demo {
                Delivery::Sent
            } else {
                Delivery::Sending
            },
            ..Default::default()
        };
        for path in &files {
            message.attachments.push(crate::model::Attachment {
                id: path.display().to_string(),
                kind: crate::model::AttachmentKind::File,
                name: path.file_name().map(|n| n.to_string_lossy().into_owned()),
                content_type: Some("reference".into()),
                url: None,
            });
        }
        if let (Some(quoted), false) = (&reply_to, conversation.is_channel())
            && let Some(original) = self
                .conversations
                .get(&conversation)
                .and_then(|c| c.find(quoted))
        {
            message.attachments.push(crate::model::Attachment {
                id: quoted.clone(),
                kind: crate::model::AttachmentKind::Quote {
                    message_id: quoted.clone(),
                    author: Some(original.author.clone()),
                    preview: original.body.summary(200),
                },
                name: None,
                content_type: Some("messageReference".into()),
                url: None,
            });
        }
        if demo {
            message.id = format!("demo-{}", self.next_local);
        }
        // Who can open the files: everyone in the chat. Channel uploads get
        // an organisation link instead (Graph does not list channel members
        // with Wren's permissions).
        let recipients: Vec<String> = match &conversation {
            ConversationId::Chat(chat) => self
                .chat(chat)
                .map(|c| {
                    c.members
                        .iter()
                        .filter(|m| m.user_id != me)
                        .filter_map(|m| m.email.clone())
                        .collect()
                })
                .unwrap_or_default(),
            ConversationId::Channel { .. } => Vec::new(),
        };
        let entry = self.conversations.entry(conversation.clone()).or_default();
        if channel_reply {
            if let Some(root) = entry
                .messages
                .iter_mut()
                .find(|m| Some(&m.id) == reply_to.as_ref())
            {
                root.replies.push(message.clone());
            }
            entry.expanded.insert(reply_to.clone().unwrap_or_default());
        } else {
            entry.messages.push(message.clone());
        }
        if let ConversationId::Chat(chat_id) = &conversation
            && let Some(chat) = self.chats.iter_mut().find(|c| &c.id == chat_id)
        {
            chat.preview = Some(crate::model::Preview {
                message_id: message.id.clone(),
                author_id: Some(me),
                author: Some(message.author.clone()),
                text: preview_text(&message),
                at: message.created,
                deleted: false,
            });
            self.chats
                .sort_by_key(|chat| std::cmp::Reverse(chat.activity()));
        }
        self.drafts.remove(&conversation);
        if !channel_reply {
            self.scroll_to_end = 3;
        }
        if !demo {
            self.pending_sends.insert(
                local_id.clone(),
                PendingSend {
                    html: html.clone(),
                    reply_to: reply_to.clone(),
                    mentions: mentions.clone(),
                    files: files.clone(),
                    recipients: recipients.clone(),
                },
            );
            self.backend.send(Command::Send {
                conversation,
                local_id,
                html,
                reply_to,
                mentions,
                files,
                recipients,
            });
        }
    }

    fn create_chat(&mut self, members: Vec<Member>, topic: Option<String>) {
        if members.is_empty() {
            return;
        }
        self.overlay = None;
        let me = self.me().to_owned();
        // A one-on-one chat that exists already just opens.
        if members.len() == 1
            && let Some(existing) = self.chats.iter().find(|chat| {
                chat.kind == ChatKind::OneOnOne
                    && chat
                        .peer(&me)
                        .is_some_and(|p| p.user_id == members[0].user_id)
            })
        {
            let id = existing.id.clone();
            self.open_conversation(ConversationId::Chat(id));
            return;
        }
        if self.demo {
            let chat = crate::demo::new_chat(self, members, topic);
            let id = chat.id.clone();
            self.chats.insert(0, chat);
            self.conversations
                .entry(ConversationId::Chat(id.clone()))
                .or_default();
            self.open_conversation(ConversationId::Chat(id));
            return;
        }
        self.backend.send(Command::CreateChat {
            others: members.into_iter().map(|m| m.user_id).collect(),
            topic,
        });
        self.toast(ToastKind::Info, "Starting the chat…");
    }

    fn sign_out(&mut self) {
        self.overlay = None;
        if self.demo {
            return;
        }
        self.sign_out_browser();
        self.backend.send(Command::SignOut);
        let index = self.active;
        self.retire_slot(index);
        if self.accounts.is_empty() {
            self.add_slot(None);
            self.active = 0;
        }
        self.remember_accounts();
        self.fetch_view(Trigger::Focused);
    }

    /// Search over loaded messages, for the demo.
    fn local_search(&self, query: &str) -> Vec<SearchHit> {
        let needle = query.to_lowercase();
        let mut hits: Vec<SearchHit> = self
            .conversations
            .iter()
            .flat_map(|(id, conversation)| {
                conversation
                    .messages
                    .iter()
                    .flat_map(|m| std::iter::once(m).chain(m.replies.iter()))
                    .filter(|m| !m.system && m.body.to_plain().to_lowercase().contains(&needle))
                    .map(move |m| SearchHit {
                        conversation: id.clone(),
                        message_id: m
                            .reply_to
                            .clone()
                            .filter(|_| id.is_channel())
                            .unwrap_or_else(|| m.id.clone()),
                        author: m.author.clone(),
                        snippet: m.body.summary(140),
                        at: m.created,
                    })
            })
            .collect();
        hits.sort_by_key(|hit| std::cmp::Reverse(hit.at));
        hits.truncate(50);
        hits
    }

    fn switch_demo(&mut self, demo: bool) {
        if self.demo == demo {
            return;
        }
        self.overlay = None;
        // Retire every account's backend off the interface thread.
        while !self.accounts.is_empty() {
            self.retire_slot(0);
        }
        self.active = 0;
        self.demo = demo;
        if demo {
            self.accounts.push(Account::new(Backend::detached().0));
            crate::demo::populate(self);
        } else {
            let ids = self.settings.accounts.clone();
            if ids.is_empty() {
                self.add_slot(None);
            }
            for id in ids {
                self.add_slot(Some(id));
            }
            self.active = 0;
        }
    }

    pub fn shutdown(&mut self) {
        self.save_settings();
        for account in &mut self.accounts {
            account.backend.shutdown();
        }
    }
}

/// What the chat list shows for a message: its text, or its files.
fn preview_text(message: &Message) -> String {
    let text = message.body.summary(140);
    if !text.is_empty() {
        return text;
    }
    let files: Vec<&str> = message
        .attachments
        .iter()
        .filter(|a| matches!(a.kind, crate::model::AttachmentKind::File))
        .filter_map(|a| a.name.as_deref())
        .collect();
    if files.is_empty() {
        String::new()
    } else {
        format!("📎 {}", files.join(", "))
    }
}

impl fastframe_shell::Resident for App {
    fn closed(&self) -> fastframe_shell::Closed {
        let tray_shown = self
            .tray
            .as_ref()
            .is_some_and(fastframe_tray::Tray::is_shown);
        if self.quit_requested || !self.settings.keep_running || !tray_shown || self.demo {
            fastframe_shell::Closed::Quit
        } else {
            fastframe_shell::Closed::Hide
        }
    }

    fn window_gone(&mut self) {
        self.window_hidden = true;
        self.window_focused = false;
        self.wants_show = false;
        self.applied_palette = None;
    }

    fn headless_frame(&mut self, ctx: &egui::Context) -> fastframe_shell::Headless {
        self.background(ctx);
        if self.quit_requested {
            fastframe_shell::Headless::Quit
        } else if self.wants_show {
            self.window_hidden = false;
            fastframe_shell::Headless::Show
        } else {
            fastframe_shell::Headless::Wait
        }
    }

    fn shutdown(&mut self) {
        App::shutdown(self);
    }
}

fn toggle_reaction(reactions: &mut Vec<Reaction>, emoji: &str, me: &str, on: bool) {
    // Teams keeps one reaction per person per emoji.
    match reactions.iter_mut().find(|r| r.emoji == emoji) {
        Some(reaction) => {
            reaction.users.retain(|u| u != me);
            if on {
                reaction.users.push(me.to_owned());
            }
        }
        None if on => reactions.push(Reaction {
            emoji: emoji.to_owned(),
            users: vec![me.to_owned()],
        }),
        None => {}
    }
    reactions.retain(|r| !r.users.is_empty());
}

fn open_external(url: &str) {
    if let Err(error) = open::that_detached(url) {
        log::warn!("could not open a link: {error}");
    }
}

/// Hands image bytes to egui under a URI with the right extension, so its
/// loaders decode them. Unknown formats are refused.
fn register_image(ctx: &egui::Context, key: &str, bytes: Vec<u8>) -> Option<String> {
    let extension = match bytes.get(..4)? {
        [0xFF, 0xD8, ..] => "jpg",
        [0x89, b'P', b'N', b'G'] => "png",
        [b'G', b'I', b'F', _] => "gif",
        _ => return None,
    };
    let uri = format!("bytes://wren/{key}.{extension}");
    ctx.include_bytes(uri.clone(), bytes);
    Some(uri)
}

fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::RichText;

    fn message(id: &str, created: i64) -> Message {
        Message {
            id: id.into(),
            created,
            body: RichText::plain(id),
            ..Default::default()
        }
    }

    fn app() -> (App, tokio::sync::mpsc::UnboundedReceiver<Command>) {
        let (backend, commands) = Backend::recording();
        let dir = std::env::temp_dir().join("wren-test-unused");
        let mut app = App::with_backend(
            AppDirs::under(&dir),
            Settings::default(),
            Waker::default(),
            backend,
            false,
        );
        app.auth = AuthStatus::SignedIn {
            account: "u-me.t1".into(),
            name: "Me".into(),
            username: "me@contoso.com".into(),
        };
        (app, commands)
    }

    #[test]
    fn latest_pages_connect_to_older_ones() {
        let mut conversation = Conversation::default();
        conversation.merge(
            (0..5).map(|i| message(&i.to_string(), i)).collect(),
            Merge::Latest,
        );
        conversation.merge(
            vec![message("3", 3), message("4", 4), message("5", 5)],
            Merge::Latest,
        );
        let ids: Vec<&str> = conversation
            .messages
            .iter()
            .map(|m| m.id.as_str())
            .collect();
        assert_eq!(ids, ["0", "1", "2", "3", "4", "5"]);
        // A page that does not connect replaces the gap-y history.
        conversation.merge(vec![message("9", 9)], Merge::Latest);
        assert_eq!(conversation.messages.len(), 1);
    }

    #[test]
    fn changes_update_in_place_and_older_pages_prepend() {
        let mut conversation = Conversation::default();
        conversation.merge(vec![message("5", 5), message("6", 6)], Merge::Latest);
        let mut edited = message("6", 6);
        edited.edited = true;
        conversation.merge(vec![edited, message("7", 7)], Merge::Changes);
        conversation.merge(vec![message("1", 1)], Merge::Older);
        let ids: Vec<&str> = conversation
            .messages
            .iter()
            .map(|m| m.id.as_str())
            .collect();
        assert_eq!(ids, ["1", "5", "6", "7"]);
        assert!(conversation.messages[2].edited);
    }

    #[test]
    fn sending_is_optimistic_and_reconciled() {
        let (mut app, mut commands) = app();
        let chat = ConversationId::Chat("19:a".into());
        let ctx = egui::Context::default();
        app.apply(
            Action::Send {
                conversation: chat.clone(),
                text: "**hi**".into(),
                reply_to: None,
                mentions: Vec::new(),
                files: Vec::new(),
            },
            &ctx,
        );
        let shown = &app.conversations[&chat].messages[0];
        assert_eq!(shown.delivery, Delivery::Sending);
        let Ok(Command::Send { local_id, html, .. }) = commands.try_recv() else {
            panic!("send command expected");
        };
        assert_eq!(html, "<p><strong>hi</strong></p>");
        let mut confirmed = message("1700", crate::time::now());
        confirmed.author_id = Some("u-me".into());
        app.handle(
            Event::Sent {
                conversation: chat.clone(),
                local_id,
                message: Box::new(confirmed),
            },
            &ctx,
        );
        let messages = &app.conversations[&chat].messages;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "1700");
        assert_eq!(messages[0].delivery, Delivery::Sent);
    }

    #[test]
    fn reactions_toggle_for_the_signed_in_person() {
        let mut reactions = Vec::new();
        toggle_reaction(&mut reactions, "👍", "me", true);
        toggle_reaction(&mut reactions, "👍", "ada", true);
        assert_eq!(reactions[0].users, ["me", "ada"]);
        toggle_reaction(&mut reactions, "👍", "me", false);
        toggle_reaction(&mut reactions, "👍", "ada", false);
        assert!(reactions.is_empty());
    }

    #[test]
    fn opening_a_chat_fetches_it_and_marks_it_read() {
        let (mut app, mut commands) = app();
        app.chats = vec![Chat {
            id: "19:a".into(),
            preview: Some(crate::model::Preview {
                message_id: "1".into(),
                author_id: Some("u-ada".into()),
                at: 10,
                ..Default::default()
            }),
            last_read: Some(1),
            ..Default::default()
        }];
        let ctx = egui::Context::default();
        app.apply(
            Action::OpenConversation(ConversationId::Chat("19:a".into())),
            &ctx,
        );
        let mut seen = Vec::new();
        while let Ok(command) = commands.try_recv() {
            seen.push(format!("{command:?}"));
        }
        assert!(
            seen.iter()
                .any(|c| c
                    .starts_with("Fetch { resource: Messages(Chat(\"19:a\")), trigger: Opened"))
        );
        assert!(seen.iter().any(|c| c == "MarkRead(\"19:a\")"));
        assert!(!app.chats[0].is_unread("u-me"));
    }

    #[test]
    fn only_teams_links_are_joined() {
        let (mut app, _commands) = app();
        let ctx = egui::Context::default();
        app.apply(Action::JoinMeeting("https://evil.example/x".into()), &ctx);
        assert_eq!(app.toasts.len(), 1);
    }

    #[test]
    fn mentions_and_files_travel_with_a_send() {
        let (mut app, mut commands) = app();
        let chat = ConversationId::Chat("19:a".into());
        app.chats = vec![Chat {
            id: "19:a".into(),
            members: vec![
                Member {
                    user_id: "u-me".into(),
                    display_name: "Me".into(),
                    email: Some("me@contoso.com".into()),
                },
                Member {
                    user_id: "u-ada".into(),
                    display_name: "Ada".into(),
                    email: Some("ada@contoso.com".into()),
                },
            ],
            ..Default::default()
        }];
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("plan.pdf");
        std::fs::write(&file, b"%PDF").unwrap();
        let ctx = egui::Context::default();
        app.apply(
            Action::Send {
                conversation: chat.clone(),
                text: "@Ada see the plan".into(),
                reply_to: None,
                mentions: vec![MentionRef {
                    user_id: "u-ada".into(),
                    name: "Ada".into(),
                }],
                files: vec![file.clone()],
            },
            &ctx,
        );
        let Ok(Command::Send {
            html,
            mentions,
            files,
            recipients,
            ..
        }) = commands.try_recv()
        else {
            panic!("send command expected");
        };
        assert_eq!(html, "<p><at id=\"0\">Ada</at> see the plan</p>");
        assert_eq!(mentions.len(), 1);
        assert_eq!(files, vec![file]);
        assert_eq!(recipients, vec!["ada@contoso.com".to_owned()]);
        let shown = &app.conversations[&chat].messages[0];
        assert_eq!(shown.attachments[0].name.as_deref(), Some("plan.pdf"));
    }

    #[test]
    fn features_needing_new_permissions_ask_first() {
        let (mut app, mut commands) = app();
        app.granted = "User.Read Chat.ReadWrite".into();
        let ctx = egui::Context::default();
        app.apply(Action::OpenNewChat, &ctx);
        assert_eq!(
            app.overlay,
            Some(Overlay::Consent(crate::auth::CHAT_CREATE_SCOPE))
        );
        app.apply(Action::SetAvailability(Some(Availability::Busy)), &ctx);
        assert_eq!(
            app.overlay,
            Some(Overlay::Consent(crate::auth::PRESENCE_WRITE_SCOPE))
        );
        app.apply(
            Action::RequestScope(crate::auth::PRESENCE_WRITE_SCOPE),
            &ctx,
        );
        let Ok(Command::SignIn { extra_scopes, .. }) = commands.try_recv() else {
            panic!("consent sign-in expected");
        };
        assert_eq!(extra_scopes, vec!["Presence.ReadWrite".to_owned()]);
        app.granted.push_str(" Presence.ReadWrite");
        app.apply(Action::SetAvailability(Some(Availability::Busy)), &ctx);
        assert!(matches!(
            commands.try_recv(),
            Ok(Command::SetAvailability(Some(Availability::Busy)))
        ));
        assert_eq!(
            app.my_presence.as_ref().unwrap().availability,
            Availability::Busy
        );
    }

    #[test]
    fn hidden_accounts_update_without_touching_the_screen() {
        let (mut app, _commands) = app();
        let (second, _second_commands) = Backend::recording();
        app.accounts.push(Account::new(second));
        app.accounts[1].auth = AuthStatus::SignedIn {
            account: "u-me2.t2".into(),
            name: "Me Too".into(),
            username: "me@fabrikam.com".into(),
        };
        app.accounts[1].open = Some(ConversationId::Chat("19:b".into()));
        let ctx = egui::Context::default();
        // An event for account 1 while account 0 shows.
        app.active = 1;
        app.hidden = true;
        app.handle(
            Event::Chats {
                chats: vec![Chat {
                    id: "19:b".into(),
                    ..Default::default()
                }],
                cached: false,
            },
            &ctx,
        );
        app.hidden = false;
        app.active = 0;
        assert!(app.chats.is_empty());
        assert_eq!(app.accounts[1].chats.len(), 1);
        app.apply(Action::SwitchAccount(1), &ctx);
        assert_eq!(app.active, 1);
        assert_eq!(app.chats[0].id, "19:b");
        assert_eq!(app.label(), "Me Too");
    }

    #[test]
    fn signing_in_twice_keeps_one_slot() {
        let (mut app, _commands) = app();
        app.accounts[0].id = Some("u-me.t1".into());
        let (second, _second_commands) = Backend::recording();
        app.accounts.push(Account::new(second));
        app.active = 1;
        let ctx = egui::Context::default();
        app.handle(
            Event::Auth(AuthStatus::SignedIn {
                account: "u-me.t1".into(),
                name: "Me".into(),
                username: "me@contoso.com".into(),
            }),
            &ctx,
        );
        app.retire_duplicates();
        app.apply_actions(&ctx);
        assert_eq!(app.accounts.len(), 1);
        assert_eq!(app.active, 0);
    }
}
