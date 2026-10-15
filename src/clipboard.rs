//! Clipboard images stay in memory until sent as Graph hosted content.

use std::io::Cursor;
use std::sync::Arc;

/// Leave room for base64 inside Graph's 4 MiB hosted-content request.
pub const MAX_BYTES: usize = 3 * 1024 * 1024 - 64 * 1024;
const MAX_PIXELS: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlineImage {
    pub png: Arc<[u8]>,
    pub width: u32,
    pub height: u32,
    uri: String,
}

impl InlineImage {
    pub fn uri(&self) -> String {
        self.uri.clone()
    }
}

/// Called on a blocking backend task, only in response to Paste.
pub fn read_image() -> Result<Option<InlineImage>, &'static str> {
    let mut clipboard = arboard::Clipboard::new().map_err(|_| "The clipboard is unavailable.")?;
    match clipboard.get_image() {
        Ok(image) => encode_rgba(image.width, image.height, &image.bytes).map(Some),
        Err(arboard::Error::ContentNotAvailable) => Ok(None),
        Err(_) => Err("The clipboard image could not be read. Copy it again and retry."),
    }
}

pub fn encode_rgba(width: usize, height: usize, rgba: &[u8]) -> Result<InlineImage, &'static str> {
    let pixels = width
        .checked_mul(height)
        .filter(|n| *n > 0 && *n <= MAX_PIXELS)
        .ok_or("The image is too large. Attach it as a file instead.")?;
    if rgba.len() != pixels * 4 {
        return Err("The clipboard image is incomplete. Copy it again.");
    }
    let image = image::RgbaImage::from_raw(width as u32, height as u32, rgba.to_vec())
        .ok_or("The clipboard image could not be decoded.")?;
    let mut png = Cursor::new(Vec::new());
    image
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|_| "The clipboard image could not be encoded.")?;
    let png = png.into_inner();
    if png.len() > MAX_BYTES {
        return Err("The image is too large. Attach it as a file instead.");
    }
    use sha2::{Digest, Sha256};
    let hex: String = Sha256::digest(&png)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(InlineImage {
        png: png.into(),
        width: width as u32,
        height: height as u32,
        uri: format!("bytes://pasted-{hex}.png"),
    })
}

/// egui-winit consumes Cmd/Ctrl+V even when no text exists. Its key release
/// still arrives. Turn that into a paste request, without duplicating a
/// text paste already reported on key down.
#[derive(Default)]
pub struct PasteInput {
    reported: bool,
    command: bool,
}

impl PasteInput {
    pub fn prepare(&mut self, input: &mut egui::RawInput) {
        let pasted = input
            .events
            .iter()
            .any(|e| matches!(e, egui::Event::Paste(_)));
        let mut image_paste = false;
        for event in &input.events {
            match event {
                egui::Event::ModifiersChanged(modifiers) => {
                    if self.command != modifiers.command {
                        // Menu pastes have no matching V release. A new
                        // shortcut must not inherit their reported state.
                        self.reported = false;
                    }
                    self.command = modifiers.command;
                }
                egui::Event::Paste(_) => self.reported = true,
                egui::Event::Key {
                    key: egui::Key::V,
                    pressed: false,
                    modifiers,
                    ..
                } if modifiers.command => {
                    image_paste |= !self.reported && !pasted;
                    self.reported = false;
                }
                egui::Event::WindowFocused(false) => *self = Self::default(),
                _ => {}
            }
        }
        if image_paste {
            input.events.push(egui::Event::Paste(String::new()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_is_encoded_as_a_real_png_and_invalid_sizes_are_rejected() {
        let image = encode_rgba(1, 1, &[10, 20, 30, 255]).unwrap();
        let decoded = image::load_from_memory(&image.png).unwrap().to_rgba8();
        assert_eq!(decoded.as_raw(), &[10, 20, 30, 255]);
        assert!(encode_rgba(usize::MAX, 2, &[]).is_err());
        assert!(encode_rgba(1, 1, &[]).is_err());
    }

    #[test]
    fn image_only_paste_is_reported_and_text_paste_is_not_duplicated() {
        let release = egui::Event::Key {
            key: egui::Key::V,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        };
        let mut bridge = PasteInput::default();
        let mut input = egui::RawInput {
            events: vec![release.clone()],
            ..Default::default()
        };
        bridge.prepare(&mut input);
        assert!(matches!(input.events.last(), Some(egui::Event::Paste(text)) if text.is_empty()));
        let mut down = egui::RawInput {
            events: vec![
                egui::Event::ModifiersChanged(egui::Modifiers::COMMAND),
                egui::Event::Paste("text".into()),
            ],
            ..Default::default()
        };
        bridge.prepare(&mut down);
        let mut up = egui::RawInput {
            events: vec![release],
            ..Default::default()
        };
        bridge.prepare(&mut up);
        assert_eq!(up.events.len(), 1);
    }

    #[test]
    fn menu_paste_does_not_suppress_the_next_image_only_shortcut() {
        let mut bridge = PasteInput::default();
        let mut menu = egui::RawInput {
            events: vec![egui::Event::Paste("text".into())],
            ..Default::default()
        };
        bridge.prepare(&mut menu);
        let release = egui::Event::Key {
            key: egui::Key::V,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        };
        let mut up = egui::RawInput {
            events: vec![
                egui::Event::ModifiersChanged(egui::Modifiers::COMMAND),
                release.clone(),
            ],
            ..Default::default()
        };
        bridge.prepare(&mut up);
        assert!(matches!(up.events.last(), Some(egui::Event::Paste(text)) if text.is_empty()));

        let mut same_frame = egui::RawInput {
            events: vec![egui::Event::Paste("text".into()), release],
            ..Default::default()
        };
        bridge.prepare(&mut same_frame);
        assert_eq!(same_frame.events.len(), 2);
    }
}
