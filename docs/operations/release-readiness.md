# Release readiness

Status: **NO-GO for public release** as of 2026-06-18. Local CI mirror and local draft release artifact checks have passed, but public release approval remains blocked by live visual/accessibility proof and UI timing evidence.

## Current scope implemented

- Dioxus desktop shell for the Editorial Reading Room design direction.
- Local-first bookmark import from CSV, JSON, and X archive JS.
- Dry-run import preview plus transactional import write path.
- Library/favorites reading layouts: Issue, Front Page, Long-Read, Spread.
- Search, author/topic directories, entry detail, filtered author/topic archive routes.
- Empty/onboarding, error states, honest session-only appearance settings, and session-only remote tweet image previews hidden by default.
- Service guardrails for import/query/favorite/delete/persistence/detail/directory APIs.
- Performance harnesses now distinguish dev guardrails, file-backed SQLite/WAL release evidence, local optimization evidence, and non-release UI timing evidence.

## Final local verification commands

Run from the repository root. These are local gates; they do not prove GitHub-hosted release upload or public release readiness by themselves.

```bash
nix develop -c cargo fmt --all -- --check
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo test --workspace
nix develop -c cargo build -p eterea-dioxus --release
nix develop -c scripts/perf-baseline.sh
python scripts/validate-github-workflows.py
python scripts/check-doc-links.py README.md 'docs/**/*.md' CHANGELOG.md CONTRIBUTING.md SECURITY.md CODE_OF_CONDUCT.md
scripts/desktop-qa.sh
```

## CI, draft release, and artifact evidence

| Area | Current evidence | Release classification | Open caveat / waiver |
| --- | --- | --- | --- |
| Docs link checker (T10) | `scripts/check-doc-links.py`; `.omo/evidence/production-grade-project/t10-full-docs.txt`; `.omo/evidence/production-grade-project/t10-verifier-doc-link-check.txt` | Local docs gate passed in T10; T26 reruns this document below. | No T10 doneclaim JSON exists; rely on direct command artifacts, not summaries. |
| CI workflow (T11) | `.github/workflows/ci.yml`; `.omo/evidence/production-grade-project/ci-workflow-validate.txt`; local mirror logs `ci-local-fmt.txt`, `ci-local-clippy.txt`, `ci-local-tests.txt`, `ci-local-release-build.txt`, `ci-local-perf-baseline.txt` | Static workflow validation and local CI mirror passed. | GitHub-hosted Actions execution was not proved by this local evidence. |
| Draft release workflow (T12) | `.github/workflows/draft-release.yml`; `.omo/evidence/production-grade-project/release-workflow-validate.txt`; `.omo/evidence/production-grade-project/t12-acceptance-rg.txt` | Local-safe/static draft release contract passed: `workflow_dispatch`, `v*` tags, artifact upload, and `gh release create --draft`. | No live GitHub release dispatch/upload/publish was run. |
| Local release artifact proof (T27) | `.omo/evidence/production-grade-project/t27-doneclaim.json`; `.omo/evidence/production-grade-project/t27-artifact-manifest.md`; `.omo/evidence/production-grade-project/release-artifact-sha256.txt` | Local release build and artifact assembly mirror passed. Archive: `release-artifacts/eterea-v0.1.0-local-t27-20260618T013924Z-Linux-x86_64.tar.gz`; SHA256 `9bc8023b26275b15a1c1e411dea527c1cdef90c5ca439ee1b0ec78ca52de2b5f`. | Dirty worktree provenance remains; artifact is local-only; no GitHub release permission/upload proof exists. |

Do not treat old `.omx` visual/accessibility artifacts as current public-release proof. T26 is based on current `.omo/evidence/production-grade-project/` artifacts from T20-T28, especially T24, T27, and T28.

## Preview asset provenance and privacy posture

- Public README/docs previews are committed under `docs/assets/previews/` and are synthetic/local/privacy-safe preview assets, not live public-release screenshots.
- `docs/assets/previews/README.md` records preview provenance and the rule that committed previews must not contain personal bookmark data, private paths, OS usernames, or remote media.
- Remote media previews are off by default. README/docs previews must not use remote Markdown image hotlinks; ordinary external text links are acceptable because they do not fetch media by default.
- T25 completed privacy/trust hardening: remote media stays hidden by default, unsafe/raw HTML rendering scans are empty under `src/`, and log-macro scans found no backend debug logging of raw imported fields such as `tweet_url`, `content`, `author_handle`, or `tags`.

## Backend performance evidence classes

| Task | Evidence | Classification | Status |
| --- | --- | --- | --- |
| T20 statistical service harness | `.omo/evidence/production-grade-project/t20-doneclaim.json`; `target/eterea/perf/performance_baseline.json`; `target/eterea/perf/performance_large_archive.json`; `target/eterea/perf/performance_author_directory.json` | Dev guardrail. Statistical fields are present, but generated in-memory reports keep `release_evidence=false`. | Passed as local guardrail only; not public release evidence. |
| T21 file-backed SQLite/WAL path | `.omo/evidence/production-grade-project/t21-report-summary.md`; `target/eterea/perf/performance_file_backed.json` | File-backed release evidence for guardrail-sized developer hardware: `storage_mode=file-backed-sqlite-wal`, `release_evidence=true`, `classification=release-evidence`, WAL proof present, all path budgets pass. | Accepted for the measured backend file-backed scope; not a full target-platform certification. |
| T22 backend optimization | `.omo/evidence/production-grade-project/t22-before-after-summary.md` | Local before/after test-profile evidence. | Import p95 improved in the comparable reports (`120ms -> 43ms` in-memory 500; `183ms -> 70ms` file-backed 750). Do not generalize as an absolute production timing guarantee. |

