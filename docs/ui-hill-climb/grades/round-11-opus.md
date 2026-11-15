# Wren UI review, round 11 (reviewer A)

**Overall: 8 / 10.** Contrast 9 · Typography 8 · Colour 8 · Spacing 7.5 · Hierarchy 8

## Summary
Wren is calm and dense, and it reads well. The text palette is now solid. Every text sample I measured in both themes passes WCAG AA:
- Body text is #1b1c1f on #fff (17:1), or #e9e9ec on #17181b (14.6:1).
- Secondary text is #4a4d55 (8.4:1) or #b4b6bf.
- Tertiary text is #62656d (5.4-5.8:1) or #9c9ea8 (5.7-6.7:1).
- The accent is #a5500f (5.0-5.6:1) or #e88c3e.
- Input and button outlines are #7c7f87, which gives 3.7-4.0:1 against their backgrounds.

Light and dark modes match well.

What keeps it at 8 is structure and polish, not colour:

1. **Overlay geometry.** The image preview is a 258 px strip pinned to the top, not a full-window viewer. Dialog tops land anywhere from y=48 to y=207.
2. **Narrow layouts.** At 760 wide the list pane keeps its full width, so names truncate to "Katherine Jo..." and "Grace Hop...".
3. **Calendar detail.** It is mostly empty: two key-value rows, a title with no emphasis, and help text about online meetings shown for an offline event.
4. **Scroll affordances.** The bars are heavy and always visible, and the setup page overflows at 800 px. The long-draft composer has a black thumb. Settings content is cut through a checkbox with no fade.
5. **Dark-mode surfaces.** Layering is inverted: the list pane (#1d1e22) is lighter than the reading pane (#17181b). In light mode the panes are separated by a 1.08:1 tint only.
6. **Accent overload.** One orange marks selection, focus, unread, own reactions, mentions and primary buttons.
7. **Smaller issues.** The sign-in tooltip overflows its card. Channel Reply buttons outweigh the posts. The loading skeleton does not match the loaded list. Vertical gaps vary within single screens.

## Screen scores
| Screen | Score | Screen | Score |
|---|---|---|---|
| chats | 8.5 | settings | 7.5 |
| chats-wide | 8.5 | shortcuts | 8.5 |
| chats-narrow | 7.5 | status-message | 8.5 |
| chat-no-selection | 8.5 | account-menu | 8 |
| chat-filter-no-match | 8.5 | permission-consent | 8.5 |
| group-chat | 8.5 | error-toast | 8.5 |
| channel | 8 | first-sync-loading | 8 |
| channel-narrow | 7.5 | first-sync-empty | 8 |
| teams | 8 | call-bar | 8 |
| calendar | 7 | call-docked | 8 |
| calls | 8 | join-on-phone | 8.5 |
| calls-narrow | 7.5 | setup | 7.5 |
| composer-mention | 8 | signin | 8 |
| composer-long-draft | 7.5 | signin-browser | 8.5 |
| composer-images | 8 | signin-device-code | 8 |
| image-preview | 7 | signin-finishing | 8.5 |
| search-switcher | 8.5 | signin-expired | 8 |
| new-chat | 8.5 | signin-failed | 8.5 |

`grade.json` has the 16 prioritised issues with measurements and concrete fixes.
