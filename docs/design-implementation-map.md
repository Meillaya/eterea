# Eterea Full-App Design Implementation Map

This is the Milestone 0 execution contract for turning `design/Eterea - Full App.html` into the Rust/Dioxus desktop app. It freezes the prototype-to-product mapping, visual QA artifact structure, and fixture/performance contract before implementation edits begin.

## Source-of-truth inputs

| Input | Role |
| --- | --- |
| `design/Eterea - Full App.html` | Canonical full-app prototype shell; loads the React/Babel design files and exposes accent, paper tone, density, and start-screen controls. |
| `design/b-system.jsx` | Editorial design tokens and primitives: paper tones, serif/mono fonts, masthead, smallcaps, tag rail, colophon. |
| `design/b-library.jsx` | Core reading layouts and bookmark entry patterns: Issue, Front Page, Long-Read, Spread, inline detail. |
| `design/b-screens.jsx` | Focused screens: entry detail, author archive, topic page, search, import, settings, onboarding. |
| `design/b-shell.jsx` | Connected prototype shell: sidebar navigation, screen state, library/favorites/authors/topics/search/import/settings routes. |
| `.omx/plans/prd-eterea-full-app.md` | Approved milestone plan and staffing/verification guidance. |
| `.omx/plans/test-spec-eterea-full-app.md` | Test, visual, accessibility, and performance verification contract. |

## Current product anchors

- The active product is local-first Rust/Dioxus, imports CSV/JSON/X archive JS, stores bookmarks in SQLite, and already supports search/filter/favorites/delete/layout switching (`README.md`).
- Workspace crates are `src/backend`, `src/app`, and `src/dioxus-app`.
- Current Dioxus UI is intentionally treated as a behavior-preserving MVP baseline, not as the final design architecture.
- `AppServices` is the service boundary for product workflows; do not bypass it from UI screens unless a new product-level method is added there first.

## Architecture target

Recommended option remains the approved RALPLAN decision: **Dioxus-native rebuild from prototype tokens**.

Target module shape:

```text
src/dioxus-app/src/
  main.rs
  app/
    mod.rs              # root composition and service setup
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

This shape is guidance, not a mandatory file list. The invariant is that route/state/actions/design-system/components/screens are separable enough to prevent another monolithic `app.rs`.

## Route and screen inventory

| Prototype route/screen | Prototype source | Dioxus target | Data/API contract |
| --- | --- | --- | --- |
| `library` | `BApp`, `BLibraryScreen`, `BLayoutIssue/Front/Long/Spread` | `ScreenRoute::Library`, `screens/library.rs` | `AppServices::query_bookmarks`, `AppServices::stats`, pagination. |
| `favorites` | `BLibraryScreen initialFavOnly` | `ScreenRoute::Favorites`, `screens/favorites.rs` or shared library screen mode | Same query API with `favorites_only=true`. |
| `authors` | `BAuthorsIndex` | `ScreenRoute::Authors`, `screens/authors.rs` | Add product-level `author_index` returning author summaries. |
| `topics` | `BTopicsIndex` | `ScreenRoute::Topics`, `screens/topics.rs` | Add product-level `topic_index` returning tag counts. |
| `search` | `BSearchScreen` | `ScreenRoute::Search`, `screens/search.rs` | Reuse `BookmarkQuery`; add scope support only if cheap/tested. |
| `import` | `BImportFlow` | `ScreenRoute::Import`, `screens/import.rs` | Add dry-run preview API, then reuse import transaction path. |
| `settings` | `BScreenSettings` | `ScreenRoute::Settings`, `screens/settings.rs` | Appearance settings must be functional; persistence preferred but not v1-blocking. |
| `onboarding` | `BScreenOnboarding` | `ScreenRoute::Onboarding`, `screens/onboarding.rs` | Empty DB / first-run route decision. |
| `entry:{id}` | `BScreenDetail` | `ScreenRoute::Entry(String)`, `screens/detail.rs` | Add `bookmark_detail(id)` product API. |
| `author:{handle}` | `BScreenAuthor` | `ScreenRoute::Author(String)`, `screens/detail.rs` or `screens/authors.rs` | Add `bookmarks_by_author(handle, page)`. |
| `topic:{tag}` | `BScreenTag` | `ScreenRoute::Topic(String)`, `screens/detail.rs` or `screens/topics.rs` | Add `bookmarks_by_tag(tag, page)`. |

## Component mapping

| Prototype component | Dioxus/CSS target | Notes |
| --- | --- | --- |
| `B_PAPERS`, `bPaper` | CSS custom properties + `PaperTone` enum | `cream`, `offwhite`, `gray` are canonical. |
| `B_FONT_SERIF`, `B_FONT_MONO` | CSS font stacks | Use Source Serif 4 / JetBrains Mono if available; retain system fallbacks. |
| `BSmallcaps` | `Smallcaps` component/class | Prefer CSS class reuse over inline style sprawl. |
| `BMasthead` | `Masthead` component | Supports compact/non-compact and dynamic subline counts. |
| `BTagRail` | `TagRail` component | Uses real top tags and layout controls. |
| `BSidebar` | `AppShell`/`Sidebar` | Shows navigation, top tags, database path/status. |
| `BEntry` | `BookmarkEntry` | Maps model fields: content, author, tags, favorite, tweeted_at/imported_at/media. |
| `BInlineDetail` | `InlineDetail` | Missing mock fields (`likes`, `saved_at`) must map to real fields or be omitted/deferred. |
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
  reference/      # baseline screenshots from design/Eterea - Full App.html
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

Reference screenshot naming convention:

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
- `FIX_REQUIRED`: visual mismatch blocks the milestone.

## Fixture and performance contract

Fixture layout:

```text
fixtures/perf/
  small/   # ~20 bookmarks
  medium/  # ~500 bookmarks
  large/   # ~10k bookmarks
  stress/  # ~50k bookmarks
```

Performance output layout:

```text
.omx/artifacts/perf/eterea-full-app/
  baseline.json
  baseline.md
  optimization-report.json
  optimization-report.md
```

Suggested runner shapes, without requiring new dependencies:

```bash
cargo test -p eterea-app --test performance_baseline -- --nocapture
# or
scripts/perf-baseline.sh
```

Budgets copied from the approved test spec:

- Warm 10k library page p95 < 100ms.
- Search p95 < 150ms on 10k fixture.
- Author/topic indexes p95 < 100ms on 10k fixture.
- Stress 50k search p95 < 500ms or documented waiver.
- Import 10k entries < 10s on dev machine.
- First usable shell < 1.5s after process start on dev machine.

Trigger `$performance-goal` if a screen/API milestone misses budget without a narrow local fix.

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

## Milestone gate for implementation start

Implementation may begin after M0 when these exist:

- `docs/design-implementation-map.md`
- `.omx/artifacts/visual/eterea-full-app/reference/README.md`
- `.omx/artifacts/visual/eterea-full-app/verdicts/README.md`
- `.omx/artifacts/perf/eterea-full-app/README.md`
- `.omx/artifacts/accessibility/eterea-full-app/checklist.md`
- `fixtures/perf/README.md`

M1 must then lock current behavior with tests/baselines before large UI refactors.
