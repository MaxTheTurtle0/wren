# Calls in a Teams web app window (design draft)

**Status: draft, not built.** The free calling path chosen in
[video-calling-options.md §0](video-calling-options.md#0-decision-calls-cost-nothing-extra).
Calls and meetings run in Microsoft Teams on the web, inside a chrome-less
Chrome or Edge window that Wren opens and places. Wren stays the companion:
meeting chat, call status, and its own window docked beside the call. Items
marked **SPIKE** are unverified and get tested before the code depends on
them.

## Goals and limits

- No extra cost. Media runs in Teams on the web, which the Teams licence
  covers.
- No browser engine in Wren, no Teams endpoints, no tokens shared with
  Teams on the web, no polling.
- Works on macOS, Windows and Linux. On Linux it is the only way to call,
  since Microsoft no longer ships Teams desktop there.
- Not in scope: call controls inside Wren (mute, camera). Microsoft offers
  no documented way to drive Teams on the web. Incoming calls also stay in
  Microsoft's clients.

## 1. What the person sees

1. They select **Join** on a meeting or **Call** in a chat, as today.
2. A Teams window opens with no address bar or tabs, filling the left part of
   where Wren was. Wren shrinks to a 380-point panel on the right ("docked").
3. Teams shows its own pre-join screen (camera, microphone). The first
   time, it may ask how to join: the person picks "Continue on this
   browser" (**SPIKE 2**).
4. Wren's panel shows a call bar above the meeting's chat:
   `Opening in Chrome…`, then `In "Design review" · 12:04` once Teams
   reports the person in the meeting.
5. When they leave, the bar shows `Call ended` and Wren returns to its
   previous size and position.

**Settings → Calls** (new section):

| Setting | Values | Default |
|---|---|---|
| Open calls and meetings in | Automatic · Teams app · Teams on the web (app window) · Default browser | Automatic: the Teams app if installed, else the web app window, else the default browser |
| Browser for Teams on the web | The supported ones found: Microsoft Edge, Google Chrome | Edge if present, else Chrome |
| Dock Wren beside calls | on/off | On (web app windows only) |
| Prepare Teams on the web | Button: opens Teams on the web once, to sign in and choose "Stay signed in" | |
| Separate browser profile per account | on/off (shown with two or more accounts) | Off |

## 2. Architecture

```
ui (Join / Call / call bar)
  │ Action::JoinMeeting(url) · Action::StartCall{..} · Action::ShowCall · Action::EndCall
  ▼
App::apply ── resolves the opener, plans the dock, starts a CallSession
  │ Command::OpenCall{opener, url, placement}
  ▼
backend worker ── spawns the browser (detached), fetches presence, finds the meeting chat
  │ Event::CallOpened · Event::CallFailed · Event::MyPresence · Event::MeetingChat
  ▼
App ── CallSession state machine, ViewportCommands for docking
```

New code, in the existing style:

| File | Role |
|---|---|
| `src/calling/mod.rs` | Today's `calling.rs` (links, host checks), plus `CallOpener` and its resolution |
| `src/calling/browser.rs` | Finding Chrome, Edge and Teams desktop; building command lines (pure functions, unit-tested) |
| `src/calling/session.rs` | `CallSession`: the state machine behind the call bar |
| `src/calling/dock.rs` | Docking geometry: the call window's rectangle and Wren's, and restoring them |
| `src/ui/callbar.rs` | The call bar |
| `src/settings.rs` | `calls: CallSettings` (`#[serde(default)]`, so old settings files load) |

Spawning a process is I/O, so it runs in the backend, like opening links
today. The interface never waits for it.

### 2.1 Choosing where a call opens

```rust
pub enum CallOpener {
    TeamsApp,              // today's hand-off (open the https deep link)
    WebApp(Browser),       // chrome-less window in Chrome or Edge
    DefaultBrowser,        // fallback: Safari, Firefox, anything
}
pub struct Browser { pub kind: BrowserKind /* Edge, Chrome */, pub program: PathBuf, pub args_prefix: Vec<String> /* flatpak run … */ }
```

`resolve(setting, found) -> CallOpener`. Automatic picks the Teams app when
found (its native media engine is the most efficient), otherwise the web app
window, otherwise the default browser. On Linux it never picks the Teams
app. Only Edge and Chrome get app windows: Microsoft lists Edge, Chrome,
Firefox and Safari as supported, and only the first two have app windows.
Other Chromium browsers can be offered later as "not supported by
Microsoft".

### 2.2 Finding browsers

Runs once in the backend at start and again when Settings opens. No shell,
no guessing from PATH on macOS or Windows.

| Platform | Edge | Chrome | Teams desktop |
|---|---|---|---|
| macOS | `Microsoft Edge.app` in `/Applications` or `~/Applications`, checking `CFBundleIdentifier` `com.microsoft.edgemac`; run `Contents/MacOS/Microsoft Edge` | `Google Chrome.app`, `com.google.Chrome`; `Contents/MacOS/Google Chrome` | `Microsoft Teams.app` (`com.microsoft.teams2`) |
| Windows | Registry `App Paths\msedge.exe` (HKCU, then HKLM) | `App Paths\chrome.exe` | The `msteams` URL protocol is registered |
| Linux | `microsoft-edge-stable`, `microsoft-edge` on PATH; Flatpak `com.microsoft.Edge` | `google-chrome-stable`, `google-chrome`; Flatpak `com.google.Chrome` | Never |

On Windows the registry is read through `windows-sys`, which is already in
the dependency tree (it needs its registry feature enabled).

### 2.3 Launching

The program is started with an argument list; there is no shell and no
string splicing:

```
<browser> --app=<teams link> [--window-position=X,Y --window-size=W,H] [--profile-directory=<profile>]
```

- The link must pass `calling::is_teams_link` (https, a Teams host), so it
  can never be read as a flag.
- No `--user-data-dir`: the window opens in the person's normal browser
  profile, which is what carries the Microsoft sign-in (§3).
- If the browser is already running, the new process hands the request to it
  and exits. On macOS, Wren runs the executable inside the bundle instead of
  `open -a`, because `open` drops arguments for an app that is already
  running.
- Wren never opens the same join link twice for one session. A second open
  could join a second time.

### 2.4 Docking

Wren reads its own outer rectangle and monitor size from egui's
`ViewportInfo`.
- **Dock:** if the frame is at least 960 points wide, the call window gets
  the frame minus 380 points on the left, passed as
  `--window-position`/`--window-size`. Wren then moves itself to the
  remaining strip with `ViewportCommand::OuterPosition` and `InnerSize`.
- **Restore:** Wren remembers its previous geometry and restores it when the
  session ends, or when the person selects **Restore**.

Limits:

- Teams desktop's window belongs to another app. Placing it would need
  Accessibility permission on macOS, so Wren does not dock beside Teams
  desktop.
- Wayland lets no app choose its own window position, so on Wayland Wren
  only narrows itself and leaves placement to the desktop.
- Whether Chrome and Edge apply the position and size when they are already
  running, and in which units on Windows with scaling, is **SPIKE 1**. If
  they don't, Wren still docks itself and the call window opens wherever the
  browser puts it.

## 3. Staying signed in with Wren

Wren never gives its tokens to Teams on the web. Both sign-ins share
Microsoft Entra's session cookie in the browser instead ([Microsoft: SSO
between apps](https://learn.microsoft.com/en-us/entra/identity-platform/msal-js-sso)).

- **Same browser for sign-in.** `browser_sign_in` opens Wren's sign-in
  page with the call browser when the opener is `WebApp`, instead of
  `open::that` (the default browser). It stays a normal tab, so password
  managers and Microsoft's single sign-on extensions work as usual.
- **Prepare Teams on the web.** Wren's first sign-in usually includes
  consent, where Entra doesn't offer "Stay signed in?". That session lasts
  24 hours or until the browser closes. The Settings button opens
  `https://teams.microsoft.com/v2/` in the app window once. Teams signs in
  from the session, and the person can choose "Stay signed in" there.
  Whether Teams then signs in silently on later calls is **SPIKE 3**, which
  needs a tenant.
- **Several accounts.** With "Separate browser profile per account", each
  account's sign-in and calls use `--profile-directory=Wren <label>`. Each
  profile then holds one Microsoft session, and Teams never asks which
  account to use. **SPIKE 5**: whether Chrome and Edge create the profile
  on first use.
- **Signing out.** Optionally, "Also sign out of Microsoft in Edge" opens
  `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/logout` in that
  browser. That signs it out of every Microsoft site, so it is off by
  default.

## 4. The call session and the call bar

```rust
pub struct CallSession {
    pub target: CallTarget,          // Meeting { subject, join_url } | People { names, video } | Elsewhere
    pub opener: CallOpener,
    pub opened_at: Millis,
    pub state: CallState,            // Opening | Live { since } | Ended { at }
    pub chat: Option<ChatId>,        // shown under the bar
    pub restore: Option<WindowGeometry>,
}
```

| From | Event | To | The bar shows |
|---|---|---|---|
| none | Join or Call selected | Opening | "Opening in Edge…" · Show call · Done |
| Opening | Own presence activity is `InACall`, `InAConferenceCall`, `InAMeeting` or `Presenting` | Live | Subject and timer · Show call · Join on phone · Done |
| Live | Activity leaves that set and stays out for 60 s | Ended | "Call ended", then the bar closes and the dock is restored |
| Opening | 10 minutes without Live | Opening | Unchanged; the person ends it with Done |
| none | Live activity seen with no session | Live (`Elsewhere`) | "You're in a call" · Open Teams |

The 60-second grace stops a short presence flicker from ending the session.
It is a deadline checked when events arrive, not a timer that fetches.

- **Show call** brings the browser forward. On macOS that means activating
  the app; on Windows, focusing another process's window is restricted.
  Raising the exact window is **SPIKE 6**. It never re-opens the link.
- **Done** ends the session and restores the dock.

### Knowing a call is running

- Wren's own presence already loads on focus (`Resource::MyPresence`, reused
  for 60 s). While docked, focus moves between Wren and the call often, so
  that alone keeps the bar roughly current.
