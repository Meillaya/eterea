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

# Optional developer/QA fullscreen smoke check
ETEREA_WINDOW_MODE=fullscreen nix develop -c cargo run -p eterea-dioxus

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

- the default desktop window is frameless, maximized, resizable, and has no
  native app menu where the platform/Dioxus host supports menu removal
- web content reaches the webview client-area edges without a top title/menu gap
- `ETEREA_WINDOW_MODE=fullscreen` (or its `kiosk` alias) enters the explicit
  fullscreen path and can be exited/restored with the platform window controls
  or window-manager shortcut
- first launch and empty/onboarding state
- import dry-run and committed import paths
- search, author/topic filters, favorites, deletion, and detail routes
- all reading layouts
- remote media preview opt-in/off behavior

## Live desktop QA harness

Real release desktop QA must launch the Dioxus desktop app, not a unit test or
web-only surrogate. Use the harness below from a graphical desktop session:

```bash
scripts/desktop-qa.sh
```

The script starts the app in tmux with the real command
`tmux new-session -d -s eterea-desktop-qa 'cd /home/mei/projects/eterea && nix develop -c cargo run -p eterea-dioxus'`,
captures live logs with
`tmux capture-pane -pt eterea-desktop-qa -S -2000 > .omo/evidence/production-grade-project/desktop-tmux.log`,
records launch timing for the first usable desktop shell in
`.omo/evidence/production-grade-project/desktop-first-usable.txt`, and captures
a desktop screenshot with `gnome-screenshot -f .omo/evidence/production-grade-project/live-desktop.png`
or `import -window root .omo/evidence/production-grade-project/live-desktop.png`.

If neither `DISPLAY` nor `WAYLAND_DISPLAY` is set, the result is
`BLOCKED: live desktop screenshot unavailable`. The harness writes
`.omo/evidence/production-grade-project/desktop-display-blocker.md`; that
no-display blocker is not a pass and must keep production approval open until a
live desktop screenshot and first usable timing are captured on a real display.

## Desktop window modes

The Dioxus shell launches with an explicit desktop window configuration instead
of relying on framework defaults. The default mode is a frameless, maximized,
resizable window with the native menu disabled where supported. This removes the
top OS/window chrome while keeping Eterea a normal desktop app rather than a
kiosk.

Fullscreen is deliberately opt-in and intended as a developer/QA smoke-check
surface, not a packaged user preference:

```bash
ETEREA_WINDOW_MODE=fullscreen nix develop -c cargo run -p eterea-dioxus
```

The `kiosk` value is accepted as a fullscreen smoke-check alias for test
scripts. Both values use borderless fullscreen on the current monitor. This is a
manual smoke-check surface, not the default release mode; verify exit/restore
behavior on every target OS/window manager before presenting it as supported.
