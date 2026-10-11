//! Process-wide settings, as JSON in the config folder.
//!
//! Unknown fields are ignored and missing ones take their defaults, so files
//! written by older or newer versions keep loading. Writes go through a
//! temporary file and a rename. Secrets never go here: refresh tokens live in
//! the OS credential store (`crate::secrets`).

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeChoice {
    /// Follow the desktop's light or dark preference.
    #[default]
    System,
    Light,
    Dark,
    /// A shared fastframe palette by file name (`Nord.json`, ...).
    Palette(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The Entra ID application (client) ID Wren signs in with. Each person or
    /// organisation registers their own; see docs/azure-app-registration.md.
    pub client_id: String,
    /// `organizations` (any work or school account), or one tenant's ID or
    /// domain.
    pub tenant: String,
    pub theme: ThemeChoice,
    pub notifications: bool,
    /// Keep syncing in the tray when the window closes.
    pub keep_running: bool,
    /// The interface zoom.
    pub zoom: f32,
    pub sidebar_width: f32,
    /// The account shown at start (kept for settings files from 0.1).
    pub account: Option<String>,
    /// Every signed-in account, in the order they were added.
    pub accounts: Vec<String>,
    /// Keep a presence session while Wren runs, so people see you available
    /// (needs Presence.ReadWrite).
    pub presence_session: bool,
    pub perf_overlay: bool,
    pub window: Option<WindowGeometry>,
    /// A wren-relay for push updates (experimental), `https://…`.
    pub relay_url: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub maximized: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            tenant: "organizations".into(),
            theme: ThemeChoice::System,
            notifications: true,
            // Off by default: without push notifications a hidden Wren
            // cannot learn about new messages (docs/realtime.md).
            keep_running: false,
            zoom: 1.0,
            sidebar_width: 300.0,
            account: None,
            accounts: Vec::new(),
            presence_session: true,
            perf_overlay: false,
            window: None,
            relay_url: None,
        }
    }
}

impl Settings {
    /// Reads the settings, falling back to defaults when the file is missing
    /// or damaged (the damaged copy is kept beside it for inspection).
    pub fn load(path: &Path) -> Self {
        let mut settings = match std::fs::read(path) {
            Ok(bytes) => match serde_json::from_slice::<Self>(&bytes) {
                Ok(settings) => settings,
                Err(error) => {
                    log::warn!("settings unreadable ({error}); starting from defaults");
                    let _ = std::fs::rename(path, path.with_extension("json.damaged"));
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        };
        // Environment overrides, for development and managed installs.
        if let Ok(client_id) = std::env::var("WREN_CLIENT_ID")
            && !client_id.trim().is_empty()
        {
            settings.client_id = client_id.trim().to_owned();
        }
        if let Ok(tenant) = std::env::var("WREN_TENANT")
            && !tenant.trim().is_empty()
        {
            settings.tenant = tenant.trim().to_owned();
        }
        // 0.1 remembered one account; carry it over.
        if settings.accounts.is_empty()
            && let Some(account) = &settings.account
        {
            settings.accounts.push(account.clone());
        }
        settings.zoom = settings.zoom.clamp(0.5, 3.0);
        settings.sidebar_width = settings.sidebar_width.clamp(220.0, 520.0);
        settings
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        crate::paths::write_atomically(path, &json)
    }

    pub fn has_app_registration(&self) -> bool {
        is_guid(self.client_id.trim())
    }
}

/// Whether `value` looks like a GUID (Entra application and tenant IDs).
pub fn is_guid(value: &str) -> bool {
    let groups: Vec<&str> = value.split('-').collect();
    groups.len() == 5
        && groups
            .iter()
            .zip([8, 4, 4, 4, 12])
            .all(|(group, len)| group.len() == len && group.chars().all(|c| c.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_take_defaults_and_unknown_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, r#"{"notifications": false, "from_the_future": 1}"#).unwrap();
        let settings = Settings::load(&path);
        assert!(!settings.notifications);
        assert_eq!(settings.tenant, "organizations");
    }

    #[test]
    fn damaged_files_are_set_aside() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, b"{not json").unwrap();
        let settings = Settings::load(&path);
        assert_eq!(settings, Settings::load(&dir.path().join("absent.json")));
        assert!(path.with_extension("json.damaged").exists());
    }

    #[test]
    fn save_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let settings = Settings {
            client_id: "00000000-0000-0000-0000-000000000000".into(),
            theme: ThemeChoice::Palette("Nord.json".into()),
            ..Default::default()
        };
        settings.save(&path).unwrap();
        // Environment overrides would make this flaky if set in CI.
        if std::env::var("WREN_CLIENT_ID").is_err() && std::env::var("WREN_TENANT").is_err() {
            assert_eq!(Settings::load(&path), settings);
        }
    }

    #[test]
    fn a_single_account_from_0_1_is_carried_over() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, r#"{"account": "o.t"}"#).unwrap();
        assert_eq!(Settings::load(&path).accounts, ["o.t"]);
    }

    #[test]
    fn guids() {
        assert!(is_guid("6731de76-14a6-49ae-97bc-6eba6914391e"));
        assert!(!is_guid("not-a-guid"));
        assert!(!is_guid("6731de76-14a6-49ae-97bc-6eba6914391"));
    }
}
