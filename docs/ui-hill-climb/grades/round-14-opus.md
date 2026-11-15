# Wren UI review, round 14 (reviewer A)

**Overall: 8.0 / 10**

| Dimension | Score |
|---|---|
| Contrast | 8.0 |
| Typography | 8.0 |
| Colour | 8.0 |
| Spacing | 7.5 |
| Hierarchy | 8.5 |

## Summary
This is a calm, consistent app built on a tight token set. Text tokens are #1b1c1f, #4a4d55 and #62656d (light) and #e9e9ec, #b4b6bf and #9c9ea8 (dark). The accent is #b4530a (light) and #e88c3e (dark). Every text sample measured passes WCAG AA. The lowest are the mention name on its tint at 4.52:1 (light) and secondary text on hover or selected rows at about 6:1. Primary buttons measure 5.02:1 (white on #b4530a) and 7.24:1 (#1c1208 on #e88c3e). Inputs, outlined buttons and checkboxes are 3.6 to 8:1. What keeps the app below 9 is polish: non-text edges, parity between the themes, stray scrollbars and spacing rhythm.

## What blocks a 9
1. Component boundaries fall below 3:1. Channel cards and the floating message toolbar measure 1.90:1 (light) and 2.11:1 (dark). Keycaps measure 1.33:1 and 1.15:1.
2. The selected nav-rail tile is invisible in light mode (1.05:1); dark mode shows it at 1.59:1.
3. Scrollbars are stray or heavy: a full-height bar on setup, an always-solid history bar, and a loose stub inside the composer.
4. Composer thumbnails of very wide or tall images are unreadable strips, and their tiles blend into the composer (1.13:1).
5. Spacing rhythm is uneven: section heading gaps in shortcuts (28px vs 14px), settings checkbox rows, the setup card title (8px), and empty states placed at different heights.
6. In calendar detail, the header actions are far from the content column and only 4px apart.
7. Link underlines sit 7px below the baseline.
8. The search switcher's last row fades to 4.17:1 and runs into the footer.
9. Dark-mode danger red (#f58a8a) reads as pink.

## Screen scores
8.5: chats, chats-wide, chat-no-selection, chat-filter-no-match, group-chat, teams, composer-mention, new-chat, status-message, account-menu, permission-consent, error-toast, first-sync-loading, join-on-phone, signin, signin-browser, signin-finishing, signin-expired, signin-failed

8.0: chats-narrow, channel, calls, calls-narrow, image-preview, search-switcher, settings, shortcuts, call-bar, call-docked, signin-device-code

7.5: channel-narrow, calendar, composer-long-draft, first-sync-empty, setup

7.0: composer-images

grade.json has the full issue list with measured evidence and fixes.
