# Wren UI review, round 3 (reviewer A)

**Overall: 7.5 / 10**

| Dimension | Score |
| --- | --- |
| Visual hierarchy | 8 |
| Spacing and alignment | 7 |
| Typography | 7.5 |
| Consistency | 7 |
| Contrast and accessibility | 8 |
| Feedback | 7.5 |
| Empty, loading, error states | 8 |
| Navigation and wayfinding | 7 |

## Summary

The app is calm and dense, and it looks the same in light and dark mode. Every
text sample I measured passes WCAG AA: secondary text #62656d on #ffffff is
5.83:1, #9c9ea8 on #17181b is 6.65:1, the accent #a9530f on white is 5.34:1, and
button labels are 5.34:1 in light and 7.24:1 in dark. No screen is capped for
text contrast. The faults are in layout and polish. Narrow and call layouts don't
adapt. The long-draft and image composers are rough. Secondary controls have
weak edges. A few components don't match each other: switcher avatars, glyph
carets, and dialogs without a close button.

Strongest screens: the sign-in family (8-8.5), the main chat views (8),
shortcuts, permission consent and the loading states.
Weakest screens: channel-narrow, calls-narrow, call-bar, composer-long-draft and
composer-images (6.5 each).

## Top issues

1. **Narrow layouts don't adapt** (high). The list stays at about 232 px of a
   760 px window, names truncate, placeholders wrap with a dangling "·", and the
   helper text runs to 2-3 lines.
2. **Call layouts** (high). The title truncates with about 160 px still free,
   and there is no rail or other way to reach other chats.
3. **Long-draft composer** (high). Leading is about 1.13, the first line is
   clipped, and the scrollbar sits next to the paperclip.
4. **Secondary button, chip and card borders** (medium). They measure 1.33:1
   in light and 1.16-1.37:1 in dark. Use a fill, or the input border tokens
   #8a8d95 / #6e7179.
5. **Switcher avatars don't match the chat list** (medium). Ada is blue in the
   list and green in the switcher, and the meeting chat shows "D0" instead of
   its calendar icon.
6. **Wide layout** (medium). The message column is capped at about 920 px,
   but the composer is 1520 px wide.
7. **Image thumbnails** (medium). They are centre-cropped into fragments, and
   the remove buttons sit on top of the image.
8. **Hover toolbar** (medium). It straddles neighbouring rows and card borders.
9. **Field, button and glyph mismatches** (medium). On Calls the Join field and
   button differ in height, disclosures use a ▸ text glyph, and settings
   dropdowns use a ▼ glyph instead of Lucide icons.
10. **Calendar detail** (medium). It is thin and repeats itself: "Location:
    Teams" duplicates the online row, and attendees are missing.
11. **Ambiguous presence icons** (medium). "Be right back" and "Appear away"
    use the same icon, and "Busy" and "Do not disturb" are both red.
12. **Error toast** (medium). It sits at the top left of the pane, covers
    messages, and is far from the failed message.
13. **Dialogs** (low). Some have no close button, and a disabled primary button
    looks like a secondary one.
14. **Loading and empty states** (low). The refresh button stays active during
    loading, the filter is enabled with nothing to filter, and the empty message
    is shown twice.
15. **Image preview** (low). It is a modal, not full-window, its "100%" segment
    has no edge, and it has no Save or Copy actions.
16. **Hyphens** (low). They render with a gap: "On -call", "Sign -in".
17. **Setup** (low). A stray scrollbar shows, and the guide link isn't styled
    like other links.

Per-screen scores, issues and concrete fixes are in `A.json`.
