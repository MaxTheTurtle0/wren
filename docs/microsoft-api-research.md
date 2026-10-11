# Microsoft API research

What Microsoft's official APIs let a third-party desktop client for Teams do,
checked against learn.microsoft.com on 2026-10-11. Every claim cites its
source (`[Rn]`, listed at the end). **UNVERIFIED** marks what the docs did
not settle; those need an empirical test before Wren relies on them.

Wren uses only documented APIs: Microsoft Graph with delegated permissions,
signed in through Entra ID. It never calls undocumented Teams endpoints and
never presents itself as Microsoft's client.

## Headlines

1. **Chat works in Graph v1.0 with delegated permissions**: list, read,
   send, edit, delete, react, quote-reply, mark read/unread, inline images.
   Personal Microsoft accounts and Teams (free) are not supported.
2. **Real-time delivery is the hard part.** Polling Graph for changes more
   than once a day is forbidden and is a Terms of Use violation [R43].
   Push needs a public HTTPS webhook, Azure Event Hubs, or Azure Event Grid.
   Wren therefore fetches only when the person looks (see
   [realtime.md](realtime.md)).
3. **Consent is the main adoption risk.** `Chat.Read`, `Chat.ReadWrite` and
   `Calendars.*` are excluded from the Microsoft-managed default user-consent
   policy, which is the default for new tenants [R86], and users cannot
   consent to unverified multi-tenant apps in many tenants [R87][R88]. Expect
   an administrator to approve Wren's permissions.
4. **Calling as the user is not possible through Graph** (application-only,
   for bots). The only documented route is Azure Communication Services
   (ACS) "Teams users", which needs an Azure resource and a token-exchange
   backend and has Calling SDKs only for the web, Windows (MSIX), iOS and
   Android. On 2026-09-23 Microsoft announced the retirement of ACS as a
   standalone offering effective 2028-09-30, with Teams interop listed as
   unaffected [R69]. Wren hands calls off to Teams (see
   [calling.md](calling.md)).
5. **Throttling is counted per client ID across all tenants.** One shared
   client ID would share one global budget (for example 200 requests/s
   across all tenants for `GET /chats/{id}/messages`) [R42]. Wren has every
   organisation register its own app.

## 1. Entra ID public client

- **Redirect URI**: platform "Mobile and desktop applications", value
  `http://localhost` [R2]. The port is ignored when matching localhost
  redirect URIs, so the app may bind any free port at run time [R1]. IPv6
  `[::1]` is not supported as a registered value [R1]. `http://127.0.0.1` is
  preferred by the docs but can only be set through the manifest [R1]; Wren
  sends `http://localhost:{port}` and listens on both loopback addresses.
- **Allow public client flows** is needed for the device code flow [R2].
  Public clients must not use secrets [R4]; PKCE is recommended for all apps
  [R4].
- **Device code** works [R7] but Conditional Access can block it, and
  Microsoft recommends blocking it where possible [R8][R9]. Wren offers it
  as a fallback only.
- **Authority**: `https://login.microsoftonline.com/organizations/oauth2/v2.0/`
  [R4][R5]; Teams chat, channel, presence and meeting APIs do not support
  personal accounts [R13][R14][R16].
- **Refresh tokens** need `offline_access`; they last 90 days outside SPAs,
  rotate on each use, and can be revoked [R3].
- **Client IDs are public values** [R6]; shipping one in open source is fine.
- **Do not reuse Microsoft's first-party client IDs** (Graph Explorer,
  Graph Command Line Tools) [R10]: the API Terms require registering your own
  application and forbid obscuring the source of requests [R11]. Wren never
  does.

## 2. Chats (Graph v1.0)

