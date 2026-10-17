#!/bin/bash
# Offline regression: rebuilds keep the same certificate-bound identity.
# Only synthetic certificates in a temporary keychain are used. No Wren
# accounts, tokens or login-keychain credentials are read.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
work="$(mktemp -d)"
keychain="$work/test.keychain-db"
trusted=0
cleanup() {
    if [ "$trusted" = 1 ]; then
        sudo security remove-trusted-cert -d "$work/cert.pem" >/dev/null 2>&1 || true
    fi
    security delete-keychain "$keychain" >/dev/null 2>&1 || true
    rm -rf "$work"
}
trap cleanup EXIT
umask 077

# Neither a missing identity nor an ad-hoc release may destroy an output.
mkdir "$work/Existing.app"
touch "$work/Existing.app/keep"
if CODESIGN_IDENTITY='' "$here/bundle.sh" /no/binary "$work/Existing.app" 0.1.0 >"$work/missing.log" 2>&1; then
    echo 'Bundling without a persistent identity unexpectedly succeeded.' >&2
    exit 1
fi
test -f "$work/Existing.app/keep"
for identity in '' -; do
    if CODESIGN_IDENTITY="$identity" "$root/packaging/release.sh" aarch64-apple-darwin 0.1.0 >"$work/release.log" 2>&1; then
        echo 'Release packaging without a persistent identity unexpectedly succeeded.' >&2
        exit 1
    fi
done
grep -q 'macOS releases require a persistent CODESIGN_IDENTITY' "$work/release.log"

cat > "$work/certificate.cnf" <<'EOF'
[req]
distinguished_name = subject
x509_extensions = extensions
prompt = no
[subject]
CN = Wren signing regression
[extensions]
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
basicConstraints = critical,CA:false
EOF
openssl req -new -newkey rsa:2048 -nodes -x509 -days 1 -config "$work/certificate.cnf" -keyout "$work/key.pem" -out "$work/cert.pem" >"$work/openssl.log" 2>&1
# Explicit algorithms also work with macOS's LibreSSL PKCS#12 importer.
openssl pkcs12 -export -inkey "$work/key.pem" -in "$work/cert.pem" -out "$work/identity.p12" -passout pass:fixture -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1
security create-keychain -p fixture "$keychain"
security unlock-keychain -p fixture "$keychain"
security import "$work/identity.p12" -k "$keychain" -P fixture -T /usr/bin/codesign >/dev/null
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k fixture "$keychain" >/dev/null
identity="$(openssl x509 -in "$work/cert.pem" -noout -fingerprint -sha1 | cut -d= -f2 | tr -d ':')"

# Hosted macOS runners reject an otherwise usable self-signed identity as
# CSSMERR_TP_NOT_TRUSTED. Trust only this fixture for code signing on the
# disposable runner, and remove it before deleting its temporary keychain.
# Never change trust settings on a contributor's Mac or a self-hosted runner.
if [ "${RUNNER_ENVIRONMENT:-}" = github-hosted ]; then
    trusted=1
    sudo security add-trusted-cert -d -r trustRoot -p codeSign -k "$keychain" "$work/cert.pem"
fi

# Different Mach-O contents and versions must still satisfy each other's DR.
for version in 1 2; do
    app="$work/Wren-$version.app"
    mkdir -p "$app/Contents/MacOS"
    sed "s/__VERSION__/$version.0.0/g" "$here/Info.plist" > "$app/Contents/Info.plist"
    printf 'int main(void) { return %s; }\n' "$version" | xcrun clang -isysroot "$(xcrun --sdk macosx --show-sdk-path)" -x c - -o "$app/Contents/MacOS/wren"
    if ! codesign --force --timestamp=none --options runtime --keychain "$keychain" --sign "$identity" "$app"; then
        # Only the synthetic fixture keychain is inspected, never real identities.
        security find-identity -p codesigning "$keychain"
        openssl x509 -in "$work/cert.pem" -noout -subject -dates
        exit 1
    fi
    codesign --verify --strict "$app"
    codesign -d -r- "$app" 2>&1 | sed -n 's/^designated => /designated => /p' > "$work/requirement-$version"
    test -s "$work/requirement-$version"
    # A bundle identifier alone would let an unrelated signer impersonate Wren.
    grep -Eq 'certificate|anchor' "$work/requirement-$version"
done
cmp "$work/requirement-1" "$work/requirement-2"
requirement="$(sed 's/^designated => //' "$work/requirement-1")"
codesign --verify --strict -R "=$requirement" "$work/Wren-2.app"

# The original ad-hoc fallback must fail that identity check.
codesign --force --sign - "$work/Wren-2.app"
if codesign --verify --strict -R "=$requirement" "$work/Wren-2.app" >"$work/adhoc.log" 2>&1; then
    echo 'An ad-hoc build unexpectedly satisfied the persistent identity.' >&2
    exit 1
fi
echo 'macOS signing regression passed: changed builds share a certificate-bound identity; ad-hoc builds do not.'
