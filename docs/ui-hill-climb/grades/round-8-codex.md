# Independent screenshot review: 8/10

Reviewed all 36 screens in both themes (72 PNGs), using only the supplied screenshots and screens.md. Scores combine both versions. Coordinates refer to original screenshot pixels; proposed sizes are points. The call-bar and phone-join captures are actually 380×800, and call-docked is 720×800.

The app has a consistent, calm visual language, clear selection accents and well-separated conversation structure. Empty, loading, permission and failed-send states are generally clear. Most remaining problems concern readability, narrow-window priorities and dialog action hierarchy.

## Highest priorities

1. **Narrow layouts truncate people’s identities while retaining permanent navigation and action columns.** At 760px, rail plus sidebar end at x=296 (39% of width). Calls labels start x=128 and truncate near x=218 before icons at x=233 and x=267; Katherine, Dorothy and Margaret are clipped. Below 800pt offer a collapsible list; reserve at least 140pt for person names. Put secondary video action in a 32pt overflow control or allow a two-line name with a 48pt row.

2. **The long draft has no clear indication of its hidden beginning.** Composer spans approximately x=380-1264, y=553-761. Visible text starts SELECT column_111 and ends column_119; no visible internal scrollbar distinguishes it from a complete nine-line draft. Show a persistent 6pt internal scrollbar while overflowing, with 8pt right clearance. Add a 32pt expand control and a 12pt Earlier text above cue when scrolled away from the start.

3. **The active search result and primary action refer to different people.** Barbara Liskov is highlighted at x=400-880, y=269-313 with Enter to add. The primary action at x=400,y=431 says Chat with Grace Hopper. Keep a 32pt Add Barbara action beside the active result and place the 32pt Chat with Grace Hopper action in a separate footer with 16pt vertical separation; make the two steps explicit.

4. **Settings is a long flat scroll surface with weak wayfinding.** Dialog occupies x=359-921, y=70-742. Four section headings are visible and the next checkbox is cut off around y=716; the scroll thumb indicates substantial content below. Use a 120pt persistent section list with Account, Appearance, Behaviour and Calls; keep the content pane scrollable with 20pt padding and 24pt separation between groups. End the viewport after a complete row where possible.

5. **Technical setup instructions dilute the main task.** The card at x=405-875, y=223-647 mixes the registration guide, GUID-shaped ID, tenant-domain rules, multi-tenant terminology and WREN_CLIENT_ID/WREN_TENANT instructions. Keep the guide and two fields in the main card; move environment-variable and multi-tenant detail under an Advanced disclosure. Use 13pt field labels, 12pt helper text and 16pt between field groups. Label the zero GUID as an example rather than an apparent value.

## Contrast measurements

Python 3 isolated mode and Pillow sampled original PNG pixels, not resized review composites. Ratios use WCAG relative luminance. Solid text-core colours were checked against their immediate surface; antialiased edge pixels were not treated as the intended text colour. Shared primary, secondary and muted text palette pixels were also checked for presence in every image.

| Sample | Foreground | Background | Ratio | Assessment |
| --- | --- | --- | --- | --- |
| Chat composer help, light, x380–850/y770–790 | #62656D | #FFFFFF | 5.83:1 | Pass body text |
| Chat composer help, dark, same region | #9C9EA8 | #17181B | 6.65:1 | Pass body text |
| Settings explanatory text, dark, x382–875/y398–450 | #B4B6BF | #25262B | 7.47:1 | Pass body text |
| Quoted message, light | #4A4D55 | #EEEEF0 | 7.30:1 | Pass body text |
| Failed-send text, light, x432–814/y675–691 | #C42B2B | #FBEEEE | 4.98:1 | Pass body text |
| Sign-in failure text, dark | #F27474 | #25262B | 5.41:1 | Pass body text |
| Chat link, light | #1F63C6 | #FFFFFF | 5.74:1 | Pass body text |
| Settings dropdown outline, light | #8A8D95 | #FFFFFF exterior / #EEEEF0 interior | 3.32:1 / 2.87:1 | Exterior passes; little headroom |
| Settings dropdown outline, dark | #6E7179 | #25262B exterior / #2C2D33 interior | 3.09:1 / about 2.81:1 | Exterior passes; little headroom |

