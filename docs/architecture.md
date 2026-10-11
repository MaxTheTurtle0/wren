# Architecture

Wren follows the architecture of [Spotifast](https://github.com/crmne/spotifast)
and [ZapFast](https://github.com/crmne/zapfast). It is one native process with
a window drawn by egui, a background runtime that does all I/O, and
one-directional data flow between them.

```
                         ┌─────────────────────────── interface thread ───────────────────────────┐
  input ──► egui frame ──► ui/* views ──(read)── App state ◄──(apply)── Action queue ◄──(push)── ui/*
                         │                         ▲                                                │
                         │                   Event │ (drained each frame)            Command │      │
                         └─────────────────────────┼─────────────────────────────────────────┼──────┘
                                                   │                                         ▼
                         ┌──────────────────────── backend thread (tokio) ─────────────────────────┐
                         │  worker loop ──► tasks: auth (PKCE, refresh) · Graph client · SQLite    │
                         │                         cache · sync policy · media downloads          │
                         └──────────────────────────────────────────────────────────────────────────┘
```

## Rules

1. **Views draw and emit actions.** `src/ui/` reads `App` and pushes
   `model::Action`s. `App::apply` runs them after the frame. A view may only
   change its own scratch state (the composer text, a filter), never shared
   state it is drawing from.
2. **The interface thread never waits.** Every network and disk operation
   runs on the backend's tokio runtime (`src/backend/worker.rs`). The two
   sides exchange `backend::Command` and `backend::Event` over channels. Each
   event wakes the window through `fastframe_shell::Waker`. Nothing repaints
   without input or an event, so an idle Wren draws no frames.
3. **Optimistic and cached.** A sent message, a reaction or an edit appears
   at once, marked *Sending* where it matters. Graph's answer reconciles it.
   Every list opens from the SQLite cache before the network answers.
4. **No polling.** `src/sync.rs` is the only place that decides whether to
   fetch, and every fetch answers something the person did (see
   [realtime.md](realtime.md)).
5. **Platform code stays behind `cfg`.** Shared code compiles on macOS,
   Windows and Linux. Platform pieces come from fastframe crates or sit
   in `#[cfg(target_os = …)]` blocks.

## Modules

| Module | Role |
|---|---|
| `main.rs` | CLI, logging, single instance, tray, the eframe window and the window shell (`fastframe-shell`), screenshot mode |
| `app.rs` | `App` state, event handling, the action reducer, optimistic sends, message merging |
| `model.rs` | Wren's own types (`Chat`, `Message`, `Team`, `Channel`, `Meeting`, `Presence`, `Action`, …) |
| `ui/` | Views: rail, chat list, teams tree, conversation (virtualized history, composer, threads), calendar, calls, welcome/sign-in, overlays (⌘K switcher, settings, shortcuts), widgets, rich text |
| `html.rs` | Teams HTML → `RichText` (a tolerant tokenizer, no browser engine) and composer markdown → HTML |
| `backend.rs`, `backend/worker.rs` | The runtime thread, `Command`/`Event`, and the worker's tasks |
| `auth/` | Entra ID: PKCE authorize URL, loopback listener, token endpoint (code, refresh, device code), `Session` with refresh-token rotation |
| `secrets.rs` | Refresh tokens in the OS credential store (`keyring-core` with each platform's native store) |
| `graph/` | Graph client (token only to the Graph origin, 401 refresh, `Retry-After`), endpoints (`api.rs`), JSON shapes (`types.rs`), translation (`convert.rs`) |
| `store.rs` | Per-account SQLite cache with FTS5 search |
| `sync.rs` | Fetch policy (no polling; dedupe repeated requests) |
| `calling.rs` | Deep-link hand-off and the capability model for future in-app calling |
| `theme.rs` | Palettes (fastframe-theme compatible), visuals, type scale, Lucide icon table |
| `platform.rs` | App icon rasterization, tray item, single-instance slot |
| `notify.rs` | Desktop notifications |
| `demo.rs` | Offline sample data |

## Data on disk

| What | Where |
|---|---|
| Settings (no secrets) | config folder, `settings.json` |
| Cache: chats, messages, teams, channels, meetings, photos | data folder, `accounts/<account>/wren.db` (0600) |
| Account record (name, granted scopes; no tokens) | `accounts/<account>/account.json` |
| Refresh token | OS credential store, service `app.wren.Wren` |
| Logs (no message contents, tokens or URLs with queries) | state folder, `wren.log`, `panic.log` |

Signing out deletes the refresh token and the account's folder.

## Rendering and performance

- **glow** (OpenGL) renderer, the same as Spotifast and ZapFast. It keeps the
  binary small (15 MB release on macOS arm64).
- The egui and winit forks shared by the crmne apps carry fixes that matter
  for an idle client, most importantly egui's busy-loop fix.
- **Virtualized lists.** The chat list uses fixed-height rows
  (`ScrollArea::show_rows`). The message history keeps a height per message,
  lays out only rows within two viewports of the screen, and estimates the
  rest.
- **One galley per paragraph.** Message text is laid out once per frame as
  selectable labels. Links are found from the galley's character positions,
  with no per-word widgets.
- **Lazy media.** Avatars and inline images are fetched only when drawn on
  screen, a few at a time. Photos are cached for a week.