| Operation | Endpoint | Least-privileged delegated permission | Notes |
|---|---|---|---|
| List chats | `GET /me/chats?$expand=members,lastMessagePreview&$orderby=lastMessagePreview/createdDateTime desc` | Chat.ReadBasic | `$top` ≤ 50; `members` capped at 25 when expanded; `viewpoint.lastMessageReadDateTime` gives unread state [R14]. Whether `lastMessagePreview` works with only Chat.ReadBasic is UNVERIFIED. |
| List messages | `GET /chats/{id}/messages` | Chat.Read | `$top` ≤ 50; `$orderby` `lastModifiedDateTime`/`createdDateTime` desc only; `$filter` on those with matching `$orderby` [R15] |
| Send | `POST /chats/{id}/messages` | ChatMessage.Send | Using Teams "as a log file" violates the terms [R16] |
| Edit | `PATCH /chats/{id}/messages/{id}` | Chat.ReadWrite | 204 [R18] |
| Delete | `POST /users/{id}/chats/{id}/messages/{id}/softDelete` | Chat.ReadWrite | [R19] |
| Reactions | `setReaction` / `unsetReaction` | ChatMessage.Send | v1.0, Unicode emoji [R20] |
| Quote reply | `POST /chats/{id}/messages/replyWithQuote` | ChatMessage.Send | v1.0, up to 10 quotes; chats reply by reference, `replyToId` is for channels [R21][R26] |
| Inline images | `hostedContents` | ChatMessage.Send / Chat.Read | 4 MB max [R17][R23] |
| Mark read/unread | `markChatReadForUser` / `markChatUnreadForUser` | Chat.ReadWrite | [R22] |
| New chat | `POST /chats` | Chat.Create | Returns the existing 1:1 chat [R25] |
| All messages / delta | `getAllMessages`, `getAllMessages/delta` | application only | Export APIs [R24][R32] |

## 3. Teams and channels

- `GET /me/joinedTeams`: Team.ReadBasic.All, no admin consent [R27].
- `GET /teams/{id}/allChannels` (includes shared channels):
  Channel.ReadBasic.All, no admin consent [R28].
- Reading channel posts and replies (`$expand=replies`, `$top` ≤ 50):
  **ChannelMessage.Read.All, admin consent required** [R29][R30][R13].
- Posting, replying, reacting: ChannelMessage.Send, no admin consent
  [R31][R20].
- Editing or deleting channel messages: ChannelMessage.ReadWrite, admin
  consent required [R18][R19][R13]. A newer `ChannelMessage.Edit` appears in
  the permissions reference but not yet on the API pages (UNVERIFIED).
- Delegated channel-message delta: the delta page now documents only the
  application-only export; treat delegated delta as unavailable
  (UNVERIFIED) [R32][R44].

## 4. Real-time updates and throttling

Delegated subscriptions exist for `/chats/{id}/messages` (Chat.Read),
`/users/{id}/chats/getAllMessages` (Chat.Read), and
`/teams/{id}/channels/{id}/messages` (ChannelMessage.Read.All) [R33]. They
last at most 3 days, need an RSA certificate to include resource data, and
share a 10,000-per-organisation quota [R34][R35][R36].

Delivery options:

| Option | Needs |
|---|---|
| Webhook | A public HTTPS URL [R35] |
| Azure Event Hubs | An Event Hubs namespace [R37] |
| Azure Event Grid | An Azure subscription and partner topic [R38] |
| socket.io | Beta, drive/list only, not supported in production [R39] |
| Web Push | GA in v1.0 since August 2026 [R34][R40][R41]; support for Teams chat messages and for non-browser clients is UNVERIFIED |

**Polling rule** (verbatim, [R43]): "If your app polls to see whether a
resource has changed, you can only do that once per day." "It's okay to GET
/me/joinedTeams every time the user visits/refreshes your web page, but it
isn't okay to GET /me/joinedTeams in a loop every 30 seconds." "Apps that
don't follow these polling requirements will be considered in violation of
the Microsoft APIs Terms of Use."

Teams throttling limits (page updated 2026-09-17) [R42]:

