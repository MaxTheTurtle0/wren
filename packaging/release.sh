#!/bin/bash
# Packages the exact filenames/layout consumed by fastframe-update.
set -euo pipefail
target="$1"
version="$2"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
if [ "$target" = aarch64-apple-darwin ] && { [ -z "${CODESIGN_IDENTITY:-}" ] || [ "$CODESIGN_IDENTITY" = - ]; }; then
    echo 'macOS releases require a persistent CODESIGN_IDENTITY; ad-hoc signing breaks saved Keychain approval on update.' >&2
    exit 1
fi
name="wren-v$version-$target"
mkdir -p dist
if [ "$target" = aarch64-apple-darwin ]; then
    mkdir -p dist/macos
    packaging/macos/bundle.sh "target/$target/release/wren" dist/macos/Wren.app "$version"
    ln -s /Applications dist/macos/Applications
    hdiutil create -volname Wren -srcfolder dist/macos -ov -format UDZO "dist/wren-v$version-macos-arm64.dmg"
else
    mkdir -p "dist/$name"
    cp "target/$target/release/wren" packaging/wren-portable.txt "dist/$name/"
    packaging/copy-notices.sh "dist/$name"
    cp packaging/linux/wren.desktop packaging/linux/wren-256.png packaging/linux/install.sh packaging/icons/wren.svg "dist/$name/"
    tar -C dist -czf "dist/$name.tar.gz" "$name"
fi
