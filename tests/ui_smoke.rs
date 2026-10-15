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
    size: egui::Vec2,
    shapes: Vec<egui::epaint::ClippedShape>,
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
            size: egui::vec2(1280.0, 820.0),
            shapes: Vec::new(),
            _dir: dir,
        }
    }

    /// Runs one frame and returns how many shapes it painted.
    fn frame_with(&mut self, events: Vec<egui::Event>) -> usize {
        self.time += 1.0 / 60.0;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.size)),
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
        let count = output.shapes.len();
        self.shapes = output.shapes;
        count
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

    fn wait_for_images(&mut self) {
        self.frames(1);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while self.ctx.has_pending_images() {
            assert!(
                std::time::Instant::now() < deadline,
                "image loader did not finish"
            );
            std::thread::sleep(std::time::Duration::from_millis(2));
            self.frames(1);
        }
        self.frames(2);
    }

    fn click(&mut self, pos: egui::Pos2) {
        self.frame_with(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        self.frame_with(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        self.frames(3);
    }
}

fn painted_shapes(h: &Harness) -> Vec<&egui::epaint::Shape> {
    fn collect<'a>(shape: &'a egui::epaint::Shape, output: &mut Vec<&'a egui::epaint::Shape>) {
        if let egui::epaint::Shape::Vec(shapes) = shape {
            for shape in shapes {
                collect(shape, output);
            }
        } else {
            output.push(shape);
        }
    }
    let mut shapes = Vec::new();
    for clipped in &h.shapes {
        collect(&clipped.shape, &mut shapes);
    }
    shapes
}

#[test]
fn composer_placeholder_is_vertically_centered_in_both_themes_and_widths() {
    for width in [720.0, 1280.0] {
        for theme in [ThemeChoice::Dark, ThemeChoice::Light] {
            let mut h = Harness::new();
            h.size.x = width;
            h.app.settings.theme = theme;
            h.frames(4);
            let shapes = painted_shapes(&h);
            let text = shapes
                .iter()
                .find_map(|shape| match shape {
                    egui::epaint::Shape::Text(text)
                        if text.galley.job.text.starts_with("Message ") =>
                    {
                        Some(text)
                    }
                    _ => None,
                })
                .expect("composer placeholder is painted");
            let text_rect = egui::Rect::from_min_size(text.pos, text.galley.size());
            let frame = shapes
                .iter()
                .filter_map(|shape| match shape {
                    egui::epaint::Shape::Rect(rect)
                        if rect.fill == h.app.palette.surface
                            && rect.rect.contains(text_rect.center()) =>
                    {
                        Some(rect.rect)
                    }
                    _ => None,
                })
                .min_by(|a, b| a.area().total_cmp(&b.area()))
                .expect("composer surface is painted");
            assert!(
                (text_rect.center().y - frame.center().y).abs() < 2.0,
                "placeholder center {:?}, composer {:?}, width {width}",
                text_rect,
                frame
            );
        }
    }
}

#[test]
fn pasted_image_preview_and_image_only_send_draw() {
    let mut h = Harness::new();
    h.frames(3);
    let chat = h.app.open.clone().unwrap();
    let image = wren::clipboard::encode_rgba(2, 2, &[0, 128, 255, 255].repeat(4)).unwrap();
    h.app.handle(
        wren::backend::Event::Pasted {
            conversation: chat.clone(),
            range: (0, 0),
            text: String::new(),
            image: Ok(Some(image)),
        },
        &h.ctx,
    );
    assert!(h.frames(3) > 50);
    let images = h.app.drafts[&chat].images.clone();
    h.act(Action::Send {
        conversation: chat.clone(),
        text: String::new(),
        reply_to: None,
        mentions: Vec::new(),
        files: Vec::new(),
        images,
    });
    assert_eq!(
        h.app.conversations[&chat]
            .messages
            .last()
            .unwrap()
            .body
            .images()
            .count(),
        1
    );
    assert!(h.frames(2) > 50);
}

