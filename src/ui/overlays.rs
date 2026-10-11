//! Dialogs over the window: the quick switcher, settings, shortcuts.

use egui::{CornerRadius, Frame, Key, Margin, Modifiers, Sense, Stroke, vec2};

use super::widgets;
use crate::app::{App, Overlay};
use crate::backend::AuthStatus;
use crate::model::{Access, Action, ConversationId, Member, SearchHit};
use crate::settings::ThemeChoice;
use crate::theme::{self, Icon};

pub fn show(app: &mut App, ctx: &egui::Context) {
    match app.overlay.clone() {
        Some(Overlay::Switcher) => switcher(app, ctx),
        Some(Overlay::Settings) => settings(app, ctx),
        Some(Overlay::Shortcuts) => shortcuts(app, ctx),
        Some(Overlay::NewChat) => new_chat(app, ctx),
        Some(Overlay::StatusMessage) => status_message(app, ctx),
        Some(Overlay::Consent(scope)) => consent(app, ctx, scope),
        None => {}
    }
}

fn modal(
    palette: theme::Palette,
    ctx: &egui::Context,
    id: &str,
    width: f32,
    top: f32,
    content: impl FnOnce(&mut egui::Ui),
) -> bool {
    let response = egui::Modal::new(egui::Id::new(id))
        .backdrop_color(palette.overlay)
        .frame(
            Frame::new()
                .fill(palette.surface)
                .stroke(Stroke::new(1.0, palette.outline))
                .corner_radius(CornerRadius::same(14))
                .inner_margin(Margin::same(0))
                .shadow(egui::Shadow {
                    offset: [0, 12],
                    blur: 40,
                    spread: 0,
                    color: palette.shadow,
                }),
        )
        .area(
            egui::Modal::default_area(egui::Id::new(id).with("area"))
                .anchor(egui::Align2::CENTER_TOP, vec2(0.0, top)),
        )
        .show(ctx, |ui| {
            ui.set_width(width);
            content(ui);
        });
    response.should_close()
}

/// A person, chat or channel the switcher can jump to.
struct Target {
    label: String,
    detail: String,
    conversation: ConversationId,
    channel: bool,
    score: i32,
}

fn score(text: &str, query: &str) -> Option<i32> {
    let text = text.to_lowercase();
    if query.is_empty() {
        return Some(0);
    }
    if text.starts_with(query) {
        return Some(300);
    }
    if text.split_whitespace().any(|word| word.starts_with(query)) {
        return Some(200);
    }
    if text.contains(query) {
        return Some(100);
    }
    // Letters in order (fuzzy).
    let mut chars = text.chars();
    query.chars().all(|q| chars.any(|c| c == q)).then_some(10)
}

