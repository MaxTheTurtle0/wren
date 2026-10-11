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

- Join an existing Teams 1:1 or group call.
- Run natively on macOS or Linux. The Calling SDK exists for the web
  (JavaScript), Windows (UWP or WinUI 3, MSIX-packaged), iOS and Android. It
  is closed source, with no C or C++ build.
- Run for free. Minutes are billed to the ACS resource's Azure subscription.
- Last forever without change. On 2026-09-23 Microsoft announced the
  retirement of ACS as a standalone offering, effective 2028-09-30. Teams
  interop is listed as not impacted, but Microsoft does not commit to new
  features.

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

1. **Now (implemented):** hand off through deep links. This works on every
   platform with no Azure resources, and the person keeps Teams' calling
   quality and features.
2. **Companion web page (experimental, not built):** an ACS Teams-user calling
   page served from a self-hosted token-exchange service and opened in the
   system browser. The Web SDK supports Chrome, Edge and Safari on macOS and
   Windows, and Chrome on Linux. The page would bring in-call controls
   (microphone, camera, screen sharing, participants) close to Wren without
   embedding a browser engine. It needs an ACS resource, a backend, and
   admin consent for the ACS scopes.
3. **Windows-only native calling (research):** call the ACS Windows SDK
   (WinRT) from Rust through the `windows` crate, with MSIX packaging.
   Plausible, but undocumented for non-.NET callers.
4. **Watch:** whether Microsoft documents delegated calling in Graph, or a
   native macOS or Linux SDK. If either appears, `calling::CallBackend` is
   the seam where an in-app backend plugs in without changing the interface.

## Microphone, camera and screen sharing

These only become relevant once calls run inside Wren (path 2 or 3). On
macOS, the bundle would then need `NSMicrophoneUsageDescription`,
`NSCameraUsageDescription` and Screen Recording permission. The current
bundle declares none, because Wren 0.1 never touches those devices.
