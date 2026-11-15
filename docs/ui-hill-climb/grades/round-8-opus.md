# Wren UI review, round 8 (reviewer A)

**Overall: 8.0 / 10.** A calm, coherent and readable app. Every text sample I measured passes WCAG AA in both themes. The light theme uses #62656d muted text and #a9530f accent; the dark theme uses #9c9ea8 and #e88c3e. Light muted text is at least 4.6:1 and the dark theme is at least 4.57:1. What holds the score back is layout at narrow sizes, inconsistent page widths and dialog patterns, and meeting mode losing navigation.

| Dimension | Score |
| --- | --- |
| Hierarchy | 8.5 |
| Spacing and alignment | 7.5 |
| Typography | 8 |
| Consistency | 7.5 |
| Contrast and accessibility | 8.5 |
| Feedback | 8.5 |
| Empty, loading, error states | 8.5 |
| Navigation | 7.5 |

## Strongest screens (8.5)
chats, chats-wide, chat-no-selection, chat-filter-no-match, group-chat, teams, search-switcher, status-message, account-menu, error-toast, first-sync-loading, join-on-phone, and the sign-in, device-code, finishing and expired cards.

## Weakest screens (7 to 7.5)
chats-narrow, calls-narrow and composer-images score 7. channel-narrow, calendar, calls, composer-mention, shortcuts, call-bar, call-docked and setup score 7.5.

## Top issues
1. **Narrow layouts (high).** At 760pt the list stays 232pt wide, so names truncate ('Design rev...', 'Dorothy Vaug...' 2pt from the phone icon). Below 900pt, shrink the list to 200pt or collapse it to an avatar strip.
2. **Page widths differ.** Calendar uses a 640pt card and Calls a 680pt card, both left-aligned, while chats use a centred column. Use one centred 720pt column everywhere.
3. **Dialog footers differ.** The primary button is on the left in New chat and Permission but on the right in Status message. Right-align the buttons, secondary then primary, with an 8pt gap.
4. **Meeting mode drops the rail.** No chat switching is possible, and 'Restore' is ambiguous.
5. **Pasted-image thumbnails crop the content.** Letterbox them in 144x96pt tiles and move the remove buttons onto the corner.
6. **Shortcuts dialog is off-centre.** It runs from 90 to 787 in an 800pt window. Centre it with a max height, or use two columns.
7. **Calendar detail is sparse.** Unlabelled boilerplate sits in the label column, and the blue/hollow event dots are unexplained.
8. **A failed send is not flagged in the chat list.**
9. **Control borders barely clear 3:1.** Light #8a8d95 is 3.07:1 on #f6f6f7 and 2.99:1 on #f3f3f4; dark #6e7179 is 3.09:1 on #25262b. Darken them to #7c7f87 (light) and #7a7d86 (dark).
10. **Small alignment slips.** The Calls Join button is 32pt next to a 28pt input, and the Settings helper text is not indented to its checkbox label.

Lower-priority items cover faint channel cards, mention popup placement, the blank list in first-sync-empty, Inter's wide hyphen, setup density and image preview actions. Details are in A.json.
