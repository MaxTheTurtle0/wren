# Independent design review, round 10 / B

Overall: **7.5/10**. Contrast **7.5**, typography **8**, colour **8.5**, spacing **7.5**, hierarchy **7.5**.

Inspected all 36 screens in both themes, 72 images, using only the supplied screen descriptions and images. The palette is calm, the Inter typography is coherent, and loading, empty, permission and failure states are clearly represented. The app is good, with clear issues that prevent a 9.

The three largest blockers are scroll-edge text fading, narrow-layout loss of context and actions, and disproportionately prominent message hover toolbars. The pointer explains why hover states appear; the review grades their design, not their mere presence. Settings organization and smaller text tiers also need refinement.

## Pixel contrast checks

Measured with Python 3 isolated mode and Pillow, using sRGB relative luminance and (Llighter + 0.05)/(Ldarker + 0.05). Text threshold: 4.5:1; large text and essential control/icon threshold: 3:1. Core glyph pixels were used for ordinary text; anti-aliased edge pixels alone were not treated as failures.

| Sample | Light | Dark | Result |
| --- | --- | --- | --- |
| Chat body | #1b1c1f / #ffffff, 17.04:1 | #e9e9ec / #17181b, 14.65:1 | Pass |
| Selected timestamp | #62656d / #e4e4e8, 4.60:1 | #9c9ea8 / #34353c, 4.57:1 | Narrow pass |
| Composer placeholder | #62656d / #ffffff, 5.83:1 | #9c9ea8 / #25262b, 5.66:1 | Pass |
| Input border | #7c7f87 / #ffffff, 4.01:1 | #82858e / #25262b, 4.09:1 | Pass, 3:1 threshold |
| Header icon | #4a4d55 / #ffffff, 8.45:1 | #b4b6bf / #17181b, 8.78:1 | Pass |
| Error text | #b42525 / #ffffff, 6.50:1 | #f58a8a / #25262b, 6.39:1 | Pass |
| Calls success text | #1b7a42 / #ffffff, 5.37:1 | #4cc36e / #17181b, 7.89:1 | Pass |
| Faded channel author at y64 | #bcbcbd / #ffffff, 1.90:1 | #555559 / #1c1d20, 2.27:1 | Fail |
| Faded error-view author at y65 | #bababa / #fefefe, 1.92:1 | #57575a / #17181b, 2.47:1 | Fail |
| Faded top message fragments at y58 | #f1f2f2 / #ffffff, 1.12:1 | #252528 / #17181b, 1.16:1 | Fail |

The gradient failures concern intentionally faded, partially scrolled content, not the normal text palette. Nevertheless, actual text remains visible in the viewport at these ratios, so the requested cap of 7 applies to chats-narrow, channel-narrow, composer-images, composer-long-draft and error-toast. Clip overflow cleanly instead of washing out glyphs. These are screenshot-based contrast observations, not a full accessibility audit. Decorative separators, skeletons, disabled buttons and dimmed background content were not required to meet active-control contrast.

## Priority changes

