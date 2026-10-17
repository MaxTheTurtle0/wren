# Third-party notices

Wren's own code and wren-relay are MIT-licensed (see [LICENSE](LICENSE)).
[licenses/dependencies.txt](licenses/dependencies.txt) contains the full
license texts and copyright notices for the dependencies pinned in
`Cargo.lock`, covering both workspace packages on all three packaged targets.
Build dependencies are included; an individual binary uses a subset of the
listed works. When a crate offers alternative licenses, the report records
the license selected by `about.toml`.

The report includes upstream authors and source locations. `dispatch` 0.2.0
declares MIT and author Steven Sheldon but publishes no standalone license
text; its entry identifies the standard SPDX text. The complete original MIT
notice for the related objc2 crates is retained from their earlier upstream
source. Their current [license summary](licenses/objc2-license.md), including
the Apple SDK note, is included too. dpi's embedded libm code has a separate
[MIT notice](licenses/dpi-libm-MIT.txt).

The font and icon notices below also accompany every package. Cargo package
metadata alone does not describe every embedded asset's license.

| Work | Licence | Where |
|---|---|---|
| [Lucide](https://lucide.dev) icons (some derived from Feather) | ISC (Feather: MIT) | `assets/icons/`, and the shared icons compiled in from fastframe-icons. Notice: [assets/icons/LICENSE.txt](assets/icons/LICENSE.txt) |
| [Inter](https://rsms.me/inter/) typeface, bundled by fastframe-fonts | SIL Open Font License 1.1 | [assets/fonts/Inter-LICENSE.txt](assets/fonts/Inter-LICENSE.txt) |
| Hack, bundled by egui | MIT, Bitstream Vera; DejaVu contributions public domain | [licenses/fonts/Hack-Regular.txt](licenses/fonts/Hack-Regular.txt) |
| Noto Emoji, bundled by egui | SIL Open Font License 1.1 | [licenses/fonts/NotoEmoji-OFL.txt](licenses/fonts/NotoEmoji-OFL.txt) |
| Ubuntu Light, bundled by egui | Ubuntu Font Licence 1.0 | [licenses/fonts/Ubuntu-UFL.txt](licenses/fonts/Ubuntu-UFL.txt) |
| emoji-icon-font, bundled by egui | MIT | [licenses/fonts/emoji-icon-font-MIT.txt](licenses/fonts/emoji-icon-font-MIT.txt) |
| [fastframe](https://github.com/crmne/fastframe) crates | MIT | Cargo dependencies |
| [egui / eframe](https://github.com/emilk/egui) (crmne/egui fork) and [winit](https://github.com/rust-windowing/winit) (crmne/winit fork) | MIT or Apache-2.0 | Cargo dependencies |
| [SQLite](https://sqlite.org) via rusqlite's bundled build | Public domain | Compiled into the binary |
| Other Rust crates | See the selected license for each crate | [licenses/dependencies.txt](licenses/dependencies.txt) |

Regenerate the notices after dependency changes with
`packaging/generate-notices.sh`. It uses cargo-about 0.9.2 and copies the
supplemental font notices verbatim from the pinned egui source. CI and the
release workflow reject stale notices. No license generator is needed to
read the notices or install a binary. `wren --licenses` and
`wren-relay --licenses` print the complete embedded notice set without opening
a window or starting a server. This keeps notices available when an updater
or administrator copies only the executable.

The relay ships the same notice set as the desktop client, including asset
licenses for consistency; it does not embed the interface fonts or icons.

Architecture and engineering conventions are inspired by
[Spotifast](https://github.com/crmne/spotifast) and
[ZapFast](https://github.com/crmne/zapfast) (both MIT), by Carmine Paolino.
Wren copies no Spotify- or WhatsApp-specific code.

Microsoft, Microsoft Teams, Microsoft 365, Entra and Outlook are trademarks of
the Microsoft group of companies. Wren is an independent project and is not
affiliated with, endorsed by, or sponsored by Microsoft.
