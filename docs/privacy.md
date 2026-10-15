# Privacy and security

- **No telemetry, no Wren servers.** Wren talks only to
  `login.microsoftonline.com` (sign-in) and `graph.microsoft.com` (your
  data). Additional connections: a file larger than 4 MiB is
  uploaded to the one-time upload URL Graph returns (your organisation's
  OneDrive host), without the Graph token; and an optional `wren-relay` that
  you run yourself ([realtime.md](realtime.md)); and GitHub for app update
  checks and downloads. Automatic app updates are checked at most daily and
  can be disabled in Settings. No Microsoft credentials or account data are
  sent to GitHub ([updates.md](updates.md)). Wren opens Microsoft Teams
  links in your browser when you ask it to.
- **Calls** run in Teams on the web, in your own Edge or Chrome (or the
  browser you choose). Wren starts the browser with the meeting or call link
  and nothing else, and opens its own Microsoft sign-in page there so Teams
  finds the same session. It never reads the browser's files, cookies or
  pages, and never passes it a token. Join links are not logged.
- **Your password never reaches Wren.** You sign in on Microsoft's page in
  your browser. Wren receives a one-time code on `localhost` and exchanges it
  with a PKCE verifier only it knows.
- **Tokens.** The refresh token is stored in the OS credential store
  (Keychain, Windows Credential Manager, Secret Service), never in files.
  Access tokens live only in memory. Tokens are only ever sent to the Graph
  origin: paging links and image URLs on other hosts are refused.
- **Least privilege.** Delegated permissions only, requested incrementally;
  see [permissions.md](permissions.md).
- **Local data.** The cache of your chats, channels and meetings is a SQLite
  file in a folder only your user can read (0700/0600 on macOS and Linux).
  It is not encrypted yet; full-disk encryption (FileVault, BitLocker, LUKS)
  protects it at rest. Encrypting it with a key held in the credential store
  is on the roadmap. Signing out deletes it.
- **Logs** never contain message text, tokens, authorization codes or URLs
  with query strings. Panics are logged without their payload
  (fastframe-log).
- **Files you send** are uploaded to your own OneDrive, in the
  `Microsoft Teams Chat Files` folder Teams uses too, and shared read-only
  with the people in that chat. In a channel, Wren creates a view link for
  your organisation, since a channel's members aren't known in advance.
  Remove the file or its sharing in OneDrive to revoke access.
- **Pasted images** stay in memory as PNGs until you send the message. They
  use Graph hosted content, so the image follows the conversation's access
  rules and needs no OneDrive sharing link or Files.ReadWrite permission.
  Clipboard contents are read only when you paste into a focused composer.
- **Presence.** While "Show me as available while I use Wren" is on (the
  default), Wren tells Microsoft you are active when you use it, at most
  every 4 minutes and only after input. Microsoft shows you as inactive
  5 minutes after the last update and Away 5 minutes later. Wren ends the
  session when you quit or sign out; otherwise it expires after an hour.
  Nothing is sent until you grant Presence.ReadWrite by setting your status
  once.
- **Call status** comes from your own presence: fetched when you return to
  Wren and, with the relay, announced by a presence notification that
  carries no presence data.
- **Several accounts** each have their own cache folder and refresh token.
  Signing out of one removes only its data.
- **Images in messages** are only fetched from Graph's own hosted content.
  Images hosted elsewhere are not loaded, so senders can't track when you
  read a message.
- **Links in calendar invitations** are only offered as "Join" when they
  point to a Teams host.

Report security issues privately to the maintainer rather than in a public
issue.
