//! Calls in Teams on the web: opening them, docking Wren beside the call
//! window, and the call session behind the call bar (docs/calling.md).

use egui::{ViewportCommand, pos2, vec2};

use super::{App, Overlay};
use crate::backend::Command;
use crate::calling::browser::{self, Browser, BrowserProfile, Placement};
use crate::calling::dock;
use crate::calling::session::{CallSession, CallState, CallTarget, ChatLookup, in_call};
use crate::calling::{Opener, TEAMS_WEB, same_meeting};
use crate::model::{Action, Chat, ChatKind, ConversationId, Section, ToastKind};

/// Wren's window before it docked, to restore afterwards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Docked {
    position: Option<egui::Pos2>,
    inner: egui::Vec2,
    maximized: bool,
    /// Where the call window was asked to open.
    pub call: Placement,
}

/// The smallest window Wren allows, except while docked.
const MIN_SIZE: egui::Vec2 = vec2(720.0, 480.0);

impl App {
    /// Edge and Chrome, found once.
    pub fn call_browsers(&mut self) -> &[Browser] {
        self.browsers.get_or_insert_with(browser::discover)
    }

    /// How a call opens now, from the setting and what is installed.
    pub fn opener(&mut self) -> Opener {
        let open_in = self.settings.calls.open_in;
        let preferred = self.settings.calls.browser;
        Opener::resolve(open_in, preferred, self.call_browsers())
    }

    /// The browser profile for an account, when each account has its own.
    fn browser_profile(&self, index: usize) -> Option<String> {
        self.settings
            .calls
            .profile_per_account
            .then(|| self.accounts.get(index).map(|a| a.browser_profile.clone()))
            .flatten()
    }

    /// Tells each account's backend where Microsoft's sign-in page opens: the
    /// call browser, so Teams on the web finds the same Microsoft session.
    pub(super) fn sync_call_browser(&mut self) {
        let opener = self.opener();
        for index in 0..self.accounts.len() {
            let target = opener.sign_in_browser().map(|browser| BrowserProfile {
                browser: browser.clone(),
                profile: self.browser_profile(index),
            });
            self.accounts[index]
                .backend
                .send(Command::SignInBrowser(target));
        }
    }

    /// Remembers which browser profile belongs to a signed-in account.
    pub(super) fn remember_browser_profile(&mut self) {
        let Some(id) = self.id.clone() else { return };
        let mine = self.browser_profile.clone();
        let profile = self
            .settings
            .calls
            .profiles
            .entry(id)
            .or_insert(mine)
            .clone();
        if profile != self.browser_profile {
            self.browser_profile = profile;
        }
        self.mark_settings_dirty();
        if self.settings.calls.profile_per_account {
            self.sync_call_browser();
        }
    }

    pub(super) fn join_meeting(&mut self, url: String, ctx: &egui::Context) {
        if !crate::calling::is_teams_link(&url) {
            self.toast(ToastKind::Error, "That is not a Microsoft Teams link.");
            return;
        }
        let subject = self
            .meetings
            .iter()
            .find(|m| m.join_url.as_deref().is_some_and(|j| same_meeting(j, &url)))
            .map(|m| m.subject.clone())
            .or_else(|| {
                let me = self.me().to_owned();
                self.chats
                    .iter()
                    .find(|c| c.join_url.as_deref().is_some_and(|j| same_meeting(j, &url)))
                    .map(|c| c.title(&me))
            })
            .unwrap_or_default();
        let target = CallTarget::Meeting {
            subject,
            join_url: url.clone(),
        };
        self.open_call(target, url, None, ctx);
    }

    pub(super) fn start_call(&mut self, users: Vec<String>, video: bool, ctx: &egui::Context) {
        let Some(link) = crate::calling::call_link(&users, video) else {
            self.toast(
                ToastKind::Error,
                "No one to call: the chat's members have no email address.",
            );
            return;
        };
        // Started from a conversation: that is the call's chat. From the
        // Calls page: the one-on-one chat with that person, if any.
        let chat = if matches!(self.section, Section::Chats | Section::Teams) {
            self.open.clone()
        } else {
            self.one_on_one_with(&users)
        };
        let names = self.names_for(&users);
        self.open_call(CallTarget::People { names, video }, link, chat, ctx);
    }

