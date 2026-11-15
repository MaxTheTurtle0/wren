# Independent screenshot review

Overall: 8/10. Contrast 8, typography 8, colour 8.5, spacing 8, hierarchy 8.

Reviewed all 36 screens in both themes, 72 images. Scores reflect the visible states together. Hover toolbars and tooltips were treated as hover states. Inter is readable, the restrained orange accent works well in both themes, and loading, empty, permission and failure states are unusually complete. The app is good, with clear issues that keep it below excellent.

The three largest blockers are the search result contrast failure, excessive navigation width at narrow sizes, and the loss of the visible video-call action in the narrow Calls view. Small metadata and weak attachment inspection also need attention.

Pixel checks used Python in isolated mode and Pillow. WCAG relative luminance was calculated after sRGB linearization. Search preview at light pixel 415,588 is #87898e on white, 3.50:1. Dark pixel 419,588 is #84878e on #25272b, 4.16:1. The darkest or lightest glyph pixels across row 587 also fail, so this is more than an antialiasing-edge observation. Search-switcher is capped at 7.

Representative passing samples: secondary text #62656d on white is 5.83:1; dark secondary #9c9ea8 on #26272c is 5.59:1; link #1f63c6 on white is 5.74:1; error #b42525 on #f9eeee is 5.72:1; mention #b4530a on #fdf1e5 is 4.52:1 and has little margin. These samples do not certify every pixel or interactive state. Channel grouping borders are only 1.90:1 light and 1.83:1 dark, but were treated as supplementary grouping decoration, not essential control outlines.

## Screen scores

- chats: 8/10; Small timestamps and composer instructions compete with message detail. Quoted list is compressed into a small inline paragraph.
- chats-wide: 8.5/10; Readable message column, but small metadata remains harder to scan than body text. Quote flattens the original list into dense inline text.
- chats-narrow: 7.5/10; Sidebar consumes 296 of 760 pixels and truncates several contact names. Conversation and quoted text wrap heavily while timestamps retain their full width.
- chat-no-selection: 8.5/10; Clear empty state; chat names and previews in the sidebar still truncate early.
- chat-filter-no-match: 8.5/10; Useful clear-filter action; compact sidebar helper text and conversation metadata remain small.
- group-chat: 8.5/10; Mention treatment is clear, but light-mode mention text has almost no contrast margin above 4.5:1. Small system-event text recedes strongly.
- channel: 8/10; Post and reply hierarchy is readable, but timestamps are very small. Card boundaries are faint, especially in dark mode; grouping depends strongly on spacing.
- channel-narrow: 7.5/10; Navigation consumes 296 of 760 pixels, forcing long posts into a narrow column. Small reply metadata and crowded top-of-viewport content reduce scanability.
- teams: 8.5/10; Clear hierarchy and empty state; channel names and rail labels are fairly small.
- calendar: 8/10; Meeting list uses small, dense titles and time metadata. Main invitation and explanatory copy form a weakly differentiated block.
- calls: 8/10; Explanatory prose takes more emphasis than the compact call actions. Audio and video icons are small relative to the generous row height.
- calls-narrow: 7.5/10; Video call icons visible at full width disappear in the narrow layout. Long explanatory paragraph and capability text occupy much of the useful main-pane area.
- composer-mention: 8/10; Suggestion name, email, attachment size and insertion hint create several small text tiers in a tight area. Popup covers the recent conversation, with no visual connection to the typed mention at the caret.
- composer-long-draft: 8/10; Draft viewport is usable, but its narrow scrollbar is the only indication that over a hundred lines precede the visible text. Attachment and send actions sit beside the last draft line without a separate action band.
- composer-images: 7.5/10; Extreme-aspect images become strips too thin to inspect inside fixed thumbnail boxes. Small dimension labels identify shape but do not help inspect the attachment.
- image-preview: 8.5/10; Clear preview controls; image dimensions and selected Fit underline are visually slight.
- search-switcher: 7/10; Bottom fade washes out the visible final result preview below 4.5:1 in both themes. Search field has no compact visible title to distinguish jump navigation from message search.
- new-chat: 8.5/10; Strong action hierarchy; result emails and Enter-to-add hint are small.
- settings: 8/10; Long scrolling dialog lacks a strong fixed lower edge for actions or closure. 300-pixel select fields leave a different right edge from the much wider setting rows. Section labels and zoom/account hints are small.
- shortcuts: 8.5/10; Readable grouping, but left-column shortcut descriptions are separated from most keycaps by a large gap. Section labels are small.
- status-message: 8.5/10; Clear focused field and action hierarchy; secondary label and buttons are compact.
- account-menu: 8.5/10; Account and presence sections scan well; dense 12-13-point text leaves limited differentiation between account detail and actions.
- permission-consent: 8.5/10; Clear permission explanation and primary action; exact permission detail is small and visually recessed.
- error-toast: 8.5/10; Persistent inline failure and Retry are clear. Failure reason uses small text despite ample horizontal room.
- first-sync-loading: 8.5/10; Clear loading explanation and skeletons; state title is modest relative to the otherwise empty pane.
- first-sync-empty: 8.5/10; Useful first-chat action; empty navigation column consumes a large share of the screen.
- call-bar: 8/10; Compact call controls work at 380 pixels. System events and beginning-of-conversation spacing use a large share of the small transcript viewport.
- call-docked: 8/10; Large blank area above the first transcript event separates context from the conversation. Meeting lifecycle events occupy substantial vertical space relative to the three short messages.
- join-on-phone: 9/10; no material issue visible.
- setup: 8/10; Tenant helper separates the label from its field, unlike the client-ID field. Footer sits near the bottom edge and a full-height scrollbar makes a short form feel longer.
- signin: 8.5/10; Clear primary and fallback actions; hovered fallback tooltip overlaps the app-registration section.
- signin-browser: 8.5/10; Useful retry, copy and cancel controls; waiting text is fairly small.
- signin-device-code: 8.5/10; Strong code hierarchy; URL shares a small instruction line and requires careful scanning.
- signin-finishing: 8.5/10; Clear finishing state; large branding remains more prominent than the active task.
- signin-expired: 8.5/10; Account identity supports recovery; fallback tooltip overlaps the app-registration section.
- signin-failed: 8.5/10; Failure and recovery actions are clear; the large brand block dominates the comparatively small recovery card.

