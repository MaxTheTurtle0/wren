# Wren UI review, round 15 (reviewer A)

**Overall: 8.0 / 10**

| Dimension | Score |
| --- | --- |
| Contrast | 9 |
| Typography | 8 |
| Colour | 8 |
| Spacing | 7.5 |
| Hierarchy | 8 |

## Summary
The app is calm, dense and coherent. The colour tokens are disciplined, every measured text sample passes WCAG AA in both themes, and dark mode matches light mode closely. What keeps it below 9 is finish: small alignment and rhythm errors, one accent doing too much, sparse wide layouts, and a few rough states (the bottom edge of Settings, the blank first-run sidebar, strip-shaped image thumbnails).

## Contrast (measured with Pillow)
- Ink ladder, light: #1b1c1f / #4a4d55 / #62656d on #ffffff gives 17.0 / 8.45 / 5.83. On the sidebar #f3f4f6 the weakest is 5.30.
- Ink ladder, dark: #e9e9ec / #b4b6bf / #9c9ea8 on #1a1b1f gives 15.1 / 8.51 / 6.45. On the raised surface #26272c the weakest is 5.59.
- Primary buttons: white on #b4530a is 5.02; #1c1208 on #e88c3e is 7.24.
- Links: #1f63c6 is 5.74 (light); #7cb4f5 is 7.95 (dark).
- Error text: #b42525 on #f9eeee is 5.72; #f58a8a on #2c2428 is 6.39.
- Input and button borders: #7c7f87 is 3.64 to 4.01 (light); #82858e is 4.04 to 4.95 (dark).
- Borderline: mention text #b4530a on #fdf1e5 is 4.52, and the unread dot on #f3f4f6 is 4.56.
- No text fails, so no screen is capped.

## What blocks a 9
1. Spacing and alignment are inconsistent:
   - The search switcher close X sits 9 px low.
   - The shortcuts section gaps are 41 px vs 28 px.
   - Empty-state heights differ (y316 vs y340).
   - The space around system lines varies between 15 px and 30 px.
   - Group chat has an empty band of about 70 px above the first date divider.
2. The orange accent is overloaded: it marks unread, selection, focus, mentions, quotes, checkboxes, send and primary actions. It also looks rust-brown in light mode and tangerine in dark mode.
3. On wide windows, Calendar and Calls are capped at about 720 px. Their header actions end at x1114 and the detail panes look empty.
4. The bottom scroll shadow in Settings renders as a hard grey band over clipped content.
5. Narrow and edge states are rough:
   - In calls-narrow, names truncate into the call icons.
   - Thumbnails of extreme-aspect images are unreadable.
   - The first-run sidebar is completely blank.
6. Text texture: Inter's wide hyphen shows in strings like "On-call", and line height jumps in paragraphs with an inline link.

## Screen scores
| Screen | Score | Screen | Score |
| --- | --- | --- | --- |
| chats | 8.5 | settings | 7.5 |
| chats-wide | 8.5 | shortcuts | 8 |
| chats-narrow | 8 | status-message | 8.5 |
| chat-no-selection | 8.5 | account-menu | 8 |
| chat-filter-no-match | 8.5 | permission-consent | 8.5 |
| group-chat | 8 | error-toast | 8.5 |
| channel | 8 | first-sync-loading | 8.5 |
| channel-narrow | 8 | first-sync-empty | 7.5 |
| teams | 8.5 | call-bar | 8 |
| calendar | 7.5 | call-docked | 8 |
| calls | 8 | join-on-phone | 8.5 |
| calls-narrow | 7.5 | setup | 8 |
| composer-mention | 8 | signin | 8.5 |
| composer-long-draft | 8 | signin-browser | 8.5 |
| composer-images | 7.5 | signin-device-code | 8.5 |
| image-preview | 8 | signin-finishing | 8.5 |
| search-switcher | 7.5 | signin-expired | 8.5 |
| new-chat | 8 | signin-failed | 8.5 |

`grade.json` has 16 prioritised issues, each with evidence and a fix.
