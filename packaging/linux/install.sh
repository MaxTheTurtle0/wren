#!/bin/sh
# Installs Wren for the current user: the binary in ~/.local/bin, the
# desktop entry and icons under ~/.local/share.
#
#   packaging/linux/install.sh [path/to/wren]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
binary="${1:-$here/../../target/release/wren}"
prefix="${PREFIX:-$HOME/.local}"

install -Dm755 "$binary" "$prefix/bin/wren"
install -Dm644 "$here/wren.desktop" "$prefix/share/applications/wren.desktop"
install -Dm644 "$here/wren-256.png" "$prefix/share/icons/hicolor/256x256/apps/wren.png"
install -Dm644 "$here/../icons/wren.svg" "$prefix/share/icons/hicolor/scalable/apps/wren.svg"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$prefix/share/applications" || true
echo "Installed Wren to $prefix/bin/wren"
