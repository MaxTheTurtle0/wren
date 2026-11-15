//! Small shared controls.

use egui::{Align2, Color32, CornerRadius, Response, Sense, Stroke, Ui, Vec2, vec2};

use crate::app::{App, Image};
use crate::model::Action;
use crate::theme::{self, Icon, Palette};

/// A padded single-line field with identical placeholder and input metrics.
pub fn text_field<'a>(text: &'a mut String, hint: &str, font: egui::FontId) -> egui::TextEdit<'a> {
    egui::TextEdit::singleline(text)
        // TextEdit's hint atoms otherwise use the body font independently.
        .hint_text(egui::RichText::new(hint).font(font.clone()))
        .font(font)
        // 8 pt above and below make a 32 pt field at the 13-14 pt sizes used.
        .margin(egui::Margin::symmetric(8, 8))
        .min_size(vec2(0.0, 32.0))
        .desired_width(f32::INFINITY)
}

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
                // Never below 11 pt; tiny avatars show one letter.
                if size <= 20.0 {
                    theme::initials(name).chars().take(1).collect()
                } else {
                    theme::initials(name)
                },
                theme::semibold((size * 0.38).round().max(11.0)),
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
        presence_dot(ui.painter(), palette, center, dot / 2.0, state.availability);
    }
    response
}

/// A presence dot. Shape tells the states apart as well as colour: Do not
/// disturb carries a bar, Away a hollow centre, Be right back clock hands,
/// Offline is a ring.
pub fn presence_dot(
    painter: &egui::Painter,
    palette: &Palette,
    center: egui::Pos2,
    radius: f32,
    availability: crate::model::Availability,
) {
    use crate::model::Availability::*;
    let color = palette.presence(availability);
    match availability {
        Offline | Unknown => {
            painter.circle_stroke(center, radius - 1.0, Stroke::new(2.0, color));
        }
        Away => {
            painter.circle_filled(center, radius, color);
            painter.circle_filled(center, radius * 0.4, palette.panel);
        }
        BeRightBack => {
            // A clock face: hands at twelve and three.
            painter.circle_filled(center, radius, color);
            let hand = Stroke::new((radius * 0.28).max(1.2), Color32::WHITE);
            painter.line_segment([center, center - vec2(0.0, radius * 0.6)], hand);
            painter.line_segment([center, center + vec2(radius * 0.5, 0.0)], hand);
        }
        DoNotDisturb => {
            painter.circle_filled(center, radius, color);
            painter.rect_filled(
                egui::Rect::from_center_size(center, vec2(radius * 1.1, (radius * 0.4).max(1.5))),
                CornerRadius::same(1),
                Color32::WHITE,
            );
        }
        Available | Busy => {
            painter.circle_filled(center, radius, color);
        }
    }
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

/// A filled accent button. Disabled, it turns neutral so it still reads as
/// a button.
pub fn primary_button(ui: &mut Ui, palette: &Palette, text: &str) -> Response {
    let enabled = ui.is_enabled();
    let (fill, fg) = if enabled {
        (palette.accent, palette.on_accent)
    } else {
        (palette.surface_active, palette.dim)
    };
    let button = egui::Button::new(
        egui::RichText::new(text)
            .color(fg)
            .font(theme::semibold(13.0)),
    )
    .fill(fill)
    // No grey control edge: the fill is the boundary.
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(8))
    .min_size(vec2(0.0, 32.0));
    // egui fades disabled widgets; this one has its own disabled colours.
    let response = ui
        .scope(|ui| {
            ui.set_opacity(1.0);
            ui.add(button)
        })
        .inner;
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

/// An outlined secondary button.
pub fn secondary_button(ui: &mut Ui, palette: &Palette, text: &str) -> Response {
    let button = egui::Button::new(
        egui::RichText::new(text)
            .color(palette.text)
            .font(theme::semibold(13.0)),
    )
    .fill(palette.surface)
    // A 3:1 edge, so it reads as a button and not a label.
    .stroke(Stroke::new(1.0, palette.control))
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
        (palette.surface, palette.text)
    };
    let galley = ui
        .ctx()
        .fonts_mut(|f| f.layout_no_wrap(text.to_owned(), theme::semibold(13.0), fg));
    // The same 32 pt height as other buttons.
    let size = vec2(galley.size().x + 18.0 + 22.0, 32.0);
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
        ui.painter().rect_filled(rect, CornerRadius::same(16), fill);
        if !accent {
            // The same 3:1 edge as other secondary controls.
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(16),
                Stroke::new(1.0, palette.control),
                egui::StrokeKind::Inside,
            );
        }
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 18.0, rect.center().y),
            Vec2::splat(15.0),
        );
        icon.image(fg, 15.0).paint_at(ui, icon_rect);
        ui.painter().galley(
            egui::pos2(rect.left() + 30.0, rect.center().y - galley.size().y / 2.0),
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
    empty_state_with_action(ui, palette, icon, title, text, None);
}

