//! Finding Microsoft Edge and Google Chrome, and the command lines that open
//! Teams on the web in one of their chrome-less app windows.
//!
//! Microsoft supports Teams on the web in Edge, Chrome, Firefox and Safari;
//! only Edge and Chrome have app windows (`--app`), so only they are used
//! here. Browsers are started directly with an argument list, never through a
//! shell. If the browser is already running, the new process hands the
//! request to it and exits, so the window opens in the person's normal
//! profile, where their Microsoft sign-in lives.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::is_teams_link;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrowserKind {
    Edge,
    Chrome,
}

impl BrowserKind {
    pub const ALL: [Self; 2] = [Self::Edge, Self::Chrome];

    pub fn name(self) -> &'static str {
        match self {
            Self::Edge => "Microsoft Edge",
            Self::Chrome => "Google Chrome",
        }
    }

    /// The short name for buttons and the call bar.
    pub fn short(self) -> &'static str {
        match self {
            Self::Edge => "Edge",
            Self::Chrome => "Chrome",
        }
    }
}

/// An installed browser and how to start it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Browser {
    pub kind: BrowserKind,
    pub program: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    MacOs,
    Windows,
    Linux,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

/// What discovery may look at, so tests can fake it.
pub trait Probe {
    fn is_file(&self, path: &Path) -> bool;
    fn var(&self, name: &str) -> Option<String>;
}

struct System;

impl Probe for System {
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok().filter(|value| !value.is_empty())
    }
}

/// The supported browsers on this computer, Edge first.
pub fn discover() -> Vec<Browser> {
    let found = discover_with(&System, Platform::current());
    log::info!(
        "call browsers found: {}",
        if found.is_empty() {
            "none".to_owned()
        } else {
            found
                .iter()
                .map(|b| b.kind.short())
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
    found
}

pub fn discover_with(probe: &dyn Probe, platform: Platform) -> Vec<Browser> {
    let mut found = Vec::new();
    for kind in BrowserKind::ALL {
        if let Some(program) = candidates(probe, platform, kind)
            .into_iter()
            .find(|path| probe.is_file(path))
        {
            found.push(Browser { kind, program });
        }
    }
    found
}

fn candidates(probe: &dyn Probe, platform: Platform, kind: BrowserKind) -> Vec<PathBuf> {
    match platform {
        Platform::MacOs => {
            let (bundle, binary) = match kind {
                BrowserKind::Edge => ("Microsoft Edge.app", "Microsoft Edge"),
                BrowserKind::Chrome => ("Google Chrome.app", "Google Chrome"),
            };
            let mut roots = vec![PathBuf::from("/Applications")];
            if let Some(home) = probe.var("HOME") {
                roots.push(Path::new(&home).join("Applications"));
            }
            roots
                .into_iter()
                .map(|root| root.join(bundle).join("Contents/MacOS").join(binary))
                .collect()
        }
        Platform::Windows => {
            let tail = match kind {
                BrowserKind::Edge => ["Microsoft", "Edge", "Application", "msedge.exe"],
                BrowserKind::Chrome => ["Google", "Chrome", "Application", "chrome.exe"],
            };
            ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"]
                .into_iter()
                .filter_map(|var| probe.var(var))
                .map(|root| {
                    tail.iter()
                        .fold(PathBuf::from(root), |path, part| path.join(part))
                })
                .collect()
        }
        Platform::Linux => {
            let (names, flatpak): (&[&str], &str) = match kind {
                BrowserKind::Edge => (
                    &["microsoft-edge-stable", "microsoft-edge"],
                    "com.microsoft.Edge",
                ),
                BrowserKind::Chrome => (
                    &["google-chrome-stable", "google-chrome"],
                    "com.google.Chrome",
                ),
            };
            let mut paths = Vec::new();
            for dir in probe
                .var("PATH")
                .unwrap_or_default()
                .split(':')
                .filter(|dir| dir.starts_with('/'))
            {
                for name in names {
                    paths.push(Path::new(dir).join(name));
                }
            }
            paths.push(Path::new("/var/lib/flatpak/exports/bin").join(flatpak));
            if let Some(home) = probe.var("HOME") {
                paths.push(
                    Path::new(&home)
                        .join(".local/share/flatpak/exports/bin")
                        .join(flatpak),
                );
            }
            paths
        }
    }
}

/// The browser to use: the preferred one if installed, else the first found.
pub fn pick(found: &[Browser], preferred: Option<BrowserKind>) -> Option<&Browser> {
    preferred
        .and_then(|kind| found.iter().find(|b| b.kind == kind))
        .or_else(|| found.first())
}

/// A browser and the profile folder to use in it (`None`: the default).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserProfile {
    pub browser: Browser,
    pub profile: Option<String>,
}

/// Where the browser should put a new window, in screen points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// A process to start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    pub program: PathBuf,
    pub args: Vec<String>,
}