1. **high / contrast**: The scroll-edge gradient fades still-visible message text and author labels below 4.5:1. Remove the gradient over text. Clip scrolling content at the header boundary, or use a solid header with a 1 px separator; keep fully visible text at #62656d or darker in light mode and #9c9ea8 or lighter in dark mode. These five screens are capped at 7.
2. **high / other**: Narrow recent-people rows lose visible call actions. Keep one visible 28-32 px call/menu control at the right of each person row, exposing Audio and Video; alternatively collapse the list into a picker and show the selected person with both labeled actions.
3. **medium / spacing**: The navigation allocation truncates identity and message context, while narrow discussions wrap excessively. At widths below 900 px, offer a collapsible conversation list with a Back/Chats control; on wide desktops permit resizing the list to 340-380 px. Preserve a minimum 160 px name measure before timestamps.
4. **medium / hierarchy**: Hover toolbars make secondary actions compete with message identity and content. Show two or three frequently used reactions, Reply and a More menu in a 32 px toolbar. Put remaining emoji, copy, edit and delete in popovers; use a quiet outline rather than a pronounced shadow.
5. **medium / hierarchy**: Settings lacks navigation and a stable lower boundary for a long preference list. Add a compact category list or tabs; group controls in 16-24 px spaced sections. Use a clear viewport clip with 16 px bottom padding so partial controls do not look like stray strokes.
6. **medium / hierarchy**: Selected recipients and the highlighted search result compete as the intended next target. Label the result action Add Barbara, add a 13 px Selected recipients label above the chip, and use a stable Start chat button with a recipient count. Keep 16 px separation between picked recipients, search and results.
7. **medium / other**: Application ID help refers to a nonexistent visible example. Use a visible example such as 00000000-0000-0000-0000-000000000000 in helper text, or say Paste the Application (client) ID from your registration. Put each field helper below its own input with an 8 px gap.
8. **medium / other**: Pending images are identified only by dimensions, and extreme aspect ratios are difficult to recognize. Add a filename or Image 1/2/3 label in 12-13 pt text. Keep Fit previews, and provide an obvious inspect/zoom affordance on hover; avoid cropping away source content.
9. **medium / typography**: Instructional text has inconsistent and sometimes overly long line lengths. Constrain explanatory copy to 55-75 characters per line, use 13-14 pt text with 19-20 pt line height, and reduce the Calls overview to one short sentence with Details for the longer explanation.
10. **medium / typography**: The lowest text tiers are consistently small for important supporting information. Use 12 pt minimum for navigation and section labels, 13 pt for actionable hints and person metadata, and retain 14 pt message text with 20 pt line height. Preserve 8 px separation between label tiers.
11. **medium / spacing**: Call status and controls consume two rows before the meeting header. At 720 px put status and actions on one 48 px row. At 380 px use one labeled Call actions menu or short Phone, Expand and Hide labels with recognizable icons.
12. **low / hierarchy**: Search preview density reduces the value of the second line. Cap results to six initially, use 14 pt names and 13 pt previews with 20 pt line height, and separate a 40 px keyboard-help footer from the explanatory note by 8 px.
13. **low / contrast**: Selected-row timestamps have little contrast headroom. They pass; this is not a failure. For metadata on selection use #555962 in light mode and #b4b6bf in dark mode. Keep other selected-row semantics and the orange selection bar.
14. **low / spacing**: Dialog padding and reference-column spacing vary visibly. Use a shared 24 px dialog inset and 16 px section gap. Give each shortcuts group a consistent 16 px key-to-description gap after its widest keycap.

## Screen scores

Each score considers both modes. Detailed issues and pixel coordinates are in grade.json.

| Screen | Score |
| --- | --- |
| account-menu | 8 |
| calendar | 8 |
| call-bar | 7.5 |
| call-docked | 8 |
| calls | 8 |
| calls-narrow | 7 |
| channel | 8 |
| channel-narrow | 7 |
| chat-filter-no-match | 8.5 |
| chat-no-selection | 8.5 |
| chats | 8 |
| chats-narrow | 7 |
| chats-wide | 8 |
| composer-images | 7 |
| composer-long-draft | 7 |
| composer-mention | 8 |
| error-toast | 7 |
| first-sync-empty | 9 |
| first-sync-loading | 8.5 |
| group-chat | 8 |
| image-preview | 8 |
| join-on-phone | 8.5 |
| new-chat | 7.5 |
| permission-consent | 8.5 |
| search-switcher | 8 |
| settings | 7.5 |
| setup | 7.5 |
| shortcuts | 8.5 |
| signin | 8.5 |
| signin-browser | 8.5 |
| signin-device-code | 8.5 |
| signin-expired | 8.5 |
| signin-failed | 8.5 |
| signin-finishing | 8.5 |
| status-message | 8.5 |
| teams | 8.5 |

Screenshot review only. No repository inspection, interaction testing, source changes or files outside the two requested review outputs.
