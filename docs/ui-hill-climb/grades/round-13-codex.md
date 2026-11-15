# Independent design review: round 13, B

Overall: **8/10**. Contrast **9**, typography **8**, colour **8.5**, spacing **8**, hierarchy **8**.

Reviewed all 36 light/dark pairs (72 images), including the actual 380px call/QR captures and 720px docked view. The app is calm, coherent and legible. Dark/light parity is strong. Empty, loading, consent and failure states communicate their purpose. It is good with clear issues rather than a 9 with only nitpicks.

The leading blockers are narrow-window identity truncation, the partially clipped top line in the long draft, and repeated use of small secondary text. Meeting detail and recipient selection need stronger hierarchy. Remaining issues concern scroll affordances, attachment identity and the visual weight of hover controls. Hover states and tooltips were assessed as the supplied pointer state, not presumed interaction bugs.

## Pixel contrast checks

Python 3 isolated mode and Pillow sampled original PNG pixels. Ratios use WCAG sRGB linearization and relative luminance, not screenshot appearance. Foreground samples use recurring interior glyph colours; antialiased edge blends are not treated as separate failing text colours. Disabled controls, dimmed modal backgrounds and decorative separators were not incorrectly scored as required active-control boundaries.

| Sample | Foreground / background | Ratio | Required |
| --- | --- | ---: | ---: |
| Light chat body, chats x442..960/y175..325 | #1b1c1f / #ffffff | 17.04 | 4.5 |
| Light composer help, chats x394..900/y770..790 | #62656d / #ffffff | 5.83 | 4.5 |
| Light onboarding footer, setup x369..918/y731..768 | #62656d / #f6f6f7 | 5.40 | 4.5 |
| Dark composer help | #9c9ea8 / #1a1b1f | 6.45 | 4.5 |
| Dark setup helper, x430..840/y448..476 | #9c9ea8 / #26272c | 5.59 | 4.5 |
| Light quoted message | #4a4d55 / #e2e3e8 | 6.60 | 4.5 |
| Dark selected-row text | #b4b6bf / #34353c | 6.04 | 4.5 |
| Light link, chats x780..1015/y654..680 | #1f63c6 / #ffffff | 5.74 | 4.5 |
| Dark link, same region | #7cb4f5 / #1a1b1f | 7.95 | 4.5 |
| Light sign-in error, x430..790/y334..354 | #b42525 / #ffffff | 6.50 | 4.5 |
| Dark sign-in error, same region | #f58a8a / #26272c | 6.31 | 4.5 |
| Light failed-send text, error-toast x442..825/y675..691 | #b42525 / #f9eeee | 5.72 | 4.5 |
| Dark failed-send text, same region | #f58a8a / #2c2428 | 6.39 | 4.5 |
| Light input outline / white interior | #7c7f87 / #ffffff | 4.01 | 3 |
| Light input outline / sidebar surface | #7c7f87 / #f6f6f7 | 3.71 | 3 |
| Dark input outline / composer interior | #82858e / #26272c | 4.04 | 3 |
| Dark input outline / main surface | #82858e / #1a1b1f | 4.67 | 3 |
| Light rail icons | #4a4d55 / #ececee | >7 | 3 |
| Light primary button text | #ffffff / #a5500f | 5.59 | 4.5 |

No sampled real-text pair fails 4.5:1; no sampled active input boundary fails 3:1. Therefore no contrast-based screen cap was applied. These are targeted screenshot measurements, not a complete WCAG 2.2 conformance audit: keyboard operation, accessible names, dynamic feedback, target hit areas and focus behavior cannot be established from still images. The small-type critique concerns comfort and hierarchy even where measured colour contrast passes.

## Screen grades