- **With the relay:** subscribe to `communications/presences/{my id}`
  (`updated`, no resource data, at most one hour, renewed like the chat
  subscription; Presence.Read.All, already granted). A notice triggers an
  ordinary fetch with `Trigger::Push`
  ([Microsoft: presence notifications](https://learn.microsoft.com/en-us/graph/changenotifications-for-presence)).
- Wren's own "available while I use Wren" presence session never hides a
  call: Graph ranks Busy above Available.
- How quickly the activity changes when joining and leaving is **SPIKE 4**,
  which needs a tenant.

### The meeting chat beside the call

1. **From the cache:** match `Meeting.join_url` against the cached chats'
   `join_url`, comparing decoded links without query strings. No request is
   needed.
2. **From Graph:** `GET /me/onlineMeetings?$filter=JoinWebUrl eq '{url}'`
   returns `chatInfo.threadId`; then `GET /chats/{id}` and add it to the
   cache. This needs OnlineMeetings.Read, requested on first use like the
   other optional scopes. Microsoft documents that invited attendees can
   use it, not just organisers
   ([Microsoft: get onlineMeeting](https://learn.microsoft.com/en-us/graph/api/onlinemeeting-get?view=graph-rest-1.0)).
3. **Calls started from a chat** use that chat.

### Join on your phone

A QR code of the join link, in the meeting details and the call bar. Wren
draws it with `qrcodegen`, a small crate with no dependencies, as egui
rectangles; no image is needed.

## 5. Rules check

| Rule | How this design keeps it |
|---|---|
| No browser engine in Wren | The call runs in a separate browser app |
| Documented APIs only | Teams deep links; Graph presence, presence subscriptions, onlineMeetings and chats; Chromium's command-line switches. No undocumented Teams URL parameters, so the join page may cost a click |
| Never impersonate Teams | Teams on the web signs in itself; Wren shares nothing with it |
| No polling | Presence on focus and through the relay; the grace period is a deadline, not a timer |
| Least privilege | One new optional scope (OnlineMeetings.Read), requested on first use, only for meetings whose chat isn't cached |
| Privacy | Join links carry meeting IDs, so they are never logged, and neither are browser command lines. Wren reads no browser files or cookies |
| No extra cost | Teams on the web is covered by the Teams licence |

## 6. Tests

- **Unit:**
  - browser discovery over a fake file system and registry (a `Probe`
    trait);
  - the exact argument lists per platform;
  - opener resolution for every setting and discovery result;
  - docking geometry (wide, narrow and multi-monitor frames, Wayland);
  - the session state machine fed presence sequences (flicker, calls
    elsewhere, the timeout);
  - join-link normalisation and matching;
  - QR encoding.
- **Against the mock Graph (`wiremock`):** the onlineMeetings lookup and chat
  fetch; the presence subscription payload and renewal, in
  `tests/push_flow.rs` style.
- **Headless UI:** every call bar state, the Calls settings section, the
  docked 380-point layout.
- **On the Mac mini:** the spikes below. They need Chrome or Edge installed
  there; only Teams desktop is today.

## 7. Spikes before building

| # | Question | Needs |
|---|---|---|
| 1 | Do `--window-position` and `--window-size` apply when the browser is already running? Units on Windows with scaling? | Chrome or Edge; no tenant |
| 2 | What does a meeting link show in an app window ("How do you want to join?"), and is the choice remembered? | A real meeting link |
| 3 | After Wren signs in through the same browser, does Teams on the web sign in without asking? After "Prepare", on later calls? | A tenant |
| 4 | How quickly does your own presence change to InAMeeting or InACall and back? | A tenant |
| 5 | Does `--profile-directory` create a profile on first use? | Chrome or Edge |
| 6 | Can Wren raise the call window itself, not just the browser? | Chrome or Edge |

## 8. Order of work

Each step is its own commit with tests.

1. **Where calls open:** discovery, launching, the setting, the Linux
   default. Small.
2. **Signed in with Wren:** sign-in through the call browser, "Prepare
   Teams on the web", optional browser sign-out. Small.
3. **Call bar and meeting chat:** the session state machine, the cached
   chat match, Done. Medium.
4. **Docking:** geometry, restore, Wayland and narrow-window fallbacks.
   Medium.
5. **Live status:** the presence subscription through the relay, the
   onlineMeetings lookup, the QR code. Medium.
6. **Several accounts:** browser profiles per account. Small.

## 9. Owner decisions

- **Automatic** prefers the Teams app where installed. Should it prefer
  the web app window instead, so calls always dock beside Wren?
- Is docking on by default acceptable, given that Wren moves its own window?
- Is one more optional permission (OnlineMeetings.Read) acceptable, to find
  meeting chats that aren't cached yet?
