//! When Wren may ask Graph for something again.
//!
//! Microsoft's Teams API rules allow fetching a resource whenever the person
//! asks to see it, but not polling: "If your app polls to see whether a
//! resource has changed, you can only do that once per day" and "it isn't
//! okay to GET /me/joinedTeams in a loop every 30 seconds"
//! (learn.microsoft.com/graph/api/resources/teams-api-overview, Polling
//! requirements). Apps that poll violate the Microsoft APIs Terms of Use.
//!
//! So every fetch here answers a [`Trigger`] that a person caused (opening a
//! chat, coming back to the window, pressing Refresh, sending), plus at most
//! one unattended refresh a day. [`Freshness`] also skips a fetch when the
//! same resource was fetched moments ago, so rapid clicking costs nothing.
//! There is deliberately no timer that fetches on its own; real-time
//! updates need Graph change notifications (docs/realtime.md).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::model::ConversationId;

/// Something Graph can be asked for.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Resource {
    Profile,
    Chats,
    Messages(ConversationId),
    Teams,
    Channels(String),
    Calendar,
    /// Presence of the people in view.
    Presence,
}

/// Why a fetch is being considered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// Wren started or the person signed in.
    Start,
    /// The person opened the view that shows the resource.
    Opened,
    /// The window came back to the foreground.
    Focused,
    /// The person asked to refresh.
    Refresh,
    /// The person sent, edited, deleted or reacted to something here.
    AfterAction,
    /// The once-a-day unattended refresh.
    Daily,
    /// A Graph change notification said this resource changed.
    Push,
}

const DAY: Duration = Duration::from_secs(24 * 60 * 60);

impl Resource {
    /// How recent a fetch must be for an `Opened`, `Focused` or `Start`
    /// trigger to skip this one.
    fn reuse_for(&self) -> Duration {
        match self {
            Self::Profile => Duration::from_secs(6 * 60 * 60),
            Self::Chats => Duration::from_secs(15),
            Self::Messages(_) => Duration::from_secs(10),
            Self::Teams => Duration::from_secs(30 * 60),
            Self::Channels(_) => Duration::from_secs(30 * 60),
            Self::Calendar => Duration::from_secs(5 * 60),
            Self::Presence => Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Default)]
pub struct Freshness {
    fetched: HashMap<Resource, Instant>,
}

impl Freshness {
    /// Whether to fetch `resource` now for `trigger`. Says yes at most once
    /// per decision: a yes records the fetch.
    pub fn claim(&mut self, resource: Resource, trigger: Trigger, now: Instant) -> bool {
        let last = self.fetched.get(&resource).copied();
        let age = last.map(|at| now.saturating_duration_since(at));
        let fetch = match trigger {
            // A person's explicit refresh always goes, but a double press
            // within two seconds is one refresh.
            Trigger::Refresh => age.is_none_or(|age| age >= Duration::from_secs(2)),
            // After the person's own change, or when Graph says something
            // changed, look once; a burst within a second is one fetch.
            Trigger::AfterAction | Trigger::Push => {
                age.is_none_or(|age| age >= Duration::from_secs(1))
            }
            Trigger::Start | Trigger::Opened | Trigger::Focused => {
                age.is_none_or(|age| age >= resource.reuse_for())
            }
            Trigger::Daily => age.is_none_or(|age| age >= DAY),
        };
        if fetch {
            self.fetched.insert(resource, now);
        }
        fetch
    }

    /// Forgets a fetch that failed, so the next trigger tries again.
    pub fn forget(&mut self, resource: &Resource) {
        self.fetched.remove(resource);
    }

    pub fn clear(&mut self) {
        self.fetched.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_twice_quickly_fetches_once() {
        let mut freshness = Freshness::default();
        let t0 = Instant::now();
        let chat = Resource::Messages(ConversationId::Chat("a".into()));
        assert!(freshness.claim(chat.clone(), Trigger::Opened, t0));
        assert!(!freshness.claim(chat.clone(), Trigger::Opened, t0 + Duration::from_secs(3)));
        assert!(freshness.claim(chat, Trigger::Opened, t0 + Duration::from_secs(11)));
    }

    #[test]
    fn refresh_goes_but_is_debounced() {
        let mut freshness = Freshness::default();
        let t0 = Instant::now();
        assert!(freshness.claim(Resource::Teams, Trigger::Start, t0));
        assert!(!freshness.claim(
            Resource::Teams,
            Trigger::Focused,
            t0 + Duration::from_secs(60)
        ));
        assert!(freshness.claim(
            Resource::Teams,
            Trigger::Refresh,
            t0 + Duration::from_secs(60)
        ));
        assert!(!freshness.claim(
            Resource::Teams,
            Trigger::Refresh,
            t0 + Duration::from_secs(61)
        ));
    }

    #[test]
    fn unattended_refresh_is_daily_at_most() {
        let mut freshness = Freshness::default();
        let t0 = Instant::now();
        assert!(freshness.claim(Resource::Chats, Trigger::Daily, t0));
        assert!(!freshness.claim(
            Resource::Chats,
            Trigger::Daily,
            t0 + Duration::from_secs(23 * 3600)
        ));
        assert!(freshness.claim(
            Resource::Chats,
            Trigger::Daily,
            t0 + Duration::from_secs(24 * 3600)
        ));
    }

    #[test]
    fn failures_can_be_retried() {
        let mut freshness = Freshness::default();
        let t0 = Instant::now();
        assert!(freshness.claim(Resource::Calendar, Trigger::Opened, t0));
        freshness.forget(&Resource::Calendar);
        assert!(freshness.claim(Resource::Calendar, Trigger::Opened, t0));
    }
}