| Request | Per app (all tenants) | Per app per tenant | Per resource | Per user |
|---|---|---|---|---|
| GET `/me/chats` | 200 rps | 20 rps | – | 1 rps |
| GET `/chats/{id}/messages` | 200 | 20 | 1 rps per chat | – |
| POST `/chats/{id}/messages` | 200 | 20 | 1 rps per chat | 1 |
| PATCH / softDelete | 300 | 30 | 1 rps per chat | – |
| GET hostedContents | 600 | 60 | 1 rps per chat | – |
| GET channel messages | 200 | 20 | 1 rps per channel | – |
| POST channel message/reply | 500 | 50 | 1 rps per channel | 1 |
| GET `/me/joinedTeams` | 300 | 30 | – | – |

A sustained limit of about 83% of these applies over longer windows [R42].
Presence: 10,000 requests per 30 seconds per app per tenant [R42].

## 5. Presence

| Action | Endpoint | Permission |
|---|---|---|
| Own presence | `GET /me/presence` | Presence.Read [R45] |
| Others' presence | `POST /communications/getPresencesByUserId` (≤ 650 ids) | Presence.Read.All, no admin consent [R46][R13] |
| Set preferred presence | `setUserPreferredPresence` | Presence.ReadWrite; only takes effect with an active presence session [R47] |
| Status message | `setStatusMessage` | Presence.ReadWrite [R48] |
| Presence session | `setPresence` (sessionId = app's client ID, 5–240 min) | Presence.ReadWrite [R49] |
| Change notifications | subscription, 1 hour max | Presence.Read.All [R50] |

## 6. Calendar and meetings

- `GET /me/calendarView` with Calendars.ReadBasic or Calendars.Read [R51];
  use `onlineMeeting.joinUrl` (`onlineMeetingUrl` is being deprecated) [R52].
  Whether ReadBasic returns `joinUrl` is UNVERIFIED, so Wren asks for
  Calendars.Read.
- Creating meetings: an event with `isOnlineMeeting` (Calendars.ReadWrite)
  or `POST /me/onlineMeetings` (OnlineMeetings.ReadWrite) [R53][R54].
- Joining: open `joinUrl`; `https://` links offer the desktop or web client,
  `msteams://` goes straight to the desktop client [R72].

## 7. Calling

See [calling.md](calling.md) for the full write-up. In short:

- Graph `POST /communications/calls`: delegated "Not supported"; bots only,
  application permissions with admin consent; app-hosted media must be C#
  on Windows Server in Azure [R55][R56][R57][R58][R13].
- ACS "Teams users": an Entra token with `Teams.ManageCalls` /
  `Teams.ManageChats` is exchanged for an ACS token against **your ACS
  resource**, recommended in a backend because the exchange is signed with a
  secret [R61][R62]. Supports VoIP and PSTN calls, incoming calls, meetings
  and group calls up to 350; does not join existing Teams 1:1 or group calls
  [R59][R60]. Needs a Teams licence; billed to the ACS resource [R63][R64].
- ACS Calling SDKs: web (JS), Windows (UWP/WinUI, MSIX required), iOS,
  Android; no native macOS, Linux or C/C++ SDK; closed source [R65][R66].
  Linux browsers: Chrome only; embedded webviews not officially supported
  [R65].
- Anonymous/guest meeting join through ACS: lobby, tenant setting, not for
  1:1 or group calls [R67][R68].
- Call deep link: `https://teams.microsoft.com/l/call/0/0?users=<upn>[&withVideo=true]`;
  Teams asks the person to confirm [R71].
- No documented delegated way to ring or receive calls without ACS;
  `callRecords` is application-only and after the fact [R73][R13].
- ACS retirement as a standalone offering, effective 2028-09-30; Teams
  interop not impacted; ACS Chat retired in favour of Graph [R69].

## 8. Search

`POST /search/query` with `entityTypes: ["chatMessage"]` is delegated only;
Chat.Read or Chat.ReadWrite (channel posts also need
ChannelMessage.Read.All). Returns only messages the person sent or received,
a subset of properties, no sorting, paging with `from`/`size` [R74][R75][R76].

## 9. Files

- Upload to OneDrive: `PUT /me/drive/root:/{path}:/content` up to 250 MB,
  `createUploadSession` beyond, Files.ReadWrite [R77][R78].
- Send a `reference` attachment (driveItem eTag GUID, `webDavUrl`, an
  `<attachment id=…>` in the body); recipients need access, granted with
  `invite` or `createLink` [R17][R79][R80]. The official client's
  "Microsoft Teams Chat Files" folder convention is only described in
  training material [R83].
- Channel files: `filesFolder`, Files.Read.All [R81]; shared links:
  `/shares/{u!…}` [R82].

## 10. Protected and metered APIs

Teams APIs are no longer metered since 2025-08-25 [R84]. The
protected-API request process was deprecated in 2023 and only ever applied to
application permissions. Neither affects Wren.

## 11. Consent

- Admin consent always required: ChannelMessage.Read.All,
  ChannelMessage.ReadWrite, ChatMember.Read(Write) [R13].
- User-consentable by flag: Chat.*, ChatMessage.Send, Team.ReadBasic.All,
  Channel.ReadBasic.All, Presence.*, Calendars.*, OnlineMeetings.ReadWrite,
  Files.* [R13].
- The Microsoft-managed default policy (default for new tenants) excludes
  Chat.Read, Chat.ReadWrite, Calendars.*, OnlineMeetings.* and others from
  user consent [R86]. Other settings: no user consent, verified publishers
  only, or legacy "allow all" [R85].
- Users can't consent to most new multi-tenant apps that are not
  publisher-verified [R87][R88]. Verification needs a Microsoft partner ID
  and a verified domain [R87].
- The admin consent workflow is off by default [R89].

**For Wren**: expect admin consent; offer the admin consent link; ask for
optional scopes incrementally (channel reading); let each organisation
register its own app.

## 12. Terms and branding

- Microsoft APIs Terms of Use (October 2025) [R11]: no clause bans
  alternative clients. Relevant: request minimum permissions; respect
  limits and do not work around them; keep only the copies of data needed;
  provide a privacy statement and a way to revoke access; clause 11(j) on
  the EU Teams commitments (impact on Graph UNVERIFIED).
- Identity Platform Terms [R12]: show an EULA disclaiming Microsoft's
  liability and a privacy statement; no Microsoft trademarks without
  approval; no confusing names.
- Branding: Teams branding is for certified partners; no Microsoft logos;
  nothing implying endorsement [R70][R72][R91]. "Wren" is fine; describe it
  as working with Microsoft Teams, never as "Teams …", and never use the
  Teams logo.

## References

[R1]: https://learn.microsoft.com/en-us/entra/identity-platform/reply-url
[R2]: https://learn.microsoft.com/en-us/entra/identity-platform/scenario-desktop-app-configuration
[R3]: https://learn.microsoft.com/en-us/entra/identity-platform/refresh-tokens
[R4]: https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow
[R5]: https://learn.microsoft.com/en-us/entra/identity-platform/v2-protocols
[R6]: https://learn.microsoft.com/en-us/entra/identity-platform/msal-client-applications
[R7]: https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code
[R8]: https://learn.microsoft.com/en-us/entra/identity/conditional-access/concept-authentication-flows
[R9]: https://learn.microsoft.com/en-us/entra/identity/conditional-access/managed-policies
[R10]: https://learn.microsoft.com/en-us/troubleshoot/entra/entra-id/governance/verify-first-party-apps-sign-in
[R11]: https://learn.microsoft.com/en-us/legal/microsoft-apis/terms-of-use
[R12]: https://learn.microsoft.com/en-us/legal/microsoft-identity-platform/terms-of-use
[R13]: https://learn.microsoft.com/en-us/graph/permissions-reference
[R14]: https://learn.microsoft.com/en-us/graph/api/chat-list?view=graph-rest-1.0
[R15]: https://learn.microsoft.com/en-us/graph/api/chat-list-messages?view=graph-rest-1.0
[R16]: https://learn.microsoft.com/en-us/graph/api/chat-post-messages?view=graph-rest-1.0
[R17]: https://learn.microsoft.com/en-us/graph/api/chatmessage-post?view=graph-rest-1.0
[R18]: https://learn.microsoft.com/en-us/graph/api/chatmessage-update?view=graph-rest-1.0
[R19]: https://learn.microsoft.com/en-us/graph/api/chatmessage-softdelete?view=graph-rest-1.0
[R20]: https://learn.microsoft.com/en-us/graph/api/chatmessage-setreaction?view=graph-rest-1.0
[R21]: https://learn.microsoft.com/en-us/graph/api/chatmessage-replywithquote?view=graph-rest-1.0
[R22]: https://learn.microsoft.com/en-us/graph/api/chat-markchatreadforuser?view=graph-rest-1.0
[R23]: https://learn.microsoft.com/en-us/graph/api/chatmessagehostedcontent-get?view=graph-rest-1.0
[R24]: https://learn.microsoft.com/en-us/graph/api/chats-getallmessages?view=graph-rest-1.0
[R25]: https://learn.microsoft.com/en-us/graph/api/chat-post?view=graph-rest-1.0
[R26]: https://learn.microsoft.com/en-us/graph/teams-messaging-overview
[R27]: https://learn.microsoft.com/en-us/graph/api/user-list-joinedteams?view=graph-rest-1.0
[R28]: https://learn.microsoft.com/en-us/graph/api/team-list-allchannels?view=graph-rest-1.0
[R29]: https://learn.microsoft.com/en-us/graph/api/channel-list-messages?view=graph-rest-1.0
[R30]: https://learn.microsoft.com/en-us/graph/api/chatmessage-list-replies?view=graph-rest-1.0
[R31]: https://learn.microsoft.com/en-us/graph/api/channel-post-messages?view=graph-rest-1.0
[R32]: https://learn.microsoft.com/en-us/graph/api/chatmessage-delta?view=graph-rest-1.0
[R33]: https://learn.microsoft.com/en-us/graph/teams-changenotifications-chatmessage
[R34]: https://learn.microsoft.com/en-us/graph/api/resources/subscription?view=graph-rest-1.0
[R35]: https://learn.microsoft.com/en-us/graph/change-notifications-overview
[R36]: https://learn.microsoft.com/en-us/graph/change-notifications-with-resource-data
[R37]: https://learn.microsoft.com/en-us/graph/change-notifications-delivery-event-hubs
[R38]: https://learn.microsoft.com/en-us/azure/event-grid/subscribe-to-graph-api-events
[R39]: https://learn.microsoft.com/en-us/graph/api/subscriptions-socketio?view=graph-rest-beta
[R40]: https://learn.microsoft.com/en-us/graph/api/subscription-getvapidpublickey?view=graph-rest-1.0
[R41]: https://learn.microsoft.com/en-us/graph/whats-new-overview
[R42]: https://learn.microsoft.com/en-us/graph/throttling-limits
[R43]: https://learn.microsoft.com/en-us/graph/api/resources/teams-api-overview?view=graph-rest-1.0#polling-requirements
[R44]: https://learn.microsoft.com/en-us/graph/delta-query-overview
[R45]: https://learn.microsoft.com/en-us/graph/api/presence-get?view=graph-rest-1.0
[R46]: https://learn.microsoft.com/en-us/graph/api/cloudcommunications-getpresencesbyuserid?view=graph-rest-1.0
[R47]: https://learn.microsoft.com/en-us/graph/api/presence-setuserpreferredpresence?view=graph-rest-1.0
[R48]: https://learn.microsoft.com/en-us/graph/api/presence-setstatusmessage?view=graph-rest-1.0
[R49]: https://learn.microsoft.com/en-us/graph/api/presence-setpresence?view=graph-rest-1.0
[R50]: https://learn.microsoft.com/en-us/graph/changenotifications-for-presence
[R51]: https://learn.microsoft.com/en-us/graph/api/user-list-calendarview?view=graph-rest-1.0
[R52]: https://learn.microsoft.com/en-us/graph/api/resources/event?view=graph-rest-1.0
[R53]: https://learn.microsoft.com/en-us/graph/outlook-calendar-online-meetings
[R54]: https://learn.microsoft.com/en-us/graph/api/application-post-onlinemeetings?view=graph-rest-1.0
[R55]: https://learn.microsoft.com/en-us/graph/api/application-post-calls?view=graph-rest-1.0
[R56]: https://learn.microsoft.com/en-us/graph/api/call-answer?view=graph-rest-1.0
[R57]: https://learn.microsoft.com/en-us/microsoftteams/platform/bots/calls-and-meetings/calls-meetings-bots-overview
[R58]: https://learn.microsoft.com/en-us/microsoftteams/platform/bots/calls-and-meetings/requirements-considerations-application-hosted-media-bots
[R59]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/teams-endpoint
[R60]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/interop/teams-user-calling
[R61]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/interop/custom-teams-endpoint-authentication-overview
[R62]: https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/manage-teams-identity
[R63]: https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/eligible-teams-licenses
[R64]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/pricing/teams-interop-pricing
[R65]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/voice-video-calling/calling-sdk-features
[R66]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/sdk-options
[R67]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/join-teams-meeting
[R68]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/interop/guest/overview
[R69]: https://learn.microsoft.com/en-us/azure/communication-services/acs-retirement-and-breaking-changes-guide
[R70]: https://learn.microsoft.com/en-us/azure/communication-services/concepts/teams-interop
[R71]: https://learn.microsoft.com/en-us/microsoftteams/platform/concepts/build-and-test/deep-link-workflow
[R72]: https://learn.microsoft.com/en-us/microsoftteams/platform/concepts/build-and-test/deep-links
[R73]: https://learn.microsoft.com/en-us/graph/api/callrecords-callrecord-get?view=graph-rest-1.0
[R74]: https://learn.microsoft.com/en-us/graph/search-concept-chat-messages
[R75]: https://learn.microsoft.com/en-us/graph/api/search-query?view=graph-rest-1.0
[R76]: https://learn.microsoft.com/en-us/graph/api/resources/search-api-overview?view=graph-rest-1.0
[R77]: https://learn.microsoft.com/en-us/graph/api/driveitem-put-content?view=graph-rest-1.0
[R78]: https://learn.microsoft.com/en-us/graph/api/driveitem-createuploadsession?view=graph-rest-1.0
[R79]: https://learn.microsoft.com/en-us/graph/api/driveitem-invite?view=graph-rest-1.0
[R80]: https://learn.microsoft.com/en-us/graph/api/driveitem-createlink?view=graph-rest-1.0
[R81]: https://learn.microsoft.com/en-us/graph/api/channel-get-filesfolder?view=graph-rest-1.0
[R82]: https://learn.microsoft.com/en-us/graph/api/shares-get?view=graph-rest-1.0
[R83]: https://learn.microsoft.com/en-us/training/modules/explore-teams/7-sharepoint-onedrive
[R84]: https://learn.microsoft.com/en-us/graph/metered-api-list
[R85]: https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/configure-user-consent
[R86]: https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/manage-app-consent-policies
[R87]: https://learn.microsoft.com/en-us/entra/identity-platform/publisher-verification-overview
[R88]: https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/configure-risk-based-step-up-consent
[R89]: https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/configure-admin-consent-workflow
[R91]: https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks
