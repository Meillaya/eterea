# Bookmark load performance + stats reliability

This note documents the approved direction from `.omx/plans/prd-db-load-performance.md`, plus the code-review guardrails that should stay true while the load-path work is implemented and maintained.

## Scope

The goal is deliberately narrow:

- make the initial bookmark library load feel fast on a local database
- keep bookmark visibility reliable on first load and refresh
- stop the stats page from hanging in a loading state
- avoid unrelated UI or architecture churn

## Review snapshot: hotspots worth protecting

### 1. Avoid coupling list fetches to full stats reads

`src-tauri/src/lib.rs` currently derives pagination metadata for `get_bookmarks` by calling `db.get_stats()` after the list query. That makes every primary list read depend on the heavier stats path.

**Guardrail:** keep `get_bookmarks` pagination metadata independent from the full stats aggregation path.

### 2. Avoid per-bookmark hydration loops

`src/backend/src/storage/database.rs` currently calls `load_bookmark_tags()` and `load_bookmark_media()` once per bookmark inside `hydrate_bookmarks()`. That is an N+1-style pattern on the hottest read path.

**Guardrail:** batch or pre-group tag/media hydration for list reads so a larger library does not multiply round trips per bookmark.

### 3. Prioritize first bookmark paint over secondary data

`src/frontend/src/routes/+page.svelte` currently awaits `loadStats()` before `refreshBookmarks()` during `onMount()`. That means the home screen can delay the first visible bookmark render behind stats work.

**Guardrail:** the home route should prioritize bookmark visibility first, then allow stats to complete independently.

### 4. Stats views must always reach a terminal state

`src/frontend/src/routes/stats/+page.svelte` is responsible for showing success, empty, or retryable error UI. Any future stats-loading changes should preserve that contract.

**Guardrail:** the stats route must always settle into one of these states:

- data rendered
- empty-state rendered
- retryable error rendered

Never reintroduce an unbounded spinner state.

## Expected behavior after the performance work

When this work is complete, maintainers should be able to rely on the following behavior:

1. The bookmark library begins loading immediately on entry.
2. Bookmark visibility is not blocked by stats aggregation.
3. Pagination metadata does not require a full stats refresh.
4. Stats loading failures degrade to a terminal error state instead of an infinite loader.
5. Search, filters, favorites, and refresh continue to behave correctly after the faster load path lands.

## Verification checklist

Use the same checks called out in `.omx/plans/test-spec-db-load-performance.md`:

### Automated

- `cargo test --workspace`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -A dead_code`
- `cd src/frontend && bun run check`
- `cd src/frontend && bun run build`

### Targeted manual verification

- Measure bookmark list query + hydration timing before and after the change on a populated local DB.
- Confirm the home route shows bookmarks before stats finish loading.
- Confirm refresh/reset/view changes stay responsive.
- Confirm the stats route reaches success, empty, or retryable error; never an infinite loading state.
- Confirm loaded bookmarks still include complete tags/media data.
- Confirm search, filter, and favorites behavior still match the underlying data.

## Maintenance notes

- Do not recouple bookmark pagination to `get_stats()` without re-measuring the hot path.
- Do not accept a faster list load if it drops tags, media, or bookmark completeness.
- If rendering remains the dominant bottleneck after backend fixes, treat render/windowing as a follow-up pass instead of widening this change unnecessarily.
