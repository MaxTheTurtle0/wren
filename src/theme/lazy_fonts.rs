//! Load the platform's large script fallbacks only when visible text needs one.

use std::collections::HashMap;
use std::sync::{Arc, Weak};

use egui::epaint::{Galley, Shape};
use egui::{Context, FontDefinitions, FullOutput, mutex::Mutex};

#[derive(Default)]
pub(super) struct LazyFonts {
    seen: HashMap<usize, Weak<Galley>>,
    result: Arc<Mutex<Option<FontDefinitions>>>,
    requested: bool,
    loaded: bool,
}

impl egui::Plugin for LazyFonts {
    fn debug_name(&self) -> &'static str {
        "wren-lazy-fonts"
    }

    fn on_begin_pass(&mut self, ui: &mut egui::Ui) {
        if let Some(fonts) = self.result.lock().take() {
            ui.ctx().set_fonts(fonts);
            ui.ctx()
                .data_mut(|d| d.insert_temp(egui::Id::new("wren-font-revision"), 1_u64));
            self.loaded = true;
            self.seen.clear();
        }
    }

    fn output_hook(&mut self, ctx: &Context, output: &mut FullOutput) {
        if self.loaded || self.requested {
            return;
        }
        let mut previous = std::mem::take(&mut self.seen);
        let needed = output
            .shapes
            .iter()
            .any(|clipped| self.visit(ctx, &clipped.shape, &mut previous));
        if needed {
            self.requested = true;
            let result = Arc::clone(&self.result);
            let ctx = ctx.clone();
            std::thread::Builder::new()
                .name("wren-script-fonts".into())
                .spawn(move || {
                    let fonts = super::font_definitions(false, true);
                    *result.lock() = Some(fonts);
                    ctx.request_repaint();
                })
                .expect("could not start the font loader");
        }
    }
}

impl LazyFonts {
    fn visit(
        &mut self,
        ctx: &Context,
        shape: &Shape,
        previous: &mut HashMap<usize, Weak<Galley>>,
    ) -> bool {
        match shape {
            Shape::Vec(shapes) => shapes.iter().any(|s| self.visit(ctx, s, previous)),
            Shape::Text(text) => {
                let galley = &text.galley;
                let key = Arc::as_ptr(galley) as usize;
                let seen = previous
                    .remove(&key)
                    .and_then(|w| w.upgrade())
                    .is_some_and(|old| Arc::ptr_eq(&old, galley));
                self.seen.insert(key, Arc::downgrade(galley));
                if seen || galley.job.text.is_ascii() {
                    return false;
                }
                ctx.fonts_mut(|fonts| {
                    galley.job.sections.iter().any(|section| {
                        let text =
                            &galley.job.text[section.byte_range.start.0..section.byte_range.end.0];
                        fastframe_emoji::pieces(text)
                            .into_iter()
                            .any(|piece| match piece {
                                fastframe_emoji::Piece::Text(text) => text.chars().any(|c| {
                                    !c.is_ascii()
                                        && !c.is_control()
                                        && !fonts.has_glyph(&section.format.font_id, c)
                                }),
                                fastframe_emoji::Piece::Emoji(_) => false,
                            })
                    })
                })
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_and_emoji_do_not_request_the_large_script_fallbacks() {
        let ctx = Context::default();
        ctx.set_fonts(super::super::font_definitions(true, false));
        ctx.add_plugin(LazyFonts::default());
        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.label("Café 🙂 Grüße");
        });
        output.textures_delta.clear();
        assert!(!ctx.plugin::<LazyFonts>().lock().requested);
        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.label("日本語");
        });
        output.textures_delta.clear();
        assert!(ctx.plugin::<LazyFonts>().lock().requested);
    }
}
