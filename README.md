# Eterea

Local-first X/Twitter bookmark manager built with Rust and Dioxus.

## What it does
- imports bookmarks from CSV, JSON, or X archive JS files
- stores everything locally in SQLite
- supports dry-run import preview, search, author/date/media filtering, top-tag filtering, favorites, delete, and editorial Issue / Front Page / Long-Read / Spread views
- includes data-backed author/topic directories, detail views, onboarding, session-only appearance settings, remote tweet image previews, and local-first performance/accessibility artifacts

## Dev
```bash
nix develop -c cargo run -p eterea-dioxus
```

## Build
```bash
nix develop -c cargo build --workspace
```

## Verify
```bash
nix develop -c cargo fmt --all
nix develop -c cargo test --workspace
nix develop -c cargo check --workspace
nix develop -c cargo clippy --workspace --all-targets -- -A dead_code
nix develop -c cargo build -p eterea-dioxus
nix develop -c cargo test -p eterea-app --test performance_baseline -- --nocapture
```

## Release evidence

- Product/design execution map: `docs/design-implementation-map.md`
- Visual QA: `.omx/artifacts/visual/eterea-full-app/visual-qa-report.md`
- Performance report: `.omx/artifacts/perf/eterea-full-app/optimization-report.md`
- Accessibility checklist: `.omx/artifacts/accessibility/eterea-full-app/checklist.md`
- Release checklist: `docs/release-readiness-eterea-full-app.md`

## Notes
- the desktop MVP keeps the existing local SQLite storage
- data stays local after import
- remote tweet images are hidden by default; enabling previews is session-only and may make network requests for stored HTTPS image URLs
- the database location follows the platform app-data directory from the Rust backend (`dirs::data_local_dir()/eterea/bookmarks.db`)
- direct X sync remains deferred for this first Dioxus pass
- browser companion/server mode remains a planned follow-on phase
