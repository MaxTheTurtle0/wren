//! The open chat or channel: header, virtualized history, composer.
//!
//! The history lays out only the rows near the viewport. Every other row
//! takes the height it had when last measured (or an estimate), so a
//! conversation with thousands of messages costs what its visible rows cost.

use egui::{Align, CornerRadius, Frame, Key, Layout, Margin, Modifiers, Sense, Stroke, vec2};

use super::{rich, widgets};
use crate::app::{App, Draft};
use crate::model::{
    Access, Action, AttachmentKind, ChatKind, ConversationId, Delivery, Message, QUICK_REACTIONS,
};
use crate::theme::{self, Icon};

/// Messages from one person within this gap share a header.
const GROUP_GAP_MS: i64 = 5 * 60 * 1000;
const GUTTER: f32 = 52.0;

pub fn show(app: &mut App, ui: &mut egui::Ui, conversation: ConversationId) {
    header(app, ui, &conversation);
    composer(app, ui, &conversation);
    history(app, ui, &conversation);
}

// -- Header -------------------------------------------------------------------

fn header(app: &mut App, ui: &mut egui::Ui, conversation: &ConversationId) {
    let palette = app.palette;
    let me = app.me().to_owned();
    egui::Panel::top("conversation-header")
        .exact_size(theme::HEADER_HEIGHT)
        .show_separator_line(false)
        .frame(
            Frame::new()
                .fill(palette.window)
                .inner_margin(Margin::symmetric(16, 0)),
        )
        .show(ui, |ui| {
            let rect = ui.max_rect();
            widgets::window_drag(ui, rect, "header-drag");
            ui.painter().hline(
                rect.x_range().expand(16.0),
                rect.bottom(),
                Stroke::new(1.0, palette.outline),
            );
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let mut call_users: Vec<String> = Vec::new();
                let mut join: Option<String> = None;
                let web: Option<String>;
                let (title, subtitle) = match conversation {
                    ConversationId::Chat(id) => {
                        let chat = app.chat(id).cloned().unwrap_or_default();
                        let title = chat.title(&me);
                        call_users = chat
                            .members
                            .iter()
                            .filter(|m| m.user_id != me)
                            .filter_map(|m| m.email.clone())
                            .collect();
                        join = chat.join_url.clone();
                        web = chat.web_url.clone();
                        match chat.kind {
                            ChatKind::OneOnOne => {
                                let peer = chat.peer(&me);
                                widgets::avatar(
                                    ui,
                                    app,
                                    peer.map(|p| p.user_id.as_str()),
                                    &title,
                                    32.0,
                                    true,
                                );
                                let subtitle = peer
                                    .and_then(|p| app.presence.get(&p.user_id))
                                    .map(|p| match &p.status_message {
                                        Some(message) => format!("{} · {message}", p.describe()),
                                        None => p.describe(),
                                    })
                                    .or_else(|| peer.and_then(|p| p.email.clone()))
                                    .unwrap_or_default();
                                (title, subtitle)
                            }
                            ChatKind::Group => {
                                widgets::avatar(ui, app, None, &title, 32.0, false);
                                (title, format!("{} people", chat.members.len()))
                            }
                            ChatKind::Meeting => {
                                let (tile, _) =
                                    ui.allocate_exact_size(egui::Vec2::splat(32.0), Sense::hover());
                                ui.painter().rect_filled(
                                    tile,
                                    CornerRadius::same(8),
                                    palette.surface_active,
                                );
                                Icon::Calendar.image(palette.accent, 16.0).paint_at(
                                    ui,
                                    egui::Rect::from_center_size(
                                        tile.center(),
                                        egui::Vec2::splat(16.0),
                                    ),
                                );
                                (title, "Meeting chat".to_owned())
                            }
                        }
                    }
                    ConversationId::Channel { team, channel } => {
                        let channel_data = app.channel(team, channel).cloned();
                        let team_name = app.team(team).map(|t| t.name.clone()).unwrap_or_default();
                        web = channel_data.as_ref().and_then(|c| c.web_url.clone());
                        let (tile, _) =
                            ui.allocate_exact_size(egui::Vec2::splat(32.0), Sense::hover());
                        ui.painter().rect_filled(
                            tile,
                            CornerRadius::same(8),
                            palette.surface_active,
                        );
                        Icon::Hash.image(palette.secondary, 16.0).paint_at(
                            ui,
                            egui::Rect::from_center_size(tile.center(), egui::Vec2::splat(16.0)),
                        );
                        let name = channel_data
                            .as_ref()
                            .map(|c| c.name.clone())
                            .unwrap_or_else(|| "Channel".into());
                        let subtitle = channel_data
                            .and_then(|c| c.description)
                            .map(|d| format!("{team_name} · {d}"))
                            .unwrap_or(team_name);
                        (name, subtitle)
                    }
                };
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    let width = (ui.available_width() - 260.0).max(120.0);
                    ui.add(egui::Label::new(super::chat_list::elided(
                        ui,
                        &title,
                        theme::semibold(15.0),
                        palette.text,
                        width,
                    )));
                    if !subtitle.is_empty() {
                        ui.add(egui::Label::new(super::chat_list::elided(
                            ui,
                            &subtitle,
                            theme::regular(12.0),
                            palette.secondary,
                            width,
                        )));
                    }
                });
                // During a call in this chat the call bar has the controls.
                let in_call = app
                    .call
                    .as_ref()
                    .is_some_and(|c| !c.is_ended() && c.chat.as_ref() == Some(conversation));
                if in_call {
                    call_users.clear();
                    join = None;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    if let Some(web) = web {
                        let open = widgets::icon_button(
                            ui,
                            &palette,
                            Icon::ExternalLink,
                            "Open in Microsoft Teams",
                        );
                        widgets::on_click(&open, &mut app.actions, Action::OpenUrl(web));
                    }
                    if !call_users.is_empty() {
                        let audio = widgets::icon_button(ui, &palette, Icon::Phone, "Audio call");
                        widgets::on_click(
                            &audio,
                            &mut app.actions,
                            Action::StartCall {
                                users: call_users.clone(),
                                video: false,
                            },
                        );
                        let video = widgets::icon_button(ui, &palette, Icon::Video, "Video call");
                        widgets::on_click(
                            &video,
                            &mut app.actions,
                            Action::StartCall {
                                users: call_users,
                                video: true,
                            },
                        );
                    }
                    if let Some(join) = join {
                        ui.add_space(4.0);
                        let response = widgets::pill(ui, &palette, Icon::Video, "Join", true)
                            .on_hover_text("Join the meeting");
                        widgets::on_click(&response, &mut app.actions, Action::JoinMeeting(join));
                    }
                    if app
                        .loading
                        .contains(&crate::sync::Resource::Messages(conversation.clone()))
                    {
                        ui.add(egui::Spinner::new().size(14.0).color(palette.dim));
                    }
                });
            });
        });
}

