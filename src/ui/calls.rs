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
                let compact = width < 260.0;
                let label_width =
                    (child.available_width() - if compact { 56.0 } else { 72.0 }).max(40.0);
                child.add(egui::Label::new(super::chat_list::elided(
                    &child,
                    name,
                    theme::medium(13.0),
                    palette.text,
                    label_width,
                )));
                child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Narrow lists use tighter buttons, so names keep room.
                    let button = if compact { 26.0 } else { 30.0 };
                    let video = widgets::icon_button_sized(
                        ui,
                        &palette,
                        Icon::Video,
                        "Video call",
                        button,
                        16.0,
                        palette.secondary,
                    );
                    widgets::on_click(
                        &video,
                        &mut actions,
                        Action::StartCall {
                            users: vec![email.clone()],
                            video: true,
                        },
                    );
                    let audio = widgets::icon_button_sized(
                        ui,
                        &palette,
                        Icon::Phone,
                        "Audio call",
                        button,
                        16.0,
                        palette.secondary,
                    );
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
    widgets::pane_header(
        ui,
        &palette,
        "Calls and meetings",
        "Join by link, or call someone from the list",
        |_| {},
    );
    egui::ScrollArea::vertical().id_salt("calls-page").auto_shrink([false, false]).show(ui, |ui| {
        Frame::new().inner_margin(Margin { left: 24, right: 24, top: 20, bottom: 0 }).show(ui, |ui| {
            ui.set_max_width(680.0f32.min(ui.available_width()));
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
                        let field = widgets::text_field(&mut link, "Paste a Teams meeting link", theme::regular(13.0))
                            .desired_width(ui.available_width() - 90.0);
                        let response = ui.add(field);
                        let valid = crate::calling::is_teams_link(link.trim());
                        let join = ui.add_enabled_ui(valid, |ui| widgets::primary_button(ui, &palette, "Join")).inner;
                        let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if valid && (join.clicked() || enter) {
                            app.actions.push(Action::JoinMeeting(link.trim().to_owned()));
                            link.clear();
                        }
                    });
                });
            ui.ctx().data_mut(|d| d.insert_temp(link_id, link));
            ui.add_space(18.0);

            ui.label(egui::RichText::new("How calls work").font(theme::semibold(15.0)).color(palette.text));
            ui.add_space(4.0);
            let place = match app.opener() {
                crate::calling::Opener::AppWindow(browser) => format!("Teams on the web, in its own {} window beside Wren", browser.kind.short()),
                crate::calling::Opener::Browser => "Teams on the web, in your default browser".to_owned(),
                crate::calling::Opener::TeamsApp => "the Microsoft Teams app".to_owned(),
            };
            ui.label(egui::RichText::new(format!("Calls open in {place}. Microsoft's own client runs the call, so it costs nothing extra; Wren shows the meeting chat and follows the call.")).color(palette.secondary));
            ui.add_space(10.0);
            // What works, at a glance; the details on request.
            for (status, color, icon) in [("Works", palette.success, Icon::Check), ("Not possible", palette.secondary, Icon::Close)] {
                // "Join meetings, start audio or video calls, call status"
                let names: Vec<String> = CAPABILITIES
                    .iter()
                    .filter(|c| c.status == status)
                    .enumerate()
                    .map(|(index, c)| {
                        let mut chars = c.name.chars();
                        match chars.next() {
                            Some(first) if index > 0 => first.to_lowercase().chain(chars).collect(),
                            _ => c.name.to_owned(),
                        }
                    })
                    .collect();
                if names.is_empty() {
                    continue;
                }
                // Icon in its own column; wrapped lines hang under the text.
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    theme::icon(ui, icon, 14.0, color);
                    let mut job = egui::text::LayoutJob::default();
                    job.append(&format!("{status}: "), 0.0, egui::TextFormat::simple(theme::semibold(13.0), color));
                    job.append(&names.join(", "), 0.0, egui::TextFormat::simple(theme::regular(13.0), palette.text));
                    ui.add(egui::Label::new(job).wrap());
                });
            }
            ui.add_space(4.0);
            egui::CollapsingHeader::new(egui::RichText::new("Details").font(theme::medium(13.0)).color(palette.secondary))
                .id_salt("call-capabilities")
                .icon(widgets::disclosure_chevron(palette.secondary))
                .default_open(false)
                .show(ui, |ui| {
                    for capability in CAPABILITIES {
                        let (color, icon) = match capability.status {
                            "Works" => (palette.success, Icon::Check),
                            "Not possible" => (palette.danger, Icon::Close),
                            _ => (palette.warning, Icon::Info),
                        };
                        Frame::new()
                            .inner_margin(Margin::symmetric(0, 6))
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(egui::RichText::new(capability.name).font(theme::semibold(13.0)).color(palette.text));
                                    ui.add_space(4.0);
                                    theme::icon(ui, icon, 14.0, color);
                                    ui.label(egui::RichText::new(capability.status).font(theme::semibold(12.0)).color(color));
                                });
                                ui.add(egui::Label::new(egui::RichText::new(capability.detail).font(theme::regular(13.0)).color(palette.secondary)).wrap());
                            });
                    }
                });
            ui.add_space(24.0);
        });
    });
}
