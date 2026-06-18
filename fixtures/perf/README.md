# Performance fixtures

Eterea keeps large performance datasets generated rather than committed. The
performance harness in `src/app/tests/performance_baseline.rs` creates
repeatable synthetic archives from a deterministic pattern of authors, tags,
dates, text, and media flags.

## Release baseline

Run from the Nix shell or through `nix develop -c`:

```bash
scripts/perf-baseline.sh
```

This runs the release-blocking generated 500 and 10k service path tests with
`RUST_TEST_THREADS=1`. Reports are written under `target/eterea/perf/`,
including `perf_environment.json` with OS, kernel, machine, and Rust version
metadata.

Today this default command is a dev guardrail: it protects the existing budget
anchors quickly, but a single run is not production-grade performance evidence.
Treat generated reports as `release_evidence=false` until a release owner runs
the statistical evidence contract below.

## Stress-lab stages

Stress runs are explicit, ignored by default, and limited to the supported
staged counts listed in `fixtures/perf/stress-tiers.txt`:

```bash
scripts/perf-baseline.sh --stress <count-from-fixtures/perf/stress-tiers.txt>
```

Lower tiers are intermediate gates. The largest tier is the strategic
production-grade stress target selected for the backend roadmap. These stress
runs are not release-blocking until a release owner records cold/warm run
classification, memory ceiling, and pass/fail budgets alongside the generated
environment metadata.

Stress runs intentionally use generated JSON strings and in-memory SQLite so
large fixture data stays out of git. Treat output from the largest configured
stress tier as stress-lab evidence until memory ceilings and file-backed
SQLite/WAL follow-up evidence are recorded.
Reports are written under `target/eterea/perf/stress-lab/` as
`performance_stress_lab_<count>.json` and include `release_evidence=false` and
`storage_mode=in-memory-sqlite` to keep lab evidence distinct from release
evidence.

## Evidence classification and sample contract

Use three labels consistently in performance reports and release notes:

| Class | When to use it | Required fields |
| --- | --- | --- |
| `dev_guardrail` | Fast deterministic service-path budget check, including the default `scripts/perf-baseline.sh` run. | `release_evidence=false`, `storage_mode`, fixture size, budget name, report path. |
| `stress_lab` | Explicit `--stress <count>` runs for staged large-count exploration. | `release_evidence=false`, `storage_mode=in-memory-sqlite` unless changed, stress count, cold/warm classification, hardware/kernel/Rust metadata. |
| `release_evidence` | Release-owner evidence on target hardware/profile. | `release_evidence=true`, `storage_mode`, hardware, OS/kernel, Rust version, cargo profile, report timestamp, cold/warm classification, pass/fail budget. |

Before any service path or UI interaction is promoted from dev guardrail or
stress-lab output to release evidence, collect at least 7 samples for that
specific path/classification. Each promoted result must record `sample_count`,
`median`, `p95`, `min`, and `max`; do not drop failed samples or outliers
without a note in the artifact. Record cold and warm runs separately:

- `cold`: process start, first import, first query, first render, or another
  path whose caches intentionally begin empty.
- `warm`: repeated service path or UI interaction after setup/import/rendering
  has already completed.

The current budget anchors remain the starting point: 10k library page < 100ms,
10k search < 150ms, 10k author/topic indexes < 100ms, 10k import < 10s, and
first usable shell < 1.5s. Any new budget is a proposal until backed by the
7-run evidence contract above.

Do not commit generated large archives or SQLite databases. Commit only the
harness, deterministic seed/shape, and small fixtures needed for correctness
tests.
