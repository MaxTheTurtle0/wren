# Independent screenshot design review, round 9 B

Overall: **8/10**. Contrast **9**, typography **8**, colour **8.5**, spacing **8**, hierarchy **8**.

Reviewed all 36 screens in both modes, 72 images total. Read only the supplied screen descriptions and images. This is a visual assessment; interactions, keyboard behaviour, animation, tooltips beyond those shown, and screen-reader support were not tested. Pointer-induced hover states were treated as intentional states, not capture defects.

The app is calm, coherent and readable, with strong light/dark parity. Empty states, inline send failure, permission consent, device-code sign-in and the QR dialog are particularly successful. It falls short of 9 because narrow layouts sacrifice identities, image crops hide attachments, and long drafts lack visible position feedback. Small metadata, long message measures and text-heavy settings/setup screens create additional reading friction.

## Measured contrast

Python 3 isolated mode and Pillow were used to sample actual solid glyph colours and adjacent rendered backgrounds. Ratios use sRGB linearisation and WCAG relative luminance. Antialiased edge pixels were not mistaken for foreground tokens. Measurements are representative checks, not proof of every WCAG 2.2 criterion.

| Element | Light | Dark |
| --- | --- | --- |
| Chat body | #1b1c1f / #ffffff, 17.04:1 | #e9e9ec / #17181b, 14.65:1 |
| Selected timestamp | #62656d / #e4e4e8, 4.60:1 | #9c9ea8 / #34353c, 4.57:1 |
| Composer helper | #62656d / #ffffff, 5.83:1 | #9c9ea8 / #17181b, 6.65:1 |
| Inline error | #b42525 / #f9eeee, 5.72:1 | #f58a8a / #292124, 6.64:1 |
| Sign-in error | #b42525 / #ffffff, 6.50:1 | #f58a8a / #25262b, 6.39:1 |
| Mention text | #a9530f / #fdf1e5, 4.81:1 | #e88c3e / #2b2219, 6.13:1 |
| Composer boundary | #7c7f87 / #ffffff, 4.01:1 | #82858e / #25262b, 4.09:1 |
| Settings select boundary | #7c7f87 / #eeeef0, 3.46:1 | #82858e / #2c2d33, 3.72:1 |
| Rail icon | #4a4d55 / #ececee, 7.17:1 | #b4b6bf / #131417, 9.11:1 |

Sampled text passes 4.5:1; sampled necessary icons and boundaries pass 3:1. No demonstrated text contrast failure requires a screen cap. Selected-row secondary text has limited headroom. Decorative separators and inactive buttons were not treated as failing required control boundaries.

## Priorities

1. **Narrow layout spends too much width on navigation and fixed secondary controls.** At 760x640 the rail plus sidebar ends at x=296, 39% of total width. Calls-narrow truncates Katherine Jo..., Dorothy Vaug... and Margaret Ha... before icons near x=233 and x=267. At widths below 800 points offer a collapsed navigation/list mode. In the retained list, move call actions to the selected or hovered row and give names the recovered 48-64 points. (both; chats-narrow, calls-narrow, channel-narrow.)

2. **Attachment previews conceal most of extreme-aspect-ratio images.** The 3072x256 and 256x3072 sources are both shown in approximately 144x96 thumbnails at x=407-551 and x=711-855, y=600-696; only cropped fragments are visible. Fit each entire image within a 144x96-point neutral frame; keep the dimensions and provide an explicit preview affordance. Reserve cropping for optional enlarged previews. (both; composer-images.)

3. **Long editor lacks visible position and length feedback.** Editor runs approximately y=553-761 and shows SELECT column_111 through column_119. No scrollbar, line-position label or hidden-content cue is visible inside it. Show a persistent internal scroll thumb for overflowing drafts, with 8 points of right padding. Add a compact line-count or expand-editor control above the attachment/send area. (both; composer-long-draft.)

4. **Message measure is too long for sustained reading.** At 1280 the last chat message runs from x=442 to about x=1197, roughly 110 characters. At 1920 a reply extends from x=762 to approximately x=1528. Cap message text at roughly 65-85 characters, about 620-680 points at the shown 14-point type, while keeping composer and header alignment deliberate. (both; chats, chats-wide, group-chat.)

5. **Long implementation explanation interrupts the settings controls.** Notification checkbox is at y=378, followed by a three-line paragraph at y=408-449; the next checkbox is at y=481. The dialog scrollbar indicates more settings below the visible browser selector. Keep a single 13-point helper line directly below notifications and place the detailed delivery explanation behind an info control or disclosure. Use 24-point section gaps and 32-36-point control rows. (both; settings.)

