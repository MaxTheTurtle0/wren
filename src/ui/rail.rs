//! The navigation rail: Chat, Teams, Calendar, Calls, and the account.

use egui::{CornerRadius, Frame, Sense, vec2};

use super::widgets;
use crate::app::App;
use crate::model::{Action, Section};
use crate::theme::{self, Icon};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    egui::Panel::left("rail")
        .exact_size(theme::RAIL_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .frame(Frame::new().fill(palette.rail))
        .show(ui, |ui| {
            let top = egui::Rect::from_min_size(
                ui.max_rect().min,
                vec2(theme::RAIL_WIDTH, theme::HEADER_HEIGHT),
            );
            widgets::window_drag(ui, top, "rail-drag");
            ui.add_space(theme::HEADER_HEIGHT + 4.0);
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                for section in Section::ALL {
                    let badge = match section {
                        Section::Chats => app.unread_chats(),
                        _ => 0,
                    };
                    let response = item(ui, app, section, badge);
                    widgets::on_click(&response, &mut app.actions, Action::Navigate(section));
                }
            });
            // The account and settings at the bottom.
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(12.0);
                let name = app
                    .profile
                    .as_ref()
                    .map(|p| p.display_name.clone())
                    .unwrap_or_else(|| "You".into());
                let me = app.me().to_owned();
                let avatar = widgets::avatar(
                    ui,
                    app,
                    Some(&me).filter(|m| !m.is_empty()).map(String::as_str),
                    &name,
                    32.0,
                    true,
                )
                .on_hover_text(format!("{name}\nSettings and account"))
                .on_hover_cursor(egui::CursorIcon::PointingHand);
                widgets::on_click(&avatar, &mut app.actions, Action::OpenSettings);
                ui.add_space(6.0);
                let settings = widgets::icon_button_sized(
                    ui,
                    &palette,
                    Icon::Settings,
                    "Settings (⌘,)",
                    36.0,
                    20.0,
                    palette.secondary,
                );
                widgets::on_click(&settings, &mut app.actions, Action::OpenSettings);
                if app.demo {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("DEMO")
                            .font(theme::bold(11.0))
                            .color(palette.accent),
                    )
                    .on_hover_text("Sample data. Nothing is sent anywhere.");
                }
            });
        });
}

fn item(ui: &mut egui::Ui, app: &App, section: Section, badge: usize) -> egui::Response {
    let palette = &app.palette;
    let (rect, response) = ui.allocate_exact_size(vec2(56.0, 52.0), Sense::click());
    let active = app.section == section;
    if ui.is_rect_visible(rect) {
        if active || response.hovered() {
            ui.painter().rect_filled(
                rect.shrink2(vec2(2.0, 0.0)),
                CornerRadius::same(10),
                if active {
                    palette.surface_active
                } else {
                    palette.surface_hover
                },
            );
        }
        if active {
            let bar = egui::Rect::from_min_size(
                egui::pos2(rect.left() - 4.0, rect.top() + 14.0),
                vec2(3.0, 24.0),
            );
            ui.painter()
                .rect_filled(bar, CornerRadius::same(2), palette.accent);
        }
        let icon = match section {
            Section::Chats => Icon::Chat,
            Section::Teams => Icon::Teams,
            Section::Calendar => Icon::Calendar,
            Section::Calls => Icon::Phone,
        };
        let tint = if active {
            palette.accent
        } else {
            palette.secondary
        };
        let icon_rect = egui::Rect::from_center_size(
            rect.center_top() + vec2(0.0, 19.0),
            egui::Vec2::splat(21.0),
        );
        icon.image(tint, 21.0).paint_at(ui, icon_rect);
        ui.painter().text(
            rect.center_bottom() - vec2(0.0, 9.0),
            egui::Align2::CENTER_CENTER,
            section.label(),
            theme::medium(11.0),
            if active {
                palette.text
            } else {
                palette.secondary
            },
        );
        if badge > 0 {
            widgets::badge(ui, palette, icon_rect.right_top() + vec2(3.0, 1.0), badge);
        }
    }
    let shortcut = match section {
        Section::Chats => "⌘1",
        Section::Teams => "⌘2",
        Section::Calendar => "⌘3",
        Section::Calls => "⌘4",
    };
    response
        .on_hover_text(format!("{} ({shortcut})", section.label()))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}
