# Permissions

Wren uses **delegated** Microsoft Graph permissions only. It acts as the
signed-in person, and can never see more than that person can in Teams.
It asks for the smallest set that covers its features. Permissions that only
one feature needs are requested the first time someone uses that feature
(incremental consent): Wren explains why, then opens Microsoft's consent page.
`ChannelMessage.Read.All`, the one that needs admin consent, is requested when
someone opens a channel.

| Permission | Requested | Endpoints | Admin consent |
|---|---|---|---|
| `openid`, `profile`, `offline_access` | At sign-in | Token endpoint (identity, refresh token) | No |
| `User.Read` | At sign-in | `GET /me` | No |
| `User.ReadBasic.All` | At sign-in | `GET /users/{id}/photos/96x96/$value`, `GET /users?$search=…` (people search for new chats and channel mentions) | No |
| `Chat.ReadWrite` | At sign-in | `GET /me/chats`, `GET /chats/{id}/messages`, `PATCH …/messages/{id}`, `POST /users/{id}/chats/{id}/messages/{id}/softDelete`, `POST /chats/{id}/markChatReadForUser`, `POST /search/query`, `POST /subscriptions` on `/users/{id}/chats/getAllMessages` (push relay only) | Excluded from the default user-consent policy |
| `ChatMessage.Send` | At sign-in | `POST /chats/{id}/messages`, `POST …/replyWithQuote`, `setReaction`, `unsetReaction` | No |
| `Team.ReadBasic.All` | At sign-in | `GET /me/joinedTeams` | No |
| `Channel.ReadBasic.All` | At sign-in | `GET /teams/{id}/allChannels` | No |
| `ChannelMessage.Send` | At sign-in | `POST /teams/{id}/channels/{id}/messages`, `…/replies`, reactions | No |
| `Presence.Read.All` | At sign-in | `POST /communications/getPresencesByUserId`, `GET /me/presence`, `POST /subscriptions` on `communications/presences/{id}` (your own presence, push relay only) | No |
| `Calendars.Read` | At sign-in | `GET /me/calendarView` | Excluded from the default user-consent policy |
| `Chat.Create` | First new chat | `POST /chats`, then `GET /chats/{id}?$expand=members` | No |
| `Files.ReadWrite` | First file sent | `PUT /me/drive/root:/Microsoft Teams Chat Files/{name}:/content`, `POST …:/createUploadSession` (above 4 MiB), `GET /me/drive/items/{id}`, `POST /me/drive/items/{id}/invite` (chats), `POST …/createLink` (channels) | No |
| `Presence.ReadWrite` | First status change, or the presence session | `POST /users/{id}/presence/setUserPreferredPresence`, `clearUserPreferredPresence`, `setStatusMessage`, `setPresence`, `clearPresence` | No |
| `OnlineMeetings.Read` | First "Meeting chat" during a call | `GET /me/onlineMeetings?$filter=JoinWebUrl eq '…'` (finds a meeting's chat from its join link) | No |
| `ChannelMessage.Read.All` | First channel opened | `GET /teams/{id}/channels/{id}/messages`, `…/replies` | **Yes** |

## Not requested

| Permission | Would enable | Why not yet |
|---|---|---|
| `ChannelMessage.ReadWrite` | Editing and deleting channel posts | Requires admin consent; the interface hides these actions without it |
| `OnlineMeetings.ReadWrite`, `Calendars.ReadWrite` | Creating meetings | Not in the interface yet |
| `Calls.*` | Graph calling | Application-only (bots). See [calling.md](calling.md) |

## Revoking access

Sign out in Wren (**Settings → Sign out**) to delete the refresh token from
the OS credential store and remove the account's local cache. To revoke the
grant at Microsoft, open [My Apps](https://myapps.microsoft.com) or ask an
administrator to remove the app's consent in Entra ID.