// -- History ------------------------------------------------------------------

/// A guess at a row's height before it is laid out.
fn estimate(message: &Message, width: f32, grouped: bool) -> f32 {
    let chars = message.body.to_plain().chars().count() as f32;
    let per_line = ((width - GUTTER - 24.0) / 7.2).max(20.0);
    let lines = (chars / per_line).ceil().max(1.0)
        + message.body.blocks.len().saturating_sub(1) as f32 * 0.4;
    let mut height = lines * 20.0 + 8.0;
    if !grouped {
        height += 22.0;
    }
    if !message.reactions.is_empty() {
        height += 30.0;
    }
    if message.quote().is_some() {
        height += 54.0;
    }
    height += message.body.images().count() as f32 * 180.0;
    height
}

fn history(app: &mut App, ui: &mut egui::Ui, conversation: &ConversationId) {
    let palette = app.palette;
    let channel = conversation.is_channel();
    let empty = app
        .conversation(conversation)
        .is_none_or(|c| c.messages.is_empty());
    if empty {
        egui::CentralPanel::no_frame().show(ui, |ui| {
            if channel && matches!(app.channel_access, Access::NeedsConsent | Access::Forbidden) {
                channel_consent(app, ui, conversation);
            } else if app
                .loading
                .contains(&crate::sync::Resource::Messages(conversation.clone()))
            {
                widgets::empty_state(ui, &palette, Icon::Loader, "Loading messages…", "");
            } else {
                widgets::empty_state(
                    ui,
                    &palette,
                    Icon::Chat,
                    "No messages yet",
                    "Say hello below.",
                );
            }
        });
        return;
    }
    let me = app.me().to_owned();
    let now = crate::time::now();
    let mut actions: Vec<Action> = Vec::new();
    let jump = app.jump_to.take();
    let hover_id = egui::Id::new(("wren-hover-row", conversation.key()));
    let previous_hover: Option<(String, egui::Rect)> = ui.ctx().data(|d| d.get_temp(hover_id));
    let pointer = ui.input(|i| i.pointer.hover_pos());
    let mut hovered: Option<(String, egui::Rect, bool)> = None;
    let scroll_to_end = app.scroll_to_end > 0;
    app.scroll_to_end = app.scroll_to_end.saturating_sub(1);

    egui::CentralPanel::no_frame().show(ui, |ui| {
        let Some(view) = app.conversations.get_mut(conversation) else {
            return;
        };
        let more = view.more;
        let restore = std::mem::take(&mut view.restore_history_anchor)
            .then(|| view.history_anchor.clone())
            .flatten();
        let mut anchor = None;
        let mut heights = std::mem::take(&mut view.heights);
        let expanded = view.expanded.clone();
        let messages = std::mem::take(&mut view.messages);
        let app_ro: &App = app;
        // Opening a conversation shows its newest messages; from then on egui
        // keeps the view at the end while it is there.
        egui::ScrollArea::vertical()
            .id_salt(("history", conversation.key()))
            .auto_shrink([false, false])
            .stick_to_bottom(jump.is_none())
            .show(ui, |ui| {
                let clip = ui.clip_rect();
                let scroll_spacing = ui.spacing().item_spacing.y;
                let margin = (clip.height() * 2.0).max(600.0);
                Frame::new()
                    .inner_margin(Margin {
                        left: 16,
                        right: 20,
                        top: 12,
                        bottom: 12,
                    })
                    .show(ui, |ui| {
                        let width = ui.available_width();
                        ui.set_width(width);
                        ui.spacing_mut().item_spacing.y = 0.0;
                        if more {
                            ui.vertical_centered(|ui| {
                                let response = widgets::secondary_button(
                                    ui,
                                    &palette,
                                    "Load earlier messages",
                                );
                                widgets::on_click(
                                    &response,
                                    &mut actions,
                                    Action::LoadOlder(conversation.clone()),
                                );
                                // Reaching the top loads more on its own.
                                if ui.is_rect_visible(response.rect)
                                    && !app_ro.loading.contains(&crate::sync::Resource::Messages(
                                        conversation.clone(),
                                    ))
                                    && ui.input(|i| i.smooth_scroll_delta.y > 0.0)
                                {
                                    actions.push(Action::LoadOlder(conversation.clone()));
                                }
                            });
                            ui.add_space(12.0);
                        }
                        let mut previous: Option<&Message> = None;
                        for message in &messages {
                            let new_day = previous.is_none_or(|p| {
                                crate::time::day_key(p.created)
                                    != crate::time::day_key(message.created)
                            });
                            let grouped = !channel
                                && !new_day
                                && !message.system
                                && previous.is_some_and(|p| {
                                    !p.system
                                        && p.author_id == message.author_id
                                        && message.created - p.created < GROUP_GAP_MS
                                });
                            if new_day {
                                day_divider(ui, app_ro, message.created, now);
                            }
                            let top = ui.cursor().top();
                            let height = heights
                                .get(&message.id)
                                .copied()
                                .unwrap_or_else(|| estimate(message, width, grouped));
                            let target = jump.as_deref() == Some(message.id.as_str())
                                || restore.as_ref().is_some_and(|(id, _)| id == &message.id);
                            let near = top + height >= clip.top() - margin
                                && top <= clip.bottom() + margin;
                            if !near && !target && jump.is_none() {
                                ui.add_space(height);
                                previous = Some(message);
                                continue;
                            }
                            let rect = if channel {
                                thread_card(
                                    ui,
                                    app_ro,
                                    conversation,
                                    message,
                                    &me,
                                    now,
                                    expanded.contains(&message.id),
                                    &mut actions,
                                )
                            } else {
                                message_row(
                                    ui,
                                    app_ro,
                                    conversation,
                                    message,
                                    &me,
                                    grouped,
                                    target,
                                    &mut actions,
                                )
                            };
                            heights.insert(message.id.clone(), rect.height());
                            if jump.as_deref() == Some(message.id.as_str()) {
                                ui.scroll_to_rect(rect, Some(Align::Center));
                            }
                            if let Some((id, offset)) = &restore
                                && id == &message.id
                            {
                                ui.scroll_to_rect_animation(
                                    egui::Rect::from_min_size(
                                        // ScrollArea adds item spacing to an
                                        // aligned target; cancel that here.
                                        rect.min - vec2(0.0, *offset - scroll_spacing),
                                        vec2(1.0, clip.height()),
                                    ),
                                    Some(Align::TOP),
                                    egui::style::ScrollAnimation::none(),
                                );
                            }
                            if anchor.is_none()
                                && rect.bottom() >= clip.top()
                                && rect.top() <= clip.bottom()
                            {
                                anchor = Some((message.id.clone(), rect.top() - clip.top()));
                            }
                            if !message.system && !message.deleted {
                                let keep = previous_hover.as_ref().is_some_and(|(id, toolbar)| {
                                    id == &message.id
                                        && pointer.is_some_and(|p| toolbar.contains(p))
                                });
                                if keep
                                    || pointer.is_some_and(|p| rect.contains(p) && clip.contains(p))
                                {
                                    hovered =
                                        Some((message.id.clone(), rect, message.is_from(&me)));
                                }
                            }
                            previous = Some(message);
                        }
                        ui.add_space(4.0);
                        if scroll_to_end && jump.is_none() {
                            let end = egui::Rect::from_min_size(ui.cursor().min, vec2(1.0, 1.0));
                            ui.scroll_to_rect_animation(
                                end,
                                Some(Align::BOTTOM),
                                egui::style::ScrollAnimation::none(),
                            );
                        }
                        if let Some((id, rect, mine)) = &hovered {
                            let message = messages.iter().find(|m| &m.id == id);
                            let toolbar = hover_toolbar(
                                ui,
                                app_ro,
                                conversation,
                                id,
                                message,
                                *rect,
                                *mine,
                                &mut actions,
                            );
                            ui.ctx()
                                .data_mut(|d| d.insert_temp(hover_id, (id.clone(), toolbar)));
                        } else {
                            ui.ctx()
                                .data_mut(|d| d.remove::<(String, egui::Rect)>(hover_id));
                        }
                    });
            });
        if let Some(view) = app.conversations.get_mut(conversation) {
            if view.messages.is_empty() {
                view.messages = messages;
            } else {
                // Messages arrived while drawing (they cannot: actions run
                // after the frame), keep both to be safe.
                let mut merged = messages;
                merged.append(&mut view.messages);
                view.messages = merged;
            }
            view.heights = heights;
            if restore.is_none() {
                view.history_anchor = anchor;
            }
        }
    });
    app.actions.extend(actions);
}

