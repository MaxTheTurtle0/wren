//! Calls and meetings.
//!
//! Microsoft Graph cannot place or receive a Teams call as the signed-in
//! person: its calling API is application-only and meant for bots. The one
//! documented way to call as a Teams user is Azure Communication Services
//! (ACS) "Teams users", whose Calling SDK exists for the web, Windows
//! (WinRT, MSIX-packaged), iOS and Android, with no native macOS or Linux
//! SDK. See docs/calling.md for the full investigation.
//!
//! So Wren hands calls and meetings off, through Microsoft's documented deep
//! links, to the Teams desktop app when it is installed or Teams on the web
//! otherwise. [`CallBackend`] is the seam where an ACS-based backend can
//! plug in later without touching the interface.

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

/// What Wren can do with calls on this platform, for the interface to say
/// honestly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallBackend {
    /// Open Microsoft's deep links in the Teams app or Teams on the web.
    HandOff,
    /// ACS Calling SDK as a Teams user. Not implemented; see docs/calling.md.
    AzureCommunicationServices,
}

impl CallBackend {
    pub fn current() -> Self {
        Self::HandOff
    }

    pub fn describe(self) -> &'static str {
        match self {
            Self::HandOff => {
                "Calls and meetings open in Microsoft Teams (the app if installed, otherwise Teams on the web). Microsoft does not offer a public API for calling as yourself from a third-party desktop app."
            }
            Self::AzureCommunicationServices => {
                "Calls run inside Wren through Azure Communication Services."
            }
        }
    }
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
        status: "Hand-off",
        detail: "Opens the meeting's join link in Teams.",
    },
    Capability {
        name: "Start audio or video calls",
        status: "Hand-off",
        detail: "Opens Microsoft's call link; Teams asks you to confirm.",
    },
    Capability {
        name: "Incoming call alerts",
        status: "Not possible yet",
        detail: "No public API reports incoming Teams calls to a third-party client. Needs ACS (see docs/calling.md).",
    },
    Capability {
        name: "Audio, video and screen sharing in Wren",
        status: "Planned (ACS)",
        detail: "Requires an Azure Communication Services resource and its Calling SDK, which has no macOS or Linux native build.",
    },
];

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
