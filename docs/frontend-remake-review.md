# Frontend remake review + documentation guardrails

This note documents the approved direction from `/home/mei/Documents/eterea/.omx/plans/prd-frontend-remake-speed-ui.md`, plus a code-review snapshot of the current frontend surfaces that matter most while the remake is implemented and maintained.

Use it together with `/home/mei/Documents/eterea/.omx/plans/test-spec-frontend-remake-speed-ui.md`.

## Scope

The goal is broad but still product-focused:

- remake the frontend around a fast, calm, content-first bookmark reader
- preserve search, filters, favorites, import, settings, sidebar/navigation, and multiple layout modes
- remove the stats page and any dashboard flavor
- keep loading, refresh, and error behavior lightweight and reliable
- document the highest-risk UX and code-quality constraints before the UI churn lands

## Current review snapshot: requirement coverage

### Already present and worth preserving

- **Search remains keyboard-friendly** via `/` focus and `Esc` clear in `src/frontend/src/lib/components/SearchBar.svelte`.
- **Multiple layout modes already exist** in `src/frontend/src/lib/components/LayoutToggle.svelte` and `src/frontend/src/lib/stores/bookmarks.svelte.ts`.
- **Favorites, recent, tags, and date filtering** already exist across `Sidebar.svelte`, `DateFilter.svelte`, and `api.ts` filter plumbing.
- **Import and settings flows already exist** in `ImportModal.svelte` and `src/frontend/src/routes/settings/+page.svelte`.
- **Cached-first library hydration plus best-effort stats refresh** already exist in `src/frontend/src/lib/api.ts` and `src/frontend/src/routes/+page.svelte`.

### Explicit PRD gaps that still need protection during the remake

- **Stats still exists as a first-class surface** in `README.md`, `src/frontend/src/lib/components/Header.svelte`, and `src/frontend/src/routes/stats/+page.svelte`, which conflicts with the approved removal.
- **The home route still carries dashboard weight** because `src/frontend/src/routes/+page.svelte` mixes hero copy, stats cards, control chrome, and feed content in one large shell.
- **Bookmark reading surfaces are split across two separate interaction models** (`BookmarkCard.svelte` and `BookmarkRow.svelte`), which makes consistency easy to regress while preserving layout modes.
- **Filter reset behavior is scattered** across sidebar/search/date components, so simplification work can easily break discoverability or keyboard flow if it is not reviewed holistically.

## Review snapshot: hotspots worth protecting

### 1. Remove stats affordances as one coherent deletion

`README.md`, `src/frontend/src/lib/components/Header.svelte`, and `src/frontend/src/routes/stats/+page.svelte` all still advertise or implement a stats/dashboard surface.

**Guardrail:** delete stats navigation, route, and documentation together. Do not leave dead links, empty nav gaps, or stale references to a stats dashboard after the remake.

### 2. Keep the library route focused on orchestration, not UI sprawl

`src/frontend/src/routes/+page.svelte` currently owns sidebar state, import modal state, filter signature tracking, stats refresh, hero content, toolbar composition, and the feed layout.

**Guardrail:** the rebuilt route should stay responsible for page orchestration only. Avoid recreating another large all-in-one route component when remaking the shell.

### 3. Preserve fast keyboard-first search while simplifying controls

`src/frontend/src/lib/components/SearchBar.svelte` already supports `/` to focus and `Esc` to clear. `Sidebar.svelte` and `DateFilter.svelte` also reset overlapping pieces of filter state.

**Guardrail:** keep keyboard-first search access intact and make filter resets more predictable, not more magical. Simplification must not hide the current fast-search affordance.

### 4. Preserve multiple layout modes without forking behavior quality

`src/frontend/src/lib/components/BookmarkList.svelte` branches between `BookmarkCard.svelte` and `BookmarkRow.svelte`, while layout labels are surfaced again in settings and the toolbar.

**Guardrail:** keep all required layout modes, but ensure they share the same core content/action contract. Do not let one mode become the only fully maintained path.

### 5. Keep tweet content primary and enrichments secondary

`src/frontend/src/lib/components/BookmarkCard.svelte` layers avatars, media, menus, tags, proxied media fallbacks, and link previews around the saved tweet text.

**Guardrail:** tweet text, author, time, media, and core actions should remain readable first. Previews, proxies, and other enrichments must stay best-effort and must never block the main reading flow.

### 6. Keep import and settings available but visually quieter than the feed

`ImportModal.svelte` and `src/frontend/src/routes/settings/+page.svelte` already preserve required workflows, but the remake intends them to become supporting surfaces rather than competing destinations.

**Guardrail:** preserve these workflows and their discoverability, while keeping the primary navigation centered on the bookmark library rather than on tool-like secondary pages.

## Expected behavior after the frontend remake

When this work is complete, maintainers should be able to rely on the following behavior:

1. The app opens directly into a content-first bookmark library.
2. Tweets/bookmarks are the dominant visual unit in every layout mode.
3. Search, filters, favorites, import, settings, sidebar/navigation, and multiple layout modes still work.
4. The stats page and all dashboard-style affordances are removed cleanly.
5. Loading, empty, refresh, and error states stay lightweight and do not block visible content unnecessarily.
6. Keyboard-centric library usage still works, especially fast search access and quick navigation.
7. Supporting workflows remain discoverable without competing with the reading surface.

## Verification checklist

Use the same checks called out in `/home/mei/Documents/eterea/.omx/plans/test-spec-frontend-remake-speed-ui.md`.

### Automated

- `cargo test --workspace`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -A dead_code`
- `cd src/frontend && bun run check`
- `cd src/frontend && bun run build`

### Targeted manual verification

- Confirm the app opens to the library quickly and reliably.
- Confirm the default library layout feels structurally new, cleaner, and more spacious.
- Confirm `/` still focuses search and `Esc` still clears it.
- Confirm filters, favorites, and layout switching still work coherently.
- Confirm import and settings remain available without overpowering the library surface.
- Confirm there are no dead links or stale references to the removed stats page.
- Confirm tweet content, media, tags, and actions remain readable in every layout mode.
- Confirm loading, refresh, empty, and error states stay lightweight.

## Maintenance notes

- Do not reintroduce a dedicated stats/dashboard route without revisiting the approved PRD.
- Do not simplify navigation by collapsing away the required layout modes.
- Do not trade keyboard-friendly search for visual minimalism.
- If the new shell starts growing back into a monolithic route, split responsibilities before adding more UI state.
