# Development

## Environment

Use the Nix dev shell as the canonical environment:

```bash
nix develop
```

The shell provides the native libraries needed by the Rust workspace. Running
plain `cargo test --workspace` outside Nix can fail on hosts without OpenSSL
pkg-config metadata.

## Common commands

```bash
# Run the desktop app
nix develop -c cargo run -p eterea-dioxus

# Fast compile check
nix develop -c cargo check --workspace

# Formatting, linting, and tests
nix develop -c cargo fmt --all -- --check
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo test --workspace

# Release build smoke check
nix develop -c cargo build -p eterea-dioxus --release
```

## Change hygiene

- Prefer small, reversible changes.
- Add or update tests before changing behavior.
- Do not add dependencies without a clear reason and review.
- Keep `Cargo.lock` committed because this workspace ships binaries.
- Keep `.omx/`, `.omc/`, local databases, and generated standalone exports out
  of version control.

## Manual smoke checks

Before a public release, run the desktop app and verify:

- first launch and empty/onboarding state
- import dry-run and committed import paths
- search, author/topic filters, favorites, deletion, and detail routes
- all reading layouts
- remote media preview opt-in/off behavior
