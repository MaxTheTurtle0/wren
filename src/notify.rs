//! Desktop notifications: the notification centre on macOS, toasts on
//! Windows, the desktop's notification daemon on Linux.
//!
//! Shown from a short-lived thread, since the platform calls can block.
//! Notification text is never logged.

pub fn show(title: &str, body: &str) {
    #[cfg(target_os = "macos")]
    claim_identity();
    let title = title.to_owned();
    let body = crate::html::truncate(body, 240);
    let _ = std::thread::Builder::new()
        .name("wren-notify".into())
        .spawn(move || {
            let mut notification = notify_rust::Notification::new();
            notification.summary(&title).body(&body).appname("Wren");
            #[cfg(target_os = "linux")]
            notification.icon("wren");
            if let Err(error) = notification.show() {
                log::info!("could not show a notification: {error}");
            }
        });
}

/// macOS attributes a notification to an app by bundle identifier. Inside
/// Wren.app, post as Wren; a bare binary (development) keeps the library's
/// default, since an identifier without a bundle shows nothing.
#[cfg(target_os = "macos")]
fn claim_identity() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let bundled = std::env::current_exe()
            .map(|path| path.to_string_lossy().contains(".app/Contents/MacOS/"))
            .unwrap_or(false);
        if bundled && let Err(error) = notify_rust::set_application("app.wren.Wren") {
            log::info!("notifications keep the default sender: {error}");
        }
    });
}
