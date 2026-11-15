# Independent screenshot review: 8/10

All 36 screens were inspected in light and dark mode, 72 images total. Only the supplied screenshots and screens.md informed this review. Scores combine both versions. The app is calm, coherent and readable, with especially good first-chat, device-code and failed-send states. It is good enough to ship with visible polish work remaining, rather than exemplary throughout.

The main problems are narrow panes that truncate identities, disappearance of video-call affordances at narrow width, cropped attachment thumbnails, a long draft with no visible internal scroll position, and confusing candidate highlighting in New chat. Hover states caused by the centred pointer were accepted as legitimate; their density was assessed rather than their mere presence.

## Dimensions

Hierarchy 8; spacing 8; typography 8; consistency 8.5; contrast/accessibility 8.5; feedback 7.5; states 8.5; navigation 8.

## Contrast measurements

Measured with python3 -I and Pillow from the original pixels, using sRGB linearisation and WCAG relative luminance ratios. Solid glyph colours were distinguished from antialiased edge colours. Backgrounds came from the same local text/control region. Recurring solid text palette colours were also checked for presence across all 72 images. These are screenshot checks, not a certification of keyboard, screen-reader or dynamic interaction accessibility.

| Example | Light | Dark |
| --- | --- | --- |
| Chat body | #1B1C1F on #FFFFFF: 17.04:1 | #E9E9EC on #17181B: 14.65:1 |
| Sign-in explanatory body | #4A4D55 on #FFFFFF: 8.45:1 | #B4B6BF on #25262B: 7.47:1 |
| Muted chat helper | #62656D on #FFFFFF: 5.83:1 | #9C9EA8 on #17181B: 6.65:1 |
| Selected chat timestamp | #62656D on #E4E4E8: 4.60:1 | #9C9EA8 on #34353C: 4.57:1 |
| Error message | #C42B2B on #FBEEEE: 4.98:1 | #F27474 on #281F22: 5.75:1 |
| Sign-in failure | #C42B2B on #FFFFFF: 5.63:1 | #F27474 on #25262B: 5.41:1 |
| Permission helper | #62656D on #FFFFFF: 5.83:1 | #9C9EA8 on #25262B: 5.66:1 |
| Attachment size | #62656D on #EEEEF0: 5.03:1 | #9C9EA8 on #2C2D33: 5.14:1 |
| Composer outline / unchecked checkbox | #8A8D95 on #FFFFFF: 3.32:1 | #6E7179 on #25262B: 3.09:1 |

No sampled solid text pair failed 4.5:1 and the sampled functional outlines passed 3:1, so no contrast cap was applied. Selected metadata and dark outlines have little margin. Tiny type and colour-only presence distinctions remain accessibility concerns despite passing text ratios. Decorative separators and dimmed content behind modal scrims were not assessed as active control/text failures.

## Priorities

1. **Medium: Narrow layouts reserve too much width for the sidebar and truncate identities.** (chats-narrow, channel-narrow, calls-narrow; both). At 760x640 the rail and sidebar end at x=296, 39% of the window. Chat names include Katherine Jo... and Margaret Hamil...; the content pane has only 464 px. Below 900 px, make the list collapsible with a labelled Back to chats/Teams control; use a 200 pt list plus the 64 pt rail when both panes are shown. Preserve full names before preview text.

2. **Medium: Video calling loses its visible entry point at narrow width.** (calls-narrow; both). Wide Calls has audio at x=295 and video at x=333. At 760 px only the audio icon near x=265 remains; there is no visible overflow menu. Keep both 24 pt icon targets with 8 pt separation, or add a 32 pt overflow button with labelled Audio call and Video call choices.

3. **Medium: Attachment thumbnails conceal the contents of extreme aspect-ratio images.** (composer-images; both). 3072x256 and 256x3072 attachments both occupy roughly 144x96 px tiles at x=393,545,697 and y=600-696. The tall attachment shows only fragments. Fit the entire bitmap within a 144x96 pt tile using contain, with a neutral #EEEFF2 light / #2C2D33 dark backing. Keep a 12 pt dimensions label and a 24 pt remove target.