/// [`empty_state`] with a primary button under the text; returns whether
/// it was clicked.
pub fn empty_state_with_action(
    ui: &mut Ui,
    palette: &Palette,
    icon: Icon,
    title: &str,
    text: &str,
    action: Option<&str>,
) -> bool {
    let mut clicked = false;
    ui.vertical_centered(|ui| {
        // Stay inside narrow panes instead of widening them.
        ui.set_max_width((ui.available_width() - 48.0).clamp(120.0, 360.0));
        // Centred, a little above the middle (optical centre).
        let content = 128.0 + if action.is_some() { 48.0 } else { 0.0 };
        ui.add_space(((ui.available_height() - content) / 2.0 - 24.0).max(24.0));
        let size = 32.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(56.0), Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 28.0, palette.surface_hover);
        icon.image(palette.secondary, size).paint_at(
            ui,
            egui::Rect::from_center_size(rect.center(), Vec2::splat(size * 0.75)),
        );
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(title)
                .font(theme::semibold(15.0))
                .color(palette.text),
        );
        if !text.is_empty() {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(text)
                    .font(theme::regular(13.0))
                    .line_height(Some(theme::leading(13.0)))
                    .color(palette.secondary),
            );
        }
        if let Some(action) = action {
            ui.add_space(14.0);
            clicked = primary_button(ui, palette, action).clicked();
        }
    });
    clicked
}

/// A square checkbox with a 3:1 edge, filled with the accent when on.
/// Returns the response; `changed()` is set when it toggles.
pub fn checkbox(
    ui: &mut Ui,
    palette: &Palette,
    checked: &mut bool,
    text: impl Into<egui::WidgetText>,
) -> Response {
    let text = text.into();
    let box_size = 16.0;
    let gap = 10.0;
    let wrap_width = ui.available_width() - box_size - gap;
    let galley = text.into_galley(
        ui,
        Some(egui::TextWrapMode::Wrap),
        wrap_width,
        egui::TextStyle::Body,
    );
    let height = galley.size().y.max(box_size).max(26.0);
    let (rect, mut response) = ui.allocate_exact_size(
        vec2(box_size + gap + galley.size().x, height),
        Sense::click(),
    );
    if response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Checkbox,
            ui.is_enabled(),
            *checked,
            galley.text(),
        )
    });
    if ui.is_rect_visible(rect) {
        let first_line = galley.rows.first().map_or(box_size, |row| row.height());
        let top = rect.top() + ((height - galley.size().y) / 2.0).max(0.0);
        let square = egui::Rect::from_min_size(
            egui::pos2(rect.left(), top + (first_line - box_size) / 2.0),
            Vec2::splat(box_size),
        );
        let hovered = response.hovered() || response.has_focus();
        if *checked {
            let fill = if hovered {
                palette.accent_hover
            } else {
                palette.accent
            };
            ui.painter()
                .rect_filled(square, CornerRadius::same(4), fill);
            Icon::Check.image(palette.on_accent, 12.0).paint_at(
                ui,
                egui::Rect::from_center_size(square.center(), Vec2::splat(12.0)),
            );
        } else {
            ui.painter()
                .rect_filled(square, CornerRadius::same(4), palette.surface);
            ui.painter().rect_stroke(
                square,
                CornerRadius::same(4),
                Stroke::new(
                    1.5,
                    if hovered {
                        palette.secondary
                    } else {
                        palette.control
                    },
                ),
                egui::StrokeKind::Inside,
            );
        }
        if response.has_focus() {
            ui.painter().rect_stroke(
                square.expand(2.0),
                CornerRadius::same(6),
                Stroke::new(2.0, palette.accent),
                egui::StrokeKind::Outside,
            );
        }
        ui.painter().galley(
            egui::pos2(rect.left() + box_size + gap, top),
            galley,
            palette.text,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The accent bar on the left of a selected list row, so selection does
/// not rest on a fill alone.
pub fn selection_marker(ui: &Ui, palette: &Palette, row: egui::Rect) {
    let bar = egui::Rect::from_min_max(
        egui::pos2(row.left(), row.top() + 8.0),
        egui::pos2(row.left() + 3.0, row.bottom() - 8.0),
    );
    ui.painter()
        .rect_filled(bar, CornerRadius::same(2), palette.accent);
}

/// The 52 pt bar along the top of a main pane: title, subtitle, actions on
/// the right and a hairline below, as the conversation header has.
pub fn pane_header(
    ui: &mut Ui,
    palette: &Palette,
    title: &str,
    subtitle: &str,
    actions: impl FnOnce(&mut Ui),
) {
    let rect = egui::Rect::from_min_size(
        ui.max_rect().min,
        vec2(ui.max_rect().width(), theme::HEADER_HEIGHT),
    );
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, palette.outline),
    );
    let inset = super::header_inset(ui.ctx(), rect.left());
    // The title lines up with the page's content column; the actions sit at
    // the pane's right edge, as in a conversation header.
    let column = detail_column(rect);
    ui.scope_builder(egui::UiBuilder::new().max_rect(column), |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(inset);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(title)
                        .font(theme::semibold(18.0))
                        .color(palette.text),
                );
                if !subtitle.is_empty() {
                    // The when of a meeting matters: a readable size.
                    ui.label(
                        egui::RichText::new(subtitle)
                            .font(theme::medium(13.0))
                            .color(palette.secondary),
                    );
                }
            });
        });
    });
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(vec2(24.0, 0.0)))
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        actions,
    );
    ui.allocate_rect(rect, Sense::hover());
}

