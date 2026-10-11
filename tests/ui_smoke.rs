//! Drives every screen of the demo through real egui frames, without a
//! window: layout must not panic, actions must apply, and each view must
//! draw something.

use wren::app::{App, Overlay};
use wren::backend::{AuthStatus, Waker};
use wren::model::{Action, ConversationId, Section};
use wren::model::{Availability, MentionRef};
use wren::paths::AppDirs;
use wren::settings::{Settings, ThemeChoice};

struct Harness {
    app: App,
    ctx: egui::Context,
    time: f64,
    _dir: tempfile::TempDir,
}

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let app = App::new(
            AppDirs::under(dir.path()),
            Settings::default(),
            Waker::default(),
            true,
        );
        let ctx = egui::Context::default();
        wren::ui::install(&ctx, true);
        Self {
            app,
            ctx,
            time: 0.0,
            _dir: dir,
        }
    }

    /// Runs one frame and returns how many shapes it painted.
    fn frame_with(&mut self, events: Vec<egui::Event>) -> usize {
        self.time += 1.0 / 60.0;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 820.0),
            )),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut output = self.ctx.run_ui(input, |ui| {
            let ctx = ui.ctx().clone();
            app.observe_window(&ctx);
            app.background(&ctx);
            wren::ui::show(app, ui);
            app.apply_actions(&ctx);
        });
        // No renderer here: drop the texture uploads on purpose.
        output.textures_delta.clear();
        output.shapes.len()
    }

    fn frames(&mut self, count: usize) -> usize {
        let mut shapes = 0;
        for _ in 0..count {
            shapes = self.frame_with(Vec::new());
        }
        shapes
    }

    fn act(&mut self, action: Action) {
        self.app.actions.push(action);
        self.frames(3);
    }
}

#[test]
fn every_screen_draws() {
    let mut h = Harness::new();
    assert!(h.frames(3) > 50, "the chat view draws");

    // Every chat, including group and meeting chats.
    let chats: Vec<String> = h.app.chats.iter().map(|c| c.id.clone()).collect();
    for chat in chats {
        h.act(Action::OpenConversation(ConversationId::Chat(chat)));
    }
    // Channels.
    let channels: Vec<ConversationId> = h
        .app
        .channels
        .values()
        .flatten()
        .map(|c| ConversationId::Channel {
            team: c.team_id.clone(),
            channel: c.id.clone(),
        })
        .collect();
    for channel in channels {
        h.act(Action::OpenConversation(channel));
    }
    assert_eq!(h.app.section, Section::Teams);
    for section in Section::ALL {
        h.act(Action::Navigate(section));
        assert!(h.frames(2) > 20, "{section:?} draws");
    }
    for overlay in [
        Action::OpenSwitcher,
        Action::OpenSettings,
        Action::OpenShortcuts,
    ] {
        h.act(overlay);
        assert!(h.app.overlay.is_some());
        h.act(Action::CloseOverlay);
    }
    for theme in [
        ThemeChoice::Light,
        ThemeChoice::Dark,
        ThemeChoice::Palette("Nord.json".into()),
    ] {
        h.act(Action::SetTheme(theme));
    }
}

#[test]
fn sending_reacting_and_editing_in_the_demo() {
    let mut h = Harness::new();
    h.frames(2);
    let chat = ConversationId::Chat("19:demo-ada@unq.gbl.spaces".into());
    h.act(Action::OpenConversation(chat.clone()));
    let before = h.app.conversation(&chat).unwrap().messages.len();
    h.act(Action::Send {
        conversation: chat.clone(),
        text: "Shipping **today**".into(),
        reply_to: None,
        mentions: Vec::new(),
        files: Vec::new(),
    });
    let conversation = h.app.conversation(&chat).unwrap();
    assert_eq!(conversation.messages.len(), before + 1);
    let sent = conversation.messages.last().unwrap().clone();
    assert_eq!(sent.body.to_plain(), "Shipping today");
    assert_eq!(
        h.app.chats[0].id, "19:demo-ada@unq.gbl.spaces",
        "the chat moves to the top"
    );

    h.act(Action::React {
        conversation: chat.clone(),
        message: sent.id.clone(),
        emoji: "🎉".into(),
        on: true,
    });
    assert_eq!(
        h.app
            .conversation(&chat)
            .unwrap()
            .find(&sent.id)
            .unwrap()
            .reactions
            .len(),
        1
    );

    h.act(Action::StartEdit(sent.id.clone()));
    assert_eq!(
        h.app.drafts[&chat].editing.as_deref(),
        Some(sent.id.as_str())
    );
    h.act(Action::Edit {
        conversation: chat.clone(),
        message: sent.id.clone(),
        text: "Shipping tomorrow".into(),
    });
    let edited = h.app.conversation(&chat).unwrap().find(&sent.id).unwrap();
    assert!(edited.edited);
    assert_eq!(edited.body.to_plain(), "Shipping tomorrow");

    h.act(Action::Delete {
        conversation: chat.clone(),
        message: sent.id.clone(),
    });
    assert!(
        h.app
            .conversation(&chat)
            .unwrap()
            .find(&sent.id)
            .unwrap()
            .deleted
    );
}

