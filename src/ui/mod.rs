//! The window: navigation rail, sidebar, the open conversation or page, and
//! overlays. Every view reads [`App`] and pushes actions.

pub mod calendar;
pub mod callbar;
pub mod calls;
pub mod chat_list;
pub mod conversation;
pub mod keys;
pub mod overlays;
pub mod rail;
pub mod rich;
pub mod signin;
pub mod teams;
pub mod widgets;

use egui::{Align2, CornerRadius, Frame, Margin, Stroke, vec2};

use crate::app::App;
use crate::model::{Action, Section, ToastKind};
use crate::theme::{self, Icon};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    keys::handle(app, &ctx);
    if matches!(app.update, crate::updates::Status::Ready(_)) {
        egui::Panel::bottom("update-ready")
            .frame(
                Frame::new()
                    .fill(app.palette.surface)
                    .inner_margin(Margin::symmetric(16, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("An app update is ready.");
                    if ui.button("Restart to update").clicked() {
                        app.actions.push(Action::InstallUpdate);
                    }
                });
            });
    }
    if app.shows_workspace() && app.docked.is_some() {
        docked(app, ui);
    } else if app.shows_workspace() {
        rail::show(app, ui);
        sidebar(app, ui);
        if app.call.is_some() {
            callbar::panel(app, ui, false);
        }
        main(app, ui);
    } else {
        signin::show(app, ui);
    }
    drop_files(app, &ctx);
    overlays::show(app, &ctx);
    toasts(app, &ctx);
    if app.settings.perf_overlay {
        perf(app, &ctx);
    }
    if !app.demo {
        widgets::request_photos(app, &ctx);
    }
}

/// Files dropped on the window go to the open conversation's draft; while
/// they hover, a hint says where they will go.
fn drop_files(app: &mut App, ctx: &egui::Context) {
    let (dropped, hovering) = ctx.input(|input| {
        let dropped: Vec<std::path::PathBuf> = input
            .raw
            .dropped_files
            .iter()
            .map(|file| file.path().to_path_buf())
            .collect();
        (dropped, !input.raw.hovered_files.is_empty())
    });
    if !dropped.is_empty() {
        app.actions.push(Action::AddFiles(dropped));
    }
    if hovering && app.open.is_some() && app.shows_workspace() {
        let palette = app.palette;
        let title = app
            .open
            .as_ref()
            .map(|open| app.title_of(open))
            .unwrap_or_default();
        egui::Area::new(egui::Id::new("wren-drop"))
            .order(egui::Order::Foreground)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .interactable(false)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(palette.surface)
                    .stroke(Stroke::new(2.0, palette.accent))
                    .corner_radius(CornerRadius::same(14))
                    .inner_margin(Margin::symmetric(28, 18))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("Drop to send to {title}"))
                                .font(theme::semibold(15.0))
                                .color(palette.text),
                        );
                    });
            });
    }
}

/// Beside a call window: the call bar on top, then the call's chat (or the
/// open conversation), with no rail or sidebar.
fn docked(app: &mut App, ui: &mut egui::Ui) {
    callbar::panel(app, ui, true);
    let palette = app.palette;
    let chat = app
        .call
        .as_ref()
        .and_then(|call| call.chat.clone())
        .or_else(|| app.open.clone());
    egui::CentralPanel::no_frame()
        .frame(Frame::new().fill(palette.window))
        .show(ui, |ui| match chat {
            Some(chat) => conversation::show(app, ui, chat),
            None => widgets::empty_state(
                ui,
                &palette,
                Icon::Chat,
                "No chat for this call",
                "A meeting's chat shows here once Wren finds it. Restore the window to see all your chats.",
            ),
        });
}

/// Left padding for a header that sits under the macOS traffic lights.
pub fn header_inset(ctx: &egui::Context, x: f32) -> f32 {
    (fastframe_titlebar::footprint(ctx, theme::HEADER_HEIGHT).left - x).max(0.0)
}

/// Below this window width the sidebar narrows, so the open conversation
/// keeps room to read.
const NARROW_WINDOW: f32 = 960.0;
const NARROW_SIDEBAR: f32 = 232.0;

