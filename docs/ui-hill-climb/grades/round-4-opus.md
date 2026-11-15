# Wren UI review, round 4 (reviewer A)

**Overall: 7.5 / 10** (mean of the 36 screen scores: 7.6)

| Dimension | Score |
| --- | --- |
| Visual hierarchy | 8 |
| Spacing and alignment | 7 |
| Typography | 7 |
| Consistency | 6.5 |
| Contrast and accessibility | 8 |
| Feedback | 8 |
| Empty, loading and error states | 8 |
| Navigation and wayfinding | 7.5 |

## Summary

Wren is calm, dense and readable in both themes. Text contrast passes everywhere I measured. Muted text is #62656d on white (5.83:1) or #f6f6f7 (5.40:1), and #9c9ea8 on #17181b (6.65:1). The accent orange is #a9530f with white (5.34:1), and #e88c3e with #1c1208 in dark mode (7.24:1). No screen is capped by a text-contrast failure.

What holds the app back is polish and consistency, not structure:

- **Buttons have no single system.** Filled primaries render at about 15pt and outlined secondaries at 13pt. Secondary buttons come in three styles: outlined, filled pill and faint outline.
- **The hover toolbar overlaps neighbouring rows** in group chats, narrow chats and narrow channels.
- **The shortcuts dialog's description column is misaligned** (x=596, 601 and 621 in its three sections).
- **Hyphens render with extra spacing everywhere** ("On - call", "Sign - in", "F4KQ - 9TZW").
- **Interactive outlines fall below 3:1.** The hover toolbar measures 1.23:1, reaction chips 1.33:1 and Reply buttons 1.16 to 1.33:1.
- **Wide and narrow layouts are rough.** At 1920 the header actions sit far from the capped content column. At 760, rail plus list keep 39% of the width.
- **Several screens are rough:**
  - Calls: the Join button is misaligned with its input, and body text is 15pt.
  - Long-draft composer: the scroll indicator floats beside the attach icon.
  - Image thumbnails: they are cropped into fragments.
  - Error toast: it is centred on the window, not the pane.
  - First-sync-empty: the list shrinks to 220px.

## Screen scores

| Screen | Score | Screen | Score |
| --- | --- | --- | --- |
| chats | 8 | shortcuts | 6.5 |
| chats-wide | 7.5 | status-message | 7 |
| chats-narrow | 7.5 | account-menu | 8 |
| chat-no-selection | 8 | permission-consent | 7.5 |
| chat-filter-no-match | 8 | error-toast | 7.5 |
| group-chat | 7.5 | first-sync-loading | 8 |
| channel | 8 | first-sync-empty | 7 |
| channel-narrow | 7.5 | call-bar | 7 |
| teams | 8 | call-docked | 7.5 |
| calendar | 7.5 | join-on-phone | 8 |
| calls | 7 | setup | 7.5 |
| calls-narrow | 6.5 | signin | 8 |
| composer-mention | 8 | signin-browser | 8 |
| composer-long-draft | 7 | signin-device-code | 8.5 |
| composer-images | 7 | signin-finishing | 8 |
| image-preview | 7 | signin-expired | 8 |
| search-switcher | 8.5 | signin-failed | 8 |
| new-chat | 8 | settings | 7.5 |

## Top issues

1. **Unify buttons.** Make every button 13pt semibold and 32px tall. Primary is #a9530f with white text (light) or #e88c3e with #1c1208 (dark). Secondary is a 1px #8a8d95 outline (light) or #6e7179 (dark). Reply, Copy link, Restore and Close bar should use the same secondary style.
2. **Keep the hover toolbar inside the hovered row.** Place it at top + 4 and right - 8 so it never covers the row above.
3. **Shortcuts: use one grid for all sections.** Fix the key column at 190px.
4. **Fix hyphen rendering.** U+2011 is drawn from a fallback font or with extra spacing; use U+002D instead.
5. **Give interactive outlines at least 3:1.** Use the control border token for the toolbar, reaction chips and Reply buttons.
6. **Align the wide-window header with the capped content column.**
7. **Calls page.** Make the input and Join button both 32px on one row, set body text to 14pt, and add a hanging indent to wrapped lines.
8. **Centre toasts in the content pane, under the header.**
9. **Long draft.** Put the scrollbar along the composer's right edge, add a top fade and cap the height.
10. **Image thumbnails.** Use contain fit and add a name and size caption.
11. **First-sync-empty.** Keep the 300px list and its filter field.
12. **Narrow windows.** Shrink or collapse the list below 900px.
13. **Image preview.** Use a proper segmented control, a file name as the title, and Save, Copy and Open actions.

Further low-severity items are in A.json: settings fade and spacing, the status message input, duplicate presence icons, sparse calendar detail, top-heavy sign-in pages, and duplicate spinners while loading.
