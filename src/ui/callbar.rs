//! The call bar: the call or meeting Wren follows, and what to do next.
//!
//! Docked beside a call window, it sits at the top of Wren's narrow panel,
//! under the macOS traffic lights. Otherwise it runs along the bottom of the
//! main area.

use egui::{Frame, Margin, Stroke, vec2};

use super::widgets;
use crate::app::App;
use crate::calling::CallsOpenIn;
use crate::calling::session::{CallState, CallTarget, ChatLookup};
use crate::model::Action;
use crate::theme::{self, Icon};

pub fn panel(app: &mut App, ui: &mut egui::Ui, docked: bool) {
    let palette = app.palette;
    let panel = if docked {
        egui::Panel::top("callbar")
    } else {
        egui::Panel::bottom("callbar")
    };
    panel
        .resizable(false)
        .show_separator_line(false)
        .frame(
            Frame::new()
                .fill(palette.panel)
                .inner_margin(Margin::symmetric(14, if docked { 10 } else { 8 })),
        )
        .show(ui, |ui| {
            let rect = ui
                .max_rect()
                .expand2(vec2(14.0, if docked { 10.0 } else { 8.0 }));
            if docked {
                widgets::window_drag(ui, rect, "callbar-drag");
                ui.painter().hline(
                    rect.x_range(),
                    rect.bottom(),
                    Stroke::new(1.0, palette.outline),
                );
            } else {
                ui.painter().hline(
                    rect.x_range(),
                    rect.top(),
                    Stroke::new(1.0, palette.outline),
                );
            }
            content(app, ui, docked);
        });
}

fn content(app: &mut App, ui: &mut egui::Ui, docked: bool) {
    let palette = app.palette;
    let now = crate::time::now();
    let inset = if docked {
        super::header_inset(ui.ctx(), ui.max_rect().left())
    } else {
        0.0
    };
    let mut actions = Vec::new();
    let Some(call) = app.call.clone() else {
        // Docked without a call (it was dismissed elsewhere): offer the way back.
        ui.horizontal(|ui| {
            ui.add_space(inset);
            ui.label(
                egui::RichText::new("No call")
                    .font(theme::semibold(13.0))
                    .color(palette.text),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let restore = widgets::pill(ui, &palette, Icon::Monitor, "Restore", false);
                widgets::on_click(&restore, &mut actions, Action::DockWindow(false));
            });
        });
        app.actions.extend(actions);
        return;
    };

    let dot = match call.state {
        CallState::Opening => palette.warning,
        CallState::Live { .. } | CallState::Leaving { .. } => palette.success,
        CallState::Ended { .. } => palette.dim,
    };
    let mut status = call.status(now);
    if call.lookup == ChatLookup::Searching {
        status.push_str(" · finding the meeting chat…");
    }

    // The buttons, in the order they matter.
    let ended = call.is_ended();
    let elsewhere = call.target == CallTarget::Elsewhere;
    let mut buttons: Vec<(Icon, &str, Action, bool, &str)> = Vec::new();
    if !ended && !elsewhere && app.can_show_call() {
        buttons.push((
            Icon::Video,
            "Show call",
            Action::ShowCall,
            false,
            "Bring the call window to the front",
        ));
    }
    if !ended
        && matches!(
            call.lookup,
            ChatLookup::NeedsPermission | ChatLookup::NotFound
        )
    {
        buttons.push((
            Icon::Chat,
            "Meeting chat",
            Action::FindMeetingChat,
            false,
            "Find this meeting's chat and show it here",
        ));
    }
    if !ended && call.join_url().is_some() {
        buttons.push((
            Icon::Phone,
            "Join on phone",
            Action::JoinOnPhone,
            false,
            "Show the join link as a QR code",
        ));
    }
    if docked {
        buttons.push((
            Icon::Monitor,
            "Restore",
            Action::DockWindow(false),
            false,
            "Give Wren its full window back",
        ));
    } else if !ended && !elsewhere && app.settings.calls.open_in == CallsOpenIn::AppWindow {
        buttons.push((
            Icon::Monitor,
            "Dock",
            Action::DockWindow(true),
            false,
            "Move Wren into a narrow panel beside the call",
        ));
    }
    buttons.push((
        Icon::Check,
        if elsewhere { "Dismiss" } else { "Done" },
        Action::CallDone,
        !elsewhere,
        "Close the call bar. The call itself goes on in Teams until you leave it there.",
    ));

    let title = |ui: &mut egui::Ui, width: f32| {
        let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.5, dot);
        ui.add(egui::Label::new(super::chat_list::elided(
            ui,
            &call.title(),
            theme::semibold(13.0),
            palette.text,
            width,
        )));
    };
    if docked {
        ui.horizontal(|ui| {
            ui.add_space(inset);
            title(ui, (ui.available_width() - 20.0).max(60.0));
        });
        ui.horizontal(|ui| {
            ui.add_space(inset + 18.0);
            ui.label(
                egui::RichText::new(&status)
                    .font(theme::regular(12.0))
                    .color(palette.secondary),
            );
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for (icon, label, action, accent, tip) in buttons {
                let response = widgets::pill(ui, &palette, icon, label, accent).on_hover_text(tip);
                widgets::on_click(&response, &mut actions, action);
            }
        });
    } else {
        ui.horizontal(|ui| {
            title(ui, 320.0);
            ui.label(
                egui::RichText::new(&status)
                    .font(theme::regular(12.0))
                    .color(palette.secondary),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for (icon, label, action, accent, tip) in buttons.into_iter().rev() {
                    let response =
                        widgets::pill(ui, &palette, icon, label, accent).on_hover_text(tip);
                    widgets::on_click(&response, &mut actions, action);
                }
            });
        });
    }
    app.actions.extend(actions);
}
