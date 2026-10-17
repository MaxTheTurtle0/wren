# Performance

Measured on 15 October 2026, after the fixes based on `origin/main` at `f402485`.
See the [baseline investigation](performance-investigation-2026-10-15.md) and
[fixes, comparisons and reproduction details](performance-improvements-2026-10-15.md).

## Native release builds on macOS

Both machines have 16 GiB RAM and macOS 27.0.1. The M4 Mac mini uses a 1× display;
the M3 MacBook Air uses a 2× Retina display. Memory is physical footprint in MiB.

| Workload | Mac mini | MacBook Air |
|---|---:|---:|
| Ordinary app, offline demo, settled | 57–59 MiB | Not measured after these fixes |
| Signed-in app after restart, 16 October | Not measured | 130 MiB |
| Performance fixture, idle | 57 MiB | 122 MiB |
| Fixture after 20 full HD image previews, viewer closed | 86–88 MiB | 161 MiB |
| Fixture after 100 full HD image previews | 92 MiB | Not measured |
| Fixture after five 4K image previews | 159 MiB | Not measured |
| Settled idle CPU | 0.0% | 0.0% |

The ordinary executable includes the tray/window shell; the fixture does not.
The original signed-in Air process was measured at 529.7 MiB during the baseline
investigation and was preserved during the demo comparisons. The 130 MiB result
was a separate observation after restarting the signed-in app on 16 October,
with four idle CPU samples at 0.0%. It is not a matched long-session comparison.

On the Air, holding a message-loading indicator changed GPU active time from
19.3% to 0.0% in matched eight-second trace windows. This is Instruments GPU
active time for the fixture PID, not Activity Monitor's `% GPU`. On the Mini,
a 5,000-message scroll workload fell from 80.85% to 22.55% of one CPU core.
Settled windows return to sleep. Loading indicators are static, and native
frames have a ceiling at the monitor refresh rate if OpenGL vsync does not block.

## Memory policy

| Storage | Policy |
|---|---|
| Compressed display-image sources | 16 MiB shared RAM budget |
| Thumbnail textures | 16 MiB RAM/GPU texture budget; current visible working set can exceed it |
| Original preview texture | Released on the first pass without the viewer |
| Decoded image pixels | One worker; released after texture upload; 128 MiB decoder allocation limit |
| Graph-hosted encoded downloads | 128 MiB SQLite payload budget per account, least recently used |
| Inactive histories | At most 12 conversations total and 2,000 inactive messages including replies |
| Draft and pending/failed-send images | Authoritative PNGs retained independently of the display cache |
| Script fonts | Loaded by a worker when visible text needs them; then retained |

The active conversation is preserved. Conversations with unsent messages or
replies can exceed the history limits. Evicted histories reopen from SQLite
without forcing a Graph fetch. SQLite metadata, reusable pages and WAL data
are outside the encoded-download budget.

These limits do not cap the whole process. Retina window surfaces, the font
atlas, decoder temporaries, protected drafts and driver/allocator allocations
also consume memory. The five-4K-image run peaked at 317 MiB despite settling
at 159 MiB with only 2.27 MiB in the texture loaders. Once script fonts were
needed, the Mini's Unicode fixture used 135 MiB rather than the Latin demo's
57 MiB. CJK and Arabic glyph availability was verified on both Macs.

## History layout

Release-mode headless frames over 100,000 messages take about 0.10 ms median
and 0.11 ms p95 on the M4, compared with 5.79 ms median before the fixes.
Only nearby rows are laid out. A cached prefix-sum index finds those rows and
updates their measured heights without walking the full history every frame.
New histories and width/font changes still require an initial index rebuild.
These timings exclude tessellation, GPU rendering, networking and frame pacing.
Tests run the same views without a display or account.

## Measuring

```sh
cargo build --locked --release --bin wren --example perf_probe
CODESIGN_IDENTITY=- \
packaging/macos/bundle.sh target/release/wren target/Wren.app 0.1.0
open -W -n --stdout /tmp/wren-startup.txt target/Wren.app --args --demo --measure-startup
open -n target/Wren.app --args --demo
# After settling, inspect this bundle's PID with footprint, vmmap and top.

cargo run --locked --release --example perf_probe -- \
  --case history --messages 100000 --frames 300 --output history.jsonl
```

The reproducible Inter configuration is the default for headless comparisons.
Use `--system-fonts` for normal platform fonts. Native probe runs should use a
separate signed bundle launched with `open`, as described in the report.
Launching a bare executable over SSH can add several seconds of LaunchServices
startup delay. In the app, ⌘⇧P shows frame CPU time, frame count and time to the
first UI frame.

Microsoft Teams was not measured side by side. Comparing clients requires
summing every helper process and matching window size, scale, account and view.
