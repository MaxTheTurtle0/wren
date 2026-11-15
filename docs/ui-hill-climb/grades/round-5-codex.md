# Independent screenshot design review

Overall: **8.0/10**. Wren is calm, readable and visually consistent across the two themes. Empty, loading, authentication and send-failure states are generally clear. It is good, with visible layout and interaction issues that keep it below an excellent native-app standard.

Reviewed all 36 screen pairs, 72 images, using only the supplied manifest and screenshots. Scores consider both modes. The actual call-bar and join-on-phone captures are 380x800; call-docked is 720x800. Hovered action strips and sign-in tooltips were treated as hover states. Screenshots cannot establish keyboard operation, screen-reader semantics, actual pointer target sizes or motion behaviour.

## Main issues

1. **high: Long draft hides the send and attachment controls.** Composer spans x=380..1263, y=553..761. Its lower-right corner contains a scrollbar, while the paperclip and send icons visible in ordinary composers are absent. The first visible line is also cut at y=561. Suggested change: Keep a fixed 36 px action footer below a separately scrolling draft editor; use 28x28 px attachment and send buttons with 8 px spacing and 12 px outer padding. Keep the editor viewport at a maximum of 160 px high.

2. **medium: Narrow layout preserves too much navigation width and truncates important identities.** At 760 px width, the rail plus sidebar consumes x=0..296, leaving 464 px for the active pane. Calls shows names such as 'Katherine ...', 'Grace Hop...' and 'Margaret ...'; chat names also truncate. Suggested change: At widths below 900 px, offer a 40 px collapse control and reduce the list pane to 200 px excluding the 64 px rail. In Calls, reserve at least 140 px for names and move call actions into a trailing 56 px cluster.

3. **medium: Attachment crops make extreme-aspect images hard to recognize.** Three roughly 144x96 px thumbnails at x=394,546,698 show equal landscape crops, although their labels are 3072x256, 256x256 and 256x3072. The tall image shows disconnected fragments rather than its full extent. Suggested change: Fit each entire image inside a 144x96 px frame using contain scaling; use #EEEEF0 in light mode and #2C2D33 in dark mode for letterboxing. Add a 12 pt filename below each frame and keep dimensions secondary.

4. **medium: Wide conversation is left-heavy and its header does not align with its content column.** At 1920x1080, the sidebar ends at x=364, the conversation and composer end around x=1308, and about 612 px remain empty to the right. Header actions remain near the far window edge. Suggested change: Center a maximum 960 px conversation column within the available pane, then align the header title, actions and composer to that column with 16 px horizontal insets.

5. **medium: Technical setup lacks a strong guided action hierarchy.** The 470 px-wide card at x=405..875 includes registration explanation, a text-style guide at y=353, GUID and tenant fields, generic Save, and two technical helper lines. The person must infer that the guide precedes the form. Suggested change: Use two numbered groups, '1. Register Wren' and '2. Connect your account', in 15 pt semibold with 16 px gaps. Promote the guide to a 32 px-high outlined button. Rename Save to 'Save and continue', and move environment-variable instructions into a disclosure.

6. **medium: Dense small supporting copy weakens scanning.** Settings behaviour guidance occupies three closely spaced lines at y=436..472; permission details occupy y=265..285; switcher help is packed into y=617..650. Their glyphs are approximately 12 px high at 1x. Suggested change: Use 13 pt supporting text with 18 px line height and at least 8 px separation from controls. Reduce settings guidance to two short sentences and put relay explanation behind a labeled 'Instant alerts' disclosure. Preserve #62656D on white and #9C9EA8 on #25262B, which measure 5.83:1 and 5.66:1.

7. **medium: Code-sign-in hover explanation is oversized and exposes configuration jargon.** The visible tooltip at approximately x=609..1118, y=458..506 crosses the right edge of the x=405..875 sign-in card and obscures the App registration area. It mentions 'Allow public client flows'. This is a captured hover state, not evidence of a permanent overlay. Suggested change: Limit the tooltip to 300 px width with 12 px padding, 13 pt text and 18 px line height. Use 'Use a code if browser sign-in cannot return to Wren.' Place configuration instructions inside App registration.

8. **medium: Keyboard result highlight competes with committed recipient selection.** Grace Hopper is selected in a chip at y=158..189 and in the orange 'Chat with Grace Hopper' button. Barbara Liskov simultaneously receives a full-width gray result highlight at y=269..313 without a visible cue explaining the difference. Suggested change: Mark committed recipients with a check icon and accent chip outline. Give the active search result a 2 px focus outline, #A9530F in light mode and #E88C3E in dark mode, rather than the same full-row selection fill. Add 'Enter to add' at 12 pt in the result footer.

