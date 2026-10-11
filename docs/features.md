# Feature support

Status as of Wren 0.1 (unreleased).

- **Works**: implemented against Microsoft Graph, unit-tested against a mock
  Graph server and drawn in the headless UI tests. It has **not yet been
  exercised against a live tenant** (see "Verification" below).
- **Demo only**: drawn and interactive with sample data, with no Graph
  integration yet.
- **Hand-off**: opens Microsoft Teams through documented deep links.
- **Experimental / planned**: designed, not built.
- **Not possible**: no official API allows it.

## Messaging

| Feature | Status | Notes |
|---|---|---|
| Chat list (1:1, group, meeting chats), unread state | Works | `GET /me/chats` with members and newest message |
| Message history with paging | Works | 50 per page; older pages by timestamp cursor; cached |
| Send messages (rich text from light markdown) | Works | Optimistic, with retry on failure |
| Quote replies in chats | Works | `replyWithQuote` |
| Reactions (any emoji) | Works | `setReaction` / `unsetReaction`; legacy names mapped to emoji |
| Edit and delete your messages | Works | Chats: Chat.ReadWrite. Channels: needs ChannelMessage.ReadWrite (admin), hidden otherwise |
| Mark read | Works | `markChatReadForUser` when a chat is opened |
| Rich text: bold, italic, underline, strike, code, quotes, lists, links, mentions, emoji | Works | Native rendering of Teams HTML |
| Inline images (hosted content) | Works | Lazy, Graph-hosted only; other hosts are never fetched (privacy) |
| File attachments: view | Works | Shown as file chips that open in the browser |
| File attachments: send | Works | Attach button or drag and drop, up to 1 GiB per file. Uploaded to your OneDrive folder `Microsoft Teams Chat Files` (resumable upload above 4 MiB), shared read-only with the chat's members (an organisation-wide view link in channels), and sent as a `reference` attachment. Files.ReadWrite, requested on first use. Pasting images: not yet |
| @mentions in the composer | Works | Type `@`: suggestions from the chat's members, or a directory search in channels. Sent as `<at>` tags with a `mentions` array, so people are notified as in Teams |
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
| Where calls open | Works | Default: a chrome-less Edge or Chrome window (`--app`); without either, the default browser. The Teams app only if chosen in Settings. The browser is started with an argument list, never a shell, and only with Teams links ([call-window.md](call-window.md)) |
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
| Tray / menu-bar item, keep running | Works | Off by default (no push yet) |
| Desktop notifications | Works | notify-rust |
| Single running instance | Works | fastframe-instance |
| Demo mode | Works | `wren --demo`, or "Explore the demo" on the welcome screen |
| Performance overlay | Works | ⌘⇧P |
| Personal Microsoft accounts / Teams (free) | Not possible | Graph's Teams APIs support work and school accounts only |

## Verification

Tested and run:

- Unit tests for the HTML parser, the PKCE values (RFC 7636 test vector),
  the loopback redirect, and token exchange, refresh and device code against
  a mock token endpoint.
- Graph client behaviour against a mock Graph server: paging, 401 refresh,
  `Retry-After`, refusing to send tokens to foreign hosts, every write
  endpoint.
- The cache: ordering, FTS5, diacritics, permissions.
- **Files, new chats, mentions and status end to end**
  (`tests/features_flow.rs`), through the real worker against a mock Graph:
  the people search (`ConsistencyLevel: eventual`, `$search`, `$count`), the
  `POST /chats` body, the upload with its read-back and the sharing invite,
  the exact message body with an `<at>` mention and an `<attachment>`,
  `setUserPreferredPresence`, `setStatusMessage`, a single `setPresence`
  with the client ID as session ID, and `clearPresence` on exit.
- Large uploads: the upload-session request and 5 MiB chunks without the
  Graph token (unit tests in `graph/files.rs`).
- Calls: finding Edge and Chrome on each platform, the exact argument lists,
  refusing anything but Teams links, docking geometry, the call session's
  states against presence sequences (flicker, calls elsewhere, calendar-only
  meetings), join-link matching, the meeting-chat lookup end to end
  (`tests/features_flow.rs`), and a presence notification through the real
  relay causing a presence fetch (`tests/push_flow.rs`). The docked panel,
  the call bar and the QR code are drawn in the UI tests and screenshotted
  on macOS.
- **The whole backend end to end** (`tests/backend_flow.rs`), against local
  stand-ins for Entra ID and Graph:
  - the browser sign-in through Wren's real loopback redirect, with PKCE;
  - the token exchange, chat listing, message history and sending;
  - a restart that restores the session from the credential store and shows
    the cache before any request;
  - refresh-token rotation;
  - sign-out deleting the token and the cache.
- **Push updates end to end** (`tests/push_flow.rs`):
  - the real `wren-relay` and backend, with Graph simulated;
  - the subscription request (resource, no resource data);
  - the validation handshake through the relay;
  - a change notification leading to a fetch of that chat;
  - deleting the subscription when push is turned off.
- Wren's requests against Microsoft's real token endpoint, with an
  unregistered client ID. Entra parsed them and answered with the expected
  AADSTS codes, which Wren now explains (single-tenant registrations need the
  tenant set).
- Model translation from Graph-shaped JSON.
- Headless UI frames for every screen, including a 5,000-message
  conversation, the new-chat, status-message and consent dialogs, switching
  accounts, and a draft with a mention and an attached file.
- The macOS app run and screenshotted on macOS 27 (Apple Silicon), including
  the account menu, the new-chat and status dialogs and the mention
  suggestions.

Not yet run:

- A live sign-in and a live tenant. These need an Entra app registration and,
  in most tenants, admin consent ([azure-app-registration.md](azure-app-registration.md)).
- Windows and Linux windows. Both compile and pass tests in CI, but nobody
  has looked at a window there yet.
- A real call in the Teams web window: the open questions are listed in
  [call-window.md §7](call-window.md#7-open-questions).
