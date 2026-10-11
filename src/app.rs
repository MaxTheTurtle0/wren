//! Application state and the one place it changes.
//!
//! Views in `ui/` read [`App`] and push [`Action`]s; [`App::apply`] runs
//! them after the frame. Backend [`Event`]s are applied at the start of each
//! frame. Nothing here waits on the network.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::backend::{AuthStatus, Backend, Command, Event, Merge, Waker};
use crate::html;
use crate::model::{
    Access, Action, Channel, Chat, ConversationId, Delivery, Meeting, Message, Presence, Profile,
    Reaction, SearchHit, Section, Team, Toast, ToastKind,
};
use crate::paths::AppDirs;
use crate::settings::Settings;
use crate::sync::{Resource, Trigger};
use crate::theme::Palette;

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Overlay {
    Switcher,
    Settings,
    Shortcuts,
}

#[derive(Debug, Default)]
pub struct Switcher {
    pub query: String,
    pub selected: usize,
    pub hits: Vec<SearchHit>,
    pub remote_hits: Vec<SearchHit>,
    pub searched: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Image {
    Loading,
    Ready(String),
    Missing,
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

pub struct App {
    pub dirs: AppDirs,
    pub settings: Settings,
    settings_dirty: bool,
    pub backend: Backend,
    pub waker: Waker,
    pub demo: bool,

    pub auth: AuthStatus,
    pub profile: Option<Profile>,
    pub granted: String,
    pub channel_access: Access,

    pub chats: Vec<Chat>,
    pub teams: Vec<Team>,
    pub channels: HashMap<String, Vec<Channel>>,
    pub conversations: HashMap<ConversationId, Conversation>,
    pub presence: HashMap<String, Presence>,
    pub meetings: Vec<Meeting>,
    pub photos: HashMap<String, Image>,
    pub media: HashMap<String, Image>,
    pub loading: HashSet<Resource>,

    pub section: Section,
    pub open: Option<ConversationId>,
    pub expanded_teams: HashSet<String>,
    pub selected_meeting: Option<String>,
    pub chat_filter: String,
    pub drafts: HashMap<ConversationId, Draft>,
    pub overlay: Option<Overlay>,
    pub switcher: Switcher,
    /// A message to scroll to once its conversation is drawn.
    pub jump_to: Option<String>,
    pub toasts: Vec<Toast>,
    next_toast: u64,
    pub actions: Vec<Action>,

    /// The registration form on the sign-in screen.
    pub form_client_id: String,
    pub form_tenant: String,

    pending_sends: HashMap<String, (ConversationId, String, Option<String>)>,
    next_local: u64,
    reminded: HashSet<String>,
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
}

impl App {
    pub fn new(dirs: AppDirs, settings: Settings, waker: Waker, demo: bool) -> Self {
        let backend = if demo {
            Backend::detached().0
        } else {
            Backend::spawn(dirs.clone(), waker.clone())
        };
        let mut app = Self::with_backend(dirs, settings, waker, backend, demo);
        if demo {
            crate::demo::populate(&mut app);
        } else {
            app.start_backend();
        }
        app
    }

    pub fn with_backend(
        dirs: AppDirs,
        settings: Settings,
        waker: Waker,
        backend: Backend,
        demo: bool,
    ) -> Self {
        let palette = crate::theme::resolve(&settings.theme, true);
        Self {
            form_client_id: settings.client_id.clone(),
            form_tenant: settings.tenant.clone(),
            dirs,
            settings,
            settings_dirty: false,
            backend,
            waker,
            demo,
            auth: AuthStatus::Restoring,
            profile: None,
            granted: String::new(),
            channel_access: Access::Unknown,
            chats: Vec::new(),
            teams: Vec::new(),
            channels: HashMap::new(),
            conversations: HashMap::new(),
            presence: HashMap::new(),
            meetings: Vec::new(),
            photos: HashMap::new(),
            media: HashMap::new(),
            loading: HashSet::new(),
            section: Section::Chats,
            open: None,
            expanded_teams: HashSet::new(),
            selected_meeting: None,
            chat_filter: String::new(),
            drafts: HashMap::new(),
            overlay: None,
            switcher: Switcher::default(),
            jump_to: None,
            toasts: Vec::new(),
            next_toast: 0,
            actions: Vec::new(),
            pending_sends: HashMap::new(),
            next_local: 0,
            reminded: HashSet::new(),
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
        }
    }

    fn start_backend(&mut self) {
        self.auth = if self.settings.has_app_registration() {
            AuthStatus::Restoring
        } else {
            AuthStatus::NotConfigured
        };
        self.backend.send(Command::Start {
            client_id: self.settings.client_id.clone(),
            tenant: self.settings.tenant.clone(),
            account: self.settings.account.clone(),
        });
    }

    /// The signed-in person's id (Graph user id).
    pub fn me(&self) -> &str {
        if let Some(profile) = &self.profile {
            return &profile.id;
        }
        match &self.auth {
            AuthStatus::SignedIn { account, .. } => account.split('.').next().unwrap_or_default(),
            _ => "",
        }
    }

    pub fn is_signed_in(&self) -> bool {
        self.demo || matches!(self.auth, AuthStatus::SignedIn { .. })
    }

    /// Whether the main interface (rather than the sign-in screen) shows.
    pub fn shows_workspace(&self) -> bool {
        self.is_signed_in()
            || (matches!(
                self.auth,
                AuthStatus::Restoring | AuthStatus::Expired { .. }
            ) && !self.chats.is_empty())
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

    pub fn has_scope(&self, scope: &str) -> bool {
        self.demo || crate::auth::has_scope(&self.granted, scope)
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

    pub fn toast(&mut self, kind: ToastKind, text: impl Into<String>) {
        let text = text.into();
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

    // -- Frame -----------------------------------------------------------------

    /// Work every frame does, with or without a window: backend events,
    /// tray clicks, reminders.
    pub fn background(&mut self, ctx: &egui::Context) {
        for event in self.backend.poll() {
            self.handle(event, ctx);
        }
        self.handle_tray();
        self.handle_requests();
        self.remind(ctx);
        self.apply_actions(ctx);
        self.save_settings();
    }

    /// Tracks focus; coming back to the window refreshes what it shows.
    pub fn observe_window(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if focused && !self.window_focused {
            self.fetch_view(Trigger::Focused);
            if let Some(ConversationId::Chat(chat)) = self.open.clone() {
                self.mark_read(&chat);
            }
        }
        self.window_focused = focused;
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

    /// Meeting reminders from the cached calendar: no network involved.
    fn remind(&mut self, ctx: &egui::Context) {
        if !self.settings.notifications {
            return;
        }
        let now = crate::time::now();
        let lead = 5 * 60 * 1000;
        let mut next: Option<i64> = None;
        let mut due = Vec::new();
        for meeting in &self.meetings {
            if meeting.cancelled || meeting.all_day || self.reminded.contains(&meeting.id) {
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
            self.reminded.insert(meeting.id.clone());
            if !self.demo {
                crate::notify::show(
                    &meeting.subject,
                    &format!("Starts {}", crate::time::until_label(meeting.start, now)),
                );
            }
        }
        if let Some(next) = next {
            ctx.request_repaint_after(Duration::from_millis(
                (next - now).clamp(1_000, 3_600_000) as u64
            ));
        }
    }

    // -- Events ------------------------------------------------------------------

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
                    self.expanded_teams.insert(first.id.clone());
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
                if query == self.switcher.searched {
                    if remote {
                        self.switcher.remote_hits = hits;
                    } else {
                        self.switcher.hits = hits;
                    }
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
        }
    }

    fn on_auth(&mut self, status: AuthStatus) {
        log::info!("account: {}", status.log_label());
        let became_signed_in = matches!(status, AuthStatus::SignedIn { .. })
            && !matches!(self.auth, AuthStatus::SignedIn { .. });
        match &status {
            AuthStatus::SignedIn { account, .. } => {
                if self.settings.account.as_deref() != Some(account) {
                    self.settings.account = Some(account.clone());
                    self.mark_settings_dirty();
                }
            }
            AuthStatus::SignedOut => {
                self.clear_account_data();
            }
            AuthStatus::Expired { username, reason } => {
                // Keep the name the account had when Graph does not repeat it.
                if username.is_empty()
                    && let AuthStatus::SignedIn {
                        username: known, ..
                    } = &self.auth
                {
                    self.auth = AuthStatus::Expired {
                        username: known.clone(),
                        reason: reason.clone(),
                    };
                    return;
                }
            }
            _ => {}
        }
        self.auth = status;
        if became_signed_in {
            for resource in [
                Resource::Profile,
                Resource::Chats,
                Resource::Teams,
                Resource::Calendar,
            ] {
                self.fetch(resource, Trigger::Start);
            }
            if let Some(open) = &self.open {
                self.fetch(Resource::Messages(open.clone()), Trigger::Start);
            }
        }
    }

    fn clear_account_data(&mut self) {
        self.profile = None;
        self.granted.clear();
        self.chats.clear();
        self.teams.clear();
        self.channels.clear();
        self.conversations.clear();
        self.presence.clear();
        self.meetings.clear();
        self.photos.clear();
        self.media.clear();
        self.drafts.clear();
        self.open = None;
        if self.settings.account.take().is_some() {
            self.mark_settings_dirty();
        }
    }

    fn on_chats(&mut self, mut chats: Vec<Chat>, cached: bool) {
        chats.sort_by_key(|chat| std::cmp::Reverse(chat.activity()));
        let me = self.me().to_owned();
        if !cached && !self.chats.is_empty() {
            for chat in &chats {
                let Some(preview) = &chat.preview else {
                    continue;
                };
                let previous = self.chat(&chat.id).and_then(|c| c.preview.as_ref());
                let new = previous
                    .is_none_or(|p| p.message_id != preview.message_id && preview.at > p.at);
                let looking =
                    self.window_focused && self.open == Some(ConversationId::Chat(chat.id.clone()));
                if new
                    && !looking
                    && preview.author_id.as_deref() != Some(me.as_str())
                    && !preview.deleted
                {
                    self.notify_message(&chat.title(&me), preview.author.as_deref(), &preview.text);
                }
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
        crate::notify::show(title, &body);
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
        let open = self.open.as_ref() == Some(&conversation);
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
        if !cached && !(open && self.window_focused) {
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
                text: last.body.summary(140),
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
        let me = self.me().to_owned();
        let Some(entry) = self.chats.iter_mut().find(|c| c.id == chat) else {
            return;
        };
        if !entry.is_unread(&me) {
            return;
        }
        entry.last_read = Some(crate::time::now());
        if !self.demo {
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
                        (Section::Chats, Some(open @ ConversationId::Chat(_))) => Some(open.clone()),
                        (Section::Teams, Some(open @ ConversationId::Channel { .. })) => Some(open.clone()),
                        _ => None,
                    };
                }
                self.fetch_view(Trigger::Opened);
            }
            Action::OpenConversation(conversation) => self.open_conversation(conversation),
            Action::OpenMessage { conversation, message } => {
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
            } => self.send(conversation, text, reply_to),
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
            Action::Delete { conversation, message } => {
                if let Some(entry) = self.conversations.get_mut(&conversation)
                    && let Some(found) = entry.find_mut(&message)
                {
                    found.deleted = true;
                    found.body = html::RichText::default();
                }
                if !self.demo {
                    self.backend.send(Command::Delete { conversation, message });
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
                            reply_to: None,
                            editing: Some(message),
                        },
                    );
                    self.focus_composer = true;
                }
            }
            Action::CancelCompose => {
                if let Some(open) = &self.open
                    && let Some(draft) = self.drafts.get_mut(open)
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
            Action::LoadReplies { conversation, message } => {
                if !self.demo {
                    self.backend.send(Command::LoadReplies {
                        conversation,
                        root: message,
                    });
                }
            }
            Action::ToggleThread(root) => {
                if let Some(open) = self.open.clone() {
                    let entry = self.conversations.entry(open.clone()).or_default();
                    if !entry.expanded.remove(&root) {
                        entry.expanded.insert(root.clone());
                    }
                }
            }
            Action::RetrySend { conversation, local_id } => {
                if let Some((_, html, reply_to)) = self.pending_sends.get(&local_id).cloned() {
                    if let Some(entry) = self.conversations.get_mut(&conversation)
                        && let Some(found) = entry.find_mut(&local_id)
                    {
                        found.delivery = Delivery::Sending;
                    }
                    self.backend.send(Command::Send {
                        conversation,
                        local_id,
                        html,
                        reply_to,
                    });
                }
            }
            Action::AttachFiles => self.toast(
                ToastKind::Info,
                "Sending files is not supported yet. It needs OneDrive upload (Files.ReadWrite), planned next.",
            ),
            Action::LoadMedia(url) => {
                if !self.media.contains_key(&url) {
                    self.media.insert(url.clone(), Image::Loading);
                    if self.demo {
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
            Action::SignOut => {
                self.backend.send(Command::SignOut);
                self.clear_account_data();
                self.auth = AuthStatus::SignedOut;
                self.overlay = None;
            }
            Action::RequestScope(scope) => {
                if !self.demo {
                    self.backend.send(Command::SignIn {
                        extra_scopes: vec![scope.to_owned()],
                        device_code: false,
                    });
                }
            }
            Action::StartDemo => self.switch_demo(true),
            Action::LeaveDemo => self.switch_demo(false),
            Action::SaveAppRegistration { client_id, tenant } => {
                let client_id = client_id.trim().to_owned();
                if !crate::settings::is_guid(&client_id) {
                    self.toast(ToastKind::Error, "The application (client) ID is a GUID like 00000000-0000-0000-0000-000000000000.");
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
                    self.start_backend();
                }
            }
            Action::SetPresence(_) => {}
            Action::JoinMeeting(url) => {
                if crate::calling::is_teams_link(&url) {
                    open_external(&url);
                } else {
                    self.toast(ToastKind::Error, "That is not a Microsoft Teams link.");
                }
            }
            Action::StartCall { users, video } => match crate::calling::call_link(&users, video) {
                Some(link) => open_external(&link),
                None => self.toast(ToastKind::Error, "No one to call: the chat's members have no email address."),
            },
            Action::OpenUrl(url) => {
                if url.starts_with("https://") || url.starts_with("http://") || url.starts_with("mailto:") {
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

    fn send(&mut self, conversation: ConversationId, text: String, reply_to: Option<String>) {
        let text = text.trim_end().to_owned();
        if text.trim().is_empty() {
            return;
        }
        let html = html::compose(&text);
        self.next_local += 1;
        let local_id = format!("local-{}", self.next_local);
        let me = self.me().to_owned();
        let author = self
            .profile
            .as_ref()
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| "You".into());
        let channel_reply = conversation.is_channel() && reply_to.is_some();
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
            delivery: if self.demo {
                Delivery::Sent
            } else {
                Delivery::Sending
            },
            ..Default::default()
        };
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
        if self.demo {
            message.id = format!("demo-{}", self.next_local);
        }
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
                text: message.body.summary(140),
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
        if !self.demo {
            self.pending_sends.insert(
                local_id.clone(),
                (conversation.clone(), html.clone(), reply_to.clone()),
            );
            self.backend.send(Command::Send {
                conversation,
                local_id,
                html,
                reply_to,
            });
        }
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
        self.clear_account_data();
        self.overlay = None;
        self.demo = demo;
        self.backend = if demo {
            Backend::detached().0
        } else {
            Backend::spawn(self.dirs.clone(), self.waker.clone())
        };
        if demo {
            crate::demo::populate(self);
        } else {
            self.auth = AuthStatus::Restoring;
            self.start_backend();
        }
    }

    pub fn shutdown(&mut self) {
        self.save_settings();
        self.backend.shutdown();
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
}
