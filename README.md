# Eterea 🌟

> Lightning-fast Twitter/X bookmarks manager

Eterea is a beautiful, performant desktop application for managing your Twitter/X bookmarks. Import your bookmarks from CSV or JSON exports, and enjoy instant search and browsing through a clean, modern interface.

![Eterea Screenshot](docs/screenshot.png)

## ✨ Features

- **⚡ Lightning Fast**: Rust-powered backend with SQLite FTS5 for sub-10ms search
- **📥 Easy Import**: Support for CSV (Dewey, Twitter exports) and JSON formats
- **🔍 Instant Search**: Full-text search across all your bookmarks
- **🏷️ Smart Tags**: Automatic tag detection and filtering
- **🎨 Beautiful UI**: Clean, modern interface built with Svelte 5
- **💾 Local First**: All data stored locally on your machine
- **🔒 Private**: No data leaves your computer

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Bun](https://bun.sh/) (for frontend)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)
- C compiler (`gcc` or `clang`)

### NixOS / Nix Users

```bash
# Enter the development shell
nix-shell

# Or with flakes
nix develop
```

The `shell.nix` provides all necessary dependencies including:
- Rust toolchain
- GCC/linker
- SQLite
- Bun/Node.js
- Tauri GTK dependencies

### Development

```bash
# Install frontend dependencies
cd src/frontend
bun install

# Run in development mode (from project root)
cd ../..
cargo tauri dev
```

### Import Your Bookmarks

```bash
# Using the migration script
cargo run -p eterea-core --bin migrate -- --all

# Or import a specific file
cargo run -p eterea-core --bin migrate -- src/legacy/legacy_bookmarks.csv

# Dry run (parse only, don't save)
cargo run -p eterea-core --bin migrate -- --dry-run --all
```

### Build for Production

```bash
cargo tauri build
```

The built application will be in `target/release/bundle/`.

## 📁 Project Structure

```
eterea/
├── src/
│   ├── backend/           # Rust core library
│   │   ├── src/
│   │   │   ├── models/    # Data models (Bookmark, Media, etc.)
│   │   │   ├── ingestion/ # CSV/JSON parsers
│   │   │   ├── storage/   # SQLite database layer
│   │   │   └── search/    # Search utilities
│   │   └── Cargo.toml
│   │
│   ├── frontend/          # Svelte 5 UI
│   │   ├── src/
│   │   │   ├── lib/
│   │   │   │   ├── components/  # UI components
│   │   │   │   ├── stores/      # State management
│   │   │   │   └── api.ts       # Tauri IPC bridge
│   │   │   └── routes/
│   │   └── package.json
│   │
│   └── legacy/            # Sample bookmark files
│
├── src-tauri/             # Tauri application
│   ├── src/
│   │   ├── lib.rs         # Tauri commands
│   │   └── main.rs
│   └── tauri.conf.json
│
└── Cargo.toml             # Workspace config
```

## 📊 Supported Formats

### CSV (Dewey Export)
```csv
Tweet Date,Posted By,Posted By Profile Pic,Profile URL,Twitter Handle,Tweet URL,Content,Tags,Comments,Media
```

### CSV (Twitter/X Export)
```csv
profile_image_url_https,screen_name,name,full_text,note_tweet_text,tweeted_at,tweet_url
```

### JSON
Standard Twitter API format with `full_text`, `screen_name`, `created_at`, etc.

## ✨ Features

### Favorites
- Click the ★ star icon on any bookmark to add/remove from favorites
- Filter to show only favorites using the sidebar

### Date Filtering
- Use the date picker in the top bar to filter by date range
- Quick presets: Today, Last 7 days, Last 30 days, etc.
- Custom date range selection

### Search
- Full-text search across all bookmark content
- Filter by tags using the sidebar
- Combine search with date filters

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `/` | Focus search |
| `Esc` | Clear search |

## 🔧 Configuration

The database is stored at:
- **Windows**: `%LOCALAPPDATA%\eterea\bookmarks.db`
- **macOS**: `~/Library/Application Support/eterea/bookmarks.db`
- **Linux**: `~/.local/share/eterea/bookmarks.db`

## 🛠️ Tech Stack

- **Backend**: Rust, SQLite with FTS5
- **Frontend**: Svelte 5, TailwindCSS 4
- **Desktop**: Tauri 2.0
- **Performance**: Sub-100ms queries, 60fps UI

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

---

Built with 🦀 + ⚡ by developers who love their bookmarks.