fn image_shapes<'a>(
    h: &'a Harness,
    image: &wren::clipboard::InlineImage,
) -> Vec<&'a egui::epaint::RectShape> {
    cached_image_shapes(h, &image.uri())
}

fn cached_image_shapes<'a>(h: &'a Harness, uri: &str) -> Vec<&'a egui::epaint::RectShape> {
    let id = egui::Image::new(uri)
        .load_for_size(&h.ctx, h.size)
        .unwrap()
        .texture_id()
        .unwrap();
    painted_shapes(h)
        .into_iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Rect(rect) if rect.fill_texture_id() == id => Some(rect),
            _ => None,
        })
        .collect()
}

#[test]
fn clicking_received_images_opens_the_shared_viewer_at_their_original_size() {
    for width in [720.0, 1280.0] {
        for theme in [ThemeChoice::Dark, ThemeChoice::Light] {
            for format in [image::ImageFormat::Png, image::ImageFormat::Jpeg] {
                let mut h = Harness::new();
                h.size.x = width;
                h.app.settings.theme = theme.clone();
                h.frames(3);
                let chat = h.app.open.clone().unwrap();
                let url = "https://graph.microsoft.com/v1.0/chats/fixture/messages/1/hostedContents/1/$value";
                let mut bytes = std::io::Cursor::new(Vec::new());
                image::RgbImage::from_pixel(1200, 800, image::Rgb([0, 128, 255]))
                    .write_to(&mut bytes, format)
                    .unwrap();
                let bytes = bytes.into_inner();
                h.app.handle(
                    wren::backend::Event::Media {
                        url: url.into(),
                        bytes: Some(bytes.clone()),
                    },
                    &h.ctx,
                );
                let wren::app::Image::Ready(uri) = h.app.media[url].clone() else {
                    panic!("fixture image is registered");
                };
                let message = h
                    .app
                    .conversations
                    .get_mut(&chat)
                    .unwrap()
                    .messages
                    .last_mut()
                    .unwrap();
                message.body = wren::html::parse(&format!(
                    "<img src=\"{url}\" alt=\"Bild\" width=\"120\" height=\"80\">"
                ));
                let count = h.app.conversations[&chat].messages.len();
                h.wait_for_images();
                let thumbnail = cached_image_shapes(&h, &uri)[0].rect;
                assert_eq!(thumbnail.size(), egui::vec2(120.0, 80.0));
                h.click(thumbnail.center());
                assert_eq!(h.app.overlay, Some(Overlay::ImagePreview(uri.clone())));
                let zoom = painted_shapes(&h)
                    .into_iter()
                    .find_map(|shape| match shape {
                        egui::epaint::Shape::Text(text) if text.galley.job.text == "100%" => {
                            Some(text.pos + text.galley.size() / 2.0)
                        }
                        _ => None,
                    })
                    .unwrap();
                h.click(zoom);
                assert_eq!(
                    cached_image_shapes(&h, &uri).last().unwrap().rect.size(),
                    egui::vec2(1200.0, 800.0)
                );
                assert!(painted_shapes(&h).iter().any(|shape| matches!(shape, egui::epaint::Shape::Text(text) if text.galley.job.text == "1200 × 800")));
                h.frame_with(vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }]);
                h.frames(3);
                assert!(h.app.overlay.is_none());
                assert_eq!(h.app.conversations[&chat].messages.len(), count);
                let egui::load::BytesPoll::Ready { bytes: cached, .. } =
                    h.ctx.try_load_bytes(&uri).unwrap()
                else {
                    panic!("original bytes remain cached");
                };
                assert_eq!(
                    cached.as_ref(),
                    bytes.as_slice(),
                    "preview reuses the original PNG/JPEG without re-encoding or fetching"
                );
            }
        }
    }
}

