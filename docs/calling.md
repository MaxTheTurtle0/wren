# Calling and meetings

**Status: hand-off.** Wren 0.1 starts calls and joins meetings by opening
Microsoft's documented deep links in Microsoft Teams: the desktop app if it
is installed, otherwise Teams on the web. It shows no incoming calls and has
no audio or video of its own. This page explains why, and what it would take
to do more. Sources are in
[microsoft-api-research.md §7](microsoft-api-research.md#7-calling).

## What Microsoft offers

### Microsoft Graph cloud communications: not usable

`POST /communications/calls` lists delegated permissions as *Not
supported*. Every `Calls.*` permission is application-only with admin
consent, and the API exists for bots that answer or place calls as
themselves. App-hosted media bots must be written in C# and run on Windows
Server in Azure. None of it lets a desktop app call **as the signed-in
person**.

### Azure Communication Services (ACS) as a Teams user: possible, with costs

ACS can act as a Teams user (the "Custom Teams Endpoint").

1. The person signs in to an Entra app with the
   `https://auth.msft.communication.azure.com/Teams.ManageCalls` (and
   `Teams.ManageChats`) scopes.
2. A service exchanges that token for an ACS token with
   `getTokenForTeamsUser`, against an **ACS resource in Azure**. Microsoft
   recommends doing this in a backend, because the exchange is authenticated
   with the resource's key or Entra RBAC.
3. The ACS Calling SDK then places and receives VoIP and PSTN calls, joins
   Teams meetings, and runs group calls, as that Teams user.

What it cannot do:

- Join an existing Teams ad-hoc group call after it has started.
- Run natively on macOS or Linux. The Calling SDK exists for the web
  (JavaScript), Windows, iOS and Android. It is closed source, with no C or
  C++ build. The iOS framework has no macOS or Mac Catalyst slice.
- Run on Windows without packaging, or on Windows on Arm natively. Windows SDK
  2.0.0 ships only x64 binaries for `net8.0-windows` (WinUI 3, MSIX), with no
  `.winmd` metadata for non-.NET callers.
- Run in an embedded webview. Microsoft lists WebView2, CEF and WebKitGTK as
  unsupported. Electron is supported on macOS and Windows; Chrome, Edge and
  Safari on macOS; Chrome only on Linux.
- Run for free. Teams users calling through ACS cost $0.004 per
  participant-minute, billed to the ACS resource's Azure subscription.
  Starting calls with Teams users may also need a Teams Phone licence
  (unverified).
- Ship without a licence review. The npm Web SDK 2.0.0 allows redistribution
  with conditions; the Windows 2.0.0 and iOS 3.0.0 licence files say
  "pre-release" and grant no right to distribute.
- Last forever without change. On 2026-09-23 Microsoft announced the
  retirement of ACS as a standalone offering, effective 2028-09-30. The
  Calling SDK continues "in conjunction with a Microsoft Teams-aligned
  product", with new major versions.

### Deep links: what Wren does today

| Action | Link |
|---|---|
| Call one or more people | `https://teams.microsoft.com/l/call/0/0?users=<upn>,<upn>[&withVideo=true]` (Teams asks to confirm) |
| Join a meeting | The meeting's `joinUrl` from the calendar or the meeting chat |
| Open a chat or channel in Teams | Its `webUrl` |

Wren only opens join links on Teams hosts (`teams.microsoft.com`,
`teams.live.com`, `teams.cloud.microsoft` and the US government clouds),
because calendar invitations can carry anyone's links.

## What is not possible

- A native WebRTC stack talking to Teams directly. The signalling is
  undocumented, and the API terms forbid working around technical
  limitations.
- Incoming-call alerts without ACS. No delegated Graph API reports ringing
  calls. `callRecords` is application-only and appears after the call ends.

## Paths forward

The full comparison, with performance techniques, a prototype plan and
sources, is in [video-calling-options.md](video-calling-options.md).

1. **Now (implemented):** hand off through deep links. This works on every
   platform with no Azure resources, and the person keeps Teams' calling
   quality and features.
2. **macOS and Linux (planned): a call window driven by Wren.** Wren serves a
   small page with the ACS Web SDK from a fixed loopback origin and opens it
   as a chrome-less app window in Chrome or Edge (Safari as the fallback).
   Call controls, the roster and incoming-call prompts stay in Wren's egui
   interface and drive the page over a loopback WebSocket. The browser
   decodes and composites video with hardware H.264, and Chrome and Edge send
   simulcast, so Wren itself stays idle during calls. It needs an ACS
   resource, a token exchange (planned in `wren-relay`) and consent for the
   ACS scopes.
3. **Windows (research): a native helper.** A small C# WinUI 3 app on the ACS
   Windows SDK, speaking the same protocol, with video rendered by Media
   Foundation. Calling the SDK directly from Rust is not practical: 2.0.0 has
   no WinRT metadata and needs package identity.
4. **Not planned:** drawing remote video inside egui (it would end Wren's
   idle-at-zero during calls), the iOS SDK on the Mac, and any own WebRTC
   stack talking to Teams.
5. **Watch:** whether Microsoft documents delegated calling in Graph, or a
   native macOS or Linux SDK. `calling::CallBackend` is the seam where an
   in-app backend plugs in without changing the interface.

## Microphone, camera and screen sharing

These only become relevant once Wren hosts calls (path 2 or 3). On
macOS, the bundle would then need `NSMicrophoneUsageDescription`,
`NSCameraUsageDescription` and Screen Recording permission. The current
bundle declares none, because Wren 0.1 never touches those devices.
