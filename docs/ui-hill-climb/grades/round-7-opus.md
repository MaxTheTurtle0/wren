# Wren UI review, round 7 (reviewer A)

**Overall: 8.0 / 10.** A calm, dense and consistent client that is close to shipping quality. I measured text contrast with Pillow and it passes WCAG AA everywhere in both themes. Secondary text is #62656d on white (5.83:1) or #9c9ea8 on #17181b (6.65:1). Accent text is #a9530f (4.95 to 5.34:1) or #e88c3e (6.5 to 7.2:1). Errors are #c42b2b on #fbeeee (4.98:1). No screen is capped by contrast. What holds the score down is layout at the extremes (narrow, wide, very tall content), a few inconsistent components, and some thin states.

| Dimension | Score |
| --- | --- |
| Hierarchy | 8.5 |
| Spacing and alignment | 7.5 |
| Typography | 8.5 |
| Consistency | 7.5 |
| Contrast and accessibility | 8.5 |
| Feedback | 8.5 |
| Empty, loading and error states | 8 |
| Navigation | 8 |

## Strongest screens (8.5)
chats, chat-no-selection, chat-filter-no-match, group-chat, channel, teams, status-message, account-menu, permission-consent, error-toast, join-on-phone and signin-device-code.

## Weakest screens (7 to 7.5)
- **calendar (7):** the detail card is thin and left-aligned, and its header buttons use a different pill style.
- **calls-narrow (7):** the video button disappears at narrow width.
- **composer-images (7):** extreme-aspect thumbnails are cropped into unrecognisable fragments.
- **7.5:** chats-wide, chats-narrow, channel-narrow, calls, composer-long-draft, image-preview, settings, shortcuts, first-sync-empty and signin-finishing.

## Top issues
1. **Primary buttons have a grey stroke around the accent fill.** Light is #8a8d95 around #a9530f; dark is #6e7179 around #e88c3e. The calendar Join pill has no stroke, so primaries are inconsistent. Draw the stroke in the fill colour or a darker accent.
2. **Narrow (760pt) layouts truncate names and collide.** Names cut to "Katherine Jo…" and "Grace Hop…", calls drop the video button, and the hover toolbar overlaps the author row. Use compact dates, keep both call buttons, and clamp the toolbar position.
3. **Wide (1920pt) chat is top-anchored.** There is about 160pt of dead space above the composer. Bottom-align short histories.
4. **Settings clips a row mid-glyph at the bottom.** The background-refresh help text also sits under the tray option rather than notifications.
5. **Calendar detail is thin.** It is a fixed 640pt left-aligned card with redundant Location and Online rows and no attendees. The 24pt pill buttons don't match the 32pt, 6pt-radius buttons used elsewhere.
6. **The sign-in stack is top-anchored and the card height changes per step.** The footer jumps between y=477 and y=630. Centre the group or give the card a minimum height.
7. **Pasted-image thumbnails crop extreme aspect ratios.** Letterbox them instead, inset the × buttons, and show file names.
8. **Calls: the Join button (32pt) and the input (28pt) differ in height.** The explainer copy is also about 15pt, which is off the type scale.
9. **The side pane and main pane are separated only by a 1.08:1 tint.** Add a 1pt #dfdfe3 / #303137 divider.
10. **The shortcuts modal isn't centred.** It has a 90pt top margin and a 13pt bottom margin, and section spacing is uneven (28pt vs 22pt).

There are seven smaller issues in A.json:
- the search switcher footer has no divider
- the filter field disappears during first sync and the empty list is blank
- a long draft has no scroll cue
- the mention popup slices through a message line
- the Retry border on the failure tint is 2.94:1
- the image preview has no actions
- dialog footer patterns are inconsistent
