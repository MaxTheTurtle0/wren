# Independent screenshot review: round 11 B

Overall: **8/10**. Contrast 8.5; typography 8; colour 8.5; spacing 7.5; hierarchy 7.5.

All 36 light/dark pairs were inspected, including all narrow and wide variants. Calm neutral surfaces, consistent Inter typography, restrained accent colour, explicit failure recovery, and clear loading/empty states make this a good desktop interface. It falls short of excellent through layout and emphasis, rather than a broken core visual system. Mouse-centre hover bars and the visible sign-in tooltip were treated as valid states.

## Three blockers to 9

1. Narrow sidebars truncate identities and squeeze messages; the wide chat column instead floats too far from its list.
2. Useful supporting text is small; Calendar and Calls need stronger task and detail hierarchy.
3. Recipient selection and first-run composition need clearer structure; composer clipping, thumbnail legibility, and inconsistent dialog rhythms need polish.

## Pixel contrast checks

Python 3 isolated mode and Pillow sampled original PNG pixels. Ratios use WCAG sRGB relative luminance, comparing solid glyph interiors with their actual local backgrounds, not antialiased edge pixels. Thresholds: 4.5:1 for normal text; 3:1 for large text and meaningful icons/control boundaries. No confirmed failure was found in these samples; no screen was capped for a contrast failure. This is a screenshot sample audit, not an exhaustive accessibility certification. Dimmed content behind modal scrims and inactive buttons were excluded from failure claims.

| Sample | Light | Dark | Result |
| --- | --- | --- | --- |
| Chat composer hint | #62656d / #ffffff, 5.83:1 | #9c9ea8 / #17181b, 6.65:1 | Normal text passes |
| Timestamp on hovered message | #62656d / #eeeef0, 5.03:1 | #9c9ea8 / #25262b, 5.66:1 | Normal text passes |
| Quote text | #4a4d55 / #e4e4e8, 6.67:1 | #b4b6bf / #2c2d33, 6.79:1 | Normal text passes |
| Chat link | #1f63c6 / #ffffff, 5.74:1 | #7cb4f5 / #17181b, 8.20:1 | Normal text passes |
| Primary new-chat CTA | #ffffff / #a5500f, 5.59:1 | #1c1208 / #e88c3e, 7.24:1 | Normal text passes |
| Sign-in failure text | #b42525 / #ffffff, 6.50:1 | #f58a8a / #25262b, 6.39:1 | Normal text passes |
| Calls Works text | #1b7a42 / #ffffff, 5.37:1 | #4cc36e / #17181b, 7.89:1 | Normal text passes |
| Filter top border | #7c7f87 / #f6f6f7, 3.71:1 | #82858e / #1d1e22, 4.52:1 | Boundary passes |
| Settings dropdown boundary | #7c7f87 / #eeeef0, 3.46:1 | #82858e / #2c2d33, 3.72:1 | Boundary passes |
| Rail phone icon | #4a4d55 / #ececee, 7.17:1 | #b4b6bf / #131417, 9.11:1 | Icon passes |

Typography size observations are estimates from the raster at the documented point sizes. Coordinates and colour samples refer to original image pixels. Low-contrast decorative dividers are not asserted to violate WCAG where text/grouping supplies the same information.

## Priorities

1. **medium**, chats-narrow, calls-narrow, channel-narrow: Narrow layouts retain too much navigation width, while identities and message text lose space. Below 800 points use a collapsible list, with a 64-point rail and a back/list control; alternatively reduce list padding and put call actions on a second row so names remain readable.

2. **medium**, calendar: The selected event lacks a strong detail hierarchy. Use a 20-point event title and 15-point time/date directly above a compact metadata block; reserve generic joining guidance for applicable online meetings or a disclosure.

3. **medium**, chats, chats-wide, channel, settings, setup, permission-consent, search-switcher, image-preview: Too much useful guidance sits at the smallest end of the type scale. Use 13-point supporting copy and 12-point timestamps; keep 11-point text for occasional category labels. Increase instruction line height to 18 points and shorten recurring composer instructions.

4. **medium**, calls, calls-narrow: The Calls page gives the most content emphasis to implementation and capability prose. Keep one short 13-point description below Join; place the extended explanation under Details. Give a selected person a clear 32-point-high Call action in the main pane.

5. **medium**, new-chat: Selection state is ambiguous between the highlighted search result and the committed recipient. Label chips Recipients and results Add a person; use an explicit Add control for the active result and a consistent Start chat CTA with recipient count.

6. **medium**, setup: First-run layout spends too much height on branding before a dense setup form. Reduce the setup brand block to 96 points, use 24-point separation before the form, and keep 24-point lower safe padding. Maintain 14-point instructions and 32-point input heights.

7. **low**, chats-wide: The reading column feels disconnected from the conversation list on a wide window. Use a 320..360-point adjustable chat list at wide sizes and reduce the leading content gutter to 48..96 points while keeping message measure around 760..860 points.

