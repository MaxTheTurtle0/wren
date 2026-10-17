# Wren performance investigation, 15 October 2026

The reported memory use is reproducible. The existing image loaders retain
full-resolution decoded images and textures after the person leaves the image
viewer. Separately, a loading spinner continuously redraws the window and can
produce sustained GPU activity. Neither finding requires a large message history.

This report records the baseline before the subsequent
[performance fixes](performance-improvements-2026-10-15.md). The baseline
session was left running throughout the investigation. Results are also
available as [machine-readable data](performance-data/2026-10-15.json).

## Builds and machines

- Baseline source: `ff6ade2`.
- The MacBook Air baseline used the installed release with a signed-in account.
- MacBook Air: Apple M3, 16 GiB, macOS 27.0.1, Retina scale 2. Its reported
  desktop size was 1470 x 956 points. The real Wren window was 1470 x 923 points.
- Mac mini: Apple M4, 16 GiB, the same macOS build, 1920 x 1080 display at
  scale 1. The original installed binary was copied into a separate demo bundle
  here to establish the ordinary-app baseline.
- The diagnostic executable used the source at `ff6ade2`, release optimizations,
  the existing glow renderer, real Wren views, and offline demo state. Its native
  window was 1280 x 800 points. It does not include the normal tray/window shell.

Memory below is MiB. macOS's command-line tools label these values MB or M.
CPU uses the macOS convention that one fully occupied core is 100%.

## The live MacBook Air process

`footprint`, `vmmap --summary`, `heap`, `sample`, `top`, and Instruments all
inspected the existing process without restarting it or changing its view.

| Measurement | Result |
|---|---:|
| Initial physical footprint | 529.7 MiB |
| Lifetime peak physical footprint | 642.2 MiB |
| Large heap allocations, footprint category | approximately 235 MiB |
| IOAccelerator graphics allocations | approximately 115 MiB |
| IOSurface window surfaces | approximately 104 MiB |
| Small heap allocations | approximately 52 MiB |
| CPU over a 30-second idle capture | 0.0 to 0.2% |
| Later footprint, same process without a restart | approximately 460 MiB |

The sampled main thread waited for events. The runtime and backend threads also
waited. Instruments reported App Nap active during that initial capture, so it
does not establish the GPU cost of a visible, busy window. About 349 MiB was
compressed; RSS alone substantially understated this process's footprint.

The later decrease means these observations do not prove a continually growing
allocation leak. They do prove a high retained footprint. Without allocation
stack logging on the original process, its large heap allocations cannot all be
assigned to individual Wren objects.

## Image retention

The final native test opened 20 unique image URIs, each holding a 1920 x 1080
PNG, and waited for each texture to become ready and paint before advancing.
It then closed the viewer, navigated to Calendar, and remained there. The pixel
fixture repeats, but each URI has its own loader entry. Measurements lasted
22 seconds per native trial.

| Mac mini release workload | Physical footprint | Decoded image cache | Image textures |
|---|---:|---:|---:|
| Ordinary Wren demo app | 139 MiB | not instrumented | not instrumented |
| Diagnostic demo, settled | 138 MiB | negligible | negligible |
| After viewing 20 HD images and leaving the viewer | 476 MiB | 158.2 MiB | 158.2 MiB |
| Same workload with diagnostic `reduce_texture_memory` enabled | 325 MiB | 0 MiB | 158.2 MiB |

The cache totals remained at the end of the trials. A separate headless test
produced the same 158.2 MiB decoded allocation and 158.2 MiB of texture metadata.
Five 3840 x 2160 images consumed the same amount, as expected from pixel count.
The headless texture number describes requested texture storage; the native run
also confirms actual driver allocations, approximately 162 MiB in the graphics
footprint category after the 20-image workload.

One HD RGBA image requires 7.91 MiB before encoding. Twenty CPU copies plus
twenty texture copies require approximately 316.4 MiB, even though the encoded
fixtures together occupy about 1 MiB. That is enough to explain hundreds of
megabytes beyond startup. It does not prove exactly which images account for
the real Air session's heap.

The code path is `App::handle(Event::Media)` and `register_image` in
[`app.rs`](../src/app.rs), followed by egui_extras' image loader and egui's
default texture loader. In the pinned egui revision, PNG/JPEG decoding ignores
the display size hint and caches full-resolution pixels. Drawing a small
thumbnail therefore does not make its decoded source small. The image-loader
cache has no size budget; Wren does not call `forget_image` when leaving a view.
The account also retains media URIs and opened conversations.

`reduce_texture_memory` is a diagnostic comparison, not an applied fix. It
removes the CPU copy after texture upload but leaves the texture allocation.
It also drops included source bytes, so source availability, retry, reopening,
and full-size preview behavior need checking before using it in production.

## GPU and repaint activity

The Air ran two separate 55-second offline native fixtures at scale 2. Each
received the same system font configuration. Synthetic inputs replaced real
typing and pointer events inside the fixture; no input was sent to the installed
Wren session.

| Retina fixture | CPU median during capture | Total paint callbacks over 55 seconds | GPU active time within an 8-second trace window |
|---|---:|---:|---:|
| Settled demo | 0.0% | 196 | 0 ms, 0.0% |
| Same demo with the message-loading spinner held on | 8.6% | 3,042 | 1,545.9 ms, 19.3% |

