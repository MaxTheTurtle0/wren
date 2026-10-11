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
                    ui.add_space((ui.available_height() * 0.10).max(48.0));
                    ui.add(egui::Image::new(LOGO).fit_to_exact_size(vec2(84.0, 84.0)));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Wren").font(theme::bold(30.0)).color(palette.text));
                    ui.label(
                        egui::RichText::new("A fast, native client for Microsoft Teams chats, channels and meetings.")
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
                    if let Some(previous) = app.adding_from().map(|account| account.label()) {
                        // Adding a second account: the way back.
                        if widgets::secondary_button(ui, &palette, &format!("  Back to {previous}  ")).clicked() {
                            app.actions.push(Action::CancelAddAccount);
                        }
                    } else if widgets::secondary_button(ui, &palette, "  Explore the demo  ")
                        .on_hover_text("Sample chats, channels and meetings. Nothing leaves this computer.")
                        .clicked()
                    {
                        app.actions.push(Action::StartDemo);
                    }
                    ui.add_space(24.0);
                    ui.label(
                        egui::RichText::new("Wren is an independent open-source project, not affiliated with Microsoft. It talks only to Microsoft's official APIs and keeps your data on this computer.")
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
            ui.vertical_centered(|ui| {
                ui.add(egui::Spinner::new().size(22.0).color(palette.accent));
                ui.label(egui::RichText::new("Signing in…").color(palette.secondary));
            });
        }
        AuthStatus::WaitingForBrowser { url } => {
            ui.label(
                egui::RichText::new("Continue in your browser")
                    .font(theme::semibold(18.0))
                    .color(palette.text),
            );
            ui.label(
                egui::RichText::new("Sign in with your work or school account in the browser window Wren opened. You'll come back here automatically.")
                    .color(palette.secondary),
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if widgets::secondary_button(ui, &palette, "Open the page again").clicked() {
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
            ui.label(
                egui::RichText::new(format!("On any device, open {url} and enter:"))
                    .color(palette.secondary),
            );
            ui.add_space(4.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(&code)
                        .font(theme::mono(28.0))
                        .color(palette.accent),
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
                AuthStatus::Expired { username, reason } => (
                    "Sign in again".to_owned(),
                    if username.is_empty() { reason.clone() } else { format!("{reason} ({username})") },
                ),
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
            ui.label(
                egui::RichText::new(text).color(if matches!(status, AuthStatus::Failed(_)) {
                    palette.danger
                } else {
                    palette.secondary
                }),
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, &palette, "  Sign in with Microsoft  ").clicked() {
                    app.actions.push(Action::SignIn);
                }
                if widgets::secondary_button(ui, &palette, "Use a code instead")
                    .on_hover_text("Device code sign-in, for when the browser can't reach Wren. Needs 'Allow public client flows' on the app registration.")
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
            egui::RichText::new("Wren signs in through an Entra ID app registration that you or your administrator create. It takes about five minutes and needs no secrets.")
                .color(palette.secondary),
        );
        ui.add_space(4.0);
    }
    ui.label(
        egui::RichText::new("Application (client) ID")
            .font(theme::medium(12.0))
            .color(palette.secondary),
    );
    ui.add(
        egui::TextEdit::singleline(&mut app.form_client_id)
            .hint_text("00000000-0000-0000-0000-000000000000")
            .desired_width(f32::INFINITY)
            .font(theme::mono(13.0))
            .margin(Margin::symmetric(8, 6)),
    );
    ui.label(
        egui::RichText::new("Tenant")
            .font(theme::medium(12.0))
            .color(palette.secondary),
    );
    ui.label(
        egui::RichText::new(
            "Your tenant ID or domain. Use 'organizations' only for a multi-tenant registration.",
        )
        .font(theme::regular(11.0))
        .color(palette.dim),
    );
    ui.add(
        egui::TextEdit::singleline(&mut app.form_tenant)
            .hint_text("Your tenant ID or domain, e.g. contoso.onmicrosoft.com")
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(8, 6)),
    );
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let valid = crate::settings::is_guid(app.form_client_id.trim());
        let save = ui.add_enabled(
            valid,
            egui::Button::new(
                egui::RichText::new("Save")
                    .color(palette.on_accent)
                    .font(theme::semibold(13.0)),
            )
            .fill(palette.accent)
            .corner_radius(CornerRadius::same(8))
            .min_size(vec2(72.0, 32.0)),
        );
        if save.clicked() {
            app.actions.push(Action::SaveAppRegistration {
                client_id: app.form_client_id.clone(),
                tenant: app.form_tenant.clone(),
            });
        }
        if ui.link("How to register Wren").clicked() {
            app.actions.push(Action::OpenUrl(DOCS.into()));
        }
    });
    ui.label(
        egui::RichText::new("Or start Wren with WREN_CLIENT_ID and WREN_TENANT set.")
            .font(theme::regular(12.0))
            .color(palette.dim),
    );
}
