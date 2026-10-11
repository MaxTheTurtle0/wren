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

Calls cost nothing extra, so they stay in Microsoft's own clients and Wren
integrates around them. The comparison is in
[video-calling-options.md](video-calling-options.md).

1. A setting for where calls open: Teams desktop, Teams on the web in a
   Chrome or Edge app window (default on Linux), or the browser.
2. Call companion: the meeting's chat opens in Wren, Wren docks beside the
   call window, and a call bar offers "Back to the call".
3. Call awareness from your own presence (on focus, and pushed through the
   relay), and a QR code to join on your phone.
