#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# From the repository root:
#   python scripts/test_validate_github_workflows.py

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Final

REPO_ROOT: Final = Path(__file__).resolve().parents[1]
VALIDATOR: Final = REPO_ROOT / "scripts" / "validate-github-workflows.py"
REAL_WORKFLOWS: Final = REPO_ROOT / ".github" / "workflows"
CI_NAME: Final = "ci.yml"
DRAFT_NAME: Final = "draft-release.yml"


@dataclass(frozen=True, slots=True)
class Case:
    name: str
    mutate: Callable[[Path], None]


@dataclass(frozen=True, slots=True)
class Result:
    name: str
    passed: bool
    output: str


def run_validator(workflow_dir: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(VALIDATOR), "--workflow-dir", str(workflow_dir)],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=20,
        check=False,
    )


def workflow_text(workflow_dir: Path, name: str) -> str:
    return (workflow_dir / name).read_text(encoding="utf-8")


def write_workflow(workflow_dir: Path, name: str, text: str) -> None:
    _ = (workflow_dir / name).write_text(text, encoding="utf-8")


def replace_once(text: str, old: str, new: str) -> str:
    if old not in text:
        raise AssertionError(f"fixture anchor missing: {old}")
    return text.replace(old, new, 1)


def copy_workflows(destination: Path) -> Path:
    workflow_dir = destination / "workflows"
    _ = shutil.copytree(REAL_WORKFLOWS, workflow_dir)
    return workflow_dir


def comment_only_ci(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, CI_NAME)
    write_workflow(
        workflow_dir,
        CI_NAME,
        replace_once(text, "run: nix develop -c cargo test --workspace", "run: '# nix develop -c cargo test --workspace'"),
    )


def echo_only_ci(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, CI_NAME)
    write_workflow(
        workflow_dir,
        CI_NAME,
        replace_once(text, "run: nix develop -c cargo test --workspace", "run: echo 'nix develop -c cargo test --workspace'"),
    )


def if_false_ci(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, CI_NAME)
    write_workflow(
        workflow_dir,
        CI_NAME,
        replace_once(
            text,
            "run: nix develop -c cargo test --workspace",
            "run: |\n          if false; then\n            nix develop -c cargo test --workspace\n          fi",
        ),
    )


def if_zero_eq_one_ci(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, CI_NAME)
    write_workflow(
        workflow_dir,
        CI_NAME,
        replace_once(
            text,
            "run: nix develop -c cargo clippy --workspace --all-targets -- -D warnings",
            "run: |\n          if [ 0 -eq 1 ]; then\n            nix develop -c cargo clippy --workspace --all-targets -- -D warnings\n          fi",
        ),
    )


def inert_function_ci(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, CI_NAME)
    write_workflow(
        workflow_dir,
        CI_NAME,
        replace_once(
            text,
            "run: nix develop -c cargo test --workspace",
            "run: |\n          pretend_success() {\n            nix develop -c cargo test --workspace\n          }",
        ),
    )


def ci_job_contents_write(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, CI_NAME)
    write_workflow(workflow_dir, CI_NAME, replace_once(text, "timeout-minutes: 45", "timeout-minutes: 45\n    permissions:\n      contents: write"))


def ci_job_quoted_contents_write(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, CI_NAME)
    write_workflow(workflow_dir, CI_NAME, replace_once(text, "timeout-minutes: 45", "timeout-minutes: 45\n    permissions:\n      contents: \"write\""))


def ci_job_quoted_packages_write(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, CI_NAME)
    write_workflow(workflow_dir, CI_NAME, replace_once(text, "timeout-minutes: 45", "timeout-minutes: 45\n    permissions:\n      packages: 'write'"))


def ci_packages_write(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, CI_NAME)
    write_workflow(workflow_dir, CI_NAME, replace_once(text, "permissions:\n  contents: read", "permissions:\n  contents: read\n  packages: write"))


def echo_only_draft(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    replacement = "run: echo 'gh release create ${GITHUB_REF_NAME} --draft'"
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "run: |\n          gh release create", f"{replacement}\n          #"))


def draft_dry_run(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "--draft \\", "--draft --dry-run \\"))


