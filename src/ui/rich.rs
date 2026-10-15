//! Draws [`RichText`] with egui: each block is one laid-out galley, so text
//! stays selectable, wraps natively and costs one shape per paragraph.
//! Links are found under the pointer from the galley's character positions.

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Color32, CornerRadius, Margin, Sense, Stroke, Ui, vec2};

use crate::app::{App, Image};
use crate::html::{Block, RichText, Span};
use crate::model::Action;
use crate::theme::{self, Palette};

pub struct Look {
    pub size: f32,
    pub color: Color32,
}

impl Look {
    pub fn body(palette: &Palette) -> Self {
        Self {
            size: 14.0,
            color: palette.text,
        }
    }
}

/// Lays out spans into a job, returning the link of each character range.
fn job(
    spans: &[Span],
    look: &Look,
    palette: &Palette,
    width: f32,
) -> (LayoutJob, Vec<(usize, usize, String)>) {
    let mut job = LayoutJob::default();
    let mut links = Vec::new();
    let mut chars = 0;
    for span in spans {
        let style = &span.style;
        let font = if style.code {
            theme::mono(look.size - 1.0)
        } else if style.bold || style.mention {
            theme::semibold(look.size)
        } else {
            theme::regular(look.size)
        };
        let color = if style.link.is_some() {
            palette.link
        } else if style.mention {
            palette.accent
        } else {
            look.color
        };
        let format = TextFormat {
            font_id: font,
            color,
            italics: style.italic,
            underline: if style.underline || style.link.is_some() {
                Stroke::new(1.0, color.gamma_multiply(0.6))
            } else {
                Stroke::NONE
            },
            strikethrough: if style.strike {
                Stroke::new(1.0, color)
            } else {
                Stroke::NONE
            },
            background: if style.code {
                palette.surface_active
            } else if style.mention {
                palette.highlight
            } else {
                Color32::TRANSPARENT
            },
            line_height: Some(look.size * 1.42),
            ..Default::default()
        };
        let count = span.text.chars().count();
        if let Some(link) = &style.link {
            links.push((chars, chars + count, link.clone()));
        }
        chars += count;
        job.append(&span.text, 0.0, format);
    }
    job.wrap = TextWrapping {
        max_width: width,
        ..Default::default()
    };
    (job, links)
}

/// One paragraph: selectable text with clickable links.
fn paragraph(
    ui: &mut Ui,
    spans: &[Span],
    look: &Look,
    palette: &Palette,
    width: f32,
    actions: &mut Vec<Action>,
) {
    let (job, links) = job(spans, look, palette, width);
    let galley = ui.ctx().fonts_mut(|f| f.layout_job(job));
    let response = ui.add(
        egui::Label::new(galley.clone())
            .selectable(true)
            .sense(Sense::click()),
    );
    if links.is_empty() {
        return;
    }
    let Some(pointer) = response.hover_pos() else {
        return;
    };
    let cursor = galley.cursor_from_pos(pointer - response.rect.min);
    let index = cursor.index.0;
    // The character under the pointer, not the gap after it.
    let glyph = galley.pos_from_cursor(cursor);
    let index = if pointer.x < response.rect.min.x + glyph.min.x {
        index.saturating_sub(1)
    } else {
        index
    };
    if let Some((_, _, link)) = links
        .iter()
        .find(|(start, end, _)| (*start..*end).contains(&index))
    {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        response.clone().on_hover_text_at_pointer(link.as_str());
        if response.clicked() {
            actions.push(Action::OpenUrl(link.clone()));
        }
    }
}

