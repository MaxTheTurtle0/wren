# Independent screenshot design review, round 15 B

Overall: **8/10**. Contrast **8.5**, typography **8**, colour **8.5**, spacing **7.5**, hierarchy **8**.

Reviewed all 36 screens in both themes, 72 source PNGs. Read only the supplied screen manifest and image inputs. This is a screenshot review: behavior, keyboard access, animation and actual hit areas are not verified. Hover toolbars and sign-in tooltips were treated as transient states, not persistent layout defects.

The app is calm, coherent and readable. Inter, restrained orange accents, selected-row markers, focused inputs, empty states and inline retry feedback form a credible desktop system. Light and dark modes largely preserve hierarchy and meaning.

The three main blockers to a 9 are narrow-window allocation, small secondary type, and uneven spatial rhythm. At 760 points navigation consumes 296 points, leaving names and content unnecessarily compressed. Metadata and help repeatedly sit around 11-12 points. Docked chat has a large blank opening area, and the brand shifts about 78 pixels between sign-in states. Extreme-aspect attachment previews, the unfinished-recipient state and dense registration instructions also need refinement.

## Pixel contrast checks

Measured with `python3 -I` and Pillow using sRGB linearization and WCAG relative luminance. Samples use solid glyph interiors and adjacent flat backgrounds, not antialiased edge pixels. No sampled real-text pair failed AA, so no contrast-based screen cap was applied. This sampling is not an exhaustive WCAG 2.2 conformance audit. Disabled controls and text behind modal scrims were excluded from failure claims.

| Visible element | Light | Dark |
| --- | --- | --- |
| Chat timestamp | #62656D / #FFFFFF, 5.83:1 | #9C9EA8 / #1A1B1F, 6.45:1 |
| Selected chat preview | #4A4D55 / #E2E3E8, 6.60:1 | #B4B6BF / #34353C, 6.04:1 |
| Quote text | #4A4D55 / #E2E3E8, 6.60:1 | #B4B6BF / #2C2D33, 6.79:1 |
| Mention text | #B4530A / #FDF1E5, 4.52:1 | #E88C3E / #2B2219, 6.13:1 |
| Send-error explanation | #B42525 / #F9EEEE, 5.72:1 | #F58A8A / #2C2428, 6.39:1 |
| Device-login link | #1F63C6 / #FFFFFF, 5.74:1 | #7CB4F5 / #26272C, 6.89:1 |
| Composer outline against interior | #7C7F87 / #FFFFFF, 4.01:1 | #82858E / #26272C, 4.04:1 |
| Thread grouping border | #B9BCC4 / #FFFFFF, 1.90:1 | #4D4F58 / #26272C, 1.83:1 |

Text is judged against 4.5:1; large text and meaningful component/icon boundaries against 3:1. Thread grouping borders are weak, but are not asserted to be an essential-control failure. The mention sample is a genuine pass with little margin. Timestamp pixels were sampled at (545,154) against (540,151) in chats; light mention at (446,428) against (442,426) in group-chat; dark thread border at (390,280) against (391,280) in channel.

## Per-screen scores

- **chats: 8**. Secondary text and reaction labels are small for sustained desktop reading. Quoted content is compressed into a small, flattened summary.
- **chats-wide: 8**. The centered conversation leaves a large gutter between navigation and messages. Small timestamps and helper text remain unchanged despite the larger canvas.
- **chats-narrow: 7.5**. The sidebar takes 39% of the window and truncates many names and nearly every preview. The message column wraps heavily and quoted content becomes dense.
- **chat-no-selection: 8.5**. The empty-state instruction is small relative to the available space.
- **chat-filter-no-match: 8**. No inline clear affordance beside the query; recovery is far down the sidebar. The empty-state sentence occupies almost the full sidebar width.
- **group-chat: 8**. Small timestamps and system messages require close reading. Light mention text has only 4.52:1 contrast, leaving little margin.
- **channel: 8**. Thread borders are faint, especially in dark mode. Reply metadata is smaller than the chat metadata and makes a dense secondary tier.
- **channel-narrow: 7.5**. Large sidebar and nested card padding leave a short text measure. The long composer prompt uses almost all the space preceding its action icons.
- **teams: 8.5**. Small channel labels and empty-state instruction underuse the available reading space.
- **calendar: 8**. Long explanatory copy stretches broadly across a mostly empty detail pane. Date labels and meeting metadata are small relative to the meeting title.
- **calls: 8**. The page gives explanatory prose more visual space than the calling actions. Recent-person audio and video controls are small and tightly paired.
- **calls-narrow: 7.5**. Names truncate while the sidebar retains two controls per person. Explanatory text becomes a long block at the narrow measure.
- **composer-mention: 8**. Mention suggestion email, insertion hint and attachment metadata create several small-text tiers. Small attachment remove control is visually slight.
- **composer-long-draft: 8**. No visible line-count or extent cue for a 120-line draft beyond a short scrollbar. Internal scrollbar and action icons share a cramped bottom-right zone.
- **composer-images: 7.5**. Extreme-aspect previews collapse into thin strips that cannot be identified. Image dimensions are small and low in hierarchy.
- **image-preview: 8.5**. Fit/100% selection relies on a very short accent underline.
- **search-switcher: 8.5**. Long result previews truncate heavily. The footer stacks two small explanatory tiers.
- **new-chat: 8**. The primary button names the selected recipient while an unfinished second-recipient query remains active. Search-result email addresses are small.
- **settings: 8**. Small help text and uppercase labels create a dense secondary text layer. The panel ends immediately below a checkbox with limited contextual preview of further settings.
- **shortcuts: 8.5**. The navigation key-to-description gap is substantially larger than the messages gap.
- **status-message: 8.5**. The single-line field gives little visible room for a longer status.
- **account-menu: 8**. The account and presence list is tall and text-heavy for a rail popover. Available and Busy use the same solid-dot shape with colour carrying the distinction.
- **permission-consent: 8.5**. The technical permission detail is small and wraps its last words onto a short second line.
- **error-toast: 8.5**. The critical failure explanation is only about 12 points. Retry is a small neutral button beside the red error sentence.
- **first-sync-loading: 8.5**. The loading explanation is small and offers little detail beyond an indeterminate state.
- **first-sync-empty: 8.5**. The small instruction and button feel modest within the very large empty pane.
- **call-bar: 8**. Phone, Expand and Hide are abbreviated compared with the wider docked version. Several system/date rows consume a substantial part of the narrow conversation.
- **call-docked: 8**. About 170 pixels of blank space separate the header and conversation-start label. System messages are separated by large gaps relative to actual chat messages.
- **join-on-phone: 8.5**. Copy link is visually distant from the QR and explanatory text.
- **setup: 7.5**. Dense registration terminology makes the first-run card feel administrative. Tenant helper copy separates its label and input by a large vertical distance.
- **signin: 8.5**. The hover tooltip covers the lower card, a transient state rather than a permanent layout defect. The brand/card position varies noticeably across sign-in states.
- **signin-browser: 8.5**. Brand and card shift down relative to the initial sign-in screen.
- **signin-device-code: 8.5**. The waiting state has no visible code lifetime or expiry cue.
- **signin-finishing: 8**. Brand and card move again when the card becomes shorter. The pending state has no visible recovery affordance if it persists.
- **signin-expired: 8.5**. The hover tooltip covers lower card content. Account metadata is visually small compared with the brand.
- **signin-failed: 8.5**. The error identifies a timeout but gives little guidance for choosing the alternate code flow.