4. **Medium: Long draft overflow has no visible position cue.** (composer-long-draft; both). Editor spans x=380-1264, y=553-761. It shows SELECT column_111 through column_119 with no visible scrollbar inside the editor, although the content clearly starts earlier. Add a persistent 6 pt internal scroll thumb with 4 pt inset while content overflows. Add a 24 pt expand-editor action beside attachment/send controls; retain the approximately 200 pt default cap.

5. **Medium: Search highlight and committed recipients are easy to confuse.** (new-chat; both). Grace Hopper is the chip at y=173, Barbara Liskov has a filled row at y=269-313, but the primary action still reads Chat with Grace Hopper. This can be keyboard highlight rather than selection, but the appearance is ambiguous. Add a visible + Add action at the trailing edge of each candidate row and a checkmark only for committed recipients. Label the bottom action Start chat (1 person), using 13 pt text.

6. **Medium: Hover action bars give many small actions equal visual weight.** (chats, chats-narrow, group-chat, composer-mention, composer-long-draft, composer-images, error-toast, channel, channel-narrow; both). Group-chat toolbar occupies approximately x=955-1252, y=412-450 with six reactions plus reply/edit/delete/copy. Narrow chat packs actions into x=519-732. The pointer-induced hover itself is expected. Show Reply, one reaction picker and More as three 28 pt targets with 4 pt gaps. Put copy/edit/delete behind More; retain 8 pt offset from author metadata.

7. **Medium: Meeting details bury the actual schedule below organiser-oriented metadata.** (calendar; both). The card at x=388-1028, y=98-309 lists Organizer, Location, Online, Starts in 24 min. Absolute times appear only as 12 pt text in the header at y=34; no absolute date is present in the detail body. Place Today, 15 November · 12:52-13:37 as a 15 pt first row and Starts in 24 min below it at 13 pt. Group organiser/location beneath with 16 pt separation.

8. **Medium: Settings mixes dense explanatory prose with controls and ambiguous bottom-edge fading.** (settings; both). The 560 px dialog shows a three-line paragraph at y=433-475 between checkboxes. Move Wren beside the call window is faded at y=713, coinciding with the scroll viewport edge. Use 13 pt helper text immediately beneath the related checkbox with 6 pt gap, move relay detail behind a Learn more disclosure, and reserve 16 pt bottom padding beyond the last fully visible row. Explain disabled settings explicitly instead of relying on a fade.

9. **Medium: Available and busy presence often rely on colour alone.** (chats, chats-wide, group-chat, calls, calls-narrow, account-menu; both). List avatars use a plain green dot for Ada and a plain red dot for Grace; DND and away already have internal glyphs. The Account menu supplies text, but the repeated sidebar dots do not. Use 10 pt presence markers with a check for Available and a horizontal bar for Busy, preserving current colour and a 2 pt backing ring. Keep the full text labels in the account menu.

10. **Low: Reading width is excessive for long message paragraphs.** (chats, chats-wide, channel, group-chat; both). At 1280 px body text can run from x=432 toward x=1260. At 1920 px the bounded conversation starts near x=680 after a 316 px gutter from the sidebar, and body text still spans very long lines. Cap message body text at 720 pt, leaving action controls in a separate trailing area. At wide size reduce the left conversation gutter to 48-96 pt and align header, message avatars and composer to the same content column.

11. **Low: First-run configuration presents several technical prerequisites at once.** (setup; both). Card x=405-875, y=222-648 combines registration prose, guide button, GUID field, tenant guidance, save explanation and WREN_CLIENT_ID/WREN_TENANT environment variable text. Use three numbered 14 pt steps: Create registration, Paste application ID, Choose tenant. Keep 16 pt between steps; move environment-variable setup behind a 13 pt Advanced disclosure.

