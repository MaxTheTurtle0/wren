//! Colours, type, spacing and icons.
//!
//! [`Palette`] holds every colour the interface uses, in a dark and a light
//! variant. It implements fastframe's palette trait, so the shared palettes
//! (Catppuccin, Nord, Rosé Pine, Tokyo Night, ...) restyle Wren too.

use std::collections::BTreeSet;

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Vec2};
use fastframe_fonts::Weight;

use crate::settings::ThemeChoice;

fastframe_icons::icons! {
    /// Every icon the interface draws: Lucide (ISC), 24 px outlines in white
    /// so egui's tint colours them.
    pub enum Icon {
        prefix: "wren-icon-",
        directory: "../assets/icons/",
        Chat => "message-circle",
        Teams => lucide "users",
        Calendar => "calendar",
        Phone => "phone",
        Video => "video",
        Settings => lucide "settings",
        Search => lucide "search",
        Send => "send",
        Paperclip => "paperclip",
        Smile => "smile",
        Reply => "reply",
        Pencil => lucide "pencil",
        Trash => lucide "trash-2",
        Copy => lucide "copy",
        More => lucide "ellipsis",
        Close => lucide "x",
        Hash => "hash",
        Lock => lucide "lock",
        At => "at-sign",
        Bell => "bell",
        BellOff => "bell-off",
        Refresh => lucide "refresh-cw",
        LogOut => lucide "log-out",
        Alert => lucide "circle-alert",
        Check => lucide "check",
        ChevronDown => lucide "chevron-down",
        ChevronRight => lucide "chevron-right",
        Plus => lucide "plus",
        ExternalLink => lucide "external-link",
        Info => lucide "info",
        Moon => lucide "moon",
        Sun => lucide "sun",
        Monitor => lucide "monitor",
        Compose => lucide "square-pen",
        User => lucide "user",
        Offline => "wifi-off",
        File => "file-text",
        Image => "image",
        Download => "download",
        Keyboard => "keyboard",
        Loader => "loader-circle",
        Sidebar => lucide "panel-left",
        Clock => lucide "clock",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Palette {
    pub dark: bool,
    /// The conversation's background.
    pub window: Color32,
    /// The sidebar.
    pub panel: Color32,
    /// The navigation rail.
    pub rail: Color32,
    pub surface: Color32,
    pub surface_hover: Color32,
    pub surface_active: Color32,
    pub outline: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub on_accent: Color32,
    pub danger: Color32,
    pub warning: Color32,
    pub success: Color32,
    pub overlay: Color32,
    pub shadow: Color32,
    /// Behind messages that mention the person, and the row being replied to.
    pub highlight: Color32,
    pub link: Color32,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            dark: true,
            window: Color32::from_rgb(0x17, 0x18, 0x1b),
            panel: Color32::from_rgb(0x1d, 0x1e, 0x22),
            rail: Color32::from_rgb(0x13, 0x14, 0x17),
            surface: Color32::from_rgb(0x25, 0x26, 0x2b),
            surface_hover: Color32::from_rgb(0x2c, 0x2d, 0x33),
            surface_active: Color32::from_rgb(0x34, 0x35, 0x3c),
            outline: Color32::from_rgb(0x30, 0x31, 0x37),
            text: Color32::from_rgb(0xe9, 0xe9, 0xec),
            secondary: Color32::from_rgb(0xa3, 0xa5, 0xad),
            dim: Color32::from_rgb(0x6d, 0x6f, 0x78),
            accent: Color32::from_rgb(0xe8, 0x8c, 0x3e),
            accent_hover: Color32::from_rgb(0xf2, 0x9e, 0x55),
            on_accent: Color32::from_rgb(0x1c, 0x12, 0x08),
            danger: Color32::from_rgb(0xef, 0x5b, 0x5b),
            warning: Color32::from_rgb(0xe3, 0xb3, 0x41),
            success: Color32::from_rgb(0x4c, 0xc3, 0x6e),
            overlay: Color32::from_rgba_premultiplied(0, 0, 0, 150),
            shadow: Color32::from_rgba_premultiplied(0, 0, 0, 110),
            highlight: Color32::from_rgba_premultiplied(0x3a, 0x25, 0x10, 0x60),
            link: Color32::from_rgb(0x7c, 0xb4, 0xf5),
        }
    }

    pub fn light() -> Self {
        Self {
            dark: false,
            window: Color32::from_rgb(0xff, 0xff, 0xff),
            panel: Color32::from_rgb(0xf6, 0xf6, 0xf7),
            rail: Color32::from_rgb(0xec, 0xec, 0xee),
            surface: Color32::from_rgb(0xff, 0xff, 0xff),
            surface_hover: Color32::from_rgb(0xee, 0xee, 0xf0),
            surface_active: Color32::from_rgb(0xe4, 0xe4, 0xe8),
            outline: Color32::from_rgb(0xdf, 0xdf, 0xe3),
            text: Color32::from_rgb(0x1b, 0x1c, 0x1f),
            secondary: Color32::from_rgb(0x58, 0x5b, 0x63),
            dim: Color32::from_rgb(0x8c, 0x8e, 0x96),
            accent: Color32::from_rgb(0xc2, 0x63, 0x18),
            accent_hover: Color32::from_rgb(0xa8, 0x53, 0x0f),
            on_accent: Color32::from_rgb(0xff, 0xff, 0xff),
            danger: Color32::from_rgb(0xd3, 0x34, 0x34),
            warning: Color32::from_rgb(0xa8, 0x6f, 0x12),
            success: Color32::from_rgb(0x1f, 0x8f, 0x4c),
            overlay: Color32::from_rgba_premultiplied(0, 0, 0, 90),
            shadow: Color32::from_rgba_premultiplied(0, 0, 0, 40),
            highlight: Color32::from_rgb(0xfd, 0xf1, 0xe5),
            link: Color32::from_rgb(0x1f, 0x63, 0xc6),
        }
    }

    pub fn presence(&self, availability: crate::model::Availability) -> Color32 {
        use crate::model::Availability::*;
        match availability {
            Available => self.success,
            Busy | DoNotDisturb => self.danger,
            BeRightBack | Away => self.warning,
            Offline | Unknown => self.dim,
        }
    }
}

