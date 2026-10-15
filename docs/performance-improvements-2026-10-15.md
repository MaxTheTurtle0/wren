# Wren performance fixes, 15 October 2026

These changes address the retained image memory and repeated rendering found in
[the baseline investigation](performance-investigation-2026-10-15.md). They also
remove the full-history scan from ordinary frames and defer large script fonts
until visible text needs them. The baseline was `origin/main` at `ff6ade2`; implementation commit is
`cb6aae8`.
The repository has `main`, not `master`; origin was checked again before the
final measurements.

## What changed

- Downloaded and pasted images use a local loader with a 16 MiB compressed-source
  budget and a separate 16 MiB thumbnail-texture budget. The current visible
  working set can exceed the texture budget. Ordinary thumbnails decode to
  physical-pixel size buckets, up to 1,024 pixels on the longest edge. Cropped
  composer images use the short edge for detail, capped at a 4,096-pixel long
  edge. Uploads also respect the renderer's maximum texture side. The viewer
  still decodes the original dimensions for Fit and 100% modes.
- Original textures are released on the first pass without the viewer. CPU
  image pixels are released after texture upload. One worker decodes images,
  keeping decoding off the interface thread and limiting concurrent allocations.
  JPEG EXIF orientation and the existing static first-frame GIF behavior remain.
  The composer loads by URI rather than also retaining PNGs in egui's unbounded
  included-byte map.
  The decoder has a 128 MiB allocation limit.
- Encoded Graph image downloads use a 128 MiB least-recently-used SQLite cache per
  account. Eviction from RAM can reload from disk. The payload budget excludes
  SQLite metadata, reusable pages and WAL data. Signing out removes the account
  cache. Draft and pending-send PNGs remain independent of the display cache.
- Loading indicators are static. Waiting for Graph no longer schedules an
  animation of the whole window. Native render loops also have a ceiling at the
  monitor refresh rate when OpenGL vsync does not block. The ceiling never wakes
  an idle window and follows monitor changes.
- A cached prefix-sum index finds visible history rows and updates measured row
  heights. Ordinary frames no longer recalculate every message's day/grouping.
  RAM retains the active conversation and a recent set of at most 12 conversations
  and 2,000 inactive messages including replies. Conversations with pending or
  failed sends are protected, even if that exceeds the limits. Drafts survive
  eviction. A separate disk-only command reopens cached history without forcing
  a Graph fetch or bypassing freshness rules. A late disk result cannot
  overwrite a newer Graph answer.
- The platform interface font still loads normally. A worker loads the existing
  script fallbacks when newly visible text lacks a non-emoji glyph. Latin-only
  sessions avoid that startup allocation. Once needed, script fonts stay loaded.

No dependency, Graph permission, polling rule, credential handling or external
image-host policy changed.

## Matched measurements

The machines and release configuration match the baseline: an M4 Mac mini
with a 1× display and an M3 MacBook Air with a 2× Retina display, both with
16 GiB RAM and macOS 27.0.1. Native fixtures use ordinary system fonts.
[Machine-readable results](performance-data/2026-10-15-after.json) include
cache counts, timings, CPU summaries and trace interval summaries.

| Workload | Before | After |
|---|---:|---:|
| Mac mini, idle fixture footprint | 138 MiB | 57 MiB |
| Mac mini, 20 distinct full HD previews, viewer closed | 476 MiB | 86–88 MiB |
| Mac mini, loading CPU median | 93.75% | 0.0% |
| Mac mini, 5,000-message scrolling CPU median | 80.85% | 22.55% |
| Mac mini, scrolling paint callbacks in 22 seconds | 25,361 | 1,175 |
| Air, loading GPU active time in eight-second trace window | 1,545.9 ms, 19.3% | 0 ms, 0.0% |
| Air, loading paint callbacks in 55 seconds | 3,042 | 7 |
| Air, loading CPU median including recorder activity | 8.6% | 0.0% |

The full HD workload's settled footprint fell by about 82%. A repeat with the
final image loader measured 88 MiB instead of 86 MiB, with identical live cache
counts. CPU copies of the viewed raster images no longer remain after upload. The texture-loader count
fell from 158.2 MiB to 6.77 MiB, including small interface assets. The original
158.2 MiB decoded-image cache is also gone; about 16 KiB of SVG loader data
remains. The compressed source images remain available within the RAM budget.

The additional native stress tests completed all 100 distinct full HD previews
in 45 seconds and settled at 92 MiB, with a 164 MiB peak. Their retained
thumbnail textures were identical to the 20-image run, 6.77 MiB. Encoded sources
grew from about 0.97 MiB to 4.77 MiB, as expected for distinct URIs.

Five 4K previews also completed. That run settled at 159 MiB with a 317 MiB
peak, although live texture-loader storage was only 2.27 MiB and decoded raster
pixels had been released. This exposes remaining process allocation costs
outside Wren's live image-cache counters. It is not evidence that the whole
process stays within the cache budgets.

The unprofiled Air fixtures settled at 122 MiB idle and 161 MiB after 20 full
HD images. These are separate offline runs, not the original signed-in account.
Holding the loading state no longer kept the GPU busy. The settled idle trace
contained one small burst, 1.44 ms across eight seconds, about 0.018% active time.