/// The Lucide chevron for a drop-down, instead of egui's triangle.
pub fn combo_chevron(
    color: Color32,
) -> impl FnOnce(&Ui, egui::Rect, &egui::style::WidgetVisuals, bool) {
    move |ui, rect, _, _| {
        Icon::ChevronDown.image(color, 16.0).paint_at(
            ui,
            egui::Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
        );
    }
}

/// The Lucide chevron for a disclosure, turning as it opens.
pub fn disclosure_chevron(color: Color32) -> impl FnOnce(&mut Ui, f32, &Response) {
    move |ui, openness, response| {
        Icon::ChevronRight
            .image(color, 14.0)
            .rotate(openness * std::f32::consts::FRAC_PI_2, Vec2::splat(0.5))
            .paint_at(
                ui,
                egui::Rect::from_center_size(response.rect.center(), Vec2::splat(14.0)),
            );
    }
}

/// Two options side by side in one bordered control; `value` is true when
/// the second is chosen.
pub fn segmented(ui: &mut Ui, palette: &Palette, value: &mut bool, first: &str, second: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (index, label) in [first, second].into_iter().enumerate() {
            let on = *value == (index == 1);
            let corner = if index == 0 {
                CornerRadius {
                    nw: 6,
                    sw: 6,
                    ne: 0,
                    se: 0,
                }
            } else {
                CornerRadius {
                    nw: 0,
                    sw: 0,
                    ne: 6,
                    se: 6,
                }
            };
            let button = egui::Button::new(
                egui::RichText::new(label)
                    .font(theme::semibold(13.0))
                    .color(if on { palette.text } else { palette.secondary }),
            )
            // A view toggle, not an action: neutral, with the chosen
            // segment filled.
            .fill(if on {
                palette.surface_active
            } else {
                palette.surface
            })
            .stroke(Stroke::new(1.0, palette.control))
            .corner_radius(corner)
            .min_size(vec2(56.0, 28.0));
            let response = ui
                .add(button)
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if on {
                // The chosen view, marked as well as filled.
                let rect = response.rect;
                ui.painter().rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(rect.left() + 6.0, rect.bottom() - 3.0),
                        egui::pos2(rect.right() - 6.0, rect.bottom() - 1.0),
                    ),
                    CornerRadius::same(1),
                    palette.accent,
                );
            }
            if response.clicked() {
                *value = index == 1;
            }
        }
    });
}

