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
chmod 755 "$app/Contents/MacOS/wren"
sed "s/__VERSION__/$numeric_version/g" "$here/Info.plist" > "$app/Contents/Info.plist"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
(cd "$root" && cargo run --quiet --release --example render-icon -- "$work/icon-1024.png" 1024)
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
