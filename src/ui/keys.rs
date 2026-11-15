//! Keyboard shortcuts. ⌘ is Ctrl on Windows and Linux.

use egui::{Key, Modifiers};

use crate::app::{App, Overlay};
use crate::model::{Action, Section};

/// Shortcuts and what they do, in groups, for the shortcuts sheet.
pub const SHORTCUTS: &[(&str, &[(&str, &str)])] = &[
    (
        "Navigation",
        &[
            ("⌘K", "Search chats, channels and messages"),
            ("⌘1 – ⌘4", "Chat, Teams, Calendar, Calls"),
            ("Alt+↑ / Alt+↓", "Previous or next conversation"),
            ("⌘⇧A", "Next account"),
        ],
    ),
    (
        "Messages",
        &[
            ("⌘N", "New chat"),
            ("Enter", "Send"),
            ("Shift+Enter", "New line"),
            ("@", "Mention someone"),
            ("↑", "Edit your last message (empty composer)"),
            ("Esc", "Cancel reply or edit, close dialogs"),
        ],
    ),
    (
        "App",
        &[
            ("⌘R", "Refresh what you're looking at"),
            ("⌘,", "Settings"),
            ("⌘ + / ⌘ − / ⌘0", "Zoom in, out, reset"),
            ("⌘⇧P", "Performance overlay"),
            ("⌘/", "This list"),
        ],
    ),
];

/// A shortcut as written in [`SHORTCUTS`] (⌘ for the command key), in the
/// platform's notation: unchanged on macOS, `Ctrl+Shift+A` elsewhere.
pub fn shortcut(keys: &str) -> std::borrow::Cow<'_, str> {
    if cfg!(target_os = "macos") || !keys.contains(['⌘', '⇧']) {
        return keys.into();
    }
    keys.replace("⌘⇧", "Ctrl+Shift+")
        .replace('⌘', "Ctrl+")
        .replace('⇧', "Shift+")
        .replace("Ctrl+ ", "Ctrl+")
        .into()
}

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
        if input.consume_key(command, Key::N) {
            actions.push(Action::OpenNewChat);
        }
        if input.consume_key(command | Modifiers::SHIFT, Key::A) && app.accounts.len() > 1 {
            actions.push(Action::SwitchAccount((app.active + 1) % app.accounts.len()));
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

#[cfg(test)]
mod tests {
    use super::shortcut;

    #[test]
    fn shortcuts_use_the_platform_notation() {
        if cfg!(target_os = "macos") {
            assert_eq!(shortcut("⌘⇧A"), "⌘⇧A");
        } else {
            assert_eq!(shortcut("⌘⇧A"), "Ctrl+Shift+A");
            assert_eq!(shortcut("⌘K"), "Ctrl+K");
            assert_eq!(shortcut("⌘ + / ⌘ − / ⌘0"), "Ctrl++ / Ctrl+− / Ctrl+0");
        }
        assert_eq!(shortcut("Shift Enter"), "Shift Enter");
    }
}