12. **Low: Small metadata and symbols require precise reading and pointer placement.** (chats, chats-wide, chats-narrow, group-chat, channel, channel-narrow, composer-mention, composer-images, error-toast, account-menu; both). Timestamps and composer helper text are approximately 12 pt; rail labels about 11 pt. The attachment chip remove mark is about 8 px, and image remove buttons about 22 px. Use 13 pt for metadata and navigation labels; use at least 24x24 pt remove targets even when glyphs are 12 pt. Preserve 4-8 pt spacing around small controls.

13. **Low: Selected timestamps pass contrast with very little safety margin.** (chats, chats-narrow, chats-wide; both). Pillow samples at chats x=313-346,y=246-261 give #62656D on #E4E4E8 = 4.598:1 light and #9C9EA8 on #34353C = 4.574:1 dark. These pass 4.5:1; antialiased edge pixels were not treated as separate text colours. For selected-row metadata use #555861 in light and #B4B6BF in dark, keeping current backgrounds; increase timestamp type to 13 pt. Do not reduce opacity on these labels.

14. **Low: Authentication fallback tooltip is excessively wide.** (signin, signin-expired; both). Tooltip begins around x=609 and ends x=1119 while the card ends x=875; it stays on one line below Use a code instead. This is a legitimate hover state, not a stray permanent element. Wrap tooltip to 300 pt maximum width with 12 pt padding and 13 pt text, aligned below its trigger. Keep 8 pt distance from the trigger.

15. **Low: Fit mode gives no numerical scale or attachment identity.** (image-preview; both). Toolbar shows Fit, 100% and 3072x256 at y=77-105. The image is fitted into approximately 928 px width, but the resulting roughly 30% scale is not displayed and no filename is shown. Show Fit (30%) alongside 100% using 13 pt text; add a single-line filename under the title and keep dimensions secondary at 12 pt.

16. **Low: Shortcut dialog sits too close to the lower window edge.** (shortcuts; both). Dialog spans y=90-788 in an 800 px window, leaving only 12 px below and 90 px above. Cap the dialog at window height minus 48 pt, centre it vertically, and use an internal scrolling body with 24 pt exterior clearance.

## Screen scores

