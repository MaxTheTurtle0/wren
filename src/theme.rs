//! Colours, type, spacing and icons.
//!
//! [`Palette`] holds every colour the interface uses, in a dark and a light
//! variant. Wren has no other themes: [`ThemeChoice`] picks light, dark, or
//! the desktop's preference.

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Vec2};
use fastframe_fonts::Weight;

use crate::settings::ThemeChoice;

mod lazy_fonts;

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
        EyeOff => lucide "eye-off",
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// Hairlines between areas.
    pub outline: Color32,
    /// The edge of a card (a channel post), firmer than a hairline.
    pub card_edge: Color32,
    /// The edge of controls a person acts on (text fields, checkboxes,
    /// secondary buttons): at least 3:1 against what they sit on.
    pub control: Color32,
    pub text: Color32,
    /// Supporting text. At least 4.5:1 on every surface, selected rows too.
    pub secondary: Color32,
    /// Quiet text (times, hints). Still at least 4.5:1 on every surface.
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
            // A ladder with visible steps: rail, list, main pane, raised.
            window: Color32::from_rgb(0x1a, 0x1b, 0x1f),
            panel: Color32::from_rgb(0x14, 0x15, 0x18),
            rail: Color32::from_rgb(0x0c, 0x0d, 0x0f),
            surface: Color32::from_rgb(0x26, 0x27, 0x2c),
            surface_hover: Color32::from_rgb(0x2c, 0x2d, 0x33),
            surface_active: Color32::from_rgb(0x34, 0x35, 0x3c),
            outline: Color32::from_rgb(0x30, 0x31, 0x37),
            card_edge: Color32::from_rgb(0x4d, 0x4f, 0x58),
            control: Color32::from_rgb(0x82, 0x85, 0x8e),
            text: Color32::from_rgb(0xe9, 0xe9, 0xec),
            secondary: Color32::from_rgb(0xb4, 0xb6, 0xbf),
            dim: Color32::from_rgb(0x9c, 0x9e, 0xa8),
            accent: Color32::from_rgb(0xe8, 0x8c, 0x3e),
            accent_hover: Color32::from_rgb(0xf2, 0x9e, 0x55),
            on_accent: Color32::from_rgb(0x1c, 0x12, 0x08),
            danger: Color32::from_rgb(0xf5, 0x8a, 0x8a),
            warning: Color32::from_rgb(0xe3, 0xb3, 0x41),
            success: Color32::from_rgb(0x4c, 0xc3, 0x6e),
            overlay: Color32::from_rgba_premultiplied(0, 0, 0, 150),
            shadow: Color32::from_rgba_premultiplied(0, 0, 0, 110),
            highlight: Color32::from_rgb(0x2b, 0x22, 0x19),
            link: Color32::from_rgb(0x7c, 0xb4, 0xf5),
        }
    }

    pub fn light() -> Self {
        Self {
            dark: false,
            window: Color32::from_rgb(0xff, 0xff, 0xff),
            panel: Color32::from_rgb(0xf3, 0xf4, 0xf6),
            rail: Color32::from_rgb(0xe7, 0xe8, 0xeb),
            surface: Color32::from_rgb(0xff, 0xff, 0xff),
            // Hover, then selected/pressed, a clear step apart.
            surface_hover: Color32::from_rgb(0xf0, 0xf0, 0xf2),
            surface_active: Color32::from_rgb(0xe2, 0xe3, 0xe8),
            outline: Color32::from_rgb(0xdf, 0xdf, 0xe3),
            card_edge: Color32::from_rgb(0xb9, 0xbc, 0xc4),
            control: Color32::from_rgb(0x7c, 0x7f, 0x87),
            text: Color32::from_rgb(0x1b, 0x1c, 0x1f),
            secondary: Color32::from_rgb(0x4a, 0x4d, 0x55),
            dim: Color32::from_rgb(0x62, 0x65, 0x6d),
            // A clear orange (not brown) that still holds white labels at 5:1.
            accent: Color32::from_rgb(0xb4, 0x53, 0x0a),
            accent_hover: Color32::from_rgb(0x96, 0x45, 0x08),
            on_accent: Color32::from_rgb(0xff, 0xff, 0xff),
            danger: Color32::from_rgb(0xb4, 0x25, 0x25),
            // Gold, clear of the orange accent in hue.
            warning: Color32::from_rgb(0x8f, 0x6a, 0x00),
            success: Color32::from_rgb(0x1b, 0x7a, 0x42),
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

/// The palette a choice resolves to; `System` follows the desktop.
pub fn resolve(choice: ThemeChoice, system_dark: bool) -> Palette {
    let dark = match choice {
        ThemeChoice::System => system_dark,
        ThemeChoice::Light => false,
        ThemeChoice::Dark => true,
    };
    if dark {
        Palette::dark()
    } else {
        Palette::light()
    }
}

pub const RAIL_WIDTH: f32 = 64.0;
pub const HEADER_HEIGHT: f32 = 52.0;
pub const ROW_HEIGHT: f32 = 60.0;
pub const RADIUS: u8 = 8;

/// The line height for running text at `size`: about 1.45, on whole points.
pub fn leading(size: f32) -> f32 {
    (size * 1.45).round()
}

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
    ctx.set_fonts(font_definitions(reproducible, false));
    if !reproducible {
        ctx.add_plugin(lazy_fonts::LazyFonts::default());
    }
}

pub fn font_revision(ctx: &egui::Context) -> u64 {
    ctx.data(|d| d.get_temp(egui::Id::new("wren-font-revision")).unwrap_or(0))
}

fn font_definitions(reproducible: bool, fallbacks: bool) -> egui::FontDefinitions {
    let mut fonts = fastframe_fonts::FontSetup::default()
        .primary(if reproducible {
            fastframe_fonts::Primary::Inter
        } else {
            fastframe_fonts::Primary::System
        })
        .weights(&[Weight::Medium, Weight::SemiBold, Weight::Bold])
        .system_fallbacks(fallbacks)
        .definitions();
    rendering().apply_to(&mut fonts);
    fonts
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
        // Also the focus ring of text fields.
        visuals.selection.stroke = Stroke::new(2.0, palette.accent);
        visuals.override_text_color = Some(palette.text);
        // Placeholders and other weak text keep 4.5:1.
        visuals.weak_text_color = Some(palette.dim);
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
        // Fields, checkboxes and buttons keep a visible edge (WCAG 1.4.11).
        widgets.inactive.bg_stroke = Stroke::new(1.0, palette.control);
        widgets.hovered.bg_stroke = Stroke::new(1.0, palette.secondary);
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
        // One button padding everywhere; labels carry no padding spaces.
        style.spacing.button_padding = Vec2::new(14.0, 5.0);
        style.spacing.interact_size.y = 28.0;
        // Menus and tooltips: 10 pt at the sides, 6 pt above and below.
        style.spacing.menu_margin = egui::Margin::symmetric(10, 6);
        style.spacing.scroll = egui::style::ScrollStyle::floating();
        style.spacing.scroll.floating_width = 6.0;
        style.interaction.selectable_labels = false;
        // Tooltips wrap at a readable measure instead of running wide.
        style.spacing.tooltip_width = 280.0;
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
    // Each keeps white initials at 4.5:1 or more.
    const HUES: [(u8, u8, u8); 10] = [
        (0xb4, 0x58, 0x1c),
        (0x2a, 0x73, 0xb5),
        (0x7a, 0x5c, 0xc4),
        (0x1f, 0x7a, 0x5c),
        (0xc2, 0x41, 0x6f),
        (0x8f, 0x6d, 0x0f),
        (0x4a, 0x7a, 0x2e),
        (0x45, 0x60, 0xc4),
        (0xb0, 0x44, 0x3a),
        (0x1e, 0x7c, 0x8e),
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
    fn choices_resolve_to_light_or_dark() {
        assert_eq!(resolve(ThemeChoice::System, true), Palette::dark());
        assert_eq!(resolve(ThemeChoice::System, false), Palette::light());
        for system_dark in [true, false] {
            assert_eq!(resolve(ThemeChoice::Light, system_dark), Palette::light());
            assert_eq!(resolve(ThemeChoice::Dark, system_dark), Palette::dark());
        }
        assert_eq!(Palette::light().window, Color32::WHITE);
    }

    #[test]
    fn avatar_colours_are_stable() {
        assert_eq!(avatar_color("u-ada", true), avatar_color("u-ada", true));
    }

    /// WCAG 2 contrast ratio between two opaque colours.
    fn contrast(a: Color32, b: Color32) -> f32 {
        fn luminance(c: Color32) -> f32 {
            let channel = |v: u8| {
                let v = f32::from(v) / 255.0;
                if v <= 0.039_28 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
        }
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn text_and_controls_meet_wcag_contrast() {
        for palette in [Palette::light(), Palette::dark()] {
            let surfaces = [
                palette.window,
                palette.panel,
                palette.surface,
                palette.surface_hover,
                palette.surface_active,
            ];
            for surface in surfaces {
                // Control edges keep 3:1 on every surface they sit on.
                assert!(contrast(palette.control, surface) >= 3.0, "{surface:?}");
                // Errors show on selected rows too ("Not sent" in the list).
                assert!(contrast(palette.danger, surface) >= 4.5, "{surface:?}");
                for text in [palette.text, palette.secondary, palette.dim] {
                    assert!(
                        contrast(text, surface) >= 4.5,
                        "{text:?} on {surface:?} in {}",
                        if palette.dark { "dark" } else { "light" }
                    );
                }
            }
            for surface in [palette.window, palette.panel, palette.surface] {
                for text in [
                    palette.accent,
                    palette.danger,
                    palette.success,
                    palette.link,
                ] {
                    assert!(contrast(text, surface) >= 4.5, "{text:?} on {surface:?}");
                }
                assert!(contrast(palette.control, surface) >= 3.0);
            }
            assert!(contrast(palette.on_accent, palette.accent) >= 4.5);
            assert!(contrast(palette.accent, palette.highlight) >= 4.5);
            for seed in 0..64 {
                let color = avatar_color(&format!("seed-{seed}"), palette.dark);
                assert!(contrast(Color32::WHITE, color) >= 4.5, "{color:?}");
            }
        }
    }
}