#[test]
fn clicking_an_optimistic_image_message_opens_the_same_viewer_as_its_draft() {
    let mut h = Harness::new();
    h.frames(3);
    let chat = h.app.open.clone().unwrap();
    let image =
        wren::clipboard::encode_rgba(640, 480, &[0, 128, 255, 255].repeat(640 * 480)).unwrap();
    h.app.handle(
        wren::backend::Event::Pasted {
            conversation: chat.clone(),
            range: (0, 0),
            text: String::new(),
            image: Ok(Some(image.clone())),
        },
        &h.ctx,
    );
    h.wait_for_images();
    h.act(Action::Send {
        conversation: chat.clone(),
        text: String::new(),
        reply_to: None,
        mentions: Vec::new(),
        files: Vec::new(),
        images: vec![image.clone()],
    });
    h.wait_for_images();
    let thumbnail = image_shapes(&h, &image)[0].rect;
    h.click(thumbnail.center());
    assert_eq!(h.app.overlay, Some(Overlay::ImagePreview(image.uri())));
}

fn image_clips(h: &Harness, image: &wren::clipboard::InlineImage) -> Vec<(egui::Rect, egui::Rect)> {
    fn collect(
        shape: &egui::epaint::Shape,
        clip: egui::Rect,
        texture: egui::TextureId,
        output: &mut Vec<(egui::Rect, egui::Rect)>,
    ) {
        match shape {
            egui::epaint::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, clip, texture, output);
                }
            }
            egui::epaint::Shape::Rect(rect) if rect.fill_texture_id() == texture => {
                output.push((rect.rect, clip))
            }
            _ => {}
        }
    }
    let texture = image_shapes(h, image)[0].fill_texture_id();
    let mut output = Vec::new();
    for clipped in &h.shapes {
        collect(&clipped.shape, clipped.clip_rect, texture, &mut output);
    }
    output
}

#[test]
fn composer_images_have_usable_thumbnails_and_open_full_previews() {
    for width in [720.0, 1280.0] {
        for theme in [ThemeChoice::Dark, ThemeChoice::Light] {
            for (w, height) in [(2000, 20), (20, 2000), (320, 240)] {
                let mut h = Harness::new();
                h.size.x = width;
                h.app.settings.theme = theme.clone();
                h.frames(3);
                let chat = h.app.open.clone().unwrap();
                let image =
                    wren::clipboard::encode_rgba(w, height, &[0, 128, 255, 255].repeat(w * height))
                        .unwrap();
                h.app.handle(
                    wren::backend::Event::Pasted {
                        conversation: chat.clone(),
                        range: (0, 0),
                        text: String::new(),
                        image: Ok(Some(image.clone())),
                    },
                    &h.ctx,
                );
                h.frames(4);
                h.wait_for_images();
                let thumbnail = image_shapes(&h, &image)[0];
                let rect = thumbnail.rect;
                assert_eq!(rect.size(), egui::vec2(144.0, 96.0));
                let uv = thumbnail.brush.as_ref().unwrap().uv;
                let cropped_aspect = uv.width() * w as f32 / (uv.height() * height as f32);
                assert!(
                    (cropped_aspect - 1.5).abs() < 0.001,
                    "thumbnail preserves proportions"
                );
                assert!(uv.min.x >= 0.0 && uv.min.y >= 0.0 && uv.max.x <= 1.0 && uv.max.y <= 1.0);

                h.click(rect.center());
                assert!(
                    matches!(&h.app.overlay, Some(Overlay::ImagePreview(shown)) if shown == &image.uri())
                );
                let fitted = image_shapes(&h, &image).last().unwrap().rect;
                // Painting snaps to pixels, which matters for a very thin
                // fitted image. Allow one pixel of rounding, not stretching.
                assert!(
                    (fitted.width() * height as f32 - fitted.height() * w as f32).abs()
                        <= w.max(height) as f32
                );
                assert!(egui::Rect::from_min_size(egui::Pos2::ZERO, h.size).contains_rect(fitted));
                let (painted, clip) = *image_clips(&h, &image).last().unwrap();
                assert!(
                    clip.expand(1.0).contains_rect(painted),
                    "the fitted preview must be painted inside its scroll viewport"
                );
                let zoom = painted_shapes(&h)
                    .into_iter()
                    .find_map(|shape| match shape {
                        egui::epaint::Shape::Text(text) if text.galley.job.text == "100%" => {
                            Some(text.pos + text.galley.size() / 2.0)
                        }
                        _ => None,
                    })
                    .unwrap();
                h.click(zoom);
                assert_eq!(
                    image_shapes(&h, &image).last().unwrap().rect.size(),
                    egui::vec2(w as f32, height as f32)
                );
                let count = h.app.conversations[&chat].messages.len();
                h.frame_with(vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }]);
                assert_eq!(
                    h.app.conversations[&chat].messages.len(),
                    count,
                    "preview must not send the draft"
                );
                h.frame_with(vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }]);
                h.frames(3);
                assert!(h.app.overlay.is_none());
                assert_eq!(h.app.drafts[&chat].images, vec![image.clone()]);
                let thumbnail = image_shapes(&h, &image)[0].rect;
                let remove = egui::pos2(
                    thumbnail.right() + h.ctx.global_style().spacing.item_spacing.x + 11.0,
                    thumbnail.center().y,
                );
                h.click(remove);
                assert!(
                    h.app
                        .drafts
                        .get(&chat)
                        .is_none_or(|draft| draft.images.is_empty())
                );
                assert!(
                    h.app.overlay.is_none(),
                    "removing an image must not open its preview"
                );
            }
        }
    }
}

