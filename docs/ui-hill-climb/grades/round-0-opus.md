# Wren UI review, round 0 (reviewer A)

**Overall: 6 / 10** (mean of the 36 screen scores: 5.9)

| Dimension | Score |
| --- | --- |
| Hierarchy | 7 |
| Spacing and alignment | 6 |
| Typography | 6.5 |
| Consistency | 5.5 |
| Contrast and accessibility | 4.5 |
| Feedback | 6 |
| Empty, loading and error states | 5.5 |
| Navigation and wayfinding | 6.5 |

## Summary

The core chat layout is calm and well structured. It has a clear three-column hierarchy, a good type scale, sensible date dividers, quotes and reactions, and matching light and dark themes. It is let down by three things:

- **Systematic contrast failures.** I measured pixels with Pillow:
  - The tertiary text colour is 3.27:1 in light and 3.02:1 on dark dialogs, and as low as 2.44:1 on hover rows.
  - The light accent `#c26318` is 4.12:1 as text and as white-label buttons.
  - Input, composer and checkbox borders are 1.15 to 1.37:1.

  Every screen that uses these colours is capped at 7.
- **Broken or fragile layouts at edge sizes and states.** The Calls page clips at 760pt, the image attachment strip is broken, Settings clips "Leave demo" under its scrollbar, the hover toolbar covers author lines, and the sidebar jumps from 300 to 360pt when it is empty.
- **Inconsistency across overlays.** Scrims range from none to faint to heavy, dialog titles are 15, 18 or 20pt, shortcuts say Ctrl in one place and ⌘ in another, and Calendar and Calls lack the header bar used by Chat.

## Lowest-scoring screens

- calls-narrow (3.5): content clipped horizontally, Join button missing
- composer-images (3.5): thumbnails mis-cropped, remove buttons detached
- settings (4.5): clipped button, invisible checkboxes, labels after controls
- chats-narrow, first-sync-loading (5): no responsive sidebar, toolbar overlap, no skeleton

## Best screens

join-on-phone (7), and chats, calendar, sign-in and permission-consent (6.5). These are clean, but each still fails the contrast checks.

## Top issues (full detail and fixes in A.json)

1. Tertiary text `#8c8e96` / `#6d6f78` fails 4.5:1 (3.27, 3.03, 2.58, 2.44:1). Use `#686b73` in light and `#989aa3` in dark.
2. Light accent `#c26318` fails as text and button fill (4.12:1). Use `#a9530f` (5.34:1), and `#1b7a42` for success green.
3. calls-narrow clips content. Reflow to the available width and stack the table.
4. composer-images thumbnails are broken. Use uniform 72pt tiles with attached remove buttons.
5. Settings: "Leave demo" is clipped, checkboxes are 1.32:1 / 1.15:1, and labels sit after their controls.
6. Component borders are 1.33:1 and focused fields have no focus ring. Use `#8a8d95` / `#62656d` borders and a 2pt accent ring.
7. The hover toolbar covers author lines when narrow and is detached when wide. Cap the message column at 760pt and anchor the toolbar to the row.
8. Sidebar width jumps from 300 to 360pt and is not responsive.
9. Light-mode modal scrims are missing or faint and inconsistent.
10. The error toast has no Retry and the failed message is missing from the timeline.
11. Weak empty and loading states: no skeleton, contradictory "Pick a chat" copy, no call to action.
12. White avatar initials fail contrast (about 3.05:1 on green and orange).
13. The dark mention highlight is a heavy brown band, and its pill is 3.36:1.
14. Shortcut notation mixes Ctrl and ⌘.
15. The account menu has ragged alignment, and presence states are colour-only and identical in pairs.
16. The composer clips its first line, has no scrollbar and no focus ring.
17. Hyphens render with wide gaps ("On - call", "Sign - in").
18. Calendar and Calls have no 52pt header bar, so the title position jumps.
19. Light image preview has an invisible canvas, and "100%" does not look like a control.
20. Calls copy: "a Edge", a docs-style table, and contradictory meeting state.