/// The browser profile folder for an account (`--profile-directory`), or the
/// person's default profile.
fn profile_args(profile: Option<&str>) -> Vec<String> {
    profile
        .filter(|p| !p.is_empty() && !p.contains(['/', '\\', '\0']))
        .map(|p| vec![format!("--profile-directory={p}")])
        .unwrap_or_default()
}

/// Teams on the web in a chrome-less app window. Only Teams links qualify, so
/// the URL can never be read as a flag or point anywhere else.
pub fn app_window(
    browser: &Browser,
    url: &str,
    placement: Option<Placement>,
    profile: Option<&str>,
) -> Option<Launch> {
    if !is_teams_link(url) {
        return None;
    }
    let mut args = profile_args(profile);
    if let Some(p) = placement {
        args.push(format!("--window-position={},{}", p.x, p.y));
        args.push(format!(
            "--window-size={},{}",
            p.width.max(320),
            p.height.max(240)
        ));
    }
    args.push(format!("--app={url}"));
    Some(Launch {
        program: browser.program.clone(),
        args,
    })
}

/// A normal browser window or tab, for Microsoft's sign-in and sign-out pages.
pub fn page(browser: &Browser, url: &str, profile: Option<&str>) -> Option<Launch> {
    let rest = url.strip_prefix("https://")?;
    if rest.is_empty() || url.chars().any(char::is_whitespace) {
        return None;
    }
    let mut args = profile_args(profile);
    args.push(url.to_owned());
    Some(Launch {
        program: browser.program.clone(),
        args,
    })
}

