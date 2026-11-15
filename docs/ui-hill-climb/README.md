# UI hill climb

A loop of independent design reviews and improvements to Wren's interface. Each round captures every screen in light and dark mode from the demo (`capture.sh`), two graders score them independently with the same rubric, and the combined findings drive the next round of changes.

- **Grader A:** Claude Opus 5.5 (effort high), acting as a senior UI/UX designer.
- **Grader B:** Codex gpt-6.1-sol (reasoning effort high), grading independently.
- **Rubric:** each screen 1 to 10 on hierarchy, spacing, typography, consistency, contrast and accessibility (WCAG 2.2, measured from pixels), feedback, empty/loading/error states, and navigation. Neither grader sees the other's scores or earlier rounds.
- **Capture:** `docs/ui-hill-climb/capture.sh round-N` (release build, Xvfb, software GL). 36 screens per mode. The mouse rests near the window centre, so some shots show a hover state.

## Overall scores

| Round | Opus 5.5 | Codex gpt-6.1-sol | Mean |
| --- | ---: | ---: | ---: |
| 0 | 6.0 | 6.5 | 6.25 |
| 1 | 7.0 | 7.0 | 7.00 |
| 2 | 7.0 | 7.5 | 7.25 |
| 3 | 7.5 | 7.8 | 7.65 |
| 4 | 7.5 | 8.0 | 7.75 |
| 5 | 7.5 | 8.0 | 7.75 |
| 6 | 8.0 | 8.0 | 8.00 |
| 7 | 8.0 | 8.0 | 8.00 |
| 8 | 8.0 | 8.0 | 8.00 |

## Per-screen scores

Each cell is Opus / Codex.

| Screen | R0 | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| chats | 6.5 / 7 | 7 / 7 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8 |
| chats-wide | 6 / 6.5 | 6.5 / 6.5 | 7 / 7.5 | 7.5 / 7.5 | 7.5 / 8 | 7 / 7.5 | 8 / 8 | 7.5 / 8 | 8.5 / 8 |
| chats-narrow | 5 / 5.5 | 6 / 6 | 7.5 / 7 | 7 / 7 | 7.5 / 7.5 | 7.5 / 7.5 | 7.5 / 7.5 | 7.5 / 7 | 7 / 7.5 |
| chat-no-selection | 6.5 / 7 | 7 / 7 | 8.5 / 8.5 | 8 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 9 | 8.5 / 8.5 | 8.5 / 8.5 |
| chat-filter-no-match | 5.5 / 6.5 | 7 / 7 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| group-chat | 6 / 7 | 7 / 7 | 8 / 8 | 8 / 8 | 7.5 / 8.5 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8.5 |
| channel | 6.5 / 7 | 7 / 7 | 7.5 / 8 | 7.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8.5 / 8 | 8 / 8.5 |
| channel-narrow | 5.5 / 5.5 | 6 / 6 | 7 / 7 | 6.5 / 7 | 7.5 / 7.5 | 7 / 7.5 | 7.5 / 7.5 | 7.5 / 7.5 | 7.5 / 8 |
| teams | 6.5 / 7 | 7.5 / 8 | 8 / 8.5 | 7.5 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 8.5 |
| calendar | 6.5 / 7 | 7.5 / 8 | 7 / 8 | 7 / 8 | 7.5 / 8 | 7 / 8 | 7.5 / 8 | 7 / 8 | 7.5 / 8.5 |
| calls | 6 / 6.5 | 6.5 / 7 | 6.5 / 7.5 | 7 / 8 | 7 / 8 | 7.5 / 8 | 7.5 / 8 | 7.5 / 8 | 7.5 / 8 |
| calls-narrow | 3.5 / 3.5 | 6 / 6 | 6 / 7 | 6.5 / 7 | 6.5 / 7.5 | 7 / 7.5 | 7 / 7.5 | 7 / 7.5 | 7 / 7.5 |
| composer-mention | 6.5 / 6.5 | 7 / 7 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 7.5 / 8.5 |
| composer-long-draft | 5.5 / 6 | 6 / 6.5 | 6.5 / 7.5 | 6.5 / 7.5 | 7 / 7.5 | 7 / 7 | 6.5 / 7.5 | 7.5 / 7.5 | 8 / 7.5 |
| composer-images | 3.5 / 5.5 | 6.5 / 7 | 6.5 / 7.5 | 6.5 / 7.5 | 7 / 8 | 7.5 / 7.5 | 7 / 8 | 7 / 7.5 | 7 / 8 |
| image-preview | 5.5 / 7 | 7 / 8 | 6.5 / 8 | 7 / 8 | 7 / 8.5 | 7.5 / 8 | 8 / 8.5 | 7.5 / 8 | 8 / 8 |
| search-switcher | 6 / 6.5 | 7 / 7 | 7.5 / 8.5 | 7 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 |
| new-chat | 6 / 6.5 | 7.5 / 8 | 7.5 / 8 | 7.5 / 7.5 | 8 / 8 | 8 / 7.5 | 8 / 8 | 8 / 8 | 8 / 7.5 |
| settings | 4.5 / 6 | 6 / 7.5 | 6 / 7.5 | 7.5 / 7 | 7.5 / 8 | 8 / 7.5 | 8 / 7.5 | 7.5 / 7.5 | 8 / 7.5 |
| shortcuts | 6 / 6.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 6.5 / 8 | 7.5 / 8.5 | 8.5 / 8.5 | 7.5 / 8 | 7.5 / 8.5 |
| status-message | 6.5 / 7 | 6.5 / 7 | 7.5 / 8 | 7 / 8 | 7 / 9 | 8 / 8.5 | 8 / 8 | 8.5 / 8.5 | 8.5 / 8.5 |
| account-menu | 5.5 / 6.5 | 6.5 / 7 | 8 / 8 | 7.5 / 8 | 8 / 8.5 | 8.5 / 8 | 8.5 / 8.5 | 8.5 / 8 | 8.5 / 8 |
| permission-consent | 6.5 / 7 | 8 / 8 | 8 / 8 | 8 / 8 | 7.5 / 8.5 | 8.5 / 8 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 |
| error-toast | 5.5 / 6 | 5.5 / 6 | 5 / 6.5 | 7.5 / 8 | 7.5 / 8 | 7 / 8 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| first-sync-loading | 5 / 6.5 | 7 / 7 | 8.5 / 8.5 | 8 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 |
| first-sync-empty | 5.5 / 6 | 7 / 7 | 7.5 / 8.5 | 7.5 / 8.5 | 7 / 8.5 | 8 / 8.5 | 8 / 9 | 7.5 / 9 | 8 / 8.5 |
| call-bar | 6 / 6.5 | 5.5 / 7 | 6.5 / 7 | 6.5 / 7.5 | 7 / 8.5 | 7.5 / 8 | 7 / 8 | 8 / 8 | 7.5 / 8.5 |
| call-docked | 6 / 7 | 6 / 7 | 7 / 7.5 | 7.5 / 8 | 7.5 / 8.5 | 8 / 8 | 7.5 / 8 | 8 / 8.5 | 7.5 / 8.5 |
| join-on-phone | 7 / 8 | 7.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 9 | 8 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 8 |
| setup | 6 / 6.5 | 6.5 / 7 | 7 / 7 | 7.5 / 8 | 7.5 / 8 | 7.5 / 7.5 | 7.5 / 7.5 | 8 / 7.5 | 7.5 / 7.5 |
| signin | 6.5 / 7 | 7.5 / 8 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 8 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 |
| signin-browser | 6 / 7 | 7.5 / 8 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 |
| signin-device-code | 6 / 7 | 7.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 9 | 8.5 / 8 | 8.5 / 8.5 | 8.5 / 9 | 8.5 / 8.5 |
| signin-finishing | 6.5 / 6.5 | 8 / 8 | 8 / 8.5 | 8.5 / 8.5 | 8 / 9 | 8 / 8.5 | 8 / 8.5 | 7.5 / 8.5 | 8.5 / 8.5 |
| signin-expired | 6.5 / 7 | 7.5 / 8 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 |
| signin-failed | 6.5 / 7 | 7.5 / 8 | 8 / 8.5 | 8 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 |

## Dimension scores

