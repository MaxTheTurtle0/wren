# App updates

Official portable downloads and macOS app bundles can update themselves.
In Settings, **App updates** controls automatic checks and downloads and
provides **Check for updates**, **Download update**, and **Restart to update**.
A ready update also shows a persistent restart button below the workspace.

Automatic checks contact GitHub at most once a day, including across app
restarts. They use no Microsoft credentials or account data. Switching the
setting off stops future automatic checks and downloads; manual checking remains
available. Demo windows do not check automatically.

Wren verifies an Ed25519 signature over the release's checksums, then verifies
the downloaded package against its checksum. The public key is compiled from
`packaging/update-public-key.hex`; no key supplied by a release is trusted.
The shared `fastframe-update` helper installs the update after Wren exits,
relaunches it, and restores the previous version if the new window does not
acknowledge a successful start. macOS replaces the whole bundle and also
checks its identity and any existing Developer ID signing team.

Wren never restarts automatically. Send or clear drafts and finish pending
sends before selecting **Restart to update**. This matters because drafts
and failed sends currently live in memory.

Package-manager installations must update through their package manager.
The updater also refuses unmarked Linux/Windows binaries and macOS copies
running from a disk image. Download and extract the official portable
archive, or move Wren.app into Applications. The Linux installer adds the
portable marker beside its installed executable. Source builds and older
CI artifacts do not opt into automatic binary replacement.

There is no public release feed until a maintainer publishes the first
GitHub release. A private repository or an empty release feed cannot serve
updates to normal users; Wren reports the failed check in Settings.

## Making a release

1. Keep Cargo.toml and Cargo.lock at the intended version. A release tag
   must match it exactly, such as `v0.1.0`.
2. Ensure the GitHub repository secret `UPDATE_SIGNING_KEY` holds the PEM
   Ed25519 private key matching `packaging/update-public-key.hex`. Keep a
   separate secure backup. Never commit the private key. The secret is
   already configured for this repository.
   For macOS, also configure `MACOS_SIGNING_CERTIFICATE` with the base64
   PKCS#12 export of the persistent code-signing certificate and private key,
   and `MACOS_SIGNING_PASSWORD` with its export password. Use a Developer ID
   Application identity for distribution and keep it consistent across
   releases. CI imports it into an isolated runner keychain and removes that
   keychain after packaging. Missing credentials stop macOS release packaging;
   it never falls back to an ad-hoc signature.
3. Push the version tag. `.github/workflows/release.yml` runs format, lint
   and tests, then builds the Linux x86_64, Windows x86_64, and macOS arm64
   packages. It checks that the signing key matches the compiled public key,
   signs `checksums.txt`, and creates a **draft** release.
4. Inspect the packages and release notes, complete the live-account and
   OSS acceptance checks, then publish the draft. This publication is what
   makes the update available to users.

Stable builds follow GitHub's latest stable release. Builds whose Cargo
version contains a prerelease suffix follow newer prereleases too.
macOS notarization and Windows publisher certificates remain separate from
the update-signing key. macOS release bundles require a persistent
`CODESIGN_IDENTITY`; ad-hoc signatures cannot preserve Keychain approval
across app updates. See [macOS signing](macos-keychain.md).

The package names, portable markers, version probe and restart protocol are
the shared fastframe-update contract. Keep `packaging/release.sh`, the
Windows packaging step, and the configuration in `src/updates.rs` together
when changing that contract. Do not replace the public key casually: existing
installations trust it and will refuse a release signed only by another key.
