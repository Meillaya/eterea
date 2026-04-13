# Eterea

Local-first X/Twitter bookmark manager built with Rust and Dioxus.

## What it does
- imports bookmarks from CSV, JSON, or X archive JS files
- stores everything locally in SQLite
- supports search, top-tag filtering, favorites, and Focus / Grid / List views

## Dev
```bash
cargo run -p eterea-dioxus
```

## Build
```bash
cargo build --workspace
```

## Verify
```bash
cargo test --workspace
cargo check --workspace
cargo clippy --workspace --all-targets -- -A dead_code
cargo build -p eterea-dioxus
```

## Notes
- the desktop MVP keeps the existing local SQLite storage
- data stays local after import
- the database location follows the platform app-data directory from the Rust backend (`dirs::data_local_dir()/eterea/bookmarks.db`)
- direct X sync remains deferred for this first Dioxus pass
- browser companion/server mode remains a planned follow-on phase