| Screen | Score | Issues |
| --- | ---: | --- |
| account-menu | 8.5 | Account emails and section labels are small; the two same-name accounts require close reading. |
| calendar | 8 | Meeting detail has weak hierarchy: organizer metadata gets a large card while time and Join remain small in the header. |
| call-bar | 8 | Conversation spends substantial height on the opening gap and meeting/date events. Phone, Expand and Hide have equal emphasis despite different importance. |
| call-docked | 8 | Meeting/date events fragment the short conversation. The hovered message fills almost the full pane with a substantial background. |
| calls | 8 | Calling explanation dominates the main pane more than the action. Join field and informational sections do not form a particularly strong visual group. |
| calls-narrow | 7.5 | Several recent-person names truncate despite generous unused vertical space. Sidebar consumes substantial width while the explanatory paragraph wraps to five lines. |
| channel | 8.5 | Repeated rounded post cards, dividers and floating action bars create more chrome than the chat view. Reply labels and composer helper text are small. |
| channel-narrow | 7.5 | Navigation occupies nearly 40% of the window. Long channel composer hint sits tightly against attachment and send controls. |
| chat-filter-no-match | 8.5 | Empty-filter feedback is clear, but its small explanation needs close reading. |
| chat-no-selection | 8.5 | Useful empty state is visually small within the large main pane. Chat list has substantial preview truncation. |
| chats | 8.5 | Hovered message background and floating reaction toolbar are visually heavy. Timestamps, previews and composer help are small. |
| chats-narrow | 7.5 | Person and group names truncate heavily in the fixed sidebar. Quotation, avatar gutter and action strip leave little room for message text. |
| chats-wide | 8.5 | Narrow fixed-width chat list still truncates previews at 1920 pixels. Small metadata and rail captions remain visually subordinate to a very large canvas. |
| composer-images | 8 | Extreme-aspect attachments become tiny motif strips inside identical thumbnail tiles. Pixel dimensions are the only visible attachment descriptors. |
| composer-long-draft | 7.5 | Fragments of the preceding draft line show at the top of the scrolled editor. Scroll indicator is a tiny bottom-right segment rather than a readily legible full-height track. |
| composer-mention | 8.5 | Mention popup email and keyboard hint are small. Mention, hover, reaction pills and attachment chip produce a crowded lower conversation area. |
| error-toast | 8.5 | Failure feedback is clear and persistent, but the recovery sentence and Retry use a small secondary scale. |
| first-sync-empty | 9 | Only a minor opportunity to enlarge the central empty-state heading. |
| first-sync-loading | 9 | Only a minor opportunity to enlarge the central loading heading; elapsed-time behavior cannot be assessed from screenshots. |
| group-chat | 8.5 | Full-row mention tint and floating reaction controls compete with conversation text. System event line is small. |
| image-preview | 8.5 | Fit/100% controls and image dimensions are small relative to the overlay. Preview has no visible human-readable image name. |
| join-on-phone | 8.5 | Meeting identity is much smaller than the dialog title. Copy link is a small secondary control beneath a large QR area. |
| new-chat | 8 | Highlighted Barbara result and orange Chat with Grace Hopper button refer to different people, requiring careful interpretation. Recipient selection and search results need stronger separation. |
| permission-consent | 8.5 | Consent explanation falls through three small text levels. Technical permission string receives a full paragraph but little typographic distinction. |
| search-switcher | 8.5 | Most two-line recent results truncate the preview. Footer explanation and keyboard hints add a small, dense third text band. |
| settings | 8 | Scrollable modal ends at a faint fade directly under the last visible checkbox. Section labels, explanatory text and shortcut information are small and closely packed. |
| setup | 8 | Technical setup form mixes descriptive, helper and disabled-action explanation text at a small scale. Registration guidance has similar visual treatment to secondary actions. |
| shortcuts | 8.5 | Small shortcut labels and an uneven gap between keycaps and descriptions make the left column slower to scan. |
| signin | 8.5 | Hover tooltip overlaps the lower card in the supplied state. Brand is much larger than the task heading and account instructions. |
| signin-browser | 8.5 | Open page, Copy link and Cancel form a compact row; progress text remains small. |
| signin-device-code | 8.5 | Code is appropriately prominent, but destination instructions and waiting state are small. |
| signin-expired | 8.5 | Hover tooltip obscures part of the lower card in the supplied state. Account identity is small relative to the importance of reauthentication. |
| signin-failed | 8.5 | Error and recovery are clear; App registration remains visually close to the recovery controls despite being a separate task. |
| signin-finishing | 8.5 | Progress title is only modestly stronger than the descriptive body. |
| status-message | 9 | Only a minor opportunity to give the explanatory line more breathing room. |
| teams | 8.5 | Empty-state title and instruction are visually small in the available main pane. |

## Prioritized changes

1. **Narrow layouts reserve too much width for navigation while clipping identities and compressing content.** At 760px, rail plus sidebar ends at x=295, taking 38.8% of the width. Calls names occupy roughly x=128..222 before call icons at x=233 and 267; Katherine Jo..., Dorothy Vaug... and Margaret Ha... are visibly truncated. Fix: Below 800pt, offer a 64pt rail plus collapsible conversation list; in Calls put audio/video actions on a second line or reveal them on row focus, preserving at least 150pt for names.

2. **The scrolled long draft has a visibly clipped preceding line at its top edge.** Editor rectangle is approximately x=394..1250, y=552..760. Text fragments appear near y=559 above the first fully legible SELECT column_111 line at y=575. The scrollbar thumb is only around x=1166, y=728..752. Fix: Keep an 8pt inset above the text clipping region and clip partial rows cleanly; place a visible 4pt scrollbar in a dedicated gutter spanning the editor viewport, with a minimum 24pt thumb.

3. **Too much useful information uses the smallest secondary type tier.** Native images show main conversation text around 14pt, with 11-12pt rail labels, timestamps, helper text, account emails and shortcut hints. For example chats composer help occupies approximately y=774..784 and account-menu emails y=363..371. These are screenshot estimates, not source font measurements. Fix: Use 13pt for helper sentences, account emails and result previews, with 18pt line height; reserve 11pt for short uppercase category labels and 12pt for timestamps.