fn sidebar(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let narrow = ui.ctx().content_rect().width() < NARROW_WINDOW;
    let range = if narrow {
        220.0..=NARROW_SIDEBAR
    } else {
        220.0..=520.0
    };
    let response = egui::Panel::left("sidebar")
        .resizable(!narrow)
        .default_size(app.settings.sidebar_width)
        .size_range(range)
        .show_separator_line(false)
        .frame(Frame::new().fill(palette.panel))
        .show(ui, |ui| {
            let header = egui::Rect::from_min_size(
                ui.max_rect().min,
                vec2(ui.max_rect().width(), theme::HEADER_HEIGHT),
            );
            widgets::window_drag(ui, header, "sidebar-drag");
            let inset = header_inset(ui.ctx(), theme::RAIL_WIDTH);
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(header.shrink2(vec2(12.0, 0.0))),
                |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(inset + 4.0);
                        ui.label(
                            egui::RichText::new(app.section.label())
                                .font(theme::bold(18.0))
                                .color(palette.text),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let refresh = widgets::icon_button(
                                ui,
                                &palette,
                                Icon::Refresh,
                                &format!("Refresh ({})", keys::shortcut("⌘R")),
                            );
                            widgets::on_click(&refresh, &mut app.actions, Action::Refresh);
                            if app.section == Section::Chats {
                                let search = widgets::icon_button(
                                    ui,
                                    &palette,
                                    Icon::Search,
                                    &format!("Search and jump ({})", keys::shortcut("⌘K")),
                                );
                                widgets::on_click(&search, &mut app.actions, Action::OpenSwitcher);
                                let new_chat = widgets::icon_button(
                                    ui,
                                    &palette,
                                    Icon::Compose,
                                    &format!("New chat ({})", keys::shortcut("⌘N")),
                                );
                                widgets::on_click(&new_chat, &mut app.actions, Action::OpenNewChat);
                            }
                            let busy = !app.loading.is_empty();
                            if busy {
                                widgets::loading(ui, palette.dim, 14.0);
                            }
                        });
                    });
                },
            );
            // The header above took its height through `scope_builder`.
            match app.section {
                Section::Chats => chat_list::show(app, ui),
                Section::Teams => teams::show(app, ui),
                Section::Calendar => calendar::list(app, ui),
                Section::Calls => calls::list(app, ui),
            }
        });
    let width = response.response.rect.width();
    // The narrow width is temporary; keep the width the person chose.
    if !narrow
        && (width - app.settings.sidebar_width).abs() > 1.0
        && !ui.ctx().input(|i| i.pointer.any_down())
    {
        app.settings.sidebar_width = width;
        app.mark_settings_dirty();
    }
    // The line between the sidebar and the content.
    let rect = response.response.rect;
    ui.painter().vline(
        rect.right(),
        rect.y_range(),
        Stroke::new(1.0, palette.outline),
    );
}

fn main(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    egui::CentralPanel::no_frame()
        .frame(Frame::new().fill(palette.window))
        .show(ui, |ui| match app.section {
            Section::Chats | Section::Teams => match app.open.clone() {
                Some(open) => conversation::show(app, ui, open),
                None => {
                    let header = egui::Rect::from_min_size(
                        ui.max_rect().min,
                        vec2(ui.max_rect().width(), theme::HEADER_HEIGHT),
                    );
                    widgets::window_drag(ui, header, "main-drag");
                    empty_main(app, ui);
                }
            },
            Section::Calendar => calendar::detail(app, ui),
            Section::Calls => calls::show(app, ui),
        });
}

/// The main pane when nothing is open: what to do next, or why there is
/// nothing yet.
fn empty_main(app: &mut App, ui: &mut egui::Ui) {
    use crate::sync::Resource;
    let palette = app.palette;
    let chats = app.section == Section::Chats;
    let (items_empty, resource) = if chats {
        (app.chats.is_empty(), Resource::Chats)
    } else {
        (app.teams.is_empty(), Resource::Teams)
    };
    let loading = items_empty && app.loading.contains(&resource);
    let jump = format!(
        "Pick a chat on the left, or press {} to jump to anyone.",
        keys::shortcut("⌘K")
    );
    let (icon, title, text, action) = match (chats, items_empty, loading) {
        (true, _, true) => (
            Icon::Loader,
            "Loading your chats…",
            "This takes a moment the first time.",
            None,
        ),
        (true, true, false) => (
            Icon::Compose,
            "Start your first chat",
            "Message anyone in your organisation.",
            Some(("New chat", Action::OpenNewChat)),
        ),
        (true, false, false) => (Icon::Chat, "No chat open", jump.as_str(), None),
        (false, _, true) => (Icon::Loader, "Loading your teams…", "", None),
        (false, true, false) => (
            Icon::Teams,
            "No teams yet",
            "Teams you join in Microsoft Teams show up here.",
            None,
        ),
        (false, false, false) => (
            Icon::Hash,
            "No channel open",
            "Pick a channel on the left.",
            None,
        ),
    };
    let clicked = widgets::empty_state_with_action(
        ui,
        &palette,
        icon,
        title,
        text,
        action.as_ref().map(|(label, _)| *label),
    );
    if clicked && let Some((_, action)) = action {
        app.actions.push(action);
    }
}

