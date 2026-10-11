# Feature support

Status as of Wren 0.1.

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
| File attachments: send | Planned | Needs OneDrive upload (Files.ReadWrite) and a sharing link |
| Adaptive Cards | Not yet | Shown as a placeholder with a hint to open in Teams |
| Local message search | Works | SQLite FTS5 over everything cached, instant |
| Microsoft Search (server-side) | Works | Merged into ⌘K results for queries of 3+ characters |
| Start a new chat | Planned | Needs Chat.Create |
| Real-time updates | Partial | Fetch on open, focus and refresh. Background polling is forbidden by Microsoft. Push needs a relay (planned), see [realtime.md](realtime.md) |
| New-message notifications | Partial | Shown when a fetch finds new messages in other chats. Not while Wren is idle in the background |

## Teams and channels

| Feature | Status | Notes |
|---|---|---|
| Joined teams and their channels (incl. private and shared) | Works | Team.ReadBasic.All, Channel.ReadBasic.All |
| Channel posts with threaded replies | Works, needs admin consent | ChannelMessage.Read.All. Without it, Wren explains and offers "Request access" and "Open in Teams" |
| New posts and thread replies | Works | ChannelMessage.Send |

## Calls and meetings

| Feature | Status | Notes |
|---|---|---|
| Upcoming meetings (14 days) and Join | Works / Hand-off | Calendars.Read; Join opens Teams |
| Meeting reminders | Works | 5 minutes before, from the cached calendar (no network) |
| Start audio or video call | Hand-off | Teams deep link; Teams asks to confirm |
| Join by pasted link | Hand-off | Teams hosts only |
| Incoming calls, in-app audio, video, screen sharing, participants | Not possible yet | Graph calling is application-only (bots); needs Azure Communication Services. See [calling.md](calling.md) |

## People and app

| Feature | Status | Notes |
|---|---|---|
| Presence of people in chats | Works | Presence.Read.All, fetched when a chat opens, at most once a minute per person |
| Setting your own presence or status message | Planned | Presence.ReadWrite plus a presence session |
| Profile photos | Works | Lazy, cached 7 days |
| Multiple accounts | Planned | The cache, credentials and folders are already per account; the switcher is not built |
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
- Model translation from Graph-shaped JSON.
- Headless UI frames for every screen, including a 5,000-message
  conversation.
- The macOS app run and screenshotted on macOS 27 (Apple Silicon).

Not yet run:

- A live sign-in and a live tenant. These need an Entra app registration and,
  in most tenants, admin consent ([azure-app-registration.md](azure-app-registration.md)).
- Windows and Linux windows. Both compile and pass tests in CI, but nobody
  has looked at a window there yet.
