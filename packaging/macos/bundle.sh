#!/bin/bash
# Builds Wren.app from a release binary.
#
#   packaging/macos/bundle.sh <binary> <output.app> <version>
#
# The icon is rendered from packaging/icons/wren.svg by the render-icon
# example, then turned into an .icns with sips and iconutil. Set
# CODESIGN_IDENTITY to sign with a Developer ID; otherwise the bundle gets
# the ad-hoc signature Apple Silicon requires.
set -euo pipefail

binary="$1"
app="$2"
version="$3"
numeric_version="${version%%-*}"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$binary" "$app/Contents/MacOS/wren"
mkdir -p "$app/Contents/Resources/assets/fonts" "$app/Contents/Resources/assets/icons"
cp "$root/LICENSE" "$root/THIRD-PARTY-NOTICES.md" "$app/Contents/Resources/"
cp "$root/assets/fonts/Inter-LICENSE.txt" "$app/Contents/Resources/assets/fonts/"
cp "$root/assets/icons/LICENSE.txt" "$app/Contents/Resources/assets/icons/"
chmod 755 "$app/Contents/MacOS/wren"
sed "s/__VERSION__/$numeric_version/g" "$here/Info.plist" > "$app/Contents/Info.plist"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
if [ -n "${WREN_ICON_PNG:-}" ]; then
    # The same SVG rendered on a build host, without compiling examples
    # and their dev dependencies again on a constrained signing host.
    cp "$WREN_ICON_PNG" "$work/icon-1024.png"
else
    (cd "$root" && cargo run --quiet --locked --release --example render-icon -- "$work/icon-1024.png" 1024)
fi
iconset="$work/wren.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
    sips -z $size $size "$work/icon-1024.png" --out "$iconset/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z $double $double "$work/icon-1024.png" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/wren.icns"

if [ -n "${CODESIGN_IDENTITY:-}" ]; then
    codesign --force --timestamp --options runtime --sign "$CODESIGN_IDENTITY" "$app"
else
    codesign --force --sign - "$app"
fi
codesign --verify --strict "$app"
echo "$app"
