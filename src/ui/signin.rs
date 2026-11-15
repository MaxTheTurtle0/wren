//! The welcome screen: connect an app registration, sign in, or explore
//! the demo. Usable with no Microsoft setup at all.

use egui::{CornerRadius, Frame, Margin, Stroke, vec2};

use super::widgets;
use crate::app::App;
use crate::backend::AuthStatus;
use crate::model::Action;
use crate::theme;

pub const LOGO: &str = "bytes://wren-logo.svg";
pub const DOCS: &str =
    "https://github.com/MaxTheTurtle0/wren/blob/main/docs/azure-app-registration.md";

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    egui::CentralPanel::no_frame()
        .frame(Frame::new().fill(palette.panel))
        .show(ui, |ui| {
            let header = egui::Rect::from_min_size(ui.max_rect().min, vec2(ui.max_rect().width(), theme::HEADER_HEIGHT));
            widgets::window_drag(ui, header, "signin-drag");
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    // The logo, card and footer sit as one group near the middle.
                    let group = if matches!(app.auth, AuthStatus::NotConfigured) {
                        780.0
                    } else {
                        560.0
                    };
                    ui.add_space(((ui.available_height() - group) / 2.0).max(16.0));
                    ui.add(egui::Image::new(LOGO).fit_to_exact_size(vec2(160.0, 121.0)).tint(palette.text).alt_text("Wren"));
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("A fast, native client for Microsoft Teams chats, channels and meetings.").line_height(Some(theme::leading(14.0)))
                            .font(theme::regular(14.0))
                            .color(palette.secondary),
                    );
                    ui.add_space(22.0);
                    Frame::new()
                        .fill(palette.surface)
                        .stroke(Stroke::new(1.0, palette.outline))
                        .corner_radius(CornerRadius::same(14))
                        .inner_margin(Margin::same(24))
                        .shadow(egui::Shadow {
                            offset: [0, 6],
                            blur: 24,
                            spread: 0,
                            color: palette.shadow,
                        })
                        .show(ui, |ui| {
                            ui.set_width(420.0);
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                card(app, ui);
                            });
                        });
                    ui.add_space(18.0);
                    let in_progress = matches!(
                        app.auth,
                        AuthStatus::Restoring
                            | AuthStatus::Finishing
                            | AuthStatus::WaitingForBrowser { .. }
                            | AuthStatus::DeviceCode { .. }
                    );
                    if in_progress {
                        // One task at a time: the demo returns once sign-in
                        // finishes or is cancelled.
                    } else if let Some(previous) = app.adding_from().map(|account| account.label()) {
                        // Adding a second account: the way back.
                        if widgets::secondary_button(ui, &palette, &format!("Back to {previous}")).clicked() {
                            app.actions.push(Action::CancelAddAccount);
                        }
                    } else if widgets::secondary_button(ui, &palette, "Explore the demo")
                        .on_hover_text("Sample chats, channels and meetings. Nothing leaves this computer.")
                        .clicked()
                    {
                        app.actions.push(Action::StartDemo);
                    }
                    ui.add_space(24.0);
                    ui.set_max_width(472.0f32.min(ui.available_width() - 32.0));
                    ui.label(
                        egui::RichText::new("Wren is an independent open-source project, not affiliated with Microsoft. It talks only to Microsoft's official APIs and keeps your data on this computer.").line_height(Some(theme::leading(12.0)))
                            .font(theme::regular(12.0))
                            .color(palette.dim),
                    );
                    ui.add_space(24.0);
                });
            });
        });
}