def if_false_draft(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(
        workflow_dir,
        DRAFT_NAME,
        replace_once(text, "run: |\n          gh release create", "run: |\n          if test 0 -eq 1; then\n            gh release create"),
    )


def inert_function_draft(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    text = replace_once(text, "run: |\n          gh release create", "run: |\n          create_release() {\n            gh release create")
    text = replace_once(
        text,
        '            --notes "Draft release generated from ${GITHUB_SHA}. See artifact PROVENANCE.md and SHA256SUMS.txt inside the archive."',
        '            --notes "Draft release generated from ${GITHUB_SHA}. See artifact PROVENANCE.md and SHA256SUMS.txt inside the archive."\n          }',
    )
    write_workflow(workflow_dir, DRAFT_NAME, text)


def inline_extra_trigger(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "on:\n  workflow_dispatch:", "on:\n  pull_request: {}\n  workflow_dispatch:"))


def quoted_extra_trigger_single(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "on:\n  workflow_dispatch:", "on:\n  'pull_request': {}\n  workflow_dispatch:"))


def quoted_extra_trigger_double(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "on:\n  workflow_dispatch:", 'on:\n  "pull_request": {}\n  workflow_dispatch:'))


def block_extra_pull_request(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "on:\n  workflow_dispatch:", "on:\n  pull_request:\n  workflow_dispatch:"))


def extra_tag_pattern(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "      - \"v*\"", "      - \"v*\"\n      - \"release-*\""))


def broad_star_tag(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "      - \"v*\"", "      - \"*\""))


def top_level_write_permission(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "permissions:\n  contents: read", "permissions:\n  contents: write"))


def non_release_job_packages_write(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    inserted = "jobs:\n  audit:\n    runs-on: ubuntu-latest\n    permissions:\n      packages: write\n    steps:\n      - run: true\n"
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "jobs:\n", inserted))


def non_release_job_quoted_packages_write(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    inserted = "jobs:\n  audit:\n    runs-on: ubuntu-latest\n    permissions:\n      packages: 'write'\n    steps:\n      - run: true\n"
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "jobs:\n", inserted))


def quoted_extra_job_packages_write(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    inserted = 'jobs:\n  "audit":\n    runs-on: ubuntu-latest\n    permissions:\n      packages: write\n    steps:\n      - run: true\n'
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "jobs:\n", inserted))


def release_job_packages_write(workflow_dir: Path) -> None:
    text = workflow_text(workflow_dir, DRAFT_NAME)
    write_workflow(workflow_dir, DRAFT_NAME, replace_once(text, "contents: write", "contents: write\n      packages: write"))


CASES: Final = (
    Case("comment-only CI", comment_only_ci),
    Case("echo-only CI", echo_only_ci),
    Case("if-false CI", if_false_ci),
    Case("if [ 0 -eq 1 ] CI", if_zero_eq_one_ci),
    Case("inert shell function CI", inert_function_ci),
    Case("CI job-level contents:write", ci_job_contents_write),
    Case("CI job-level quoted contents:write", ci_job_quoted_contents_write),
    Case("CI job-level quoted packages:write", ci_job_quoted_packages_write),
    Case("CI top-level packages:write", ci_packages_write),
    Case("echo-only draft", echo_only_draft),
    Case("draft dry-run/no-op", draft_dry_run),
    Case("if-false draft", if_false_draft),
    Case("inert shell function draft", inert_function_draft),
    Case("inline extra trigger", inline_extra_trigger),
    Case("single-quoted extra trigger", quoted_extra_trigger_single),
    Case("double-quoted extra trigger", quoted_extra_trigger_double),
    Case("block extra pull_request trigger", block_extra_pull_request),
    Case("extra tag pattern", extra_tag_pattern),
    Case("broad star tag", broad_star_tag),
    Case("top-level write permission", top_level_write_permission),
    Case("non-release packages:write", non_release_job_packages_write),
    Case("non-release quoted packages:write", non_release_job_quoted_packages_write),
    Case("quoted extra job packages:write", quoted_extra_job_packages_write),
    Case("release job packages:write", release_job_packages_write),
)


def run_case(case: Case) -> Result:
    with tempfile.TemporaryDirectory(prefix="workflow-validator-") as temp_dir:
        workflow_dir = copy_workflows(Path(temp_dir))
        case.mutate(workflow_dir)
        completed = run_validator(workflow_dir)
    passed = completed.returncode != 0 and "GitHub workflow validation: FAIL" in completed.stdout
    return Result(case.name, passed, completed.stdout)


def run_all(cases: Sequence[Case]) -> list[Result]:
    with tempfile.TemporaryDirectory(prefix="workflow-validator-real-") as temp_dir:
        workflow_dir = copy_workflows(Path(temp_dir))
        real = run_validator(workflow_dir)
    results = [Result("real workflows pass", real.returncode == 0, real.stdout)]
    results.extend(run_case(case) for case in cases)
    return results


def main() -> int:
    results = run_all(CASES)
    for result in results:
        status = "PASS" if result.passed else "FAIL"
        print(f"{status}: {result.name}")
        if not result.passed:
            print(result.output)
    return 0 if all(result.passed for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
