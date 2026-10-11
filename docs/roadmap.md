# Roadmap

Roughly in order. Each step lands as its own focused change with tests.

## Next

1. **First live-tenant run.** Register the app, sign in, and walk the
   checklist: chat list, history paging, send, quote reply, reactions,
   edit/delete, mark read, teams/channels, channel consent flow, calendar,
   presence, photos, inline images, search, file upload and sharing, new
   chats, mentions, two accounts, availability, status message (including
   clearing it) and the presence session. Fix what
   the real data shows (expect edge cases in Teams HTML and system events).
2. **Windows and Linux visual pass.** Run on both, capture screenshots, fix
   platform chrome (title bar, fonts, tray).
3. **Push relay, live.** Run `wren-relay` against a real tenant, deploy
   guide with a container image, channel-post subscriptions.
4. **Paste images** into the composer as hosted content, and upload
   progress for large files.

## Then

- Encrypt the cache (SQLCipher with a key in the OS credential store).
- Adaptive Cards (a native renderer for the common card elements).
- Lazy per-script fallback fonts (upstream in fastframe-fonts), about 29 MB
  less memory.
- Unread divider and "jump to latest", pinned chats, chat muting.
- Translations (fastframe-i18n).
- Signed and notarized macOS builds, a Windows installer, Flatpak and AUR.

## Calling

The plan, with measurements and acceptance criteria, is in
[video-calling-options.md](video-calling-options.md).

0. Token exchange in `wren-relay`, an ACS resource, and a licence review of
   the Calling SDKs.
1. macOS: a call window from the ACS Web SDK, served by Wren on loopback and
   opened as a Chrome or Edge app window, with call controls in Wren. Meeting
   join and 1:1 calls.
2. Picture-in-picture, an opt-in resident agent for incoming calls, screen
   sharing, Linux.
3. Windows: a native C# WinUI 3 helper on the ACS Windows SDK, measured
   against the web path.