| Dimension | R0 | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| hierarchy | 7 / 7 | 7.5 / 7 | 7.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8.5 / 8 | 8.5 / 8 |
| spacing | 6 / 6 | 6.5 / 6.5 | 7 / 7.5 | 7 / 7.5 | 7 / 8 | 7.5 / 7.5 | 7.5 / 8 | 7.5 / 8 | 7.5 / 8 |
| typography | 6.5 / 6.5 | 6.5 / 7 | 6.5 / 7.5 | 7.5 / 7.5 | 7 / 8 | 7 / 8 | 8 / 7.5 | 8.5 / 8 | 8 / 7.5 |
| consistency | 5.5 / 7.5 | 7 / 8 | 6.5 / 8 | 7 / 8.5 | 6.5 / 8.5 | 7.5 / 8.5 | 7.5 / 8.5 | 7.5 / 8.5 | 7.5 / 9 |
| contrast_accessibility | 4.5 / 4 | 6.5 / 6 | 7.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8.5 | 8.5 / 8 | 8.5 / 8.5 | 8.5 / 8.5 |
| feedback | 6 / 6 | 6.5 / 6.5 | 7 / 7 | 7.5 / 8 | 8 / 8 | 8 / 7.5 | 7.5 / 8 | 8.5 / 7.5 | 8.5 / 8 |
| states | 5.5 / 6.5 | 7.5 / 8 | 7.5 / 8 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 |
| navigation | 6.5 / 6.5 | 7 / 7 | 7.5 / 7.5 | 7 / 8 | 7.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 7.5 / 8 |

## Rounds

### Round 0

Scores: Opus 6.0, Codex 6.5.

- Baseline after removing every theme but light, dark and System.

Top findings from this round, which the next round worked on:

- Opus, high: The tertiary text colour fails WCAG 4.5:1 everywhere: timestamps, list dates, section labels, helper text, emails, hints and footers.
- Opus, high: The light accent #c26318 is too light for text and for white button labels. Light-mode success green also fails.
- Opus, high: At 760pt the Calls page clips horizontally: the intro text, the Join button and the capability descriptions are cut off.
- Opus, high: Pasted-image thumbnails render broken: mixed sizes, bad crops, a clipped strip and detached remove buttons.
- Opus, high: Settings dialog is broken: 'Leave demo' is clipped by the scrollbar, unchecked checkboxes are nearly invisible, and dropdown labels sit after their controls.
- Codex, high: Narrow Calls layout clips its primary task.
- Codex, high: Muted informational text repeatedly fails the 4.5:1 text requirement.
- Codex, high: White text on orange primary buttons fails body-text contrast.
- Codex, high: Fixed sidebar and hover action placement compromise message reading.
- Codex, medium: Orange small text fails contrast on white and tinted surfaces.

### Round 1

Scores: Opus 7.0, Codex 7.0.

- Colour tokens: secondary and dim text reach 4.5:1 on every surface in both modes, including selected rows. The light accent darkens from #c26318 to #a9530f, so white button labels pass. Success, warning and danger pass too. A unit test enforces all of it.
- A new `control` token gives text fields, checkboxes and secondary buttons a 3:1 edge, and focused fields get a 2 pt accent ring.
- Avatar colours are re-tuned so white initials reach 4.5:1.
- Shortcut hints use the platform's notation everywhere (Ctrl+K on Linux and Windows, ⌘K on macOS). The shortcuts sheet shows key caps and a close button.
- Empty states fit inside narrow panes (the sidebar no longer widens) and offer the next step: New chat on a first start, Clear filter when nothing matches, skeleton rows while chats load, and Teams-specific copy and icons.
- Settings: content clears the scroll bar, drop-down labels sit above their controls, and the zoom hint is quieter. All dialog titles share one size (20 pt bold).
- Calls: the page reflows at narrow widths instead of clipping, capabilities are listed with icons, Join is the primary button, and the copy is fixed ("in its own Edge window").
- Composer: pasted-image thumbnails sit on tiles with a backdrop and border, with the remove button on each tile's corner. Message rows tint on hover. Mention rows get a 3 pt accent edge, and the dark mention tint is calmer.
- Sign-in: a spinner while waiting for the browser or code, the device code in a frame with a clickable link, Try again after a failure, and the account shown on its own line when a session expires. The demo button is hidden while sign-in runs. The setup form explains why Save is disabled.

Top findings from this round, which the next round worked on:

- Opus, high: Placeholder text fails WCAG 1.4.3 in light mode in every input (composer, filter, search, status, setup)
- Opus, high: Failed send is shown only as a transient toast that covers the last message, sits off-centre, and has no Retry
- Opus, high: Settings checkboxes look like radio buttons and the unchecked state fails 3:1
- Opus, high: Call bar contradicts the chat and wastes space
- Opus, medium: Hover action toolbar covers message author lines and reactions
- Codex, high: Narrow layouts preserve a desktop-width sidebar at the expense of the task pane.
- Codex, high: Tinted-input placeholder text narrowly fails WCAG 2.2 body-text contrast.
- Codex, high: White-field placeholder text also fails the 4.5:1 threshold.
- Codex, high: Failed-send feedback does not provide a visible recovery path tied to the failed message.
- Codex, high: Hover action strips obscure sender metadata.

### Round 2

Scores: Opus 7.0, Codex 7.5.

- Placeholders use the dim token, so they reach 4.5:1. The composer gets a 3:1 edge and a 2 pt accent focus ring. Text fields and the Join field match the 32 pt button height.
- A failed send is marked on the message itself: an alert icon, Not sent and a Retry button. The demo's error state now reproduces a real failed send, not a toast alone.
- Settings uses square checkboxes with a 3:1 edge, an accent fill when checked and a focus ring, replacing egui's round ones. A UI test covers toggling from the label.
- Primary buttons are 32 pt, the same as secondary ones. Disabled, they turn neutral instead of faded orange.
- Narrow windows: below 960 pt the sidebar narrows to 232 pt without overwriting the saved width, and the hover toolbar shows three reactions in narrow panes. Wide windows: the message column stops at 920 pt.
- Presence dots differ by shape as well as colour: a bar for Do not disturb, a hollow centre for Away and Be right back, a ring for Offline.
- Call bar: title and status sit together, and Done becomes a neutral Close bar.
- A hairline under the conversation header. Dark dialogs get a lighter edge. Account-menu rows share one icon column, with a badge for other accounts' unread chats. The switcher footer shows key caps. Empty states are centred. Calls puts Join first.
- The long-draft composer keeps Send and Attach at the bottom and shows its scroll bar.

Top findings from this round, which the next round worked on:

- Opus, high: The error toast covers the failed message, including its 'Not sent' label and Retry button, which are the recovery path it describes.
- Opus, high: 'Show me as available while I use Wren' renders a ☑ text glyph instead of the checkbox widget, misaligned with every other checkbox.
- Opus, medium: Hyphens render with wide side bearings across the app, so 'On-call' reads 'On - call', and 'sign-in', 'go/no-go', 'full-text', 'third-party' and 'F4KQ-9TZW' are affected too.
- Opus, medium: Selected rows are shown only by a near-invisible fill, which fails the 3:1 non-text contrast guidance for a state indicator.
- Opus, medium: The Calendar and Calls detail panes have no top header bar, while Chat and Channel have a 52 pt bar with title and actions. The frame jumps between sections.
- Codex, high: Error toast obscures the failed message and its recovery action.
- Codex, medium: Fixed navigation consumes too much of a narrow window and truncates identities.
- Codex, medium: Live call status conflicts with the visible ended-meeting state.
- Codex, medium: The first-run hierarchy foregrounds technical configuration over the step that makes it possible.
- Codex, medium: Calls screen reads like a capability document instead of an action surface.

### Round 3

Scores: Opus 7.5, Codex 7.8.

- Toasts appear at the top of the window, so a failed message and its Retry button stay visible above the composer.
- Calendar and Calls share the 52 pt header bar that conversations use: title, subtitle, right-aligned actions and a hairline. The meeting's Join and Copy link move into it.
- Selected chats, channels and switcher results get a 3 pt accent bar, so selection no longer rests on a fill alone. Switcher results show initials for people and chats.
- The composer matches the message column's width. Its hint is 12 pt and shorter.
- Calls: two lines on how calls work, a one-line summary of what works and what doesn't, and the details behind a disclosure.
- Channel posts get a Reply button with an icon. The last Settings checkbox now uses the shared widget. The status dialog enables Save only after a change and offers Remove status only when one is set. Setup shows the registration guide before the fields. Settings copy is plainer.
- Demo: joining the meeting adds today's Meeting started line, so the call bar no longer contradicts yesterday's Meeting ended.

Top findings from this round, which the next round worked on:

