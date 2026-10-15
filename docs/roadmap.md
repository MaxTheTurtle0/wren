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
4. **Upload progress** for large file attachments.

## Then

- Encrypt the cache (SQLCipher with a key in the OS credential store).
- Adaptive Cards (a native renderer for the common card elements).
- Lazy per-script fallback fonts (upstream in fastframe-fonts), about 29 MB
  less memory.
- Unread divider and "jump to latest", pinned chats, chat muting.
- Translations (fastframe-i18n).
- Signed and notarized macOS builds, a Windows installer, Flatpak and AUR.

## Calling

Calls run in Teams on the web beside Wren ([call-window.md](call-window.md)).
Next:

1. Answer the open questions in [call-window.md §7](call-window.md#7-open-questions)
   with a real browser, meeting and tenant: window placement with a running
   browser, the join page, silent sign-in, presence timing.
2. Decide on Teams desktop's third-party app API for in-call buttons (its
   protocol is unpublished).
