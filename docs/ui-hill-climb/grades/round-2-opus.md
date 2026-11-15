# Wren UI review, round 2 (reviewer A)

**Overall: 7 / 10.** The app is good, with clear issues. The core chat views are calm, dense and readable in both themes. The secondary screens (Calls, Calendar, Settings, image handling, call bar) are less finished, and a few concrete bugs remain.

Mean of the 36 screen scores: 7.4.

| Dimension | Score |
| --- | --- |
| Hierarchy | 7.5 |
| Spacing and alignment | 7 |
| Typography | 6.5 |
| Consistency | 6.5 |
| Contrast and accessibility | 7.5 |
| Feedback | 7 |
| Empty, loading, error states | 7.5 |
| Navigation | 7.5 |

## Contrast (measured with Pillow)

Every text sample I measured passes 4.5:1 in both themes. Lowest values:

- DEMO label: #a9530f on #ececee, 4.53:1
- Hovered-row email: #62656d on #e4e4e8, 4.60:1
- Same in dark: #9c9ea8 on #34353c, 4.57:1

No screen is capped by the text contrast rule.

The weak spots are non-text elements:

- **Selected chat row:** 1.17:1 light, 1.36:1 dark, shown by fill only.
- **Secondary button borders:** 1.33:1 light, 1.16:1 dark.
- **Unselected theme pills:** 1.16:1 light, 1.10:1 dark.
- **Card borders:** 1.33:1 light, 1.37:1 dark.

Input borders pass: 3.07:1 light, 3.64:1 dark.

## Top issues

1. **The error toast hides the failed message and its Retry button** (error-toast). High severity.
2. **One Settings checkbox is a ☑ text glyph**, offset 8 px from the real checkboxes (settings). High severity.
3. **Hyphens render spaced throughout** ('On - call', 'Sign - in', 'F4KQ - 9TZW').
4. **Selection is fill-only and fails 3:1** in the chat list and switcher. Add a 3 pt accent bar.
5. **Calendar and Calls have no header bar**, unlike Chat and Channel.
6. **At 1920 px the message column is capped**, but the composer and header stretch, so they misalign.
7. **Focus rings are inconsistent**: the composer gets an orange ring, dialog inputs only a grey border.
8. **The long-draft composer is cramped** (about 17 px lines), with a stray scrollbar stub beside the attach icon.
9. **The Calls page reads as help documentation**, with weak grouping, truncated names at narrow width and an input/button height mismatch.
10. **The call bar doesn't stand out** from the header and has no leave action. The demo state contradicts itself.
11. **Image thumbnails crop into meaningless slices.** The preview modal's Fit/100% toggle and margins are weak.
12. **Button heights and label sizes vary** between screens and within button groups.

The full list, with evidence and fixes, is in A.json (17 items).

## Best screens

- shortcuts, signin-device-code, first-sync-loading, chat-no-selection: 8.5 each.

## Weakest screens

- error-toast: 5
- settings: 6
- calls-narrow: 6
- calls, composer-long-draft, composer-images, image-preview, call-bar: 6.5 each