fn targets(app: &App, query: &str) -> Vec<Target> {
    let query = query.trim().to_lowercase();
    let me = app.me();
    let mut targets = Vec::new();
    for (rank, chat) in app.chats.iter().enumerate() {
        let title = chat.title(me);
        let member_names = chat
            .members
            .iter()
            .map(|m| m.display_name.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let best = score(&title, &query).or_else(|| score(&member_names, &query).map(|s| s / 2));
        if let Some(score) = best {
            targets.push(Target {
                detail: chat
                    .preview
                    .as_ref()
                    .map(|p| p.text.clone())
                    .unwrap_or_default(),
                label: title,
                conversation: ConversationId::Chat(chat.id.clone()),
                channel: false,
                score: score - rank.min(50) as i32,
            });
        }
    }
    for team in &app.teams {
        for channel in app.channels.get(&team.id).into_iter().flatten() {
            let label = format!("{} › {}", team.name, channel.name);
            if let Some(score) = score(&label, &query).or_else(|| score(&channel.name, &query)) {
                targets.push(Target {
                    label,
                    detail: channel.description.clone().unwrap_or_default(),
                    conversation: ConversationId::Channel {
                        team: team.id.clone(),
                        channel: channel.id.clone(),
                    },
                    channel: true,
                    score: score - 60,
                });
            }
        }
    }
    targets.sort_by_key(|t| std::cmp::Reverse(t.score));
    targets.truncate(if query.is_empty() { 8 } else { 12 });
    targets
}

fn switcher(app: &mut App, ctx: &egui::Context) {
    let palette = app.palette;
    let mut actions = Vec::new();
    let mut query = app.switcher.query.clone();
    let found = targets(app, &query);
    let mut hits: Vec<SearchHit> = app.switcher.hits.clone();
    for hit in &app.switcher.remote_hits {
        if !hits.iter().any(|h| h.message_id == hit.message_id) {
            hits.push(hit.clone());
        }
    }
    let total = found.len() + hits.len();
    let mut selected = app.switcher.selected.min(total.saturating_sub(1));
    let mut chosen = false;
    ctx.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
            selected = (selected + 1).min(total.saturating_sub(1));
        }
        if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
            selected = selected.saturating_sub(1);
        }
        if input.consume_key(Modifiers::NONE, Key::Enter) {
            chosen = true;
        }
    });
    let close = modal(palette, ctx, "switcher", 600.0, 90.0, |ui| {
        Frame::new()
            .inner_margin(Margin::symmetric(14, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    theme::icon(ui, Icon::Search, 18.0, palette.dim);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut query)
                            .hint_text("Jump to a chat, channel or person, or search messages")
                            .frame(egui::Frame::NONE)
                            .desired_width(f32::INFINITY)
                            .font(theme::regular(15.0)),
                    );
                    response.request_focus();
                });
            });
        ui.painter().hline(
            ui.max_rect().x_range(),
            ui.cursor().top(),
            Stroke::new(1.0, palette.outline),
        );
        egui::ScrollArea::vertical()
            .max_height(440.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                Frame::new()
                    .inner_margin(Margin::symmetric(8, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if !found.is_empty() {
                            group(
                                ui,
                                &palette,
                                if query.trim().is_empty() {
                                    "Recent"
                                } else {
                                    "Conversations"
                                },
                            );
                        }
                        for (index, target) in found.iter().enumerate() {
                            let response = result_row(
                                ui,
                                app,
                                &target.label,
                                &target.detail,
                                if target.channel {
                                    Icon::Hash
                                } else {
                                    Icon::Chat
                                },
                                index == selected,
                            );
                            if response.clicked() || (chosen && index == selected) {
                                actions.push(Action::OpenConversation(target.conversation.clone()));
                                actions.push(Action::CloseOverlay);
                            }
                            if response.hovered()
                                && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO)
                            {
                                selected = index;
                            }
                        }
                        if query.trim().chars().count() >= 2 {
                            group(ui, &palette, "Messages");
                            if hits.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    let text = if app.switcher.searched == query {
                                        "No messages found."
                                    } else {
                                        "Searching…"
                                    };
                                    ui.label(egui::RichText::new(text).color(palette.dim));
                                });
                            }
                            for (offset, hit) in hits.iter().enumerate() {
                                let index = found.len() + offset;
                                let title = app.title_of(&hit.conversation);
                                let detail = format!(
                                    "{} · {} · {}",
                                    hit.author,
                                    crate::time::short_label(hit.at, crate::time::now()),
                                    title
                                );
                                let response = result_row(
                                    ui,
                                    app,
                                    &hit.snippet,
                                    &detail,
                                    Icon::Search,
                                    index == selected,
                                );
                                if response.clicked() || (chosen && index == selected) {
                                    actions.push(Action::OpenMessage {
                                        conversation: hit.conversation.clone(),
                                        message: hit.message_id.clone(),
                                    });
                                }
                                if response.hovered()
                                    && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO)
                                {
                                    selected = index;
                                }
                            }
                        }
                    });
            });
        Frame::new().inner_margin(Margin::symmetric(14, 8)).show(ui, |ui| {
            ui.label(
                egui::RichText::new("↑↓ to move · Enter to open · Esc to close · message search covers this computer's cache, then Microsoft Search")
                    .font(theme::regular(11.0))
                    .color(palette.dim),
            );
        });
    });
    if query != app.switcher.query {
        app.switcher.query = query.clone();
        selected = 0;
        if query.trim().chars().count() >= 2 {
            actions.push(Action::SearchMessages(query));
        } else {
            app.switcher.hits.clear();
            app.switcher.remote_hits.clear();
        }
    }
    app.switcher.selected = selected;
    if close {
        actions.push(Action::CloseOverlay);
    }
    app.actions.extend(actions);
}

