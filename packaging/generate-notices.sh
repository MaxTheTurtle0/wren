#!/bin/bash
# Refresh the committed license texts, or check them without changing files.
set -euo pipefail
cd "$(dirname "$0")/.."
mode="${1:-}"
if [ "$#" -gt 1 ] || { [ -n "$mode" ] && [ "$mode" != --check ]; }; then
    echo 'Usage: packaging/generate-notices.sh [--check]' >&2
    exit 1
fi
if [ "$(cargo about --version)" != 'cargo-about 0.9.2' ]; then
    echo 'Install the pinned generator: cargo install cargo-about --locked --version 0.9.2' >&2
    exit 1
fi
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/licenses/fonts"
cargo about generate --locked --workspace --fail packaging/licenses.hbs -o "$work/licenses/dependencies.txt"
cargo metadata --locked --format-version 1 > "$work/metadata.json"
python3 - "$work" <<'PY'
import json
import re
import shutil
import sys
import urllib.request
from pathlib import Path

work = Path(sys.argv[1])
packages = json.loads((work / "metadata.json").read_text())["packages"]
report = (work / "licenses/dependencies.txt").read_text()
# dispatch 0.2.0 declares MIT and its author in Cargo.toml but publishes no
# standalone license text, even in its source repository. Every other crate
# must retain upstream license texts rather than fall back to SPDX templates.
fallbacks = set(re.findall(r"SPDX template attribution: (.+)", report))
if fallbacks - {"dispatch 0.2.0"}:
    raise SystemExit(f"Missing upstream license notices: {sorted(fallbacks)}")
fonts = next(p for p in packages if p["name"] == "epaint_default_fonts")
source = Path(fonts["manifest_path"]).parent / "fonts"
for upstream, notice in {
    "Hack-Regular.txt": "Hack-Regular.txt",
    "OFL.txt": "NotoEmoji-OFL.txt",
    "UFL.txt": "Ubuntu-UFL.txt",
    "emoji-icon-font-mit-license.txt": "emoji-icon-font-MIT.txt",
}.items():
    shutil.copyfile(source / upstream, work / "licenses/fonts" / notice)

# Newer dpi includes libm code under an additional MIT notice. Preserve it
# even though dpi's package-level license allows choosing Apache-2.0.
for package in packages:
    root = Path(package["manifest_path"]).parent
    if package["name"] == "dpi" and (root / "LICENSE-LIBM-MIT").is_file():
        shutil.copyfile(root / "LICENSE-LIBM-MIT", work / "licenses/dpi-libm-MIT.txt")
    if package["name"] == "objc2-core-foundation":
        revision = json.loads((root / ".cargo_vcs_info.json").read_text())["git"]["sha1"]
        url = f"https://raw.githubusercontent.com/madsmtm/objc2/{revision}/LICENSE.md"
        with urllib.request.urlopen(url, timeout=30) as response:
            (work / "licenses/objc2-license.md").write_bytes(response.read())
PY
if [ "$mode" = --check ]; then
    diff -ru licenses "$work/licenses"
else
    mkdir -p licenses/fonts
    cp "$work/licenses/"*.* licenses/
    cp "$work/licenses/fonts/"*.txt licenses/fonts/
fi