6. **Setup presents a dense wall of configuration work with weak step hierarchy.** Card at x=405-875, y=225-672 combines a three-line introductory paragraph, guide button, two fields, two-line tenant guidance, disabled save explanation and Advanced disclosure. Use explicit short steps: 1 Create registration, 2 Paste client ID. Keep the client-ID label and field together with 8-point spacing; put tenant explanation behind a disclosure and use 13-point helper text. (both; setup.)

7. **Frequent metadata and helper text are small relative to primary reading text.** Typical metadata glyphs occupy roughly 8-10 vertical pixels versus body glyphs around 13-14; timestamps, dates, shortcut hints and account addresses recur throughout. Selected timestamps measure #62656d on #e4e4e8 = 4.60:1 light; #9c9ea8 on #34353c = 4.57:1 dark, both passing but close to 4.5. Use 13-point metadata with an 18-point line height where people need to read it. Increase selected-row secondary text headroom using #555963 in light and #b0b3bd in dark. (both; chats, chats-wide, group-chat, channel, calendar, account-menu, search-switcher.)

8. **At the smallest call width a low-priority utility consumes a full additional row.** At 380x800, Join on phone and Full window occupy y=38-70 and Hide call bar occupies y=76-108; total call bar height is 118 pixels versus 80 pixels in call-docked. Keep the primary two actions on one 32-point row and move Hide call bar into a compact overflow button or a trailing icon, preserving a labelled menu item. (both; call-bar.)

9. **Calling page foregrounds explanatory prose over the calling task.** At 1280 How calls work starts y=207, with body copy y=230-259, capability rows y=291 and y=314, and Details y=344. At 760 the paragraph occupies y=230-300. Keep a one-line 13-point explanation near Join a meeting and move the fuller explanation into the existing Details disclosure. Add visible Audio/Video labels for the selected person. (both; calls, calls-narrow.)

10. **Close icon is misaligned with the dialog outer padding.** Dialog spans approximately x=166-1115; close icon centre is x=1010, about 104 pixels from the right edge, while title starts 23 pixels from the left edge. Align the close control 24 points from the right edge, around x=1091 in this screenshot, and use the same header padding as the other dialogs. (both; shortcuts.)

11. **Keyboard notation is inconsistent within a single reference panel.** Ctrl+N and Ctrl+K include plus signs; Shift Enter uses a space; Alt arrow shortcuts separate the modifier and arrow differently. Use a consistent keycap convention, such as Ctrl+N, Shift+Enter, Alt+Up and Alt+Down, with the same 12-13-point label treatment. (both; shortcuts.)

12. **Meeting time is less prominent than a redundant relative start field.** Absolute time appears in the small header subtitle near y=34; detail card gives Starts / in 24 min a full row near y=205, beneath organiser and online type. Promote the date and absolute time to a 15-point semibold line at the top of the detail card; put the relative countdown beside it in 13-point text and remove the redundant Starts row. (both; calendar.)

13. **Small footer has a much longer reading measure than the main content.** Footer runs approximately x=212-1068, about 856 pixels, while the sign-in card is only 470 pixels wide. Its glyphs are approximately 9 pixels tall. Wrap footer into two or three lines within a 560-640-point maximum width, at 13 points with an 18-point line height, separated by 24-32 points from the preceding control. (both; signin, signin-browser, signin-device-code, signin-finishing, signin-expired, signin-failed, setup.)

14. **Bottom-aligned short history produces a large visually unstructured void.** Call-docked header ends at y=132 and first date divider is y=348, leaving about 216 pixels. Chats-wide header ends y=52 while first date divider is y=193. For short histories, reduce the top void with a small contextual meeting/date summary or a bounded top spacer of 48-72 points while preserving sensible newest-message access. Avoid adding decorative filler. (both; call-docked, chats-wide.)

15. **Own-author names and explicit mentions use the same strong orange, reducing semantic distinction.** In group-chat the mention Alex Rivera around x=442,y=455 uses #a9530f on #fdf1e5; the own-author name around x=442,y=491 uses the same accent family, alongside orange selected reactions and focus border. Keep ordinary author names in the primary neutral colour at semibold weight. Reserve orange text for actionable or mentioned content and retain the tinted mention row. (both; group-chat, composer-mention.)

## Screen scores

Each score considers light and dark together.