/// Draws a message body within `width`.
pub fn show(
    ui: &mut Ui,
    app: &App,
    text: &RichText,
    look: &Look,
    width: f32,
    actions: &mut Vec<Action>,
) {
    let palette = &app.palette;
    ui.spacing_mut().item_spacing.y = 4.0;
    for block in &text.blocks {
        match block {
            Block::Paragraph(spans) => paragraph(ui, spans, look, palette, width, actions),
            Block::Heading(spans) => {
                let heading = Look {
                    size: look.size + 1.0,
                    color: look.color,
                };
                let bold: Vec<Span> = spans
                    .iter()
                    .map(|s| {
                        let mut s = s.clone();
                        s.style.bold = true;
                        s
                    })
                    .collect();
                paragraph(ui, &bold, &heading, palette, width, actions);
            }
            Block::Quote(spans) => {
                egui::Frame::new()
                    .inner_margin(Margin {
                        left: 10,
                        right: 4,
                        top: 2,
                        bottom: 2,
                    })
                    .show(ui, |ui| {
                        let rect = ui.max_rect();
                        let quote = Look {
                            size: look.size,
                            color: palette.secondary,
                        };
                        paragraph(ui, spans, &quote, palette, width - 14.0, actions);
                        let bar = egui::Rect::from_min_max(
                            rect.left_top() - vec2(10.0, 0.0),
                            egui::pos2(rect.left() - 7.0, ui.min_rect().bottom()),
                        );
                        ui.painter()
                            .rect_filled(bar, CornerRadius::same(2), palette.outline);
                    });
            }
            Block::Item { ordered, spans } => {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let marker = match ordered {
                        Some(n) => format!("{n}."),
                        None => "•".into(),
                    };
                    ui.add_sized(
                        vec2(14.0, look.size * 1.42),
                        egui::Label::new(
                            egui::RichText::new(marker)
                                .font(theme::regular(look.size))
                                .color(palette.secondary),
                        ),
                    );
                    paragraph(ui, spans, look, palette, width - 20.0, actions);
                });
            }
            Block::Code(code) => {
                egui::Frame::new()
                    .fill(palette.surface_hover)
                    .stroke(Stroke::new(1.0, palette.outline))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.set_max_width(width - 20.0);
                        let mut job = LayoutJob::single_section(
                            code.clone(),
                            TextFormat {
                                font_id: theme::mono(12.0),
                                color: palette.text,
                                ..Default::default()
                            },
                        );
                        job.wrap.max_width = width - 20.0;
                        ui.add(egui::Label::new(job).selectable(true));
                    });
            }
            Block::Image {
                src,
                alt,
                width: w,
                height: h,
            } => image(ui, app, src, alt.as_deref(), *w, *h, width, actions),
            Block::Rule => {
                ui.add_space(2.0);
                let y = ui.cursor().top();
                ui.painter().hline(
                    ui.cursor().left()..=ui.cursor().left() + width,
                    y,
                    Stroke::new(1.0, palette.outline),
                );
                ui.add_space(4.0);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn image(
    ui: &mut Ui,
    app: &App,
    src: &str,
    alt: Option<&str>,
    w: Option<f32>,
    h: Option<f32>,
    width: f32,
    actions: &mut Vec<Action>,
) {
    let palette = &app.palette;
    let max = vec2(width.min(360.0), 280.0);
    let natural = match (w, h) {
        (Some(w), Some(h)) if w > 0.0 && h > 0.0 => vec2(w, h),
        _ => vec2(240.0, 160.0),
    };
    let scale = (max.x / natural.x).min(max.y / natural.y).min(1.0);
    let size = natural * scale;
    match app.media.get(src) {
        Some(Image::Ready(uri)) => {
            let response = ui
                .add(
                    egui::Image::new(uri.clone())
                        .fit_to_exact_size(size)
                        .corner_radius(CornerRadius::same(6))
                        .sense(Sense::click())
                        .alt_text(alt.unwrap_or("Image")),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text("Open image preview");
            if response.clicked() {
                actions.push(Action::PreviewImage(uri.clone()));
            }
            if ui.is_rect_visible(response.rect)
                && crate::images::needs_source(
                    ui.ctx(),
                    uri,
                    egui::load::SizeHint::Size {
                        width: (size.x * ui.ctx().pixels_per_point()).ceil() as u32,
                        height: (size.y * ui.ctx().pixels_per_point()).ceil() as u32,
                        maintain_aspect_ratio: true,
                    },
                )
            {
                actions.push(Action::LoadMedia(src.to_owned()));
            }
        }
        state => {
            let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter()
                .rect_filled(rect, CornerRadius::same(6), palette.surface_hover);
            let label = match state {
                Some(Image::Missing) => "Image unavailable",
                _ => "Loading image…",
            };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                theme::regular(12.0),
                palette.dim,
            );
            // Only images on screen are fetched.
            if state.is_none() && ui.is_rect_visible(response.rect) {
                actions.push(Action::LoadMedia(src.to_owned()));
            }
        }
    }
}

/// A one-line rich preview for quotes: plain text, secondary colour.
pub fn quote_block(
    ui: &mut Ui,
    palette: &Palette,
    author: Option<&str>,
    preview: &str,
    width: f32,
) -> egui::Response {
    egui::Frame::new()
        .fill(palette.surface_hover)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin {
            left: 10,
            right: 10,
            top: 5,
            bottom: 6,
        })
        .show(ui, |ui| {
            ui.set_max_width(width - 20.0);
            let rect = ui.max_rect();
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                if let Some(author) = author {
                    ui.label(
                        egui::RichText::new(author)
                            .font(theme::semibold(12.0))
                            .color(palette.secondary),
                    );
                }
                let mut job = LayoutJob::single_section(
                    preview.to_owned(),
                    TextFormat {
                        font_id: theme::regular(13.0),
                        color: palette.secondary,
                        ..Default::default()
                    },
                );
                job.wrap = TextWrapping {
                    max_width: width - 24.0,
                    max_rows: 3,
                    break_anywhere: false,
                    overflow_character: Some('…'),
                };
                ui.add(egui::Label::new(job));
            });
            let bar = egui::Rect::from_min_max(
                rect.left_top() - vec2(10.0, 5.0),
                egui::pos2(rect.left() - 7.0, ui.min_rect().bottom() + 6.0),
            );
            ui.painter().rect_filled(
                bar,
                CornerRadius {
                    nw: 6,
                    sw: 6,
                    ne: 0,
                    se: 0,
                },
                palette.accent,
            );
        })
        .response
}