    fn one_on_one_with(&self, users: &[String]) -> Option<ConversationId> {
        let [email] = users else { return None };
        let me = self.me();
        self.chats
            .iter()
            .filter(|c| c.kind == ChatKind::OneOnOne)
            .find(|c| {
                c.peer(me)
                    .and_then(|p| p.email.as_deref())
                    .is_some_and(|e| e.eq_ignore_ascii_case(email))
            })
            .map(|c| ConversationId::Chat(c.id.clone()))
    }

    fn names_for(&self, users: &[String]) -> String {
        let names: Vec<String> = users
            .iter()
            .map(|email| {
                self.chats
                    .iter()
                    .flat_map(|c| &c.members)
                    .find(|m| {
                        m.email
                            .as_deref()
                            .is_some_and(|e| e.eq_ignore_ascii_case(email))
                    })
                    .map(|m| m.display_name.clone())
                    .unwrap_or_else(|| email.clone())
            })
            .collect();
        match names.as_slice() {
            [] => String::new(),
            [one] => one.clone(),
            [first, rest @ ..] => format!("{first} and {} more", rest.len()),
        }
    }

    /// Opens a call or meeting link and starts following it.
    fn open_call(
        &mut self,
        target: CallTarget,
        url: String,
        chat: Option<ConversationId>,
        ctx: &egui::Context,
    ) {
        // The same meeting again: bring it forward rather than join twice.
        if let Some(call) = &self.call
            && !call.is_ended()
            && call.join_url().is_some_and(|open| same_meeting(open, &url))
        {
            self.show_call();
            return;
        }
        let opener = self.opener();
        // Dock first, so the call window opens where Wren was.
        let placement = match &opener {
            Opener::AppWindow(_) if self.settings.calls.dock => self.dock(ctx),
            _ => None,
        };
        let place = opener.place();
        if self.demo {
            self.toast(
                ToastKind::Info,
                format!("Demo: this opens Teams on the web in {place}."),
            );
        } else {
            let profile = self.browser_profile(self.active);
            let launched = match &opener {
                Opener::AppWindow(browser) => {
                    browser::app_window(browser, &url, placement, profile.as_deref())
                        .map(|launch| browser::spawn(&launch))
                }
                Opener::Browser | Opener::TeamsApp => None,
            };
            match launched {
                Some(Ok(())) => {}
                Some(Err(error)) => {
                    log::warn!("could not start the call browser: {error}");
                    super::open_external(&url);
                }
                None => super::open_external(&url),
            }
        }
        let now = crate::time::now();
        let mut session = CallSession::new(target, now, self.id.clone(), place);
        if self.demo {
            session.state = CallState::Live { since: now };
        }
        session.chat = chat.or_else(|| self.cached_meeting_chat(&session));
        if session.chat.is_some() {
            if session.is_meeting() {
                session.lookup = ChatLookup::Found;
            }
        } else if let Some(url) = session.join_url().map(str::to_owned) {
            session.lookup = if self.demo {
                ChatLookup::NotFound
            } else if self.has_scope(crate::auth::ONLINE_MEETINGS_SCOPE) {
                self.backend.send(Command::FindMeetingChat(url));
                ChatLookup::Searching
            } else {
                ChatLookup::NeedsPermission
            };
        }
        if let Some(chat) = session.chat.clone() {
            self.section = Section::Chats;
            self.actions.push(Action::OpenConversation(chat));
        }
        self.call = Some(session);
    }

    fn cached_meeting_chat(&self, session: &CallSession) -> Option<ConversationId> {
        let url = session.join_url()?;
        self.chats
            .iter()
            .find(|c| c.join_url.as_deref().is_some_and(|j| same_meeting(j, url)))
            .map(|c| ConversationId::Chat(c.id.clone()))
    }

    /// The person asked for the meeting's chat: look it up through Graph,
    /// asking for OnlineMeetings.Read first.
    pub(super) fn find_meeting_chat(&mut self) {
        let Some(session) = &self.call else { return };
        if let Some(chat) = self.cached_meeting_chat(session) {
            self.attach_chat(chat);
            return;
        }
        let Some(url) = session.join_url().map(str::to_owned) else {
            return;
        };
        if !self.ensure_scope(crate::auth::ONLINE_MEETINGS_SCOPE) {
            return;
        }
        self.backend.send(Command::FindMeetingChat(url));
        if let Some(call) = &mut self.call {
            call.lookup = ChatLookup::Searching;
        }
    }

