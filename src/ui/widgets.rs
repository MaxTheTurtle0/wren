//! Small shared controls.

use egui::{Align2, Color32, CornerRadius, Response, Sense, Stroke, Ui, Vec2, vec2};

use crate::app::{App, Image};
use crate::model::Action;
use crate::theme::{self, Icon, Palette};

/// Loading is event driven. A static indicator lets a slow request leave the
/// entire window asleep until its backend sends the next event.
pub fn loading(ui: &mut Ui, color: Color32, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        Icon::Loader.image(color, size).paint_at(ui, rect);
    }
    response.on_hover_text("Loading…")
}

/// A round avatar: the person's photo when Wren has it, their initials
/// otherwise, and a presence dot when `presence` is known.
pub fn avatar(
    ui: &mut Ui,
    app: &App,
    user: Option<&str>,
    name: &str,
    size: f32,
    presence: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let palette = &app.palette;
    let photo = user.and_then(|u| match app.photos.get(u) {
        Some(Image::Ready(uri)) => Some(uri.clone()),
        _ => None,
    });
    match &photo {
        Some(uri) => {
            egui::Image::new(uri.clone())
                .corner_radius(CornerRadius::same((size / 2.0) as u8))
                .paint_at(ui, rect);
        }
        None => {
            let seed = user.unwrap_or(name);
            ui.painter().circle_filled(
                rect.center(),
                size / 2.0,
                theme::avatar_color(seed, palette.dark),
            );
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                theme::initials(name),
                theme::semibold((size * 0.38).round()),
                Color32::WHITE,
            );
        }
    }
    // Ask for the photo once the avatar is on screen.
    let missing_photo = match photo {
        Some(ref uri) => crate::images::needs_source(
            ui.ctx(),
            uri,
            egui::load::SizeHint::Width((size * ui.ctx().pixels_per_point()).ceil() as u32),
        ),
        None => user.is_some_and(|user| !app.photos.contains_key(user)),
    };
    if let Some(user) = user
        && !app.demo
        && missing_photo
    {
        ui.ctx().data_mut(|data| {
            data.get_temp_mut_or_default::<Vec<String>>(egui::Id::new("wren-photo-wants"))
                .push(user.to_owned())
        });
    }
    if presence
        && let Some(user) = user
        && let Some(state) = app.presence.get(user)
    {
        let dot = (size * 0.30).max(9.0);
        let center = rect.right_bottom() - vec2(dot * 0.42, dot * 0.42);
        ui.painter()
            .circle_filled(center, dot / 2.0 + 1.5, palette.panel);
        ui.painter()
            .circle_filled(center, dot / 2.0, palette.presence(state.availability));
    }
    response
}

/// A borderless icon button with a hover background and tooltip.
pub fn icon_button(ui: &mut Ui, palette: &Palette, icon: Icon, tooltip: &str) -> Response {
    icon_button_sized(ui, palette, icon, tooltip, 30.0, 18.0, palette.secondary)
}

pub fn icon_button_sized(
    ui: &mut Ui,
    palette: &Palette,
    icon: Icon,
    tooltip: &str,
    size: f32,
    icon_size: f32,
    tint: Color32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if ui.is_rect_visible(rect) {
        let hovered = response.hovered() || response.has_focus();
        if hovered {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(6), palette.surface_hover);
        }
        let tint = if hovered { palette.text } else { tint };
        icon.image(tint, icon_size).paint_at(
            ui,
            egui::Rect::from_center_size(rect.center(), Vec2::splat(icon_size)),
        );
    }
    response
        .on_hover_text(tooltip)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A filled accent button.