9. **medium: Duplicated meeting identity consumes vertical space in the docked conversation.** The meeting name appears around y=20 and y=113, separated by the call-control row. Header content consumes 148 of 800 px even in the 380 px-wide view. Suggested change: Consolidate meeting identity and status into one 52 px block, retain a 36 px controls row, and start messages below y=100. Rename Restore to 'Expand Wren' and Close bar to 'Hide call bar' where space permits; use concise tooltips at 380 px.

10. **low: Image preview lacks attachment identity.** The header says only 'Image preview' at x=176,y=55 and dimensions 3072x256 at x=306,y=92. No filename or source message identifies which attachment is open. Suggested change: Show the filename in 15 pt semibold beside the heading and dimensions in 12 pt secondary text. If no filename exists, show 'Image from Ada Lovelace' or an equivalent known sender label; reserve 40 px for close.

11. **low: Account menu is tall and gives many secondary presence actions equal visual weight.** Popup spans roughly x=56..337, y=304..788, with two accounts, Add account, seven presence/reset rows, status message and settings. Presence rows are spaced about 30 px apart and occupy most of the popup. Suggested change: Keep account switching visible, then show current presence as a 36 px row opening a presence submenu. Keep status message and settings in a separate group with 8 px vertical separation. Target a primary popup height below 320 px.

12. **low: Global error toast obscures conversation content despite an already clear inline failure.** Toast occupies x=575..1069, y=64..112 and covers part of the first message. The failed message has its own tinted surface, 'Not sent' label and Retry at y=605..700. Suggested change: Put the global error in a reserved 44 px banner beneath the conversation header, shifting content below it, or show the rate-limit explanation beside the inline Retry. Keep 12 px padding and a 28x28 px dismiss button.

## Contrast measurements

Pixels were sampled with Pillow under python3 -I. Ratios use WCAG sRGB linearization and relative luminance. Solid glyph-core colours were used, not partially antialiased edge pixels. Active foreground text was sampled in every image; dimmed content behind modal scrims was excluded. No sampled body-text failure was found, so no contrast score cap was applied. This is a screenshot sample audit, not a claim of complete WCAG 2.2 conformance.

| Visible element | Light foreground / background | Ratio | Dark foreground / background | Ratio |
| --- | --- | ---: | --- | ---: |
| Chat body, x432..700 y194..212 | #1B1C1F / #FFFFFF | 17.04 | #E9E9EC / #17181B | 14.65 |
| Dialog body | #1B1C1F / #FFFFFF | 17.04 | #E9E9EC / #25262B | 12.46 |
| Composer placeholder, x391..700 y727..749 | #62656D / #FFFFFF | 5.83 | #9C9EA8 / #25262B | 5.66 |
| Chat timestamp, x530..566 y174..188 | #62656D / #FFFFFF | 5.83 | #9C9EA8 / #17181B | 6.65 |
| Calendar supporting label | #62656D / #F6F6F7 | 5.40 | #9C9EA8 / #1D1E22 | 6.24 |
| Chat link, x772..1005 y677..696 | #1F63C6 / #FFFFFF | 5.74 | #7CB4F5 / #17181B | 8.20 |
| Sign-in error, x452..730 y358..375 | #C42B2B / #FFFFFF | 5.63 | #F27474 / #25262B | 5.41 |
| Inline Not sent, x452..502 y677..693 | #C42B2B / #FBEEEE | 4.98 | #F27474 / #281F22 | 5.75 |
| Primary Try again button | #FFFFFF / #A9530F | 5.34 | #1C1208 / #E88C3E | 7.24 |
| Composer border, x600..800 y715..721 | #8A8D95 / #FFFFFF | 3.32 | #6E7179 / #25262B | 3.09 |
| Unchecked settings checkbox, x382..399 y399..416 | #8A8D95 / #FFFFFF | 3.32 | #6E7179 / #25262B | 3.09 |

Body and small text require 4.5:1; large text and meaningful non-text controls require 3:1. These samples pass. Dark input strokes pass with a small margin; preserve their solid stroke colour when changing surfaces.

## Dimensions

- hierarchy: 8/10
- spacing: 7.5/10
- typography: 8/10
- consistency: 8.5/10
- contrast_accessibility: 8.5/10
- feedback: 7.5/10
- states: 8.5/10
- navigation: 8/10

## Screen scores