impl fastframe_theme::Palette for Palette {
    fn base(base: fastframe_theme::Base) -> Self {
        match base {
            fastframe_theme::Base::Dark => Self::dark(),
            fastframe_theme::Base::Light => Self::light(),
        }
    }

    fn set(&mut self, name: &str, color: Color32) -> bool {
        let slot = match name {
            "window" => &mut self.window,
            "panel" => &mut self.panel,
            "rail" => &mut self.rail,
            "surface" => &mut self.surface,
            "surface_hover" => &mut self.surface_hover,
            "surface_active" => &mut self.surface_active,
            "outline" => &mut self.outline,
            "text" => &mut self.text,
            "secondary" => &mut self.secondary,
            "dim" => &mut self.dim,
            "accent" => &mut self.accent,
            "accent_hover" => &mut self.accent_hover,
            "on_accent" => &mut self.on_accent,
            "danger" => &mut self.danger,
            "warning" => &mut self.warning,
            "success" => &mut self.success,
            "overlay" => &mut self.overlay,
            "shadow" => &mut self.shadow,
            "highlight" => &mut self.highlight,
            "link" => &mut self.link,
            _ => return false,
        };
        *slot = color;
        true
    }

    fn derive(&mut self, given: &BTreeSet<&str>) {
        if given.contains("panel") && !given.contains("rail") {
            self.rail = self
                .panel
                .gamma_multiply(if self.dark { 0.85 } else { 0.97 });
            self.rail = Color32::from_rgb(self.rail.r(), self.rail.g(), self.rail.b());
        }
        if given.contains("accent") && !given.contains("highlight") {
            self.highlight = self
                .accent
                .gamma_multiply(if self.dark { 0.18 } else { 0.10 });
        }
        if given.contains("accent") && !given.contains("link") {
            self.link = self.accent;
        }
        if !given.contains("success") {
            self.success = if self.dark {
                Palette::dark().success
            } else {
                Palette::light().success
            };
        }
    }
}