- Opus, high: Narrow layouts do not adapt: the list pane stays ~232 px, names truncate, the composer placeholder wraps with a dangling '·' and the helper wraps onto 2-3 lines.
- Opus, high: Call layouts truncate the chat title early and remove all navigation.
- Opus, high: Long drafts are cramped and clipped: tight leading, no top padding, scrollbar beside the attach icon.
- Opus, medium: Secondary buttons, reaction chips, hover toolbar and cards rely on borders far below 3:1, so secondary controls barely read as buttons, especially in dark mode.
- Opus, medium: Avatar colours and icons in the switcher do not match the chat list, breaking recognition.
- Codex, medium: Fixed navigation widths squeeze the task pane and truncate identity information.
- Codex, medium: The scrollbar thumb and track do not meet the 3:1 non-text contrast threshold.
- Codex, medium: Helper and metadata text is too small for comfortable desktop reading, even though sampled colors pass contrast.
- Codex, medium: Reading and composing use different horizontal constraints on wide windows.
- Codex, medium: The long-draft viewport cuts through the first visible line and packs many lines tightly against its edges.

### Round 4

Scores: Opus 7.5, Codex 8.0.

- The composer now stops at the message column on wide windows (round 3's cap had no effect: the bottom panel stretches its Ui, so the composer now draws into a sized child). It uses a 20 pt line height, and its hints shorten in narrow panes so they stay on one line.
- The switcher draws each chat with the chat list's own avatar (person, group initials or meeting tile), so colours and icons match.
- Secondary buttons get the 3:1 control edge. Drop-downs and disclosures use Lucide chevrons. The Settings scroll bar is 8 pt with a 3:1 thumb.
- The status and permission dialogs get close buttons. The permission dialog leads with the capability and puts the Graph scope last, in small print.
- Failed messages get a light danger tint. The conversation title uses the width its header buttons leave instead of a fixed 260 pt reserve, so docked meeting titles no longer truncate early.
- The shortcuts sheet is grouped (Navigation, Messages, App), and the account menu gets Accounts and Status headings.
- First start: the list shows no filter field or duplicate empty text while there's nothing to list. The sign-in page sits higher, so setup fits without scrolling.

Top findings from this round, which the next round worked on:

- Opus, high: Buttons have no single system: filled primaries render ~15pt bold while outlined secondaries are 13pt; secondary buttons come in three styles (outlined #8a8d95, filled borderless pill #eeeef0 e.g. 'Copy link'/'Restore', and faint-bordered #dfdfe3 'Reply'/reaction chips); disabled Save/Join are 15pt grey slabs.
- Opus, high: Message hover toolbar overlaps neighbouring rows (mention highlight above, previous reaction chips, the post's own name row).
- Opus, medium: Shortcut descriptions are not aligned on one column; each section sizes its own key column.
- Opus, medium: Hyphens render with wide side bearings across the app ('On - call', 'go/no - go', 'Step - by - step', 'Sign - in', 'F4KQ - 9TZW').
- Opus, medium: Interactive outlines below 3:1: hover toolbar, reaction chips and Reply buttons.
- Codex, medium: Reply-button borders do not meet the requested 3:1 component-boundary contrast. Their labels remain readable.
- Codex, medium: Narrow layouts spend too much width on navigation while still truncating important identities.
- Codex, medium: The scrolled draft clips the first visible line against the editor's top edge.
- Codex, medium: Pasted-image thumbnails crop away content needed to verify attachments before sending.
- Codex, medium: People search loses its purpose label once the user types; the existing-chat explanation is too compressed.

### Round 5

Scores: Opus 7.5, Codex 8.0.

- One button style: primary and secondary buttons share 13 pt semibold labels and a 32 pt height. Reply buttons, reaction chips and the hover toolbar get the 3:1 control edge.
- The hover toolbar sits inside the hovered row instead of across its neighbours. On wide windows the conversation header's actions end where the message column ends.
- Toasts centre on the main pane, not the window. The sidebar keeps its width when its list is empty. The header spinner stays hidden while the first chat list loads, since the pane shows progress.
- A scrolled long draft shows whole lines: the editor's height is a whole number of 20 pt lines. The composer hint is shorter.
- The shortcuts sheet is one grid, so descriptions align across groups. New chat labels its field (To / Add someone else). Email addresses in mention suggestions and people search are 12 pt.
- Calls: the link field matches the Join button's height. Summary lines hang under their text. Limits read as neutral information, not as errors.
- Image preview uses a segmented Fit / 100% control. Pasted images show their dimensions. The status dialog uses a one-line field with Cancel and Save on the right, and Enter saves. Be right back shows clock hands. The sign-in group is centred vertically.

Top findings from this round, which the next round worked on:

- Opus, medium: Hyphens render with wide side gaps across the app ('On - call', 'Sign - in', 'Ad - hoc', 'go/no - go', 'Step - by - step', 'F4KQ - 9TZW', 'open - source').
- Opus, medium: At 1920 wide the conversation column is capped at about 920 px and left-aligned, leaving a ~600 px dead band. Header actions stop at x≈1290 while the header rule runs to 1920.
- Opus, medium: Error toast is placed over the conversation's top message and covers its text. It also repeats the inline failure without its own action.
- Opus, medium: The orange left bar marks the selected row in Chat/Teams lists but appears on every Calendar row. Calendar selection is only a fill with 1.17:1 against the list.
- Opus, medium: Mixed button language: filled pills with almost no boundary (Copy link, call bar buttons, theme segment) sit next to 6 px-radius outlined buttons (Reply, Cancel, Leave demo). Pill buttons fail 3:1 for component boundaries.
- Codex, high: Long draft hides the send and attachment controls.
- Codex, medium: Narrow layout preserves too much navigation width and truncates important identities.
- Codex, medium: Attachment crops make extreme-aspect images hard to recognize.
- Codex, medium: Wide conversation is left-heavy and its header does not align with its content column.
- Codex, medium: Technical setup lacks a strong guided action hierarchy.

### Round 6

Scores: Opus 8.0, Codex 8.0.

- On wide windows the conversation header, history and composer share one centred 920 pt reading column.
- A failed send keeps its reason on the message (Not sent: reason, then Retry). The toast appears only when the failure is in a chat that isn't open, and retrying clears the reason. The new Message.failure field is optional, so existing caches keep loading. A unit test covers both paths.
- Calendar rows mark the kind of meeting with a dot (live, online, other), leaving the accent bar to mean selection everywhere.
- Pill buttons (Copy link, the call bar, the theme choice) get the 3:1 control edge.
- History fades out under the header instead of cutting messages off, and the hover toolbar no longer shows under dialogs.
- Setup: the registration guide is a button, and Save reads Save and continue on first run. The code sign-in tooltip is one plain sentence. The status field is labelled.

Top findings from this round, which the next round worked on:

- Opus, high: Long draft composer shows clipped glyph fragments at the top edge and a stray scrollbar stub beside the attach icon; no caret visible.
- Opus, high: Call bar duplicates the meeting title, has no leave/hang-up or duration, and the 'Restore' and 'Close bar' labels are ambiguous; the 'Meeting ended' line attaches to the preceding message.
- Opus, medium: Text fields don't show focus. Only the composer gets the 2pt accent ring, so a field that is being typed in looks idle.
- Opus, medium: Pending-image thumbnails are centre-cropped, so wide or tall images are misrepresented, and remove buttons cover the image corners.
- Opus, medium: At 760pt width the list pane keeps its full 232pt; names truncate heavily and the conversation gets only 464pt.
- Codex, medium: Narrow layouts sacrifice identity and reading space before adapting the secondary pane.
- Codex, medium: Hover toolbars present too many equally weighted, tightly packed commands.
- Codex, medium: Small supporting text carries consequential instructions and state.
- Codex, medium: Explanatory prose crowds the task and weakens progressive disclosure.
- Codex, medium: Active-call controls do not clearly explain the consequence of closing the bar.

### Round 7

Scores: Opus 8.0, Codex 8.0.

- Capture fix: screenshot mode treats the window as focused, so focus rings and the caret show as they do in use. Under Xvfb with no window manager the window never gets focus, and egui only draws focus while the window has it. The composer's own focus ring follows window focus the same way egui's fields do.
- A scrolled long draft fades out at its top edge instead of showing a cut line. Its scroll bar no longer sits as a stub next to the icons.
- Docked beside a call, the call bar shows only the call's state, since the meeting chat's header right below names the meeting. Close bar reads Hide call bar (docs/calling.md updated).
- System lines (Meeting started, Meeting ended) get room above and below.
- The hover toolbar separates reactions from message actions with a divider.
- A narrow Calls list keeps names whole by dropping the video button. Video calls stay one click away in the chat header.
- The Settings note on when messages refresh is 13 pt in the secondary colour.

Top findings from this round, which the next round worked on:

- Opus, medium: Primary (accent) buttons carry the neutral control stroke around the accent fill, so their edge reads muddy; the calendar header Join pill has no stroke, so primaries are inconsistent.
- Opus, medium: Narrow (760pt) layouts keep a 232pt side pane, truncating names and wrapping posts at about 380pt. Calls silently drops the video button and the hover toolbar collides with header rows.
- Opus, medium: At 1920x1080 the conversation is top-anchored, leaving a large dead gap above the composer; chat apps bottom-anchor short histories.
- Opus, medium: The settings scroll area cuts a row mid-glyph at the bottom edge, and the background-refresh help paragraph is attached to the tray option instead of notifications.
- Opus, medium: Meeting detail is thin and oddly placed: a fixed 640pt left-aligned card, redundant Location/Online rows, no attendees, and header buttons in a different shape and size from the app's buttons.
- Codex, medium: Narrow layouts reserve too much width for the sidebar and truncate identities.
- Codex, medium: Video calling loses its visible entry point at narrow width.
- Codex, medium: Attachment thumbnails conceal the contents of extreme aspect-ratio images.
- Codex, medium: Long draft overflow has no visible position cue.
- Codex, medium: Search highlight and committed recipients are easy to confuse.

### Round 8

Scores: Opus 8.0, Codex 8.0.

- Primary buttons drop egui's grey control edge. Pills are 32 pt, like every other button.
- A conversation shorter than its pane sits at the bottom, next to the composer, instead of leaving a gap above it.
- New chat marks the keyboard's choice with the accent bar and an Enter to add hint, so it reads differently from people already added.
- A narrow Calls list keeps both call buttons, made smaller.
- Calendar leaves out a Location that only repeats the Online row.
- Settings puts the note on when messages arrive under the notifications option it explains.

Remaining top findings:

- Opus, high: Side list keeps its full 232pt width in a 760pt window, so names truncate after 9-12 characters and the conversation column shrinks to ~460pt.
- Opus, medium: Detail pages use left-aligned fixed-width cards with different widths (Calendar 640pt, Calls 680pt) while conversations use a centred column, so pages feel unrelated and leave ~250pt dead space on the right.
- Opus, medium: Dialog footers are inconsistent: primary bottom-left with helper text (New chat), primary-left then secondary (Permission), right-aligned Cancel/Save (Status message), lone left-aligned secondary (Join on phone).
- Opus, medium: In meeting mode the navigation rail and chat list vanish, so there is no way to answer another chat without leaving the meeting layout; 'Restore' is ambiguous.
- Opus, medium: Pasted-image thumbnails are cropped to fill, so wide or tall images are unrecognisable, and the remove buttons sit on the image.
- Codex, medium: Narrow layouts truncate people’s identities while retaining permanent navigation and action columns.
- Codex, medium: The long draft has no clear indication of its hidden beginning.
- Codex, medium: The active search result and primary action refer to different people.
- Codex, medium: Settings is a long flat scroll surface with weak wayfinding.
- Codex, medium: Technical setup instructions dilute the main task.

## Screenshots

Every round keeps its screenshots under `round-N/light/` and `round-N/dark/`. Main screens, first and last round:

| Screen | round-0 light | round-8 light | round-0 dark | round-8 dark |
| --- | --- | --- | --- | --- |
| chats | [before](round-0/light/chats.png) | [after](round-8/light/chats.png) | [before](round-0/dark/chats.png) | [after](round-8/dark/chats.png) |
| channel | [before](round-0/light/channel.png) | [after](round-8/light/channel.png) | [before](round-0/dark/channel.png) | [after](round-8/dark/channel.png) |
| calendar | [before](round-0/light/calendar.png) | [after](round-8/light/calendar.png) | [before](round-0/dark/calendar.png) | [after](round-8/dark/calendar.png) |
| calls | [before](round-0/light/calls.png) | [after](round-8/light/calls.png) | [before](round-0/dark/calls.png) | [after](round-8/dark/calls.png) |
| settings | [before](round-0/light/settings.png) | [after](round-8/light/settings.png) | [before](round-0/dark/settings.png) | [after](round-8/dark/settings.png) |
| signin | [before](round-0/light/signin.png) | [after](round-8/light/signin.png) | [before](round-0/dark/signin.png) | [after](round-8/dark/signin.png) |
| first-sync-loading | [before](round-0/light/first-sync-loading.png) | [after](round-8/light/first-sync-loading.png) | [before](round-0/dark/first-sync-loading.png) | [after](round-8/dark/first-sync-loading.png) |
| chats-narrow | [before](round-0/light/chats-narrow.png) | [after](round-8/light/chats-narrow.png) | [before](round-0/dark/chats-narrow.png) | [after](round-8/dark/chats-narrow.png) |

### All screens

<details><summary>round-0</summary>

**light:** [account-menu](round-0/light/account-menu.png) · [calendar](round-0/light/calendar.png) · [call-bar](round-0/light/call-bar.png) · [call-docked](round-0/light/call-docked.png) · [calls-narrow](round-0/light/calls-narrow.png) · [calls](round-0/light/calls.png) · [channel-narrow](round-0/light/channel-narrow.png) · [channel](round-0/light/channel.png) · [chat-filter-no-match](round-0/light/chat-filter-no-match.png) · [chat-no-selection](round-0/light/chat-no-selection.png) · [chats-narrow](round-0/light/chats-narrow.png) · [chats-wide](round-0/light/chats-wide.png) · [chats](round-0/light/chats.png) · [composer-images](round-0/light/composer-images.png) · [composer-long-draft](round-0/light/composer-long-draft.png) · [composer-mention](round-0/light/composer-mention.png) · [error-toast](round-0/light/error-toast.png) · [first-sync-empty](round-0/light/first-sync-empty.png) · [first-sync-loading](round-0/light/first-sync-loading.png) · [group-chat](round-0/light/group-chat.png) · [image-preview](round-0/light/image-preview.png) · [join-on-phone](round-0/light/join-on-phone.png) · [new-chat](round-0/light/new-chat.png) · [permission-consent](round-0/light/permission-consent.png) · [search-switcher](round-0/light/search-switcher.png) · [settings](round-0/light/settings.png) · [setup](round-0/light/setup.png) · [shortcuts](round-0/light/shortcuts.png) · [signin-browser](round-0/light/signin-browser.png) · [signin-device-code](round-0/light/signin-device-code.png) · [signin-expired](round-0/light/signin-expired.png) · [signin-failed](round-0/light/signin-failed.png) · [signin-finishing](round-0/light/signin-finishing.png) · [signin](round-0/light/signin.png) · [status-message](round-0/light/status-message.png) · [teams](round-0/light/teams.png)

**dark:** [account-menu](round-0/dark/account-menu.png) · [calendar](round-0/dark/calendar.png) · [call-bar](round-0/dark/call-bar.png) · [call-docked](round-0/dark/call-docked.png) · [calls-narrow](round-0/dark/calls-narrow.png) · [calls](round-0/dark/calls.png) · [channel-narrow](round-0/dark/channel-narrow.png) · [channel](round-0/dark/channel.png) · [chat-filter-no-match](round-0/dark/chat-filter-no-match.png) · [chat-no-selection](round-0/dark/chat-no-selection.png) · [chats-narrow](round-0/dark/chats-narrow.png) · [chats-wide](round-0/dark/chats-wide.png) · [chats](round-0/dark/chats.png) · [composer-images](round-0/dark/composer-images.png) · [composer-long-draft](round-0/dark/composer-long-draft.png) · [composer-mention](round-0/dark/composer-mention.png) · [error-toast](round-0/dark/error-toast.png) · [first-sync-empty](round-0/dark/first-sync-empty.png) · [first-sync-loading](round-0/dark/first-sync-loading.png) · [group-chat](round-0/dark/group-chat.png) · [image-preview](round-0/dark/image-preview.png) · [join-on-phone](round-0/dark/join-on-phone.png) · [new-chat](round-0/dark/new-chat.png) · [permission-consent](round-0/dark/permission-consent.png) · [search-switcher](round-0/dark/search-switcher.png) · [settings](round-0/dark/settings.png) · [setup](round-0/dark/setup.png) · [shortcuts](round-0/dark/shortcuts.png) · [signin-browser](round-0/dark/signin-browser.png) · [signin-device-code](round-0/dark/signin-device-code.png) · [signin-expired](round-0/dark/signin-expired.png) · [signin-failed](round-0/dark/signin-failed.png) · [signin-finishing](round-0/dark/signin-finishing.png) · [signin](round-0/dark/signin.png) · [status-message](round-0/dark/status-message.png) · [teams](round-0/dark/teams.png)

</details>

<details><summary>round-1</summary>

**light:** [account-menu](round-1/light/account-menu.png) · [calendar](round-1/light/calendar.png) · [call-bar](round-1/light/call-bar.png) · [call-docked](round-1/light/call-docked.png) · [calls-narrow](round-1/light/calls-narrow.png) · [calls](round-1/light/calls.png) · [channel-narrow](round-1/light/channel-narrow.png) · [channel](round-1/light/channel.png) · [chat-filter-no-match](round-1/light/chat-filter-no-match.png) · [chat-no-selection](round-1/light/chat-no-selection.png) · [chats-narrow](round-1/light/chats-narrow.png) · [chats-wide](round-1/light/chats-wide.png) · [chats](round-1/light/chats.png) · [composer-images](round-1/light/composer-images.png) · [composer-long-draft](round-1/light/composer-long-draft.png) · [composer-mention](round-1/light/composer-mention.png) · [error-toast](round-1/light/error-toast.png) · [first-sync-empty](round-1/light/first-sync-empty.png) · [first-sync-loading](round-1/light/first-sync-loading.png) · [group-chat](round-1/light/group-chat.png) · [image-preview](round-1/light/image-preview.png) · [join-on-phone](round-1/light/join-on-phone.png) · [new-chat](round-1/light/new-chat.png) · [permission-consent](round-1/light/permission-consent.png) · [search-switcher](round-1/light/search-switcher.png) · [settings](round-1/light/settings.png) · [setup](round-1/light/setup.png) · [shortcuts](round-1/light/shortcuts.png) · [signin-browser](round-1/light/signin-browser.png) · [signin-device-code](round-1/light/signin-device-code.png) · [signin-expired](round-1/light/signin-expired.png) · [signin-failed](round-1/light/signin-failed.png) · [signin-finishing](round-1/light/signin-finishing.png) · [signin](round-1/light/signin.png) · [status-message](round-1/light/status-message.png) · [teams](round-1/light/teams.png)

**dark:** [account-menu](round-1/dark/account-menu.png) · [calendar](round-1/dark/calendar.png) · [call-bar](round-1/dark/call-bar.png) · [call-docked](round-1/dark/call-docked.png) · [calls-narrow](round-1/dark/calls-narrow.png) · [calls](round-1/dark/calls.png) · [channel-narrow](round-1/dark/channel-narrow.png) · [channel](round-1/dark/channel.png) · [chat-filter-no-match](round-1/dark/chat-filter-no-match.png) · [chat-no-selection](round-1/dark/chat-no-selection.png) · [chats-narrow](round-1/dark/chats-narrow.png) · [chats-wide](round-1/dark/chats-wide.png) · [chats](round-1/dark/chats.png) · [composer-images](round-1/dark/composer-images.png) · [composer-long-draft](round-1/dark/composer-long-draft.png) · [composer-mention](round-1/dark/composer-mention.png) · [error-toast](round-1/dark/error-toast.png) · [first-sync-empty](round-1/dark/first-sync-empty.png) · [first-sync-loading](round-1/dark/first-sync-loading.png) · [group-chat](round-1/dark/group-chat.png) · [image-preview](round-1/dark/image-preview.png) · [join-on-phone](round-1/dark/join-on-phone.png) · [new-chat](round-1/dark/new-chat.png) · [permission-consent](round-1/dark/permission-consent.png) · [search-switcher](round-1/dark/search-switcher.png) · [settings](round-1/dark/settings.png) · [setup](round-1/dark/setup.png) · [shortcuts](round-1/dark/shortcuts.png) · [signin-browser](round-1/dark/signin-browser.png) · [signin-device-code](round-1/dark/signin-device-code.png) · [signin-expired](round-1/dark/signin-expired.png) · [signin-failed](round-1/dark/signin-failed.png) · [signin-finishing](round-1/dark/signin-finishing.png) · [signin](round-1/dark/signin.png) · [status-message](round-1/dark/status-message.png) · [teams](round-1/dark/teams.png)

</details>

<details><summary>round-2</summary>

**light:** [account-menu](round-2/light/account-menu.png) · [calendar](round-2/light/calendar.png) · [call-bar](round-2/light/call-bar.png) · [call-docked](round-2/light/call-docked.png) · [calls-narrow](round-2/light/calls-narrow.png) · [calls](round-2/light/calls.png) · [channel-narrow](round-2/light/channel-narrow.png) · [channel](round-2/light/channel.png) · [chat-filter-no-match](round-2/light/chat-filter-no-match.png) · [chat-no-selection](round-2/light/chat-no-selection.png) · [chats-narrow](round-2/light/chats-narrow.png) · [chats-wide](round-2/light/chats-wide.png) · [chats](round-2/light/chats.png) · [composer-images](round-2/light/composer-images.png) · [composer-long-draft](round-2/light/composer-long-draft.png) · [composer-mention](round-2/light/composer-mention.png) · [error-toast](round-2/light/error-toast.png) · [first-sync-empty](round-2/light/first-sync-empty.png) · [first-sync-loading](round-2/light/first-sync-loading.png) · [group-chat](round-2/light/group-chat.png) · [image-preview](round-2/light/image-preview.png) · [join-on-phone](round-2/light/join-on-phone.png) · [new-chat](round-2/light/new-chat.png) · [permission-consent](round-2/light/permission-consent.png) · [search-switcher](round-2/light/search-switcher.png) · [settings](round-2/light/settings.png) · [setup](round-2/light/setup.png) · [shortcuts](round-2/light/shortcuts.png) · [signin-browser](round-2/light/signin-browser.png) · [signin-device-code](round-2/light/signin-device-code.png) · [signin-expired](round-2/light/signin-expired.png) · [signin-failed](round-2/light/signin-failed.png) · [signin-finishing](round-2/light/signin-finishing.png) · [signin](round-2/light/signin.png) · [status-message](round-2/light/status-message.png) · [teams](round-2/light/teams.png)

**dark:** [account-menu](round-2/dark/account-menu.png) · [calendar](round-2/dark/calendar.png) · [call-bar](round-2/dark/call-bar.png) · [call-docked](round-2/dark/call-docked.png) · [calls-narrow](round-2/dark/calls-narrow.png) · [calls](round-2/dark/calls.png) · [channel-narrow](round-2/dark/channel-narrow.png) · [channel](round-2/dark/channel.png) · [chat-filter-no-match](round-2/dark/chat-filter-no-match.png) · [chat-no-selection](round-2/dark/chat-no-selection.png) · [chats-narrow](round-2/dark/chats-narrow.png) · [chats-wide](round-2/dark/chats-wide.png) · [chats](round-2/dark/chats.png) · [composer-images](round-2/dark/composer-images.png) · [composer-long-draft](round-2/dark/composer-long-draft.png) · [composer-mention](round-2/dark/composer-mention.png) · [error-toast](round-2/dark/error-toast.png) · [first-sync-empty](round-2/dark/first-sync-empty.png) · [first-sync-loading](round-2/dark/first-sync-loading.png) · [group-chat](round-2/dark/group-chat.png) · [image-preview](round-2/dark/image-preview.png) · [join-on-phone](round-2/dark/join-on-phone.png) · [new-chat](round-2/dark/new-chat.png) · [permission-consent](round-2/dark/permission-consent.png) · [search-switcher](round-2/dark/search-switcher.png) · [settings](round-2/dark/settings.png) · [setup](round-2/dark/setup.png) · [shortcuts](round-2/dark/shortcuts.png) · [signin-browser](round-2/dark/signin-browser.png) · [signin-device-code](round-2/dark/signin-device-code.png) · [signin-expired](round-2/dark/signin-expired.png) · [signin-failed](round-2/dark/signin-failed.png) · [signin-finishing](round-2/dark/signin-finishing.png) · [signin](round-2/dark/signin.png) · [status-message](round-2/dark/status-message.png) · [teams](round-2/dark/teams.png)

</details>

<details><summary>round-3</summary>

**light:** [account-menu](round-3/light/account-menu.png) · [calendar](round-3/light/calendar.png) · [call-bar](round-3/light/call-bar.png) · [call-docked](round-3/light/call-docked.png) · [calls-narrow](round-3/light/calls-narrow.png) · [calls](round-3/light/calls.png) · [channel-narrow](round-3/light/channel-narrow.png) · [channel](round-3/light/channel.png) · [chat-filter-no-match](round-3/light/chat-filter-no-match.png) · [chat-no-selection](round-3/light/chat-no-selection.png) · [chats-narrow](round-3/light/chats-narrow.png) · [chats-wide](round-3/light/chats-wide.png) · [chats](round-3/light/chats.png) · [composer-images](round-3/light/composer-images.png) · [composer-long-draft](round-3/light/composer-long-draft.png) · [composer-mention](round-3/light/composer-mention.png) · [error-toast](round-3/light/error-toast.png) · [first-sync-empty](round-3/light/first-sync-empty.png) · [first-sync-loading](round-3/light/first-sync-loading.png) · [group-chat](round-3/light/group-chat.png) · [image-preview](round-3/light/image-preview.png) · [join-on-phone](round-3/light/join-on-phone.png) · [new-chat](round-3/light/new-chat.png) · [permission-consent](round-3/light/permission-consent.png) · [search-switcher](round-3/light/search-switcher.png) · [settings](round-3/light/settings.png) · [setup](round-3/light/setup.png) · [shortcuts](round-3/light/shortcuts.png) · [signin-browser](round-3/light/signin-browser.png) · [signin-device-code](round-3/light/signin-device-code.png) · [signin-expired](round-3/light/signin-expired.png) · [signin-failed](round-3/light/signin-failed.png) · [signin-finishing](round-3/light/signin-finishing.png) · [signin](round-3/light/signin.png) · [status-message](round-3/light/status-message.png) · [teams](round-3/light/teams.png)

**dark:** [account-menu](round-3/dark/account-menu.png) · [calendar](round-3/dark/calendar.png) · [call-bar](round-3/dark/call-bar.png) · [call-docked](round-3/dark/call-docked.png) · [calls-narrow](round-3/dark/calls-narrow.png) · [calls](round-3/dark/calls.png) · [channel-narrow](round-3/dark/channel-narrow.png) · [channel](round-3/dark/channel.png) · [chat-filter-no-match](round-3/dark/chat-filter-no-match.png) · [chat-no-selection](round-3/dark/chat-no-selection.png) · [chats-narrow](round-3/dark/chats-narrow.png) · [chats-wide](round-3/dark/chats-wide.png) · [chats](round-3/dark/chats.png) · [composer-images](round-3/dark/composer-images.png) · [composer-long-draft](round-3/dark/composer-long-draft.png) · [composer-mention](round-3/dark/composer-mention.png) · [error-toast](round-3/dark/error-toast.png) · [first-sync-empty](round-3/dark/first-sync-empty.png) · [first-sync-loading](round-3/dark/first-sync-loading.png) · [group-chat](round-3/dark/group-chat.png) · [image-preview](round-3/dark/image-preview.png) · [join-on-phone](round-3/dark/join-on-phone.png) · [new-chat](round-3/dark/new-chat.png) · [permission-consent](round-3/dark/permission-consent.png) · [search-switcher](round-3/dark/search-switcher.png) · [settings](round-3/dark/settings.png) · [setup](round-3/dark/setup.png) · [shortcuts](round-3/dark/shortcuts.png) · [signin-browser](round-3/dark/signin-browser.png) · [signin-device-code](round-3/dark/signin-device-code.png) · [signin-expired](round-3/dark/signin-expired.png) · [signin-failed](round-3/dark/signin-failed.png) · [signin-finishing](round-3/dark/signin-finishing.png) · [signin](round-3/dark/signin.png) · [status-message](round-3/dark/status-message.png) · [teams](round-3/dark/teams.png)

</details>

<details><summary>round-4</summary>

**light:** [account-menu](round-4/light/account-menu.png) · [calendar](round-4/light/calendar.png) · [call-bar](round-4/light/call-bar.png) · [call-docked](round-4/light/call-docked.png) · [calls-narrow](round-4/light/calls-narrow.png) · [calls](round-4/light/calls.png) · [channel-narrow](round-4/light/channel-narrow.png) · [channel](round-4/light/channel.png) · [chat-filter-no-match](round-4/light/chat-filter-no-match.png) · [chat-no-selection](round-4/light/chat-no-selection.png) · [chats-narrow](round-4/light/chats-narrow.png) · [chats-wide](round-4/light/chats-wide.png) · [chats](round-4/light/chats.png) · [composer-images](round-4/light/composer-images.png) · [composer-long-draft](round-4/light/composer-long-draft.png) · [composer-mention](round-4/light/composer-mention.png) · [error-toast](round-4/light/error-toast.png) · [first-sync-empty](round-4/light/first-sync-empty.png) · [first-sync-loading](round-4/light/first-sync-loading.png) · [group-chat](round-4/light/group-chat.png) · [image-preview](round-4/light/image-preview.png) · [join-on-phone](round-4/light/join-on-phone.png) · [new-chat](round-4/light/new-chat.png) · [permission-consent](round-4/light/permission-consent.png) · [search-switcher](round-4/light/search-switcher.png) · [settings](round-4/light/settings.png) · [setup](round-4/light/setup.png) · [shortcuts](round-4/light/shortcuts.png) · [signin-browser](round-4/light/signin-browser.png) · [signin-device-code](round-4/light/signin-device-code.png) · [signin-expired](round-4/light/signin-expired.png) · [signin-failed](round-4/light/signin-failed.png) · [signin-finishing](round-4/light/signin-finishing.png) · [signin](round-4/light/signin.png) · [status-message](round-4/light/status-message.png) · [teams](round-4/light/teams.png)

**dark:** [account-menu](round-4/dark/account-menu.png) · [calendar](round-4/dark/calendar.png) · [call-bar](round-4/dark/call-bar.png) · [call-docked](round-4/dark/call-docked.png) · [calls-narrow](round-4/dark/calls-narrow.png) · [calls](round-4/dark/calls.png) · [channel-narrow](round-4/dark/channel-narrow.png) · [channel](round-4/dark/channel.png) · [chat-filter-no-match](round-4/dark/chat-filter-no-match.png) · [chat-no-selection](round-4/dark/chat-no-selection.png) · [chats-narrow](round-4/dark/chats-narrow.png) · [chats-wide](round-4/dark/chats-wide.png) · [chats](round-4/dark/chats.png) · [composer-images](round-4/dark/composer-images.png) · [composer-long-draft](round-4/dark/composer-long-draft.png) · [composer-mention](round-4/dark/composer-mention.png) · [error-toast](round-4/dark/error-toast.png) · [first-sync-empty](round-4/dark/first-sync-empty.png) · [first-sync-loading](round-4/dark/first-sync-loading.png) · [group-chat](round-4/dark/group-chat.png) · [image-preview](round-4/dark/image-preview.png) · [join-on-phone](round-4/dark/join-on-phone.png) · [new-chat](round-4/dark/new-chat.png) · [permission-consent](round-4/dark/permission-consent.png) · [search-switcher](round-4/dark/search-switcher.png) · [settings](round-4/dark/settings.png) · [setup](round-4/dark/setup.png) · [shortcuts](round-4/dark/shortcuts.png) · [signin-browser](round-4/dark/signin-browser.png) · [signin-device-code](round-4/dark/signin-device-code.png) · [signin-expired](round-4/dark/signin-expired.png) · [signin-failed](round-4/dark/signin-failed.png) · [signin-finishing](round-4/dark/signin-finishing.png) · [signin](round-4/dark/signin.png) · [status-message](round-4/dark/status-message.png) · [teams](round-4/dark/teams.png)

</details>

<details><summary>round-5</summary>

**light:** [account-menu](round-5/light/account-menu.png) · [calendar](round-5/light/calendar.png) · [call-bar](round-5/light/call-bar.png) · [call-docked](round-5/light/call-docked.png) · [calls-narrow](round-5/light/calls-narrow.png) · [calls](round-5/light/calls.png) · [channel-narrow](round-5/light/channel-narrow.png) · [channel](round-5/light/channel.png) · [chat-filter-no-match](round-5/light/chat-filter-no-match.png) · [chat-no-selection](round-5/light/chat-no-selection.png) · [chats-narrow](round-5/light/chats-narrow.png) · [chats-wide](round-5/light/chats-wide.png) · [chats](round-5/light/chats.png) · [composer-images](round-5/light/composer-images.png) · [composer-long-draft](round-5/light/composer-long-draft.png) · [composer-mention](round-5/light/composer-mention.png) · [error-toast](round-5/light/error-toast.png) · [first-sync-empty](round-5/light/first-sync-empty.png) · [first-sync-loading](round-5/light/first-sync-loading.png) · [group-chat](round-5/light/group-chat.png) · [image-preview](round-5/light/image-preview.png) · [join-on-phone](round-5/light/join-on-phone.png) · [new-chat](round-5/light/new-chat.png) · [permission-consent](round-5/light/permission-consent.png) · [search-switcher](round-5/light/search-switcher.png) · [settings](round-5/light/settings.png) · [setup](round-5/light/setup.png) · [shortcuts](round-5/light/shortcuts.png) · [signin-browser](round-5/light/signin-browser.png) · [signin-device-code](round-5/light/signin-device-code.png) · [signin-expired](round-5/light/signin-expired.png) · [signin-failed](round-5/light/signin-failed.png) · [signin-finishing](round-5/light/signin-finishing.png) · [signin](round-5/light/signin.png) · [status-message](round-5/light/status-message.png) · [teams](round-5/light/teams.png)

**dark:** [account-menu](round-5/dark/account-menu.png) · [calendar](round-5/dark/calendar.png) · [call-bar](round-5/dark/call-bar.png) · [call-docked](round-5/dark/call-docked.png) · [calls-narrow](round-5/dark/calls-narrow.png) · [calls](round-5/dark/calls.png) · [channel-narrow](round-5/dark/channel-narrow.png) · [channel](round-5/dark/channel.png) · [chat-filter-no-match](round-5/dark/chat-filter-no-match.png) · [chat-no-selection](round-5/dark/chat-no-selection.png) · [chats-narrow](round-5/dark/chats-narrow.png) · [chats-wide](round-5/dark/chats-wide.png) · [chats](round-5/dark/chats.png) · [composer-images](round-5/dark/composer-images.png) · [composer-long-draft](round-5/dark/composer-long-draft.png) · [composer-mention](round-5/dark/composer-mention.png) · [error-toast](round-5/dark/error-toast.png) · [first-sync-empty](round-5/dark/first-sync-empty.png) · [first-sync-loading](round-5/dark/first-sync-loading.png) · [group-chat](round-5/dark/group-chat.png) · [image-preview](round-5/dark/image-preview.png) · [join-on-phone](round-5/dark/join-on-phone.png) · [new-chat](round-5/dark/new-chat.png) · [permission-consent](round-5/dark/permission-consent.png) · [search-switcher](round-5/dark/search-switcher.png) · [settings](round-5/dark/settings.png) · [setup](round-5/dark/setup.png) · [shortcuts](round-5/dark/shortcuts.png) · [signin-browser](round-5/dark/signin-browser.png) · [signin-device-code](round-5/dark/signin-device-code.png) · [signin-expired](round-5/dark/signin-expired.png) · [signin-failed](round-5/dark/signin-failed.png) · [signin-finishing](round-5/dark/signin-finishing.png) · [signin](round-5/dark/signin.png) · [status-message](round-5/dark/status-message.png) · [teams](round-5/dark/teams.png)

</details>

<details><summary>round-6</summary>

**light:** [account-menu](round-6/light/account-menu.png) · [calendar](round-6/light/calendar.png) · [call-bar](round-6/light/call-bar.png) · [call-docked](round-6/light/call-docked.png) · [calls-narrow](round-6/light/calls-narrow.png) · [calls](round-6/light/calls.png) · [channel-narrow](round-6/light/channel-narrow.png) · [channel](round-6/light/channel.png) · [chat-filter-no-match](round-6/light/chat-filter-no-match.png) · [chat-no-selection](round-6/light/chat-no-selection.png) · [chats-narrow](round-6/light/chats-narrow.png) · [chats-wide](round-6/light/chats-wide.png) · [chats](round-6/light/chats.png) · [composer-images](round-6/light/composer-images.png) · [composer-long-draft](round-6/light/composer-long-draft.png) · [composer-mention](round-6/light/composer-mention.png) · [error-toast](round-6/light/error-toast.png) · [first-sync-empty](round-6/light/first-sync-empty.png) · [first-sync-loading](round-6/light/first-sync-loading.png) · [group-chat](round-6/light/group-chat.png) · [image-preview](round-6/light/image-preview.png) · [join-on-phone](round-6/light/join-on-phone.png) · [new-chat](round-6/light/new-chat.png) · [permission-consent](round-6/light/permission-consent.png) · [search-switcher](round-6/light/search-switcher.png) · [settings](round-6/light/settings.png) · [setup](round-6/light/setup.png) · [shortcuts](round-6/light/shortcuts.png) · [signin-browser](round-6/light/signin-browser.png) · [signin-device-code](round-6/light/signin-device-code.png) · [signin-expired](round-6/light/signin-expired.png) · [signin-failed](round-6/light/signin-failed.png) · [signin-finishing](round-6/light/signin-finishing.png) · [signin](round-6/light/signin.png) · [status-message](round-6/light/status-message.png) · [teams](round-6/light/teams.png)

**dark:** [account-menu](round-6/dark/account-menu.png) · [calendar](round-6/dark/calendar.png) · [call-bar](round-6/dark/call-bar.png) · [call-docked](round-6/dark/call-docked.png) · [calls-narrow](round-6/dark/calls-narrow.png) · [calls](round-6/dark/calls.png) · [channel-narrow](round-6/dark/channel-narrow.png) · [channel](round-6/dark/channel.png) · [chat-filter-no-match](round-6/dark/chat-filter-no-match.png) · [chat-no-selection](round-6/dark/chat-no-selection.png) · [chats-narrow](round-6/dark/chats-narrow.png) · [chats-wide](round-6/dark/chats-wide.png) · [chats](round-6/dark/chats.png) · [composer-images](round-6/dark/composer-images.png) · [composer-long-draft](round-6/dark/composer-long-draft.png) · [composer-mention](round-6/dark/composer-mention.png) · [error-toast](round-6/dark/error-toast.png) · [first-sync-empty](round-6/dark/first-sync-empty.png) · [first-sync-loading](round-6/dark/first-sync-loading.png) · [group-chat](round-6/dark/group-chat.png) · [image-preview](round-6/dark/image-preview.png) · [join-on-phone](round-6/dark/join-on-phone.png) · [new-chat](round-6/dark/new-chat.png) · [permission-consent](round-6/dark/permission-consent.png) · [search-switcher](round-6/dark/search-switcher.png) · [settings](round-6/dark/settings.png) · [setup](round-6/dark/setup.png) · [shortcuts](round-6/dark/shortcuts.png) · [signin-browser](round-6/dark/signin-browser.png) · [signin-device-code](round-6/dark/signin-device-code.png) · [signin-expired](round-6/dark/signin-expired.png) · [signin-failed](round-6/dark/signin-failed.png) · [signin-finishing](round-6/dark/signin-finishing.png) · [signin](round-6/dark/signin.png) · [status-message](round-6/dark/status-message.png) · [teams](round-6/dark/teams.png)

</details>

<details><summary>round-7</summary>

**light:** [account-menu](round-7/light/account-menu.png) · [calendar](round-7/light/calendar.png) · [call-bar](round-7/light/call-bar.png) · [call-docked](round-7/light/call-docked.png) · [calls-narrow](round-7/light/calls-narrow.png) · [calls](round-7/light/calls.png) · [channel-narrow](round-7/light/channel-narrow.png) · [channel](round-7/light/channel.png) · [chat-filter-no-match](round-7/light/chat-filter-no-match.png) · [chat-no-selection](round-7/light/chat-no-selection.png) · [chats-narrow](round-7/light/chats-narrow.png) · [chats-wide](round-7/light/chats-wide.png) · [chats](round-7/light/chats.png) · [composer-images](round-7/light/composer-images.png) · [composer-long-draft](round-7/light/composer-long-draft.png) · [composer-mention](round-7/light/composer-mention.png) · [error-toast](round-7/light/error-toast.png) · [first-sync-empty](round-7/light/first-sync-empty.png) · [first-sync-loading](round-7/light/first-sync-loading.png) · [group-chat](round-7/light/group-chat.png) · [image-preview](round-7/light/image-preview.png) · [join-on-phone](round-7/light/join-on-phone.png) · [new-chat](round-7/light/new-chat.png) · [permission-consent](round-7/light/permission-consent.png) · [search-switcher](round-7/light/search-switcher.png) · [settings](round-7/light/settings.png) · [setup](round-7/light/setup.png) · [shortcuts](round-7/light/shortcuts.png) · [signin-browser](round-7/light/signin-browser.png) · [signin-device-code](round-7/light/signin-device-code.png) · [signin-expired](round-7/light/signin-expired.png) · [signin-failed](round-7/light/signin-failed.png) · [signin-finishing](round-7/light/signin-finishing.png) · [signin](round-7/light/signin.png) · [status-message](round-7/light/status-message.png) · [teams](round-7/light/teams.png)

**dark:** [account-menu](round-7/dark/account-menu.png) · [calendar](round-7/dark/calendar.png) · [call-bar](round-7/dark/call-bar.png) · [call-docked](round-7/dark/call-docked.png) · [calls-narrow](round-7/dark/calls-narrow.png) · [calls](round-7/dark/calls.png) · [channel-narrow](round-7/dark/channel-narrow.png) · [channel](round-7/dark/channel.png) · [chat-filter-no-match](round-7/dark/chat-filter-no-match.png) · [chat-no-selection](round-7/dark/chat-no-selection.png) · [chats-narrow](round-7/dark/chats-narrow.png) · [chats-wide](round-7/dark/chats-wide.png) · [chats](round-7/dark/chats.png) · [composer-images](round-7/dark/composer-images.png) · [composer-long-draft](round-7/dark/composer-long-draft.png) · [composer-mention](round-7/dark/composer-mention.png) · [error-toast](round-7/dark/error-toast.png) · [first-sync-empty](round-7/dark/first-sync-empty.png) · [first-sync-loading](round-7/dark/first-sync-loading.png) · [group-chat](round-7/dark/group-chat.png) · [image-preview](round-7/dark/image-preview.png) · [join-on-phone](round-7/dark/join-on-phone.png) · [new-chat](round-7/dark/new-chat.png) · [permission-consent](round-7/dark/permission-consent.png) · [search-switcher](round-7/dark/search-switcher.png) · [settings](round-7/dark/settings.png) · [setup](round-7/dark/setup.png) · [shortcuts](round-7/dark/shortcuts.png) · [signin-browser](round-7/dark/signin-browser.png) · [signin-device-code](round-7/dark/signin-device-code.png) · [signin-expired](round-7/dark/signin-expired.png) · [signin-failed](round-7/dark/signin-failed.png) · [signin-finishing](round-7/dark/signin-finishing.png) · [signin](round-7/dark/signin.png) · [status-message](round-7/dark/status-message.png) · [teams](round-7/dark/teams.png)

</details>

<details><summary>round-8</summary>

**light:** [account-menu](round-8/light/account-menu.png) · [calendar](round-8/light/calendar.png) · [call-bar](round-8/light/call-bar.png) · [call-docked](round-8/light/call-docked.png) · [calls-narrow](round-8/light/calls-narrow.png) · [calls](round-8/light/calls.png) · [channel-narrow](round-8/light/channel-narrow.png) · [channel](round-8/light/channel.png) · [chat-filter-no-match](round-8/light/chat-filter-no-match.png) · [chat-no-selection](round-8/light/chat-no-selection.png) · [chats-narrow](round-8/light/chats-narrow.png) · [chats-wide](round-8/light/chats-wide.png) · [chats](round-8/light/chats.png) · [composer-images](round-8/light/composer-images.png) · [composer-long-draft](round-8/light/composer-long-draft.png) · [composer-mention](round-8/light/composer-mention.png) · [error-toast](round-8/light/error-toast.png) · [first-sync-empty](round-8/light/first-sync-empty.png) · [first-sync-loading](round-8/light/first-sync-loading.png) · [group-chat](round-8/light/group-chat.png) · [image-preview](round-8/light/image-preview.png) · [join-on-phone](round-8/light/join-on-phone.png) · [new-chat](round-8/light/new-chat.png) · [permission-consent](round-8/light/permission-consent.png) · [search-switcher](round-8/light/search-switcher.png) · [settings](round-8/light/settings.png) · [setup](round-8/light/setup.png) · [shortcuts](round-8/light/shortcuts.png) · [signin-browser](round-8/light/signin-browser.png) · [signin-device-code](round-8/light/signin-device-code.png) · [signin-expired](round-8/light/signin-expired.png) · [signin-failed](round-8/light/signin-failed.png) · [signin-finishing](round-8/light/signin-finishing.png) · [signin](round-8/light/signin.png) · [status-message](round-8/light/status-message.png) · [teams](round-8/light/teams.png)

**dark:** [account-menu](round-8/dark/account-menu.png) · [calendar](round-8/dark/calendar.png) · [call-bar](round-8/dark/call-bar.png) · [call-docked](round-8/dark/call-docked.png) · [calls-narrow](round-8/dark/calls-narrow.png) · [calls](round-8/dark/calls.png) · [channel-narrow](round-8/dark/channel-narrow.png) · [channel](round-8/dark/channel.png) · [chat-filter-no-match](round-8/dark/chat-filter-no-match.png) · [chat-no-selection](round-8/dark/chat-no-selection.png) · [chats-narrow](round-8/dark/chats-narrow.png) · [chats-wide](round-8/dark/chats-wide.png) · [chats](round-8/dark/chats.png) · [composer-images](round-8/dark/composer-images.png) · [composer-long-draft](round-8/dark/composer-long-draft.png) · [composer-mention](round-8/dark/composer-mention.png) · [error-toast](round-8/dark/error-toast.png) · [first-sync-empty](round-8/dark/first-sync-empty.png) · [first-sync-loading](round-8/dark/first-sync-loading.png) · [group-chat](round-8/dark/group-chat.png) · [image-preview](round-8/dark/image-preview.png) · [join-on-phone](round-8/dark/join-on-phone.png) · [new-chat](round-8/dark/new-chat.png) · [permission-consent](round-8/dark/permission-consent.png) · [search-switcher](round-8/dark/search-switcher.png) · [settings](round-8/dark/settings.png) · [setup](round-8/dark/setup.png) · [shortcuts](round-8/dark/shortcuts.png) · [signin-browser](round-8/dark/signin-browser.png) · [signin-device-code](round-8/dark/signin-device-code.png) · [signin-expired](round-8/dark/signin-expired.png) · [signin-failed](round-8/dark/signin-failed.png) · [signin-finishing](round-8/dark/signin-finishing.png) · [signin](round-8/dark/signin.png) · [status-message](round-8/dark/status-message.png) · [teams](round-8/dark/teams.png)

</details>

## Outcome

Both graders moved from 6.0 and 6.5 at the baseline to 8.0 each. Scores held at 8.0 for both from round 6 to round 8, which was the last round allowed, so the loop stopped there short of the 9 target. The remaining findings are listed below. Most would need features beyond a UI pass: a collapsible list for narrow windows, richer meeting details, navigation while docked. The rest are trade-offs kept on purpose (cropped thumbnails, all quick reactions in the toolbar) or properties of the capture font.

| Check | Baseline (main, a4ebc91) | Final (this branch) |
| --- | --- | --- |
| `cargo build --locked --release` | pass | pass |
| `cargo test --locked --all-targets` | pass, 181 tests | pass, 186 tests |
| `cargo clippy --locked --all-targets -- -D warnings` | pass | pass |
| `cargo fmt --all --check` | pass | pass |

## Capture notes

- **Font.** Screenshots render with Wren's bundled Inter, so they're reproducible. Normal runs use the platform's interface font. Inter's hyphen glyph is wide by design (it renders the same in Firefox), which is why "On-call" looks spaced in the shots. Both graders flagged it in most rounds; it isn't a layout bug, and the app was not changed for it.
- **Focus.** Under Xvfb with no window manager the window never receives focus, and egui only draws focus rings and the caret in a focused window. Up to round 6 the shots showed text fields as unfocused. From round 7, screenshot mode treats the window as focused (`raw_input_hook` in `main.rs`).
- **Timing.** Software GL takes a few seconds to the first frame, and dialogs fade in after that. The first round-0 capture caught dialogs mid fade-in, so it was redone with a 9 s delay before grading.
- **Pointer.** The pointer rests at the centre of the virtual screen, so some shots show hover states, such as the message toolbar or the code sign-in tooltip. Once round 5 centred the sign-in page vertically, that tooltip appears in the sign-in shots.
- **New `--page` states** for the review: sign-in steps and failures, Teams with no channel, no chat open, no filter matches, first-sync loading and empty, a failed send, and the consent dialog.

## Decisions and open items

- **Narrow windows (760 pt).** Below 960 pt the sidebar narrows to 232 pt, but names still truncate. Both graders asked for a collapsible list or a list/detail view. That's a new navigation feature, so it was left out of this UI pass.
- **Pasted-image thumbnails** stay centre-cropped to 144 × 96, the behaviour the existing tests pin down ("crop instead of shrinking a wide screenshot into a line"). They now sit on tiles with a border, the remove button on the corner and the image's dimensions underneath. Both graders still prefer letterboxing extreme aspect ratios.
- **Calendar detail** shows only what Wren has (organizer, location, online, start). Attendees and agenda would need more Graph data and are a feature, not a UI fix.
- **Hover toolbar** keeps all six quick reactions. Graders suggested fewer, but dropping them would remove a quick action.
- **Call bar** has no elapsed-time display. Wren follows presence, not call media, so a timer would only estimate.

## Grades

The graders' full reports, with every issue and the measured contrast ratios, are in [`grades/`](grades/).