Paint counts include startup and recorder-related events. They are measured by
a callback in the actual glow painter, rather than inferred from UI updates.
The spinner averaged 55.3 paint callbacks per second over the whole trial.
CPU samples include profiling overhead and changes in window visibility; these
are not dedicated-machine laboratory CPU benchmarks.

GPU measurements came from Instruments' Metal System Trace, which also records
the GPU work submitted by this OpenGL app. Only active intervals belonging to
the fixture PID were counted. The union of Vertex, Fragment and Compute
intervals avoids double counting overlapping stages, following Apple's
[GPU timeline description](https://developer.apple.com/documentation/xcode/analyzing-apple-gpu-performance-using-a-visual-timeline).
The comparison uses trace time 1 through 9 seconds, excluding recording
startup and shutdown. This percentage is a time-based trace measurement, not
Activity Monitor's `% GPU` counter.

The loading indicator at [`conversation.rs`](../src/ui/conversation.rs), line
205, uses `egui::Spinner`. Its `paint_at` calls `request_repaint()` every frame.
Other spinners appear in the sidebar and sign-in view. Each requests another
whole-window frame for a 14-point or 22-point indicator. A slow request can keep
this going: Graph permits up to four 45-second attempts plus retry waits.
This is a reproducible explanation for sustained GPU use while the person is
waiting. The screenshots alone do not establish that a spinner was present in
the original 14% observation.

The Mac mini exposed an additional pacing issue in this display configuration.
The spinner painted 71,035 times in 22 seconds and consumed a median 93.75% CPU.
A 5,000-message scroll workload, with synthetic wheel events capped at 60 Hz,
painted 25,361 times and consumed 80.85% CPU. Paint callbacks confirmed these
were renderer calls. The Air behaved differently. Check glow frame pacing on
the Mini before treating its active CPU figures as representative of the Air.
Settled idle and a static hover both returned to 0% CPU on the Mini.

## History stress

These are release-mode, headless UI-layout timings on the M4. Each case ran
300 frames through Wren's real views and tessellated the resulting shapes.
The timed UI region excludes tessellation, GPU work, and network work.

| Messages in the open conversation | Median UI time | p95 UI time |
|---|---:|---:|
| 500 | 0.138 ms | 0.146 ms |
| 5,000 | 0.400 ms | 0.412 ms |
| 50,000 | 2.991 ms | 3.147 ms |
| 100,000 | 5.788 ms | 6.206 ms |

The history skips drawing distant rows, but still walks every message and
computes grouping and day boundaries each frame in `conversation::history`.
The roughly linear cost is measurable. It was less urgent than image retention
and repeated rendering in these workloads. A separate debug-build Linux run
measured about 4 ms at 5,000 messages and 36 ms at 50,000; those numbers should
not be substituted for release performance.

## Fix order suggested by the measurements

1. Give decoded images and textures explicit byte budgets and evict unused
   entries. Use thumbnail-sized decoded images for normal chat/composer views;
   load full resolution when opening the viewer. Preserve sources needed for
   drafts, failed sends, retry, and preview reopening.
2. Stop unbounded spinner-driven whole-window rendering. Measure a slower
   loading animation or a static indicator, and verify that settled, hidden,
   minimized, and waiting windows stay cheap. Inspect frame pacing in the
   pinned glow/winit forks on the Mini.
3. Revisit startup font costs. The Mini's ordinary font/emoji configuration
   used 138 MiB idle; the reproducible CI configuration used 57 MiB. That
   81 MiB difference includes the primary font, fallback setup and emoji
   initialization. It is not an isolated measurement of fallback fonts.
4. Index history offsets/group boundaries if very large retained histories
   become common. Keep paging and scroll anchors correct.

## Repeating the measurements

The example writes JSON Lines containing cache sizes, UI timings, repaint
call sites and native paint counts. It uses temporary demo directories and
does not read the credential store, send messages, or fetch Graph resources.

```sh
cargo run --locked --release --example perf_probe -- \
  --case history --messages 100000 --frames 300 --output /tmp/history.jsonl

cargo run --locked --release --example perf_probe -- \
  --case images --images 20 --frames 400 --output /tmp/images.jsonl

cargo run --locked --release --example perf_probe -- \
  --case images --images 20 --frames 400 --reduce-texture-memory \
  --output /tmp/images-reduced.jsonl
```

For native macOS runs, put `target/release/examples/perf_probe` into a separate
app bundle as `Contents/MacOS/wren`, use a distinct bundle identifier, and
ad-hoc sign that bundle. Launch through `open`, as for normal Wren measurements:

```sh
open -W -n Probe.app --args --native --case spinner --system-fonts \
  --seconds 55 --output /tmp/spinner.jsonl

top -l 30 -s 1 -pid PID -stats pid,cpu,mem,threads,ports,time
footprint PID
vmmap --summary PID
xcrun xctrace record --template 'Metal System Trace' --attach PID \
  --time-limit 10s --output /tmp/spinner.trace
```

Allow Instruments time to save after the recording limit. One early capture
was terminated during saving and was discarded. `top` has no per-process GPU
column, `powermetrics` required an unavailable sudo password, and the Mach GPU
counter returned zero even for active Apple Silicon rendering. None of these
were used as evidence of zero GPU usage.

The native workloads were run on macOS only. Linux ran the headless probe and
all 161 tests. Formatting, Clippy with warnings denied, and Rust documentation
checks passed. Windows was not run or compiled for this investigation.
No fixes were applied, and no product release was published or installed.
