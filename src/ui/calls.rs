//! Calls: start a call with someone, join a meeting by link, and an honest
//! account of what calling can and cannot do today.

use egui::{CornerRadius, Frame, Margin, Sense, Stroke, vec2};

use super::widgets;
use crate::app::App;
use crate::calling::CAPABILITIES;
use crate::model::{Action, ChatKind};
use crate::theme::{self, Icon};

/// People from one-on-one chats, most recent first.
pub fn list(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let me = app.me().to_owned();
    let people: Vec<(String, String, String)> = app
        .chats
        .iter()
        .filter(|c| c.kind == ChatKind::OneOnOne)
        .filter_map(|c| c.peer(&me))
        .filter_map(|m| Some((m.user_id.clone(), m.display_name.clone(), m.email.clone()?)))
        .take(40)
        .collect();
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        widgets::section_label(ui, &palette, "Recent people");
    });
    ui.add_space(4.0);
    if people.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            ui.label(egui::RichText::new("People you chat with appear here.").color(palette.dim));
        });
        return;
    }
    let mut actions = Vec::new();
    egui::ScrollArea::vertical()
        .id_salt("call-people")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (user, name, email) in &people {
                let width = ui.available_width();
                let (rect, response) = ui.allocate_exact_size(vec2(width, 48.0), Sense::hover());
                let inner = rect.shrink2(vec2(8.0, 2.0));
                if response.hovered() {
                    ui.painter()
                        .rect_filled(inner, CornerRadius::same(8), palette.surface_hover);
                }
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(inner.shrink2(vec2(8.0, 0.0)))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                widgets::avatar(&mut child, app, Some(user), name, 32.0, true);
                child.add_space(8.0);
                let label_width = (child.available_width() - 72.0).max(40.0);
                child.add(egui::Label::new(super::chat_list::elided(
                    &child,
                    name,
                    theme::medium(13.0),
                    palette.text,
                    label_width,
                )));
                child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let video = widgets::icon_button(ui, &palette, Icon::Video, "Video call");
                    widgets::on_click(
                        &video,
                        &mut actions,
                        Action::StartCall {
                            users: vec![email.clone()],
                            video: true,
                        },
                    );
                    let audio = widgets::icon_button(ui, &palette, Icon::Phone, "Audio call");
                    widgets::on_click(
                        &audio,
                        &mut actions,
                        Action::StartCall {
                            users: vec![email.clone()],
                            video: false,
                        },
                    );
                });
            }
        });
    app.actions.extend(actions);
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let header = egui::Rect::from_min_size(
        ui.max_rect().min,
        vec2(ui.max_rect().width(), theme::HEADER_HEIGHT),
    );
    widgets::window_drag(ui, header, "calls-drag");
    ui.add_space(theme::HEADER_HEIGHT + 12.0);
    egui::ScrollArea::vertical().id_salt("calls-page").auto_shrink([false, false]).show(ui, |ui| {
        Frame::new().inner_margin(Margin::symmetric(32, 0)).show(ui, |ui| {
            ui.set_max_width(680.0);
            ui.label(egui::RichText::new("Calls and meetings").font(theme::bold(24.0)).color(palette.text));
            ui.add_space(6.0);
            let place = match app.opener() {
                crate::calling::Opener::AppWindow(browser) => format!("Teams on the web, in a {} window beside Wren", browser.kind.short()),
                crate::calling::Opener::Browser => "Teams on the web, in your default browser".to_owned(),
                crate::calling::Opener::TeamsApp => "the Microsoft Teams app".to_owned(),
            };
            ui.label(egui::RichText::new(format!("Calls and meetings open in {place}. The call itself runs in Microsoft's own client, so it costs nothing extra; Wren shows the meeting chat and follows the call. Change this in Settings.")).color(palette.secondary));
            ui.add_space(18.0);

            // Join by link.
            let link_id = egui::Id::new("wren-join-link");
            let mut link: String = ui.ctx().data(|d| d.get_temp(link_id)).unwrap_or_default();
            Frame::new()
                .fill(palette.panel)
                .stroke(Stroke::new(1.0, palette.outline))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(egui::RichText::new("Join a meeting").font(theme::semibold(15.0)).color(palette.text));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let field = egui::TextEdit::singleline(&mut link)
                            .hint_text("Paste a Teams meeting link")
                            .desired_width(ui.available_width() - 90.0)
                            .margin(Margin::symmetric(8, 6));
                        let response = ui.add(field);
                        let valid = crate::calling::is_teams_link(link.trim());
                        let join = ui.add_enabled(valid, egui::Button::new("Join").min_size(vec2(72.0, 30.0)));
                        let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if valid && (join.clicked() || enter) {
                            app.actions.push(Action::JoinMeeting(link.trim().to_owned()));
                            link.clear();
                        }
                    });
                });
            ui.ctx().data_mut(|d| d.insert_temp(link_id, link));
            ui.add_space(18.0);

            ui.label(egui::RichText::new("What works today").font(theme::semibold(15.0)).color(palette.text));
            ui.add_space(6.0);
            egui::Grid::new("call-capabilities").num_columns(3).spacing(vec2(16.0, 10.0)).striped(false).show(ui, |ui| {
                for capability in CAPABILITIES {
                    ui.label(egui::RichText::new(capability.name).font(theme::medium(13.0)).color(palette.text));
                    let color = match capability.status {
                        "Works" => palette.success,
                        "Not possible" => palette.danger,
                        _ => palette.warning,
                    };
                    ui.label(egui::RichText::new(capability.status).font(theme::semibold(12.0)).color(color));
                    ui.add(egui::Label::new(egui::RichText::new(capability.detail).font(theme::regular(12.0)).color(palette.secondary)).wrap());
                    ui.end_row();
                }
            });
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("Why: Microsoft Graph's calling API works only for bots. The Teams licence covers calls in Microsoft's own clients only; Azure Communication Services would bill every minute. The full investigation is in docs/calling.md and docs/video-calling-options.md.")
                    .font(theme::regular(12.0))
                    .color(palette.dim),
            );
        });
    });
}