fn group(ui: &mut egui::Ui, palette: &theme::Palette, label: &str) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        widgets::section_label(ui, palette, label);
    });
    ui.add_space(2.0);
}

fn result_row(
    ui: &mut egui::Ui,
    app: &App,
    label: &str,
    detail: &str,
    icon: Icon,
    selected: bool,
) -> egui::Response {
    let palette = &app.palette;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 44.0), Sense::click());
    if selected || response.hovered() {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(8),
            if selected {
                palette.surface_active
            } else {
                palette.surface_hover
            },
        );
    }
    if selected {
        ui.scroll_to_rect(rect, None);
    }
    icon.image(
        if selected {
            palette.accent
        } else {
            palette.dim
        },
        16.0,
    )
    .paint_at(
        ui,
        egui::Rect::from_center_size(
            egui::pos2(rect.left() + 20.0, rect.center().y),
            egui::Vec2::splat(16.0),
        ),
    );
    let left = rect.left() + 40.0;
    let title = super::chat_list::elided(
        ui,
        label,
        theme::medium(14.0),
        palette.text,
        rect.right() - left - 12.0,
    );
    ui.painter()
        .galley(egui::pos2(left, rect.top() + 5.0), title, palette.text);
    let detail = super::chat_list::elided(
        ui,
        detail,
        theme::regular(12.0),
        palette.secondary,
        rect.right() - left - 12.0,
    );
    ui.painter().galley(
        egui::pos2(left, rect.top() + 24.0),
        detail,
        palette.secondary,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn settings(app: &mut App, ctx: &egui::Context) {
    let palette = app.palette;
    let mut actions = Vec::new();
    let close = modal(palette, ctx, "settings", 560.0, 70.0, |ui| {
        Frame::new().inner_margin(Margin::same(22)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Settings").font(theme::bold(20.0)).color(palette.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close = widgets::icon_button(ui, &palette, Icon::Close, "Close (Esc)");
                    widgets::on_click(&close, &mut actions, Action::CloseOverlay);
                });
            });
            ui.add_space(10.0);
            let height = (ui.ctx().content_rect().height() - 220.0).clamp(240.0, 640.0);
            egui::ScrollArea::vertical()
                .max_height(height)
                .min_scrolled_height(height)
                .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                // Account.
                widgets::section_label(ui, &palette, "Account");
                match &app.auth {
                    AuthStatus::SignedIn { name, username, .. } => {
                        ui.horizontal(|ui| {
                            let me = app.me().to_owned();
                            widgets::avatar(ui, app, Some(&me), name, 40.0, true);
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.label(egui::RichText::new(name).font(theme::semibold(14.0)).color(palette.text));
                                ui.label(egui::RichText::new(username).color(palette.secondary));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if app.demo {
                                    if widgets::secondary_button(ui, &palette, "Leave demo").clicked() {
                                        actions.push(Action::LeaveDemo);
                                    }
                                } else if widgets::secondary_button(ui, &palette, "Sign out").on_hover_text("Signs out and deletes this account's local cache").clicked() {
                                    actions.push(Action::SignOut);
                                }
                            });
                        });
                    }
                    other => {
                        ui.label(egui::RichText::new(format!("Not signed in ({})", other.log_label())).color(palette.secondary));
                    }
                }
                ui.horizontal(|ui| {
                    let accounts = app.accounts.len();
                    ui.label(
                        egui::RichText::new(if accounts == 1 {
                            "1 account in Wren".to_owned()
                        } else {
                            format!("{accounts} accounts in Wren · switch from your picture or with ⌘⇧A")
                        })
                        .font(theme::regular(12.0))
                        .color(palette.dim),
                    );
                    if !app.demo && ui.link("Add account").clicked() {
                        actions.push(Action::AddAccount);
                    }
                });
                if !app.demo {
                    let channel = match app.channel_access {
                        Access::Granted => "granted",
                        Access::NeedsConsent => "needs admin consent",
                        Access::Forbidden => "refused by the tenant",
                        Access::Unknown => "unknown",
                    };
                    ui.label(
                        egui::RichText::new(format!("Channel posts: {channel}. Granted permissions: {}", if app.granted.is_empty() { "–" } else { &app.granted }))
                            .font(theme::regular(12.0))
                            .color(palette.dim),
                    );
                }
                ui.add_space(8.0);

                widgets::section_label(ui, &palette, "Appearance");
                ui.horizontal_wrapped(|ui| {
                    for (choice, label, icon) in [
                        (ThemeChoice::System, "System", Icon::Monitor),
                        (ThemeChoice::Light, "Light", Icon::Sun),
                        (ThemeChoice::Dark, "Dark", Icon::Moon),
                    ] {
                        let on = app.settings.theme == choice;
                        let response = widgets::pill(ui, &palette, icon, label, on);
                        widgets::on_click(&response, &mut actions, Action::SetTheme(choice));
                    }
                });
                let current = match &app.settings.theme {
                    ThemeChoice::Palette(name) => fastframe_theme::display_name(name).to_owned(),
                    _ => "Wren".to_owned(),
                };
                egui::ComboBox::from_label(egui::RichText::new("Palette").color(palette.secondary))
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(!matches!(app.settings.theme, ThemeChoice::Palette(_)), "Wren").clicked() {
                            actions.push(Action::SetTheme(ThemeChoice::System));
                        }
                        for preset in theme::presets() {
                            let on = app.settings.theme == ThemeChoice::Palette(preset.filename.clone());
                            if ui.selectable_label(on, fastframe_theme::display_name(&preset.filename)).clicked() {
                                actions.push(Action::SetTheme(ThemeChoice::Palette(preset.filename.clone())));
                            }
                        }
                    });
                ui.label(egui::RichText::new(format!("Zoom {:.0}% (⌘+ / ⌘− / ⌘0)", app.settings.zoom * 100.0)).color(palette.secondary));
                ui.add_space(8.0);

                widgets::section_label(ui, &palette, "Behaviour");
                let mut notifications = app.settings.notifications;
                if ui.checkbox(&mut notifications, "Desktop notifications for new messages and meeting reminders").changed() {
                    actions.push(Action::SetNotifications(notifications));
                }
                let mut keep = app.settings.keep_running;
                if ui.checkbox(&mut keep, "Keep running in the tray when the window closes").changed() {
                    actions.push(Action::SetKeepRunning(keep));
                }
                ui.label(
                    egui::RichText::new("New messages are fetched when you open a chat, come back to the window or press ⌘R. Microsoft's rules for Teams forbid background polling; for instant alerts, connect a push relay below (docs/realtime.md).")
                        .font(theme::regular(12.0))
                        .color(palette.dim),
                );
                let mut session = app.settings.presence_session;
                if ui
                    .checkbox(&mut session, "Show me as available while I use Wren")
                    .on_hover_text("Keeps a Microsoft presence session, as a Teams app does: you show Available while you use Wren, and Away a few minutes after you stop. Needs the Presence.ReadWrite permission, asked for the first time you set your status.")
                    .changed()
                {
                    actions.push(Action::SetPresenceSession(session));
                }
                let mut perf = app.settings.perf_overlay;
                if ui.checkbox(&mut perf, "Show the performance overlay (⌘⇧P)").changed() {
                    actions.push(Action::TogglePerfOverlay);
                }
                ui.add_space(8.0);

                if !app.demo {
                    widgets::section_label(ui, &palette, "Push updates (experimental)");
                    ui.label(
                        egui::RichText::new(
                            "With a wren-relay you run, Microsoft Graph tells Wren about new messages as they happen, also in the background. Only chat and message IDs pass through the relay.",
                        )
                        .font(theme::regular(12.0))
                        .color(palette.dim),
                    );
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut app.form_relay)
                                .hint_text("https://relay.example.com")
                                .desired_width(ui.available_width() - 70.0),
                        );
                        let changed = app.form_relay.trim() != app.settings.relay_url.as_deref().unwrap_or_default();
                        if ui.add_enabled(changed, egui::Button::new("Save")).clicked() {
                            actions.push(Action::SetRelay(app.form_relay.clone()));
                        }
                    });
                    let color = match app.push {
                        crate::realtime::PushStatus::Live => palette.success,
                        crate::realtime::PushStatus::Failed(_) => palette.danger,
                        _ => palette.secondary,
                    };
                    ui.label(
                        egui::RichText::new(format!("Status: {}", app.push.label()))
                            .font(theme::regular(12.0))
                            .color(color),
                    );
                    ui.add_space(8.0);
                    widgets::section_label(ui, &palette, "App registration");
                    ui.label(egui::RichText::new("Application (client) ID").color(palette.secondary));
                    ui.add(egui::TextEdit::singleline(&mut app.form_client_id).font(theme::mono(12.0)).desired_width(f32::INFINITY));
                    ui.label(egui::RichText::new("Tenant").color(palette.secondary));
                    ui.add(egui::TextEdit::singleline(&mut app.form_tenant).desired_width(f32::INFINITY));
                    let changed = app.form_client_id.trim() != app.settings.client_id || app.form_tenant.trim() != app.settings.tenant;
                    if ui.add_enabled(changed, egui::Button::new("Save and restart sign-in")).clicked() {
                        actions.push(Action::SaveAppRegistration {
                            client_id: app.form_client_id.clone(),
                            tenant: app.form_tenant.clone(),
                        });
                    }
                    if crate::settings::is_guid(&app.settings.client_id) {
                        let consent = format!(
                            "https://login.microsoftonline.com/{}/adminconsent?client_id={}",
                            if app.settings.tenant == "organizations" { "common" } else { &app.settings.tenant },
                            app.settings.client_id
                        );
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Admin consent link for your tenant's administrator:").font(theme::regular(12.0)).color(palette.dim));
                            if ui.link("Copy").clicked() {
                                actions.push(Action::CopyText(consent));
                            }
                        });
                    }
                    ui.add_space(8.0);
                }

                widgets::section_label(ui, &palette, "About");
                ui.label(
                    egui::RichText::new(format!(
                        "Wren {} · MIT licence · no telemetry. Not affiliated with Microsoft.",
                        env!("CARGO_PKG_VERSION")
                    ))
                    .color(palette.secondary),
                );
                ui.horizontal(|ui| {
                    if ui.link("Keyboard shortcuts").clicked() {
                        actions.push(Action::OpenShortcuts);
                    }
                    if ui.link("Log file folder").clicked() {
                        actions.push(Action::OpenUrl(format!("file://{}", app.dirs.state.display())));
                    }
                });
            });
        });
    });
    if close {
        actions.push(Action::CloseOverlay);
    }
    app.actions.extend(actions);
}