fn card(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    ui.spacing_mut().item_spacing.y = 8.0;
    match app.auth.clone() {
        AuthStatus::NotConfigured => registration(app, ui, true),
        AuthStatus::Restoring | AuthStatus::Finishing => {
            // The same title and body styles as the other sign-in cards.
            ui.horizontal(|ui| {
                widgets::loading(ui, palette.accent, 18.0);
                ui.label(
                    egui::RichText::new("Signing in…")
                        .font(theme::semibold(18.0))
                        .color(palette.text),
                );
            });
            ui.label(
                egui::RichText::new(
                    "Wren is finishing the sign-in with Microsoft. This takes a few seconds.",
                )
                .line_height(Some(theme::leading(14.0)))
                .color(palette.secondary),
            );
        }
        AuthStatus::WaitingForBrowser { url } => {
            ui.label(
                egui::RichText::new("Continue in your browser")
                    .font(theme::semibold(18.0))
                    .color(palette.text),
            );
            ui.label(
                egui::RichText::new("Sign in with your work or school account in the browser window Wren opened. You'll come back here automatically.").line_height(Some(theme::leading(14.0)))
                    .color(palette.secondary),
            );
            ui.horizontal(|ui| {
                widgets::loading(ui, palette.accent, 16.0);
                ui.label(
                    egui::RichText::new("Waiting for Microsoft…")
                        .font(theme::regular(13.0))
                        .color(palette.secondary),
                );
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, &palette, "Open the page again").clicked() {
                    app.actions.push(Action::OpenUrl(url.clone()));
                }
                if widgets::secondary_button(ui, &palette, "Copy link").clicked() {
                    app.actions.push(Action::CopyText(url.clone()));
                }
                if widgets::secondary_button(ui, &palette, "Cancel").clicked() {
                    app.actions.push(Action::CancelSignIn);
                }
            });
        }
        AuthStatus::DeviceCode { code, url } => {
            ui.label(
                egui::RichText::new("Sign in with a code")
                    .font(theme::semibold(18.0))
                    .color(palette.text),
            );
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(
                    egui::RichText::new("On any device, go to")
                        .line_height(Some(theme::leading(14.0)))
                        .color(palette.secondary),
                );
                if ui
                    .link(egui::RichText::new(&url).line_height(Some(theme::leading(14.0))))
                    .clicked()
                {
                    app.actions.push(Action::OpenUrl(url.clone()));
                }
                ui.label(
                    egui::RichText::new("and enter this code:")
                        .line_height(Some(theme::leading(14.0)))
                        .color(palette.secondary),
                );
            });
            ui.add_space(4.0);
            ui.vertical_centered(|ui| {
                Frame::new()
                    .fill(palette.panel)
                    .stroke(Stroke::new(1.0, palette.outline))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(20, 8))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&code)
                                .font(theme::semibold(30.0))
                                .color(palette.text)
                                .extra_letter_spacing(3.0),
                        );
                    });
            });
            ui.horizontal(|ui| {
                widgets::loading(ui, palette.accent, 16.0);
                ui.label(
                    egui::RichText::new("Waiting for you to enter the code…")
                        .font(theme::regular(13.0))
                        .color(palette.secondary),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, &palette, "Open the page").clicked() {
                    app.actions.push(Action::OpenUrl(url.clone()));
                }
                if widgets::secondary_button(ui, &palette, "Copy code").clicked() {
                    app.actions.push(Action::CopyText(code.clone()));
                }
                if widgets::secondary_button(ui, &palette, "Cancel").clicked() {
                    app.actions.push(Action::CancelSignIn);
                }
            });
        }
        status => {
            let (title, text) = match &status {
                AuthStatus::Expired { reason, .. } => ("Sign in again".to_owned(), reason.clone()),
                AuthStatus::Failed(reason) => ("Sign-in did not finish".to_owned(), reason.clone()),
                _ => (
                    "Sign in".to_owned(),
                    "Use your work or school Microsoft 365 account. Microsoft's sign-in page opens in your browser; Wren never sees your password.".to_owned(),
                ),
            };
            ui.label(
                egui::RichText::new(title)
                    .font(theme::semibold(18.0))
                    .color(palette.text),
            );
            if let AuthStatus::Failed(_) = status {
                ui.horizontal_top(|ui| {
                    theme::icon(ui, crate::theme::Icon::Alert, 16.0, palette.danger);
                    ui.add(
                        egui::Label::new(egui::RichText::new(&text).color(palette.danger)).wrap(),
                    );
                });
            } else {
                ui.label(
                    egui::RichText::new(&text)
                        .line_height(Some(theme::leading(14.0)))
                        .color(palette.secondary),
                );
            }
            if let AuthStatus::Expired { username, .. } = &status
                && !username.is_empty()
            {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Account")
                            .font(theme::medium(13.0))
                            .color(palette.dim),
                    );
                    ui.label(
                        egui::RichText::new(username)
                            .font(theme::medium(13.0))
                            .color(palette.text),
                    );
                });
            }
            ui.add_space(6.0);
            let label = if matches!(status, AuthStatus::Failed(_)) {
                "Try again"
            } else {
                "Sign in with Microsoft"
            };
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, &palette, label).clicked() {
                    app.actions.push(Action::SignIn);
                }
                if widgets::secondary_button(ui, &palette, "Use a code instead")
                    .on_hover_text("Sign in on any device with a short code, if the browser can't return to Wren.")
                    .clicked()
                {
                    app.actions.push(Action::SignInWithCode);
                }
            });
            ui.add_space(8.0);
            ui.separator();
            egui::CollapsingHeader::new(
                egui::RichText::new("App registration")
                    .font(theme::medium(13.0))
                    .color(palette.secondary),
            )
            .id_salt("registration")
            .icon(widgets::disclosure_chevron(palette.secondary))
            .default_open(false)
            .show(ui, |ui| registration(app, ui, false));
        }
    }
}