| Screen | Score | Issues |
| --- | --- | --- |
| chats | 8 | Long message lines weaken reading rhythm at the standard width. Secondary metadata is small; selected timestamps have little contrast headroom. |
| chats-wide | 8 | Conversation is isolated in a narrow central island with large side gutters and a large blank region above history. Message lines still run long despite the capped column. |
| chats-narrow | 7.5 | Sidebar retains 296 of 760 pixels, truncating common names and previews. Message text has to wrap around a relatively expensive avatar gutter. |
| chat-no-selection | 8.5 | Clear empty state, but its 15-point heading is modest against the large empty pane. |
| chat-filter-no-match | 8.5 | Clear filter recovery; small helper copy and a very tall empty sidebar make the state feel slightly sparse. |
| group-chat | 8 | Message lines are long and spacing between days, system events and messages is uneven. Own-author orange and mention orange share the same emphasis. |
| channel | 8.5 | Good post/reply grouping; reply metadata and compact Reply controls are noticeably smaller than message text. |
| channel-narrow | 8 | Persistent 296-pixel navigation area limits post reading width. Top post begins partially scrolled beneath the fixed header; the fade makes the author less distinct. |
| teams | 8.5 | Tree is legible and well spaced; empty-state heading could carry more emphasis. |
| calendar | 8 | Meeting time is secondary in the header while a redundant relative Starts value gets a full detail row. Details occupy only the upper quarter of a mostly empty main pane. |
| calls | 8 | Explanatory copy and capability statements occupy more visual space than the actual calling actions. Repeated audio/video icons have no visible text labels. |
| calls-narrow | 7.5 | Katherine, Dorothy and Margaret are truncated while the two call icons retain fixed horizontal space. Explanatory paragraph becomes four lines before the capability rows. |
| composer-mention | 8.5 | Mention popup is clear, but compact helper and attachment metadata introduce several tiny type treatments. Popup covers part of the preceding conversation, expected but visually busy. |
| composer-long-draft | 7.5 | Only lines 111 through 119 of a much longer draft are visible, with no visible internal scrollbar or length cue. Tall draft has little distinction between editor content and send controls. |
| composer-images | 7.5 | Uniform thumbnail crops hide most of the very wide and very tall attachments. Dimensions alone do not communicate the omitted image content. |
| image-preview | 8.5 | Clear preview and zoom controls; dimension label is small and far less prominent than the title. |
| search-switcher | 8.5 | Strong keyboard affordances; nearly every long preview ends in an ellipsis. Footer combines several small instructions with another line of explanatory copy. |
| new-chat | 8.5 | Clear selected recipient and primary action; selected result styling can be mistaken for an already added person at a glance. |
| settings | 8 | Long notification explanation interrupts the checkbox rhythm. Fixed narrow dialog requires scrolling while unused horizontal room remains outside it. |
| shortcuts | 8 | Close icon sits about 104 pixels inside the dialog right edge. Key labels use mixed conventions such as Shift Enter versus Ctrl+N. |
| status-message | 8.5 | Well proportioned compact dialog; hint is entirely inside the empty field and disappears when typing. |
| account-menu | 8.5 | Clear account and presence grouping; two Alex Rivera accounts rely on small secondary addresses for differentiation. |
| permission-consent | 8.5 | Useful task-specific explanation; permission identifier and timing explanation form a small dense final paragraph. |
| error-toast | 8.5 | Good persistent inline failure and Retry; failed message body is visually de-emphasised relative to normal messages. |
| first-sync-loading | 8.5 | Clear loading text and skeleton; no visible secondary recovery cue if this state persists, though timing cannot be judged from a still. |
| first-sync-empty | 8.5 | Clear actionable first-use state; large vacant sidebar still presents all three chat toolbar actions. |
| call-bar | 8 | Three utility buttons wrap into two rows at 380 pixels, consuming 118 pixels before the meeting header. Small meeting-event text makes the timeline quieter than the utility controls. |
| call-docked | 8 | About 216 pixels of blank space separate the header and first date divider. Utility buttons have more visual prominence than the call status. |
| join-on-phone | 9 | No material visual defect in the shown QR dialog; clear instructions, generous quiet zone and readable fallback. |
| setup | 7.5 | Registration guide and configuration explanation dominate first use. Tenant guidance is dense and the default organizations value needs careful reading. Small save guidance and footer increase text burden. |
| signin | 8.5 | Clean sign-in hierarchy; very long footer competes with otherwise compact reading width. Hover tooltip extends well beyond the card, though it remains readable. |
| signin-browser | 8.5 | Clear waiting state and recovery actions; footer spans roughly 856 pixels in a single small line. |
| signin-device-code | 9 | Clear code hierarchy, legible code and obvious copy/open actions; only a minor footer readability nit. |
| signin-finishing | 8.5 | Clear finishing state; branded mark is much larger than the active status, and the footer remains disproportionately wide. |
| signin-expired | 8.5 | Account and recovery are clear; large brand and wide footer distract slightly from the reauthentication task. |
| signin-failed | 8.5 | Good specific error and two recovery paths; wide small footer remains the main readability nit. |
