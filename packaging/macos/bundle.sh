#!/bin/bash
# Builds Wren.app from a release binary.
#
#   packaging/macos/bundle.sh <binary> <output.app> <version>
#
# The icon is rendered from packaging/icons/wren.svg by the render-icon
# example, then turned into an .icns with sips and iconutil. Set
# CODESIGN_IDENTITY to a persistent Developer ID or Apple Development
# identity. Ad-hoc signing is only available explicitly, with
# CODESIGN_IDENTITY=-, for offline demos and disposable CI artifacts.
set -euo pipefail

binary="$1"
app="$2"
version="$3"
numeric_version="${version%%-*}"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"

# Keychain ACLs track the app's designated requirement. An ad-hoc signature
# uses this build's cdhash, so every rebuild loses access to saved tokens.
# Check before touching the output bundle, including an existing installation.
if [ -z "${CODESIGN_IDENTITY:-}" ]; then
    echo 'Set CODESIGN_IDENTITY to a persistent code-signing identity (security find-identity -v -p codesigning).' >&2
    echo 'For offline demos only, explicitly set CODESIGN_IDENTITY=-. Ad-hoc builds ask for Keychain access again after updates.' >&2
    exit 1
fi

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

if [ "$CODESIGN_IDENTITY" = - ]; then
    echo 'Warning: ad-hoc Wren build. Keychain approval will not survive a rebuild. Use a persistent CODESIGN_IDENTITY for signed-in use.' >&2
    codesign --force --sign - "$app"
else
    signing_args=(--force --timestamp --options runtime --sign "$CODESIGN_IDENTITY")
    if [ -n "${CODESIGN_KEYCHAIN:-}" ]; then
        signing_args+=(--keychain "$CODESIGN_KEYCHAIN")
    fi
    codesign "${signing_args[@]}" "$app"
fi
codesign --verify --strict "$app"
echo "$app"
