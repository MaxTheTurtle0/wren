# Independent screenshot design review B

**Overall: 7.5/10.** Reviewed all 36 screens in both themes, 72 images total, including the actual 380×800 call-bar and 720×800 docked layouts. Read only the supplied screen manifest and PNGs. No implementation or interaction behaviour was inspected. Hover toolbars were treated as expected hover states, not persistent UI defects.

Wren has a calm, coherent native layout, strong body-text readability, consistent iconography and clear loading, empty and sign-in states. It falls short of excellent because narrow layouts preserve too much navigation, the error toast hides recovery, call status is ambiguous, and several task surfaces devote excessive space to explanation. No screen looked fundamentally broken; the failed-send view is the weakest at 6.5.

Dimensions: hierarchy 8; spacing 7.5; typography 7.5; consistency 8; contrast/accessibility 8; feedback 7; states 8; navigation 7.5.

## Contrast measurements

Used `python3 -I` and Pillow to sample real RGB pixels. Ratios use WCAG 2.2 sRGB relative luminance and `(Llighter + .05)/(Ldarker + .05)`. Foregrounds below are full-coverage glyph pixels, not blended antialias fringes. Antialias edges were not incorrectly treated as nominal text-colour failures. Checked rendered text palette presence in every image, with targeted local samples of body text, metadata, help, selected rows, status labels and errors. This is screenshot evidence, not a complete WCAG conformance audit: keyboard operation, semantic labels, animation and hidden states cannot be established.

| Local sample | Light | Dark |
| --- | --- | --- |
| Chat body, x432–850/y199–216 | #1b1c1f on #ffffff: 17.04:1 | #e9e9ec on #17181b: 14.65:1 |
| Chat timestamp, x530–567/y177–191 | #62656d on #ffffff: 5.83:1 | #9c9ea8 on #17181b: 6.65:1 |
| Selected timestamp, x312–347/y246–258 | #62656d on #e4e4e8: 4.60:1 | #9c9ea8 on #34353c: 4.57:1 |
| Mention suggestion email, x438–590/y645–657 | #62656d on #e4e4e8: 4.60:1 | #9c9ea8 on #34353c: 4.57:1 |
| Settings explanatory text, x382–865/y430–472 | #62656d on #ffffff: 5.83:1 | #9c9ea8 on #25262b: 5.66:1 |
| Sign-in failure, x454–672/y319–334 | #c42b2b on #ffffff: 5.63:1 | #f27474 on #25262b: 5.41:1 |
| Calls Works label, x520–556/y379–393 | #1b7a42 on #ffffff: 5.37:1 | #4cc36e on #17181b: 7.89:1 |
| Clear button boundary, x516–569/y276–308 | #dfdfe3 on #ffffff: 1.33:1 | #303137 on #25262b: 1.16:1 |
| Composer/checkbox boundary | #8a8d95 on #ffffff: 3.32:1 | #6e7179 on #25262b: 3.09:1 |

No sampled full-coverage text pair failed 4.5:1, so no text-contrast score cap was triggered. Selected-row metadata passes narrowly. Neutral button boundaries fall below the requested 3:1 target; readable labels can identify some of these controls without a boundary, so the measurements alone do not prove every such button violates WCAG. Small typography remains a readability issue even when colour ratios pass.

## Ordered fixes

1. **High: Error toast obscures the failed message and its recovery action.** (error-toast; both). Toast occupies approximately x393–887, y644–692; failed-message status and Retry sit immediately beneath it around y690. Place the toast above the failed-message row with a 16-point gap, or reserve a 48-point notification region above the composer; keep the entire Retry button visible.

2. **Medium: Fixed navigation consumes too much of a narrow window and truncates identities.** (chats-narrow, channel-narrow, calls-narrow; both). At 760×640, the rail plus list ends at x296, consuming 39% of width. Chat names such as Katherine Johnson and several Calls names are ellipsized. Below 900 points, provide a collapsible 200-point list or a list/detail transition with a 32-point Back control; preserve at least 440 points for conversation content.

3. **Medium: Live call status conflicts with the visible ended-meeting state.** (call-bar, call-docked; both). Top band y14–44 says In the meeting / just joined, while the final event says Meeting ended at y476 in call-bar and y416 in call-docked. If these refer to separate sessions, label the separator Previous meeting in 12-point semibold and date the ended event; otherwise replace the top status with Meeting ended and a Rejoin action.

4. **Medium: The first-run hierarchy foregrounds technical configuration over the step that makes it possible.** (setup; both). Card x405–875, y262–642 contains GUID and tenant fields; How to register Wren appears at y555 beside disabled Save, after both fields. Move a primary Set up app registration action above the fields, use a 3-step list in 14-point text, and put environment-variable instructions behind an Advanced disclosure.