/// The shared palettes, by file name.
pub fn presets() -> Vec<fastframe_theme::CustomTheme<Palette>> {
    fastframe_theme::presets::themes::<Palette>().collect()
}

/// The palette a choice resolves to; `System` follows the desktop.
pub fn resolve(choice: &ThemeChoice, system_dark: bool) -> Palette {
    match choice {
        ThemeChoice::System => {
            if system_dark {
                Palette::dark()
            } else {
                Palette::light()
            }
        }
        ThemeChoice::Light => Palette::light(),
        ThemeChoice::Dark => Palette::dark(),
        ThemeChoice::Palette(name) => presets()
            .into_iter()
            .find(|theme| &theme.filename == name)
            .map(|theme| theme.palette)
            .unwrap_or_else(|| resolve(&ThemeChoice::System, system_dark)),
    }
}

pub const RAIL_WIDTH: f32 = 64.0;
pub const HEADER_HEIGHT: f32 = 52.0;
pub const ROW_HEIGHT: f32 = 60.0;
pub const RADIUS: u8 = 8;

pub fn regular(size: f32) -> FontId {
    Weight::Regular.font_id(size)
}

pub fn medium(size: f32) -> FontId {
    Weight::Medium.font_id(size)
}

pub fn semibold(size: f32) -> FontId {
    Weight::SemiBold.font_id(size)
}

pub fn bold(size: f32) -> FontId {
    Weight::Bold.font_id(size)
}

pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// The desktop's text rendering settings, read once: on Linux this asks the
/// desktop portal over D-Bus and may block for up to a second.
fn rendering() -> &'static fastframe_text::TextRendering {
    static RENDERING: std::sync::OnceLock<fastframe_text::TextRendering> =
        std::sync::OnceLock::new();
    RENDERING.get_or_init(fastframe_text::detect)
}

/// Fonts: the platform's interface face (San Francisco, Segoe UI, the
/// desktop's), installed fonts for other scripts, and colour emoji.
pub fn install_fonts(ctx: &egui::Context, reproducible: bool) {
    let mut fonts = fastframe_fonts::FontSetup::default()
        .primary(if reproducible {
            fastframe_fonts::Primary::Inter
        } else {
            fastframe_fonts::Primary::System
        })
        .weights(&[Weight::Medium, Weight::SemiBold, Weight::Bold])
        .system_fallbacks(!reproducible)
        .definitions();
    rendering().apply_to(&mut fonts);
    ctx.set_fonts(fonts);
}

/// Applies a palette to egui's visuals and Wren's text styles.
pub fn apply(ctx: &egui::Context, palette: &Palette) {
    let rendering = rendering();
    ctx.all_styles_mut(|style| {
        let mut visuals = if palette.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = palette.window;
        visuals.window_fill = palette.surface;
        visuals.window_stroke = Stroke::new(1.0, palette.outline);
        visuals.window_corner_radius = CornerRadius::same(12);
        visuals.menu_corner_radius = CornerRadius::same(10);
        visuals.window_shadow = egui::Shadow {
            offset: [0, 8],
            blur: 28,
            spread: 0,
            color: palette.shadow,
        };
        visuals.popup_shadow = egui::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: palette.shadow,
        };
        // Text fields: a shade apart from the cards they sit on.
        visuals.extreme_bg_color = if palette.dark {
            palette.window
        } else {
            palette.panel
        };
        visuals.faint_bg_color = palette.panel;
        visuals.code_bg_color = palette.surface_hover;
        visuals.hyperlink_color = palette.link;
        visuals.selection.bg_fill = palette.accent.gamma_multiply(0.35);
        visuals.selection.stroke = Stroke::new(1.0, palette.accent);
        visuals.override_text_color = Some(palette.text);
        visuals.warn_fg_color = palette.warning;
        visuals.error_fg_color = palette.danger;
        let widgets = &mut visuals.widgets;
        for (state, fill) in [
            (&mut widgets.noninteractive, palette.surface),
            (&mut widgets.inactive, palette.surface_hover),
            (&mut widgets.hovered, palette.surface_active),
            (&mut widgets.active, palette.surface_active),
            (&mut widgets.open, palette.surface_active),
        ] {
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.corner_radius = CornerRadius::same(RADIUS);
            state.fg_stroke = Stroke::new(1.0, palette.text);
        }
        widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.outline);
        widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.secondary);
        widgets.inactive.bg_stroke = Stroke::new(1.0, palette.outline);
        widgets.hovered.bg_stroke = Stroke::new(1.0, palette.outline);
        widgets.active.bg_stroke = Stroke::new(1.0, palette.accent);
        rendering.apply_to_visuals(&mut visuals);
        style.visuals = visuals;

        style.text_styles = [
            (TextStyle::Small, regular(12.0)),
            (TextStyle::Body, regular(14.0)),
            (TextStyle::Button, regular(13.0)),
            (TextStyle::Heading, semibold(18.0)),
            (TextStyle::Monospace, mono(13.0)),
        ]
        .into();
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(10.0, 5.0);
        style.spacing.interact_size.y = 28.0;
        style.spacing.menu_margin = egui::Margin::same(6);
        style.spacing.scroll = egui::style::ScrollStyle::floating();
        style.spacing.scroll.floating_width = 6.0;
        style.interaction.selectable_labels = false;
        style.url_in_tooltip = true;
    });
}

