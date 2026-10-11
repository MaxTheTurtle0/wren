//! Writes Wren's icon from packaging/icons/wren.svg:
//!
//! ```sh
//! cargo run --example render-icon -- out.png 1024   # one PNG
//! cargo run --example render-icon -- wren.ico       # Windows icon, 16 to 256 px
//! ```
//!
//! Packaging uses it for the macOS .icns, the Windows .ico and the Linux
//! hicolor icons, so every platform's icon comes from the one SVG.

fn png(size: usize) -> Vec<u8> {
    let pixels = wren::platform::icon_rgba(size);
    let image = image::RgbaImage::from_raw(size as u32, size as u32, pixels).expect("square RGBA");
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("encode PNG");
    bytes
}

/// An ICO file whose entries are PNG images (supported since Windows Vista).
fn ico(sizes: &[usize]) -> Vec<u8> {
    let images: Vec<Vec<u8>> = sizes.iter().map(|&size| png(size)).collect();
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]);
    out.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len();
    for (&size, image) in sizes.iter().zip(&images) {
        let side = if size >= 256 { 0 } else { size as u8 };
        out.extend_from_slice(&[side, side, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&(image.len() as u32).to_le_bytes());
        out.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += image.len();
    }
    for image in images {
        out.extend_from_slice(&image);
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or_else(|| "wren.png".into());
    let bytes = if path.ends_with(".ico") {
        ico(&[16, 24, 32, 48, 64, 128, 256])
    } else {
        png(args.next().and_then(|s| s.parse().ok()).unwrap_or(1024))
    };
    std::fs::write(&path, bytes).expect("write the icon");
}
