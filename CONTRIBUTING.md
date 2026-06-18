# Contributing

Thanks for helping improve Eterea. Keep changes small, evidence-backed, and safe
for a shared dirty worktree.

## Development environment

Use the Nix shell as the canonical environment:

```bash
nix develop
```

Plain `cargo` can fail on hosts without system OpenSSL/pkg-config metadata. See
[docs/development.md](docs/development.md) for the complete workflow.

## Required local commands

Before proposing a change, run the commands relevant to the files you touched;
for release-facing changes, run the full set:

```bash
nix develop -c cargo fmt --all -- --check
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo test --workspace
python scripts/check-doc-links.py README.md 'docs/**/*.md' CHANGELOG.md CONTRIBUTING.md SECURITY.md CODE_OF_CONDUCT.md
```

For desktop or release work, also follow the manual QA notes in
[docs/operations/release-readiness.md](docs/operations/release-readiness.md).

## TDD and failing-first expectations

- For behavior changes, capture current behavior first when it is not already
  protected.
- Add a failing test or a failing manual reproduction before implementation.
- Prefer assertions against observable behavior over tests that merely mirror an
  implementation detail.
- Fix the smallest surface that makes the failing proof pass, then rerun the
  relevant verification commands.

## Dirty-worktree care

This repository is often worked on by multiple agents or contributors at once.

- Check `git status --short` before editing.
- Do not revert, reformat, or overwrite files outside your task.
- Keep evidence and generated artifacts in their documented locations.
- If another person's change blocks yours, report the exact file and conflict
  instead of guessing or cleaning it up.

## Commit message protocol

Use the Lore commit style: the first line should explain why the change was
made, not just what changed. Add trailers when they preserve useful context:

```text
Constraint: external constraint that shaped the decision
Rejected: alternative considered | reason it was not used
Confidence: low|medium|high
Scope-risk: narrow|moderate|broad
Directive: warning for future maintainers
Tested: exact commands or scenarios verified
Not-tested: known gaps
```

Do not claim unrun checks. If a check is skipped, record why.