/// An icon at `size`, tinted.
pub fn icon(ui: &mut egui::Ui, icon: Icon, size: f32, tint: Color32) -> egui::Response {
    ui.add(icon.image(tint, size))
}

/// Initials for an avatar: the first letters of the first and last words.
pub fn initials(name: &str) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    let words: Vec<&str> = name
        .split(|c: char| c.is_whitespace() || c == ',' || c == '(')
        .filter(|w| w.chars().next().is_some_and(char::is_alphanumeric))
        .collect();
    let first = |word: &str| {
        word.graphemes(true)
            .next()
            .unwrap_or_default()
            .to_uppercase()
    };
    match words.as_slice() {
        [] => "?".into(),
        [only] => first(only),
        [head, .., tail] => format!("{}{}", first(head), first(tail)),
    }
}

/// A stable colour for a person's avatar, from their id.
pub fn avatar_color(seed: &str, dark: bool) -> Color32 {
    const HUES: [(u8, u8, u8); 10] = [
        (0xd9, 0x7a, 0x34),
        (0x3f, 0x8f, 0xd2),
        (0x8e, 0x6b, 0xd6),
        (0x2e, 0xa6, 0x7f),
        (0xd2, 0x5b, 0x8a),
        (0xc9, 0xa2, 0x27),
        (0x5b, 0x8a, 0x3a),
        (0x4f, 0x6d, 0xd1),
        (0xc0, 0x4f, 0x45),
        (0x2b, 0x9a, 0xb0),
    ];
    let hash = seed.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    let (r, g, b) = HUES[(hash % HUES.len() as u64) as usize];
    let color = Color32::from_rgb(r, g, b);
    if dark {
        color.gamma_multiply(0.85)
    } else {
        color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_from_names() {
        assert_eq!(initials("Ada Lovelace"), "AL");
        assert_eq!(initials("grace"), "G");
        assert_eq!(initials("Katherine G. Johnson"), "KJ");
        assert_eq!(initials(""), "?");
        assert_eq!(initials("Émile Zola"), "ÉZ");
    }

    #[test]
    fn every_shared_palette_resolves() {
        let presets = presets();
        assert!(!presets.is_empty());
        for preset in presets {
            let palette = resolve(&ThemeChoice::Palette(preset.filename.clone()), true);
            assert_eq!(palette, preset.palette);
        }
        assert!(resolve(&ThemeChoice::Palette("missing.json".into()), true).dark);
        assert!(!resolve(&ThemeChoice::System, false).dark);
    }

    #[test]
    fn avatar_colours_are_stable() {
        assert_eq!(avatar_color("u-ada", true), avatar_color("u-ada", true));
    }
}