/// Starts the process detached from Wren: no shell, no inherited input or
/// output, its own process group (so Ctrl-C in a terminal running Wren does
/// not close the browser), reaped on a short-lived thread.
pub fn spawn(launch: &Launch) -> std::io::Result<()> {
    let mut command = std::process::Command::new(&launch.program);
    command
        .args(&launch.args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let _ = std::thread::Builder::new()
        .name("wren-browser".into())
        .spawn(move || {
            let _ = child.wait();
        });
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// Paths compare by component, so `/` and `\` match on Windows as they
    /// do for the real file system.
    #[derive(Default)]
    struct Fake {
        files: Vec<PathBuf>,
        vars: HashMap<&'static str, String>,
    }

    impl Probe for Fake {
        fn is_file(&self, path: &Path) -> bool {
            self.files.iter().any(|file| file == path)
        }
        fn var(&self, name: &str) -> Option<String> {
            self.vars.get(name).cloned()
        }
    }

    fn fake(files: &[&str], vars: &[(&'static str, &str)]) -> Fake {
        Fake {
            files: files.iter().map(PathBuf::from).collect(),
            vars: vars.iter().map(|(k, v)| (*k, (*v).to_owned())).collect(),
        }
    }

    #[test]
    fn finds_chrome_in_the_mac_applications_folders() {
        let probe = fake(
            &["/Users/ada/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"],
            &[("HOME", "/Users/ada")],
        );
        let found = discover_with(&probe, Platform::MacOs);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, BrowserKind::Chrome);
        let probe = fake(
            &[
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            ],
            &[],
        );
        let kinds: Vec<_> = discover_with(&probe, Platform::MacOs)
            .iter()
            .map(|b| b.kind)
            .collect();
        assert_eq!(kinds, [BrowserKind::Edge, BrowserKind::Chrome]);
    }

    #[test]
    fn finds_edge_on_windows_and_flatpak_chrome_on_linux() {
        let probe = fake(
            &[r"C:\Program Files (x86)/Microsoft/Edge/Application/msedge.exe"],
            &[("ProgramFiles(x86)", r"C:\Program Files (x86)")],
        );
        let found = discover_with(&probe, Platform::Windows);
        assert_eq!(found[0].kind, BrowserKind::Edge);

        let probe = fake(
            &["/home/ada/.local/share/flatpak/exports/bin/com.google.Chrome"],
            &[("HOME", "/home/ada"), ("PATH", "/usr/bin:relative:/bin")],
        );
        let found = discover_with(&probe, Platform::Linux);
        assert_eq!(found[0].kind, BrowserKind::Chrome);
        let probe = fake(
            &["/usr/bin/microsoft-edge"],
            &[("PATH", "/usr/local/bin:/usr/bin")],
        );
        assert_eq!(
            discover_with(&probe, Platform::Linux)[0].program,
            PathBuf::from("/usr/bin/microsoft-edge")
        );
        // Relative PATH entries are never searched.
        let probe = fake(&["relative/google-chrome"], &[("PATH", "relative")]);
        assert!(discover_with(&probe, Platform::Linux).is_empty());
    }

    #[test]
    fn the_preferred_browser_wins_when_installed() {
        let found = vec![
            Browser {
                kind: BrowserKind::Edge,
                program: "/e".into(),
            },
            Browser {
                kind: BrowserKind::Chrome,
                program: "/c".into(),
            },
        ];
        assert_eq!(
            pick(&found, Some(BrowserKind::Chrome)).unwrap().kind,
            BrowserKind::Chrome
        );
        assert_eq!(pick(&found, None).unwrap().kind, BrowserKind::Edge);
        assert_eq!(
            pick(&found[..1], Some(BrowserKind::Chrome)).unwrap().kind,
            BrowserKind::Edge
        );
        assert!(pick(&[], None).is_none());
    }

    #[test]
    fn app_windows_open_only_teams_links() {
        let chrome = Browser {
            kind: BrowserKind::Chrome,
            program: "/chrome".into(),
        };
        let launch = app_window(
            &chrome,
            "https://teams.microsoft.com/l/meetup-join/19%3ameeting_x%40thread.v2/0",
            Some(Placement {
                x: 10,
                y: 20,
                width: 900,
                height: 700,
            }),
            Some("Wren 1a2b"),
        )
        .unwrap();
        assert_eq!(
            launch.args,
            [
                "--profile-directory=Wren 1a2b",
                "--window-position=10,20",
                "--window-size=900,700",
                "--app=https://teams.microsoft.com/l/meetup-join/19%3ameeting_x%40thread.v2/0",
            ]
        );
        assert!(app_window(&chrome, "https://evil.example/teams", None, None).is_none());
        assert!(app_window(&chrome, "--disable-web-security", None, None).is_none());
        // A profile name cannot reach outside the browser's profile folder.
        let launch = app_window(
            &chrome,
            "https://teams.microsoft.com/v2/",
            None,
            Some("../x"),
        )
        .unwrap();
        assert_eq!(launch.args, ["--app=https://teams.microsoft.com/v2/"]);
    }

    #[test]
    fn pages_must_be_https() {
        let edge = Browser {
            kind: BrowserKind::Edge,
            program: "/edge".into(),
        };
        let launch = page(
            &edge,
            "https://login.microsoftonline.com/x/oauth2/v2.0/authorize?a=b",
            None,
        )
        .unwrap();
        assert_eq!(launch.args.len(), 1);
        assert!(page(&edge, "http://login.microsoftonline.com", None).is_none());
        assert!(page(&edge, "--flag", None).is_none());
        assert!(page(&edge, "https://", None).is_none());
    }
}