| Screen | Score | Main observations |
| --- | --- | --- |
| chats | 8/10 | Long message lines slow scanning; hover toolbar is visually heavy. Selected timestamps have little contrast margin: 4.60:1 light, 4.57:1 dark. |
| chats-wide | 8/10 | Conversation begins around x=680, leaving a large disconnected gutter after the sidebar. Long lines and a large gap above the composer weaken conversation continuity. |
| chats-narrow | 7/10 | Sidebar consumes 296 of 760 px; many names and previews truncate. Reaction/action toolbar is crowded above the message; timestamps remain very small. |
| chat-no-selection | 8.5/10 | Helpful empty state, but the keyboard jump action is only an inline instruction. Small rail labels and truncated chat previews reduce scanability. |
| chat-filter-no-match | 8.5/10 | Clear recovery action; filter field lacks an adjacent clear affordance. |
| group-chat | 8/10 | Hover toolbar has ten similarly weighted actions. Presence dots in the conversation list distinguish available and busy primarily by colour. |
| channel | 8/10 | Post bodies span long lines; author metadata and threaded replies are visually compressed. Reply buttons and reaction pills are small relative to primary composition controls. |
| channel-narrow | 7.5/10 | Sidebar takes 39% of the window while post text wraps into short lines. Hover toolbar crowds the first post at its top edge. |
| teams | 8.5/10 | Clean empty state; 11-13 pt navigation labels are small for prolonged desktop use. |
| calendar | 8/10 | Detail card emphasizes organiser and location while the actual date/time stays in the small header. Online status in the list is communicated with unexplained blue versus hollow dots. |
| calls | 8/10 | Explanatory copy takes more visual space than the task controls. Audio and video icons in the people list have no visible text labels. |
| calls-narrow | 7.5/10 | Video call icons visible at wide size disappear without a visible overflow action. Names truncate despite substantial unused vertical space. |
| composer-mention | 8/10 | Attachment size and remove control are tiny. Suggestion panel covers the preceding message and has little visual separation in dark mode. |
| composer-long-draft | 7.5/10 | Draft shows lines 111-119 without a visible internal scrollbar or indication of earlier text. Tall draft editor offers no visible expand control. |
| composer-images | 7.5/10 | Wide and tall images are cropped to equal landscape tiles, concealing much of their content. Remove buttons are only about 22 px and image metadata is very small. |
| image-preview | 8/10 | Fit is selected but the actual resulting zoom percentage is absent. Preview has dimensions but no filename or attachment identity. |
| search-switcher | 8.5/10 | Good keyboard guidance and selection cue. Long preview snippets truncate and command input has only placeholder guidance. |
| new-chat | 8/10 | The first search result looks selected while the primary action still targets only Grace Hopper. Picked-person remove icon is tiny. |
| settings | 7.5/10 | Long explanatory paragraphs interrupt the scan of settings. Bottom option is faded at the scroll edge, making its enabled/disabled state unclear. Unchecked control outline passes narrowly at 3.09:1 in dark mode. |
| shortcuts | 8/10 | Tall dialog ends only 12 px above the window bottom. Keycaps use small type; lengthy chord notation makes the left column busy. |
| status-message | 8.5/10 | Focused input and disabled Save are clear. Single-line field and redundant Status label make the dialog slightly mechanical. |
| account-menu | 8/10 | Two accounts share both name and initials; small email text does most of the identification work. Long menu puts account switching and presence controls in one dense column. |
| permission-consent | 8.5/10 | Clear purpose, cancellation and primary action. Important permission scope explanation is demoted to small 12 pt text. |
| error-toast | 8.5/10 | Failure stays attached to the message with a clear Retry action. Recovery text and Retry are small; no specific wait interval is provided. |
| first-sync-loading | 8.5/10 | Clear skeleton and labelled loading state. No visible elapsed-time or recovery guidance if this state persists; screenshot cannot establish timeout behaviour. |
| first-sync-empty | 9/10 | Clear next action and restrained hierarchy; small rail labels are the only notable readability issue. |
| call-bar | 8/10 | Three outlined actions form a crowded second row at 380 px. Restore and Hide call bar do not explain how to return to full navigation. |
| call-docked | 8.5/10 | Meeting context and composer remain clear. Restore is a vague label for returning to the main window. |
| join-on-phone | 8.5/10 | QR code is clear in both themes. Copy the link sits far left under a centred QR code; minor alignment inconsistency. |
| setup | 7.5/10 | Registration, client ID, tenant and environment variables create a dense first-run form. Prerequisite steps are prose rather than a short visible sequence. |
| signin | 8.5/10 | Primary sign-in action is clear. Long single-line tooltip extends far outside the card. |
| signin-browser | 8.5/10 | Good progress explanation and recovery actions. Footer legal text spans an unnecessarily long line in small type. |
| signin-device-code | 9/10 | Code and copy action are excellent; code expiry is not visible. |
| signin-finishing | 8.5/10 | Clear finishing state. No visible escape or recovery action if completion stalls; actual timeout behaviour is unknown. |
| signin-expired | 8.5/10 | Explains expiry and identifies the account. Wide tooltip visually overhangs the card. |
| signin-failed | 8.5/10 | Clear reason with retry and alternative authentication. Error state does not distinguish a transient timeout from a connectivity problem. |

Interaction behaviour beyond the screenshots was not inferred. Missing visible recovery controls or overflow choices describe these shots only. All proposed dimensions are in points; evidence coordinates refer to original image pixels.
