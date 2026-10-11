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
    egui::Frame::new()
        .inner_margin(Margin::symmetric(12, 0))
        .show(ui, |ui| {
            let field = egui::TextEdit::singleline(&mut app.chat_filter)
                .hint_text("Filter chats")
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(8, 6))
                .font(theme::regular(13.0));
            let response = ui.add(field);
            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.chat_filter.clear();
            }
        });
    ui.add_space(8.0);

    let me = app.me().to_owned();
    let visible: Vec<usize> = app
        .chats
        .iter()
        .enumerate()
        .filter(|(_, chat)| !chat.hidden && matches_filter(chat, &app.chat_filter, &me))
        .map(|(index, _)| index)
        .collect();
    if visible.is_empty() {
        let (title, text) = if app.chats.is_empty() {
            if app.loading.contains(&crate::sync::Resource::Chats) {
                (
                    "Loading chats…",
                    "Your chats appear here as soon as Microsoft answers.",
                )
            } else {
                ("No chats yet", "Chats you start in Teams show up here.")
            }
        } else {
            ("No matches", "No chat or person matches the filter.")
        };
        widgets::empty_state(ui, &palette, Icon::Chat, title, text);
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
    let title = chat.title(me);
    let avatar_rect =
        egui::Rect::from_min_size(inner.left_top() + vec2(8.0, 9.0), egui::Vec2::splat(38.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(avatar_rect));
    match chat.kind {
        ChatKind::OneOnOne => {
            let peer = chat.peer(me);
            widgets::avatar(
                &mut child,
                app,
                peer.map(|p| p.user_id.as_str()),
                peer.map_or(title.as_str(), |p| p.display_name.as_str()),
                38.0,
                true,
            );
        }
        ChatKind::Group => {
            widgets::avatar(&mut child, app, None, &title, 38.0, false);
        }
        ChatKind::Meeting => {
            child.painter().rect_filled(
                avatar_rect,
                CornerRadius::same(10),
                palette.surface_active,
            );
            Icon::Calendar.image(palette.accent, 18.0).paint_at(
                &child,
                egui::Rect::from_center_size(avatar_rect.center(), egui::Vec2::splat(18.0)),
            );
        }
    }

    let text_left = avatar_rect.right() + 10.0;
    let time = chat
        .preview
        .as_ref()
        .map(|p| crate::time::short_label(p.at, now))
        .unwrap_or_default();
    let time_galley = ui.ctx().fonts_mut(|f| {
        f.layout_no_wrap(
            time,
            if unread {
                theme::semibold(12.0)
            } else {
                theme::regular(12.0)
            },
            if unread { palette.accent } else { palette.dim },
        )
    });
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

    let preview = chat.preview.as_ref().map(|p| {
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
            palette.accent,
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