pub fn primary_button(ui: &mut Ui, palette: &Palette, text: &str) -> Response {
    let button = egui::Button::new(
        egui::RichText::new(text)
            .color(palette.on_accent)
            .font(theme::semibold(14.0)),
    )
    .fill(palette.accent)
    .corner_radius(CornerRadius::same(8))
    .min_size(vec2(0.0, 34.0));
    ui.add(button)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// An outlined secondary button.
pub fn secondary_button(ui: &mut Ui, palette: &Palette, text: &str) -> Response {
    let button = egui::Button::new(
        egui::RichText::new(text)
            .color(palette.text)
            .font(theme::medium(13.0)),
    )
    .fill(palette.surface)
    .stroke(Stroke::new(1.0, palette.outline))
    .corner_radius(CornerRadius::same(8))
    .min_size(vec2(0.0, 32.0));
    ui.add(button)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A small pill button with an icon and a label (Join, Call, ...).
pub fn pill(ui: &mut Ui, palette: &Palette, icon: Icon, text: &str, accent: bool) -> Response {
    let (fill, fg) = if accent {
        (palette.accent, palette.on_accent)
    } else {
        (palette.surface_hover, palette.text)
    };
    let galley = ui
        .ctx()
        .fonts_mut(|f| f.layout_no_wrap(text.to_owned(), theme::semibold(13.0), fg));
    let size = vec2(galley.size().x + 16.0 + 22.0, 28.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if response.hovered() {
            if accent {
                palette.accent_hover
            } else {
                palette.surface_active
            }
        } else {
            fill
        };
        ui.painter().rect_filled(rect, CornerRadius::same(14), fill);
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 17.0, rect.center().y),
            Vec2::splat(15.0),
        );
        icon.image(fg, 15.0).paint_at(ui, icon_rect);
        ui.painter().galley(
            egui::pos2(rect.left() + 29.0, rect.center().y - galley.size().y / 2.0),
            galley,
            fg,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A count badge (unread chats).
pub fn badge(ui: &Ui, palette: &Palette, center: egui::Pos2, count: usize) {
    let text = if count > 99 {
        "99+".to_owned()
    } else {
        count.to_string()
    };
    let galley = ui
        .ctx()
        .fonts_mut(|f| f.layout_no_wrap(text, theme::bold(11.0), palette.on_accent));
    let width = (galley.size().x + 9.0).max(17.0);
    let rect = egui::Rect::from_center_size(center, vec2(width, 17.0));
    ui.painter()
        .rect_filled(rect, CornerRadius::same(9), palette.accent);
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        palette.on_accent,
    );
}

/// One line of text, cut with an ellipsis to the width it has.
pub fn line(ui: &mut Ui, text: &str, font: egui::FontId, color: Color32) -> Response {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat {
            font_id: font,
            color,
            ..Default::default()
        },
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(ui.available_width().max(10.0));
    ui.add(egui::Label::new(job).selectable(false))
}

/// A quiet message in an empty pane.
pub fn empty_state(ui: &mut Ui, palette: &Palette, icon: Icon, title: &str, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() * 0.30).max(24.0));
        icon.image(palette.dim, 40.0).paint_at(
            ui,
            egui::Rect::from_center_size(
                ui.cursor().center_top() + vec2(0.0, 20.0),
                Vec2::splat(40.0),
            ),
        );
        ui.add_space(52.0);
        ui.label(
            egui::RichText::new(title)
                .font(theme::semibold(15.0))
                .color(palette.text),
        );
        ui.add_space(4.0);
        ui.set_max_width(360.0);
        ui.label(egui::RichText::new(text).color(palette.secondary));
    });
}

/// A small uppercase heading for groups in lists.
pub fn section_label(ui: &mut Ui, palette: &Palette, text: &str) {
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .font(theme::semibold(11.0))
            .color(palette.dim)
            .extra_letter_spacing(0.6),
    );
}

/// Requests photos for avatars drawn this frame (collected by [`avatar`]).
pub fn request_photos(app: &mut App, ctx: &egui::Context) {
    let wants: Vec<String> = ctx.data_mut(|data| {
        std::mem::take(
            data.get_temp_mut_or_default::<Vec<String>>(egui::Id::new("wren-photo-wants")),
        )
    });
    for user in wants {
        let missing = match app.photos.get(&user) {
            Some(Image::Ready(uri)) => !crate::images::contains(ctx, uri),
            None => true,
            _ => false,
        };
        if missing {
            app.photos.insert(user.clone(), Image::Loading);
            app.backend.send(crate::backend::Command::Photo(user));
        }
    }
}

/// Makes `rect` drag the window (the custom title bar on macOS), and
/// double-clicking it zoom as the system setting says.
pub fn window_drag(ui: &mut Ui, rect: egui::Rect, id: &str) {
    let response = ui.interact(rect, ui.id().with(id), Sense::click_and_drag());
    if response.drag_started_by(egui::PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if response.double_clicked() {
        match fastframe_macos::double_click_action() {
            fastframe_macos::DoubleClick::Minimize => ui
                .ctx()
                .send_viewport_cmd(egui::ViewportCommand::Minimized(true)),
            fastframe_macos::DoubleClick::Nothing => {}
            _ => {
                let maximized = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }
        }
    }
}

/// Pushes `action` when `response` was clicked.
pub fn on_click(response: &Response, actions: &mut Vec<Action>, action: Action) {
    if response.clicked() {
        actions.push(action);
    }
}
