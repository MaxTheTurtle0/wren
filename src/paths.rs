//! Where Wren keeps its files.
//!
//! | What | Where |
//! | --- | --- |
//! | `settings.json` | the config folder |
//! | `accounts/<id>/wren.db` (cache of chats, messages, teams) | the data folder |
//! | `wren.log`, `panic.log` | the state folder |
//!
//! Every folder is created private to the user (0700 on Unix). Each account
//! has its own database; nothing is shared between accounts.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct AppDirs {
    pub config: PathBuf,
    pub data: PathBuf,
    pub state: PathBuf,
}

impl AppDirs {
    pub fn discover() -> Self {
        match directories::ProjectDirs::from("", "", "Wren") {
            Some(dirs) => Self {
                config: dirs.config_dir().to_path_buf(),
                data: dirs.data_local_dir().to_path_buf(),
                state: dirs
                    .state_dir()
                    .unwrap_or_else(|| dirs.data_local_dir())
                    .to_path_buf(),
            },
            None => Self::under(&std::env::temp_dir().join("wren")),
        }
    }

    /// Everything below one folder: demos and tests.
    pub fn under(root: &Path) -> Self {
        Self {
            config: root.join("config"),
            data: root.join("data"),
            state: root.join("state"),
        }
    }

    pub fn ensure(&self) -> std::io::Result<()> {
        for dir in [&self.config, &self.data, &self.state] {
            create_private_dir(dir)?;
        }
        Ok(())
    }

    pub fn settings_file(&self) -> PathBuf {
        self.config.join("settings.json")
    }

    pub fn log_file(&self) -> PathBuf {
        self.state.join("wren.log")
    }

    pub fn panic_log(&self) -> PathBuf {
        self.state.join("panic.log")
    }

    pub fn account(&self, account: &str) -> PathBuf {
        self.data.join("accounts").join(safe_name(account))
    }

    pub fn database(&self, account: &str) -> PathBuf {
        self.account(account).join("wren.db")
    }
}

/// An account id as a folder name: anything but ASCII letters, digits, `-`
/// and `.` becomes `_`.
pub fn safe_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    match cleaned.trim_matches('.') {
        "" => "_".into(),
        trimmed => trimmed.to_owned(),
    }
}

pub fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Writes through a temporary file in the same folder and renames it over
/// the target, so a crash never leaves a half-written file.
pub fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let temporary = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    {
        let mut file = std::fs::File::create(&temporary)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    std::fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_names_cannot_escape_their_folder() {
        assert_eq!(safe_name("../../etc"), "_.._etc");
        assert_eq!(safe_name("a1b2.c3d4-e5"), "a1b2.c3d4-e5");
        assert_eq!(safe_name(".."), "_");
        let dirs = AppDirs::under(Path::new("/x"));
        assert!(dirs.database("../evil").starts_with("/x/data/accounts"));
    }

    #[test]
    fn atomic_writes_replace_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.json");
        write_atomically(&path, b"one").unwrap();
        write_atomically(&path, b"two").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
