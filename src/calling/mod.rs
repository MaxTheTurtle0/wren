//! Calls and meetings.
//!
//! Microsoft Graph cannot place or receive a Teams call as the signed-in
//! person: its calling API is application-only and meant for bots. Azure
//! Communication Services can, but bills every participant-minute, while the
//! Teams licence covers calls only in Microsoft's own clients. So the call
//! itself runs in Teams on the web, by default in a chrome-less Edge or
//! Chrome window that Wren opens beside itself ([`browser`], [`dock`]), and
//! Wren follows it as the companion: the call bar, the meeting chat and the
//! person's presence ([`session`]). See docs/call-window.md.

pub mod browser;
pub mod dock;
pub mod session;

use serde::{Deserialize, Serialize};

use browser::{Browser, BrowserKind};

/// Where calls and meetings open (a setting).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallsOpenIn {
    /// Teams on the web in a chrome-less Edge or Chrome window, or a browser
    /// tab when neither is installed.
    #[default]
    AppWindow,
    /// Teams on the web in the default browser.
    Browser,
    /// The Teams desktop app (through the browser's hand-off page).
    TeamsApp,
}

impl CallsOpenIn {
    pub const ALL: [Self; 3] = [Self::AppWindow, Self::Browser, Self::TeamsApp];

    pub fn label(self) -> &'static str {
        match self {
            Self::AppWindow => "Teams on the web, in its own window",
            Self::Browser => "Teams on the web, in my default browser",
            Self::TeamsApp => "The Microsoft Teams app",
        }
    }
}

/// How one call is opened, resolved from the setting and what is installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opener {
    AppWindow(Browser),
    Browser,
    TeamsApp,
}

impl Opener {
    pub fn resolve(
        open_in: CallsOpenIn,
        preferred: Option<BrowserKind>,
        found: &[Browser],
    ) -> Self {
        match open_in {
            CallsOpenIn::AppWindow => browser::pick(found, preferred)
                .cloned()
                .map_or(Self::Browser, Self::AppWindow),
            CallsOpenIn::Browser => Self::Browser,
            CallsOpenIn::TeamsApp => Self::TeamsApp,
        }
    }

    /// Where the call runs, for the call bar.
    pub fn place(&self) -> &'static str {
        match self {
            Self::AppWindow(browser) => browser.kind.short(),
            Self::Browser => "your browser",
            Self::TeamsApp => "Teams",
        }
    }

    /// The browser used for Microsoft sign-in, so Teams on the web shares the
    /// session; `None` means the default browser.
    pub fn sign_in_browser(&self) -> Option<&Browser> {
        match self {
            Self::AppWindow(browser) => Some(browser),
            _ => None,
        }
    }
}

/// Teams on the web's address, for signing in to it ahead of a call.
pub const TEAMS_WEB: &str = "https://teams.microsoft.com/v2/";

/// Hosts that serve Teams meeting and call links.
const TEAMS_HOSTS: &[&str] = &[
    "teams.microsoft.com",
    "teams.live.com",
    "teams.cloud.microsoft",
    "gov.teams.microsoft.us",
    "dod.teams.microsoft.us",
];

/// Whether `url` is an `https` link on a Teams host. Join links come from
/// calendar data anyone can write to, so only these are offered as "Join".
pub fn is_teams_link(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    // No credentials or ports in the authority.
    if host.contains('@') || host.contains(':') {
        return false;
    }
    TEAMS_HOSTS.contains(&host.as_str())
}

/// The documented deep link that starts an audio or video call with one or
/// more people (by UPN or email). Teams asks the person to confirm.
pub fn call_link(users: &[String], video: bool) -> Option<String> {
    let users: Vec<String> = users
        .iter()
        .map(|user| user.trim())
        .filter(|user| !user.is_empty() && user.contains('@'))
        .map(|user| urlencoding::encode(user).into_owned())
        .collect();
    if users.is_empty() {
        return None;
    }
    let mut link = format!(
        "https://teams.microsoft.com/l/call/0/0?users={}",
        users.join(",")
    );
    if video {
        link.push_str("&withVideo=true");
    }
    Some(link)
}

