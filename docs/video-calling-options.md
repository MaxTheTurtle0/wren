# In-app calling: options and a plan

*Research as of 2026-10-11, for the question "how could calls run inside Wren, as fast as possible?". Nothing here is built yet; [calling.md](calling.md) describes what Wren does today. Claims that could not be confirmed are marked **UNVERIFIED**. Sources are listed as [S#] at the end.*

## 1. Recommendation

**macOS first:** put the official ACS Web SDK into a lean call page that Wren itself serves from a fixed loopback origin (`http://127.0.0.1:<port>`). Open it as a chrome-less app window in Chrome or Edge, with Safari as the fallback. Wren's egui UI keeps all call controls, the roster and incoming-call prompts, and drives the page over a loopback WebSocket. The page only draws video tiles, and its window is composited by the OS, so Wren stays idle at 0% CPU.

This is the only macOS path that is officially supported, keeps Wren browser-free and uses hardware H.264 with simulcast.

**Windows:** add a native helper (C#/WinUI 3, MSIX-packaged) using the ACS Windows SDK, speaking the same protocol. Windows on Arm and anyone without the helper fall back to the web page.

**Linux:** the web page in Chrome is the only supported option.

Two things not to build:
- **The iOS SDK on Mac (C)** is not feasible.
- **Drawing remote frames inside egui (F)** is either impossible or would lose performance on every platform.

**Phases:**
- **0.** Token-exchange backend, ACS resource and a licence review.
- **1.** macOS web page, egui call bar, meeting join and 1:1 calls.
- **2.** Picture-in-picture, a resident agent for incoming calls, screen sharing, and Linux.
- **3.** The native Windows helper, measured against the web path.
- **4.** Optional: an owner decision on an Electron add-on.

## 2. Comparison

| Idea | macOS | Windows | Linux | In-app-ness | Expected performance | Effort | Compliance risk |
|---|---|---|---|---|---|---|---|
| **A1** Web SDK in a Chrome/Edge app window | Supported | Supported | Chrome only | Medium: own window, no browser UI, Document PiP | Good: hardware H.264, send simulcast; browser memory | Low–Med | Low |
| **A2** Web SDK in Safari (tab or Dock web app) | Supported | – | – | Low–Med | Good: hardware H.264; no send simulcast or Document PiP | Low | Low |
| **B** System webview in a small helper | Not supported (may work, UNVERIFIED) | WebView2 not supported | WebKitGTK not supported | High | Lightest web path on macOS | Med | Med: unsupported |
| **C** iOS SDK on Mac (Catalyst / iPad-on-Mac) | Not feasible | – | – | – | – | High | High |
| **D** Windows SDK in a C# helper | – | Supported (x64, MSIX) | – | High | Likely best on Windows: native media stack (UNVERIFIED) | Med–High | Med: licence text |
| **D′** Windows SDK called directly from Rust | – | Undocumented | – | Highest | Same as D | High | Med–High |
| **F** Remote frames drawn in egui | Not feasible | Possible via raw frames | Not feasible | Highest | Worse: egui redraws at 30 fps, plus copies | High | Low–Med |
| **G** webrtc-rs or reverse-engineered signalling | Impossible | Impossible | Impossible | – | – | – | Prohibited |
| **H** Teams web in an app window (better hand-off) | Yes | Yes | Yes | Medium | Teams web's weight | Very low | None |
| **I** Electron call helper (optional add-on) | Supported | Supported | Not supported | High | Chromium memory, ~100 MB add-on (UNVERIFIED) | Med | Low; ships a browser engine |

## 3. Facts that constrain every option

- **Support matrix [S1]:**
  - Supported: Chrome, Edge, Safari and Firefox (preview) on macOS; Chrome, Edge and Firefox on Windows; only Chrome on Linux.
  - **Electron is listed as supported on macOS and Windows.**
  - Webviews are supported only on Android and on iOS WKWebView (public preview). "GeckoView, Chromium Embedded Framework (CEF), Microsoft Edge WebView2… aren't officially supported."
- **Teams-user (CTE) limits [S3]:**
  - "Join a group call after it has started": ❌ on every SDK.
  - Together mode, custom backgrounds and consent to recording are web-only.
  - Wren must still hand off joins to existing ad-hoc group calls.
- **Cost and licences [S11][S30]:**
  - $0.004 per participant-minute, billed to the ACS resource.
  - Base Teams licences are eligible.
  - The pricing page lists a **Teams Phone licence** for "Use the Calling SDK to start a call with Teams users". Which licence 1:1 VoIP actually needs is UNVERIFIED; test it.
- **Retirement [S10]:** announced 2026-09-23, effective 2028-09-30.
  - The Calling SDK is a *breaking change*, not retired. It stays supported "in conjunction with a Microsoft Teams-aligned product".
  - New major SDK versions will follow.
  - In September 2026 all three SDKs moved to a new major baseline "with an update to the licensing framework": Web 2.0.0, Windows 2.0.0 and iOS 3.0.0 [S13][S15][S17].
- **Licence text inspected:**
  - npm `@azure/communication-calling` 2.0.0 ("STABLE") has a **Distributable Code** clause. Conditions: add significant functionality, pass on protective terms, indemnify Microsoft, and keep it out of copyleft.
  - The iOS 3.0.0 `EULA.md` and the Windows 2.0.0 `license.txt` both say "PRE-RELEASE" and forbid you to "share, publish, distribute". iOS also has a confidentiality clause.
  - This contradicts normal app use, so **get legal review**.
  - Until then, Wren should **not bundle** native SDKs. It should download them on first use from Microsoft's package feeds, with pinned hashes. The MIT licence of Wren's own code is not affected either way.
- **Terms:** the ACS SDK licence 2(a) forbids working "around any technical limitations in the software", and the Microsoft APIs terms say the same [S27]. That rules out user-agent spoofing and binary patching.
- **Codec and quality knobs:**
  - ACS uses H.264 [S8].
  - Desktop Chrome and Edge send 3-layer simulcast by default [S7].
  - Receive resolution follows the size of the rendered tile [S6].
  - The web sender can cap resolution, frame rate and bitrate [S6].
  - Desktop web renders up to 16 remote videos (25 from 1.40); native SDKs render 9 [S1].

## 4. The ideas in detail

### A. Web SDK in the system browser (recommended for macOS and Linux)

**How it works.**
- Wren's tokio runtime serves `call.html`, Wren's own `call.js` (vanilla TypeScript, no framework or UI library, roughly 10–20 KB) and `sdk.bundle.js` on a fixed loopback port.
- The fixed port keeps the origin stable, so camera and microphone permissions persist.
- The SDK bundle is 7.4 MB minified, 1.8 MB gzipped (inspected). It is fetched once from the npm registry with a pinned integrity hash.
- The launcher runs `chrome|msedge --app=http://127.0.0.1:P/call#s=<nonce>` and falls back to the default browser (Safari).
- Serving the page from loopback avoids Chrome's Local Network Access prompt, which gates *public* sites reaching loopback [S20].
- The page connects back to `ws://127.0.0.1:P/ws`. Wren checks the `Origin` and `Host` headers and the one-time nonce.
- On the page, `VideoStreamRenderer.createView()` returns `<video>` elements [S5]. The browser decodes and composites them on the GPU.
- Chromium's Document Picture-in-Picture (Chrome/Edge 116+, not Safari) gives a floating mini-call when focus returns to Wren [S21].
- Safari "Add to Dock" web apps (macOS 14+) give a separate Dock app, but the user has to create it, and it keeps its own website data [S19].

**Performance.**
- Media runs in the browser's WebRTC stack with hardware H.264. VideoToolbox is used on macOS. On Linux, Chrome's WebRTC hardware path is UNVERIFIED and likely software.
- Wren pays nothing while the call window is open, because nothing is drawn in egui.
- The cost is the browser's processes. If Chrome is already running, the app window reuses them, so the added cost is about one renderer process (UNVERIFIED).
- Compared with Teams desktop: Teams uses a WebView2 UI plus a native media engine ("SlimCore", per a secondary source) [S24][S25]. Expect media CPU similar to Teams *web*. The likely win is memory and UI weight, not media efficiency. **Measure; do not assume.**
- Join latency: page load (cached code, UNVERIFIED cost) + `TeamsCallAgent` creation + server join. Pre-warm the first two.

**Compliance.** Fully supported environment. Microsoft distributes the SDK; Wren only fetches it.

**Effort and risk.** Low–Med. Risks: the browser must be installed, users may close the window by accident, and incoming calls need the page resident (see §5).

### B. A native webview helper only for the call window

**How it works.** A roughly 1 MB Swift helper hosts a borderless or floating `NSPanel` with a WKWebView that loads the same page and protocol. It needs:
- `requestMediaCapturePermissionFor` (macOS 12+) [S28],
- `allowsInlineMediaPlayback`,
- camera and microphone usage strings and entitlements.

The Windows equivalent would use WebView2; the Linux one, WebKitGTK.

**Feasibility.**
- **macOS:** listed ❌ [S1]. The same WebKit engine is supported in Safari on macOS and as WKWebView on iOS (preview) [S9], so it may work (UNVERIFIED). WebKit has a history of WKWebView `getUserMedia` problems on macOS.
- **Windows:** WebView2 is named as unsupported.
- **Linux:** WebKitGTK's WebRTC goes through GStreamer and is not supported; the experience is poor.

**Performance.** On macOS this is probably the lightest web option: shared system WebKit, VideoToolbox, no second browser. It is also the most in-app: exact window placement, a native always-on-top PiP panel, and a hidden resident agent.

**Compliance.** Allowed only without user-agent spoofing. Run `callClient.getEnvironmentInfo()` [S29]; if the SDK rejects the environment, stop. Ship it, if at all, as an opt-in experiment.

**Effort and risk.** Medium effort. The risk is high because it is unsupported and can break in any WebKit or SDK update.

### C. ACS iOS SDK on macOS: ruled out

- The 3.0.0 XCFramework (inspected) contains only `ios-arm64` and `ios-arm64_x86_64-simulator`. There is no `macos` or `ios-arm64_x86_64-maccatalyst` slice, so Catalyst builds fail to link.
- Issue #288: Microsoft said it doesn't "officially support MacOSX", and a Catalyst build failed with "framework not found" [S14].
- "Designed for iPad" on Apple Silicon runs the unmodified iOS binary, but it is distributed only through the Mac App Store or TestFlight [S18]. It would be a separate sandboxed app, not a helper Wren could launch, and it is untested by Microsoft.
- Loading the iOS slice into a macOS process would require rewriting the Mach-O platform. That breaks the EULA (no reverse engineering or working around limitations).

### D. ACS Windows SDK (recommended for Windows, phase 3)

**How it works.**
- Rendering: `await stream.StartPreviewAsync()` returns a URI, which goes into `MediaPlayerElement.Source = MediaSource.CreateFromUri(uri)` [S5].
- The manifest must register an in-process activatable class, `VideoN.VideoSchemeHandler` in `RtmMvrUap.dll` [S5]. That is why packaging is required; "Unpackaged app is currently not supported" [S2].
- Video therefore renders through Media Foundation into the XAML/DWM-composited window: hardware decode, no copies through Wren.
- The 2.0.0 nupkg (inspected, 27.8 MB) ships **only `win-x64` native assets for `net8.0-windows`** (about 60 MB: `RtmCodecs`, `RtmMvrMf`, `skypert`, `SlimCV`…). It has no UWP target, no Arm64 and no `.winmd`. Version 1.15.0 still had UWP x86/x64/Arm64 and a `.winmd`.
- So **D′ (Rust through the `windows` crate)** has no current metadata to generate bindings from and needs package activation anyway. Avoid it.
- The helper is a C# WinUI 3 app. A sparse package (packaged with external location) may give package identity without a full MSIX install; whether ACS works that way is UNVERIFIED.

**Window placement.**
- Option 1: a top-level borderless window *owned* by Wren's HWND, which keeps z-order. Wren sends move and resize events over IPC, so it lags a frame or two while dragging.
- Option 2: a cross-process child (`SetParent`), which moves with Wren's window. Raymond Chen calls this legal but "juggling chainsaws", because the input queues get attached [S22].
- Start with the owned window.

**Performance.** Likely the best of all options on Windows: the native RTM media stack, the same family as Teams desktop (UNVERIFIED), and no browser. Costs: .NET 8 plus WinUI startup (pre-warm it) and an add-on of roughly 60–100 MB (UNVERIFIED).

**Compliance.** Supported environment. The shipped licence text says pre-release and no distribution (§3), so download the SDK on first use until legal review is done.

**Effort.** Medium–high: C# helper, MSIX signing and the IPC protocol.

### E. Helper-process architecture (the shape every path above uses)

- **Ownership:** Wren keeps identity, Graph, the roster UI, call controls and notifications in egui. A call process (browser page, Electron, WKWebView or WinUI helper) owns media and video windows.
- **Protocol:** newline-delimited JSON over loopback WebSocket (browser and Electron) or stdio/named pipe (native helpers). It is the same message set everywhere (§6), so `calling::CallBackend` gets one `Acs` implementation with interchangeable transports.
- **Embedding in Wren's window:**
  - macOS: cross-process `NSView`/`NSWindow` parenting is not public API; `CALayerHost` is private, so it is excluded. Use a separate window, optionally a floating panel synced to Wren's frame.
  - Wayland: no cross-process embedding.
  - X11: XEmbed exists, but the web path cannot use it.
  - Treat the call window as a *companion window*: placed beside Wren, borderless where possible, with PiP.

### F. Remote video frames drawn inside egui

**What the SDKs expose [S4]:**
- **Windows:** `RawIncomingVideoStream` with `RawVideoFrameKind.Buffer` (CPU `MemoryBuffer`) or `Texture`. What the incoming texture carries (perhaps a `MediaStreamSample` with a D3D11 surface) is UNVERIFIED.
- **iOS:** `RawVideoFrameBuffer` (`CVPixelBuffer`), which is useless on macOS (C).
- **Web:** only a `MediaStream`. Getting frames into Wren would mean reading back `VideoFrame` pixels (about 41 MB/s for 720p30 NV12) or re-encoding with WebCodecs, then a VideoToolbox decode in Wren. That is double work.

**GPU paths, where frames exist:**
- macOS: IOSurface passed over XPC [S23], bound with `CGLTexImageIOSurface2D` to `GL_TEXTURE_RECTANGLE`, one texture per plane, drawn in an `egui_glow` paint callback.
- Windows: a DXGI shared handle plus `WGL_NV_DX_interop2`.
- Linux: dmabuf plus EGL. No native Linux SDK exists, so this is moot.

**Why not.** egui then has to redraw the whole UI at 30 fps for the whole call, which ends Wren's 0% idle in calls. An OS-composited video layer costs Wren nothing. Revisit only on Windows, as an experiment after D.

### G. Other ideas

- **webrtc-rs or a native libwebrtc client:** impossible. "The Calling SDK uses proprietary network interfaces and is closed-source" [S2], and reverse engineering is forbidden.
- **Graph media bots:** application-only, C# on Windows Server, cannot act as the user.
- **Teams Phone extensibility:** a contact-centre offering (resource accounts, Call Automation plus the same Calling SDK) [S26]. It is no new client path.
- **Cloud Video Interop gateways:** certified-partner meeting-room services; not applicable.
- **H. Teams web in an app window:** zero cost, no backend, and it can join existing calls. Improve today's hand-off by opening Teams web links in an app-mode window when Teams desktop is absent. This remains the fallback for everything CTE cannot do.
- **I. Electron helper:** the only *supported* embedded engine (macOS and Windows). It gives a frameless window, a hidden resident agent (`backgroundThrottling:false`) and exact placement. Costs: Chromium-size download and memory, and it ships a browser engine (not for the app UI). That is an owner decision; offer it only as an on-demand add-on.

## 5. Performance techniques for whichever path is chosen

1. **Pre-warm.**
   - Load the page or helper, and fetch or cache the ACS token, when the person hovers Call or Join.
   - Create `TeamsCallAgent` before the click.
   - Keep the SDK code cached (fixed origin).
2. **A resident agent only on opt-in.** Incoming calls need a live CallAgent. Native/Electron push only applies to apps; web push needs extra plumbing, and the CTE push TTL is at most 24 h [S1]. Make "Ring me in Wren" a setting, with a hidden helper or a minimized app window, and measure its idle wake-ups.
3. **Subscribe only to visible tiles.** Create renderers only for on-screen participants and dispose of them off-screen. Follow `OptimalVideoCount`.
4. **Size tiles to the resolution you want.** Receive resolution follows renderer size [S6].
5. **Cap sending:** for example `frameHeight ≤ 720`, `frameRate ≤ 30`, and lower in gallery view [S6].
6. **Prefer Chrome/Edge for sending** because of simulcast [S7]. Safari is fine for receiving.
7. **Let the OS compositor work:** `<video>`, `MediaPlayerElement` or a CAMetalLayer-backed view. Never draw video in egui.
8. **Audio-only fast path:** join with audio only (supported since Web 1.39.2-beta) [S17]. No camera pipeline; the window can be minimized. Chrome throttling with active WebRTC is UNVERIFIED.
9. **Wren stays event-driven:** call state arrives as `backend::Event`s; no polling.

## 6. Prototype plan (macOS, path A)

**Components**

1. **Token exchange in `wren-relay`:** `POST /acs/token`. It receives an Entra token for `https://auth.msft.communication.azure.com/Teams.ManageCalls`, checks the tenant and app ID, and calls `getTokenForTeamsUser` with the resource key or a managed identity [S12]. A backend-less single-tenant variant with Azure RBAC may be possible (UNVERIFIED).
2. **`calling::acs` in Wren:** a hyper/tungstenite loopback server on a fixed port, a browser launcher (Chrome → Edge → default), an SDK fetcher with a pinned hash, and the protocol.
3. **`call.js`:** `CallClient` → `createTeamsCallAgent`; `join({meetingLink})` and `startCall([{microsoftTeamsUserId}])`; renderers; stats.
4. **egui call bar and incoming-call prompt.**

**IPC messages**

Wren → page:
- `hello{proto,session}`
- `token{token,expiresOn}`
- `joinMeeting{joinUrl,video}`
- `startCall{userIds[],video}`
- `accept{callId,video}`, `reject{callId}`
- `mute{on}`, `camera{on,deviceId?}`, `share{on}`, `hangUp{callId}`
- `layout{maxTiles,maxHeight,maxFps}`
- `pip{on}`, `focus`, `selectDevice{kind,id}`

Page → Wren:
- `ready{sdkVersion,env}`, `needToken`, `agent{state}`
- `incoming{callId,from,video}`
- `call{callId,state,endReason?}`
- `participants[{id,name,muted,speaking,video}]`
- `local{mic,camera,share}`, `devices{…}`
- `stats{rtt,loss,sendRes,recvRes[],fps,encoder,decoder}`
- `error{code,message}`

**What to measure** (same machine, same call, alongside Teams desktop and Teams web)

- **Time to join**, click to `Connected` and to the first remote frame, in three states: cold (browser not running), warm, and hot (pre-warmed agent).
- **CPU** for a 10-minute 1:1 720p30 call, summed over *all* processes: `top`/`powermetrics` on macOS. Record Wren's own process separately.
- **Memory:** `footprint` summed over all processes, as in `docs/performance.md`.
- **Energy:** `powermetrics --samplers tasks,gpu_power` and battery drain over 30 minutes on battery.
- **Hardware codecs:** check `chrome://webrtc-internals` for the encoder and decoder implementation.
- **Resident-agent idle cost** per hour: wake-ups and memory.

**Acceptance criteria**

- Hot join ≤ Teams desktop's join time; cold join ≤ Teams desktop + 2 s.
- Total CPU and memory ≤ Teams desktop in the same call; Wren's process averages < 1% CPU, and its idle CPU stays 0.0% when the call window isn't in use.
- Hardware H.264 encode and decode confirmed; send simulcast active in Chrome.
- No permission re-prompt between sessions.
- The WebSocket rejects a foreign `Origin` or a wrong nonce; no tokens persist in browser storage.
- Works in Chrome, Edge and Safari on macOS. Unsupported cases (joining an existing ad-hoc group call, PSTN without a licence) fall back to the deep link with a clear message.

## Sources

- [S1] https://learn.microsoft.com/en-us/azure/communication-services/concepts/voice-video-calling/calling-sdk-features
- [S2] https://learn.microsoft.com/en-us/azure/communication-services/concepts/sdk-options
- [S3] https://learn.microsoft.com/en-us/azure/communication-services/concepts/interop/teams-user-calling
- [S4] https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/voice-video-calling/get-started-raw-media-access
- [S5] https://learn.microsoft.com/en-us/azure/communication-services/how-tos/calling-sdk/manage-video?pivots=platform-windows
- [S6] https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/voice-video-calling/get-started-video-constraints
- [S7] https://learn.microsoft.com/en-us/azure/communication-services/concepts/voice-video-calling/simulcast
- [S8] https://learn.microsoft.com/en-us/azure/communication-services/concepts/voice-video-calling/about-call-types
- [S9] https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/voice-video-calling/get-started-webview
- [S10] https://learn.microsoft.com/en-us/azure/communication-services/acs-retirement-and-breaking-changes-guide
- [S11] https://learn.microsoft.com/en-us/azure/communication-services/concepts/pricing/teams-interop-pricing
- [S12] https://learn.microsoft.com/en-us/azure/communication-services/concepts/interop/custom-teams-endpoint-authentication-overview
- [S13] https://github.com/Azure/Communication/releases/tag/v3.0.0 (XCFramework `Info.plist` and `EULA.md` inspected)
- [S14] https://github.com/Azure/Communication/issues/288
- [S15] https://www.nuget.org/packages/Azure.Communication.Calling.WindowsClient (2.0.0 and 1.15.0 inspected); https://github.com/Azure/Communication/blob/master/releasenotes/acs-calling-windowsclient-sdk-release-notes.md
- [S16] https://www.npmjs.com/package/@azure/communication-calling (2.0.0 LICENSE and bundle inspected)
- [S17] https://github.com/Azure/Communication/blob/master/releasenotes/acs-javascript-calling-library-release-notes.md
- [S18] https://developer.apple.com/help/app-store-connect/manage-your-apps-availability/manage-availability-of-iphone-and-ipad-apps-on-macs-with-apple-silicon
- [S19] https://support.apple.com/en-us/104996
- [S20] https://developer.chrome.com/blog/local-network-access
- [S21] https://developer.chrome.com/docs/web-platform/document-picture-in-picture
- [S22] https://devblogs.microsoft.com/oldnewthing/20130412-00/?p=4683
- [S23] https://developer.apple.com/forums/thread/18958
- [S24] https://learn.microsoft.com/en-us/microsoftteams/teams-client-system-requirements ; https://techcommunity.microsoft.com/t5/microsoft-teams-blog/microsoft-teams-advantages-of-the-new-architecture/ba-p/3775704
- [S25] https://duo-infernale.ch/teams-on-vdi-levels-up-updates-in-march-2026/ (secondary)
- [S26] https://learn.microsoft.com/en-us/azure/communication-services/concepts/interop/tpe/teams-phone-extensibility-capabilities
- [S27] https://developer.microsoft.com/en-us/graph/terms-of-use
- [S28] https://developer.apple.com/documentation/webkit/wkuidelegate
- [S29] https://learn.microsoft.com/en-us/azure/communication-services/how-tos/calling-sdk/browser-support
- [S30] https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/eligible-teams-licenses
