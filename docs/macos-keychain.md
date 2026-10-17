# macOS signing and Keychain prompts

Wren saves each account's refresh token in the login Keychain under
`app.wren.Wren`. macOS decides whether to allow access by checking the app's
code-signing designated requirement. Wren keeps using the same service and
account keys when its signing identity changes, so existing tokens remain
available after approval.

An ad-hoc signature identifies one exact build by its code-directory hash.
Replacing it with another build changes that identity, even at the same path
and with the same bundle identifier. Keychain then asks for permission again.
Selecting **Allow** grants access once; **Always Allow** saves approval for
the requesting identity. Repeated ad-hoc builds invalidate that approval.

## Build with a persistent identity

List the identities installed on the build Mac:

```sh
security find-identity -v -p codesigning
```

Set `CODESIGN_IDENTITY` to a Developer ID Application identity for distribution
or an Apple Development identity for local use. A certificate's SHA-1
fingerprint from the list also works. Keep using the same identity for later
builds.

```sh
export CODESIGN_IDENTITY='Developer ID Application: Your Name (TEAMID)'
cargo build --locked --release
packaging/macos/bundle.sh target/release/wren Wren.app 0.1.0
```

The bundler stops before changing the output if no identity is supplied.
`CODESIGN_KEYCHAIN` optionally selects a separate signing keychain, as used
by the [release workflow](updates.md). The signature keeps the default
certificate-bound designated requirement. Do not replace it with a requirement
that checks only Wren's bundle identifier, or allow every application to read
Wren's Keychain entries.

When switching an existing installation from ad-hoc signing to a persistent
identity, the first read of each saved account's token may still prompt.
Choose **Always Allow** for that signed Wren. Later builds with the same
identity retain approval. A locked login Keychain can still require unlocking.
Wren does not collect the Keychain password or change the account's tokens
to work around the prompt.

For offline demos and disposable CI artifacts, explicitly set
`CODESIGN_IDENTITY=-`. These builds warn that approval will be lost after a
rebuild. Release packaging rejects this setting. Bare `cargo run` builds also
lack a persistent signing identity; use the signed bundle for real accounts.

`packaging/macos/test-signing.sh` builds two different native fixtures, signs
them with a synthetic certificate, and verifies that each satisfies the same
certificate-bound requirement. It also verifies that an ad-hoc replacement
fails that check. The test uses a temporary keychain and no Wren credentials.
On disposable GitHub-hosted runners, it temporarily adds the fixture keychain
to the user search list and restores the original list during cleanup. Local
and self-hosted runs leave the search list alone. No trust settings are changed.

Apple documents [Keychain tracking and designated requirements](https://developer.apple.com/library/archive/technotes/tn2206/_index.html)
and [signing identity setup](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html).
