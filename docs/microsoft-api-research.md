# Microsoft API reference

API constraints used by Wren 0.1. [Feature support](features.md) records what
is implemented and verified; [permissions](permissions.md) maps features to
scopes, and [app registration](azure-app-registration.md) covers setup.

Wren uses documented Microsoft Graph APIs with delegated permissions and
Entra ID sign-in. It never impersonates Microsoft's clients or calls
undocumented Teams endpoints.

## Identity and consent

- Use a public client with authorization code + PKCE, a loopback redirect,
  and no client secret. Device code is a fallback and may be blocked by
  Conditional Access. See Microsoft's [code flow][code-flow] and
  [device code flow][device-flow].
- Wren registers `http://localhost` and binds a free port. Entra ignores
  the port when matching localhost redirect URIs. See [redirect rules][redirect].
- Wren's Teams features require work or school accounts. The
  [chat API][chats] does not support delegated personal Microsoft accounts.
- Permission flags alone do not guarantee user consent. The
  [Microsoft-managed policy][consent] restricts chat, calendar and meeting
  scopes, and [publisher verification][publisher] affects multi-tenant apps.
  Expect an administrator to approve Wren in many tenants. Request optional
  scopes only when their feature is used.

## Graph behavior

| Area | Constraint and official reference |
|---|---|
| [Chat list][chats] | Maximum page size 50; expanded members are capped at 25 |
| [Chat messages][messages] | Maximum page size 50; descending creation/modification order, with matching date filters; follow paging links |
| [Message export][export] | `GET /chats/getAllMessages` is application-only; it is not the delegated notification resource below |
| [Channel messages][channels] | Wren uses `ChannelMessage.Read.All` for reads, requested incrementally with admin consent |
| [Presence sessions][presence] | Sessions expire; Wren updates its session only on input and clears it at sign-out or exit |
| [Calendar events][calendar] | Use `onlineMeeting.joinUrl` for meeting links; Wren requests `Calendars.Read` |
| [Meeting lookup][meeting] | Lookup by `JoinWebUrl` can return the meeting chat; Wren requests `OnlineMeetings.Read` on first use |
| [Message search][search] | Delegated search returns messages the person sent or received, with a subset of properties and `from`/`size` paging |
| [Resumable uploads][uploads] | Upload chunks to the returned `uploadUrl` without the Graph Authorization header |

For message attachments and sharing permissions, see [permissions](permissions.md)
and [privacy](privacy.md). Do not fetch message-provided images from other hosts
or send Graph credentials outside the Graph origin.

## Refresh and throttling

Microsoft's [Teams polling requirements][polling] allow repeated fetches
when a person visits or refreshes a view, but restrict unattended polling
for changes to once a day. Wren adds no polling timers; every fetch goes
through `sync::Freshness::claim`. Refresh messages by date range where supported.

[Chat notifications][notifications] support delegated subscriptions for
`/users/{id}/chats/getAllMessages` and individual chats. Wren uses a
self-hosted HTTPS relay and requests no resource data, so the relay receives
no message content. Respect subscription expiration and lifecycle notices.
The implementation and deployment steps are in [realtime.md](realtime.md).

Honor `429` and `Retry-After`; do not retry immediately. Microsoft's
[throttling guidance][throttling] and [service limits][limits] cover limits
per app, tenant, resource and user. Check the source when changing request
patterns rather than relying on a copied limit table.

## Calling

Graph's [create-call API][calls] supports bots, not delegated user calling.
Microsoft's [Teams interop pricing][pricing] distinguishes calls in its own
Teams clients from billable ACS usage. Wren opens documented
[Teams deep links][deep-links] and keeps media and call controls in Teams.
See [calling.md](calling.md) for current behavior and remaining live checks.

## Terms and branding

Keep the [Microsoft APIs Terms of Use][api-terms] and
[Identity Platform Terms][identity-terms] as the source for registration,
minimum permissions/data, privacy, user terms, and limits on use. Provide
the privacy statement and revocation instructions in [privacy.md](privacy.md)
and [permissions.md](permissions.md).

Follow Microsoft's [trademark guidelines][trademarks]. Wren is independent
and works with Microsoft Teams; names and artwork must not imply endorsement.

[code-flow]: https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow
[device-flow]: https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code
[redirect]: https://learn.microsoft.com/en-us/entra/identity-platform/reply-url
[chats]: https://learn.microsoft.com/en-us/graph/api/chat-list?view=graph-rest-1.0
[messages]: https://learn.microsoft.com/en-us/graph/api/chat-list-messages?view=graph-rest-1.0
[export]: https://learn.microsoft.com/en-us/graph/api/chats-getallmessages?view=graph-rest-1.0
[channels]: https://learn.microsoft.com/en-us/graph/api/channel-list-messages?view=graph-rest-1.0
[presence]: https://learn.microsoft.com/en-us/graph/api/presence-setpresence?view=graph-rest-1.0
[calendar]: https://learn.microsoft.com/en-us/graph/api/resources/event?view=graph-rest-1.0
[meeting]: https://learn.microsoft.com/en-us/graph/api/onlinemeeting-get?view=graph-rest-1.0
[search]: https://learn.microsoft.com/en-us/graph/search-concept-chat-messages
[uploads]: https://learn.microsoft.com/en-us/graph/api/driveitem-createuploadsession?view=graph-rest-1.0
[notifications]: https://learn.microsoft.com/en-us/graph/teams-changenotifications-chatmessage
[polling]: https://learn.microsoft.com/en-us/graph/api/resources/teams-api-overview?view=graph-rest-1.0#polling-requirements
[throttling]: https://learn.microsoft.com/en-us/graph/throttling
[limits]: https://learn.microsoft.com/en-us/graph/throttling-limits
[consent]: https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/manage-app-consent-policies
[publisher]: https://learn.microsoft.com/en-us/entra/identity-platform/publisher-verification-overview
[calls]: https://learn.microsoft.com/en-us/graph/api/application-post-calls?view=graph-rest-1.0
[pricing]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/pricing/teams-interop-pricing
[deep-links]: https://learn.microsoft.com/en-us/microsoftteams/platform/concepts/build-and-test/deep-link-workflow
[api-terms]: https://learn.microsoft.com/en-us/legal/microsoft-apis/terms-of-use
[identity-terms]: https://learn.microsoft.com/en-us/legal/microsoft-identity-platform/terms-of-use
[trademarks]: https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks
