# Independent screenshot review B

**Overall: 8/10.** A coherent, calm native desktop interface with useful empty states, persistent send-failure recovery, distinct light/dark palettes and clear primary actions. It is good rather than exemplary: narrow views truncate identity, hover controls are crowded, and important supporting instructions are too small. All 36 screen pairs (72 images) were inspected. Screenshots alone do not establish keyboard behavior, screen-reader support, animation, or working interactions. Hover strips and tooltips were treated as intentional hover states.

Dimensions: hierarchy 8; spacing 8; typography 7.5; consistency 8.5; contrast/accessibility 8; feedback 8; states 8.5; navigation 8.

## Contrast evidence

Measured actual PNG pixels with Python (`python3 -I`) and Pillow. Ratios use linearized sRGB relative luminance, (Llighter + 0.05)/(Ldarker + 0.05). Foreground core colors were distinguished from antialias edge colors. Dimmed content behind modal overlays was not treated as active text. Disabled controls and decorative separators were not treated as failed text. No sampled active text pair failed 4.5:1; no sampled essential input outline failed 3:1. This is a screenshot sample, not a blanket WCAG 2.2 conformance claim; consequently no failed-text score cap was triggered.

| Sample | Foreground / background | Ratio |
| --- | --- | --- |
| Light secondary text, chat footer and settings help | #62656D / #FFFFFF | 5.83:1 |
| Dark modal help, settings and permission dialog | #9C9EA8 / #25262B | 5.66:1 |
| Dark selected chat timestamp | #9C9EA8 / #34353C | 4.57:1 |
| Light selected preview | #4A4D55 / #E4E4E8 | 6.67:1 |
| Light filter outline | #8A8D95 / #F6F6F7 | 3.07:1 |
| Dark composer / unchecked checkbox outline | #6E7179 / #25262B | 3.09:1 |
| Light failed-send explanation | #C42B2B / #FBEEEE | 4.98:1 |
| Dark failed-send explanation | #F27474 / #281F22 | 5.75:1 |
| Light sign-in failure | #C42B2B / #FFFFFF | 5.63:1 |
| Dark sign-in failure | #F27474 / #25262B | 5.41:1 |
| Light author accent | #A9530F / #FFFFFF | 5.34:1 |
| Dark author accent | #E88C3E / #17181B | 6.98:1 |
| Light device-login link | #1F63C6 / #FFFFFF | 5.74:1 |
| Dark device-login link | #7CB4F5 / #25262B | 6.98:1 |

Representative pixel regions (x0,y0,x1,y1): selected timestamp chats (311,246,347,260); light/dark filter (76,58,350,86); composer border (380,717,1264,762); settings checkbox (382,400,399,416); failed send (431,675,818,694); device-login link (570,363,794,380). Selected fill alone has only 1.17:1 light and about 1.35:1 dark separation, but a contrasting orange edge provides redundant selection information.

## Priorities

1. **Narrow layouts sacrifice identity and reading space before adapting the secondary pane.** (both; chats-narrow, calls-narrow, channel-narrow). At 760 px, rail plus list occupies x0-296 (39% of the window). Calls names start at x128 with call controls already at x219 and x257; Katherine, Ada, Grace and Dorothy truncate. For widths below 800 pt, allow a collapsible 200 pt secondary pane; use one 28 pt call menu per row, preserving at least 140 pt for the name. Keep full identity in a visible selected header.

2. **Hover toolbars present too many equally weighted, tightly packed commands.** (both; chats, group-chat, composer-mention, composer-long-draft, composer-images, error-toast, chats-narrow). Group-chat strip spans approximately x964-1252 and y402-441 with ten commands; roughly 28 px per item, combining six emoji with reply, edit, delete and copy. Narrow chat still presents seven commands in about 204 px. Keep three reactions, Reply and a More menu; use 32 x 32 pt targets, 4 pt group gaps and a 1 pt separator before message-management commands. Put Delete inside More.

3. **Small supporting text carries consequential instructions and state.** (both; settings, setup, permission-consent, error-toast, account-menu, channel, composer-mention, shortcuts). Composer footer occupies roughly y774-783 (10 px ink height); settings explanatory paragraph is three lines at y432-471; attachment size occupies about y691-698. Palette passes: #62656D on #FFFFFF 5.83:1 and #9C9EA8 on #25262B 5.66:1. Small size, rather than failed color contrast, limits readability. Use 13 pt for actionable help and errors, 12 pt minimum for timestamps and attachment size, with 17-18 pt line height. Reserve 11 pt for optional section captions only.

4. **Explanatory prose crowds the task and weakens progressive disclosure.** (both; settings, setup, calls, calls-narrow). Settings background-refresh explanation spans y432-471; at y710 the dock-call option meets the bottom fade. Setup uses y280-325 for a paragraph, y470-495 for tenant advice, and y589-620 for two more notes. Calls-narrow explanation occupies four lines at y229-290. Keep one 13 pt explanatory sentence per group and move registration/environment details into a labelled disclosure. Use 20 pt between groups and 8 pt between label, field and helper. Reduce the initial calls explanation to two lines.

5. **Active-call controls do not clearly explain the consequence of closing the bar.** (both; call-bar, call-docked). At x239-337,y58-86 the control is labelled Close bar beside Restore. The screenshot says In the meeting, but exposes no explicit distinction between hiding this interface and leaving the call. Label the action Hide call bar and place 12 pt helper text saying Meeting continues in Teams. Rename Restore to Restore Wren window; allow wrapping into two rows at 380 pt width.

6. **Highlighting an unpicked search result competes with the actual recipient selection.** (both; new-chat). Grace Hopper is a chip at x400-543,y157-189; Barbara Liskov simultaneously has a full-width gray highlight at x400-880,y269-313. Primary action still says Chat with Grace Hopper. Add a checked marker and Selected label to recipient chips; use a 2 pt focus outline for the active search result and retain fill plus check only for added recipients. Add 12 pt Enter to add helper below results.

7. **The long draft clips a partial text line against the upper edge, and its scroll affordance crowds the action area.** (both; composer-long-draft). Focused composer spans x380-1264,y553-761; fragments of a preceding line remain around y561. Small thumb sits around x1178-1184,y738-753 beside the bottom action row. Give the text viewport 12 pt top padding and a 16 pt bottom inset; keep a separate 32 pt action row outside the scroll area. Use an 8 pt scroll thumb and 8 pt separation from actions.

8. **Selection fills are very subtle; the accent stripe is doing most of the work.** (both; chats, group-chat, chats-wide, search-switcher, channel, calendar). Measured selected chat fill #E4E4E8 versus list #F6F6F7 is 1.17:1; dark #34353C versus #1D1E22 is about 1.35:1. The orange stripe is visible, so this is a robustness concern, not a claimed WCAG selection failure. Keep the 3 pt accent stripe and add a selected check or consistent bold title. Increase fill separation to light #DDDDE3 and dark #3D3E47, retaining secondary text at #4A4D55 / #C2C4CC for comfortable contrast.

9. **Input and checkbox outlines have almost no contrast margin.** (both; chats, chats-narrow, settings, calls, calls-narrow, status-message, setup). Real border pixels: #8A8D95 on #F6F6F7 3.07:1; #6E7179 on #25262B 3.09:1. These pass the 3:1 threshold, but look thin, especially the 16 px unchecked settings checkbox. Use light border #7A7D85 and dark border #858892 with a 1 pt stroke; use 18 pt checkbox boxes within at least 28 pt rows. Preserve the existing 2 pt orange focus outline.

10. **Attachment identity is weak when aspect ratios differ dramatically.** (both; composer-images, image-preview). Composer previews are about 144 x 96 px for dimensions 3072 x 256, 256 x 256 and 256 x 3072. The first and third are strongly cropped. Preview header shows only 3072 x 256 beside Fit/100%. Show a 12 pt filename above dimensions; use contain-fit previews within 144 x 96 pt tiles and retain a 24 pt remove target. Include the filename in the image-preview header.

11. **Meeting actions are spatially detached from their supporting details.** (both; calendar). Main card occupies approximately x388-1028,y98-309, but Join is at x1199-1264,y12-40. Nearly the whole lower pane is blank. Place a 32 pt high Join button and Copy link at the bottom of the detail card with 16 pt top spacing, or align the header action group to the same 640 pt content column. Raise meeting title to 18 pt.

12. **Time-dependent states lack visible duration guidance.** (both; signin-device-code, status-message, signin-browser). Device code card at y302-572 shows code, waiting text and three buttons without a validity line. Status message dialog at y120-337 shows only text entry, Cancel and Save without a clearing policy. Browser waiting card similarly has no visible validity guidance. Add one 12-13 pt validity/expiry line below the device code and browser waiting status; show the actual status-clearing policy beneath the status input, with 8 pt separation. Do not invent a duration; display the underlying service value.

## Screen grades

| Screen | Score | Visible concerns |
| --- | --- | --- |
| chats | 8 | Hover toolbar gives six emoji equal prominence to message actions; metadata is small. Selected-row fill is subtle, though the orange edge provides a second cue. |
| chats-wide | 8 | At 1920 px the sidebar truncates the same titles despite ample horizontal space. Composer is separated from the last message by roughly 160 px of blank space. |
| chats-narrow | 7.5 | Several contact names and nearly every preview are truncated in the 232 px list. Seven small hover actions crowd the 464 px conversation pane. |
| chat-no-selection | 9 | Useful empty-state instruction; only minor small-type and subtle-selection concerns in the surrounding shell. |
| chat-filter-no-match | 8.5 | Clear recovery action, but no inline clear control at the filter itself. |
| group-chat | 8 | Hover strip contains ten small actions with weak grouping. Timestamp and reaction text are small relative to message text. |
| channel | 8 | Reply metadata is small. Long post lines and repeated inline Reply controls weaken thread scanning. |
| channel-narrow | 7.5 | Sidebar consumes 296 of 760 px including the rail. Composer placeholder uses nearly all available text width; top post begins against the header fade. |
| teams | 9 | Clear, restrained empty state; channel labels are relatively small. |
| calendar | 8 | Join sits far from the meeting-detail content. Meeting title is only modestly differentiated from secondary fields; detail pane feels underdeveloped. |
| calls | 8 | Explanatory copy and capability list dominate the pane below one short join form. Audio/video controls are repeated as small adjacent icons for every person. |
| calls-narrow | 7.5 | Recent people names truncate while two call icons reserve substantial row width. How calls work wraps into a dense paragraph that competes with the task. |
| composer-mention | 8 | Attachment size and remove mark are tiny. Suggestion highlights a person but gives no visible keyboard selection hint. |
| composer-long-draft | 7.5 | Top of the scrolled draft has a visibly clipped line inside the focused border. Draft scroll thumb is small and close to attachment/send controls. |
| composer-images | 8 | All thumbnails use the same cropped rectangle even for 12:1 and 1:12 images. Dimensions are shown, but filenames are absent from the visible attachment summary. |
| image-preview | 8.5 | Dimension metadata is small; image identity is reduced to dimensions rather than a filename. Fit and 100% are clear, but the header offers little context about which attachment is open. |
| search-switcher | 8.5 | Search footer explains implementation order in small type. Result preview lines are heavily truncated and visually close in weight to names. |
| new-chat | 8 | Grace is picked while Barbara is highlighted; the two states need clearer differentiation. Small one-line footer competes with the primary action. |
| settings | 7.5 | Multiple small explanatory paragraphs create a dense vertical form with important controls below the fold. Checkboxes are visually only about 16 px; dark unchecked outlines pass contrast narrowly. |
| shortcuts | 8.5 | Dialog extends to y788 in an 800 px window, leaving only 12 px bottom clearance. Small section captions and widely separated columns require sustained scanning. |
| status-message | 8 | Clean form, but no visible explanation of duration or when a status clears. Save is disabled without a visible concise reason. |
| account-menu | 8.5 | Account identity and presence commands share a tall, dense menu. Secondary account emails and section captions are small. |
| permission-consent | 8.5 | Useful purpose and browser action; small permission identifier text is hard to scan. Left-aligned primary action differs from the right-aligned status dialog footer. |
| error-toast | 8.5 | Persistent failed message and Retry are clear. Important error explanation is compressed into small text near the composer. |
| first-sync-loading | 8.5 | Loading is communicated clearly, but no visible recovery path if this state persists. |
| first-sync-empty | 9 | Clear first task and primary action; only minor small-type concerns in the shell. |
| call-bar | 8 | Close bar can be confused with ending the meeting. Three compact top actions compete at 380 px; no explicitly labelled return-to-call action is visible. |
| call-docked | 8 | Meeting title repeats in adjacent header bands. Close bar describes interface chrome rather than its effect on the active call. |
| join-on-phone | 9 | Readable QR and fallback copy action; minor issue: title says Join while the originating control says Join on phone. |
| setup | 7.5 | Long administrative instructions, GUID entry and environment-variable alternative crowd the first task. Tenant helper copy is small; demo access is visually detached below the card. |
| signin | 8.5 | Hover tooltip is about 510 px wide and extends well outside the 470 px card. Footer is a long line of small text. |
| signin-browser | 8.5 | Clear recovery actions; card lacks a visible indication of how long the browser handoff remains valid. |
| signin-device-code | 8.5 | Code is prominent and copyable, but no visible expiry or validity guidance. Device-login link is embedded in prose rather than paired directly with the code. |
| signin-finishing | 8.5 | Clear short progress explanation; no visible cancel or recovery affordance if finishing stalls. |
| signin-expired | 8.5 | Explains the expired account well; hover tooltip extends outside the card and overlaps its lower area. |
| signin-failed | 8.5 | Clear reason, retry and alternative sign-in; only minor small-footer and card-spacing concerns. |
