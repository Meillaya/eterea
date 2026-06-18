# Design system and implementation map

This document maps the implemented Rust/Dioxus UI system, runtime modules, verification artifacts, and release guardrails.

## Source-of-truth inputs

The design folder has been removed. The production source of truth is now the
implemented Dioxus UI and its tests:

| Input | Role |
| --- | --- |
| `src/dioxus-app/src/app.rs` | Root desktop shell composition and screen rendering. |
| `src/dioxus-app/src/app/` | Route, state, action, design-system, component, and screen helpers. |
| `src/dioxus-app/assets/app.css` | Typography, density, paper-tone, layout, and interaction styling. |
| `src/app/src/services/app.rs` | Data-backed product operations consumed by the UI. |
| `src/app/tests/` and Dioxus unit tests | Guardrails for service behavior, routes, media policy, design-system classes, and performance. |

Historical prototype context, if needed, belongs in external design tooling or
`docs/archive/`; do not recreate a repository-level prototype folder unless it is
part of the production source tree.

## Current product anchors

- The active product is local-first Rust/Dioxus, imports CSV/JSON/X archive JS, stores bookmarks in SQLite, and already supports search/filter/favorites/delete/layout switching (`README.md`).
- Workspace crates are `src/backend`, `src/app`, and `src/dioxus-app`.
- Current Dioxus UI is the active desktop product surface and should remain data-backed rather than mock-backed.
- `AppServices` is the service boundary for product workflows; do not bypass it from UI screens unless a new product-level method is added there first.

## Architecture target

The active implementation follows a Dioxus-native structure with route/state/action helpers split away from the root composition file. Continue evolving toward this shape:

```text
src/dioxus-app/src/
  main.rs
  app.rs                # root composition and service setup
  app/
    route.rs            # ScreenRoute enum and route helpers
    state.rs            # AppState, filters, layout, import/settings state
    actions.rs          # service-backed UI actions
    design_system.rs    # Smallcaps, Masthead, TagRail, Colophon, buttons
    components/
      bookmark_entry.rs
      inline_detail.rs
      layout_switcher.rs
      shell.rs
    screens/
      library.rs
      favorites.rs
      authors.rs
      topics.rs
      search.rs
      import.rs
      settings.rs
      onboarding.rs
      detail.rs
src/dioxus-app/assets/app.css
```

This shape is guidance, not a mandatory file list. The invariant is that route/state/actions/design-system/components/screens remain separable enough to prevent a monolithic UI module.

## Route and screen inventory

| Product route/screen | UI source | Dioxus target | Data/API contract |
| --- | --- | --- | --- |
| `library` | `src/dioxus-app/src/app.rs` + `app/screens/library.rs` | `ScreenRoute::Library`, `screens/library.rs` | `AppServices::query_bookmarks`, `AppServices::stats`, pagination. |
| `favorites` | shared library screen mode | `ScreenRoute::Favorites`, `screens/favorites.rs` or shared library screen mode | Same query API with `favorites_only=true`. |
| `authors` | data-backed directory UI | `ScreenRoute::Authors`, `screens/authors.rs` | Add product-level `author_index` returning author summaries. |
| `topics` | data-backed topic UI | `ScreenRoute::Topics`, `screens/topics.rs` | Add product-level `topic_index` returning tag counts. |
| `search` | search route UI | `ScreenRoute::Search`, `screens/search.rs` | Reuse `BookmarkQuery`; add scope support only if cheap/tested. |
| `import` | import route UI | `ScreenRoute::Import`, `screens/import.rs` | Add dry-run preview API, then reuse import transaction path. |
| `settings` | settings route UI | `ScreenRoute::Settings`, `screens/settings.rs` | Appearance settings must be functional; persistence preferred but not v1-blocking. |
| `onboarding` | empty-state/onboarding UI | `ScreenRoute::Onboarding`, `screens/onboarding.rs` | Empty DB / first-run route decision. |
| `entry:{id}` | detail route UI | `ScreenRoute::Entry(String)`, `screens/detail.rs` | Add `bookmark_detail(id)` product API. |
| `author:{handle}` | filtered author route UI | `ScreenRoute::Author(String)`, `screens/detail.rs` or `screens/authors.rs` | Add `bookmarks_by_author(handle, page)`. |
| `topic:{tag}` | filtered topic route UI | `ScreenRoute::Topic(String)`, `screens/detail.rs` or `screens/topics.rs` | Add `bookmarks_by_tag(tag, page)`. |

## Component mapping

