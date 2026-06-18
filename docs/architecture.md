# Architecture

Eterea is organized as a small Rust workspace with one active desktop runtime and
one shared application/service boundary.

## Workspace crates

| Path | Crate | Responsibility |
| --- | --- | --- |
| `src/backend` | `eterea-core` | Core domain models, import parsers, SQLite persistence, search, statistics, and CLI/migration binaries. |
| `src/app` | `eterea-app` | Product-facing service API used by the UI and integration tests. UI code should call this layer instead of reaching directly into storage. |
| `src/dioxus-app` | `eterea-dioxus` | Active Dioxus desktop shell, route/state/action helpers, reusable components, and CSS. |

## Runtime flow

```text
Dioxus UI
  -> src/app AppServices
    -> src/backend ingestion/search/storage
      -> local SQLite database
```

The UI owns presentation state such as active route, selected layout, filters,
expanded cards, import progress, and session-only appearance/media choices. The
service layer owns product operations such as importing, dry-run previews,
querying bookmarks, favorites, deletion, detail lookups, and directory data.

## Source layout conventions

- Keep product operations in `src/app/src/services/app.rs` unless a new module is
  justified by clear growth.
- Keep Dioxus route/state/action helpers under `src/dioxus-app/src/app/` so the
  root `app.rs` stays composition-focused.
- Keep backend storage SQL in `src/backend/src/storage/` and parser-specific
  logic in `src/backend/src/ingestion/`.
- Keep generated screenshots, local runtime databases, OMX state, and standalone
  design exports out of the repository root.

## Persistence

The default database path is resolved with `dirs::data_local_dir()` and stored
under `eterea/bookmarks.db`. Tests use temporary databases and fixtures so they
remain isolated from a developer's local archive.

## Trust boundaries

Imported bookmark archives are untrusted input. Parse CSV, JSON, and X archive
JavaScript exports at the ingestion/service boundary before storage, and keep
their fields untrusted when rendering or opening them later: `tweet_url`,
`content`, `note_text`, `author_handle`, `author_name`, `tags`, `comments`,
profile URLs, and media URLs may contain attacker-controlled text. Dioxus views
must render these values as text nodes, never raw HTML, and backend debug paths
must log counts, durations, and policy decisions instead of raw bookmark fields.

Remote media is a second trust boundary. The default UI state hides remote image
previews, media preview URLs must be HTTPS, and opening an external URL requires
an explicit user action that passes the HTTPS-only opening policy. Generated
artifacts such as `.omo/`, `.omx/`, `target/`, and screenshot/preview scratch
outputs are untrusted build evidence until reviewed and copied into
`docs/assets/previews/` with synthetic/local-fixture provenance.

## Deferred surfaces

Direct X sync, browser companion/server mode, signed installers, and persisted
appearance preferences are future product surfaces. Do not represent them as
shipped runtime behavior until they have implementation and verification evidence.
