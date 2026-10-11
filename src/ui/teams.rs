//! Teams and their channels, as a collapsible tree.

use egui::{CornerRadius, Sense, vec2};

use super::widgets;
use crate::app::App;
use crate::model::{Action, ChannelKind, ConversationId};
use crate::theme::{self, Icon};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    if app.teams.is_empty() {
        let (title, text) = if app.loading.contains(&crate::sync::Resource::Teams) {
            (
                "Loading teams…",
                "Your teams appear here as soon as Microsoft answers.",
            )
        } else {
            ("No teams", "Teams you're a member of show up here.")
        };
        widgets::empty_state(ui, &palette, Icon::Teams, title, text);
        return;
    }
    let mut actions = Vec::new();
    egui::ScrollArea::vertical()
        .id_salt("teams")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            for team in &app.teams {
                let expanded = app.expanded_teams.contains(&team.id);
                let response = team_row(ui, app, &team.name, expanded);
                if response.clicked() {
                    actions.push(Action::ToggleTeam(team.id.clone()));
                }
                if !expanded {
                    continue;
                }
                match app.channels.get(&team.id) {
                    Some(channels) if !channels.is_empty() => {
                        for channel in channels {
                            let id = ConversationId::Channel {
                                team: team.id.clone(),
                                channel: channel.id.clone(),
                            };
                            let selected = app.open.as_ref() == Some(&id);
                            let response =
                                channel_row(ui, app, &channel.name, channel.kind, selected);
                            if response.clicked() {
                                actions.push(Action::OpenConversation(id));
                            }
                        }
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.add_space(52.0);
                            ui.label(
                                egui::RichText::new("Loading channels…")
                                    .color(palette.dim)
                                    .font(theme::regular(12.0)),
                            );
                        });
                    }
                }
                ui.add_space(6.0);
            }
            ui.add_space(12.0);
        });
    app.actions.extend(actions);
}

fn team_row(ui: &mut egui::Ui, app: &App, name: &str, expanded: bool) -> egui::Response {
    let palette = &app.palette;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 40.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let inner = rect.shrink2(vec2(8.0, 2.0));
        if response.hovered() {
            ui.painter()
                .rect_filled(inner, CornerRadius::same(8), palette.surface_hover);
        }
        let chevron = if expanded {
            Icon::ChevronDown
        } else {
            Icon::ChevronRight
        };
        chevron.image(palette.dim, 14.0).paint_at(
            ui,
            egui::Rect::from_center_size(
                egui::pos2(inner.left() + 12.0, inner.center().y),
                egui::Vec2::splat(14.0),
            ),
        );
        let tile = egui::Rect::from_center_size(
            egui::pos2(inner.left() + 38.0, inner.center().y),
            egui::Vec2::splat(26.0),
        );
        ui.painter().rect_filled(
            tile,
            CornerRadius::same(7),
            theme::avatar_color(name, palette.dark),
        );
        ui.painter().text(
            tile.center(),
            egui::Align2::CENTER_CENTER,
            theme::initials(name),
            theme::bold(11.0),
            egui::Color32::WHITE,
        );
        let galley = super::chat_list::elided(
            ui,
            name,
            theme::semibold(14.0),
            palette.text,
            inner.right() - tile.right() - 16.0,
        );
        ui.painter().galley(
            egui::pos2(
                tile.right() + 10.0,
                inner.center().y - galley.size().y / 2.0,
            ),
            galley,
            palette.text,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn channel_row(
    ui: &mut egui::Ui,
    app: &App,
    name: &str,
    kind: ChannelKind,
    selected: bool,
) -> egui::Response {
    let palette = &app.palette;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let inner = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 36.0, rect.top() + 1.0),
            egui::pos2(rect.right() - 8.0, rect.bottom() - 1.0),
        );
        if selected || response.hovered() {
            ui.painter().rect_filled(
                inner,
                CornerRadius::same(7),
                if selected {
                    palette.surface_active
                } else {
                    palette.surface_hover
                },
            );
        }
        let icon = match kind {
            ChannelKind::Private => Icon::Lock,
            ChannelKind::Shared => Icon::Teams,
            ChannelKind::Standard => Icon::Hash,
        };
        let tint = if selected {
            palette.accent
        } else {
            palette.dim
        };
        icon.image(tint, 14.0).paint_at(
            ui,
            egui::Rect::from_center_size(
                egui::pos2(inner.left() + 16.0, inner.center().y),
                egui::Vec2::splat(14.0),
            ),
        );
        let galley = super::chat_list::elided(
            ui,
            name,
            if selected {
                theme::semibold(13.0)
            } else {
                theme::regular(13.0)
            },
            if selected {
                palette.text
            } else {
                palette.secondary
            },
            inner.width() - 40.0,
        );
        ui.painter().galley(
            egui::pos2(
                inner.left() + 32.0,
                inner.center().y - galley.size().y / 2.0,
            ),
            galley,
            palette.text,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
