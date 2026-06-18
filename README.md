# Eterea

**Eterea is a local-first Rust/Dioxus desktop archive for X/Twitter bookmarks:**
import exported bookmarks, keep them in local SQLite, and read them through a
fast editorial desktop interface.

Status: [CI](https://github.com/Meillaya/eterea/actions/workflows/ci.yml) · [MIT License](LICENSE) · [Rust workspace](Cargo.toml) ·
[Dioxus UI](src/dioxus-app/) · [release: local artifacts / draft workflow planned](docs/operations/release-readiness.md) · [privacy: local-first](#privacy-model)

## Features

- Import CSV, JSON, and X archive JavaScript bookmark exports with dry-run
  previews before writing.
- Store the archive locally in SQLite under the platform app-data directory.
- Search text and filter by author, topic, date, media, and favorites.
- Read through editorial layouts: Issue, Front Page, Long-Read, and Spread.
- Browse data-backed entry detail, author directory, topic directory, and
  filtered archive routes.
- Keep remote tweet image previews hidden by default; enabling them is a
  session-only choice that may request stored HTTPS media URLs.

## Privacy model

Eterea is local-first: imported bookmark data stays on your machine in local
SQLite. Direct X sync, browser companion/server mode, and background remote media
fetching are not part of this desktop release surface. Public preview assets must
use synthetic or local fixture data only; see the
[preview asset contract](docs/assets/previews/README.md).

Treat every imported bookmark field as untrusted user-controlled data, including
tweet URLs, content, note text, author handles/names, tags, comments, profile
URLs, and media URLs. The UI renders text through Dioxus text nodes rather than
raw HTML, remote image previews stay off by default, previewable media must be
HTTPS, and external URL opening is limited to explicit HTTPS user actions.
Generated artifacts under `.omo/`, `.omx/`, `target/`, and preview-generation
scratch paths are also untrusted until reviewed; do not publish them as README
assets unless they are synthetic/local-fixture outputs recorded in the preview
provenance log.

## Quick start

The supported development environment is Nix because the workspace needs native
system libraries such as OpenSSL/pkg-config.

```bash
nix develop
nix develop -c cargo run -p eterea-dioxus
```

## Verify

Run CI-equivalent Rust gates plus the local docs-link checker before proposing or shipping changes:

```bash
nix develop -c cargo fmt --all -- --check
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo test --workspace
nix develop -c cargo build -p eterea-dioxus --release
python scripts/check-doc-links.py README.md 'docs/**/*.md'
```

## Release and workflow status

- CI exists at [`.github/workflows/ci.yml`](.github/workflows/ci.yml) and covers
  the Rust format, lint, test, release build, and performance baseline gates.
- The docs-link checker remains a local verification step via
  `python scripts/check-doc-links.py README.md 'docs/**/*.md'`.
- Public release packaging is not claimed complete: signed installers,
  notarization, live desktop screenshots, and percentile performance evidence
  remain release-owner follow-ups.
- Current release gates, waivers, and evidence classes are documented in
  [release readiness](docs/operations/release-readiness.md).

## Documentation

- [Documentation index](docs/README.md)
- [Architecture](docs/architecture.md)
- [Development workflow](docs/development.md)
- [Design system](docs/design-system.md)
- [Preview asset contract](docs/assets/previews/README.md)
- [Release readiness](docs/operations/release-readiness.md)

## License

MIT. See [LICENSE](LICENSE).
