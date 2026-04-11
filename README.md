# Eterea

Local-first X/Twitter bookmark manager built with Rust, Tauri, and Svelte.

## What it does
- imports bookmarks from CSV, JSON, X archive JS, or direct X sync
- stores everything locally in SQLite
- supports search, tags, favorites, date filters, and Focus / Grid / List views

## Dev
```bash
cd src/frontend && bun install
export ETEREA_X_CLIENT_ID=your_x_app_client_id # optional, for direct X import
cd ../..
cargo tauri dev
```

## Build
```bash
cd src/frontend && bun install
cd ../..
cargo tauri build
```

## Verify
```bash
cargo test --workspace
cargo check --workspace
cargo clippy --workspace --all-targets -- -A dead_code
cd src/frontend && bun test
cd src/frontend && bun run check
cd src/frontend && bun run build
```

## Notes
- direct X import requires `ETEREA_X_CLIENT_ID`
- data stays local after import
- database location follows the platform app-data directory used by Tauri