fn day_divider(ui: &mut egui::Ui, app: &App, at: i64, now: i64) {
    let palette = &app.palette;
    ui.add_space(10.0);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 22.0), Sense::hover());
    let label = crate::time::day_label(at, now);
    let galley = ui
        .ctx()
        .fonts_mut(|f| f.layout_no_wrap(label, theme::semibold(12.0), palette.secondary));
    let center = rect.center();
    let half = galley.size().x / 2.0 + 12.0;
    let stroke = Stroke::new(1.0, palette.outline);
    ui.painter()
        .hline(rect.left()..=center.x - half, center.y, stroke);
    ui.painter()
        .hline(center.x + half..=rect.right(), center.y, stroke);
    ui.painter()
        .galley(center - galley.size() / 2.0, galley, palette.secondary);
    ui.add_space(6.0);
}

#[allow(clippy::too_many_arguments)]
fn message_row(
    ui: &mut egui::Ui,
    app: &App,
    conversation: &ConversationId,
    message: &Message,
    me: &str,
    grouped: bool,
    highlighted: bool,
    actions: &mut Vec<Action>,
) -> egui::Rect {
    let palette = &app.palette;
    let width = ui.available_width();
    if message.system {
        let response = ui.allocate_ui(vec2(width, 24.0), |ui| {
            ui.set_width(width);
            ui.add_space(4.0);
            ui.vertical_centered(|ui| {
                let text = format!(
                    "{} · {}",
                    message.body.to_plain(),
                    crate::time::clock(message.created)
                );
                ui.label(
                    egui::RichText::new(text)
                        .font(theme::regular(12.0))
                        .color(palette.dim),
                );
            });
            ui.add_space(4.0);
        });
        return response.response.rect;
    }
    let mentioned = message.body.blocks.iter().any(|block| match block {
        crate::html::Block::Paragraph(spans) => spans.iter().any(|s| s.style.mention),
        _ => false,
    }) && app
        .profile
        .as_ref()
        .is_some_and(|p| message.body.to_plain().contains(&p.display_name));
    let background = if highlighted || mentioned {
        Some(palette.highlight)
    } else if message.importance_high {
        Some(palette.danger.gamma_multiply(0.08))
    } else {
        None
    };
    let top_pad = if grouped { 1.0 } else { 8.0 };
    let start = ui.cursor().min;
    let where_ = ui.painter().add(egui::Shape::Noop);
    let response = ui.allocate_ui(vec2(width, 0.0), |ui| {
        ui.set_width(width);
        ui.add_space(top_pad);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            // Gutter: the avatar, or the time on hover for grouped rows.
            let (gutter, _) = ui.allocate_exact_size(
                vec2(GUTTER, if grouped { 20.0 } else { 36.0 }),
                Sense::hover(),
            );
            if !grouped {
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(
                    egui::Rect::from_min_size(gutter.min + vec2(4.0, 2.0), egui::Vec2::splat(36.0)),
                ));
                widgets::avatar(
                    &mut child,
                    app,
                    message.author_id.as_deref(),
                    &message.author,
                    36.0,
                    false,
                );
            } else if ui
                .rect_contains_pointer(egui::Rect::from_min_size(gutter.min, vec2(width, 22.0)))
            {
                ui.painter().text(
                    gutter.left_top() + vec2(4.0, 4.0),
                    egui::Align2::LEFT_TOP,
                    crate::time::clock(message.created),
                    theme::regular(11.0),
                    palette.dim,
                );
            }
            ui.vertical(|ui| {
                let content_width = (width - GUTTER - 8.0).max(80.0);
                ui.set_max_width(content_width);
                ui.spacing_mut().item_spacing.y = 3.0;
                if !grouped {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let name_color = if message.is_from(me) {
                            palette.accent
                        } else {
                            palette.text
                        };
                        ui.label(
                            egui::RichText::new(&message.author)
                                .font(theme::semibold(14.0))
                                .color(name_color),
                        );
                        ui.label(
                            egui::RichText::new(crate::time::clock(message.created))
                                .font(theme::regular(12.0))
                                .color(palette.dim),
                        )
                        .on_hover_text(crate::time::day_label(message.created, crate::time::now()));
                        if message.importance_high {
                            ui.label(
                                egui::RichText::new("IMPORTANT")
                                    .font(theme::bold(11.0))
                                    .color(palette.danger),
                            );
                        }
                    });
                }
                body(ui, app, conversation, message, me, content_width, actions);
            });
        });
        ui.add_space(2.0);
    });
    let rect = egui::Rect::from_min_max(
        start,
        egui::pos2(start.x + width, response.response.rect.bottom()),
    );
    if let Some(fill) = background {
        ui.painter().set(
            where_,
            egui::Shape::rect_filled(rect.expand2(vec2(8.0, 0.0)), CornerRadius::same(6), fill),
        );
    }
    rect
}