| UI concept | Dioxus/CSS target | Notes |
| --- | --- | --- |
| Paper tones | CSS custom properties + `PaperTone` enum | `cream`, `offwhite`, `gray` are canonical. |
| Font stacks | CSS font stacks | Use Source Serif 4 / JetBrains Mono if available; retain system fallbacks. |
| Smallcaps labels | `Smallcaps` component/class | Prefer CSS class reuse over inline style sprawl. |
| Masthead | `Masthead` component | Supports compact/non-compact and dynamic subline counts. |
| Tag rail | `TagRail` component | Uses real top tags and layout controls. |
| Sidebar | `AppShell`/`Sidebar` | Shows navigation, top tags, database path/status. |
| Bookmark entry | `BookmarkEntry` | Maps model fields: content, author, tags, favorite, tweeted_at/imported_at/media. |
| Inline detail | `InlineDetail` | Missing mock fields (`likes`, `saved_at`) must map to real fields or be omitted/deferred. |
| `BLayoutIssue/Front/Long/Spread` | layout-specific components/classes | All data-backed, with shared entry components. |
| Import stepper | `ImportFlow` state machine | Source → Preview → Importing → Done. |
| Tweaks panel | Settings screen / local state | Do not ship design-host edit mode. |

## Data-field mapping and deferred mock fields

| Prototype field | Current model/source | Decision |
| --- | --- | --- |
| `id` | `Bookmark.id` | Use directly. |
| `handle` | `Bookmark.author_handle` | Use directly. |
| `name` | `Bookmark.author_name` | Use directly. |
| `tweeted_at` | `Bookmark.tweeted_at` | Use directly. |
| `saved_at` | `Bookmark.imported_at` | Label as imported/saved locally unless a true saved timestamp exists later. |
| `content` | `Bookmark.content` + optional `note_text` | Show content first; note text as secondary detail. |
| `tags` | `Bookmark.tags` | Use directly. |
| `media` | `Bookmark.media.len()` and URLs | Use count now; richer media previews are follow-up. |
| `is_favorite` | `Bookmark.is_favorite` | Use directly. |
| `likes`, `reach`, mock footnotes | Not available | Do not fake; omit or replace with local archive metadata. |
| avatar hue | Not available | Generate deterministic visual mark from author handle if needed. |

## Service boundary additions

Add product-level APIs to `src/app/src/services/app.rs` / `src/app/src/types.rs` when a screen needs them. Avoid screen-specific structs named after UI screens unless they represent a reusable product concept.

Preferred concepts:

- `AuthorSummary { handle, name, profile_image, bookmark_count, favorite_count, top_tags }`
- `TopicSummary { tag, bookmark_count, favorite_count }`
- `ImportPreview { format, candidate_count, duplicate_count: Option<usize>, sample: Vec<BookmarkPreview> }`
- `BookmarkDetail` may initially be `Bookmark` plus related tags/authors if cheap.

## Visual artifact structure

```text
.omx/artifacts/visual/eterea-full-app/
  reference/      # baseline screenshots from the running Dioxus app
  verdicts/       # visual-ralph / visual-verdict JSON + markdown reports
```

Canonical reference IDs:

- `onboarding`
- `library-issue`
- `library-front`
- `library-long`
- `library-spread`
- `favorites`
- `authors`
- `topics`
- `search`
- `import-source`
- `import-preview`
- `import-importing`
- `import-done`
- `settings`
- `entry-detail`
- `author-detail`
- `topic-detail`

Reference screenshot naming convention, when captures are regenerated from the app:

```text
.omx/artifacts/visual/eterea-full-app/reference/<screen-id>-1440x1000.png
.omx/artifacts/visual/eterea-full-app/verdicts/<screen-id>-verdict.json
.omx/artifacts/visual/eterea-full-app/verdicts/<screen-id>-verdict.md
```

## Visual rubric

Visual verdicts should score these areas per screen:

1. **Paper and ink:** tone, background, panel contrast, rules, accent color.
2. **Typography:** serif hierarchy, italic headline feel, mono smallcaps, date/count treatments.
3. **Layout rhythm:** masthead scale, sidebar width, tag rail spacing, entry columns, scroll regions.
4. **Interaction affordance:** active nav states, buttons, inline expansion cues, keyboard hints.
5. **Content truthfulness:** no mock-only fields presented as real data.
6. **Density behavior:** compact/regular/comfy settings produce visible but controlled spacing changes.
7. **Responsive minimums:** app remains usable at the agreed minimum desktop size.

Verdict statuses:

- `PASS`: close enough to reference and no blocking deviations.
- `ACCEPTED_DEVIATION`: mismatch documented with rationale.
- `FIX_REQUIRED`: visual mismatch blocks release.