## Prioritized changes

1. **spacing, medium:** Narrow navigation consumes too much of the available width, forcing avoidable truncation and wrapping. Below 900 points, offer a 200-point list plus the 64-point rail, or collapse the list while reading. Hide row-level call controls until hover/focus and reserve at least 140 points for names.
2. **typography, medium:** Secondary information is consistently smaller than comfortable sustained reading size. Use 13-point secondary text and 14-point recovery/error text with 18-20-point line height. Keep 11-point uppercase labels only for short, infrequent section labels.
3. **spacing, medium:** Conversation-start and system-message spacing creates disconnected islands of content. Limit top conversation padding to 32 points, use 16 points around date dividers, and group related meeting-status rows with 8-point internal spacing. Preserve bottom anchoring for the latest content.
4. **spacing, medium:** Sign-in state changes move the brand and main task vertically. Anchor the brand to a common y position and the card top to a common baseline, e.g. brand top 128 and card top 284 points. Allow the card bottom to grow with content.
5. **hierarchy, medium:** Extreme-aspect attachments are technically fitted but visually unrecognizable. Use a centered cropped thumbnail with a 64-point minimum recognizable image area and an aspect-ratio badge; retain full aspect ratio in the image preview. Keep dimensions at 13 points beneath it.
6. **hierarchy, medium:** The submit action does not clearly resolve the in-progress recipient search. Keep a stable "Start chat" action and show "1 recipient selected" next to it at 13 points. Explicitly separate pending search results from the selected-recipient area with 16-point spacing.
7. **hierarchy, medium:** First-run registration gives technical configuration too much immediate reading weight. Present two numbered groups, "1. Create the registration" and "2. Paste your application ID". Keep 8 points between labels and fields, move tenant guidance below the field, and use a 13-point single-line explanation with expandable detail.
8. **contrast, low:** Thread boundaries are visually weak, reducing the clarity of stacked conversations. If the border carries thread separation, use #7C7F87 in light and #82858E in dark for at least 3:1, or replace the boxes with stronger spacing and a clear thread heading.
9. **contrast, low:** Mention text passes AA with almost no margin. Use #994300 for mention text in the light theme, retaining the pale fill; this provides more comfortable contrast without increasing saturation across the whole UI.
10. **hierarchy, low:** Explanatory prose has too much visual weight relative to the primary task. Keep one 13-point, two-line maximum browser-handoff explanation directly under the primary action. Move the remaining details into the existing Details disclosure and limit explanatory copy to 520 points wide.
11. **colour, low:** Presence distinctions depend heavily on colour at very small sizes. Give Busy a distinct minus or filled-square mark and retain Available as a check/dot. Use a consistent 14-point status symbol in menus and a contrasting outline around avatar badges.
12. **spacing, low:** Keyboard shortcut columns use noticeably different key-to-description gaps. Use a shared 136-point key column and a 16-point gutter in both groups, retaining equal row heights and a 32-point inter-group gap.
13. **other, low:** Clearing a short failed filter requires looking far from the input. Add a 24x24-point clear button at the right edge of the filter, with a 3:1 icon, while retaining the central recovery action.
14. **other, low:** Pending authentication screens provide limited visible information about waiting and recovery. Show code expiry beneath the code at 13 points, and provide a low-emphasis Cancel action during finishing. Preserve the same card position and primary hierarchy.

All measurements are screenshot pixels; proposed layout/font values are points. Font sizes inferred from screenshots are approximate. No files other than grade.json and grade.md were written.