#[test]
fn multiple_pasted_images_wrap_without_clipping() {
    for width in [720.0, 1280.0] {
        for theme in [ThemeChoice::Light, ThemeChoice::Dark] {
            let mut h = Harness::new();
            h.size.x = width;
            h.app.settings.theme = theme;
            h.frames(3);
            let chat = h.app.open.clone().unwrap();
            for (w, height) in [(1200, 24), (24, 1200), (256, 256)] {
                let image =
                    wren::clipboard::encode_rgba(w, height, &[0, 128, 255, 255].repeat(w * height))
                        .unwrap();
                h.app.handle(
                    wren::backend::Event::Pasted {
                        conversation: chat.clone(),
                        range: (0, 0),
                        text: String::new(),
                        image: Ok(Some(image)),
                    },
                    &h.ctx,
                );
            }
            h.frames(4);
            h.wait_for_images();
            let images = &h.app.drafts[&chat].images;
            let mut rects = Vec::new();
            for image in images {
                let (rect, clip) = image_clips(&h, image)[0];
                assert_eq!(rect.size(), egui::vec2(144.0, 96.0));
                assert!(
                    clip.contains_rect(rect),
                    "all thumbnails must remain visible at width {width}"
                );
                assert!(
                    rect.right() + 30.0 <= width,
                    "the remove control must fit too"
                );
                rects.push(rect);
            }
            for (index, rect) in rects.iter().enumerate() {
                assert!(
                    rects[index + 1..]
                        .iter()
                        .all(|next| !rect.intersects(*next))
                );
            }
        }
    }
}

