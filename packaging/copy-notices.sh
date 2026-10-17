#!/bin/sh
# Use the same complete notice set in every archive, app bundle and relay.
set -eu
root="$(cd "$(dirname "$0")/.." && pwd)"
destination="$1"
mkdir -p "$destination/assets/fonts" "$destination/assets/icons"
cp "$root/LICENSE" "$root/THIRD-PARTY-NOTICES.md" "$destination/"
cp -R "$root/licenses" "$destination/"
cp "$root/assets/fonts/Inter-LICENSE.txt" "$destination/assets/fonts/"
cp "$root/assets/icons/LICENSE.txt" "$destination/assets/icons/"