/// `max` wide at most, centred in `rect`: the reading column shared by a
/// conversation's header, history and composer, and by detail pages.
pub fn centred_column(rect: egui::Rect, max: f32) -> egui::Rect {
    let width = rect.width().min(max);
    egui::Rect::from_min_size(
        egui::pos2(rect.center().x - width / 2.0, rect.top()),
        vec2(width, rect.height()),
    )
}

/// Help for the checkbox above it, indented to the checkbox's label.
pub fn checkbox_note(ui: &mut Ui, palette: &Palette, text: &str) -> Response {
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 26,
            right: 0,
            top: 0,
            bottom: 4,
        })
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .font(theme::regular(13.0))
                    .line_height(Some(theme::leading(13.0)))
                    .color(palette.secondary),
            )
        })
        .inner
}

/// A detail page's column (Calendar, Calls): at most 720 pt wide, starting
/// 30 pt in, where a conversation's header starts at the usual window width.
pub fn detail_column(rect: egui::Rect) -> egui::Rect {
    let inner = rect.shrink2(vec2(30.0, 0.0));
    egui::Rect::from_min_size(
        inner.min,
        vec2(
            inner.width().min(super::calendar::DETAIL_WIDTH),
            inner.height(),
        ),
    )
}

/// One row of a menu: an icon column, then the label.
pub fn menu_row(
    ui: &mut Ui,
    palette: &Palette,
    width: f32,
    icon: Icon,
    label: &str,
    color: Color32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 28.0), Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(6), palette.surface_hover);
    }
    icon.image(palette.secondary, 14.0).paint_at(
        ui,
        egui::Rect::from_center_size(
            egui::pos2(rect.left() + 16.0, rect.center().y),
            Vec2::splat(14.0),
        ),
    );
    ui.painter().text(
        egui::pos2(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        theme::regular(13.0),
        color,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A soft shadow along the top (or bottom) edge of a scroll area, where
/// content continues out of view. It darkens the background slightly
/// instead of fading text, so what shows keeps its contrast.
pub fn scroll_shadow(ui: &Ui, palette: &Palette, viewport: egui::Rect, top: bool) {
    let depth = 8.0;
    let strength = if palette.dark { 90.0 } else { 26.0 };
    for step in 0..4 {
        let t = step as f32 / 4.0;
        let alpha = (strength * (1.0 - t) * (1.0 - t)) as u8;
        let y = if top {
            viewport.top() + step as f32 * depth / 4.0
        } else {
            viewport.bottom() - (step + 1) as f32 * depth / 4.0
        };
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(viewport.left(), y),
                vec2(viewport.width(), depth / 4.0),
            ),
            0.0,
            Color32::from_black_alpha(alpha),
        );
    }
}

/// A label above a field or drop-down.
pub fn field_label(ui: &mut Ui, palette: &Palette, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(theme::medium(13.0))
            .color(palette.secondary),
    );
}

/// A shortcut such as `Ctrl+Shift+A` drawn as key caps.
pub fn keycaps(ui: &mut Ui, palette: &Palette, keys: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (index, part) in keys.split(" / ").enumerate() {
            if index > 0 {
                ui.label(
                    egui::RichText::new("/")
                        .font(theme::regular(12.0))
                        .color(palette.dim),
                );
            }
            egui::Frame::new()
                .fill(palette.surface_hover)
                .stroke(Stroke::new(1.0, palette.outline))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(egui::Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(part)
                            .font(theme::medium(12.0))
                            .color(palette.text),
                    );
                });
        }
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
    #[cfg(target_os = "macos")]
    ui.push_id(id, |ui| fastframe_titlebar::drag(ui, rect));

    // Wren keeps the native frame on Windows and Linux. The shared drag
    // helper only handles those platforms when its drawn title bar is on.
    #[cfg(not(target_os = "macos"))]
    {
        let response = ui.interact(rect, ui.id().with(id), Sense::click_and_drag());
        if response.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if response.double_clicked() {
            let maximized = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
        }
    }
}

/// Pushes `action` when `response` was clicked.
pub fn on_click(response: &Response, actions: &mut Vec<Action>, action: Action) {
    if response.clicked() {
        actions.push(action);
    }
}
