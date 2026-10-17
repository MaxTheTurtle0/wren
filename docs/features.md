# Feature support

Status as of Wren 0.1 (unreleased).

- **Works**: implemented against Microsoft Graph, unit-tested against a mock
  Graph server and drawn in the headless UI tests. Automated coverage and
  maintainer-confirmed live-account acceptance are listed below.
- **Demo only**: drawn and interactive with sample data, with no Graph
  integration yet.
- **Hand-off**: opens Microsoft Teams through documented deep links.
- **Experimental / planned**: designed, not built.
- **Not possible**: no official API allows it.

## Messaging

| Feature | Status | Notes |
|---|---|---|
| Chat list (1:1, group, meeting chats), unread state | Works | `GET /me/chats` with members and newest message |
| Message history with paging | Works | 50 per page; follows Graph paging links, with a timestamp cursor after reopening; cached older pages load first and preserve the visible row |
| Send messages (rich text from light markdown) | Works | Optimistic, with retry on failure. Enter sends; Shift+Enter adds a line, including while editing or showing mention suggestions. Long drafts scroll inside a bounded editor, keeping history and composer controls visible |
| Quote replies in chats | Works | `replyWithQuote` |
| Reactions (any emoji) | Works | `setReaction` / `unsetReaction`; legacy names mapped to emoji |
| Edit and delete your messages | Works | Chats: Chat.ReadWrite. Channels: needs ChannelMessage.ReadWrite (admin), hidden otherwise |
| Mark read | Works | `markChatReadForUser` when a chat is opened |
| Rich text: bold, italic, underline, strike, code, quotes, lists, links, mentions, emoji | Works | Native rendering of Teams HTML |
| Inline images (hosted content) | Works | Lazy, Graph-hosted only; other hosts are never fetched (privacy). Downloads have a bounded local cache. Thumbnails use reduced-resolution textures; originals are released after closing the viewer. Click an image to open the same fit/100% viewer as draft images, using the original decoded dimensions |
| File attachments: view | Works | Shown as file chips that open in the browser |
| File attachments: send | Works | Attach button or drag and drop, up to 1 GiB per file. Uploaded to your OneDrive folder `Microsoft Teams Chat Files` (resumable upload above 4 MiB), shared read-only with the chat's members (an organisation-wide view link in channels), and sent as a `reference` attachment. Files.ReadWrite, requested on first use |
| Paste images into the composer | Works | Fixed-size thumbnails crop without stretching. Click to preview the complete image, fit it to the window, or view at 100% with scrolling. Inline PNG hosted-content sends, just under 3 MiB combined per message; no Files.ReadWrite needed |
| @mentions in the composer | Works | Type `@`: suggestions from the chat's members, or a directory search in channels. Tab or Enter picks a suggestion and keeps focus in the composer; Space then inserts a space. Sent as `<at>` tags with a `mentions` array, so people are notified as in Teams |
| Adaptive Cards | Not yet | Shown as a placeholder with a hint to open in Teams |
| Local message search | Works | SQLite FTS5 over everything cached, instant |
| Microsoft Search (server-side) | Works | Merged into ⌘K results for queries of 3+ characters |
| Start a new chat | Works | ⌘N or the compose button: search your organisation's directory (User.ReadBasic.All), pick one or more people, optionally name a group. `POST /chats` with Chat.Create, requested on first use. An existing 1:1 chat opens instead of a new one |
| Real-time updates | Works by default on intent; push is experimental | Fetch on open, focus and refresh (Microsoft forbids background polling). With a self-hosted `wren-relay`, Graph change notifications push new messages. See [realtime.md](realtime.md) |
| New-message notifications | Partial | Whenever a fetch finds new messages in other chats. With the relay, also while Wren runs in the background |

## Teams and channels

| Feature | Status | Notes |
|---|---|---|
| Joined teams and their channels (incl. private and shared) | Works | Team.ReadBasic.All, Channel.ReadBasic.All |
| Channel posts with threaded replies | Works, needs admin consent | ChannelMessage.Read.All. Without it, Wren explains and offers "Request access" and "Open in Teams" |
| New posts and thread replies | Works | ChannelMessage.Send |

## Calls and meetings