fn shortcuts(app: &mut App, ctx: &egui::Context) {
    let palette = app.palette;
    let close = modal(palette, ctx, "shortcuts", 480.0, 90.0, |ui| {
        Frame::new().inner_margin(Margin::same(22)).show(ui, |ui| {
            ui.label(
                egui::RichText::new("Keyboard shortcuts")
                    .font(theme::bold(20.0))
                    .color(palette.text),
            );
            ui.add_space(10.0);
            egui::Grid::new("shortcuts")
                .num_columns(2)
                .spacing(vec2(24.0, 8.0))
                .show(ui, |ui| {
                    for (keys, what) in super::keys::SHORTCUTS {
                        let keys = if cfg!(target_os = "macos") {
                            (*keys).to_owned()
                        } else {
                            keys.replace('⌘', "Ctrl ")
                        };
                        ui.label(
                            egui::RichText::new(keys)
                                .font(theme::mono(12.0))
                                .color(palette.accent),
                        );
                        ui.label(egui::RichText::new(*what).color(palette.text));
                        ui.end_row();
                    }
                });
        });
    });
    if close {
        app.actions.push(Action::CloseOverlay);
    }
}

/// Start a chat: pick people from the directory, name a group, start.
fn new_chat(app: &mut App, ctx: &egui::Context) {
    let palette = app.palette;
    let mut actions = Vec::new();
    let mut query = app.new_chat.query.clone();
    let mut topic = app.new_chat.topic.clone();
    let mut picked = app.new_chat.picked.clone();
    // People known from chats first, then the directory's answer.
    let needle = query.trim().to_lowercase();
    let mut found: Vec<Member> = app
        .directory()
        .into_iter()
        .filter(|p| {
            !needle.is_empty()
                && (p.display_name.to_lowercase().contains(&needle)
                    || p.email
                        .as_deref()
                        .is_some_and(|e| e.to_lowercase().contains(&needle)))
        })
        .collect();
    if app.people.query == query {
        for person in &app.people.results {
            if !found.iter().any(|p| p.user_id == person.user_id) {
                found.push(person.clone());
            }
        }
    }
    found.retain(|p| !picked.iter().any(|q| q.user_id == p.user_id) && p.user_id != app.me());
    found.truncate(8);
    let mut selected = app.new_chat.selected.min(found.len().saturating_sub(1));
    let mut choose: Option<Member> = None;
    let mut start = false;
    ctx.input_mut(|input| {
        if !found.is_empty() {
            if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                selected = (selected + 1) % found.len();
            }
            if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                selected = (selected + found.len() - 1) % found.len();
            }
            if input.consume_key(Modifiers::NONE, Key::Enter) {
                choose = found.get(selected).cloned();
            }
        } else if !picked.is_empty() && input.consume_key(Modifiers::COMMAND, Key::Enter) {
            start = true;
        }
    });
    let close = modal(palette, ctx, "new-chat", 520.0, 90.0, |ui| {
        Frame::new().inner_margin(Margin::same(20)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("New chat")
                        .font(theme::bold(20.0))
                        .color(palette.text),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close = widgets::icon_button(ui, &palette, Icon::Close, "Close (Esc)");
                    widgets::on_click(&close, &mut actions, Action::CloseOverlay);
                });
            });
            ui.add_space(10.0);
            // The people picked so far.
            if !picked.is_empty() {
                let mut remove = None;
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                    for (index, person) in picked.iter().enumerate() {
                        Frame::new()
                            .fill(palette.surface_hover)
                            .corner_radius(CornerRadius::same(14))
                            .inner_margin(Margin {
                                left: 4,
                                right: 2,
                                top: 2,
                                bottom: 2,
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    widgets::avatar(
                                        ui,
                                        app,
                                        Some(&person.user_id),
                                        &person.display_name,
                                        22.0,
                                        false,
                                    );
                                    ui.label(
                                        egui::RichText::new(&person.display_name)
                                            .font(theme::medium(13.0))
                                            .color(palette.text),
                                    );
                                    let close = widgets::icon_button_sized(
                                        ui,
                                        &palette,
                                        Icon::Close,
                                        "Remove",
                                        20.0,
                                        12.0,
                                        palette.dim,
                                    );
                                    if close.clicked() {
                                        remove = Some(index);
                                    }
                                });
                            });
                    }
                });
                if let Some(index) = remove {
                    picked.remove(index);
                }
                ui.add_space(8.0);
            }
            let response = ui.add(
                egui::TextEdit::singleline(&mut query)
                    .hint_text("Type a name or email address")
                    .desired_width(f32::INFINITY)
                    .margin(Margin::symmetric(10, 8))
                    .font(theme::regular(14.0)),
            );
            response.request_focus();
            ui.add_space(6.0);
            for (index, person) in found.iter().enumerate() {
                let (rect, row) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
                if index == selected || row.hovered() {
                    ui.painter()
                        .rect_filled(rect, CornerRadius::same(8), palette.surface_active);
                }
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(rect.shrink2(vec2(10.0, 0.0)))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                widgets::avatar(
                    &mut child,
                    app,
                    Some(&person.user_id),
                    &person.display_name,
                    30.0,
                    true,
                );
                child.add_space(8.0);
                child.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(
                        egui::RichText::new(&person.display_name)
                            .font(theme::medium(14.0))
                            .color(palette.text),
                    );
                    if let Some(email) = &person.email {
                        ui.label(
                            egui::RichText::new(email)
                                .font(theme::regular(12.0))
                                .color(palette.dim),
                        );
                    }
                });
                if row.clicked() {
                    choose = Some(person.clone());
                }
            }
            if found.is_empty() && query.trim().chars().count() >= 2 {
                ui.label(egui::RichText::new("Searching your organisation…").color(palette.dim));
            }
            if picked.len() > 1 {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Group name (optional)")
                        .font(theme::medium(12.0))
                        .color(palette.secondary),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut topic)
                        .hint_text("Launch crew")
                        .desired_width(f32::INFINITY)
                        .margin(Margin::symmetric(10, 6)),
                );
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let label = match picked.len() {
                    0 => "Pick someone to chat with".to_owned(),
                    1 => format!("Chat with {}", picked[0].display_name),
                    n => format!("Start a group chat with {n} people"),
                };
                let button = ui.add_enabled(
                    !picked.is_empty(),
                    egui::Button::new(
                        egui::RichText::new(label)
                            .font(theme::semibold(13.0))
                            .color(palette.on_accent),
                    )
                    .fill(palette.accent)
                    .corner_radius(CornerRadius::same(8))
                    .min_size(vec2(0.0, 32.0)),
                );
                if button.clicked() {
                    start = true;
                }
                if picked.len() == 1 {
                    ui.label(
                        egui::RichText::new("An existing chat with this person opens instead.")
                            .font(theme::regular(11.0))
                            .color(palette.dim),
                    );
                }
            });
        });
    });
    if let Some(person) = choose {
        picked.push(person);
        query.clear();
        selected = 0;
    }
    if query != app.new_chat.query {
        selected = 0;
        if query.trim().chars().count() >= 2 {
            actions.push(Action::SearchPeople(query.clone()));
        }
    }
    if start && !picked.is_empty() {
        actions.push(Action::CreateChat {
            members: picked.clone(),
            topic: (!topic.trim().is_empty()).then(|| topic.clone()),
        });
    }
    app.new_chat.query = query;
    app.new_chat.topic = topic;
    app.new_chat.picked = picked;
    app.new_chat.selected = selected;
    if close {
        actions.push(Action::CloseOverlay);
    }
    app.actions.extend(actions);
}

