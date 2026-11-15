# Wren UI review, round 16 (reviewer A)

**Overall: 8.0 / 10.** Contrast 8.5 · Typography 8.0 · Colour 7.5 · Spacing 8.0 · Hierarchy 8.0

A calm, dense, consistent app. Inter is used on a disciplined scale, and the burnt-orange accent is legible in both themes. Light and dark modes are close in parity. Every real text sample I measured passes WCAG AA:
- Secondary text: #4a4d55 is 7.7:1 on the list and 8.5:1 on white. #62656d is 5.3:1 at its worst.
- Dark secondary text: #9c9ea8 is 5.6:1 at its worst, on #26272c.
- Accent buttons: white on #b4530a is 5.0:1. Dark #1c1208 on #e88c3e is 7.2:1.
- Links: 5.7:1 light, 7.9:1 dark.

Interactive borders also pass: inputs and outline buttons are #7c7f87 at 3.6-4.0:1, and dark #82858e at 4.0-4.7:1. Only the disabled loading-state placeholder falls below (2.1:1 light, 2.6:1 dark). Disabled controls are exempt.

## What keeps it below 9
1. Scrolled history is hard-clipped under the header. A line of text or a card is sliced in half on most chat and channel screens (y≈52).
2. The surface ladder is flat:
   - card fill to page: 1.10:1 light, 1.06:1 dark
   - card border: 1.33:1
   - rail to list: 1.11:1 light, 1.06:1 dark
   - skeletons: 1.16:1

   Dark cards are recessed rather than raised.
3. Identity colours collide. AL and AT share #4560c4, and O and HL share #1f7a5c. The Engineering and Contoso tiles are both #c2416f. The Product tile (#b4581c) is nearly the accent (#b4530a).
4. Selection and focus have no single language. Settings uses a solid accent segment, image preview a grey segment, and list rows grey plus a bar. The 2 px accent composer ring is the loudest thing on screen.
5. Secondary-screen rough edges:
   - an artefact band at the bottom of Settings
   - 'Meeting ended' lines 8 px under messages
   - a composer scrollbar jammed beside the paperclip
   - the channel hover toolbar overlapping card borders
6. The Calendar and Calls detail pages are sparse. Their header actions float at x=1114 mid-bar.

## Screen scores
- **8.5:** chats-wide, chat-no-selection, search-switcher, new-chat, shortcuts, status-message, permission-consent, error-toast, and all sign-in screens except setup
- **8:** chats, chat-filter-no-match, group-chat, teams, calls, calls-narrow, composer-mention, composer-images, image-preview, account-menu, first-sync-loading, first-sync-empty, call-docked, join-on-phone, setup
- **7.5:** chats-narrow, channel, channel-narrow, calendar, composer-long-draft, settings, call-bar

`grade.json` has per-screen issues and 15 prioritised fixes with measured evidence.