/// A message's content below its header: quote, text, attachments,
/// reactions, delivery state.
fn body(
    ui: &mut egui::Ui,
    app: &App,
    conversation: &ConversationId,
    message: &Message,
    me: &str,
    width: f32,
    actions: &mut Vec<Action>,
) {
    let palette = &app.palette;
    if message.deleted {
        ui.label(
            egui::RichText::new("This message was deleted.")
                .italics()
                .color(palette.dim),
        );
        return;
    }
    if let Some(quote) = message.quote()
        && let AttachmentKind::Quote {
            author,
            preview,
            message_id,
        } = &quote.kind
    {
        let response = rich::quote_block(ui, palette, author.as_deref(), preview, width.min(560.0));
        if response.interact(Sense::click()).clicked() {
            actions.push(Action::OpenMessage {
                conversation: conversation.clone(),
                message: message_id.clone(),
            });
        }
    }
    let look = rich::Look {
        size: 14.0,
        color: if message.delivery == Delivery::Sent {
            palette.text
        } else {
            palette.secondary
        },
    };
    rich::show(ui, app, &message.body, &look, width, actions);
    for attachment in &message.attachments {
        match &attachment.kind {
            AttachmentKind::File => file_chip(ui, app, attachment, actions),
            AttachmentKind::Card => {
                ui.label(
                    egui::RichText::new(
                        "A card Wren can't show yet. Open the message in Teams to see it.",
                    )
                    .font(theme::regular(12.0))
                    .color(palette.dim),
                );
            }
            _ => {}
        }
    }
    if message.edited {
        ui.label(
            egui::RichText::new("Edited")
                .font(theme::regular(11.0))
                .color(palette.dim),
        );
    }
    match message.delivery {
        Delivery::Sending => {
            ui.label(
                egui::RichText::new("Sending…")
                    .font(theme::regular(11.0))
                    .color(palette.dim),
            );
        }
        Delivery::Failed => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Not sent.")
                        .font(theme::regular(12.0))
                        .color(palette.danger),
                );
                if ui
                    .link(egui::RichText::new("Retry").font(theme::semibold(12.0)))
                    .clicked()
                {
                    actions.push(Action::RetrySend {
                        conversation: conversation.clone(),
                        local_id: message.id.clone(),
                    });
                }
            });
        }
        Delivery::Sent => {}
    }
    if !message.reactions.is_empty() {
        reactions(ui, app, conversation, message, me, actions);
    }
}

fn file_chip(
    ui: &mut egui::Ui,
    app: &App,
    attachment: &crate::model::Attachment,
    actions: &mut Vec<Action>,
) {
    let palette = &app.palette;
    let name = attachment
        .name
        .clone()
        .unwrap_or_else(|| "Attachment".into());
    let response = Frame::new()
        .fill(palette.surface)
        .stroke(Stroke::new(1.0, palette.outline))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                theme::icon(ui, Icon::File, 18.0, palette.accent);
                ui.label(
                    egui::RichText::new(&name)
                        .font(theme::medium(13.0))
                        .color(palette.text),
                );
                if attachment.url.is_some() {
                    theme::icon(ui, Icon::ExternalLink, 13.0, palette.dim);
                }
            });
        })
        .response
        .interact(Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Open in your browser");
    if response.clicked()
        && let Some(url) = &attachment.url
    {
        actions.push(Action::OpenUrl(url.clone()));
    }
}

fn reactions(
    ui: &mut egui::Ui,
    app: &App,
    conversation: &ConversationId,
    message: &Message,
    me: &str,
    actions: &mut Vec<Action>,
) {
    let palette = &app.palette;
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        for reaction in &message.reactions {
            let mine = reaction.users.iter().any(|u| u == me);
            let text = format!("{} {}", reaction.emoji, reaction.users.len());
            let button = egui::Button::new(
                egui::RichText::new(text)
                    .font(theme::medium(12.0))
                    .color(if mine { palette.accent } else { palette.text }),
            )
            .fill(if mine {
                palette.highlight
            } else {
                palette.surface_hover
            })
            .stroke(Stroke::new(
                1.0,
                if mine {
                    palette.accent.gamma_multiply(0.7)
                } else {
                    palette.outline
                },
            ))
            .corner_radius(CornerRadius::same(12))
            .min_size(vec2(0.0, 24.0));
            let names: Vec<String> = reaction
                .users
                .iter()
                .map(|u| {
                    if u == me {
                        "You".to_owned()
                    } else {
                        app.chats
                            .iter()
                            .flat_map(|c| c.members.iter())
                            .find(|m| &m.user_id == u)
                            .map(|m| m.display_name.clone())
                            .unwrap_or_else(|| "Someone".into())
                    }
                })
                .collect();
            let response = ui.add(button).on_hover_text(names.join(", "));
            if response.clicked() {
                actions.push(Action::React {
                    conversation: conversation.clone(),
                    message: message.id.clone(),
                    emoji: reaction.emoji.clone(),
                    on: !mine,
                });
            }
        }
    });
}

/// The actions shown over the hovered message. Returns the toolbar's rect.
#[allow(clippy::too_many_arguments)]
fn hover_toolbar(
    ui: &mut egui::Ui,
    app: &App,
    conversation: &ConversationId,
    id: &str,
    message: Option<&Message>,
    row: egui::Rect,
    mine: bool,
    actions: &mut Vec<Action>,
) -> egui::Rect {
    let palette = &app.palette;
    let buttons = QUICK_REACTIONS.len() as f32 + 2.0 + if mine { 2.0 } else { 0.0 };
    let size = vec2(buttons * 28.0 + 12.0, 34.0);
    let min = egui::pos2(
        row.right() - size.x - 4.0,
        (row.top() - 12.0).max(ui.clip_rect().top() + 2.0),
    );
    let rect = egui::Rect::from_min_size(min, size);
    let reacted = |emoji: &str| {
        message.is_some_and(|m| {
            m.reactions
                .iter()
                .any(|r| r.emoji == emoji && r.users.iter().any(|u| u == app.me()))
        })
    };
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        Frame::new()
            .fill(palette.surface)
            .stroke(Stroke::new(1.0, palette.outline))
            .corner_radius(CornerRadius::same(8))
            .shadow(egui::Shadow {
                offset: [0, 2],
                blur: 8,
                spread: 0,
                color: palette.shadow,
            })
            .inner_margin(Margin::same(3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    for emoji in QUICK_REACTIONS {
                        let on = reacted(emoji);
                        let response = ui.add_sized(
                            vec2(28.0, 26.0),
                            egui::Button::new(egui::RichText::new(emoji).size(15.0))
                                .frame(on)
                                .fill(palette.highlight),
                        );
                        if response.clicked() {
                            actions.push(Action::React {
                                conversation: conversation.clone(),
                                message: id.to_owned(),
                                emoji: emoji.to_owned(),
                                on: !on,
                            });
                        }
                    }
                    let reply_label = if conversation.is_channel() {
                        "Reply in thread"
                    } else {
                        "Reply"
                    };
                    let reply = widgets::icon_button_sized(
                        ui,
                        palette,
                        Icon::Reply,
                        reply_label,
                        28.0,
                        16.0,
                        palette.secondary,
                    );
                    widgets::on_click(&reply, actions, Action::StartReply(id.to_owned()));
                    if mine
                        && (!conversation.is_channel() || app.has_scope("ChannelMessage.ReadWrite"))
                    {
                        let edit = widgets::icon_button_sized(
                            ui,
                            palette,
                            Icon::Pencil,
                            "Edit",
                            28.0,
                            16.0,
                            palette.secondary,
                        );
                        widgets::on_click(&edit, actions, Action::StartEdit(id.to_owned()));
                        let delete = widgets::icon_button_sized(
                            ui,
                            palette,
                            Icon::Trash,
                            "Delete",
                            28.0,
                            16.0,
                            palette.secondary,
                        );
                        widgets::on_click(
                            &delete,
                            actions,
                            Action::Delete {
                                conversation: conversation.clone(),
                                message: id.to_owned(),
                            },
                        );
                    }
                    let copy = widgets::icon_button_sized(
                        ui,
                        palette,
                        Icon::Copy,
                        "Copy text",
                        28.0,
                        16.0,
                        palette.secondary,
                    );
                    if copy.clicked()
                        && let Some(message) = message
                    {
                        actions.push(Action::CopyText(message.body.to_plain()));
                    }
                });
            });
    });
    rect
}

