# Wren UI review, round 13 (reviewer A)

**Overall: 8 / 10**

| Dimension | Score |
| --- | --- |
| Contrast | 8.5 |
| Typography | 8 |
| Colour | 7.5 |
| Spacing | 7.5 |
| Hierarchy | 8 |

## Summary
This is a calm, consistent and well-built app in both themes. I sampled pixels across all 36 screens, and every real text sample passes WCAG AA. The lowest is placeholder and caption grey #62656d on #f6f6f7, at 5.40:1. Secondary text is #4a4d55 in light and #b4b6bf in dark, and the dark accent text #e88c3e measures 7.17:1. Disabled controls measure about 4.55:1. Input, composer and button borders measure 3.7-4.9:1. The only sub-3:1 text is the disabled filter placeholder while loading (2.08:1 light, 2.62:1 dark), which is exempt.

The app falls short of 9 because of layout and state problems, not text contrast:

1. **Narrow windows.** At 760 px the list column stays full width, so names truncate and the conversation pane is cramped.
2. **Long-draft composer.** A stray scrollbar sits in the button row, and the top line is clipped.
3. **Dark hover toolbar.** Its fill is the same as the hovered row (1.00:1), so it has no surface of its own.
4. **Flat light surfaces.** Rail, sidebar and main differ by about 1.08:1, and the selected row is 1.19:1. Panes and cards are held together by hairlines at 1.33:1.
5. **Overloaded accent.** It marks unread state, selection, focus, primary actions, checkboxes and mentions. In light mode #a5500f reads brown, against the bright orange of dark mode.
6. **Dense popovers.** The search switcher and mention popup are denser than the main lists, and their footers crowd the rows.
7. **Thin pages.** The calendar detail and Calls pages have little content and a lot of empty pane.
8. **Uneven rhythm.** Spacing varies inside dialogs and cards: shortcuts section gaps are 41 px vs 28 px, and the setup card's gaps are irregular.

## Screen scores
- 8.5: chats, chats-wide, chat-no-selection, group-chat, teams, first-sync-loading, signin-browser, signin-finishing
- 8: chat-filter-no-match, channel, new-chat, status-message, account-menu, permission-consent, error-toast, first-sync-empty, call-docked, join-on-phone, signin, signin-device-code, signin-expired, signin-failed
- 7.5: channel-narrow, calls, composer-mention, composer-images, image-preview, search-switcher, settings, shortcuts, call-bar, setup
- 7: chats-narrow, calls-narrow, calendar
- 6.5: composer-long-draft

`grade.json` has the full per-screen issues and 16 prioritised fixes with measured evidence.
