//! Desktop notifications: the notification centre on macOS, toasts on
//! Windows, the desktop's notification daemon on Linux.
//!
//! Shown from a short-lived thread, since the platform calls can block.
//! Notification text is never logged.

pub fn show(title: &str, body: &str) {
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
