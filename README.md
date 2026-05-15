# Eterea

Eterea is a local-first desktop archive for X/Twitter bookmarks. It imports
exported bookmark data, stores it in a local SQLite database, and provides a
fast Rust/Dioxus reading room for search, filtering, favorites, and long-form
review.

## Current status

Eterea is a production-candidate desktop app. The active runtime is the Dioxus
shell in `src/dioxus-app`, backed by shared application services in `src/app`
and SQLite/storage/search code in `src/backend`. Historical OMX planning output,
older frontend notes, and generated design exports are intentionally kept out of
the root product surface.

## Features

- Import CSV, JSON, and X archive JavaScript bookmark exports.
- Preview imports with a dry-run path before writing to the archive.
- Store data locally in SQLite under the platform app-data directory.
- Search by text and filter by author, date, media, favorites, and topics.
- Browse editorial reading layouts: Issue, Front Page, Long-Read, and Spread.
- Use data-backed entry detail, author directory, topic directory, and filtered
  author/topic archive routes.
- Keep remote tweet image previews hidden by default; enabling previews is a
  session-only choice that may request stored HTTPS media URLs.

## Repository layout

```text
src/backend/     Core Rust library: ingestion, storage, search, models, CLI.
src/app/         Product service layer shared by desktop UI and tests.
src/dioxus-app/  Active Dioxus desktop application shell and CSS.
scripts/         Developer/maintenance binaries wired into the workspace.
docs/            Architecture, development, operations, and historical notes.
fixtures/        Non-production fixtures used by tests and benchmarks.
```

See `docs/architecture.md` for the crate boundaries and app flow.

## Requirements

The supported development environment is Nix. Plain `cargo` may fail on hosts
without system OpenSSL/pkg-config headers even though the Nix environment works.

```bash
nix develop
```

## Run

```bash
nix develop -c cargo run -p eterea-dioxus
```

## Build

```bash
nix develop -c cargo build --workspace
nix develop -c cargo build -p eterea-dioxus --release
```

## Verify

Run these before proposing or shipping changes:

```bash
nix develop -c cargo fmt --all -- --check
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo test --workspace
```

Additional release evidence and manual checks are listed in
`docs/operations/release-readiness.md`.

## Documentation

- Documentation index: `docs/README.md`
- Architecture and repository layout: `docs/architecture.md`
- Development workflow: `docs/development.md`
- Implemented UI design system: `docs/design-system.md`
- Release readiness: `docs/operations/release-readiness.md`
- Historical implementation notes: `docs/archive/`

## Data and privacy model

- Eterea is local-first: imported bookmark data is written to local SQLite.
- The default database path is `dirs::data_local_dir()/eterea/bookmarks.db`.
- Direct X sync and browser companion/server mode are not part of the current
  desktop release.
- Remote media previews are opt-in per session because they may perform network
  requests for stored HTTPS image URLs.

## License

MIT. See `LICENSE`.