5. **Medium: Calls screen reads like a capability document instead of an action surface.** (calls, calls-narrow; both). At 1280×800, explanatory sections occupy roughly y238–748; at 760×640, they continue below the viewport after Call status. Keep Join and recent people first; reduce explanation to two 14-point lines and put the capability list under a How calls work disclosure with 16-point section spacing.

6. **Medium: Long draft text touches the top clipping edge and reads as an unstructured block.** (composer-long-draft; both). Composer starts at y550; first visible line is cut at y557, and successive lines are approximately 17 pixels apart. Use 12-point internal padding, 14-point draft text with 20-point line spacing, and reveal the caret line with at least 8 points above and below it.

7. **Medium: Technical explanation interrupts behavioural settings and uses a repository-path reference in product copy.** (settings; both). Paragraph at x382–865, y432–471 discusses polling, a push relay and docs/realtime.md between checkbox rows. Replace with a two-line 13-point explanation of when messages refresh; put connection setup behind a labelled Configure instant alerts action with a 16-point gap.

8. **Medium: Persistent formatting help and some metadata are too small for comfortable reading, despite passing sampled colour contrast.** (chats, chats-wide, chats-narrow, group-chat, channel, channel-narrow, composer-mention, composer-long-draft, composer-images, call-bar, call-docked; both). Composer-help glyphs occupy about y777–785 at 1280×800; in 380-pixel call-bar the same help wraps across y763–784. Measured helper foreground #62656d on #ffffff is 5.83:1; dark #9c9ea8 on #17181b is 6.65:1. Use 12-point help text; show only Enter to send · Shift+Enter for a new line by default and move formatting syntax into a help popover. Use 12-point attachment and mention metadata.

9. **Medium: Cropped thumbnails make it difficult to identify the exact image being sent.** (composer-images; both). Three previews occupy x393–841, y622–718; first image shows partial birds and the third contains only clipped fragments of the image. Fit each full image within a 144×96-point tile with letterboxing; add a 12-point filename or dimensions label and keep remove controls at least 24×24 points.

10. **Medium: Reading and writing regions use mismatched widths.** (chats-wide; both). Message cards and date rules stop near x1300, leaving over 600 pixels to the right, while composer spans x380–1904. Cap the composer at the same 920-point width as the message column, align their left edges, and anchor message actions to that shared column.

11. **Low: Neutral selection fills are subtle and similar to hover fills.** (chats, chats-narrow, chat-no-selection, group-chat, channel, channel-narrow, calendar, search-switcher; both). Chat selected fill #e4e4e8 against sidebar #f6f6f7 is approximately 1.17:1; dark selected fill #34353c against #1d1e22 is approximately 1.36:1. Orange active icons in rail and channel views help, but the selected chat lacks an equivalent marker. Add a persistent 3-point leading selection marker in #a9530f light / #e88c3e dark; retain the neutral fill for breadth. Do not rely on fill contrast alone.

12. **Low: Secondary button outlines provide very little visual separation from their surfaces.** (status-message, join-on-phone, settings, signin, signin-browser, signin-device-code, signin-expired, signin-failed; both). Sampled Clear boundary #dfdfe3 on #ffffff is 1.33:1; dark #303137 on #25262b is 1.16:1. Same values sampled on Copy the link and Leave demo. Labels themselves remain legible, so this is not a demonstrated text-contrast failure. Where an outline communicates the hit area, use #8a8d95 light and #6e7179 dark, measured at 3.32:1 and 3.09:1 against the respective surfaces; keep 32-point minimum button height.

13. **Low: Save versus Clear is unclear with an empty-looking status field.** (status-message; both). Placeholder is visible in the field at y208–258, while both filled Save and outlined Clear remain available at y277–308. Disable Save until the draft differs from the current status; disable Clear when no status exists, or label the destructive action Remove status. Maintain a 32-point action height and 8-point spacing.

14. **Low: Zoom toolbar does not explicitly distinguish the active fit mode from actual image scale.** (image-preview; both). Fit is tinted at x176–211; 100% appears beside it, while the displayed 3072-pixel-wide image fits a roughly 928-pixel-wide viewport. The screenshot cannot establish whether 100% is a command or a readout. Label the command Actual size (100%), show an explicit Zoom: 30% readout for this fit, and indicate Fit selection with the same selected-control treatment used in Settings.

## Screen grades