No sampled real-text contrast failure was established, so no contrast-based score cap was applied. The outline finding is a robustness concern, not an asserted WCAG failure. Small text can pass contrast and still be uncomfortable. Screenshots cannot establish keyboard operation, screen-reader semantics, target hitboxes, animation quality or QR-code destination correctness.

## Dimensions

- hierarchy: 8/10
- spacing: 8/10
- typography: 7.5/10
- consistency: 9/10
- contrast_accessibility: 8.5/10
- feedback: 8/10
- states: 8.5/10
- navigation: 8/10

## Screen scores

- **account-menu: 8/10.** Long flat menu mixes account switching, presence and application settings; small secondary labels.
- **calendar: 8.5/10.** Meeting details require scanning between the far-right Join action and the detail card; small supporting text.
- **call-bar: 8.5/10.** Compact action labels and meeting-status text are small; browser-call ownership is less explicit here than on Calls.
- **call-docked: 8.5/10.** Hover reaction toolbar is visually prominent relative to the short message; small meeting-event labels.
- **calls-narrow: 7.5/10.** Three recent-person names are truncated beside two permanent call buttons. Capability explanation becomes a dense four-line paragraph plus wrapped limitations.
- **calls: 8/10.** Capability explanation dominates the page below the join form; small supporting labels.
- **channel-narrow: 8/10.** Fixed navigation leaves limited reading width. Hover actions compete with the post heading at this width; small timestamps.
- **channel: 8.5/10.** Long post lines and dense hover reaction toolbar; small timestamps and Reply labels.
- **chat-filter-no-match: 8.5/10.** No-match state is clear, but supporting text and the Clear filter control are small.
- **chat-no-selection: 8.5/10.** Empty-state instruction is small relative to the available pane. Meeting-chat title in the sidebar is truncated.
- **chats-narrow: 7.5/10.** Navigation consumes 296 of 760 pixels; several names and most previews truncate. Message actions remain dense at narrow width.
- **chats-wide: 8/10.** Message column permits very long single lines, slowing scanning. Dense hover action strip and small metadata.
- **chats: 8/10.** Meeting-chat title is truncated even at standard width. Dense hover action strip and small timestamps/helper text.
- **composer-images: 8/10.** Image dimensions and remove controls are small. Attachment previews lack human-readable file identifiers.
- **composer-long-draft: 7.5/10.** Draft shows only lines 111-119 without an obvious internal scrollbar or cue that earlier text is above. The composer occupies about 207 pixels vertically, but has no visible expansion control.
- **composer-mention: 8.5/10.** Mention result email and attachment metadata are small. Attachment removal is a tiny icon beside the file size.
- **error-toast: 8.5/10.** Failed-send explanation and Retry are visually small despite being the key recovery action.
- **first-sync-empty: 8.5/10.** The central explanation and primary action are small for a first-use screen.
- **first-sync-loading: 8.5/10.** Loading explanation is small; the screenshot offers no further progress detail.
- **group-chat: 8.5/10.** Small membership-event text and timestamps. Hover reaction strip visually competes with the highlighted mention.
- **image-preview: 8/10.** Preview title gives no filename or other attachment identity. Zoom controls and pixel dimensions are small.
- **join-on-phone: 8/10.** QR dialog does not name the meeting being joined. Copy the link is small and isolated at the bottom left.
- **new-chat: 7.5/10.** Barbara is highlighted with Enter to add while the main button still says Chat with Grace Hopper. Helper copy and email addresses are small.
- **permission-consent: 8.5/10.** Scope detail is small and densely wrapped. Generic One more permission title requires the body to explain the actual action.
- **search-switcher: 8.5/10.** Long result snippets truncate heavily. Keyboard hints and local-versus-Microsoft search explanation are small.
- **settings: 7.5/10.** Long flat scroll surface with the next checkbox cut by the bottom fade. Account, appearance, behaviour and calls lack persistent section navigation. Control outlines have little contrast headroom.
- **setup: 7.5/10.** GUID, tenant, multi-tenant registration and environment-variable instructions compete within the primary setup card. The all-zero identifier example resembles a prefilled value.
- **shortcuts: 8.5/10.** Small keycaps and section labels. New-line shortcut is rendered Shift Enter instead of the clearer Shift+Enter convention.
- **signin-browser: 8.5/10.** Waiting status is small; the prominent Open the page again action competes with the wait instruction.
- **signin-device-code: 8.5/10.** Waiting label and destination instructions are small compared with the well-emphasized code.
- **signin-expired: 8.5/10.** Account label and app-registration disclosure are small.
- **signin-failed: 8.5/10.** Timeout explanation is small relative to its importance; recovery actions are otherwise clear.
- **signin-finishing: 8.5/10.** Finishing explanation is small in a very sparse full-window composition.
- **signin: 8.5/10.** Small registration disclosure and footer. The hover tooltip extends far beyond the sign-in card; acceptable as transient help but visually dominant.
- **status-message: 8.5/10.** No material visual issue in this capture.
- **teams: 8.5/10.** Empty-state instruction and channel labels are small.

