//! Desktop integration: the app icon, the tray item, one running copy.

use std::sync::mpsc;

use crate::backend::Waker;

const ICON_SVG: &[u8] = include_bytes!("../packaging/icons/wren.svg");
const TRAY_SVG: &[u8] = include_bytes!("../packaging/icons/tray-template.svg");

/// Rasterizes an SVG into square RGBA pixels.
fn rasterize(svg: &[u8], size: usize) -> Vec<u8> {
    let options = resvg::usvg::Options::default();
    let Ok(tree) = resvg::usvg::Tree::from_data(svg, &options) else {
        return vec![0; size * size * 4];
    };
    let Some(mut pixmap) = resvg::tiny_skia::Pixmap::new(size as u32, size as u32) else {
        return vec![0; size * size * 4];
    };
    let view = tree.size();
    let scale = size as f32 / view.width().max(view.height());
    let dx = (size as f32 - view.width() * scale) / 2.0;
    let dy = (size as f32 - view.height() * scale) / 2.0;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, dx, dy),
        &mut pixmap.as_mut(),
    );
    // tiny-skia stores premultiplied alpha; icons want straight alpha.
    pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect()
}

pub fn icon_rgba(size: usize) -> Vec<u8> {
    rasterize(ICON_SVG, size)
}

fn tray_template_rgba(size: usize) -> Vec<u8> {
    rasterize(TRAY_SVG, size)
}

pub fn window_icon() -> egui::IconData {
    let size = 256;
    egui::IconData {
        rgba: icon_rgba(size),
        width: size as u32,
        height: size as u32,
    }
}

/// The tray item: StatusNotifier on Linux, the notification area on
/// Windows, the menu bar on macOS.
pub fn tray(waker: &Waker) -> Option<fastframe_tray::Tray> {
    use fastframe_tray::{Config, MenuItem, Tray};
    let waker = waker.clone();
    Tray::spawn(
        Config {
            id: "wren",
            title: "Wren".into(),
            icon: icon_rgba,
            template_icon: Some(tray_template_rgba),
            themed_icon: true,
            menu_on_click: false,
            menu: vec![
                MenuItem::action("show", "Open Wren"),
                MenuItem::Separator,
                MenuItem::action("quit", "Quit Wren"),
            ],
        },
        move || waker.wake(),
    )
}

/// One running copy per user. A second launch asks the first to show its
/// window and exits. Returns the guard to keep, and the requests the first
/// copy receives; `None` when another copy took the request.
pub fn single_instance(
    waker: &Waker,
) -> Option<(Option<fastframe_instance::Guard>, mpsc::Receiver<String>)> {
    use fastframe_instance::{Claim, Slot};
    let (sender, receiver) = mpsc::channel();
    let waker = waker.clone();
    let slot = Slot::new("app.wren.Wren");
    match slot.claim("show", move |request| {
        let _ = sender.send(request.to_owned());
        waker.wake();
        Some("ok".to_owned())
    }) {
        Claim::First(guard) => Some((Some(guard), receiver)),
        Claim::Running(_) | Claim::Declined | Claim::Unanswered => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn icons_rasterize_with_opaque_pixels() {
        let pixels = super::icon_rgba(64);
        assert_eq!(pixels.len(), 64 * 64 * 4);
        assert!(pixels.chunks(4).any(|p| p[3] == 255));
        let tray = super::tray_template_rgba(32);
        assert!(tray.chunks(4).any(|p| p[3] > 0));
    }
}
