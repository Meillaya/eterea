# Eterea

Eterea is a local-first Twitter/X bookmarks manager built with Rust, Tauri, and Svelte.

It can:
- import bookmark exports from CSV, JSON, and X archive JS files
- organize and search bookmarks locally at high speed
- favorite, filter, and browse bookmarks with a desktop UI
- optionally import bookmarks directly from X using OAuth 2.0 PKCE

---

## Current Feature Set

### Local library
- Local SQLite bookmark database
- Full-text search
- Favorites
- Tag filtering
- Date filtering
- Stats dashboard
- Link previews
- Card / compact layout modes

### Import sources
- Legacy CSV bookmark exports
- Twitter/X CSV bookmark exports
- JSON bookmark exports
- X archive JS exports (`window.YTD...` style)
- Direct X bookmark import via API

### X import flow
- End-user sign-in via browser-based OAuth 2.0 PKCE
- Manual one-time import from X
- Manual re-sync from X
- Local-first persistence after import
- Sync status metadata and last-sync status in app

### Intentionally out of scope in the current version
- Posting to X
- Adding/removing bookmarks on X
- Background sync
- Multi-account X support
- Cloud-first bookmark storage

---

## Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [Bun](https://bun.sh/)
- C compiler (`gcc` or `clang`)

### Install the Tauri CLI

If `cargo tauri` is not available, install the CLI first:

```bash
cargo install tauri-cli --version "^2.0.0" --locked
```

Then verify it:

```bash
cargo tauri --version
```

### Nix / NixOS

```bash
nix-shell
# or
nix develop
```

---

## Development

```bash
cd src/frontend && bun install
export ETEREA_X_CLIENT_ID=your_x_app_client_id
# optional if your X app uses a different localhost callback
# export ETEREA_X_REDIRECT_URI=http://127.0.0.1:38347/callback
cd ../..
cargo tauri dev
```

### Direct X import configuration

To enable direct X import in the desktop app:

- set `ETEREA_X_CLIENT_ID`
- optionally set `ETEREA_X_REDIRECT_URI`
- register the same redirect URI in your X app configuration

Default redirect URI used by Eterea:

```text
http://127.0.0.1:38347/callback
```

Notes:
- end users do **not** need their own developer account
- Eterea stores imported bookmarks locally after sync
- current sync is session-based, so some re-syncs may require signing in again

---

## Importing Bookmarks

### From files inside the app
Use the Import modal and choose:
- CSV
- JSON
- X archive JS

### From X directly
Use **Import from X** in the Import modal.

The app will:
1. open X login in your browser
2. complete OAuth in the desktop app
3. fetch bookmarks from X
4. import them into the local Eterea database

### From CLI / migration tool

```bash
# Import all legacy files
cargo run -p eterea-core --bin migrate -- --all

# Import a specific file
cargo run -p eterea-core --bin migrate -- path/to/bookmarks.csv

# Dry run
cargo run -p eterea-core --bin migrate -- --dry-run --all
```

---

## Build

```bash
cd src/frontend && bun install
cd ../..
cargo tauri build
```

Output bundle:

```text
target/release/bundle/
```

---

## Bookmark load-performance notes

For the active load-path performance and stats-reliability workstream, see [`docs/bookmark-load-performance.md`](docs/bookmark-load-performance.md). It captures the approved scope, review guardrails, and verification checklist for this area.

## Verification Commands

Current recommended checks:

```bash
cargo test --workspace
cargo check --workspace
cargo clippy --workspace --all-targets -- -A dead_code
cd src/frontend && bun run check
cd src/frontend && bun run build
```

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `/` | Focus search |
| `Esc` | Clear search |

---

## Database Location

- **Windows**: `%LOCALAPPDATA%\eterea\bookmarks.db`
- **macOS**: `~/Library/Application Support/eterea/bookmarks.db`
- **Linux**: `~/.local/share/eterea/bookmarks.db`

---

## Project Structure

```text
src/backend/     Rust core library: ingestion, storage, search
src-tauri/       Tauri desktop shell and X API integration
src/frontend/    Svelte UI
scripts/         migration/import helpers
```

---

## Current Operational Notes

- X import only works when the app is configured with a valid X client ID
- X API availability, rate limits, and token expiry can affect direct import
- File import remains the fallback path if X auth or API access is unavailable
