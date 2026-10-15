#!/bin/bash
# Packages the exact filenames/layout consumed by fastframe-update.
set -euo pipefail
target="$1"
version="$2"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
name="wren-v$version-$target"
mkdir -p dist
if [ "$target" = aarch64-apple-darwin ]; then
    mkdir -p dist/macos
    packaging/macos/bundle.sh "target/$target/release/wren" dist/macos/Wren.app "$version"
    ln -s /Applications dist/macos/Applications
    hdiutil create -volname Wren -srcfolder dist/macos -ov -format UDZO "dist/wren-v$version-macos-arm64.dmg"
else
    mkdir -p "dist/$name/assets/fonts" "dist/$name/assets/icons"
    cp "target/$target/release/wren" packaging/wren-portable.txt LICENSE THIRD-PARTY-NOTICES.md "dist/$name/"
    cp assets/fonts/Inter-LICENSE.txt "dist/$name/assets/fonts/"
    cp assets/icons/LICENSE.txt "dist/$name/assets/icons/"
    cp packaging/linux/wren.desktop packaging/linux/wren-256.png packaging/linux/install.sh packaging/icons/wren.svg "dist/$name/"
    tar -C dist -czf "dist/$name.tar.gz" "$name"
fi
