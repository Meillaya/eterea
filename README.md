# Eterea

Lightning-fast Twitter/X bookmarks manager built with Rust and Tauri.

![Eterea Screenshot](docs/screenshot.png)

## Features

- **Fast**: Rust backend with SQLite FTS5 for sub-10ms search
- **Easy Import**: CSV (Dewey, Twitter exports) and JSON formats
- **Full-text Search**: Instant search across all bookmarks
- **Smart Tags**: Automatic tag detection and filtering
- **Local First**: All data stored locally, nothing leaves your machine

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Bun](https://bun.sh/)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)
- C compiler (`gcc` or `clang`)

### NixOS / Nix Users

```bash
nix-shell
# or with flakes
nix develop
```

### Development

```bash
cd src/frontend && bun install
cd ../.. && cargo tauri dev
```

### Import Bookmarks

```bash
# Import all legacy files
cargo run -p eterea-core --bin migrate -- --all

# Import specific file
cargo run -p eterea-core --bin migrate -- src/legacy/legacy_bookmarks.csv

# Dry run
cargo run -p eterea-core --bin migrate -- --dry-run --all
```

### Build

```bash
cargo tauri build
```

Output: `target/release/bundle/`

## Project Structure

```
eterea/
├── src/
│   ├── backend/           # Rust core (models, ingestion, storage, search)
│   ├── frontend/          # Svelte 5 UI
│   └── legacy/            # Sample bookmark files
├── src-tauri/             # Tauri application
└── Cargo.toml
```

## Supported Formats

**CSV (Dewey)**: `Tweet Date,Posted By,Profile URL,Twitter Handle,Tweet URL,Content,Tags,Media`

**CSV (Twitter/X)**: `profile_image_url_https,screen_name,name,full_text,tweeted_at,tweet_url`

**JSON**: Standard Twitter API format

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `/` | Focus search |
| `Esc` | Clear search |

## Database Location

- **Windows**: `%LOCALAPPDATA%\eterea\bookmarks.db`
- **macOS**: `~/Library/Application Support/eterea/bookmarks.db`
- **Linux**: `~/.local/share/eterea/bookmarks.db`

## Tech Stack

Rust, SQLite FTS5, Svelte 5, TailwindCSS 4, Tauri 2.0

## License

MIT
