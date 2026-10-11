# Performance

Measured, not assumed. Re-measure after changes that touch startup, fonts,
rendering or the history, and update this page.

## macOS 27.0.1, Apple Silicon (Mac mini), Wren 0.1.0 release build, demo data

| Metric | Result | How |
|---|---|---|
| Launch to first frame | **143–157 ms** warm (6 runs); **301 ms** for the first launch after a rebuild (cold disk cache) | `open -W -n Wren.app --args --demo --measure-startup` (prints the time from `main` to the first painted frame) |
| Idle CPU, window open and visible | **0.0%** | `top -l 3 -s 10 -pid <pid> -stats cpu`, after 8 s idle |
| Physical memory footprint | **135–141 MB** | `footprint <pid>` |
| …of which system fallback fonts (CJK, Arabic, …) | ~29 MB | Same run with fallbacks off (`--screenshot` mode) measures 112 MB |
| …of which window surfaces (IOSurface) | ~20 MB | `vmmap --summary` |
| Threads | 6 | `top -stats threads` |
| Binary size | 15 MB (stripped, thin LTO) | `target/release/wren` |
| App bundle | 15 MB | `du -sh Wren.app` |

A 6-second first frame shows up when the bare binary is launched over SSH.
That is macOS's LaunchServices timing out for a process outside the GUI
session, not Wren; launching the bundle with `open` is the representative
measurement.

## Headless (Linux CI or any machine)

`tests/ui_smoke.rs::large_conversations_stay_cheap` lays out a conversation
of **5,000 messages** and requires a frame to take under 250 ms even in an
unoptimized debug build. Virtualization means only rows near the viewport
are laid out.

## Why it is cheap

- No browser engine. egui draws with OpenGL (glow).
- Frames are drawn only on input or backend events. There is no animation
  timer and no network polling.
- Every view opens from the SQLite cache; network answers update in place.
- Lists are virtualized, and media loads lazily, only when on screen.

## Known costs and next steps

1. **Fallback fonts (~29 MB).** fastframe-fonts loads one face per script
   that the interface font lacks, at startup. Loading them only when text in
   that script first appears would save most of this for people who read
   only Latin scripts. That is an upstream (fastframe) improvement.
2. **Font atlas.** egui keeps a CPU-side copy of its glyph atlas. The type
   scale is kept to ten sizes to limit it.
3. **Comparison with Microsoft's client.** Not measured yet: Microsoft Teams
   was not running on the test machine. To compare, run both side by side and
   use `footprint` / Activity Monitor's "Memory" column for every process of
   each app (Teams uses several helper processes, which all count).

## Measuring yourself

```sh
cargo build --release
packaging/macos/bundle.sh target/release/wren target/Wren.app 0.1.0   # macOS
open -W -n target/Wren.app --args --demo --measure-startup              # time to first frame
open -n target/Wren.app --args --demo; sleep 8
footprint $(pgrep -f Wren.app/Contents/MacOS/wren)                      # memory
top -l 3 -s 10 -pid $(pgrep -f Wren.app/Contents/MacOS/wren) -stats cpu # idle CPU
```

In the app, ⌘⇧P shows frame CPU time, frame count and time to first frame.