    fn attach_chat(&mut self, chat: ConversationId) {
        if let Some(call) = &mut self.call {
            call.chat = Some(chat.clone());
            call.lookup = ChatLookup::Found;
        }
        if !self.hidden {
            self.section = Section::Chats;
            self.open_conversation(chat);
        }
    }

    pub(super) fn on_meeting_chat(&mut self, join_url: String, chat: Option<Box<Chat>>) {
        let ours = self
            .call
            .as_ref()
            .and_then(|c| c.join_url())
            .is_some_and(|open| same_meeting(open, &join_url));
        match chat {
            Some(chat) => {
                let chat = *chat;
                let id = chat.id.clone();
                self.chats.retain(|c| c.id != id);
                self.chats.insert(0, chat);
                if ours {
                    self.attach_chat(ConversationId::Chat(id));
                }
            }
            None if ours => {
                if let Some(call) = &mut self.call {
                    call.lookup = ChatLookup::NotFound;
                }
            }
            None => {}
        }
    }

    /// Follows the call from a reading of the person's own presence. A call
    /// Wren did not open shows too, but not a calendar entry alone.
    pub(super) fn observe_call(&mut self, activity: Option<&str>) {
        let now = crate::time::now();
        let id = self.id.clone();
        match &mut self.call {
            Some(call) if call.account == id => call.observe(activity, now),
            Some(_) => {}
            None if !self.hidden && !self.demo && in_call(activity, true) => {
                self.call = Some(CallSession::elsewhere(now, id));
            }
            None => {}
        }
    }

    /// Moves the call past its deadlines; once a frame.
    pub(super) fn tick_call(&mut self, ctx: &egui::Context) {
        let Some(call) = &mut self.call else { return };
        let now = crate::time::now();
        let next = call.advance(now);
        let ended = call.is_ended();
        if call.is_finished(now) {
            self.call = None;
        } else if let Some(ms) = next {
            ctx.request_repaint_after(std::time::Duration::from_millis(
                ms.clamp(250, 60_000) as u64
            ));
        }
        if ended && self.docked.is_some() {
            self.undock(ctx);
        }
    }

    pub(super) fn end_call(&mut self, ctx: &egui::Context) {
        self.call = None;
        self.undock(ctx);
    }

    /// Whether "Show call" can bring the call window forward here: on macOS,
    /// by activating the browser or Teams.
    pub fn can_show_call(&self) -> bool {
        use crate::calling::CallsOpenIn;
        let no_app_window = self.settings.calls.open_in == CallsOpenIn::AppWindow
            && self.browsers.as_deref().is_some_and(<[Browser]>::is_empty);
        cfg!(target_os = "macos")
            && !self.demo
            && self.settings.calls.open_in != CallsOpenIn::Browser
            && !no_app_window
    }