#[test]
fn shift_enter_adds_a_line_and_plain_enter_sends_or_saves() {
    for mode in ["chat", "channel", "edit", "mention"] {
        let mut h = Harness::new();
        h.frames(3);
        if mode == "channel" {
            h.act(Action::OpenConversation(ConversationId::Channel {
                team: "t-eng".into(),
                channel: "19:eng-desktop@thread.tacv2".into(),
            }));
        } else if mode == "mention" {
            h.act(Action::OpenConversation(ConversationId::Chat(
                "19:demo-launch@thread.v2".into(),
            )));
        }
        let chat = h.app.open.clone().unwrap();
        let edited = if mode == "edit" {
            let id = h.app.conversations[&chat]
                .messages
                .iter()
                .find(|m| m.is_from(h.app.me()))
                .unwrap()
                .id
                .clone();
            h.act(Action::StartEdit(id.clone()));
            Some(id)
        } else {
            None
        };
        let first = if mode == "mention" {
            "@Gra"
        } else {
            "First line"
        };
        let draft = h.app.drafts.entry(chat.clone()).or_default();
        draft.text = first.into();
        draft.paste_cursor = Some(first.chars().count());
        h.app.focus_composer = true;
        h.frames(4);
        let count = h.app.conversations[&chat].messages.len();
        let enter = |modifiers, pressed| egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        };
        h.frame_with(vec![
            egui::Event::ModifiersChanged(egui::Modifiers::SHIFT),
            enter(egui::Modifiers::SHIFT, true),
        ]);
        assert_eq!(h.app.drafts[&chat].text, format!("{first}\n"), "{mode}");
        assert!(
            h.app.drafts[&chat].mentions.is_empty(),
            "Shift+Enter must not pick a mention"
        );
        assert_eq!(h.app.conversations[&chat].messages.len(), count);
        if let Some(id) = &edited {
            assert_eq!(h.app.drafts[&chat].editing.as_ref(), Some(id));
        }
        h.frame_with(vec![
            enter(egui::Modifiers::SHIFT, false),
            egui::Event::ModifiersChanged(egui::Modifiers::NONE),
            egui::Event::Text("Second line".into()),
        ]);
        h.frame_with(vec![enter(egui::Modifiers::NONE, true)]);
        h.frames(3);
        let messages = &h.app.conversations[&chat].messages;
        let result = if let Some(id) = &edited {
            assert_eq!(messages.len(), count);
            messages.iter().find(|m| &m.id == id).unwrap()
        } else {
            assert_eq!(messages.len(), count + 1, "{mode}");
            messages.last().unwrap()
        };
        assert_eq!(
            result.body.to_plain(),
            format!("{first}\nSecond line"),
            "{mode}"
        );
    }
}

#[test]
fn prepending_history_preserves_the_visible_row() {
    let mut h = Harness::new();
    let chat = h.app.open.clone().unwrap();
    let now = wren::time::now();
    h.app.conversations.get_mut(&chat).unwrap().messages = (0..80)
        .map(|i| wren::model::Message {
            id: format!("anchor-{i}"),
            created: now - (80 - i) * 60_000,
            body: wren::html::RichText::plain(
                "A message with enough text to occupy a complete row.",
            ),
            ..Default::default()
        })
        .collect();
    h.frames(5);
    h.frame_with(vec![
        egui::Event::PointerMoved(egui::pos2(900.0, 300.0)),
        egui::Event::MouseWheel {
            phase: egui::TouchPhase::Move,
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, 300.0),
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    h.frames(25);
    let before = h.app.conversations[&chat].history_anchor.clone().unwrap();
    h.app.handle(
        wren::backend::Event::Messages {
            conversation: chat.clone(),
            messages: vec![wren::model::Message {
                id: "prepended".into(),
                created: now - 81 * 60_000,
                body: wren::html::RichText::plain(
                    "Earlier history\nwith several lines\nto move the existing content.",
                ),
                ..Default::default()
            }],
            merge: wren::backend::Merge::Older,
            more: Some(true),
            cached: false,
        },
        &h.ctx,
    );
    h.frames(4);
    let after = h.app.conversations[&chat].history_anchor.clone().unwrap();
    assert_eq!(before.0, after.0);
    assert!(
        (before.1 - after.1).abs() < 2.0,
        "before {before:?}, after {after:?}"
    );
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
        images: Vec::new(),
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
        images: Vec::new(),
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
        images: Vec::new(),
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
        images: Vec::new(),
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
