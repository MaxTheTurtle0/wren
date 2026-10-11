# Calling and meetings

**Status: calls in a Teams web window, with Wren as the companion.** Wren
opens calls and meetings in Teams on the web, by default in a chrome-less
Edge or Chrome window placed beside Wren, and docks itself next to it with
the meeting chat and a call bar that follows your presence
([call-window.md](call-window.md)). The call itself runs in Microsoft's own
client, so it costs nothing extra. Wren has no audio or video of its own and
shows no incoming calls. This page explains why. Sources are in
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

1. **Implemented: Teams on the web beside Wren.** Calls and meetings open
   in a chrome-less Edge or Chrome window (`--app`), by default; or in the
   default browser; or, if chosen in Settings, the Teams app. Wren docks
   itself beside the call window, shows the meeting chat, and follows the
   call through your own presence (`InACall`, `InAConferenceCall`,
   `Presenting`, and `InAMeeting` for meetings Wren opened). Wren signs in
   through the same browser, so Teams on the web finds your Microsoft
   session. A QR code joins on your phone. Design and limits:
   [call-window.md](call-window.md).
2. **Owner decision:** in-call buttons (mute, camera, leave) in Wren through
   Teams desktop's third-party app API. Free and approved by the person, but
   its protocol is not published.
3. **Not pursued:** the ACS Calling SDK (it costs per minute), Teams web in
   an embedded webview (Teams refuses it, see
   [video-calling-options.md](video-calling-options.md)), an own WebRTC
   stack, and drawing call video in egui.
4. **Watch:** whether Microsoft documents delegated calling in Graph, a
   native macOS or Linux SDK covered by the Teams licence, or the
   third-party app API's protocol.

## Microphone, camera and screen sharing

The browser asks for them, for Teams on the web, as it does for any site.
Wren itself never touches those devices, so its macOS bundle declares no
`NSMicrophoneUsageDescription`, `NSCameraUsageDescription` or Screen
Recording permission.