| Feature | Status | Notes |
|---|---|---|
| Upcoming meetings (14 days) and Join | Works | Calendars.Read. Join opens the meeting in Teams on the web, beside Wren (below) |
| Meeting reminders | Works | 5 minutes before, from the cached calendar (no network) |
| Start audio or video call | Works | Microsoft's call deep link in Teams on the web; Teams asks to confirm |
| Join by pasted link | Works | Teams hosts only |
| Where calls open | Works | Default: a chrome-less Edge or Chrome window (`--app`); without either, the default browser. The Teams app only if chosen in Settings. The browser is started with an argument list, never a shell, and only with Teams links ([calling.md](calling.md)) |
| Docking beside the call | Works | Wren becomes a 380-point panel on the right of its frame and asks the browser to open the call on the left; restored when the call ends. Not on Wayland (no window positions) or for windows under 960 points |
| Call bar and call status | Works | Follows your own presence: on focus, and with the relay through a presence subscription (Presence.Read.All). Ends a minute after presence leaves the call. Shows calls started elsewhere too |
| Meeting chat beside the call | Works | From the cache by join link; otherwise `GET /me/onlineMeetings?$filter=JoinWebUrl eq '…'` with OnlineMeetings.Read, requested on first use |
| Signed in like Wren | Works | Wren's sign-in opens in the call browser, so Teams on the web finds the same Microsoft session; "Sign in to Teams on the web" in Settings for "Stay signed in"; optional browser profile per account and browser sign-out |
| Join on a phone | Works | QR code of the join link |
| Incoming calls, audio and video inside Wren's window, call controls | Not possible at no cost | Graph calling is application-only (bots). Microsoft's licence covers call media only in its own clients; Azure Communication Services bills per participant-minute. See [calling.md](calling.md) |

## People and app

| Feature | Status | Notes |
|---|---|---|
| Presence of people in chats | Works | Presence.Read.All, fetched when a chat opens, at most once a minute per person |
| Setting your own availability and status message | Works | From the account menu (your picture in the rail): Available, Busy, Do not disturb, Be right back, Appear away, Appear offline, or Reset. `setUserPreferredPresence` with Graph's default expiry, `setStatusMessage`. Presence.ReadWrite, requested on first use. Clearing a message sends empty content, which Microsoft does not document (unverified) |
| Showing you as available while you use Wren | Works | A Graph presence session (`setPresence`), refreshed at most every 4 minutes and only on input, so stepping away shows you inactive after 5 minutes and Away after 10. Ended at sign-out and exit, expires after an hour otherwise. Starts once Presence.ReadWrite is granted; can be turned off in Settings |
| Profile photos | Works | Lazy, cached 7 days |
| Multiple accounts | Works | Add from the account menu or Settings; ⌘⇧A switches. Each account has its own backend, cache, credentials and push connection. Unread chats in other accounts show as a dot on your picture; their notifications still arrive. Signing in to an account twice keeps one |
| Light, dark, system theme, 8 shared palettes | Works | fastframe-theme presets |
| Keyboard shortcuts, ⌘K switcher | Works | ⌘/ lists them |
| Tray / menu-bar item, keep running | Works | Off by default; with a self-hosted relay, push notifications continue while the window is closed |
| Desktop notifications | Works | notify-rust |
| Single running instance | Works | fastframe-instance |
| Demo mode | Works | `wren --demo`, or "Explore the demo" on the welcome screen |
| Performance overlay | Works | ⌘⇧P |
| Personal Microsoft accounts / Teams (free) | Not possible | Graph's Teams APIs support work and school accounts only |

## App updates

Official portable downloads and macOS bundles check GitHub daily and download signed updates in the background. Restart is explicit, with rollback if the new app fails to start. Package-manager and unmarked source builds use their existing update method. The first public release still needs to be published. See [updates.md](updates.md).

## Verification

| Coverage | Verification |
|---|---|
| Authentication, Graph, cache and calling logic | Unit tests and local mock servers: PKCE, token rotation, paging, retries, token isolation, writes, uploads, search, browser arguments, docking and call state |
| Backend and feature flows | `tests/backend_flow.rs` and `tests/features_flow.rs`: sign-in, session restoration, sign-out, messaging, files, new chats, mentions, presence and meeting-chat lookup |
| Push | `tests/push_flow.rs`: real relay and backend with simulated Graph, subscription validation, notifications and cleanup |
| Interface | `tests/ui_smoke.rs`: headless frames for every screen, dialogs, account switching and a 5,000-message chat |
| Native macOS | Apple Silicon, macOS 27; demo screenshots at 1280 × 800, call bar, docked panel and QR code; signed-in performance checks in [performance.md](performance.md) |
| Persistent macOS signing | Synthetic-certificate regression and maintainer-confirmed reopening without another Keychain prompt; setup in [macos-keychain.md](macos-keychain.md) |

**Live-account acceptance**, confirmed by the maintainer on 17 October 2026:
a fresh sign-in and consent flow, chat and channel reads and writes, file
sharing, presence, calendar, account switching, sign-out cleanup, and real
push notifications. This was a manual check with a real account.

Remaining checks and known issues:

- Native Windows and Linux windows have not been inspected. Both compile
  and pass tests in CI.
- A real call in the Teams web window has not been confirmed. See the
  [calling open questions](calling.md#open-questions).
- The Calls capability table clips at 760 × 700, observed in both themes
  during the macOS demo checks.