    /// Brings the browser (or Teams) forward. Never re-opens the link, which
    /// could join a second time.
    pub(super) fn show_call(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let bundle = match self.opener() {
                Opener::AppWindow(browser) => browser
                    .program
                    .ancestors()
                    .find(|p| p.extension().is_some_and(|e| e == "app"))
                    .map(|p| p.to_path_buf()),
                Opener::TeamsApp => Some("/Applications/Microsoft Teams.app".into()),
                Opener::Browser => None,
            };
            if let Some(bundle) = bundle {
                let launch = browser::Launch {
                    program: "/usr/bin/open".into(),
                    args: vec!["-a".into(), bundle.to_string_lossy().into_owned()],
                };
                if let Err(error) = browser::spawn(&launch) {
                    log::info!("could not bring the call forward: {error}");
                }
            }
        }
    }

    /// Moves Wren into a narrow panel on the right of its frame and returns
    /// where the call window should go, or `None` when it cannot dock
    /// (unknown position on Wayland, full screen, a small window).
    pub(super) fn dock(&mut self, ctx: &egui::Context) -> Option<Placement> {
        if let Some(docked) = self.docked {
            return Some(docked.call);
        }
        let (outer, inner, fullscreen, maximized) = ctx.input(|i| {
            let viewport = i.viewport();
            (
                viewport.outer_rect,
                viewport.inner_rect,
                viewport.fullscreen.unwrap_or(false),
                viewport.maximized.unwrap_or(false),
            )
        });
        let (Some(outer), Some(inner)) = (outer, inner) else {
            return None;
        };
        if fullscreen {
            return None;
        }
        // egui measures in zoomed points; browsers in screen points.
        let zoom = ctx.zoom_factor();
        let plan = dock::plan(dock::Frame {
            x: outer.min.x * zoom,
            y: outer.min.y * zoom,
            width: outer.width() * zoom,
            height: outer.height() * zoom,
        })?;
        let decorations = outer.size() - inner.size();
        let size = vec2(
            plan.wren.width / zoom - decorations.x,
            plan.wren.height / zoom - decorations.y,
        );
        self.docked = Some(Docked {
            position: Some(outer.min),
            inner: inner.size(),
            maximized,
            call: plan.call,
        });
        if maximized {
            ctx.send_viewport_cmd(ViewportCommand::Maximized(false));
        }
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(
            size.x.min(MIN_SIZE.x),
            size.y.min(MIN_SIZE.y),
        )));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos2(
            plan.wren.x / zoom,
            plan.wren.y / zoom,
        )));
        Some(plan.call)
    }

    /// Puts Wren's window back where it was before docking.
    pub(super) fn undock(&mut self, ctx: &egui::Context) {
        let Some(docked) = self.docked.take() else {
            return;
        };
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(docked.inner));
        if let Some(position) = docked.position {
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(position));
        }
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(MIN_SIZE));
        if docked.maximized {
            ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
        }
    }

    /// For tests and screenshots: the docked layout without moving a window.
    pub fn pretend_docked(&mut self) {
        self.docked = Some(Docked {
            position: None,
            inner: MIN_SIZE,
            maximized: false,
            call: Placement {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
        });
    }

    pub(super) fn join_on_phone(&mut self) {
        if let Some(url) = self.call.as_ref().and_then(|c| c.join_url()) {
            self.overlay = Some(Overlay::JoinOnPhone(url.to_owned()));
        }
    }

    /// Opens Teams on the web once in the call browser, to sign in there (and
    /// choose "Stay signed in") ahead of calls.
    pub(super) fn prepare_teams_web(&mut self) {
        let profile = self.browser_profile(self.active);
        let launched = match self.opener() {
            Opener::AppWindow(browser) => {
                browser::app_window(&browser, TEAMS_WEB, None, profile.as_deref())
                    .map(|launch| browser::spawn(&launch))
            }
            _ => None,
        };
        if !matches!(launched, Some(Ok(()))) {
            super::open_external(TEAMS_WEB);
        }
    }

    /// With the setting on, signing out of Wren also signs the call browser
    /// out of Microsoft (and so out of Teams on the web).
    pub(super) fn sign_out_browser(&mut self) {
        if !self.settings.calls.sign_out_browser || self.demo {
            return;
        }
        let Opener::AppWindow(browser) = self.opener() else {
            return;
        };
        let tenant = self.settings.tenant.trim().to_owned();
        let url = format!(
            "{}/{}/oauth2/v2.0/logout",
            crate::auth::DEFAULT_AUTHORITY,
            urlencoding::encode(if tenant.is_empty() {
                "organizations"
            } else {
                &tenant
            })
        );
        let profile = self.browser_profile(self.active);
        if let Some(launch) = browser::page(&browser, &url, profile.as_deref())
            && let Err(error) = browser::spawn(&launch)
        {
            log::info!("could not open the sign-out page: {error}");
        }
    }

    pub(super) fn set_call_settings(&mut self, calls: crate::settings::CallSettings) {
        if self.settings.calls == calls {
            return;
        }
        self.settings.calls = calls;
        self.mark_settings_dirty();
        self.sync_call_browser();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calling::session::GRACE;

    fn demo() -> (App, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let app = App::new(
            crate::paths::AppDirs::under(dir.path()),
            crate::settings::Settings::default(),
            crate::backend::Waker::default(),
            true,
        );
        (app, dir)
    }

    const UNKNOWN: &str =
        "https://teams.microsoft.com/l/meetup-join/19%3ameeting_elsewhere%40thread.v2/0";

    #[test]
    fn presence_ends_a_call_after_the_grace_period() {
        let (mut app, _dir) = demo();
        let ctx = egui::Context::default();
        app.join_meeting(UNKNOWN.into(), &ctx);
        app.pretend_docked();
        // The demo treats the call as live at once.
        app.observe_call(Some("Available"));
        let Some(call) = &mut app.call else {
            panic!("a call")
        };
        let CallState::Leaving { since, .. } = call.state else {
            panic!("leaving, not {:?}", call.state)
        };
        // A minute later presence still says the call is over.
        call.state = CallState::Leaving {
            since,
            left: crate::time::now() - GRACE - 1,
        };
        app.tick_call(&ctx);
        assert!(app.call.as_ref().is_some_and(CallSession::is_ended));
        assert!(
            app.docked.is_none(),
            "the window is restored when the call ends"
        );
    }

    #[test]
    fn calls_elsewhere_show_but_calendar_meetings_do_not() {
        let (mut app, _dir) = demo();
        app.demo = false;
        app.observe_call(Some("InAMeeting"));
        assert!(app.call.is_none());
        app.observe_call(Some("InACall"));
        assert_eq!(app.call.as_ref().unwrap().target, CallTarget::Elsewhere);
    }

    #[test]
    fn a_meeting_chat_found_later_is_shown_beside_the_call() {
        let (mut app, _dir) = demo();
        let ctx = egui::Context::default();
        app.join_meeting(UNKNOWN.into(), &ctx);
        assert_eq!(app.call.as_ref().unwrap().lookup, ChatLookup::NotFound);
        assert_eq!(app.call.as_ref().unwrap().title(), "Meeting");
        let chat = Chat {
            id: "19:meeting_elsewhere@thread.v2".into(),
            kind: ChatKind::Meeting,
            join_url: Some(UNKNOWN.into()),
            ..Default::default()
        };
        // Another meeting's answer is kept but not attached.
        app.on_meeting_chat(
            "https://teams.microsoft.com/l/meetup-join/other/0".into(),
            None,
        );
        assert_eq!(app.call.as_ref().unwrap().lookup, ChatLookup::NotFound);
        app.on_meeting_chat(UNKNOWN.into(), Some(Box::new(chat)));
        let call = app.call.as_ref().unwrap();
        assert_eq!(call.lookup, ChatLookup::Found);
        let id = ConversationId::Chat("19:meeting_elsewhere@thread.v2".into());
        assert_eq!(call.chat.as_ref(), Some(&id));
        assert_eq!(app.open.as_ref(), Some(&id));
        assert!(
            app.chats
                .iter()
                .any(|c| c.id == "19:meeting_elsewhere@thread.v2")
        );
    }

    #[test]
    fn calls_started_from_a_chat_follow_that_chat() {
        let (mut app, _dir) = demo();
        let ctx = egui::Context::default();
        let me = app.me().to_owned();
        let chat = app
            .chats
            .iter()
            .find(|c| {
                c.kind == ChatKind::OneOnOne && c.peer(&me).is_some_and(|p| p.email.is_some())
            })
            .cloned()
            .unwrap();
        let email = chat.peer(&me).and_then(|p| p.email.clone()).unwrap();
        let name = chat.peer(&me).map(|p| p.display_name.clone()).unwrap();
        // From the Calls page: the one-on-one chat is found by address.
        app.section = Section::Calls;
        app.start_call(vec![email], true, &ctx);
        let call = app.call.as_ref().unwrap();
        assert_eq!(call.title(), name);
        assert_eq!(call.chat, Some(ConversationId::Chat(chat.id.clone())));
        assert_eq!(call.lookup, ChatLookup::NotApplicable);
    }

    #[test]
    fn settings_reach_every_backend_and_profiles_stick_to_accounts() {
        let (mut app, _dir) = demo();
        app.id = Some("oid.tid".into());
        app.remember_browser_profile();
        let profile = app.browser_profile.clone();
        assert_eq!(app.settings.calls.profiles.get("oid.tid"), Some(&profile));
        assert!(app.browser_profile(0).is_none(), "off by default");
        let mut calls = app.settings.calls.clone();
        calls.profile_per_account = true;
        app.set_call_settings(calls);
        assert_eq!(app.browser_profile(0), Some(profile));
    }
}