## Fixture and performance contract

Fixture layout:

```text
fixtures/perf/
  small/   # ~20 bookmarks
  medium/  # ~500 bookmarks
  large/   # ~10k bookmarks
  stress/  # generated stress-lab archives; do not commit large data
```

Performance output layout:

```text
target/eterea/perf/
  performance_baseline.json
  performance_large_archive.json
  performance_author_directory.json
  stress-lab/
    performance_stress_lab_<count>.json
  perf_environment.json
```

Suggested runner shapes, without requiring new dependencies:

```bash
scripts/perf-baseline.sh
scripts/perf-baseline.sh --stress <count-from-fixtures/perf/stress-tiers.txt>
```

Current automated guardrails record 7 deterministic samples per service path
and emit min/median/p95/max JSON summaries. They remain dev guardrails unless a
release owner records target hardware, file-backed storage mode, and
`release_evidence=true`.

Performance evidence classes:

| Class | Purpose | Minimum sample contract | Required metadata |
| --- | --- | --- | --- |
| Dev guardrail | Fast local regression check for known service budgets. | 7 deterministic samples with median, p95, min, max, sample count, and pass/fail budget; `release_evidence=false`. | Budget name, fixture size, `storage_mode`, generated dataset metadata, environment metadata, and generated report path. |
| Stress-lab | Explicit large-count lab run from `fixtures/perf/stress-tiers.txt`. | 7-run minimum before comparing trends; record median, p95, min, max, and sample count for every service path or UI interaction under review. | Hardware, OS/kernel, Rust toolchain, cold/warm classification, storage mode, memory ceiling when available, and `release_evidence=false`. |
| Release evidence | Release-owner sign-off on target hardware/profile. | 7-run minimum for each service path and UI interaction; report median, p95, min, max, and sample count, with failed/outlier runs retained or explained. | Hardware/kernel/Rust metadata, cargo profile, cold/warm classification, `storage_mode` such as file-backed SQLite/WAL, pass/fail budget, report timestamp, and `release_evidence=true`. |

Use `cold` only for first-run/process-start paths with caches intentionally
empty. Use `warm` for repeated interaction/service-path samples after setup has
completed. A mixed run must split cold and warm samples into separate records
instead of averaging them together.

- Warm 10k library page single-run guardrail < 100ms; release evidence keeps
  this budget anchor but must measure 7 warm samples and report median/p95/min/max.
- Search single-run guardrail < 150ms on 10k fixture; release evidence keeps
  this budget anchor but must measure 7 warm samples and report median/p95/min/max.
- Author/topic index single-run guardrail < 100ms on 10k fixture; release
  evidence keeps this budget anchor but must measure 7 warm samples and report
  median/p95/min/max.
- Import 10k entries single-run guardrail < 10s on dev machine; release
  evidence keeps this budget anchor but must measure 7 samples and report
  median/p95/min/max.
- Stress tiers are defined in `fixtures/perf/stress-tiers.txt`; lower tiers are
  intermediate lab gates, and the largest tier is the strategic
  production-grade stress target.
- Stress-lab reports live under `target/eterea/perf/stress-lab/` and include
  `release_evidence=false`; they are not release-blocking until fixture
  generation mode, memory ceiling, hardware, cold/warm run classification,
  pass/fail budgets, and file-backed SQLite/WAL follow-up evidence are recorded.
- First usable shell < 1.5s after process start on dev machine is a proposed UI
  interaction budget until measured with the same 7-run release evidence
  contract.

Trigger `$performance-goal` if a screen/API path misses budget without a narrow local fix.

## Accessibility artifact contract

Accessibility checklist path:

```text
.omx/artifacts/accessibility/eterea-full-app/checklist.md
```

Checklist must cover:

- Tab order for shell, nav, tag rail, search, import, settings.
- Visible focus on all primary controls.
- Keyboard shortcuts disabled while inputs are focused.
- Escape/Enter/j/k behavior is non-trapping and documented.
- Inputs have labels or clear accessible text.
- Empty/error states are readable and actionable.
- Main text, muted text, accent buttons, and active navigation receive a contrast spot-check.

## Production implementation guardrails

Before large UI refactors, keep or update these production-readiness anchors:

- `docs/design-system.md` documents the implemented UI system and route mapping.
- `docs/operations/release-readiness.md` records release checks, waivers, and manual evidence.
- `fixtures/perf/README.md` explains performance fixture expectations.
- Automated tests cover service guardrails, performance budgets, route helpers, design-system class contracts, and media-safety policy.

Lock current behavior with tests or baselines before changing layout, service boundaries, import semantics, or media-loading policy.
