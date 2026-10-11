//! The navigation rail: Chat, Teams, Calendar, Calls, and the account.

use egui::{CornerRadius, Frame, Sense, vec2};

use crate::model::Availability;

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
                .on_hover_text(format!("{name}\nAccounts and status"))
                .on_hover_cursor(egui::CursorIcon::PointingHand);
                // Another account has unread chats.
                if app.other_unread() > 0 {
                    ui.painter().circle_filled(
                        avatar.rect.right_top() + vec2(-3.0, 3.0),
                        4.5,
                        palette.accent,
                    );
                }
                if std::mem::take(&mut app.open_account_menu) {
                    egui::Popup::open_id(ui.ctx(), egui::Popup::default_response_id(&avatar));
                }
                account_menu(app, &avatar);
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

/// The avatar's menu: accounts, availability, status message, settings.
fn account_menu(app: &mut App, avatar: &egui::Response) {
    let palette = app.palette;
    let mut actions = Vec::new();
    egui::Popup::menu(avatar)
        .align(egui::RectAlign::RIGHT_END)
        .gap(8.0)
        .width(280.0)
        .show(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            // Accounts.
            for (index, account) in app.accounts.iter().enumerate() {
                let active = index == app.active;
                let (rect, response) = ui.allocate_exact_size(vec2(268.0, 46.0), Sense::click());
                if response.hovered() {
                    ui.painter()
                        .rect_filled(rect, CornerRadius::same(8), palette.surface_hover);
                }
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(rect.shrink2(vec2(8.0, 0.0)))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                let label = account.label();
                let me = account.me().to_owned();
                // Avatars draw from the shown account's photos; others use
                // initials.
                widgets::avatar(
                    &mut child,
                    app,
                    if active {
                        Some(me.as_str()).filter(|m| !m.is_empty())
                    } else {
                        None
                    },
                    &label,
                    30.0,
                    active,
                );
                child.add_space(8.0);
                child.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(
                        egui::RichText::new(&label)
                            .font(theme::semibold(13.0))
                            .color(palette.text),
                    );
                    let detail = match &account.auth {
                        crate::backend::AuthStatus::SignedIn { username, .. } => username.clone(),
                        crate::backend::AuthStatus::Expired { .. } => "Sign in again".into(),
                        _ => "Signing in…".into(),
                    };
                    ui.label(
                        egui::RichText::new(detail)
                            .font(theme::regular(11.0))
                            .color(palette.dim),
                    );
                });
                child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if active {
                        theme::icon(ui, Icon::Check, 16.0, palette.accent);
                    } else if account.unread_chats() > 0 {
                        ui.label(
                            egui::RichText::new(account.unread_chats().to_string())
                                .font(theme::bold(11.0))
                                .color(palette.accent),
                        );
                    }
                });
                if response.clicked() && !active {
                    actions.push(Action::SwitchAccount(index));
                    ui.close();
                }
            }
            if ui
                .add(
                    egui::Button::new(egui::RichText::new("+  Add account").color(palette.text))
                        .frame(false),
                )
                .clicked()
            {
                actions.push(Action::AddAccount);
                ui.close();
            }
            ui.separator();
            // Availability.
            let current = app.my_presence.as_ref().map(|p| p.availability);
            for (availability, label) in [
                (Some(Availability::Available), "Available"),
                (Some(Availability::Busy), "Busy"),
                (Some(Availability::DoNotDisturb), "Do not disturb"),
                (Some(Availability::BeRightBack), "Be right back"),
                (Some(Availability::Away), "Appear away"),
                (Some(Availability::Offline), "Appear offline"),
                (None, "Reset status"),
            ] {
                let (rect, response) = ui.allocate_exact_size(vec2(268.0, 28.0), Sense::click());
                if response.hovered() {
                    ui.painter()
                        .rect_filled(rect, CornerRadius::same(6), palette.surface_hover);
                }
                let dot = egui::pos2(rect.left() + 16.0, rect.center().y);
                match availability {
                    Some(availability) => {
                        ui.painter()
                            .circle_filled(dot, 5.0, palette.presence(availability));
                    }
                    None => {
                        Icon::Refresh.image(palette.dim, 12.0).paint_at(
                            ui,
                            egui::Rect::from_center_size(dot, egui::Vec2::splat(12.0)),
                        );
                    }
                }
                ui.painter().text(
                    egui::pos2(rect.left() + 32.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    theme::regular(13.0),
                    palette.text,
                );
                if availability.is_some() && availability == current {
                    Icon::Check.image(palette.accent, 14.0).paint_at(
                        ui,
                        egui::Rect::from_center_size(
                            egui::pos2(rect.right() - 14.0, rect.center().y),
                            egui::Vec2::splat(14.0),
                        ),
                    );
                }
                if response.clicked() {
                    actions.push(Action::SetAvailability(availability));
                    ui.close();
                }
            }
            let message = app
                .my_presence
                .as_ref()
                .and_then(|p| p.status_message.clone())
                .unwrap_or_else(|| "Set a status message".into());
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(format!("✎  {}", crate::html::truncate(&message, 34)))
                            .color(palette.secondary),
                    )
                    .frame(false),
                )
                .clicked()
            {
                actions.push(Action::OpenStatusMessage);
                ui.close();
            }
            ui.separator();
            if ui
                .add(
                    egui::Button::new(egui::RichText::new("Settings…").color(palette.text))
                        .frame(false),
                )
                .clicked()
            {
                actions.push(Action::OpenSettings);
                ui.close();
            }
        });
    app.actions.extend(actions);
}