Release notes must keep performance artifacts in one of three classes:

- **Dev guardrail:** fast local or CI check of current service-path budget anchors. It may be single-run, must use `release_evidence=false`, and must not be described as production-grade performance proof.
- **Stress-lab:** staged large-count exploration under `target/eterea/perf/stress-lab/`. It uses `release_evidence=false` until the run records sample statistics, hardware/kernel/Rust metadata, cold/warm classification, storage mode, memory ceiling, and pass/fail budgets.
- **Release evidence:** release-owner artifact with `release_evidence=true` on target hardware/profile. For each service path and UI interaction, collect at least 7 samples with `sample_count`, median, p95, min, and max. Record hardware, OS/kernel, Rust version, cargo profile, report timestamp, `storage_mode` such as file-backed SQLite/WAL, cold/warm classification, and the budget used for pass/fail.

Cold and warm measurements are separate evidence rows. Do not average first-run startup/import/render samples with repeated warm service-path or UI interaction samples.

## Frontend/UI timing, visual, and accessibility status

| Area | Current evidence | Status | Public release implication |
| --- | --- | --- | --- |
| T23 UI launch/interaction harness | `.omo/evidence/production-grade-project/t23-doneclaim.json`; `.omo/evidence/production-grade-project/ui-launch-timing.json`; `scripts/ui-launch-timing.sh` | Harness exists with release waivers. | Harness existence is not release timing proof. |
| T24 UI timing after optimization | `.omo/evidence/production-grade-project/t24-after-ui-launch.json`; `.omo/evidence/production-grade-project/t24-ui-timing-summary.md` | **Failing non-release evidence:** first usable shell `5315ms` over `1500ms`, `pass=false`, `release_evidence=false`, `sample_count=1`; interactions are `blocked_no_stable_ui_driver`. | Keep performance waiver open; public release is NO-GO until first usable timing and interactions meet release criteria. |
| T28 live visual/accessibility proof | `.omo/evidence/production-grade-project/desktop-display-blocker.md`; `.omo/evidence/production-grade-project/t28-screenshot-manifest.md`; `.omo/evidence/production-grade-project/t28-accessibility-checklist.md` | Accepted blocker path only. No valid live Eterea screenshots exist; focus traversal, focus visibility, pixel/contrast checks, and screen coverage are blocked. | Keep release visual/accessibility waiver open. Synthetic `docs/assets/previews/` images do not substitute for live screenshots. |

Live desktop QA must use the real Dioxus app (`scripts/desktop-qa.sh` or the T23/T28 tmux harness), fixture-only/synthetic data, a valid Eterea foreground screenshot, and captured timing. A no-display or non-Eterea screenshot blocker is honest evidence but not a pass.

## Known open waivers and blockers

- **Public release visual proof:** open. T28 left `desktop-display-blocker.md`; no valid live Eterea screenshots are present.
- **Accessibility proof:** open. Keyboard traversal, focus visibility, contrast/pixel checks, and screen coverage remain blocked without valid live screenshots or a stable desktop UI driver.
- **UI performance release evidence:** open. T24 records first usable shell `5315ms > 1500ms`, `pass=false`, `release_evidence=false`, and blocked interactions.
- **GitHub release publishing proof:** open. T27 proves local artifact assembly and checksums only; it does not prove GitHub release permissions, upload, draft creation, or public publishing.
- **Dirty worktree provenance:** open risk. T27 and T28 preserved unrelated dirty/untracked state; local artifacts were produced from that shared dirty worktree.
- **Installer/signing/platform smoke:** open. The current draft release workflow intentionally does not sign/notarize or run target-platform installer smoke tests.

## Packaging notes

- Keep the `nix develop` path as the canonical verification environment because plain `cargo test --workspace` may miss system OpenSSL/pkg-config outside Nix on this host.
- The backend stores the default database at the platform app-data path from `dirs::data_local_dir()/eterea/bookmarks.db`.
- Imports skip duplicate tweet URLs and parse before writing, so unsupported/broken files should leave the archive unchanged.
- Current local draft release artifacts live under `release-artifacts/` and are intentionally local proof, not GitHub-hosted assets.

## go / no-go checklist

- [x] CI workflow present and statically validated locally (T11).
- [x] Local CI mirror passed for fmt, clippy, tests, release build, and perf baseline (T11/T27).
- [x] Draft release workflow present and statically validated locally (T12).
- [x] Local draft release artifact assembly and checksum proof passed (T27).
- [x] Preview asset provenance documented for `docs/assets/previews/`; previews are synthetic/local/privacy-safe and remote media fetch is off by default (T3/T8/T25).
- [x] Backend statistical/dev guardrail and file-backed SQLite/WAL evidence recorded with classifications (T20/T21/T22).
- [x] Privacy/trust hardening complete for remote media default, unsafe rendering, and raw logging checks (T25).
- [ ] GitHub-hosted CI/release permission and draft release upload/publish proof.
- [ ] Live Eterea screenshot set for canonical screens with no unrelated/private desktop captures.
- [ ] Live accessibility proof for keyboard traversal, focus visibility, contrast/pixel checks, and screen coverage.
- [ ] Release-grade UI performance proof: first usable shell within the `1500ms` budget, 7-sample release evidence, and unblocked interactions.
- [ ] Optional before public release: target-platform installer/signing smoke test.

Decision: **NO-GO for public release** while any live visual/accessibility/timing item above remains unchecked. Local CI and local artifact proof are valuable merge/readiness evidence, but they are not GitHub publish proof and do not close the live desktop waiver.
