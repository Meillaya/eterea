# Eterea Full-App Release Readiness

Status: Milestone 11 release-readiness pass prepared on 2026-05-15.

## Scope shipped by this plan

- Dioxus desktop shell for the Editorial Reading Room design direction.
- Local-first bookmark import from CSV, JSON, and X archive JS.
- Dry-run import preview plus transactional import write path.
- Library/favorites reading layouts: Issue, Front Page, Long-Read, Spread.
- Search, author/topic directories, entry detail, filtered author/topic archive routes.
- Empty/onboarding, error states, honest session-only appearance settings, and session-only remote tweet image previews hidden by default.
- Service guardrails for import/query/favorite/delete/persistence/detail/directory APIs.
- Performance baseline and 10k budget regression test.
- Visual, accessibility, and performance OMX evidence artifacts.

## Final verification commands

Run from the repository root:

```bash
nix develop -c cargo fmt --all
nix develop -c cargo test --workspace
nix develop -c cargo check --workspace
nix develop -c cargo clippy --workspace --all-targets -- -A dead_code
nix develop -c cargo build -p eterea-dioxus
nix develop -c cargo test -p eterea-app --test performance_baseline -- --nocapture
```

## Evidence artifacts

| Area | Artifact | Status |
| --- | --- | --- |
| Planning | `.omx/plans/prd-eterea-full-app.md` | Complete |
| Test contract | `.omx/plans/test-spec-eterea-full-app.md` | Complete |
| Design map | `docs/design-implementation-map.md` | Complete |
| Accessibility | `.omx/artifacts/accessibility/eterea-full-app/checklist.md` | Static pass complete, including remote media states; live traversal remains final release check |
| Visual QA | `.omx/artifacts/visual/eterea-full-app/visual-qa-report.md` + verdicts | Accepted-deviation pass, including tweet media states; live screenshot/pixel diff deferred |
| Performance | `.omx/artifacts/perf/eterea-full-app/optimization-report.md` | 10k budgets pass; 50k/manual desktop-start waivers documented |
| Codex/OMX ledger | `.omx/ultragoal/ledger.jsonl` | Story checkpoints recorded |

## Known release risks / waivers

- Live Dioxus desktop screenshot capture was not performed in this shell; visual verdicts are accepted-deviation artifacts until a desktop capture pass updates them.
- First usable desktop shell time was not measured; final packaging should add a launch-time measurement on target hardware.
- 50k stress search budget is waived pending a committed or generated stress fixture strategy; 10k search currently has large headroom.
- Appearance settings and remote image loading are intentionally session-only. The UI labels this honestly and remote images stay hidden by default because enabling previews may fetch stored HTTPS image URLs.
- Direct X sync and browser companion/server mode remain out of scope for this release slice.

## Packaging notes

- Keep the `nix develop` path as the canonical verification environment because plain `cargo test --workspace` may miss system OpenSSL/pkg-config outside Nix on this host.
- The backend stores the default database at the platform app-data path from `dirs::data_local_dir()/eterea/bookmarks.db`.
- Imports skip duplicate tweet URLs and parse before writing, so unsupported/broken files should leave the archive unchanged.

## Go / no-go checklist

- [x] Workspace tests pass under Nix.
- [x] Workspace check passes under Nix.
- [x] Workspace clippy passes under Nix with the existing `-A dead_code` allowance.
- [x] Dioxus package builds under Nix.
- [x] Performance report exists with budgets met or waivers documented.
- [x] Accessibility artifact exists with static pass and live follow-ups.
- [x] Visual verdict artifacts exist for every canonical screen.
- [ ] Optional before public release: live Dioxus visual screenshot/pixel-diff pass.
- [ ] Optional before public release: target-platform installer/signing smoke test.