8. **low**, composer-long-draft: The scrolled composer has a rough clipped edge and inconsistent scrollbar treatment. Clip text within an inner rectangle with 8-point top padding, add a 4-point edge fade for partial lines, and match composer thumb colour to the history scrollbar; keep a compact 32..40-point action gutter.

9. **low**, composer-images: Equal thumbnail boxes obscure extreme-aspect-ratio attachments. Keep 96-point previews but crop to fill with an aspect-ratio badge; show the complete image on open and add a 13-point filename rather than relying on dimensions alone.

10. **low**, settings: Settings mixes incompatible vertical rhythms and exposes a clipped content tail. Use 16-point separation after a setting with helper copy and 12 points between simple rows. Add a 16-point lower inset and a subtle scroll-edge fade so continuation looks intentional.

11. **low**, call-bar, call-docked: Sparse call history floats far below its identifying header. Place the start-of-conversation marker 24..32 points below the header for short histories; use 16-point event spacing and preserve clear separation before the composer.

12. **low**, shortcuts: Keyboard reference columns do not share a consistent scanning rhythm. Use a shared 140-point key column and 16-point gap in each group, or split the long Zoom combination onto its own row. Keep 14-point descriptions and 34-point row rhythm.

13. **low**, chats, group-chat, channel, settings: Orange carries too many independent meanings. Reserve #a5500f light / #e88c3e dark for focus, primary actions and mentions. Use weight plus a neutral unread dot for unread rows and neutral selected-reaction fills, retaining explicit selection outlines.

## Screen grades

| Screen | Score | Main observation |
| --- | --- | --- |
| account-menu | 8 | Account identities have nearly identical visual treatment; organisation is relegated to small email text. |
| calendar | 7.5 | Meeting title has little more emphasis than list labels. |
| call-bar | 8 | Large blank interval separates the meeting header from conversation start. |
| call-docked | 8 | Large blank interval above the conversation makes the sparse history feel detached from its header. |
| calls-narrow | 7 | Several recent-person names truncate despite substantial unused vertical space. |
| calls | 7.5 | The main pane reads like a help article beneath the join form. |
| channel-narrow | 7.5 | Navigation consumes 296 of 760 pixels, leaving heavily wrapped thread content. |
| channel | 8 | Repeated card boundaries and reply buttons add visual weight to every post. |
| chat-filter-no-match | 8.5 | Narrow-column empty-state guidance is quite small. |
| chat-no-selection | 8.5 | The central empty state has modest emphasis relative to the large empty pane. |
| chats-narrow | 7.5 | Names and previews truncate in the fixed sidebar. |
| chats-wide | 8 | The centred conversation leaves a roughly 350-pixel gulf from sidebar to message column. |
| chats | 8 | Persistent composer instructions and timestamps are small. |
| composer-images | 8 | Very wide and very tall image thumbnails have tiny identifiable content inside equal-sized tiles. |
| composer-long-draft | 7.5 | A partially clipped text line is visible immediately inside the top composer edge. |
| composer-mention | 8 | Suggestion popup is anchored to the left of the composer rather than near the typed mention. |
| error-toast | 8.5 | Recovery is explicit and the failed message remains visible; the error sentence is small for an important failure. |
| first-sync-empty | 8.5 | Empty state has clear action but a very small heading and explanatory line in a large pane. |
| first-sync-loading | 8.5 | Loading feedback is clear; centre status text is small relative to the empty canvas. |
| group-chat | 8.5 | Mention highlighting is clear; event descriptions and composer instructions are small. |
| image-preview | 8 | Wide image dialog sits near the top while most of the lower window remains dimmed. |
| join-on-phone | 8.5 | QR code is prominent and legible in both modes; Copy link feels detached in the lower right. |
| new-chat | 8 | Highlighted candidate and primary button refer to different people, increasing interpretation effort. |
| permission-consent | 8.5 | Action hierarchy is clear; permission name and consent qualification are small secondary text. |
| search-switcher | 8.5 | Clear result and keyboard hierarchy; result snippets truncate early and bottom guidance is small. |
| settings | 8 | Large gap follows the notification helper, while later checkboxes use a tighter rhythm. |
| setup | 7.5 | Large brand lockup pushes a dense technical setup form into almost the entire window height. |
| shortcuts | 8 | Unequal key-to-description gaps make the two columns scan at different rhythms. |
| signin-browser | 8.5 | Recovery controls and waiting status are clear; footer and supporting text are small. |
| signin-device-code | 8.5 | Code is prominent; code-entry URL is embedded in a small prose paragraph. |
| signin-expired | 8.5 | Clear recovery and account context; logo is much stronger than the session-expired explanation. |
| signin-failed | 8.5 | Failure and retry are clear; error reason is small relative to branding. |
| signin-finishing | 8.5 | Clear progress state; status card is visually subordinate to the large brand lockup. |
| signin | 8.5 | Primary and fallback actions are clear; large branding outweighs practical sign-in content. |
| status-message | 8.5 | Focused field and disabled Save communicate state clearly; supporting explanation is small. |
| teams | 8.5 | Tree is clear and calm; empty-pane guidance has little visual emphasis. |
