//! The chat list: a filter field and a virtualized list (only the rows on
//! screen are laid out, however many chats there are).

use egui::{CornerRadius, Margin, Sense, vec2};

use super::widgets;
use crate::app::App;
use crate::model::{Action, Chat, ChatKind, ConversationId};
use crate::theme::{self, Icon};

/// Whether a chat matches the filter: its title or any member's name.
pub fn matches_filter(chat: &Chat, filter: &str, me: &str) -> bool {
    let filter = filter.trim().to_lowercase();
    if filter.is_empty() {
        return true;
    }
    chat.title(me).to_lowercase().contains(&filter)
        || chat
            .members
            .iter()
            .any(|m| m.display_name.to_lowercase().contains(&filter))
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    // Nothing to filter yet: the main pane explains why.
    let loading_first = app.chats.is_empty() && app.loading.contains(&crate::sync::Resource::Chats);
    if app.chats.is_empty() && !loading_first {
        return;
    }
    egui::Frame::new()
        .inner_margin(Margin::symmetric(12, 0))
        .show(ui, |ui| {
            // While the first list loads, the field is there but disabled,
            // so nothing moves when the chats arrive.
            ui.add_enabled_ui(!loading_first, |ui| {
                let field =
                    widgets::text_field(&mut app.chat_filter, "Filter chats", theme::regular(13.0));
                let response = ui.add(field);
                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    app.chat_filter.clear();
                }
                // Clear, right where the text is.
                if !app.chat_filter.is_empty() {
                    let clear = egui::Rect::from_center_size(
                        egui::pos2(response.rect.right() - 16.0, response.rect.center().y),
                        vec2(24.0, 24.0),
                    );
                    let clicked = ui
                        .put(clear, |ui: &mut egui::Ui| {
                            widgets::icon_button_sized(
                                ui,
                                &palette,
                                Icon::Close,
                                "Clear filter",
                                24.0,
                                14.0,
                                palette.secondary,
                            )
                        })
                        .clicked();
                    if clicked {
                        app.chat_filter.clear();
                    }
                }
            });
        });
    ui.add_space(8.0);
    if loading_first {
        skeleton(ui, &palette);
        return;
    }

    let me = app.me().to_owned();
    let visible: Vec<usize> = app
        .chats
        .iter()
        .enumerate()
        .filter(|(_, chat)| !chat.hidden && matches_filter(chat, &app.chat_filter, &me))
        .map(|(index, _)| index)
        .collect();
    if visible.is_empty() {
        if widgets::empty_state_with_action(
            ui,
            &palette,
            Icon::Search,
            "No matches",
            "No chat or person matches the filter.",
            Some("Clear filter"),
        ) {
            app.chat_filter.clear();
        }
        return;
    }
    let now = crate::time::now();
    let mut actions = Vec::new();
    egui::ScrollArea::vertical()
        .id_salt("chat-list")
        .auto_shrink([false, false])
        .show_rows(ui, theme::ROW_HEIGHT, visible.len(), |ui, range| {
            for index in range {
                let chat = &app.chats[visible[index]];
                let response = row(ui, app, chat, &me, now);
                if response.clicked() {
                    actions.push(Action::OpenConversation(ConversationId::Chat(
                        chat.id.clone(),
                    )));
                }
            }
        });
    app.actions.extend(actions);
}

