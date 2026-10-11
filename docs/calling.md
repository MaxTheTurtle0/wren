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

Calls must cost nothing beyond the Microsoft 365 licence people already have.
Microsoft's licence covers calls only in its own clients (Teams web, desktop
and mobile); a third-party client carrying call media is billed through ACS
per participant-minute. So the call itself stays in a Microsoft client, and
Wren integrates around it. The comparison and sources are in
[video-calling-options.md](video-calling-options.md).

1. **Now (implemented):** hand off through deep links to Teams desktop, or
   Teams on the web.
2. **Planned: choose where calls open.** Teams desktop, or Teams on the web
   in a chrome-less Chrome or Edge app window (`--app`), or the default
   browser. Teams on the web is the default on Linux, where Microsoft no
   longer ships a desktop client.
3. **Planned: Wren as the call companion.** Opening a call or meeting opens
   its chat in Wren, docks Wren's window beside the call, and shows a slim
   call bar with "Back to the call". Your own presence (`InACall`,
   `InAMeeting`, `Presenting`) tells Wren a call is running, also for calls
   started elsewhere. A QR code joins on your phone.
4. **Owner decision:** in-call buttons (mute, camera, leave) in Wren through
   Teams desktop's third-party app API. Free and approved by the person, but
   its protocol is not published.
5. **Not pursued:** the ACS Calling SDK (it costs per minute), Teams web in
   an embedded webview, an own WebRTC stack, and drawing call video in egui.
6. **Watch:** whether Microsoft documents delegated calling in Graph, a
   native macOS or Linux SDK covered by the Teams licence, or the
   third-party app API's protocol. `calling::CallBackend` is the seam where
   an in-app backend plugs in without changing the interface.

## Microphone, camera and screen sharing

These only become relevant if Wren ever carries call media itself, which no free path allows today. On
macOS, the bundle would then need `NSMicrophoneUsageDescription`,
`NSCameraUsageDescription` and Screen Recording permission. The current
bundle declares none, because Wren 0.1 never touches those devices.