fn toasts(app: &mut App, ctx: &egui::Context) {
    if app.toasts.is_empty() {
        return;
    }
    let palette = app.palette;
    let now = ctx.input(|i| i.time);
    let shown_id = egui::Id::new("wren-toast-times");
    let mut times: std::collections::HashMap<u64, f64> =
        ctx.data(|d| d.get_temp(shown_id)).unwrap_or_default();
    for toast in &app.toasts {
        times.entry(toast.id).or_insert(now);
    }
    // Information fades after a few seconds; errors stay until dismissed.
    let mut expired = Vec::new();
    for toast in &app.toasts {
        if toast.kind == ToastKind::Info {
            let age = now - times[&toast.id];
            if age > 3.5 {
                expired.push(toast.id);
            } else {
                ctx.request_repaint_after(std::time::Duration::from_secs_f64(3.6 - age));
            }
        }
    }
    for id in expired {
        app.actions.push(Action::DismissToast(id));
    }
    times.retain(|id, _| app.toasts.iter().any(|t| t.id == *id));
    ctx.data_mut(|d| d.insert_temp(shown_id, times));

    egui::Area::new(egui::Id::new("wren-toasts"))
        // At the top, clear of the newest messages and the composer, where
        // a failed message and its Retry button appear.
        .anchor(Align2::CENTER_TOP, vec2(0.0, theme::HEADER_HEIGHT + 12.0))
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            for toast in app.toasts.clone() {
                Frame::new()
                    .fill(palette.surface)
                    .stroke(Stroke::new(
                        1.0,
                        if toast.kind == ToastKind::Error {
                            palette.danger
                        } else {
                            palette.outline
                        },
                    ))
                    .corner_radius(CornerRadius::same(10))
                    .shadow(egui::Shadow {
                        offset: [0, 4],
                        blur: 16,
                        spread: 0,
                        color: palette.shadow,
                    })
                    .inner_margin(Margin::symmetric(14, 9))
                    .show(ui, |ui| {
                        ui.set_max_width(520.0);
                        ui.horizontal(|ui| {
                            let (icon, tint) = match toast.kind {
                                ToastKind::Error => (Icon::Alert, palette.danger),
                                ToastKind::Info => (Icon::Info, palette.secondary),
                            };
                            theme::icon(ui, icon, 16.0, tint);
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&toast.text).color(palette.text),
                                )
                                .wrap(),
                            );
                            if toast.kind == ToastKind::Error {
                                let close = widgets::icon_button_sized(
                                    ui,
                                    &palette,
                                    Icon::Close,
                                    "Dismiss",
                                    22.0,
                                    14.0,
                                    palette.dim,
                                );
                                widgets::on_click(
                                    &close,
                                    &mut app.actions,
                                    Action::DismissToast(toast.id),
                                );
                            }
                        });
                    });
                ui.add_space(6.0);
            }
        });
}

fn perf(app: &App, ctx: &egui::Context) {
    let palette = app.palette;
    let max = app.perf.cpu.iter().copied().fold(0.0f32, f32::max) * 1000.0;
    let messages: usize = app.conversations.values().map(|c| c.messages.len()).sum();
    let first = app
        .perf
        .first_frame
        .map_or_else(|| "–".into(), |d| format!("{} ms", d.as_millis()));
    let text = format!(
        "frame {:.2} ms avg · {:.2} ms max · {} frames · first frame {} · {} chats · {} messages loaded",
        app.perf.average_ms(),
        max,
        app.perf.frames,
        first,
        app.chats.len(),
        messages,
    );
    egui::Area::new(egui::Id::new("wren-perf"))
        .anchor(Align2::RIGHT_BOTTOM, vec2(-12.0, -12.0))
        .order(egui::Order::Tooltip)
        .interactable(false)
        .show(ctx, |ui| {
            Frame::new()
                .fill(palette.surface.gamma_multiply(0.92))
                .stroke(Stroke::new(1.0, palette.outline))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(text)
                            .font(theme::mono(11.0))
                            .color(palette.secondary),
                    );
                });
        });
}

/// Per-window setup: image loaders, icons, fonts, colour emoji.
pub fn install(ctx: &egui::Context, reproducible: bool) {
    egui_extras::install_image_loaders(ctx);
    crate::images::install(ctx);
    fastframe_icons::install::<theme::Icon>(ctx);
    ctx.include_bytes(
        signin::LOGO,
        include_bytes!("../../packaging/icons/wren-wordmark-light.svg").as_slice(),
    );
    theme::install_fonts(ctx, reproducible);
    fastframe_emoji::EmojiSetup::default()
        .synchronous(reproducible)
        .install();
    ctx.add_plugin(fastframe_emoji::EmojiPlugin::default());
    if !reproducible {
        let _ = std::thread::Builder::new()
            .name("emoji-font".into())
            .spawn(fastframe_emoji::warm_up);
    }
    ctx.options_mut(|options| options.zoom_with_keyboard = false);
}