fn status_message(app: &mut App, ctx: &egui::Context) {
    let palette = app.palette;
    let mut actions = Vec::new();
    let mut text = app.form_status.clone();
    let close = modal(palette, ctx, "status-message", 440.0, 120.0, |ui| {
        Frame::new().inner_margin(Margin::same(20)).show(ui, |ui| {
            ui.label(
                egui::RichText::new("Status message")
                    .font(theme::bold(18.0))
                    .color(palette.text),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("People see it next to your name in Teams and Wren.")
                    .color(palette.secondary),
            );
            ui.add_space(10.0);
            let response = ui.add(
                egui::TextEdit::multiline(&mut text)
                    .hint_text("Back at 3 pm")
                    .desired_rows(2)
                    .desired_width(f32::INFINITY)
                    .margin(Margin::symmetric(10, 8)),
            );
            response.request_focus();
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, &palette, "  Save  ").clicked() {
                    actions.push(Action::SetStatusMessage(text.clone()));
                }
                if widgets::secondary_button(ui, &palette, "Clear").clicked() {
                    actions.push(Action::SetStatusMessage(String::new()));
                }
                if widgets::secondary_button(ui, &palette, "Cancel").clicked() {
                    actions.push(Action::CloseOverlay);
                }
            });
        });
    });
    app.form_status = text;
    if close {
        actions.push(Action::CloseOverlay);
    }
    app.actions.extend(actions);
}

