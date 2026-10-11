# Wren agent guide

Wren is a small native client for Microsoft Teams: Rust, egui (glow), tokio,
Microsoft Graph. These rules are for coding agents and contributors. Follow
`CONTRIBUTING.md` for the checks.

## Product boundaries

- Keep Wren a small native client. No browser engine (no Electron, Tauri or
  embedded web views) for the interface, no telemetry, no Wren-operated
  backend that sees user data.
- **Official, documented Microsoft APIs only.** Microsoft Graph with
  delegated permissions, and Entra ID OAuth (authorization code + PKCE,
  device code as a fallback). Never call undocumented Teams endpoints
  (`*.teams.microsoft.com/api/...`, chatsvc, trouter), never reuse Microsoft's
  first-party client IDs, never present Wren as Microsoft's client.
- **Never poll Graph.** Microsoft's Teams API rules allow fetching when the
  person looks and at most one unattended refresh a day. Polling violates
  the Microsoft APIs Terms of Use. Every fetch goes through
  `sync::Freshness::claim` with a `Trigger` caused by the person. Do not add
  timers that fetch. Real-time work goes through Graph change notifications
  (`src/realtime.rs`, the `relay/` crate, `docs/realtime.md`), which trigger
  ordinary fetches with `Trigger::Push`. Subscriptions carry no resource
  data, so the relay never sees message content; keep it that way.
- **Least privilege.** Do not add a Graph scope to `auth::SIGN_IN_SCOPES`
  without a feature in the interface that needs it. Scopes needing admin
  consent are requested incrementally (`auth::OPTIONAL_SCOPES`). Update
  `docs/permissions.md` and `docs/azure-app-registration.md` in the same
  change.
- Don't advertise a capability the API lacks. `docs/features.md` and
  `docs/microsoft-api-research.md` are the source of truth. Before promising
  a Teams feature, check it there. If the API gained a capability, update
  those pages in the same change.
- Don't use Microsoft or Teams logos or names in a way that implies
  endorsement. Wren "works with Microsoft Teams".
- Don't broaden a task into adjacent features or a general refactor.

## Architecture

- `src/ui/` draws views and pushes `model::Action`s. `App::apply`
  (`src/app.rs`) runs them after the frame. Never mutate shared state from
  inside a view, beyond the view's own scratch fields (composer, filters).
- Network and disk work belongs to the backend runtime
  (`src/backend/worker.rs`), never the interface thread. The sides talk only
  through `backend::Command` and `backend::Event`. Each event wakes the window.
  Nothing should request continuous repaints.
- Graph JSON stays in `src/graph/types.rs`. Translate it into `model` types
  in `src/graph/convert.rs`. Views never see Graph shapes.
- `graph::Graph` sends the access token only to its own origin. Never build
  requests that bypass it, and never fetch message-provided URLs on other
  hosts with credentials. Don't fetch them at all for images, since that
  leaks read receipts.
- Platform integrations go behind `cfg(target_os = …)` or fastframe crates.
  A change for one platform must keep the other two compiling.
- Settings files stay readable and backward compatible (`#[serde(default)]`,
  unknown fields ignored) and are written atomically
  (`paths::write_atomically`). Secrets go to `secrets.rs`, never to files.
- The interface is optimistic: what the person did shows at once (marked
  *Sending* where it matters), and the backend makes it true. A failed send
  stays visible with Retry; it never silently disappears.
- egui and winit come from the crmne forks shared with Spotifast and ZapFast
  (`[patch.crates-io]` in `Cargo.toml`). Move all their crates to a new
  revision together, matching fastframe's README.
- Prefer existing dependencies. Explain any new crate in `Cargo.toml` next
  to it.

## Privacy

- Never log message contents, names, email addresses, tokens, authorization
  codes, or URLs with query strings at levels that ship. Use
  `AuthStatus::log_label` and the fixed error texts in `auth::token` and
  `graph`.
- Graph error messages can quote request data; keep only the error code.
- Tests use fixtures and mock servers (`wiremock`); they never touch a real
  tenant or the OS credential store (`secrets::MemoryStore`).

## Interface

- Dense, calm, readable. Use the palette in `theme.rs`, the shared widgets in
  `ui/widgets.rs`, and the type scale (11, 12, 13, 14, 15, 18, 20, 24, 30
  points). Avoid new sizes; each one grows the glyph atlas.
- New icons are 24 px Lucide outlines in white (`assets/icons/`), listed in
  the `icons!` table in `theme.rs`. Prefer fastframe's shared Lucide icons.
- Check changes in both themes and at narrow and wide sizes:
  `wren --screenshot out.png --page chats,light --size 1280x800`.

## Definition of done

- Focused tests for changed behaviour: unit tests next to the code, Graph
  behaviour against `wiremock`, interface behaviour in `tests/ui_smoke.rs`
  (headless egui frames over the demo).
- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets` pass. Don't weaken a lint or delete a test to get
  there without saying why.
- Update README and `docs/` when behaviour, permissions, files or network
  access change.
- Report platform coverage honestly: say whether something was run, or only
  compiled.
- Never use em dashes in public replies or release notes.

## Branches

Work on `main` in small commits, each compiling and passing the checks. Keep
history linear: squash pull requests, no merge commits.
