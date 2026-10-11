# Calls in a Teams web app window

**Status: built, not yet tried in a real call.** This is the free calling
path chosen in
[video-calling-options.md §0](video-calling-options.md#0-decision-calls-cost-nothing-extra).
Calls and meetings run in Microsoft Teams on the web, in a chrome-less Edge
or Chrome window that Wren opens and places. Wren is the companion: the
meeting chat, the call status, and its own window docked beside the call.
The questions only a real tenant and a real call can answer are in §7.

## Goals and limits

- No extra cost. Media runs in Teams on the web, which the Teams licence
  covers.
- No browser engine in Wren, no Teams endpoints, no tokens shared with
  Teams on the web, no polling.
- Works on macOS, Windows and Linux. On Linux it is the only way to call,
  since Microsoft no longer ships Teams desktop there.
- Not possible: call controls inside Wren (mute, camera). Microsoft offers
  no documented way to drive Teams on the web. Incoming calls also stay in
  Microsoft's clients.

## 1. What the person sees

1. They select **Join** on a meeting or **Call** in a chat.
2. A Teams window opens with no address bar or tabs, filling the left part of
   where Wren was. Wren shrinks to a 380-point panel on the right ("docked"),
   showing the call bar and the meeting chat.
3. Teams shows its own pre-join screen (camera, microphone), possibly after a
   "how do you want to join?" page (§7).
4. The call bar says `Opened in Chrome`, then `In the meeting · 12 min` once
   presence shows the person in the meeting. Its buttons:
   - **Show call** (macOS);
   - **Meeting chat**, when the chat isn't known yet;
   - **Join on phone** (a QR code);
   - **Restore** or **Dock**;
   - **Done**.
5. A minute after presence leaves the call, the bar says `Call ended` and
   Wren returns to its previous size and position. **Done** does the same at
   once.

Without docking (a small window, Wayland, or the setting off), the call bar
runs along the bottom of the main area.

**Settings → Calls and meetings:**

| Setting | Values | Default |
|---|---|---|
| Open in | Teams on the web, in its own window · Teams on the web, in my default browser · The Microsoft Teams app | Its own window. Without Edge or Chrome, the default browser |
| Browser | Microsoft Edge, Google Chrome (those installed) | Edge if present, else Chrome |
| Move Wren beside the call window during calls | on/off | On |
| A separate browser profile for each account | on/off (shown with two or more accounts) | Off |
| Signing out of Wren also signs the browser out of Microsoft | on/off | Off |
| **Sign in to Teams on the web** | Opens Teams on the web once, to sign in and choose "Stay signed in" | |

## 2. Architecture

```
ui (Join / Call / call bar)
  │ Action::JoinMeeting · StartCall · CallDone · ShowCall · DockWindow · FindMeetingChat · JoinOnPhone
  ▼
App (src/app/calls.rs) ── resolves the opener, docks, starts the browser, keeps the CallSession
  │ Command::FindMeetingChat · Command::SignInBrowser
  ▼
backend worker ── meeting-chat lookup, presence fetches, the presence subscription (relay)
  │ Event::MeetingChat · Event::MyPresence
  ▼
App ── CallSession state, ViewportCommands for docking
```

| File | Role |
|---|---|
| `src/calling/mod.rs` | Links and host checks, `CallsOpenIn` (the setting), `Opener` (how one call opens), join-link matching |
| `src/calling/browser.rs` | Finding Edge and Chrome; the exact argument lists; starting the browser |
| `src/calling/dock.rs` | Docking geometry |
| `src/calling/session.rs` | `CallSession`: the state machine behind the call bar |
| `src/app/calls.rs` | Opening calls, docking, presence, the meeting chat, sign-in and sign-out through the call browser |
| `src/ui/callbar.rs` | The call bar; `ui/mod.rs` draws the docked panel |
| `src/settings.rs` | `CallSettings` (`#[serde(default)]`, so older settings files load) |

### 2.1 Finding browsers

Discovery runs once, on first use (a few file checks). It uses no shell and,
on macOS and Windows, never searches PATH.

| Platform | Edge | Chrome |
|---|---|---|
| macOS | `Microsoft Edge.app/Contents/MacOS/Microsoft Edge` in `/Applications` or `~/Applications` | `Google Chrome.app/Contents/MacOS/Google Chrome`, same folders |
| Windows | `Microsoft\Edge\Application\msedge.exe` under `ProgramFiles(x86)`, `ProgramFiles`, `LOCALAPPDATA` | `Google\Chrome\Application\chrome.exe`, same folders |
| Linux | `microsoft-edge-stable`, `microsoft-edge` in absolute PATH entries; Flatpak `com.microsoft.Edge` exports | `google-chrome-stable`, `google-chrome`; Flatpak `com.google.Chrome` |

Only Edge and Chrome get app windows: Microsoft supports Teams on the web in
Edge, Chrome, Firefox and Safari, and only the first two have them. The
Teams app opener opens the https link with the system's handler, as before.

### 2.2 Launching

```
<browser> [--profile-directory=<profile>] [--window-position=X,Y --window-size=W,H] --app=<teams link>
```

- The link must pass `calling::is_teams_link` (https, a Teams host), so it
  can never be read as a flag.
- The process is started from an argument list, with no inherited input or
  output, in its own process group, and reaped on a short-lived thread.
- There is no `--user-data-dir`: the window opens in the person's normal
  browser profile, where their Microsoft sign-in lives.
- A running browser takes the request over from the new process. On macOS
  Wren runs the executable inside the bundle; `open -a` would drop the
  arguments for an app that is already running.
- Joining the same meeting again brings the call forward instead of opening
  the link twice, which could join a second time.

### 2.3 Docking

Wren reads its outer and inner rectangles from egui's `ViewportInfo`.
- **Dock:** if the frame is at least 960 by 400 points and not full screen,
  the call window gets the frame minus 388 points on the left. Wren lowers
  its minimum size, then moves itself into the remaining 380 points with
  `ViewportCommand::InnerSize` and `OuterPosition`. A maximized window is
  un-maximized first.
- **Restore:** Wren puts back its previous size, position, minimum size and
  maximized state.
- **Saved geometry:** the docked size is never saved as the window's
  geometry.
- **Units:** egui works in zoomed points and browsers in screen points, so
  Wren multiplies by the zoom factor.
- **Not docked:** on Wayland (no positions) and beside Teams desktop (another
  app's window).

## 3. Staying signed in with Wren

Wren never gives its tokens to Teams on the web. Both sign-ins share
Microsoft Entra's session cookie in the browser instead ([Microsoft: SSO
between apps](https://learn.microsoft.com/en-us/entra/identity-platform/msal-js-sso)).

- **Same browser for sign-in.** When calls open in an app window, every
  account's backend gets `Command::SignInBrowser`. Wren's sign-in page then
  opens in that browser, as a normal window with its extensions, instead of
  the default browser. If that fails, Wren falls back to the default
  browser.
- **Sign in to Teams on the web.** Wren's first sign-in usually includes
  consent, where Entra doesn't offer "Stay signed in?". That session lasts
  24 hours or until the browser closes. The Settings button opens
  `https://teams.microsoft.com/v2/` in the app window once. Teams signs in
  from the session, and the person can choose "Stay signed in" there.
- **Several accounts.** With "A separate browser profile for each account",
  each account gets a profile folder (`Wren <random>`), kept in settings by
  account ID. Its sign-ins and calls use `--profile-directory`, so each
  profile holds one Microsoft session.
- **Signing out.** With the setting on, signing out of Wren opens
  `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/logout` in the call
  browser. That signs it out of every Microsoft site.

## 4. The call session and the call bar

| From | Event | To | The bar shows |
|---|---|---|---|
| none | Join or Call | Opening | "Opened in Chrome" |
| Opening | Own presence activity is `InACall`, `InAConferenceCall`, `InAMeeting` or `Presenting` | Live | "In the meeting · N min" |
| Live | Activity leaves that set | Leaving | Unchanged |
| Leaving | Activity back within a minute | Live | Unchanged (flicker) |
| Leaving | A minute passes | Ended | "Call ended" for 4 s; the dock is restored |
| none | `InACall`, `InAConferenceCall` or `Presenting` with no session | Live (elsewhere) | "You're in a call" · Dismiss |

- **Calendar entries:** Microsoft sets `InAMeeting` from the calendar alone.
  It therefore counts only for meetings Wren opened, never for calls
  noticed elsewhere.
- **Leaving early:** if the person leaves a meeting before its scheduled
  end, the calendar may keep `InAMeeting` and the bar stays until **Done**.
- **Repaints:** deadlines are checked when the window repaints. Wren asks
  for a repaint when the minute counter or a deadline is due, never a fetch.

### Knowing a call is running

- Wren's own presence loads when the window gains focus (reused for 60 s).
  While docked, focus moves between Wren and the call often.
- **With the relay:** a subscription to `communications/presences/{my id}`
  (`updated`, no resource data, one hour, renewed with the chat
  subscription; Presence.Read.All, already granted). A notice triggers an
  ordinary fetch with `Trigger::Push`
  ([Microsoft: presence notifications](https://learn.microsoft.com/en-us/graph/changenotifications-for-presence)).
  If Graph refuses it, chat push still works.
- **Wren's own "available" presence session:** it never hides a call,
  because Graph ranks Busy above Available.

### The meeting chat beside the call

1. **From the cache:** Wren matches the join link against cached chats'
   join links. It compares decoded paths and ignores case and query
   strings.
2. **From Graph:** `GET /me/onlineMeetings?$filter=JoinWebUrl eq '{url}'`
   (the link URL-encoded once, as Microsoft's example shows) returns
   `chatInfo.threadId`; then `GET /chats/{id}`, cached. This needs
   OnlineMeetings.Read. Wren asks for it only when the person selects
   **Meeting chat**, never in the middle of joining. Microsoft documents that
   invited attendees can use it, not just organisers
   ([Microsoft: get onlineMeeting](https://learn.microsoft.com/en-us/graph/api/onlinemeeting-get?view=graph-rest-1.0)).
3. **Calls started from a chat** use that chat. From the Calls page, Wren
   uses the one-on-one chat with that person.

While a call runs in a chat, that chat's header hides its own Join and call
buttons: the call bar has them.

### Join on your phone

**Join on phone** shows a QR code of the join link (the `qrcodegen` crate,
drawn as egui rectangles, whole pixels per module) and a **Copy the link**
button.

## 5. Rules check

| Rule | How this keeps it |
|---|---|
| No browser engine in Wren | The call runs in a separate browser app |
| Documented APIs only | Teams deep links; Graph presence, presence subscriptions, onlineMeetings and chats; Chromium's documented command-line switches. No undocumented Teams URL parameters |
| Never impersonate Teams | Teams on the web signs in itself; Wren shares nothing with it |
| No polling | Presence on focus and through the relay; deadlines only repaint |
| Least privilege | One new optional scope (OnlineMeetings.Read), requested when the person asks for a meeting chat Wren doesn't have |
| Privacy | Join links carry meeting IDs, so they are never logged, and neither are command lines. The log records only which browsers were found. Wren reads no browser files or cookies |
| No extra cost | Teams on the web is covered by the Teams licence |

## 6. Tests

- **Unit:**
  - browser discovery on all three platforms with a fake file system;
  - the argument lists, refusing other links and unsafe profile names;
  - opener resolution;
  - docking geometry;
  - the session against presence sequences;
  - join-link matching;
  - App-level behaviour: the grace period restores the window; calls
    elsewhere show but calendar meetings don't; a meeting chat found later
    is attached; calls from the Calls page find the one-on-one chat;
    profiles stick to accounts.
- **`tests/features_flow.rs`:** the meeting-chat lookup through the real
  worker, matching the documented `$filter`.
- **`tests/push_flow.rs`:** both subscriptions through the real relay; a
  presence notification causes a presence fetch (the test fails if that path
  is removed).
- **`tests/ui_smoke.rs`:**
  - joining the demo meeting finds its chat;
  - joining again doesn't open a second call;
  - the bottom bar, the docked panel and the QR code draw;
  - Done restores;
  - the settings section applies.
- **macOS 27:** the docked panel, the bottom bar, the QR code and the Calls
  page screenshotted from the app bundle.

## 7. Open questions

Each needs a real browser, a real meeting or a tenant.

| # | Question | What Wren does meanwhile |
|---|---|---|
| 1 | Do `--window-position` and `--window-size` apply when the browser is already running? Which units does Windows use with display scaling? | Docks itself regardless; the call window may open where the browser chooses |
| 2 | What does a meeting link show in an app window, and is the choice remembered? | Nothing special; the person clicks "Continue on this browser" |
| 3 | After Wren signs in through the same browser, does Teams on the web sign in without asking? | Offers "Sign in to Teams on the web" |
| 4 | How quickly does presence change to InAMeeting or InACall and back? | A one-minute grace period; Done always works |
| 5 | Do Chrome and Edge create a missing `--profile-directory` profile? | The option is off by default |
| 6 | Can Wren raise the call window itself, not just the browser? | **Show call** activates the browser (macOS only) |

## 8. Decisions

- **Default:** calls open in Teams on the web in an app window, never the
  Teams app. The Teams app is an explicit choice in Settings.
- **Docking:** on by default, with **Restore** always one click away.
- **OnlineMeetings.Read:** requested only when the person asks for a meeting
  chat Wren doesn't have.
