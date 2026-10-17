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

Graph pushes changes only to infrastructure a desktop app does not have:

- a public HTTPS webhook;
- Azure Event Hubs;
- Azure Event Grid.

Wren therefore has two modes.

## Without a relay: fetch on intent (default)

`src/sync.rs` decides every fetch. There is no timer that fetches on its own.

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
(`$filter=lastModifiedDateTime gt …`), as the polling rules request. Every
view shows the cache first and updates when Graph answers.

What this means in practice:

- A chat you open or return to is current.
- While Wren sits untouched, it learns nothing new and shows no new-message
  notifications. Meeting reminders still work, because they come from the
  cached calendar.

## With a relay: push updates (experimental)

**Status:** implemented and tested end to end against a local relay and a
simulated Graph (`tests/push_flow.rs`). The maintainer confirmed real push
notifications during live-account acceptance on 17 October 2026. See the
[verification record](features.md#verification). Push remains experimental.

How it works:

1. You run `wren-relay` (in this repository, `relay/`) on a public HTTPS
   hostname and enter its URL in **Settings → Push updates**.
2. Wren opens a WebSocket to `{relay}/ws/{channel}`, with a random channel
   per run.
3. Wren subscribes to Graph change notifications for all of your chat
   messages: `POST /subscriptions` on `/users/{id}/chats/getAllMessages`,
   delegated `Chat.Read` (covered by `Chat.ReadWrite`), delivered to
   `{relay}/graph/{channel}`, **without resource data**. A second
   subscription, on your own presence (`communications/presences/{id}`,
   `Presence.Read.All`), lets the call bar follow calls; if Graph refuses
   it, chat updates still work.
4. Graph validates the URL (the relay echoes the `validationToken`), then
   posts each change. The relay forwards it over the WebSocket.
5. Wren checks the `clientState` secret and fetches the changed chat with the
   usual date-ranged request, plus the chat list for unread state and
   notifications. A burst of changes within a second is one fetch.
6. Wren renews the subscription every 40 minutes, reauthorizes it when Graph
   asks, recreates it when Graph removes it or the relay drops, and deletes
   it when push is turned off or Wren exits.

What the relay sees: notification envelopes with a subscription id, the
`clientState`, and resource paths such as `chats('19:…')/messages('1702')`
or `communications/presences/{id}`.
**No message content and no tokens.** It keeps nothing on disk. Anyone who
could forge a notification could only make Wren re-fetch from Graph.

With push on, keeping Wren running in the tray (**Settings → Behaviour**)
gives background notifications for new messages.

### Running the relay

```sh
cargo build --release -p wren-relay
./target/release/wren-relay --listen 127.0.0.1:8787
```

Put it behind HTTPS on a public hostname, because Graph only posts to
`https://` URLs. For example, with [Caddy](https://caddyserver.com):

```
relay.example.com {
    reverse_proxy 127.0.0.1:8787
}
```

A Cloudflare tunnel (`cloudflared tunnel --url http://127.0.0.1:8787`) or
any cloud load balancer works too. One relay serves any number of people.
Routes:

| Route | Who calls it |
|---|---|
| `GET /` | Health check |
| `GET /ws/{channel}` (WebSocket) | Wren |
| `POST /graph/{channel}` | Microsoft Graph: change notifications and validation |
| `POST /lifecycle/{channel}` | Microsoft Graph: lifecycle notifications |

### Limits

- Graph allows 10,000 Teams subscriptions per organisation. Each running Wren
  holds one.
- Channel posts are not subscribed yet. They would need `ChannelMessage.Read.All`
  and one subscription per channel.
- Organisations on Azure could deliver through Event Hubs instead. Not
  implemented.