## Prioritized changes

1. The bottom scroll fade is painted over a real result preview, making text fail WCAG 2.2 AA. Keep the whole final visible result above the fade; clip partial rows cleanly or apply the fade only to empty padding. Use #62656d on white and #9c9ea8 on #26272c for full-opacity secondary text.
2. The narrow layout preserves too much navigation width and loses useful conversation or contact information. At widths below 800 points, offer a 200-point navigation pane including the rail, or collapse the list behind a back control. Put dates on the preview line to preserve name width.
3. The narrow calls list loses the visible video-call action. Keep both 24-point action targets by moving them into a second row, or expose a 24-point overflow button with both call choices.
4. Too much useful information uses the smallest text tiers. Use 13-point secondary text with an 18-point line height for timestamps, result emails and useful hints. Reserve 11 points for nonessential badges. Use 12-point medium-weight section labels.
5. Extreme-aspect attachment previews are too small to inspect. Retain the full-image fit view, but provide a clearly visible 24-point inspect button and a crop/zoom preview on activation. Show a filename or attachment ordinal at 13 points alongside dimensions.
6. Quoted lists lose their structure and become dense miniature paragraphs. Preserve list line breaks in quotes, use 13-point text with 18-point leading, and show two lines plus an expand control when a compact quote is needed.
7. The calls page gives explanatory prose more visual space than the actual call choices. Keep the visible explanation to one 13-point sentence, put the full capability list under Details, and use 24-point phone/video icons with 32-point targets in recent-person rows.
8. Settings mixes control widths and vertical rhythms in a tall scrolling surface. Use a common 24-point inset and a consistent 16-point section gap. Align selects to a shared content grid, and keep a 48-point footer with a visible Done button outside the scroll area.
9. Setup uses inconsistent label-to-field spacing. Place each label 8 points above its field, put helper copy 6 points below the field, and use 24 points between field groups. Keep at least 32 points below the footer at 800-point height.
10. Meeting system events consume excessive transcript height. Use 12-16-point spacing around compact event rows and reduce the empty lead-in above the first divider to 32 points. Group meeting start/end information where possible.
11. A long draft has weak orientation feedback. Add a quiet 12-point line-count/position label above the action row, such as Lines 111-119 of 120, or a 24-point Expand composer action. Separate the action row with 8 points of padding.
12. The shortcut table has uneven key-to-description spacing. Use a consistent key column width of about 160 points on both sides, with a 16-point gutter, or align each description 16 points after its key group.