#[test]
fn channel_replies_land_in_their_thread() {
    let mut h = Harness::new();
    let channel = ConversationId::Channel {
        team: "t-eng".into(),
        channel: "19:eng-desktop@thread.tacv2".into(),
    };
    h.act(Action::OpenConversation(channel.clone()));
    let root = h.app.conversation(&channel).unwrap().messages[0].clone();
    let replies = root.replies.len();
    h.act(Action::StartReply(root.id.clone()));
    h.act(Action::Send {
        conversation: channel.clone(),
        text: "Agreed".into(),
        reply_to: Some(root.id.clone()),
        mentions: Vec::new(),
        files: Vec::new(),
    });
    let root = h
        .app
        .conversation(&channel)
        .unwrap()
        .find(&root.id)
        .unwrap();
    assert_eq!(root.replies.len(), replies + 1);
    assert!(
        h.app
            .conversation(&channel)
            .unwrap()
            .expanded
            .contains(&root.id)
    );
}

#[test]
fn switcher_finds_people_and_messages() {
    let mut h = Harness::new();
    h.act(Action::OpenSwitcher);
    assert_eq!(h.app.overlay, Some(Overlay::Switcher));
    h.act(Action::SearchMessages("relay".into()));
    assert!(
        !h.app.switcher.hits.is_empty(),
        "the demo searches loaded messages"
    );
    // Typing in the switcher and pressing Enter opens the best match.
    h.app.switcher.query = "grace".into();
    h.frame_with(vec![egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    h.frames(2);
    assert_eq!(
        h.app.open,
        Some(ConversationId::Chat("19:demo-grace@unq.gbl.spaces".into()))
    );
    assert_eq!(h.app.overlay, None);
}

#[test]
fn hovering_messages_shows_their_actions() {
    let mut h = Harness::new();
    h.frames(3);
    // Sweep the pointer over the conversation; the hover toolbar must lay
    // out wherever it lands.
    for y in (120..760).step_by(40) {
        h.frame_with(vec![egui::Event::PointerMoved(egui::pos2(900.0, y as f32))]);
    }
}

#[test]
fn the_welcome_screen_draws_without_an_account() {
    let mut h = Harness::new();
    h.app.demo = false;
    h.app.chats.clear();
    for status in [
        AuthStatus::NotConfigured,
        AuthStatus::SignedOut,
        AuthStatus::WaitingForBrowser {
            url: "https://login.microsoftonline.com/x".into(),
        },
        AuthStatus::DeviceCode {
            code: "ABCD-EFGH".into(),
            url: "https://microsoft.com/devicelogin".into(),
        },
        AuthStatus::Expired {
            username: "ada@contoso.com".into(),
            reason: "Sign in again.".into(),
        },
        AuthStatus::Failed("Microsoft refused the sign-in.".into()),
    ] {
        h.app.auth = status;
        assert!(!h.app.shows_workspace());
        assert!(h.frames(2) > 10);
    }
}

#[test]
fn large_conversations_stay_cheap() {
    let mut h = Harness::new();
    let chat = ConversationId::Chat("19:demo-ada@unq.gbl.spaces".into());
    let template = h.app.conversation(&chat).unwrap().messages[0].clone();
    let messages: Vec<_> = (0..5_000)
        .map(|i| {
            let mut m = template.clone();
            m.id = format!("bulk-{i}");
            m.created = template.created - (5_000 - i) * 60_000;
            m.author_id = Some(if i % 3 == 0 { "u-me" } else { "u-ada" }.into());
            m
        })
        .collect();
    h.app.conversations.get_mut(&chat).unwrap().messages = messages;
    h.act(Action::OpenConversation(chat));
    // The first frames estimate and measure; later frames reuse heights.
    h.frames(4);
    let started = std::time::Instant::now();
    h.frames(10);
    let per_frame = started.elapsed() / 10;
    // Generous bound for debug builds on CI: virtualization keeps this far
    // below laying out 5,000 rows.
    assert!(
        per_frame < std::time::Duration::from_millis(250),
        "{per_frame:?} per frame"
    );
}

#[test]
fn accounts_switch_and_keep_their_own_chats() {
    let mut h = Harness::new();
    h.frames(2);
    assert_eq!(h.app.accounts.len(), 2, "the demo has a second account");
    let first = h.app.chats[0].id.clone();
    h.act(Action::SwitchAccount(1));
    assert_eq!(h.app.active, 1);
    assert!(h.app.chats.iter().all(|c| c.id.starts_with("19:fabrikam")));
    assert!(h.frames(2) > 50, "the second account draws");
    h.act(Action::SwitchAccount(0));
    assert_eq!(h.app.chats[0].id, first);
    // The other account's unread chat shows as a dot on the avatar.
    assert!(h.app.other_unread() >= 1);
}

#[test]
fn starting_a_new_chat_from_the_directory() {
    let mut h = Harness::new();
    h.act(Action::OpenNewChat);
    assert_eq!(h.app.overlay, Some(Overlay::NewChat));
    h.app.new_chat.query = "barb".into();
    h.act(Action::SearchPeople("barb".into()));
    let barbara = h
        .app
        .people
        .results
        .iter()
        .find(|p| p.display_name == "Barbara Liskov")
        .cloned()
        .expect("found in the directory");
    h.frames(2);
    h.act(Action::CreateChat {
        members: vec![barbara],
        topic: None,
    });
    assert_eq!(h.app.overlay, None);
    let open = h.app.open.clone().expect("the new chat opens");
    assert_eq!(h.app.title_of(&open), "Barbara Liskov");
    h.act(Action::Send {
        conversation: open.clone(),
        text: "Hi Barbara".into(),
        reply_to: None,
        mentions: Vec::new(),
        files: Vec::new(),
    });
    assert_eq!(h.app.conversation(&open).unwrap().messages.len(), 1);
    // Starting a one-on-one with someone again opens the same chat.
    let chats = h.app.chats.len();
    let ada = h
        .app
        .directory()
        .into_iter()
        .find(|p| p.display_name == "Ada Lovelace")
        .unwrap();
    h.act(Action::CreateChat {
        members: vec![ada],
        topic: None,
    });
    assert_eq!(h.app.chats.len(), chats);
    assert_eq!(
        h.app.open,
        Some(ConversationId::Chat("19:demo-ada@unq.gbl.spaces".into()))
    );
}

#[test]
fn mentions_and_files_in_the_demo() {
    let mut h = Harness::new();
    let chat = ConversationId::Chat("19:demo-launch@thread.v2".into());
    h.act(Action::OpenConversation(chat.clone()));
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("checklist.pdf");
    std::fs::write(&file, b"%PDF-1.7").unwrap();
    h.act(Action::AddFiles(vec![
        file.clone(),
        dir.path().to_path_buf(),
    ]));
    assert_eq!(
        h.app.drafts[&chat].files,
        vec![file.clone()],
        "folders are refused"
    );
    assert!(h.frames(2) > 50, "the composer draws its file chips");
    h.act(Action::Send {
        conversation: chat.clone(),
        text: "@Grace Hopper here it is".into(),
        reply_to: None,
        mentions: vec![MentionRef {
            user_id: "u-grace".into(),
            name: "Grace Hopper".into(),
        }],
        files: vec![file],
    });
    let sent = h
        .app
        .conversation(&chat)
        .unwrap()
        .messages
        .last()
        .unwrap()
        .clone();
    assert_eq!(sent.attachments[0].name.as_deref(), Some("checklist.pdf"));
    let first = &sent.body.blocks[0];
    let wren::html::Block::Paragraph(spans) = first else {
        panic!("paragraph expected")
    };
    assert!(spans[0].style.mention && spans[0].text == "Grace Hopper");
    assert!(!h.app.drafts.contains_key(&chat));
}

#[test]
fn setting_status_and_message() {
    let mut h = Harness::new();
    h.act(Action::SetAvailability(Some(Availability::DoNotDisturb)));
    assert_eq!(
        h.app.my_presence.as_ref().unwrap().availability,
        Availability::DoNotDisturb
    );
    h.act(Action::OpenStatusMessage);
    assert_eq!(h.app.overlay, Some(Overlay::StatusMessage));
    h.frames(2);
    h.act(Action::SetStatusMessage("Back at 3 pm".into()));
    assert_eq!(
        h.app
            .my_presence
            .as_ref()
            .unwrap()
            .status_message
            .as_deref(),
        Some("Back at 3 pm")
    );
    let me = h.app.me().to_owned();
    assert_eq!(h.app.presence[&me].availability, Availability::DoNotDisturb);
    h.act(Action::SetAvailability(None));
    assert_eq!(
        h.app.my_presence.as_ref().unwrap().availability,
        Availability::Available
    );
}

#[test]
fn new_dialogs_draw() {
    let mut h = Harness::new();
    h.app.overlay = Some(Overlay::Consent(wren::auth::FILES_SCOPE));
    assert!(h.frames(2) > 50);
    h.app.overlay = Some(Overlay::NewChat);
    h.app.new_chat.query = "a".into();
    assert!(h.frames(2) > 50);
    h.app.overlay = Some(Overlay::StatusMessage);
    assert!(h.frames(2) > 50);
}

#[test]
fn joining_a_meeting_follows_it_beside_its_chat() {
    let mut h = Harness::new();
    h.frames(2);
    let url = h
        .app
        .meetings
        .iter()
        .find(|m| m.id == "demo-design-review")
        .and_then(|m| m.join_url.clone())
        .unwrap();
    h.app.section = Section::Calendar;
    h.act(Action::JoinMeeting(url.clone()));
    let call = h.app.call.clone().expect("a call session");
    assert_eq!(call.title(), "Design review: Wren 0.2");
    let chat = ConversationId::Chat("19:meeting_demo-design-review@thread.v2".into());
    assert_eq!(call.chat.as_ref(), Some(&chat), "the meeting chat is found");
    assert_eq!(h.app.open.as_ref(), Some(&chat));
    assert_eq!(h.app.section, Section::Chats);

    // Joining again brings the call forward instead of joining twice.
    h.act(Action::JoinMeeting(url));
    assert_eq!(h.app.call.as_ref().unwrap().opened_at, call.opened_at);

    // The bar along the bottom, then docked as a narrow panel.
    assert!(h.frames(2) > 50);
    h.app.pretend_docked();
    assert!(h.frames(2) > 30, "the docked panel draws");

    // The QR code to join on a phone.
    h.act(Action::JoinOnPhone);
    assert!(matches!(h.app.overlay, Some(Overlay::JoinOnPhone(_))));
    assert!(h.frames(2) > 200, "the QR code draws its modules");
    h.act(Action::CloseOverlay);

    h.act(Action::CallDone);
    assert!(h.app.call.is_none());
    assert!(h.app.docked.is_none(), "Done restores the window");
    assert!(h.frames(2) > 50);
}

#[test]
fn call_settings_draw_and_apply() {
    let mut h = Harness::new();
    h.act(Action::OpenSettings);
    assert!(h.frames(2) > 50);
    let mut calls = h.app.settings.calls.clone();
    assert_eq!(calls.open_in, wren::calling::CallsOpenIn::AppWindow);
    assert!(calls.dock);
    calls.open_in = wren::calling::CallsOpenIn::Browser;
    h.act(Action::SetCallSettings(calls));
    assert_eq!(
        h.app.settings.calls.open_in,
        wren::calling::CallsOpenIn::Browser
    );
    assert_eq!(h.app.opener(), wren::calling::Opener::Browser);
    // A call from the Calls page, with the bar along the bottom.
    h.act(Action::CloseOverlay);
    h.app.section = Section::Calls;
    h.act(Action::StartCall {
        users: vec!["ada@contoso.com".into()],
        video: false,
    });
    assert!(h.app.call.is_some());
    assert!(h.frames(2) > 50);
}
