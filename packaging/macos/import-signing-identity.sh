#!/bin/bash
# Import the release identity from GitHub secrets into an isolated runner
# keychain. Reuse the same certificate across releases; never generate it here.
set -euo pipefail

: "${MACOS_SIGNING_CERTIFICATE:?Set the MACOS_SIGNING_CERTIFICATE repository secret to the base64 PKCS#12 identity}"
: "${MACOS_SIGNING_PASSWORD:?Set the MACOS_SIGNING_PASSWORD repository secret to its export password}"
: "${RUNNER_TEMP:?}"
: "${GITHUB_ENV:?}"

umask 077
work="$(mktemp -d "$RUNNER_TEMP/wren-signing.XXXXXX")"
certificate="$work/identity.p12"
keychain="$work/signing.keychain-db"
ready=0
cleanup() {
    rm -f "$certificate"
    if [ "$ready" != 1 ]; then
        security delete-keychain "$keychain" >/dev/null 2>&1 || true
    fi
}
trap cleanup EXIT
printf '%s' "$MACOS_SIGNING_CERTIFICATE" | base64 --decode > "$certificate"
keychain_password="$(openssl rand -hex 32)"
security create-keychain -p "$keychain_password" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$keychain_password" "$keychain"
security import "$certificate" -k "$keychain" -P "$MACOS_SIGNING_PASSWORD" -T /usr/bin/codesign >/dev/null
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$keychain_password" "$keychain" >/dev/null
identities="$(security find-identity -v -p codesigning "$keychain" | sed -nE 's/^[[:space:]]*[0-9]+\) ([0-9A-F]{40}) .*/\1/p')"
if [ -z "$identities" ] || [ "$(printf '%s\n' "$identities" | wc -l | tr -d ' ')" != 1 ]; then
    echo 'The macOS signing certificate must contain exactly one valid code-signing identity.' >&2
    exit 1
fi
printf 'CODESIGN_IDENTITY=%s\nCODESIGN_KEYCHAIN=%s\n' "$identities" "$keychain" >> "$GITHUB_ENV"
ready=1
