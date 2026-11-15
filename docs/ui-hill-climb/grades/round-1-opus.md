# Wren design review, round 1 (reviewer A)

**Overall: 7.0 / 10** (mean of 36 screen scores: 6.9)

| Dimension | Score |
| --- | --- |
| Hierarchy | 7.5 |
| Spacing and alignment | 6.5 |
| Typography | 6.5 |
| Consistency | 7 |
| Contrast and accessibility | 6.5 |
| Feedback | 6.5 |
| Empty, loading, error states | 7.5 |
| Navigation | 7 |

A calm, coherent palette that works in both themes, with good empty states and a clear rail. Most body and secondary text passes WCAG comfortably: secondary text #62656d on white is 5.83:1, and in dark mode #9c9ea8 on #17181b is 6.65:1. What holds it back is a set of repeated small defects: light placeholders that just miss 4.5:1, which caps most screens at 7, weak input boundaries, a broken hyphen glyph, overlapping hover toolbars, and a few states that are wrong or missing (failed send, call status).

## Top issues

1. **Light placeholders fail 4.5:1.** #767779 on #fff measures 4.48:1 and #727376 on #f6f6f7 measures 4.39:1. Use #62656d.
2. **A failed send shows only as a toast.** The toast covers the last message, is centred on the window (x 640) instead of the pane (x 822), and has no Retry. Mark the message inline with Retry.
3. **Settings checkboxes look like radio buttons.** Unchecked is a 13 px circle at 2.72:1. Use 16 px squares with a border of at least 3:1.
4. **The call bar contradicts the chat.** It says "In the meeting" while the chat says "Meeting ended". There is a 34 px gap under the title, the title is cut off with space to spare, and the primary action is labelled "Done", which is unclear.
5. **The hover toolbar covers author lines and reactions** in narrow views (chats-narrow y=268-303, channel-narrow).
6. **Hyphens render with wide gaps** everywhere ("On - call", "Sign - in", "F4KQ - 9TZW").
7. **Inputs have no visible boundary or focus ring.** The composer border is 1.33:1 in light and 1.37:1 in dark.
8. **A long draft clips its top line** and shows no scrollbar. The send and attach icons sit vertically centred.
9. **Narrow windows keep the full 300 px sidebar**, and wide windows have no maximum reading width (lines of about 800 px at 1920 wide).
10. **Disabled buttons are illegible** (1.5:1 to 1.9:1), and in Calls the link field and the Join button differ in height (29 px against 34 px).
11. **Presence states share colours.** Busy and DND use the same red dot, and BRB and Away the same amber dot. In light mode the away colour (#8a5a00) is close to the unread colour (#a9530f).
12. **The account menu has three different left edges**, and the conversation header has no divider, so content clips under it.

## Weakest screens
error-toast 5.5, call-bar 5.5, chats-narrow 6, channel-narrow 6, calls-narrow 6, composer-long-draft 6, settings 6, call-docked 6.

## Strongest screens
shortcuts 8, permission-consent 8, signin-finishing 8.
