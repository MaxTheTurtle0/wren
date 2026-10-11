//! A call or meeting that Wren opened, or noticed from presence, and what
//! the call bar says about it.
//!
//! Wren cannot see inside Teams on the web. It learns that a call is running
//! from the person's own presence activity, read when the window gains focus
//! and, with a relay, when Graph says the presence changed. Presence can
//! flicker, so a call ends only after it has looked over for a minute. That
//! minute is a deadline checked when the window repaints, never a timer
//! that fetches.

use crate::model::{ConversationId, Millis};

/// How long presence must stay out of the call before the call counts as
/// over.
pub const GRACE: Millis = 60_000;
/// How long "Call ended" stays on the bar.
pub const ENDED_SHOWN: Millis = 4_000;
const MINUTE: Millis = 60_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallTarget {
    Meeting {
        subject: String,
        join_url: String,
    },
    People {
        names: String,
        video: bool,
    },
    /// A call Wren did not open, noticed from presence.
    Elsewhere,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallState {
    /// Opened; presence does not show the call yet.
    Opening,
    Live {
        since: Millis,
    },
    /// Presence left the call at `left`. The call ends after [`GRACE`]
    /// unless presence comes back.
    Leaving {
        since: Millis,
        left: Millis,
    },
    Ended {
        at: Millis,
    },
}

/// Finding the meeting's chat, to show beside the call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatLookup {
    Found,
    Searching,
    /// Looking it up needs OnlineMeetings.Read, not granted yet.
    NeedsPermission,
    NotFound,
    /// A call, not a meeting: the chat is known or there is none.
    NotApplicable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallSession {
    pub target: CallTarget,
    pub opened_at: Millis,
    pub state: CallState,
    /// The account that opened it (`oid.tid`).
    pub account: Option<String>,
    /// Where the call runs, for the bar: "Chrome", "your browser", "Teams".
    pub place: String,
    pub chat: Option<ConversationId>,
    pub lookup: ChatLookup,
}

/// Whether a presence activity means the person is in a call or meeting.
/// `strict` leaves out `InAMeeting`, which a calendar entry alone sets.
pub fn in_call(activity: Option<&str>, strict: bool) -> bool {
    let Some(activity) = activity else {
        return false;
    };
    let activity = activity.to_ascii_lowercase();
    matches!(
        activity.as_str(),
        "inacall" | "inaconferencecall" | "presenting"
    ) || (!strict && activity == "inameeting")
}

impl CallSession {
    pub fn new(target: CallTarget, now: Millis, account: Option<String>, place: &str) -> Self {
        let lookup = match target {
            CallTarget::Meeting { .. } => ChatLookup::Searching,
            _ => ChatLookup::NotApplicable,
        };
        Self {
            target,
            opened_at: now,
            state: CallState::Opening,
            account,
            place: place.to_owned(),
            chat: None,
            lookup,
        }
    }

    /// A call noticed from presence that Wren did not open.
    pub fn elsewhere(now: Millis, account: Option<String>) -> Self {
        Self {
            state: CallState::Live { since: now },
            ..Self::new(CallTarget::Elsewhere, now, account, "Teams")
        }
    }

    /// Applies a reading of the person's presence activity.
    pub fn observe(&mut self, activity: Option<&str>, now: Millis) {
        let strict = self.target == CallTarget::Elsewhere;
        let here = in_call(activity, strict);
        self.state = match (self.state, here) {
            (CallState::Opening, true) => CallState::Live { since: now },
            (CallState::Live { since }, false) => CallState::Leaving { since, left: now },
            (CallState::Leaving { since, .. }, true) => CallState::Live { since },
            (state, _) => state,
        };
    }

    /// Moves past deadlines. Returns how long until the bar next changes,
    /// so the window can repaint then (and not before).
    pub fn advance(&mut self, now: Millis) -> Option<Millis> {
        match self.state {
            CallState::Leaving { left, .. } if now - left >= GRACE => {
                self.state = CallState::Ended { at: now };
                Some(ENDED_SHOWN)
            }
            CallState::Leaving { since, left } => {
                Some((GRACE - (now - left)).min(MINUTE - (now - since).rem_euclid(MINUTE)))
            }
            CallState::Live { since } => Some(MINUTE - (now - since).rem_euclid(MINUTE)),
            CallState::Ended { at } => Some((at + ENDED_SHOWN - now).max(0)),
            CallState::Opening => None,
        }
    }

    pub fn is_ended(&self) -> bool {
        matches!(self.state, CallState::Ended { .. })
    }

    /// Whether "Call ended" has been shown long enough to remove the bar.
    pub fn is_finished(&self, now: Millis) -> bool {
        matches!(self.state, CallState::Ended { at } if now - at >= ENDED_SHOWN)
    }

    pub fn is_meeting(&self) -> bool {
        matches!(self.target, CallTarget::Meeting { .. })
    }

    pub fn join_url(&self) -> Option<&str> {
        match &self.target {
            CallTarget::Meeting { join_url, .. } => Some(join_url),
            _ => None,
        }
    }

    pub fn title(&self) -> String {
        match &self.target {
            CallTarget::Meeting { subject, .. } if !subject.trim().is_empty() => subject.clone(),
            CallTarget::Meeting { .. } => "Meeting".into(),
            CallTarget::People { names, video } if names.is_empty() => {
                if *video { "Video call" } else { "Call" }.into()
            }
            CallTarget::People { names, .. } => names.clone(),
            CallTarget::Elsewhere => "You're in a call".into(),
        }
    }

    /// The bar's status line.
    pub fn status(&self, now: Millis) -> String {
        let what = if self.is_meeting() {
            "In the meeting"
        } else {
            "In the call"
        };
        match self.state {
            CallState::Opening => format!("Opened in {}", self.place),
            CallState::Live { since } | CallState::Leaving { since, .. } => {
                let minutes = (now - since).max(0) / MINUTE;
                if minutes == 0 {
                    format!("{what} · just joined")
                } else {
                    format!("{what} · {minutes} min")
                }
            }
            CallState::Ended { .. } => "Call ended".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meeting() -> CallSession {
        CallSession::new(
            CallTarget::Meeting {
                subject: "Design review".into(),
                join_url: "https://teams.microsoft.com/l/meetup-join/x".into(),
            },
            0,
            None,
            "Chrome",
        )
    }

    #[test]
    fn presence_moves_a_meeting_from_opening_to_live_to_ended() {
        let mut call = meeting();
        assert_eq!(call.status(0), "Opened in Chrome");
        assert_eq!(call.advance(0), None);
        call.observe(Some("Available"), 10_000);
        assert_eq!(call.state, CallState::Opening);
        call.observe(Some("InAMeeting"), 20_000);
        assert_eq!(call.state, CallState::Live { since: 20_000 });
        assert_eq!(
            call.status(20_000 + 3 * MINUTE + 5),
            "In the meeting · 3 min"
        );
        // Repaint when the minute counter changes.
        assert_eq!(call.advance(20_000 + 30_000), Some(30_000));

        call.observe(Some("Available"), 500_000);
        assert!(matches!(
            call.state,
            CallState::Leaving { left: 500_000, .. }
        ));
        call.advance(500_000 + GRACE - 1);
        assert!(!call.is_ended());
        assert_eq!(call.advance(500_000 + GRACE), Some(ENDED_SHOWN));
        assert!(call.is_ended());
        assert_eq!(call.status(0), "Call ended");
        assert!(!call.is_finished(500_000 + GRACE + 1));
        assert!(call.is_finished(500_000 + GRACE + ENDED_SHOWN));
    }

    #[test]
    fn a_short_flicker_does_not_end_the_call() {
        let mut call = meeting();
        call.observe(Some("InACall"), 1_000);
        call.observe(Some("Available"), 50_000);
        call.observe(Some("InACall"), 70_000);
        assert_eq!(call.state, CallState::Live { since: 1_000 });
        call.advance(70_000 + GRACE * 2);
        assert!(!call.is_ended());
    }

    #[test]
    fn calendar_meetings_alone_are_not_calls_elsewhere() {
        assert!(in_call(Some("InAMeeting"), false));
        assert!(!in_call(Some("InAMeeting"), true));
        assert!(in_call(Some("inaconferencecall"), true));
        assert!(!in_call(None, false));
        let mut elsewhere = CallSession::elsewhere(0, None);
        assert_eq!(elsewhere.title(), "You're in a call");
        elsewhere.observe(Some("InAMeeting"), 10_000);
        assert!(matches!(elsewhere.state, CallState::Leaving { .. }));
    }

    #[test]
    fn titles_name_the_meeting_or_the_people() {
        assert_eq!(meeting().title(), "Design review");
        let call = CallSession::new(
            CallTarget::People {
                names: "Ada Lovelace".into(),
                video: true,
            },
            0,
            None,
            "Edge",
        );
        assert_eq!(call.title(), "Ada Lovelace");
        assert_eq!(call.lookup, ChatLookup::NotApplicable);
        assert_eq!(meeting().lookup, ChatLookup::Searching);
    }
}