## Remaining priorities

6. **Supporting text is readable by contrast but consistently too small for comfortable sustained use.** (chats, channel, group-chat, error-toast, composer-mention, calendar, search-switcher; both; medium). At native resolution timestamps/helper text have roughly 9-12px ink height, versus about 14px body text; chat composer help sits at y=779. Sampled muted colours pass: #62656D on #FFFFFF 5.83:1 and #9C9EA8 on #17181B 6.65:1. Use 13pt timestamps and important recovery/helper copy, 12pt minimum for secondary metadata, and 16pt line height. Keep message body at 14pt; reduce the amount of helper copy where necessary.

7. **QR dialog omits the meeting identity.** (join-on-phone; both; medium). At 380x800, the dialog spans about x=19-361,y=100-478; its heading is only Join on your phone and the dimmed meeting header is obscured. Add the meeting title directly below the heading in 14pt semibold, with the meeting time in 12pt secondary text and 12pt spacing before the QR code.

8. **Attachment views provide dimensions without a useful attachment name.** (image-preview, composer-images; both; low). Preview header says Image preview with 3072 × 256; composer image metadata consists of dimensions such as 3072 × 256 and 256 × 3072. Show a 13pt filename, or Image 1 of 3 if no filename exists, above 12pt dimensions. Keep the filename in the preview header with 16pt clearance from the close control.

9. **Wide message lines are harder to scan than necessary.** (chats-wide, chats, channel; both; low). At 1920px the chat content extends roughly x=680-1600; the throttling-response message stays on one line for more than 120 characters. Cap message prose at 760-800pt while allowing code and attachments to expand. Keep 14pt body text and approximately 20pt line height.

10. **Hover actions have too much visual weight and too many equally prominent choices.** (chats, chats-wide, chats-narrow, channel, call-docked, group-chat; both; low). Own-message toolbar in the standard chat spans about x=955-1252, nearly 300px, with six emoji and reply/edit/delete/copy icons in one strip. This is the visible hover state, not a persistent-state defect. Show three frequent reactions plus Reply and a More control by default. Keep 28-32pt targets and 4pt gaps; put edit/delete/copy and additional reactions in More.

11. **Control outlines pass against the outer surface but have little contrast headroom.** (settings; both; low). Measured dropdown stroke: light #8A8D95 against exterior #FFFFFF 3.32:1 and interior #EEEEF0 2.87:1; dark #6E7179 against exterior #25262B 3.09:1 and interior #2C2D33 about 2.81:1. This is marginal, not an asserted WCAG failure because the exterior boundary passes. Use #7D8089 in light mode and #80838D in dark mode for 1pt borders; preserve a separate 2pt accent focus ring. Recheck contrast on both sides of the outline.

12. **Capability explanation is wordier than the action area warrants.** (calls-narrow, calls; both; low). In the 760px view, How calls work occupies y=202-359, including a four-line paragraph followed by two capability rows; the actual join field occupies only y=125-153. Use a two-line 14pt explanation such as Calls open in Teams in your browser. Wren shows the meeting chat. Keep the supported/unsupported rows at 13pt with 8pt spacing, and move window/browser implementation detail into Details.