/// The application (client) ID and tenant Wren signs in with.
fn registration(app: &mut App, ui: &mut egui::Ui, first_run: bool) {
    let palette = app.palette;
    if first_run {
        ui.label(
            egui::RichText::new("Connect Wren to Microsoft 365")
                .font(theme::semibold(18.0))
                .color(palette.text),
        );
        ui.label(
            egui::RichText::new("Wren signs in through an Entra ID app registration that you or your administrator create. It takes about five minutes and needs no secrets.").line_height(Some(theme::leading(14.0)))
                .color(palette.secondary),
        );
        ui.add_space(2.0);
        // The guide first: the fields below need what it produces.
        if widgets::secondary_button(ui, &palette, "Open the registration guide")
            .on_hover_text("Step by step, in your browser")
            .clicked()
        {
            app.actions.push(Action::OpenUrl(DOCS.into()));
        }
        ui.add_space(8.0);
    }
    ui.label(
        egui::RichText::new("Application (client) ID")
            .font(theme::medium(12.0))
            .color(palette.secondary),
    );
    ui.add(widgets::text_field(
        &mut app.form_client_id,
        "Paste the application ID",
        theme::regular(13.0),
    ));
    // Field groups stand apart; a label stays close to its field.
    ui.add_space(10.0);
    ui.label(
        egui::RichText::new("Tenant")
            .font(theme::medium(12.0))
            .color(palette.secondary),
    );
    ui.label(
        egui::RichText::new(
            "Your tenant ID or domain. Use 'organizations' only for a multi-tenant registration.",
        )
        .font(theme::regular(12.0))
        .color(palette.dim),
    );
    ui.add(widgets::text_field(
        &mut app.form_tenant,
        "Your tenant ID or domain, e.g. contoso.onmicrosoft.com",
        theme::regular(13.0),
    ));
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let valid = crate::settings::is_guid(app.form_client_id.trim());
        let save = ui
            .add_enabled_ui(valid, |ui| {
                widgets::primary_button(
                    ui,
                    &palette,
                    if first_run {
                        "Save and continue"
                    } else {
                        "Save"
                    },
                )
            })
            .inner;
        if save.clicked() {
            app.actions.push(Action::SaveAppRegistration {
                client_id: app.form_client_id.clone(),
                tenant: app.form_tenant.clone(),
            });
        }
        if !first_run && ui.link("How to register Wren").clicked() {
            app.actions.push(Action::OpenUrl(DOCS.into()));
        }
    });
    if !crate::settings::is_guid(app.form_client_id.trim()) {
        ui.label(
            egui::RichText::new("Save needs the Application (client) ID from your registration.")
                .font(theme::regular(12.0))
                .color(palette.dim),
        );
    }
    if first_run {
        egui::CollapsingHeader::new(
            egui::RichText::new("Advanced")
                .font(theme::medium(13.0))
                .color(palette.secondary),
        )
        .id_salt("registration-advanced")
        .icon(widgets::disclosure_chevron(palette.secondary))
        .default_open(false)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(
                    "Managed installs can set WREN_CLIENT_ID and WREN_TENANT instead.",
                )
                .font(theme::regular(12.0))
                .color(palette.secondary),
            );
        });
    } else {
        ui.label(
            egui::RichText::new("Or start Wren with WREN_CLIENT_ID and WREN_TENANT set.")
                .font(theme::regular(12.0))
                .color(palette.dim),
        );
    }
}
