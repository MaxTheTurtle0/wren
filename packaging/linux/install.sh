#!/bin/sh
# Installs Wren for the current user: the binary in ~/.local/bin, the
# desktop entry and icons under ~/.local/share.
#
#   packaging/linux/install.sh [path/to/wren]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$here/wren" ]; then
    binary="${1:-$here/wren}"
    icon="$here/wren.svg"
else
    binary="${1:-$here/../../target/release/wren}"
    icon="$here/../icons/wren.svg"
fi
prefix="${PREFIX:-$HOME/.local}"

install -Dm755 "$binary" "$prefix/bin/wren"
# Marks this user-owned copy for verified self-updates. Package managers
# never run this installer and remain responsible for their own updates.
printf '%s\n' wren-portable-v1 > "$prefix/bin/wren-portable.txt"
install -Dm644 "$here/wren.desktop" "$prefix/share/applications/wren.desktop"
install -Dm644 "$here/wren-256.png" "$prefix/share/icons/hicolor/256x256/apps/wren.png"
install -Dm644 "$icon" "$prefix/share/icons/hicolor/scalable/apps/wren.svg"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$prefix/share/applications" || true
echo "Installed Wren to $prefix/bin/wren"
