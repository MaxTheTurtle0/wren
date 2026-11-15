# Independent screenshot review B

**Overall: 7/10.** A calm, coherent desktop interface with readable message hierarchy and generally clear empty, loading and authentication states. Narrow layouts, subtle input affordances and incomplete failed-send feedback prevent an excellent score. All 36 light/dark pairs (72 images) were inspected. Only the supplied screenshots and screens.md informed this review.

| Dimension | Score |
| --- | ---: |
| hierarchy | 7 |
| spacing | 6.5 |
| typography | 7 |
| consistency | 8 |
| contrast accessibility | 6 |
| feedback | 6.5 |
| states | 8 |
| navigation | 7 |

## Principal findings

1. At 760 px, navigation consumes 364 px (48%); collapse or shrink the list so messages retain at least 440 pt.
2. Light tinted-field placeholders measure **4.390:1** (#727376 on #F6F6F7), below 4.5:1. Change to #62656D.
3. Light white-field placeholders measure **4.482:1** (#767779 on white). Use #62656D here too. Affected screens are capped at 7.
4. Failed-send toast has only dismissal, obscures the latest message, and shows no visible retry control or failed-message association. Add persistent inline failure and Retry.
5. Narrow hover strips cover author names/timestamps. Reserve space or use a dedicated overflow column.

Composer outlines also fail the 3:1 UI-boundary threshold (light 1.329:1; dark 1.165:1). The focused long draft needs a visible focus ring, scrollbar and bottom-aligned send controls. The Calls view needs less explanatory copy above its tasks; call panels need clearer current-state wording. Wide chats need a bounded reading column. Settings needs clearer scrolling and shorter help text. Attachment previews need less cropping and larger removal controls. Search footer and authentication small print need more comfortable typography. Empty states sit high; presence symbols need clearer distinctions.

## Measurement notes

Pixel colours were sampled with `python3 -I` and Pillow directly from original PNGs. Ratios use WCAG sRGB linearisation and relative luminance. Text measurements use the strongest glyph-core pixels within an isolated text region, rather than low-coverage antialiasing edges. Example coordinates: filter (86,68), composer (395,737), switcher (386,111), calls placeholder (423,236), status placeholder (452,219). Body threshold: 4.5:1; large text and necessary UI boundaries/icons: 3:1. Disabled controls and dimmed modal backgrounds were excluded from text-failure scoring.

Dark placeholders pass (5.45:1 on composer, 6.00:1 in tinted-input equivalents). Sampled light secondary text (#62656D on white) passes at 5.83:1; error text (#C42B2B on white) at 5.63:1; links (#1F63C6 on white) at 5.74:1. Selected-row email text passes narrowly in both modes (4.60:1 light, 4.57:1 dark). Static screenshots cannot establish keyboard behaviour, hitbox size, announcements or timed feedback; no such behaviour is claimed as tested.

## Screen scores

| Screen | Score |
| --- | ---: |
| chats | 7 |
| chats-wide | 6.5 |
| chats-narrow | 6 |
| chat-no-selection | 7 |
| chat-filter-no-match | 7 |
| group-chat | 7 |
| channel | 7 |
| channel-narrow | 6 |
| teams | 8 |
| calendar | 8 |
| calls | 7 |
| calls-narrow | 6 |
| composer-mention | 7 |
| composer-long-draft | 6.5 |
| composer-images | 7 |
| image-preview | 8 |
| search-switcher | 7 |
| new-chat | 8 |
| settings | 7.5 |
| shortcuts | 8.5 |
| status-message | 7 |
| account-menu | 7 |
| permission-consent | 8 |
| error-toast | 6 |
| first-sync-loading | 7 |
| first-sync-empty | 7 |
| call-bar | 7 |
| call-docked | 7 |
| join-on-phone | 8.5 |
| setup | 7 |
| signin | 8 |
| signin-browser | 8 |
| signin-device-code | 8.5 |
| signin-finishing | 8 |
| signin-expired | 8 |
| signin-failed | 8 |

B.json records all screen-specific findings and 16 prioritised issues with evidence and concrete fixes.