/// Explains a permission before Wren asks Microsoft for it.
fn consent(app: &mut App, ctx: &egui::Context, scope: &'static str) {
    let palette = app.palette;
    let mut actions = Vec::new();
    let close = modal(palette, ctx, "consent", 460.0, 120.0, |ui| {
        Frame::new().inner_margin(Margin::same(22)).show(ui, |ui| {
            ui.horizontal(|ui| {
                theme::icon(ui, Icon::Lock, 20.0, palette.accent);
                ui.label(egui::RichText::new("One more permission").font(theme::bold(18.0)).color(palette.text));
            });
            ui.add_space(8.0);
            ui.label(egui::RichText::new(crate::auth::scope_reason(scope)).color(palette.text));
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(format!(
                    "Wren asks for Microsoft Graph's {scope} permission only now that you use this. Microsoft's consent page opens in your browser; some organisations need an administrator to approve it."
                ))
                .font(theme::regular(12.0))
                .color(palette.secondary),
            );
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, &palette, "  Continue in browser  ").clicked() {
                    actions.push(Action::RequestScope(scope));
                }
                if widgets::secondary_button(ui, &palette, "Not now").clicked() {
                    actions.push(Action::CloseOverlay);
                }
            });
        });
    });
    if close {
        actions.push(Action::CloseOverlay);
    }
    app.actions.extend(actions);
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn prefixes_beat_substrings_beat_fuzzy() {
        assert!(score("ada lovelace", "ada") > score("ada lovelace", "lace"));
        assert!(score("ada lovelace", "love") > score("ada lovelace", "elac"));
        assert!(score("ada lovelace", "alc").is_some());
        assert!(score("ada lovelace", "xyz").is_none());
    }
}
