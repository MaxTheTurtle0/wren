# How Wren stays up to date

## The rule

Microsoft's Teams API overview says:

> If your app polls to see whether a resource has changed, you can only do
> that once per day. […] It's okay to GET /me/joinedTeams every time the user
> visits/refreshes your web page, but it isn't okay to GET /me/joinedTeams in
> a loop every 30 seconds to refresh that web page. Apps that don't follow
> these polling requirements will be considered in violation of the Microsoft
> APIs Terms of Use.

([source](https://learn.microsoft.com/en-us/graph/api/resources/teams-api-overview?view=graph-rest-1.0#polling-requirements))

The ways Graph pushes changes all need infrastructure a desktop app does not
have:

- a public HTTPS webhook;
- Azure Event Hubs;
- Azure Event Grid;
- possibly Web Push, though its support for Teams messages is unverified.

## What Wren does (implemented)

`src/sync.rs` decides every fetch. It has no timer that fetches on its own.

| Trigger | Fetches | Skipped if fetched within |
|---|---|---|
| Wren starts / you sign in | Profile, chats, teams and channels, calendar | (always) |
| You open a chat or channel | Its messages (date-ranged when cached) | 10 s |
| You switch to Chat, Teams or Calendar | That list | 15 s / 30 min / 5 min |
| The window comes back to the foreground | The current list and the open conversation | 15 s / 10 s |
| You press ⌘R (Refresh) | The current list and the open conversation | 2 s |
| You send, edit, delete or react | The conversation, date-ranged | 1 s |
| Presence | People in the chat you open | 60 s per person |

Message refreshes ask only for what changed since the newest cached change
(`$filter=lastModifiedDateTime gt …`), as the polling rules request.
Everything is cached in SQLite, so every view first shows the cache
immediately and updates when Graph answers.

What this means in practice:

- While you are using Wren, a chat you open or return to is current.
- While Wren sits untouched in the background, it learns nothing new and
  shows no notifications for new messages. Meeting reminders still work,
  because they come from the cached calendar.

## Real push (planned)

Instant updates and background notifications need Graph change
notifications. The design Wren intends to support:

1. **A small relay** that anyone can self-host (a single binary or a
   container). It receives Graph webhooks for
   `/users/{id}/chats/getAllMessages` (`Chat.Read`) and channel subscriptions,
   answers the validation handshake, and forwards each notification to the
   person's connected Wren over a WebSocket.
2. **End-to-end encryption through Graph's own mechanism.** Wren creates the
   subscription with `includeResourceData` and its own RSA certificate, so
   the relay only ever handles ciphertext it cannot read.
3. **Renewal.** Chat message subscriptions last at most three days. Wren
   renews them while it runs.
4. **For organisations on Azure:** Event Hubs delivery instead of a relay.

Until a relay is configured, Wren keeps working with the fetch-on-intent
rules above.
