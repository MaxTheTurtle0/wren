# Wren UI review, round 5 (reviewer A)

**Overall: 7.5 / 10.** A calm, coherent, well-built app. Contrast passes almost everywhere. Empty, loading and error states are mostly well covered. What keeps it from 8+ is a set of medium-severity polish issues spread across many screens, not one broken screen.

Dimensions: hierarchy 8, spacing 7.5, typography 7, consistency 7.5, contrast/accessibility 8, feedback 8, states 8.5, navigation 8.

## Contrast (measured)
- All sampled body text passes 4.5:1 in both themes. Secondary text #62656D on white is 5.83:1. Dark #9C9EA8 on #17181B is 6.65:1.
- Borderline passes: DEMO label 4.53:1 (light), selected-row timestamp 4.60/4.57:1, mention name on peach 4.81:1, unread time 4.95:1.
- Input and outline-button borders pass 3:1 (#8A8D95 3.07-3.32:1; dark #6E7179 3.09-3.64:1).
- Pill buttons fail as component boundaries: Copy link 1.16:1, call bar pills 1.07:1, theme segments 1.16:1 / 1.10:1, keycaps 1.33:1 / 1.16:1.

## Top issues
1. Spaced hyphen glyph app-wide ("On - call", "Sign - in", "F4KQ - 9TZW").
2. Wide layout: the conversation column is left-aligned and capped, leaving a ~600 px dead band. The header actions end mid-header.
3. Error toast covers conversation text and repeats the inline failure without Retry.
4. Calendar rows reuse the orange "selected" bar on every item. The actual selection is a 1.17:1 fill.
5. Mixed button styles: boundary-less pills next to outlined rounded rectangles.
6. Narrow windows: the fixed 232 px list pane truncates names badly.
7. Long-draft composer: a clipped glyph row at the top and a misplaced scroll thumb.
8. Tight and inconsistent paragraph leading (≈1.15 vs 1.6) across sign-in, setup and Calls.
9. Input and button height mismatches (28 vs 32 px) in Calls and setup.
10. Half-cut avatars at the top of scrolled history, with no fade.

## Screen scores
Lowest (7): chats-wide, channel-narrow, calendar, calls-narrow, composer-long-draft, error-toast.
Highest (8.5): chat-no-selection, chat-filter-no-match, group-chat, teams, search-switcher, account-menu, permission-consent, first-sync-loading, signin-device-code, signin-failed.
Full per-screen issues are in A.json.