| Screen | Score | Visible issues |
| --- | ---: | --- |
| chats | 8 | Hover action strip gives reactions more prominence than reply; secondary timestamps and composer help are small. |
| chats-wide | 7.5 | Conversation column ends near x=1308 while its header spans 1920 px, leaving about 612 px unused on the right. Composer is visually detached from the full-width header. |
| chats-narrow | 7.5 | Persistent 296 px navigation area leaves only 464 px for conversation. Names and previews truncate heavily; hover toolbar competes with author row. |
| chat-no-selection | 8.5 | Clear empty-state guidance, but starting a chat remains an icon-only header action. |
| chat-filter-no-match | 8.5 | Clear recovery action; filter has no inline clear affordance, requiring the separated empty-state button. |
| group-chat | 8 | Member count is visually plain and offers weak wayfinding to the people in the group. Hover toolbar has many equally weighted emoji actions. |
| channel | 8 | Reply buttons repeat without a visible reply count; parent/reply hierarchy depends mainly on avatar size and indentation. |
| channel-narrow | 7.5 | Navigation uses 296 of 760 px. Composer destination text runs close to attachment controls; post action toolbar crowds the author row. |
| teams | 8.5 | Clear no-selection state; long team/channel lists have no visible filter or jump affordance within this pane. |
| calendar | 8 | Meeting detail and supporting explanation use very different line lengths. Large detail panel has little hierarchy between organizer, location, online type and start time. |
| calls | 8 | Explanatory copy dominates the content pane while calling actions are small unlabeled icons in the sidebar. |
| calls-narrow | 7.5 | Recent-person names are truncated even when the main pane has extensive empty space. Long explanatory copy wraps into a dense paragraph. |
| composer-mention | 8 | Suggestion popup has no visible keyboard acceptance hint. Attachment removal glyph is very small relative to the chip. |
| composer-long-draft | 7 | Long draft occupies x=380..1263, y=553..761 and exposes no send or attach controls. Top visible draft line is partially clipped; no visible draft length or position cue beyond the scrollbar. |
| composer-images | 7.5 | Uniform landscape crops obscure the contents of both the very wide and very tall images. Dimensions alone are weak attachment identification; removal glyphs are small. |
| image-preview | 8 | Preview heading is generic and shows dimensions without a filename or attachment identity. Fit canvas retains extensive empty space around a very wide image. |
| search-switcher | 8.5 | Long recent-message snippets are cut off; secondary text and bottom help are visually dense. Recent result types are not explicitly labeled. |
| new-chat | 7.5 | Barbara's full-row highlight competes with Grace's selected-recipient chip while the primary button still names Grace. Helper sentence sits tightly beside the primary action. |
| settings | 7.5 | Long 12 px explanatory copy makes the behaviour section dense. Lower call settings reach the bottom of the viewport, with more settings hidden by scrolling. |
| shortcuts | 8.5 | Long key labels and descriptions nearly fill both columns; section labels are small. |
| status-message | 8.5 | Concise dialog, but the text field relies on an example placeholder rather than a persistent field label. |
| account-menu | 8 | Popup is approximately 484 px tall and mixes account management, presence and application settings. Several presence options differ only by small icon details and adjacent wording. |
| permission-consent | 8 | Generic 'One more permission' heading makes the feature purpose secondary. Permission details use small print across two lines. |
| error-toast | 8 | Toast at x=575..1069, y=64..112 covers the first visible message. Toast and inline failed-message treatment compete for attention, although retry is clear. |
| first-sync-loading | 8.5 | Loading state is clear; no visible stage or amount of work distinguishes first sync from a slow connection. |
| first-sync-empty | 8.5 | Good first-chat CTA; empty sidebar and repeated new-chat header affordance create slight redundancy. |
| call-bar | 8 | Call status plus repeated meeting title occupy the top 148 px in a 380 px-wide window. Restore and Close bar labels require interpretation. |
| call-docked | 8 | Same 148 px stacked call header persists at 720 px width. Meeting name is duplicated; call-state controls are separated from the meeting header. |
| join-on-phone | 8.5 | QR is legible with a quiet zone; dialog does not repeat the meeting name, reducing confirmation context. |
| setup | 7.5 | Setup requires technical app-registration decisions but the guide looks subordinate to explanatory text. Save is a generic action label; 12 px technical helper text and environment variables add density. |
| signin | 8 | Hovered code-sign-in tooltip covers the registration area and stretches outside the card. Tooltip exposes public-client-flow configuration jargon at the point of choosing a sign-in method. |
| signin-browser | 8.5 | Clear waiting state and recovery actions; browser page identity is not shown beyond generic Microsoft wording. |
| signin-device-code | 8 | Code is prominent and copyable, but no visible expiry or remaining-time information accompanies the waiting state. |
| signin-finishing | 8.5 | Clear progress wording; finishing card is much shorter than preceding authentication cards, shifting the footer upward. |
| signin-expired | 8 | Hovered code-sign-in tooltip obscures the registration section and contains app-registration jargon. Account identity is less prominent than the repeated brand block. |
| signin-failed | 8.5 | Reason and retry are clear; failure guidance gives no next step beyond retry or switching methods. |

B.json contains the same 36 scores and 12 prioritized findings with concrete adjustments. Overall is a holistic judgment, not an arithmetic average. No code, repository files or other review rounds were inspected.