| Screen | Score | Observations |
| --- | --- | --- |
| chats | 8 | Formatting help is very small and competes with composer space. Selected chat relies on a subtle neutral fill. |
| chats-wide | 7.5 | Message column stops around x1300 while composer extends to x1904, disconnecting reading and writing widths. Large unused right region makes message actions feel detached from the window edge. |
| chats-narrow | 7 | Rail and list consume 296 of 760 pixels; multiple names are truncated. Composer help is tiny and dense across the bottom. |
| chat-no-selection | 8.5 | Clear empty state, but the instruction offers a keyboard shortcut without an inline new-chat action. |
| chat-filter-no-match | 8.5 | Clear recovery action; open conversation remains visible while its selected list item disappears, weakening location context. |
| group-chat | 8 | Mention highlighting is clear. Long lines approach the full pane width; formatting help remains undersized. |
| channel | 8 | Reply affordances are small text with little separation from thread content. Nested replies use very small timestamp and author details. |
| channel-narrow | 7 | 296-pixel navigation leaves a narrow thread column. Long contextual composer placeholder wraps and crowds the entry area. |
| teams | 8.5 | Clear hierarchy and channel nesting; empty pane offers only a passive instruction. |
| calendar | 8 | Join is a small 28-pixel-high pill despite being the primary task. Meeting detail leaves considerable space unused while supporting text is small. |
| calls | 7.5 | Capability documentation dominates the page beneath the join input. Repeated explanatory text makes it harder to scan actual call actions. |
| calls-narrow | 7 | Recent-person names truncate despite the large fixed navigation region. Capability explanation extends below the viewport; join and call tasks compete with prose. |
| composer-mention | 8 | Suggestion email and attachment size use very small metadata text. Attachment remove affordance is a tiny unboxed cross. |
| composer-long-draft | 7.5 | First visible draft line is clipped against the top boundary. Dense draft text has little vertical breathing room; scrollbar is separated from the right edge by the action area. |
| composer-images | 7.5 | Thumbnails crop the pasted images heavily, especially the third image. No filename, dimensions, or explicit pending-image count is shown. |
| image-preview | 8 | Fit and 100% are presented without a clear current zoom readout. Preview toolbar could distinguish zoom actions from the active scale more explicitly. |
| search-switcher | 8.5 | Good keyboard instructions and clear result hierarchy. Long preview snippets are frequently truncated. |
| new-chat | 8 | Search results remain prominent while the footer action targets the already-picked person. Existing-chat explanation is tiny and placed beside the action. |
| settings | 7.5 | Long implementation-oriented behaviour explanation interrupts the settings flow. The bottom option is faded and scrollbar continuation is subtle, especially in dark mode. |
| shortcuts | 8.5 | Readable consistent rows; category grouping would make the long list quicker to scan. |
| status-message | 8 | Save and Clear are simultaneously available with an empty-looking field, making the distinction unclear. Secondary action boundaries are visually weak. |
| account-menu | 8 | Account switching, presence, status message and settings form a long mixed-purpose menu. Secondary identity information is small. |
| permission-consent | 8 | Reason and browser handoff are clear. API permission terminology makes the explanation more technical than necessary. |
| error-toast | 6.5 | Toast covers the failed message and most of its Retry control. Error feedback and recovery are spatially in conflict. |
| first-sync-loading | 8.5 | Clear loading label and skeletons. Three simultaneous loading treatments are visually redundant: header spinner, skeleton list and centre spinner. |
| first-sync-empty | 8.5 | Clear first-chat action. Empty sidebar retains an inactive-looking filter above a redundant no-chats label. |
| call-bar | 7 | Top says In the meeting / just joined while history says Meeting ended. Meeting header truncates even though the full title is available directly above. Formatting help wraps into two very small lines. |
| call-docked | 7.5 | Top live-call status conflicts with the visible meeting-ended event. Meeting title is repeated in adjacent header bands. |
| join-on-phone | 8.5 | QR is large and high contrast with useful copy-link fallback. Copy-link secondary button has a faint boundary. |
| setup | 7 | First-run form asks for app registration, tenant and GUID concepts before the main task. Registration help is visually secondary to a disabled Save button. Environment-variable instructions add technical detail to the primary setup surface. |
| signin | 8.5 | Clear primary action and fallback. Legal footer is an excessively long single line at a small size. |
| signin-browser | 8.5 | Clear waiting state and recovery actions. Legal footer remains much wider than the sign-in card. |
| signin-device-code | 8.5 | Code and copy action are clear. No expiry time is visible for the temporary code. |
| signin-finishing | 8.5 | Clear busy state. No visible recovery affordance if finishing stalls; actual timeout behaviour cannot be judged from the shot. |
| signin-expired | 8.5 | Account and reason are clear. App-registration disclosure and demo option create secondary distractions in account recovery. |
| signin-failed | 8.5 | Clear reason, retry and alternate sign-in. Failure has no further troubleshooting link beyond app-registration disclosure. |