/// One capability row for the Calls page.
pub struct Capability {
    pub name: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

pub const CAPABILITIES: &[Capability] = &[
    Capability {
        name: "Join meetings",
        status: "Works",
        detail: "Opens the meeting in Teams on the web, in its own window beside Wren, with the meeting chat in Wren.",
    },
    Capability {
        name: "Start audio or video calls",
        status: "Works",
        detail: "Opens Microsoft's call link in Teams on the web; Teams asks you to confirm.",
    },
    Capability {
        name: "Call status",
        status: "Works",
        detail: "Wren follows your presence: the call bar shows when you are in a call and closes when it ends.",
    },
    Capability {
        name: "Incoming calls",
        status: "Not possible",
        detail: "Only Microsoft's clients ring. Keep Teams on the web or Teams mobile signed in to be reachable.",
    },
    Capability {
        name: "Audio and video inside Wren's window",
        status: "Not possible",
        detail: "Free only in Microsoft's own clients: the Teams licence does not cover third-party apps, and Azure Communication Services bills per minute.",
    },
];

/// Whether two join links are for the same meeting: the same path, ignoring
/// percent-encoding, case and query strings (Teams adds context there).
pub fn same_meeting(a: &str, b: &str) -> bool {
    fn key(url: &str) -> Option<String> {
        let rest = url.trim().strip_prefix("https://")?;
        let path = rest.split(['?', '#']).next()?;
        let decoded = urlencoding::decode(path).ok()?;
        let key = decoded.trim_end_matches('/').to_ascii_lowercase();
        key.contains('/').then_some(key)
    }
    matches!((key(a), key(b)), (Some(a), Some(b)) if a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_teams_hosts_count() {
        assert!(is_teams_link(
            "https://teams.microsoft.com/l/meetup-join/19%3ameeting%40thread.v2/0?context=%7b%7d"
        ));
        assert!(is_teams_link("https://teams.live.com/meet/9876"));
        assert!(!is_teams_link("http://teams.microsoft.com/l/meetup-join/x"));
        assert!(!is_teams_link(
            "https://teams.microsoft.com.evil.example/l/x"
        ));
        assert!(!is_teams_link("https://user@teams.microsoft.com/l/x"));
        assert!(!is_teams_link(
            "https://evil.example/?u=https://teams.microsoft.com"
        ));
        assert!(!is_teams_link("msteams:/l/meetup-join/x"));
    }

    #[test]
    fn join_links_match_across_encodings_and_context() {
        assert!(same_meeting(
            "https://teams.microsoft.com/l/meetup-join/19%3ameeting_abc%40thread.v2/0?context=%7b%22Tid%22%7d",
            "https://teams.microsoft.com/l/meetup-join/19%3Ameeting_abc%40thread.v2/0"
        ));
        assert!(same_meeting(
            "https://teams.microsoft.com/l/meetup-join/19:meeting_abc@thread.v2/0",
            "https://teams.microsoft.com/l/meetup-join/19%3ameeting_abc%40thread.v2/0/"
        ));
        assert!(!same_meeting(
            "https://teams.microsoft.com/l/meetup-join/19%3ameeting_abc%40thread.v2/0",
            "https://teams.microsoft.com/l/meetup-join/19%3ameeting_xyz%40thread.v2/0"
        ));
        assert!(!same_meeting("", ""));
        assert!(!same_meeting(
            "https://teams.microsoft.com",
            "https://teams.microsoft.com"
        ));
    }

    #[test]
    fn calls_open_in_an_app_window_unless_no_browser_has_one() {
        let chrome = Browser {
            kind: BrowserKind::Chrome,
            program: "/chrome".into(),
        };
        assert_eq!(
            Opener::resolve(CallsOpenIn::AppWindow, None, std::slice::from_ref(&chrome)),
            Opener::AppWindow(chrome.clone())
        );
        assert_eq!(
            Opener::resolve(CallsOpenIn::AppWindow, Some(BrowserKind::Edge), &[]),
            Opener::Browser
        );
        assert_eq!(
            Opener::resolve(CallsOpenIn::TeamsApp, None, &[chrome]),
            Opener::TeamsApp
        );
        assert_eq!(CallsOpenIn::default(), CallsOpenIn::AppWindow);
    }

    #[test]
    fn call_links_encode_users_and_video() {
        assert_eq!(
            call_link(
                &["ada@contoso.com".into(), "grace@contoso.com".into()],
                true
            )
            .as_deref(),
            Some(
                "https://teams.microsoft.com/l/call/0/0?users=ada%40contoso.com,grace%40contoso.com&withVideo=true"
            )
        );
        assert_eq!(call_link(&["not-an-address".into()], false), None);
    }
}
