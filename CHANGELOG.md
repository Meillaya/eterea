# Changelog

All notable changes to Eterea are recorded here. This project follows a
human-readable changelog and keeps release notes honest about the evidence that
has actually been captured.

## Unreleased

- Added production community documentation: contributing guide, security policy,
  code of conduct, and this changelog.
- Keep release notes aligned with the local-first privacy model documented in
  [release readiness](docs/operations/release-readiness.md).

## 0.1.0 production-candidate

Initial production-candidate desktop release surface.

- Local-first X/Twitter bookmark archive with CSV, JSON, and X archive import
  paths.
- Dry-run import preview before committed writes.
- SQLite-backed library, favorites, search, author/topic directories, detail
  routes, and editorial reading layouts.
- Dioxus desktop shell with session-only appearance settings and remote media
  previews hidden by default.
- Nix-based verification path documented in [development](docs/development.md).
- Licensed under the [MIT](LICENSE) license.

Known release follow-ups remain tracked in
[release readiness](docs/operations/release-readiness.md), including live desktop
screenshot/timing, installer/signing, and release-grade performance evidence.
