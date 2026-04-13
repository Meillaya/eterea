# Frontend remake review + documentation guardrails

> Historical note: this document described the earlier frontend-remake work while
> Eterea still used the `src/frontend` Svelte app. The active app path is now the
> Dioxus desktop shell in `src/dioxus-app`, backed by shared Rust services in
> `src/app`.

## Current relevance

The product-direction guidance still matters:

- keep the app content-first
- preserve fast search/filter flows
- keep import discoverable but visually secondary
- keep multiple reading layouts
- avoid reintroducing dashboard-heavy chrome

The file-by-file references from the old Svelte shell are no longer active and
should be treated as historical only.

## Active touchpoints now

- UI shell: `src/dioxus-app/src/app.rs`
- UI styling: `src/dioxus-app/assets/app.css`
- Shared app services: `src/app/src/services/app.rs`
- Migration guardrails: `src/app/tests/migration_guardrails.rs`

## Verification checklist

### Automated

- `cargo test --workspace`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -A dead_code`
- `cargo build --workspace`

### Targeted manual verification

- Confirm the Dioxus desktop app opens into the library reliably.
- Confirm import, search, favorites, and layout switching still work coherently.
- Confirm the shell still feels calm, dark, and archive-first.
- Confirm there are no active startup/build paths that depend on the removed
  Tauri/Svelte app.

## Maintenance notes

- Do not treat `src/frontend` references in older plans or notes as active code.
- Do not reintroduce a dashboard-style shell without revisiting the product
  direction.
- If browser companion/server mode is added later, keep it on the same shared
  Rust service boundary rather than creating a second independent app shell.
