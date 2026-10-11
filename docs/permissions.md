# Permissions

Wren uses **delegated** Microsoft Graph permissions only. It acts as the
signed-in person, and can never see more than that person can in Teams.
It asks for the smallest set that covers its features. It requests the one
permission that needs admin consent, `ChannelMessage.Read.All`, separately
and only when someone opens a channel (incremental consent).

| Permission | Requested | Endpoints | Admin consent |
|---|---|---|---|
| `openid`, `profile`, `offline_access` | At sign-in | Token endpoint (identity, refresh token) | No |
| `User.Read` | At sign-in | `GET /me` | No |
| `User.ReadBasic.All` | At sign-in | `GET /users/{id}/photos/96x96/$value` | No |
| `Chat.ReadWrite` | At sign-in | `GET /me/chats`, `GET /chats/{id}/messages`, `PATCH …/messages/{id}`, `POST /users/{id}/chats/{id}/messages/{id}/softDelete`, `POST /chats/{id}/markChatReadForUser`, `POST /search/query`, `POST /subscriptions` on `/users/{id}/chats/getAllMessages` (push relay only) | Excluded from the default user-consent policy |
| `ChatMessage.Send` | At sign-in | `POST /chats/{id}/messages`, `POST …/replyWithQuote`, `setReaction`, `unsetReaction` | No |
| `Team.ReadBasic.All` | At sign-in | `GET /me/joinedTeams` | No |
| `Channel.ReadBasic.All` | At sign-in | `GET /teams/{id}/allChannels` | No |
| `ChannelMessage.Send` | At sign-in | `POST /teams/{id}/channels/{id}/messages`, `…/replies`, reactions | No |
| `Presence.Read.All` | At sign-in | `POST /communications/getPresencesByUserId` | No |
| `Calendars.Read` | At sign-in | `GET /me/calendarView` | Excluded from the default user-consent policy |
| `ChannelMessage.Read.All` | On demand | `GET /teams/{id}/channels/{id}/messages`, `…/replies` | **Yes** |

## Not requested

| Permission | Would enable | Why not yet |
|---|---|---|
| `Chat.Create` | Starting new chats | Not in the interface yet |
| `Presence.ReadWrite` | Setting your own availability and status message | Not in the interface yet; also needs a presence session |
| `Files.ReadWrite` | Sending files (OneDrive upload) | Not in the interface yet |
| `ChannelMessage.ReadWrite` | Editing and deleting channel posts | Requires admin consent; the interface hides these actions without it |
| `OnlineMeetings.ReadWrite`, `Calendars.ReadWrite` | Creating meetings | Not in the interface yet |
| `Calls.*` | Graph calling | Application-only (bots). See [calling.md](calling.md) |

## Revoking access

Sign out in Wren (**Settings → Sign out**) to delete the refresh token from
the OS credential store and remove the account's local cache. To revoke the
grant at Microsoft, open [My Apps](https://myapps.microsoft.com) or ask an
administrator to remove the app's consent in Entra ID.
