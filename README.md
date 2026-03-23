# Eterea

Lightning-fast Twitter/X bookmarks manager built with Rust and Tauri.

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

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `/` | Focus search |
| `Esc` | Clear search |

## Database Location

- **Windows**: `%LOCALAPPDATA%\eterea\bookmarks.db`
- **macOS**: `~/Library/Application Support/eterea/bookmarks.db`
- **Linux**: `~/.local/share/eterea/bookmarks.db`


MIT