4. **Search focus and the primary action identify different people with insufficient separation.** Selected Grace Hopper chip sits near y=184; highlighted Barbara Liskov result spans y=282..326; bottom orange CTA at y=477 says Chat with Grace Hopper. The two strong targets compete. Fix: Group selected people in a labelled Recipients section with 16pt separation from search results; use a stable Start chat CTA with an adjacent recipient count, preserving the selected result marker.

5. **The scrollable settings viewport gives a weak cue that more content follows.** Modal spans approximately y=49..722. Last visible checkbox is at y=672 with only a faint fade around y=695; scrollbar thumb runs y=117..455, well above the viewport end. Fix: Reserve 16pt bottom padding plus a consistent 16pt scroll fade, or a small More settings below cue; keep a visible scrollbar track to the viewport bottom.

6. **Meeting time and joining deserve stronger emphasis than organizer metadata.** At 1280px, time is a secondary line at y=35 and Join is a roughly 66x32px control at the far right. Organizer/Online metadata occupies a 720x100px card at x=394, y=78. Fix: Put an 18pt meeting-time line and 36pt-high Join action together in the detail content; render organizer/online as a compact 48-64pt metadata group below.

7. **Opening whitespace and repeated meeting events consume disproportionate conversation height.** In the 380x800 call-bar screenshot, header ends at y=132, beginning-of-conversation text is near y=247 and the first actual message starts near y=360. Date and meeting-event labels add separate rows at approximately y=288,322,598,635,669. Fix: Reduce the opening inset to 24-32pt and combine each date with its meeting-start event into one compact 28pt row; retain the bottom-anchored composer.

8. **Uniform attachment tiles make extreme-aspect images difficult to identify.** Three approximately 144x96px tiles start near x=408,560,712. The 3072x256 and 256x3072 images reduce to a tiny horizontal strip and vertical strip; labels show only dimensions. Fix: Add a 13pt filename or short attachment label, retain dimensions at 12pt, and offer a visibly labelled Preview action so the thin strip is not the only identity cue.

9. **Hovered message treatment is heavier than needed and competes with persistent semantic highlights.** In chats, hover background spans approximately x=382..1258, y=330..487 and includes a roughly 172x40px toolbar; group-chat adds a full-width peach mention band immediately above a grey hover band. The centered pointer explains the hover, so this is not treated as a stuck-state defect. Fix: Keep the hover state but limit tint to the message content area, use 8pt row insets, and reduce the action bar to 32pt height; preserve the mention accent stroke as the strongest row-level marker.

10. **Explanatory copy visually outweighs the main calling task.** In calls-narrow, Join panel ends near y=174, while the How calls work section and capability rows occupy roughly y=201..375; the paragraph alone takes five lines. Fix: Use one 13pt summary sentence under Join, then put detailed explanation behind the existing Details disclosure; keep recent-person actions prominent.

11. **Technical setup and permission details are insufficiently separated from the plain-language task.** Setup shows tenant hint over two short lines around y=450..473 and a second save explanation around y=575. Permission dialog includes a two-line Permission: Chat.Create paragraph beneath browser-consent copy. Fix: Use 14pt task instructions with 20pt leading; present technical IDs/scopes in a compact 12pt detail row or disclosure and remove repeated explanatory sentences.

12. **Shortcut column spacing is uneven.** At 1280px, left-column keycaps begin around x=200 while their descriptions begin around x=386, creating a broad gap for short keys; multi-key rows occupy much more of that space. Fix: Use consistent 160pt key and flexible description columns, with 16pt separation and 32pt row height; align category labels to the same column starts.

13. **Preview controls and image identity have little prominence in the large overlay.** Overlay occupies nearly the entire 1280x800 window while Fit/100% controls are roughly 12pt at top left and identity is only 3072x256. Fix: Use 13pt labels in 32pt-high Fit/100% controls and display a 14pt filename beside the preview title, with dimensions at 12pt.

14. **Orange performs several unrelated semantic jobs, weakening distinction between attention and selection.** Light #a5500f and dark #e88c3e appear on navigation selection, unread time/dot, account unread badge, reaction selection and primary action treatments. This is a semantic-overload finding, not a measured contrast failure. Fix: Keep orange for primary actions and selected navigation; use bold 13pt neutral text plus the unread dot for unread time, and a neutral outlined reaction selection with a distinct check marker.

15. **The visible hover tooltip overlaps the lower authentication card.** In both supplied states the Use a code instead tooltip extends rightward and covers part of the divider/App registration area below the button row. Pointer placement explains its presence. Fix: Place the tooltip above the fallback button with an 8pt offset and maximum 260pt width; keep it out of the next card section.