#[allow(clippy::too_many_arguments)]
fn thread_card(
    ui: &mut egui::Ui,
    app: &App,
    conversation: &ConversationId,
    root: &Message,
    me: &str,
    now: i64,
    expanded: bool,
    actions: &mut Vec<Action>,
) -> egui::Rect {
    let palette = &app.palette;
    let width = ui.available_width();
    if root.system {
        return message_row(ui, app, conversation, root, me, false, false, actions);
    }
    ui.add_space(6.0);
    let response = Frame::new()
        .fill(palette.surface)
        .stroke(Stroke::new(
            1.0,
            if root.importance_high {
                palette.danger.gamma_multiply(0.6)
            } else {
                palette.outline
            },
        ))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin {
            left: 12,
            right: 14,
            top: 10,
            bottom: 8,
        })
        .show(ui, |ui| {
            ui.set_width(width - 26.0);
            let inner_width = width - 26.0;
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                widgets::avatar(
                    ui,
                    app,
                    root.author_id.as_deref(),
                    &root.author,
                    34.0,
                    false,
                );
                ui.vertical(|ui| {
                    let content = inner_width - 44.0;
                    ui.set_max_width(content);
                    ui.spacing_mut().item_spacing.y = 3.0;
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&root.author)
                                .font(theme::semibold(14.0))
                                .color(palette.text),
                        );
                        ui.label(
                            egui::RichText::new(stamp(root.created, now))
                                .font(theme::regular(12.0))
                                .color(palette.dim),
                        );
                        if root.importance_high {
                            ui.label(
                                egui::RichText::new("IMPORTANT")
                                    .font(theme::bold(11.0))
                                    .color(palette.danger),
                            );
                        }
                    });
                    if let Some(subject) = &root.subject {
                        ui.label(
                            egui::RichText::new(subject)
                                .font(theme::bold(15.0))
                                .color(palette.text),
                        );
                    }
                    body(ui, app, conversation, root, me, content, actions);
                });
            });
            // Replies.
            ui.add_space(6.0);
            let count = root.replies.len();
            if count > 0 {
                let line = ui.cursor().top();
                ui.painter().hline(
                    ui.max_rect().x_range(),
                    line,
                    Stroke::new(1.0, palette.outline),
                );
                ui.add_space(6.0);
                if !expanded && count > 2 {
                    let mut names: Vec<&str> =
                        root.replies.iter().map(|r| r.author.as_str()).collect();
                    names.sort_unstable();
                    names.dedup();
                    let text = format!(
                        "{count} replies from {}",
                        names.into_iter().take(3).collect::<Vec<_>>().join(", ")
                    );
                    if ui
                        .link(
                            egui::RichText::new(text)
                                .font(theme::semibold(12.0))
                                .color(palette.accent),
                        )
                        .clicked()
                    {
                        actions.push(Action::ToggleThread(root.id.clone()));
                    }
                }
                let shown: Vec<&Message> = if expanded || count <= 2 {
                    root.replies.iter().collect()
                } else {
                    root.replies.iter().rev().take(1).collect()
                };
                for reply in shown {
                    ui.horizontal_top(|ui| {
                        ui.add_space(8.0);
                        ui.spacing_mut().item_spacing.x = 8.0;
                        widgets::avatar(
                            ui,
                            app,
                            reply.author_id.as_deref(),
                            &reply.author,
                            26.0,
                            false,
                        );
                        ui.vertical(|ui| {
                            let content = inner_width - 52.0;
                            ui.set_max_width(content);
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&reply.author)
                                        .font(theme::semibold(13.0))
                                        .color(palette.text),
                                );
                                ui.label(
                                    egui::RichText::new(crate::time::clock(reply.created))
                                        .font(theme::regular(11.0))
                                        .color(palette.dim),
                                );
                            });
                            body(ui, app, conversation, reply, me, content, actions);
                        });
                    });
                    ui.add_space(4.0);
                }
                if expanded
                    && count > 2
                    && ui
                        .link(
                            egui::RichText::new("Collapse")
                                .font(theme::regular(12.0))
                                .color(palette.secondary),
                        )
                        .clicked()
                {
                    actions.push(Action::ToggleThread(root.id.clone()));
                }
            }
            ui.horizontal(|ui| {
                let replying = app
                    .drafts
                    .get(conversation)
                    .and_then(|d| d.reply_to.as_deref())
                    == Some(root.id.as_str());
                let label = if replying {
                    "Replying below…"
                } else {
                    "Reply"
                };
                let response = ui.add(
                    egui::Button::new(
                        egui::RichText::new(label)
                            .font(theme::medium(12.0))
                            .color(palette.secondary),
                    )
                    .frame(false),
                );
                if response.clicked() {
                    actions.push(Action::StartReply(root.id.clone()));
                }
            });
        });
    ui.add_space(6.0);
    response.response.rect
}

// -- Composer -----------------------------------------------------------------

/// The `@name` being typed before the cursor: where its `@` is (in
/// characters) and what follows it. Names have spaces, so up to two words
/// count; a line break or a third word ends it.
pub fn mention_query(text: &str, cursor: usize) -> Option<(usize, String)> {
    let chars: Vec<char> = text.chars().collect();
    let cursor = cursor.min(chars.len());
    let mut at = None;
    for index in (0..cursor).rev() {
        let c = chars[index];
        if c == '\n' || cursor - index > 32 {
            return None;
        }
        if c == '@' {
            at = Some(index);
            break;
        }
    }
    let at = at?;
    if at > 0 && chars[at - 1].is_alphanumeric() {
        // An email address, not a mention.
        return None;
    }
    let query: String = chars[at + 1..cursor].iter().collect();
    if query.split(' ').count() > 3 || query.starts_with(' ') {
        return None;
    }
    Some((at, query))
}

