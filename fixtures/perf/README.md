# Performance Fixtures

Fixture directories for Eterea full-app performance baselines:

- `small/` — approximately 20 bookmarks.
- `medium/` — approximately 500 bookmarks.
- `large/` — approximately 10,000 bookmarks.
- `stress/` — approximately 50,000 bookmarks.

Use generated or checked-in CSV/JSON/archive-JS files. If generated, record the generator command and seed in `.omx/artifacts/perf/eterea-full-app/baseline.md`.

No production code should assume these fixtures exist at runtime.
