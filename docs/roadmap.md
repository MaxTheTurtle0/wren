# Roadmap

Roughly in order. Each step lands as its own focused change with tests.

## Next

1. **First live-tenant run.** Register the app, sign in, and walk the
   checklist: chat list, history paging, send, quote reply, reactions,
   edit/delete, mark read, teams/channels, channel consent flow, calendar,
   presence, photos, inline images, search. Fix what the real data shows
   (expect edge cases in Teams HTML and system events).
2. **Windows and Linux visual pass.** Run on both, capture screenshots, fix
   platform chrome (title bar, fonts, tray).
3. **Push relay, live.** Run `wren-relay` against a real tenant, deploy
   guide with a container image, channel-post subscriptions.
4. **Send files.** OneDrive upload (Files.ReadWrite), sharing link, and a
   `reference` attachment. Drag and drop, and paste of images (hosted
   content).
5. **New chats** (Chat.Create) and **@mentions** in the composer.

## Then

- Multiple accounts. Storage and credentials are already per account; this
  needs an account switcher and one backend per account.
- Encrypt the cache (SQLCipher with a key in the OS credential store).
- Set your own presence and status message (Presence.ReadWrite and a
  presence session).
- Adaptive Cards (a native renderer for the common card elements).
- Lazy per-script fallback fonts (upstream in fastframe-fonts), about 29 MB
  less memory.
- Unread divider and "jump to latest", pinned chats, chat muting.
- Translations (fastframe-i18n).
- Signed and notarized macOS builds, a Windows installer, Flatpak and AUR.

## Calling

- Companion ACS calling page (web SDK in the system browser) with a
  self-hostable token exchange, as an opt-in experiment for organisations
  with an Azure subscription. See [calling.md](calling.md) for the limits.
- Research: ACS Windows SDK through WinRT from Rust.
