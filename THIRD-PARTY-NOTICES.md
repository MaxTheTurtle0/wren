# Third-party notices

Wren's own code is MIT-licensed (see [LICENSE](LICENSE)). It includes or is
built from these works:

| Work | Licence | Where |
|---|---|---|
| [Lucide](https://lucide.dev) icons (some derived from Feather) | ISC (Feather: MIT) | `assets/icons/`, and the shared icons compiled in from fastframe-icons. Notice: [assets/icons/LICENSE.txt](assets/icons/LICENSE.txt) |
| [Inter](https://rsms.me/inter/) typeface, bundled by fastframe-fonts | SIL Open Font License 1.1 | [assets/fonts/Inter-LICENSE.txt](assets/fonts/Inter-LICENSE.txt) |
| [fastframe](https://github.com/crmne/fastframe) crates | MIT | Cargo dependencies |
| [egui / eframe](https://github.com/emilk/egui) (crmne/egui fork) and [winit](https://github.com/rust-windowing/winit) (crmne/winit fork) | MIT or Apache-2.0 | Cargo dependencies |
| [SQLite](https://sqlite.org) via rusqlite's bundled build | Public domain | Compiled into the binary |
| Other Rust crates | Their own licences, listed in `Cargo.lock` | `cargo about` or `cargo license` prints the full list |

Architecture and engineering conventions are inspired by
[Spotifast](https://github.com/crmne/spotifast) and
[ZapFast](https://github.com/crmne/zapfast) (both MIT), by Carmine Paolino.
Wren copies no Spotify- or WhatsApp-specific code.

Microsoft, Microsoft Teams, Microsoft 365, Entra and Outlook are trademarks of
the Microsoft group of companies. Wren is an independent project and is not
affiliated with, endorsed by, or sponsored by Microsoft.
