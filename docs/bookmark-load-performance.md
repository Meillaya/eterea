# Bookmark load performance + stats reliability

> Historical note: this document was written while the app still used the
> Tauri/Svelte shell. The core backend observations remain useful, but any
> references to `src-tauri` or `src/frontend` are historical and not part of the
> active app path anymore.

## What still matters

These performance principles still apply to the current Rust + Dioxus app:

1. bookmark list reads should stay independent from heavyweight aggregate work
2. hydration should avoid N+1-style per-bookmark fetch loops
3. visible bookmark content should appear before secondary/summary data
4. stats/error/loading flows should always settle into a terminal state
5. cached/local-first behavior should prefer visible content over blank-screen loading

## Active code areas

- backend storage/query path: `src/backend/src/storage/database.rs`
- shared service layer: `src/app/src/services/app.rs`
- active desktop shell: `src/dioxus-app/src/app.rs`

## Verification checklist

### Automated

- `cargo test --workspace`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -A dead_code`
- `cargo build --workspace`

### Targeted manual verification

- Confirm the library loads promptly from a populated local DB.
- Confirm import, browse, and search/filter remain responsive.
- Confirm the desktop app does not block core content behind secondary status work.
- Confirm persisted data still appears after restart.

## Maintenance notes

- Do not recouple hot list reads to heavyweight aggregate work without
  re-measuring the path.
- Do not regress the active app path back to a blank-screen-first loading model
  when cached/local data is available.
- Treat old Tauri/Svelte-specific profiling notes as historical implementation
  context only.
