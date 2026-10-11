# Contributing

Thanks for helping. Read [AGENTS.md](AGENTS.md) first: it holds the product
boundaries (official APIs only, no polling, least privilege) and the
architecture rules.

## Setup

- Rust 1.98 or newer (`rust-toolchain.toml` pins it for rustup).
- Linux: `libxkbcommon-dev libwayland-dev libgl1-mesa-dev` (Debian and
  Ubuntu names), plus a Secret Service provider (GNOME Keyring or KWallet)
  for signing in.
- macOS: Xcode command line tools.
- Windows: the MSVC toolchain.

```sh
cargo run -- --demo        # sample data, no account needed
cargo run                  # real account (see docs/azure-app-registration.md)
```

## Checks

Run before every push:

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps
```

The tests need no network and no account. Graph is a local `wiremock` server,
and the interface runs headless over the demo data.

## Screenshots

```sh
cargo run --release -- --screenshot shot.png --page channel,dark --size 1280x800
```

Pages: `chats`, `group`, `channel`, `calendar`, `calls`, `switcher`,
`settings`, `shortcuts`, `signin`, `setup`, `perf`. Add `light` or `dark`.
On macOS, run the bundle (`packaging/macos/bundle.sh`) through
`open -W -n Wren.app --args …`, because a bare binary launched over SSH
starts slowly.

## Pull requests

- One topic per pull request, with tests for changed behaviour.
- For interface changes, attach before and after screenshots in both themes.
- Do not add Graph permissions, background fetching or new dependencies
  without explaining why in the pull request.
