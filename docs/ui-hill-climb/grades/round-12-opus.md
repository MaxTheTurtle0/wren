# Wren UI review, round 12 (reviewer A)

**Overall: 8 / 10**. Calm, consistent and readable. All real text passes WCAG. What remains is polish: how edges are handled, alignment, and a few components styled differently from the rest.

| Dimension | Score |
| --- | --- |
| Contrast | 9 |
| Typography | 8 |
| Colour | 8 |
| Spacing | 7.5 |
| Hierarchy | 8 |

## Contrast (measured with Pillow)
- Body text #1b1c1f on #fff is 17.0:1 (light) and #e9e9ec on #17181b is 12.5–15:1 (dark).
- The secondary tier #62656d / #9c9ea8 ranges from 5.4:1 to 6.9:1.
- The tertiary tier #4a4d55 / #b4b6bf ranges from 6.7:1 to 9.1:1.
- Accent text #a5500f on #f6f6f7 is 5.18:1. Primary buttons are 5.59:1 (light) and 7.24:1 (dark).
- The link is 5.74:1 (light) and 8.20:1 (dark).
- Avatar initials range from 4.82:1 to 6.8:1.
- Input and outline-button borders range from 3.71:1 to 4.99:1.
- Disabled buttons are about 4.6:1.
- No real text fails.
- Weak spots are not WCAG failures:
  - Light hover and selected tints are about 1.16:1.
  - In dark mode, rail, list and main pane differ by about 1.04:1.
  - Light card borders are 1.52:1.

## What blocks a 9
1. Scroll areas hard-clip content under headers and at dialog bottoms, with no fade. This shows in chats, channel, the composer draft and settings.
2. Containers are inconsistent:
   - Channel cards have ragged right edges (x = 1251, 1253 and 1255).
   - The mention popup touches the composer.
   - The settings scrollbar sits 11 px from "Leave demo".
3. Thin secondary screens:
   - The calendar detail is sparse and sits to the left.
   - The calendar list uses three different left edges.
   - The meeting chat has a large empty band at the top.
4. Components that don't match the rest:
   - The composer scrollbar is a solid bar in the text colour.
   - The filter field shows no focus ring.
   - Links have two styles.
   - The sign-in tooltip overflows its card.
5. Surface tints are too close together, so states and regions blur.
6. Inter's wide hyphen splits "On-call", "Sign-in" and the device code.
7. The setup screen starts 18 px from the top. The other auth screens start at about 120 px.

## Screens
- **8.5:** chats-wide, chat-no-selection, group-chat, teams, new-chat, status-message, account-menu, permission-consent, first-sync-loading, first-sync-empty, join-on-phone, signin-browser, signin-finishing
- **8:** chats, chat-filter-no-match, calls, composer-images, image-preview, search-switcher, shortcuts, error-toast, call-docked, signin, signin-device-code, signin-expired, signin-failed
- **7.5:** chats-narrow, channel, channel-narrow, calendar, calls-narrow, composer-mention, settings, call-bar, setup
- **7:** composer-long-draft

`grade.json` has per-screen issues and 17 prioritised fixes with measurements.
