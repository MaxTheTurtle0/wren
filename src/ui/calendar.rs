//! The next two weeks of meetings: an agenda in the sidebar, the selected
//! meeting on the right with its Join button.

use egui::{CornerRadius, Frame, Margin, Sense, Stroke, vec2};

use super::widgets;
use crate::app::App;
use crate::model::{Action, Meeting};
use crate::theme::{self, Icon};

pub fn list(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let now = crate::time::now();
    let upcoming: Vec<Meeting> = app
        .meetings
        .iter()
        .filter(|m| m.end > now && !m.cancelled)
        .cloned()
        .collect();
    if upcoming.is_empty() {
        let (title, text) = if app.loading.contains(&crate::sync::Resource::Calendar) {
            ("Loading your calendar…", "")
        } else {
            ("Nothing scheduled", "No meetings in the next two weeks.")
        };
        widgets::empty_state(ui, &palette, Icon::Calendar, title, text);
        return;
    }
    let mut actions = Vec::new();
    egui::ScrollArea::vertical()
        .id_salt("agenda")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut day = None;
            for meeting in &upcoming {
                let key = crate::time::day_key(meeting.start);
                if day != Some(key) {
                    day = Some(key);
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.add_space(18.0);
                        widgets::section_label(
                            ui,
                            &palette,
                            &crate::time::day_label(meeting.start, now),
                        );
                    });
                    ui.add_space(4.0);
                }
                let selected = app.selected_meeting.as_deref() == Some(meeting.id.as_str());
                let response = row(ui, app, meeting, selected, now);
                if response.clicked() {
                    app.selected_meeting = Some(meeting.id.clone());
                }
                if response.double_clicked()
                    && let Some(url) = &meeting.join_url
                {
                    actions.push(Action::JoinMeeting(url.clone()));
                }
            }
            ui.add_space(12.0);
        });
    if app.selected_meeting.is_none() {
        app.selected_meeting = upcoming.first().map(|m| m.id.clone());
    }
    app.actions.extend(actions);
}

fn row(
    ui: &mut egui::Ui,
    app: &App,
    meeting: &Meeting,
    selected: bool,
    now: i64,
) -> egui::Response {
    let palette = &app.palette;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 54.0), Sense::click());
    if ui.is_rect_visible(rect) {
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
        let live = meeting.start <= now && meeting.end > now;
        let bar = egui::Rect::from_min_size(inner.left_top() + vec2(8.0, 9.0), vec2(3.0, 34.0));
        ui.painter().rect_filled(
            bar,
            CornerRadius::same(2),
            if live {
                palette.success
            } else if meeting.join_url.is_some() {
                palette.accent
            } else {
                palette.dim
            },
        );
        let left = bar.right() + 10.0;
        let title = super::chat_list::elided(
            ui,
            &meeting.subject,
            theme::semibold(13.0),
            palette.text,
            inner.right() - left - 10.0,
        );
        ui.painter()
            .galley(egui::pos2(left, inner.top() + 8.0), title, palette.text);
        let mut detail = crate::time::span_label(meeting.start, meeting.end, meeting.all_day);
        if live {
            detail.push_str(" · now");
        } else if meeting.start > now && meeting.start - now < 60 * 60 * 1000 {
            detail.push_str(&format!(
                " · {}",
                crate::time::until_label(meeting.start, now)
            ));
        }
        if let Some(location) = &meeting.location {
            detail.push_str(&format!(" · {location}"));
        }
        let detail = super::chat_list::elided(
            ui,
            &detail,
            theme::regular(12.0),
            palette.secondary,
            inner.right() - left - 10.0,
        );
        ui.painter().galley(
            egui::pos2(left, inner.top() + 28.0),
            detail,
            palette.secondary,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn detail(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let header = egui::Rect::from_min_size(
        ui.max_rect().min,
        vec2(ui.max_rect().width(), theme::HEADER_HEIGHT),
    );
    widgets::window_drag(ui, header, "calendar-drag");
    let Some(meeting) = app
        .selected_meeting
        .as_ref()
        .and_then(|id| app.meetings.iter().find(|m| &m.id == id))
        .cloned()
    else {
        widgets::empty_state(
            ui,
            &palette,
            Icon::Calendar,
            "Your calendar",
            "Upcoming meetings from Outlook appear on the left. Join Teams meetings with one click.",
        );
        return;
    };
    let now = crate::time::now();
    ui.add_space(theme::HEADER_HEIGHT + 12.0);
    Frame::new().inner_margin(Margin::symmetric(32, 0)).show(ui, |ui| {
        ui.set_max_width(640.0);
        ui.label(egui::RichText::new(&meeting.subject).font(theme::bold(24.0)).color(palette.text));
        ui.add_space(6.0);
        let when = format!(
            "{} · {}",
            crate::time::day_label(meeting.start, now),
            crate::time::span_label(meeting.start, meeting.end, meeting.all_day)
        );
        ui.label(egui::RichText::new(when).font(theme::medium(14.0)).color(palette.secondary));
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if let Some(url) = &meeting.join_url {
                let join = widgets::pill(ui, &palette, Icon::Video, "Join in Teams", true);
                widgets::on_click(&join, &mut app.actions, Action::JoinMeeting(url.clone()));
                let copy = widgets::pill(ui, &palette, Icon::Copy, "Copy link", false);
                widgets::on_click(&copy, &mut app.actions, Action::CopyText(url.clone()));
            }
            if let Some(web) = &meeting.web_link {
                let open = widgets::pill(ui, &palette, Icon::ExternalLink, "Open in Outlook", false);
                widgets::on_click(&open, &mut app.actions, Action::OpenUrl(web.clone()));
            }
        });
        ui.add_space(20.0);
        Frame::new()
            .fill(palette.panel)
            .stroke(Stroke::new(1.0, palette.outline))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::Grid::new("meeting-facts").num_columns(2).spacing(vec2(18.0, 10.0)).show(ui, |ui| {
                    let fact = |ui: &mut egui::Ui, label: &str, value: &str| {
                        ui.label(egui::RichText::new(label).color(palette.dim));
                        ui.label(egui::RichText::new(value).color(palette.text));
                        ui.end_row();
                    };
                    if let Some(organizer) = &meeting.organizer {
                        fact(ui, "Organizer", organizer);
                    }
                    if let Some(location) = &meeting.location {
                        fact(ui, "Location", location);
                    }
                    fact(ui, "Online", if meeting.join_url.is_some() { "Microsoft Teams meeting" } else { "No" });
                    if meeting.start > now {
                        fact(ui, "Starts", &crate::time::until_label(meeting.start, now));
                    }
                });
                if let Some(preview) = &meeting.preview {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(preview).color(palette.secondary));
                }
            });
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new("Joining opens the meeting in Microsoft Teams (the app if installed, otherwise Teams on the web). Wren reminds you five minutes before online meetings start.")
                .font(theme::regular(12.0))
                .color(palette.dim),
        );
    });
}
