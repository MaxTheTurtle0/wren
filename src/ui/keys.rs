//! Keyboard shortcuts. ⌘ is Ctrl on Windows and Linux.

use egui::{Key, Modifiers};

use crate::app::{App, Overlay};
use crate::model::{Action, Section};

/// Shortcuts and what they do, for the shortcuts sheet.
pub const SHORTCUTS: &[(&str, &str)] = &[
    ("⌘K", "Search chats, channels and messages"),
    ("⌘1 – ⌘4", "Chat, Teams, Calendar, Calls"),
    ("Alt ↑ / Alt ↓", "Previous or next conversation"),
    ("Enter", "Send"),
    ("Shift Enter", "New line"),
    ("↑ in an empty composer", "Edit your last message"),
    ("Esc", "Cancel reply or edit, close dialogs"),
    ("⌘R", "Refresh what you're looking at"),
    ("⌘,", "Settings"),
    ("⌘ + / ⌘ − / ⌘0", "Zoom in, out, reset"),
    ("⌘⇧P", "Performance overlay"),
    ("⌘/", "This list"),
];

pub fn handle(app: &mut App, ctx: &egui::Context) {
    let mut actions = Vec::new();
    ctx.input_mut(|input| {
        let command = Modifiers::COMMAND;
        if input.consume_key(command, Key::K) || input.consume_key(command, Key::F) {
            actions.push(if app.overlay == Some(Overlay::Switcher) {
                Action::CloseOverlay
            } else {
                Action::OpenSwitcher
            });
        }
        for (key, section) in [
            (Key::Num1, Section::Chats),
            (Key::Num2, Section::Teams),
            (Key::Num3, Section::Calendar),
            (Key::Num4, Section::Calls),
        ] {
            if input.consume_key(command, key) {
                actions.push(Action::Navigate(section));
            }
        }
        if input.consume_key(Modifiers::ALT, Key::ArrowDown) {
            actions.push(Action::Step(1));
        }
        if input.consume_key(Modifiers::ALT, Key::ArrowUp) {
            actions.push(Action::Step(-1));
        }
        if input.consume_key(command, Key::R) {
            actions.push(Action::Refresh);
        }
        if input.consume_key(command, Key::Comma) {
            actions.push(Action::OpenSettings);
        }
        if input.consume_key(command, Key::Slash) {
            actions.push(Action::OpenShortcuts);
        }
        if input.consume_key(command | Modifiers::SHIFT, Key::P) {
            actions.push(Action::TogglePerfOverlay);
        }
        let zoom =
            if input.consume_key(command, Key::Plus) || input.consume_key(command, Key::Equals) {
                Some(app.settings.zoom + 0.1)
            } else if input.consume_key(command, Key::Minus) {
                Some(app.settings.zoom - 0.1)
            } else if input.consume_key(command, Key::Num0) {
                Some(1.0)
            } else {
                None
            };
        if let Some(zoom) = zoom {
            app.settings.zoom = (zoom * 10.0).round().clamp(5.0, 30.0) / 10.0;
            app.mark_settings_dirty();
        }
        if app.overlay.is_some() && input.consume_key(Modifiers::NONE, Key::Escape) {
            actions.push(Action::CloseOverlay);
        }
    });
    // egui's own zoom keys would fight the setting.
    ctx.options_mut(|options| options.zoom_with_keyboard = false);
    app.actions.extend(actions);
}