/// People who can be mentioned here whose name matches `query`.
fn mention_candidates(
    app: &App,
    conversation: &ConversationId,
    query: &str,
) -> Vec<crate::model::Member> {
    let needle = query.trim().to_lowercase();
    let me = app.me().to_owned();
    let matches = |name: &str| {
        needle.is_empty()
            || name
                .to_lowercase()
                .split_whitespace()
                .any(|word| word.starts_with(&needle))
            || name.to_lowercase().starts_with(&needle)
    };
    let mut people: Vec<crate::model::Member> = match conversation {
        ConversationId::Chat(chat) => app
            .chat(chat)
            .map(|c| {
                c.members
                    .iter()
                    .filter(|m| m.user_id != me)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default(),
        ConversationId::Channel { .. } => app.directory(),
    };
    if conversation.is_channel() && app.people.query == query {
        for person in &app.people.results {
            if !people.iter().any(|p| p.user_id == person.user_id) {
                people.push(person.clone());
            }
        }
    }
    people.retain(|p| matches(&p.display_name));
    people.truncate(6);
    people
}

fn file_size(path: &std::path::Path) -> String {
    let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    match bytes {
        b if b >= 1024 * 1024 => format!("{:.1} MB", b as f64 / (1024.0 * 1024.0)),
        b if b >= 1024 => format!("{} KB", b / 1024),
        b => format!("{b} B"),
    }
}

fn composer(app: &mut App, ui: &mut egui::Ui, conversation: &ConversationId) {
    let palette = app.palette;
    let title = app.title_of(conversation);
    let channel = conversation.is_channel();
    let mut actions = Vec::new();
    let mut draft = app.drafts.get(conversation).cloned().unwrap_or_default();
    let context_line = draft_context(app, conversation, &draft);
    let can_post = !channel || app.has_scope("ChannelMessage.Send");
    let id = ui.make_persistent_id(("composer", conversation.key()));
    if let Some(cursor) = draft.paste_cursor.take() {
        let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(
                egui::text::CCursor::new(cursor),
            )));
        state.store(ui.ctx(), id);
    }
    // The mention being typed, from the cursor the editor had last frame.
    let cursor = egui::TextEdit::load_state(ui.ctx(), id)
        .and_then(|state| state.cursor.char_range())
        .map(|range| range.primary.index.0);
    let dismissed_id = id.with("mention-dismissed");
    let dismissed: Option<usize> = ui.ctx().data(|d| d.get_temp(dismissed_id));
    let mention = cursor
        .and_then(|cursor| {
            mention_query(&draft.text, cursor).map(|(at, query)| (at, query, cursor))
        })
        .filter(|(at, ..)| dismissed != Some(*at));
    let candidates = mention
        .as_ref()
        .map(|(_, query, _)| mention_candidates(app, conversation, query))
        .unwrap_or_default();
    if let Some((_, query, _)) = &mention
        && channel
        && query.chars().count() >= 2
        && app.people.query != *query
    {
        actions.push(Action::SearchPeople(query.clone()));
    }
    let selected_id = id.with("mention-selected");
    let mut selected: usize = ui.ctx().data(|d| d.get_temp(selected_id)).unwrap_or(0);
    selected = selected.min(candidates.len().saturating_sub(1));
    // A person clicked in the list last frame.
    let pick_id = id.with("mention-pick");
    let clicked: Option<usize> = ui.ctx().data_mut(|d| d.remove_temp(pick_id));
    let mut picked: Option<crate::model::Member> =
        clicked.and_then(|index| candidates.get(index).cloned());
    let mut composer_top = None;
    egui::Panel::bottom("composer")
        .show_separator_line(false)
        .frame(Frame::new().fill(palette.window).inner_margin(Margin {
            left: 16,
            right: 16,
            top: 6,
            bottom: 14,
        }))
        .show(ui, |ui| {
            composer_top = Some(ui.cursor().min);
            Frame::new()
                .fill(palette.surface)
                .stroke(Stroke::new(1.0, palette.outline))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin {
                    left: 12,
                    right: 8,
                    top: 6,
                    bottom: 6,
                })
                .show(ui, |ui| {
                    if let Some((icon, text)) = &context_line {
                        ui.horizontal(|ui| {
                            theme::icon(ui, *icon, 14.0, palette.accent);
                            let width = ui.available_width() - 40.0;
                            ui.add(egui::Label::new(super::chat_list::elided(
                                ui,
                                text,
                                theme::regular(12.0),
                                palette.secondary,
                                width,
                            )));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let close = widgets::icon_button_sized(
                                    ui,
                                    &palette,
                                    Icon::Close,
                                    "Cancel (Esc)",
                                    22.0,
                                    14.0,
                                    palette.dim,
                                );
                                widgets::on_click(&close, &mut actions, Action::CancelCompose);
                            });
                        });
                        ui.add_space(2.0);
                    }
                    // Files waiting to go with the message.
                    if !draft.files.is_empty() {
                        let mut remove = None;
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                            for (index, path) in draft.files.iter().enumerate() {
                                let name = path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_default();
                                Frame::new()
                                    .fill(palette.surface_hover)
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(Margin { left: 8, right: 2, top: 3, bottom: 3 })
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 4.0;
                                            theme::icon(ui, Icon::File, 14.0, palette.accent);
                                            ui.label(
                                                egui::RichText::new(crate::html::truncate(&name, 32))
                                                    .font(theme::medium(12.0))
                                                    .color(palette.text),
                                            );
                                            ui.label(
                                                egui::RichText::new(file_size(path))
                                                    .font(theme::regular(11.0))
                                                    .color(palette.dim),
                                            );
                                            let close = widgets::icon_button_sized(
                                                ui, &palette, Icon::Close, "Remove", 20.0, 12.0, palette.dim,
                                            );
                                            if close.clicked() {
                                                remove = Some(index);
                                            }
                                        });
                                    });
                            }
                        });
                        if let Some(index) = remove {
                            draft.files.remove(index);
                        }
                        ui.add_space(4.0);
                    }
                    if !draft.images.is_empty() {
                        let mut remove = None;
                        ui.horizontal_wrapped(|ui| {
                            for (index, image) in draft.images.iter().enumerate() {
                                let group_size = vec2(144.0 + ui.spacing().item_spacing.x + 22.0, 96.0);
                                ui.allocate_ui_with_layout(group_size, Layout::left_to_right(Align::Center), |ui| {
                                        // Crop a fixed-size thumbnail instead of shrinking a
                                        // wide screenshot into a line. The preview keeps
                                        // the full image and its original proportions.
                                        let size = vec2(144.0, 96.0);
                                        let natural = vec2(image.width as f32, image.height as f32);
                                        let scale = (size.x / natural.x).max(size.y / natural.y);
                                        let uv = egui::Rect::from_center_size(
                                            egui::pos2(0.5, 0.5), size / (natural * scale),
                                        );
                                        let response = ui.add(
                                            egui::Image::from_bytes(image.uri(), image.png.clone())
                                                .fit_to_exact_size(size)
                                                .maintain_aspect_ratio(false)
                                                .uv(uv)
                                                .corner_radius(6)
                                                .sense(Sense::click())
                                                .alt_text("Preview pasted image"),
                                        ).on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text("Preview image");
                                        if response.clicked() {
                                            actions.push(Action::PreviewImage(image.uri()));
                                        }
                                        if widgets::icon_button_sized(ui, &palette, Icon::Close, "Remove image", 22.0, 14.0, palette.dim).clicked() {
                                            remove = Some(index);
                                        }
                                });
                            }
                        });
                        if let Some(index) = remove { draft.images.remove(index); }
                        ui.add_space(4.0);
                    }
                    ui.horizontal(|ui| {
                        let focused = ui.memory(|m| m.has_focus(id));
                        let mut send = false;
                        let mut paste_requested = false;
                        let range = egui::TextEdit::load_state(ui.ctx(), id)
                            .and_then(|state| state.cursor.char_range())
                            .map(|r| (r.primary.index.0.min(r.secondary.index.0), r.primary.index.0.max(r.secondary.index.0)))
                            .unwrap_or((draft.text.chars().count(), draft.text.chars().count()));
                        if focused {
                            ui.input_mut(|input| {
                                let mut pastes = Vec::new();
                                let mut enter = false;
                                input.events.retain(|event| {
                                    if let egui::Event::Paste(text) = event {
                                        pastes.push(text.clone()); false
                                    } else if matches!(event, egui::Event::Key {
                                        key: Key::Enter, pressed: true, modifiers, ..
                                    } if modifiers.matches_exact(Modifiers::NONE)) {
                                        // consume_key ignores extra Shift and Alt.
                                        // Keep Shift+Enter for the multiline editor.
                                        enter = true; false
                                    } else { true }
                                });
                                for text in pastes {
                                    paste_requested = true;
                                    actions.push(Action::Paste { conversation: conversation.clone(), range, text });
                                }
                                // The mention list takes the arrows, Enter
                                // and Tab while it shows.
                                if !candidates.is_empty() {
                                    if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                                        selected = (selected + 1) % candidates.len();
                                    }
                                    if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                                        selected = (selected + candidates.len() - 1) % candidates.len();
                                    }
                                    if enter
                                        || input.consume_key(Modifiers::NONE, Key::Tab)
                                    {
                                        picked = candidates.get(selected).cloned();
                                        enter = false;
                                    }
                                    if input.consume_key(Modifiers::NONE, Key::Escape)
                                        && let Some((at, ..)) = &mention
                                    {
                                        ui.ctx().data_mut(|d| d.insert_temp(dismissed_id, *at));
                                    }
                                }
                                send |= enter;
                                if input.consume_key(Modifiers::NONE, Key::Escape) {
                                    actions.push(Action::CancelCompose);
                                }
                                if draft.text.is_empty()
                                    && input.consume_key(Modifiers::NONE, Key::ArrowUp)
                                {
                                    let me = app.me();
                                    if let Some(last) =
                                        app.conversation(conversation).and_then(|c| {
                                            c.messages
                                                .iter()
                                                .rev()
                                                .find(|m| m.is_from(me) && !m.deleted && !m.system)
                                        })
                                        && !channel
                                    {
                                        actions.push(Action::StartEdit(last.id.clone()));
                                    }
                                }
                            });
                        }
                        // Put the picked name in place of `@query`.
                        if let (Some(person), Some((at, _, cursor))) = (&picked, &mention) {
                            let chars: Vec<char> = draft.text.chars().collect();
                            let before: String = chars[..*at].iter().collect();
                            let after: String = chars[(*cursor).min(chars.len())..].iter().collect();
                            let inserted = format!("@{} ", person.display_name);
                            draft.text = format!("{before}{inserted}{after}");
                            if !draft.mentions.iter().any(|m| m.user_id == person.user_id) {
                                draft.mentions.push(crate::model::MentionRef {
                                    user_id: person.user_id.clone(),
                                    name: person.display_name.clone(),
                                });
                            }
                            let caret = at + inserted.chars().count();
                            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), id) {
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(caret),
                                )));
                                egui::TextEdit::store_state(ui.ctx(), id, state);
                            }
                        }
                        let hint = if !can_post {
                            "Posting needs the ChannelMessage.Send permission".to_owned()
                        } else if draft.editing.is_some() {
                            "Edit your message".to_owned()
                        } else if channel && draft.reply_to.is_none() {
                            format!("Start a new post in {title}  ·  @ to mention")
                        } else if channel {
                            "Reply to the thread".to_owned()
                        } else {
                            format!("Message {title}  ·  @ to mention")
                        };
                        let width = ui.available_width() - 70.0;
                        let edit = egui::TextEdit::multiline(&mut draft.text)
                            .id(id)
                            .hint_text(hint)
                            .desired_rows(1)
                            .desired_width(width)
                            // A custom Frame overrides TextEdit::margin.
                            // Put the padding on the frame itself.
                            .frame(Frame::new().inner_margin(Margin::symmetric(0, 6)))
                            .font(theme::regular(14.0))
                            .vertical_align(Align::Center);
                        let response = ui.add_enabled(can_post, edit);
                        if std::mem::take(&mut app.focus_composer) {
                            response.request_focus();
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            let ready = !draft.text.trim().is_empty() || !draft.files.is_empty() || !draft.images.is_empty();
                            let tint = if ready { palette.accent } else { palette.dim };
                            let label = if draft.editing.is_some() {
                                "Save (Enter)"
                            } else {
                                "Send (Enter)"
                            };
                            let button = widgets::icon_button_sized(
                                ui,
                                &palette,
                                if draft.editing.is_some() {
                                    Icon::Check
                                } else {
                                    Icon::Send
                                },
                                label,
                                30.0,
                                18.0,
                                tint,
                            );
                            if button.clicked() {
                                send = true;
                            }
                            if draft.editing.is_none() {
                                let attach = widgets::icon_button_sized(
                                    ui,
                                    &palette,
                                    Icon::Paperclip,
                                    "Attach files (or drop them here)",
                                    30.0,
                                    18.0,
                                    palette.secondary,
                                );
                                widgets::on_click(&attach, &mut actions, Action::AttachFiles);
                            }
                        });
                        let has_content = !draft.text.trim().is_empty() || !draft.files.is_empty() || !draft.images.is_empty();
                        if send && has_content && can_post && picked.is_none() && draft.pending_pastes == 0 && !paste_requested {
                            match &draft.editing {
                                Some(message) => actions.push(Action::Edit {
                                    conversation: conversation.clone(),
                                    message: message.clone(),
                                    text: std::mem::take(&mut draft.text),
                                }),
                                None => actions.push(Action::Send {
                                    conversation: conversation.clone(),
                                    text: std::mem::take(&mut draft.text),
                                    reply_to: draft.reply_to.take(),
                                    mentions: std::mem::take(&mut draft.mentions),
                                    files: std::mem::take(&mut draft.files),
                                    images: std::mem::take(&mut draft.images),
                                }),
                            }
                            draft.editing = None;
                        }
                    });
                });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(
                    "**bold**  *italic*  `code`  > quote  - list  @ mention   ·   Shift+Enter for a new line",
                )
                .font(theme::regular(11.0))
                .color(palette.dim),
            );
        });
    ui.ctx().data_mut(|d| d.insert_temp(selected_id, selected));
    // The mention list, just above the composer.
    if picked.is_none()
        && !candidates.is_empty()
        && let Some(top) = composer_top
    {
        let height = candidates.len() as f32 * 40.0 + 12.0;
        egui::Area::new(id.with("mentions"))
            .order(egui::Order::Foreground)
            .fixed_pos(top + vec2(0.0, -height - 4.0))
            .show(ui.ctx(), |ui| {
                Frame::new()
                    .fill(palette.surface)
                    .stroke(Stroke::new(1.0, palette.outline))
                    .corner_radius(CornerRadius::same(10))
                    .shadow(egui::Shadow {
                        offset: [0, 4],
                        blur: 16,
                        spread: 0,
                        color: palette.shadow,
                    })
                    .inner_margin(Margin::same(6))
                    .show(ui, |ui| {
                        ui.set_width(300.0);
                        for (index, person) in candidates.iter().enumerate() {
                            let (rect, response) =
                                ui.allocate_exact_size(vec2(300.0, 38.0), Sense::click());
                            if index == selected || response.hovered() {
                                ui.painter().rect_filled(
                                    rect,
                                    CornerRadius::same(7),
                                    palette.surface_active,
                                );
                            }
                            let mut child = ui.new_child(
                                egui::UiBuilder::new()
                                    .max_rect(rect.shrink2(vec2(8.0, 0.0)))
                                    .layout(Layout::left_to_right(Align::Center)),
                            );
                            widgets::avatar(
                                &mut child,
                                app,
                                Some(&person.user_id),
                                &person.display_name,
                                26.0,
                                true,
                            );
                            child.add_space(8.0);
                            child.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.label(
                                    egui::RichText::new(&person.display_name)
                                        .font(theme::medium(13.0))
                                        .color(palette.text),
                                );
                                if let Some(email) = &person.email {
                                    ui.label(
                                        egui::RichText::new(email)
                                            .font(theme::regular(11.0))
                                            .color(palette.dim),
                                    );
                                }
                            });
                            if response.clicked() {
                                // Put in place next frame, by the composer.
                                ui.ctx().data_mut(|d| d.insert_temp(pick_id, index));
                                ui.ctx().memory_mut(|m| m.request_focus(id));
                                ui.ctx().request_repaint();
                            }
                        }
                    });
            });
    }
    let changed = app.drafts.get(conversation).cloned().unwrap_or_default() != draft;
    let sending = actions
        .iter()
        .any(|a| matches!(a, Action::Send { .. } | Action::Edit { .. }));
    if changed && !sending {
        if draft == Draft::default() {
            app.drafts.remove(conversation);
        } else {
            app.drafts.insert(conversation.clone(), draft);
        }
    } else if sending {
        app.drafts.remove(conversation);
    }
    app.actions.extend(actions);
}

