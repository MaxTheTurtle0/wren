# Wren UI review, round 9 (reviewer A)

**Overall: 8.0 / 10**

| Dimension | Score |
| --- | --- |
| Contrast | 8.5 |
| Typography | 8 |
| Colour | 8 |
| Spacing | 7.5 |
| Hierarchy | 8 |

## Summary

Wren looks calm and consistent, with good light/dark parity. Every real-text sample passes WCAG AA. Secondary text is #62656d on #ffffff (5.83:1) and #9c9ea8 on #17181b (6.65:1). Body text is 17.0:1 and 14.7:1. Interactive borders are #7c7f87 / #82858e (3.7-4.8:1). The lowest margins are on light-mode accent text: DEMO is 4.53:1, unread times 4.95:1 and the mention name 4.81:1. These pass, but with little headroom.

What keeps it below 9 is consistency and polish rather than any one broken screen.

## What blocks a 9

1. Dialogs do not share a frame. The shortcuts close button sits about 100 px in from the right edge, and title insets range from 21 to 24 px.
2. Scrolled content is cut off under headers: half avatars, clipped card borders, and a faded 2.1:1 text line.
3. The Calendar and Calls detail panes are sparse and off the grid. Their headers start at x 462 instead of 394. On Calls, the 28 px input sits next to a 32 px button.
4. At 760 px the sidebars stay full width, so names truncate ("Grace Hop...", "Katherine Jo...").
5. Leading (17, 20 or 23 px) and control heights vary between the sign-in cards, setup and other forms.
6. The orange accent does too many jobs: your own name, mentions, unread markers, selection, primary buttons and focus. The calendar adds a competing blue.
7. Message-row hover is barely visible (1.07-1.08:1).
8. Smaller polish slips: thread card edges are staggered by 2 px, the sign-in tooltip overflows the card, the setup page shows a scrollbar, and image preview wastes space.

## Screen scores

| Screen | Score | Screen | Score |
| --- | --- | --- | --- |
| chats | 8.5 | composer-images | 7.5 |
| chats-wide | 8 | image-preview | 7.5 |
| chats-narrow | 7.5 | search-switcher | 8.5 |
| chat-no-selection | 8.5 | new-chat | 8.5 |
| chat-filter-no-match | 8 | settings | 8 |
| group-chat | 8.5 | shortcuts | 7 |
| channel | 8 | status-message | 8.5 |
| channel-narrow | 7 | account-menu | 8 |
| teams | 8 | permission-consent | 8.5 |
| calendar | 7 | error-toast | 8 |
| calls | 7.5 | first-sync-loading | 8.5 |
| calls-narrow | 7.5 | first-sync-empty | 8 |
| composer-mention | 8.5 | call-bar | 7.5 |
| composer-long-draft | 8 | call-docked | 8.5 |
| join-on-phone | 8 | setup | 7.5 |
| signin | 8 | signin-browser | 8.5 |
| signin-device-code | 8.5 | signin-finishing | 8 |
| signin-expired | 8 | signin-failed | 8.5 |

`grade.json` has the per-screen issues and 17 prioritised fixes, with measured evidence for each.