fn row(ui: &mut egui::Ui, app: &App, chat: &Chat, me: &str, now: i64) -> egui::Response {
    let palette = &app.palette;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, theme::ROW_HEIGHT), Sense::click());
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let selected = app.open.as_ref() == Some(&ConversationId::Chat(chat.id.clone()));
    let unread = chat.is_unread(me);
    let inner = rect.shrink2(vec2(8.0, 2.0));
    if selected || response.hovered() {
        ui.painter().rect_filled(
            inner,
            CornerRadius::same(8),
            if selected {
                palette.surface_active
            } else {
                palette.surface_hover
            },
        );
    }
    if selected {
        widgets::selection_marker(ui, palette, inner);
    }
    let title = chat.title(me);
    let avatar_rect =
        egui::Rect::from_min_size(inner.left_top() + vec2(8.0, 9.0), egui::Vec2::splat(38.0));
    chat_avatar(ui, app, chat, me, &title, avatar_rect);

    let text_left = avatar_rect.right() + 10.0;
    // A message that failed to send shows here too, where the time would.
    let failed: Option<String> = app
        .conversation(&ConversationId::Chat(chat.id.clone()))
        .and_then(|c| {
            c.messages
                .iter()
                .rev()
                .find(|m| m.delivery == crate::model::Delivery::Failed)
        })
        .map(|m| m.body.to_plain());
    let (time, time_font, time_color) = if failed.is_some() {
        ("Not sent".to_owned(), theme::semibold(13.0), palette.danger)
    } else {
        (
            chat.preview
                .as_ref()
                .map(|p| crate::time::short_label(p.at, now))
                .unwrap_or_default(),
            if unread {
                theme::semibold(13.0)
            } else {
                theme::regular(13.0)
            },
            // Unread reads as weight; the dot alone carries the accent.
            if unread {
                palette.text
            } else if selected {
                // More headroom on the selected row's fill.
                palette.secondary
            } else {
                palette.dim
            },
        )
    };
    let time_galley = ui
        .ctx()
        .fonts_mut(|f| f.layout_no_wrap(time, time_font, time_color));
    let time_pos = egui::pos2(
        inner.right() - 10.0 - time_galley.size().x,
        inner.top() + 12.0,
    );
    let title_width = (time_pos.x - text_left - 8.0).max(20.0);
    let title_galley = elided(
        ui,
        &title,
        if unread {
            theme::bold(14.0)
        } else {
            theme::medium(14.0)
        },
        palette.text,
        title_width,
    );
    ui.painter().galley(
        egui::pos2(text_left, inner.top() + 10.0),
        title_galley,
        palette.text,
    );
    ui.painter().galley(time_pos, time_galley, palette.dim);

    // The message that did not go out, which the row's Not sent is about.
    let failed_preview = failed.map(|text| format!("You: {text}"));
    let preview = failed_preview.or_else(|| {
        chat.preview.as_ref().map(|p| {
            let who = match (&p.author_id, &p.author) {
                (Some(id), _) if id == me => Some("You".to_owned()),
                (_, Some(author)) if chat.kind != ChatKind::OneOnOne => Some(
                    author
                        .split_whitespace()
                        .next()
                        .unwrap_or(author)
                        .to_owned(),
                ),
                _ => None,
            };
            match who {
                Some(who) => format!("{who}: {}", p.text),
                None => p.text.clone(),
            }
        })
    });
    let preview_width = (inner.right() - text_left - if unread { 28.0 } else { 10.0 }).max(20.0);
    let preview_galley = elided(
        ui,
        preview.as_deref().unwrap_or(""),
        theme::regular(13.0),
        if unread {
            palette.text
        } else {
            palette.secondary
        },
        preview_width,
    );
    ui.painter().galley(
        egui::pos2(text_left, inner.top() + 31.0),
        preview_galley,
        palette.secondary,
    );
    if unread {
        ui.painter().circle_filled(
            egui::pos2(inner.right() - 16.0, inner.top() + 39.0),
            4.5,
            // Strong ink: orange is kept for actions and selection.
            palette.text,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A single line cut to `width` with an ellipsis.
pub fn elided(
    ui: &egui::Ui,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        text.replace('\n', " "),
        egui::TextFormat {
            font_id: font,
            color,
            ..Default::default()
        },
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(width);
    ui.ctx().fonts_mut(|f| f.layout_job(job))
}

/// A chat's picture in `rect`: the person for a 1:1 chat, initials for a
/// group, a calendar tile for a meeting. Shared with the switcher so a
/// chat looks the same everywhere.
pub fn chat_avatar(
    ui: &mut egui::Ui,
    app: &App,
    chat: &Chat,
    me: &str,
    title: &str,
    rect: egui::Rect,
) {
    let palette = &app.palette;
    let size = rect.width();
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    match chat.kind {
        ChatKind::OneOnOne => {
            let peer = chat.peer(me);
            widgets::avatar(
                &mut child,
                app,
                peer.map(|p| p.user_id.as_str()),
                peer.map_or(title, |p| p.display_name.as_str()),
                size,
                true,
            );
        }
        ChatKind::Group => {
            widgets::avatar(&mut child, app, None, title, size, false);
        }
        ChatKind::Meeting => {
            child.painter().rect_filled(
                rect,
                CornerRadius::same((size * 0.26) as u8),
                palette.surface_active,
            );
            let icon = (size * 0.47).round();
            Icon::Calendar.image(palette.accent, icon).paint_at(
                &child,
                egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(icon)),
            );
        }
    }
}

/// Placeholder rows while the first chat list loads.
pub fn skeleton(ui: &mut egui::Ui, palette: &crate::theme::Palette) {
    let fill = palette.surface_active;
    for index in 0..7 {
        let (rect, _) = ui.allocate_exact_size(
            vec2(ui.available_width(), theme::ROW_HEIGHT),
            egui::Sense::hover(),
        );
        let painter = ui.painter();
        let left = rect.left() + 16.0;
        painter.circle_filled(egui::pos2(left + 20.0, rect.center().y), 20.0, fill);
        let text_left = left + 52.0;
        let width = rect.right() - 16.0 - text_left;
        let widths = [0.55, 0.8, 0.45, 0.7, 0.6, 0.5, 0.75];
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(text_left, rect.center().y - 12.0),
                vec2(width * widths[index], 10.0),
            ),
            CornerRadius::same(5),
            fill,
        );
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(text_left, rect.center().y + 4.0),
                vec2(width * 0.9, 9.0),
            ),
            CornerRadius::same(5),
            fill,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Member;

    #[test]
    fn filter_matches_titles_and_members() {
        let chat = Chat {
            topic: Some("Launch crew".into()),
            members: vec![Member {
                user_id: "u-ada".into(),
                display_name: "Ada Lovelace".into(),
                email: None,
            }],
            ..Default::default()
        };
        assert!(matches_filter(&chat, "", "me"));
        assert!(matches_filter(&chat, "LAUNCH", "me"));
        assert!(matches_filter(&chat, "ada", "me"));
        assert!(!matches_filter(&chat, "grace", "me"));
    }
}