fn draft_context(
    app: &App,
    conversation: &ConversationId,
    draft: &Draft,
) -> Option<(Icon, String)> {
    if let Some(editing) = &draft.editing {
        let _ = editing;
        return Some((Icon::Pencil, "Editing your message".into()));
    }
    let reply_to = draft.reply_to.as_ref()?;
    let message = app.conversation(conversation)?.find(reply_to)?;
    let text = if conversation.is_channel() {
        let subject = message
            .subject
            .clone()
            .unwrap_or_else(|| message.body.summary(80));
        format!("Replying to {}: {subject}", message.author)
    } else {
        format!("Quoting {}: {}", message.author, message.body.summary(100))
    };
    Some((Icon::Reply, text))
}

fn channel_consent(app: &mut App, ui: &mut egui::Ui, conversation: &ConversationId) {
    let palette = app.palette;
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() * 0.22).max(24.0));
        ui.set_max_width(460.0);
        theme::icon(ui, Icon::Lock, 36.0, palette.dim);
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Channel posts need admin approval").font(theme::semibold(18.0)).color(palette.text));
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "Reading channel posts uses Microsoft Graph's ChannelMessage.Read.All permission, which a tenant administrator must approve. You can still post here; posting needs only ChannelMessage.Send.",
            )
            .color(palette.secondary),
        );
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            let total = 300.0;
            ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
            if widgets::primary_button(ui, &palette, "Request access").clicked() {
                app.actions.push(Action::RequestScope(crate::auth::CHANNEL_READ_SCOPE));
            }
            if let ConversationId::Channel { team, channel } = conversation
                && let Some(url) = app.channel(team, channel).and_then(|c| c.web_url.clone())
                && widgets::secondary_button(ui, &palette, "Open in Teams").clicked()
            {
                app.actions.push(Action::OpenUrl(url));
            }
        });
    });
}

/// `14:05` today, `Yesterday 14:05`, `Mon 14:05`, `2 Oct 14:05`.
fn stamp(at: i64, now: i64) -> String {
    if crate::time::day_key(at) == crate::time::day_key(now) {
        crate::time::clock(at)
    } else {
        format!(
            "{} {}",
            crate::time::short_label(at, now),
            crate::time::clock(at)
        )
    }
}

#[cfg(test)]
mod mention_tests {
    use super::mention_query;

    #[test]
    fn mentions_start_at_an_at_sign_and_end_at_a_line_break() {
        assert_eq!(mention_query("hi @ad", 6), Some((3, "ad".into())));
        assert_eq!(mention_query("@", 1), Some((0, String::new())));
        assert_eq!(mention_query("hi @Ada Lo", 10), Some((3, "Ada Lo".into())));
        assert_eq!(mention_query("mail me@contoso", 15), None);
        assert_eq!(mention_query("@ada\nnext", 9), None);
        assert_eq!(mention_query("no mention", 10), None);
        assert_eq!(mention_query("@a b c d e", 10), None);
    }
}
