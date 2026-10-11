# Registering Wren with Microsoft Entra ID

Wren signs in through an app registration in your organisation's Microsoft
Entra ID (formerly Azure AD). Wren ships no shared client ID, for three
reasons:

- Microsoft counts Teams throttling **per client ID across all tenants**,
  so a shared registration would share one global request budget
  ([research §4](microsoft-api-research.md#4-real-time-updates-and-throttling)).
- Most tenants block users from consenting to unverified multi-tenant apps,
  and the default consent policy excludes `Chat.*` and `Calendars.*`
  ([research §11](microsoft-api-research.md#11-consent)).
- Reusing one of Microsoft's own client IDs would breach the API terms.

The registration takes about five minutes and costs nothing. App
registrations are part of the free tier of Entra ID that every Microsoft 365
organisation has, and need no Azure subscription. The Graph calls Wren makes
are not billed either: Microsoft stopped metering the Teams APIs on
2025-08-25. It needs no client secret, because Wren is a public client using
PKCE.

## 1. Create the registration

1. Open the [Entra admin center](https://entra.microsoft.com) and go to
   **Identity → Applications → App registrations → New registration**.
2. **Name**: `Wren` (or anything your users will recognise on the consent
   screen).
3. **Supported account types**: *Accounts in this organizational directory
   only* (single tenant) is the simplest. Choose *any organizational
   directory* (multi-tenant) if you will add accounts from more than one
   organisation, such as work and school (see
   [Accounts in several organisations](#accounts-in-several-organisations)).
4. **Redirect URI**: platform **Public client/native (mobile & desktop)**,
   value `http://localhost`. Entra ignores the port for `localhost`, so Wren
   can listen on any free port.
5. Select **Register**, then copy the **Application (client) ID** and the
   **Directory (tenant) ID** from the overview page.

## 2. Authentication settings

Under **Authentication**:

- Check that `http://localhost` is listed under *Mobile and desktop
  applications*.
- Set **Allow public client flows** to **Yes** only if you want the
  "Use a code instead" (device code) sign-in. The browser sign-in does not
  need it. Conditional Access policies often block device code flow.

## 3. API permissions

Under **API permissions → Add a permission → Microsoft Graph → Delegated
permissions**, add:

| Permission | Why | Admin consent |
|---|---|---|
| `User.Read` | Your profile | No |
| `User.ReadBasic.All` | Names and photos of people in your chats; finding people for new chats and mentions | No |
| `Chat.ReadWrite` | List chats, read messages, edit, delete, mark read | Not by flag, but excluded from the default user-consent policy |
| `ChatMessage.Send` | Send messages, quote replies, reactions | No |
| `Team.ReadBasic.All` | The teams you belong to | No |
| `Channel.ReadBasic.All` | Their channels | No |
| `ChannelMessage.Send` | Post and reply in channels | No |
| `Presence.Read.All` | Availability of people in your chats | No |
| `Calendars.Read` | Upcoming meetings and their join links | Not by flag, but excluded from the default user-consent policy |
| `Chat.Create` *(optional)* | Starting new chats | No |
| `Files.ReadWrite` *(optional)* | Sending files (upload to your OneDrive and share) | No |
| `Presence.ReadWrite` *(optional)* | Setting your availability and status message | No |
| `OnlineMeetings.Read` *(optional)* | Finding a meeting's chat to show beside the call | No |
| `ChannelMessage.Read.All` *(optional)* | Reading channel posts | **Yes** |

`openid`, `profile` and `offline_access` are added automatically during
sign-in. Wren requests the optional permissions only when someone first uses
the feature. Add them here anyway: a tenant that blocks user consent can then
approve everything at once.

Then select **Grant admin consent for <your organisation>** if you are an
administrator. Otherwise send your administrator the link Wren shows under
**Settings → App registration** (`…/adminconsent?client_id=…`).

[permissions.md](permissions.md) explains each permission and the
endpoints that use it.

## 4. Point Wren at it

Use any of these:

- On Wren's welcome screen, enter the client ID and the tenant, then select
  **Save**. For a single-tenant registration (the default above) the tenant
  **must** be your directory (tenant) ID or a verified domain such as
  `contoso.onmicrosoft.com`. `organizations` only works for multi-tenant
  registrations; with a single-tenant one, Entra answers AADSTS50059 or
  AADSTS50194.
- Set them in the environment:

  ```sh
  WREN_CLIENT_ID=00000000-0000-0000-0000-000000000000 WREN_TENANT=contoso.onmicrosoft.com wren
  ```

- Edit `settings.json` in Wren's config folder:

  | Platform | Config folder |
  |---|---|
  | macOS | `~/Library/Application Support/Wren` |
  | Windows | `%APPDATA%\Wren\config` |
  | Linux | `~/.config/wren` |

Then select **Sign in with Microsoft**. Your browser opens Microsoft's sign-in
page. After you approve, the browser tab says "You're signed in" and Wren
continues.

## Accounts in several organisations

Wren uses one registration for all its accounts. To add accounts from more
than one organisation, for example a work account and a school account:

- Register the app as multi-tenant (*Accounts in any organizational
  directory*; `--sign-in-audience AzureADMultipleOrgs` with the Azure CLI),
  in whichever organisation lets you, and set Wren's tenant to
  `organizations`.
- Each other organisation decides whether to allow it. To them it is an app
  from another publisher. Most only let their administrator approve that,
  and `Chat.ReadWrite` and `Calendars.Read` usually need an administrator
  anyway. Their administrator can use
  `https://login.microsoftonline.com/{their tenant}/adminconsent?client_id={your client ID}`.
- Personal Microsoft accounts (Outlook.com, Teams free) cannot be added:
  Graph's Teams APIs serve work and school accounts only.

## With the Azure CLI

```sh
az ad app create --display-name Wren \
  --sign-in-audience AzureADMyOrg \
  --public-client-redirect-uris http://localhost \
  --is-fallback-public-client true
# Note the appId it prints, then add the delegated Graph permissions:
APP=<appId>
GRAPH=00000003-0000-0000-c000-000000000000
for scope in User.Read User.ReadBasic.All Chat.ReadWrite ChatMessage.Send \
             Team.ReadBasic.All Channel.ReadBasic.All ChannelMessage.Send \
             Presence.Read.All Calendars.Read Chat.Create Files.ReadWrite \
             Presence.ReadWrite OnlineMeetings.Read ChannelMessage.Read.All; do
  id=$(az ad sp show --id $GRAPH --query "oauth2PermissionScopes[?value=='$scope'].id" -o tsv)
  az ad app permission add --id $APP --api $GRAPH --api-permissions "$id=Scope"
done
# As an administrator, consent for the whole organisation:
az ad app permission admin-consent --id $APP
```

## Troubleshooting

| Message | Meaning |
|---|---|
| AADSTS50011 | The redirect URI is missing: add `http://localhost` under *Mobile and desktop applications*. |
| AADSTS700016 | The client ID is wrong, or the registration lives in another tenant. Check the tenant setting. |
| AADSTS50059, AADSTS50194 | A single-tenant registration used with `organizations`: set the tenant to your tenant ID or domain. |
| AADSTS65001, AADSTS90094 | Consent is missing; an administrator must grant it. |
| AADSTS7000218 | Device code sign-in needs *Allow public client flows*. |
| AADSTS50173, AADSTS70043 | The session expired (password change, revoked sessions). Sign in again. |
