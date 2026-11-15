# Wren UI review, round 6 (reviewer A)

**Overall: 8.0 / 10.** Polished and calm, with a well-tuned palette. Every text colour sampled passes WCAG AA in both themes. The lowest pair is light-mode DEMO #a9530f on #ececee at 4.53:1, and the next lowest is the error text #c42b2b on #fbeeee at 4.98:1. What holds the score back is a few rough edge cases (long draft, pending images, call bar), focus that isn't shown consistently, and narrow layouts that don't adapt.

| Dimension | Score |
| --- | --- |
| Hierarchy | 8.0 |
| Spacing and alignment | 7.5 |
| Typography | 8.0 |
| Consistency | 7.5 |
| Contrast and accessibility | 8.5 |
| Feedback | 7.5 |
| States | 8.5 |
| Navigation | 8.0 |

## Strongest screens (8.5)
chats, chat-no-selection, group-chat, shortcuts, account-menu, error-toast, first-sync-loading, signin, signin-browser, signin-device-code, signin-expired, signin-failed.

## Weakest screens
- composer-long-draft (6.5): clipped glyph fragments at the top, a stray scrollbar stub, no caret.
- call-bar (7.0): title shown twice, no Leave button or timer, ambiguous button labels.
- calls-narrow (7.0): names truncated by always-visible call buttons.
- composer-images (7.0): centre-cropped thumbnails misrepresent attachments.

## Top issues
1. **Long draft composer** clips text badly and shows a scrollbar stub beside the attach icon. Fix: clip to a 12pt inset with a top fade, and give the scrollbar its own column.
2. **Call bar** duplicates the title, has no Leave button or timer, and uses the vague labels "Restore" and "Close bar". Fix: one merged header with timer and a red Leave button.
3. **Focus is shown inconsistently.** Text fields keep their idle 1pt #8a8d95 border while typing; only the composer gets the 2pt accent ring. Fix: use the accent ring on every input.
4. **Pending image thumbnails** are centre-cropped and the remove buttons cover the corners. Fix: letterbox the images and move the x half outside the tile.
5. **Narrow layouts (760pt)** keep a 232pt list, so names truncate heavily. Fix: 200pt list with 32pt avatars, and hover-only call buttons.
6. **The accent orange is overloaded** (own name, unread, mention, selection, focus, primary, DEMO). Fix: own name in neutral semibold, DEMO as a neutral pill.
7. **Control heights vary**: Calls Join is 32pt beside a 28pt input, and the New chat input is 33pt. Fix: 28pt inline, 32pt in dialog footers.
8. **Primary buttons have a grey 1pt outline** (#8a8d95 / #6e7179) around the accent fill. Fix: border = fill or a darker accent.
9. **Calendar detail is sparse and badly placed**: a left-aligned card, actions far away in the header, no attendees.
10. **Weak surface separation**: list vs main pane is 1.08:1 with no divider, and cards and dividers are 1.33:1.

Lower-priority items cover the search switcher's missing scroll cue and footer divider, heavy Reply buttons in channels, the Settings scrollbar and theme pills, layout shift across sign-in steps, presence glyphs that look alike, and the status dialog's placeholder and missing expiry. Details and exact values are in A.json.