Native Unicode fixtures on both Macs changed the fallback-font revision from
0 to 1 and reported available CJK and Arabic glyphs. The Mini's footprint with
script fonts loaded was 135 MiB, versus 57 MiB for the Latin demo. Deferring
fonts saves memory only until the session needs those fonts.

The ordinary app, including its tray/window shell, settled at 57–59 MiB on
the Mini. Six final demo launches reached the first UI frame in 101–120 ms.

The following release-mode history comparisons use the same reproducible Inter
font configuration and 300 headless frames on the M4. Timings cover the UI region.

| Messages | Before median / p95 | After median / p95 |
|---:|---:|---:|
| 500 | 0.138 / 0.146 ms | 0.112 / 0.142 ms |
| 5,000 | 0.400 / 0.412 ms | 0.104 / 0.113 ms |
| 50,000 | 2.991 / 3.147 ms | 0.103 / 0.105 ms |
| 100,000 | 5.788 / 6.206 ms | 0.102 / 0.105 ms |

At 100,000 messages, steady UI layout is about 57 times faster. The first
index build remains proportional to history size. Native scrolling now paints
about 53 times per second instead of about 1,153 in the previous configuration.
The ceiling stops the run-away render loop; it does not guarantee 60 rendered
frames every second.

## Reproduction

The offline [performance probe](../examples/perf_probe.rs) uses Wren's real views
and the same native frame pacer as the ordinary executable. It never opens an
account or reads credentials. Native image workloads wait for each texture and
an actual glow paint before advancing, then close the viewer and switch to the
calendar. Each image has a distinct URI; the 1,920 x 1,080 fixture is about
50 KiB encoded but 7.91 MiB as RGBA pixels.

```sh
cargo build --locked --release --bin wren --example perf_probe
cargo run --locked --release --example perf_probe -- \
  --case history --messages 100000 --frames 300 \
  --output history.jsonl
cargo run --locked --release --example perf_probe -- \
  --native --case images --images 100 --seconds 45 --system-fonts \
  --output images.jsonl
```

On macOS, put the release probe in a separate signed app bundle and launch it
with `open -W -n ... --args ...`, as in the baseline method. A bare executable
over SSH has different LaunchServices startup behavior. Use `footprint` and
`vmmap --summary` for memory, and `top` for CPU. GPU captures attach Instruments'
Metal System Trace for ten seconds and union only the fixture PID's active
Vertex/Fragment/Compute intervals in trace seconds 1 through 9. GPU active time
is not Activity Monitor's `% GPU` counter. CPU figures include any profiler
activity and ordinary machine contention. Paint counts include startup and
recorder-related events.

Headless UI timings exclude tessellation, GPU rendering, networking and the
native frame pacer's wait. They measure steady frames after the first index
build; a new history or width/font change still requires rebuilding estimates.
Memory values are MiB, despite the macOS tools' MB/M labels.

## Verification and limits

- Linux: formatting, Clippy with warnings denied, all 175 tests, and rustdoc
  with warnings denied passed. UI tests use headless egui frames; a native
  Linux window was not run on the headless host.
- macOS arm64: release app and probe built on the M3. Formatting, Clippy with
  warnings denied, and all 175 platform tests passed. Rustdoc with warnings
  denied passed. Native fixtures ran on both the Air and Mini. Test targets were forced to rebuild, and source
  hashes were matched with Linux before accepting the final result.
- Native screenshots cover chats, pasted images and original previews in
  both themes at 1,280 x 800 and 760 x 700 points. The screenshot check caught
  a cropped-panorama quality issue; cropped thumbnails now retain detail on
  the short edge. The final app was checked again after this correction.
- Regressions cover cache budgets and eviction, original re-opening and release,
  JPEG orientation, GIF URI handling, cropped detail and texture limits, draft
  preservation, exclusion from egui's unbounded byte map, pending/failed sends
  including channel replies, inactive-history limits, disk-only re-opening
  and late-cache races, repeated media downloads hitting the disk cache, loading repaint delays and
  the prefix-sum index. Existing geometry, scrolling and backend tests remain.
- Windows was not compiled or run. The reachable Windows host has no Rust
  toolchain. No platform-specific dependencies or fork revisions changed.

The final ordinary release was also launched on the Air from the separate
`/tmp/wren-perf-486982bd/Wren-fixed.app` bundle. Its first UI frame took 455 ms
on that first launch. The binary SHA-256 is
`8c633d099a3a0a08ad4c6c6a9be94674590126e0bfedd42ba2399b5fbeb56afa`.
This bundle was not installed over `/Applications/Wren.app`.

The original signed-in MacBook Air process was left running. The test bundles
use offline demo state and separate identities. No before/after result here
claims that the existing 530 MiB session was restarted into the fixed build or
that a particular real account will use the demo footprint. A large Retina
window, active original image, loaded script fonts, and protected unsent content
all have legitimate costs. The fixes remove the measured unbounded display-image
retention and continuous loading animation; they do not promise a fixed total
process-memory ceiling.
