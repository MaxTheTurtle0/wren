# Privacy and security

- **No telemetry, no Wren servers.** Wren talks only to
  `login.microsoftonline.com` (sign-in) and `graph.microsoft.com` (your
  data). It opens Microsoft Teams links in your browser or the Teams app when
  you ask it to.
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
- **Images in messages** are only fetched from Graph's own hosted content.
  Images hosted elsewhere are not loaded, so senders can't track when you
  read a message.
- **Links in calendar invitations** are only offered as "Join" when they
  point to a Teams host.

Report security issues privately to the maintainer rather than in a public
issue.
