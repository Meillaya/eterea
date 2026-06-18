#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# From the repository root:
#   python scripts/validate-github-workflows.py
# For an isolated fixture directory:
#   python scripts/validate-github-workflows.py --workflow-dir /tmp/workflows

from __future__ import annotations

import re
import sys
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Final

PROJECT_ROOT: Final = Path(__file__).resolve().parents[1]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.validate_github_workflows_semantics import (
    ci_semantic_failures,
    draft_release_semantic_failures,
    has_least_privilege_contents_read,
    push_branch_includes_main,
    push_tags_only_v_star,
    release_job_has_contents_write_only,
    strip_yaml_comments,
    top_level_block,
    workflow_job_block,
    yaml_key_at_indent,
)
from scripts.validate_github_workflows_syntax import draft_release_triggers_are_exact

DEFAULT_WORKFLOW_DIR: Final = Path(".github/workflows")
CI_FILE_NAME: Final = "ci.yml"
DRAFT_RELEASE_FILE_NAME: Final = "draft-release.yml"


@dataclass(frozen=True, slots=True)
class LiteralRequirement:
    label: str
    needle: str


@dataclass(frozen=True, slots=True)
class RegexRequirement:
    label: str
    pattern: re.Pattern[str]


@dataclass(frozen=True, slots=True)
class WorkflowSpec:
    file_name: str
    display_name: str
    literals: tuple[LiteralRequirement, ...]
    regexes: tuple[RegexRequirement, ...]
    forbidden_literals: tuple[LiteralRequirement, ...]


@dataclass(frozen=True, slots=True)
class ValidationFailure:
    workflow: str
    message: str


class UsageError(Exception):
    pass


class HelpRequested(Exception):
    pass


CI_SPEC: Final = WorkflowSpec(
    file_name=CI_FILE_NAME,
    display_name="CI",
    literals=(
        LiteralRequirement("checkout action pinned to T2 major", "actions/checkout@v6"),
        LiteralRequirement("Nix installer action pinned to T2 major", "cachix/install-nix-action@v31"),
        LiteralRequirement("Cargo cache action pinned to T2 major", "actions/cache@v5"),
        LiteralRequirement("cache key includes OS and Cargo.lock", "${{ runner.os }}-cargo-${{ hashFiles('Cargo.lock') }}"),
        LiteralRequirement("format command mirrors local QA", "nix develop -c cargo fmt --all -- --check"),
        LiteralRequirement("clippy command mirrors local QA", "nix develop -c cargo clippy --workspace --all-targets -- -D warnings"),
        LiteralRequirement("test command mirrors local QA", "nix develop -c cargo test --workspace"),
        LiteralRequirement("Dioxus release build command", "nix develop -c cargo build -p eterea-dioxus --release"),
        LiteralRequirement("performance baseline command", "nix develop -c scripts/perf-baseline.sh"),
    ),
    regexes=(
        RegexRequirement("workflow name is CI", re.compile(r"(?m)^name:\s*CI\s*$")),
        RegexRequirement("pull_request trigger exists", re.compile(r"(?m)^\s{2}[\"']?pull_request[\"']?:\s*$")),
        RegexRequirement("push trigger exists", re.compile(r"(?m)^\s{2}[\"']?push[\"']?:\s*$")),
        RegexRequirement("workflow_dispatch trigger exists", re.compile(r"(?m)^\s{2}[\"']?workflow_dispatch[\"']?:\s*$")),
        RegexRequirement("runner is GitHub-hosted Ubuntu", re.compile(r"(?m)^\s{4}runs-on:\s*ubuntu-latest\s*$")),
    ),
    forbidden_literals=(
        LiteralRequirement("workflow must not reference repository secrets", "secrets."),
        LiteralRequirement("workflow must not request write permissions", "write"),
        LiteralRequirement("workflow must not require id-token permission", "id-token:"),
    ),
)

DRAFT_RELEASE_SPEC: Final = WorkflowSpec(
    file_name=DRAFT_RELEASE_FILE_NAME,
    display_name="Draft Release",
    literals=(
        LiteralRequirement("checkout action pinned to T2 major", "actions/checkout@v6"),
        LiteralRequirement("Nix installer action pinned to T2 major", "cachix/install-nix-action@v31"),
        LiteralRequirement("Cargo cache action pinned to T2 major", "actions/cache@v5"),
        LiteralRequirement("artifact upload action pinned to T2 major", "actions/upload-artifact@v7"),
        LiteralRequirement("Dioxus release build command", "nix develop -c cargo build -p eterea-dioxus --release"),
        LiteralRequirement("artifact retention is explicit", "retention-days:"),
        LiteralRequirement("checksums are generated", "sha256sum"),
        LiteralRequirement("artifact provenance is included", "PROVENANCE.md"),
        LiteralRequirement("GitHub release is created by GitHub CLI", "gh release create"),
        LiteralRequirement("release remains a draft", "--draft"),
        LiteralRequirement("workflow uses ephemeral GitHub token", "GH_TOKEN: ${{ github.token }}"),
    ),
    regexes=(
        RegexRequirement("workflow name is Draft Release", re.compile(r"(?m)^name:\s*Draft Release\s*$")),
        RegexRequirement("workflow_dispatch trigger exists", re.compile(r"(?m)^\s{2}[\"']?workflow_dispatch[\"']?:\s*$")),
        RegexRequirement("push trigger exists", re.compile(r"(?m)^\s{2}[\"']?push[\"']?:\s*$")),
        RegexRequirement("tag trigger is scoped to v*", re.compile(r"(?m)^\s{6}-\s*[\"']?v\*[\"']?\s*$")),
        RegexRequirement("runner is GitHub-hosted Ubuntu", re.compile(r"(?m)^\s{4}runs-on:\s*ubuntu-latest\s*$")),
    ),
    forbidden_literals=(
        LiteralRequirement("workflow must not reference repository secrets", "secrets."),
        LiteralRequirement("workflow must not publish a non-draft release", "--draft=false"),
        LiteralRequirement("workflow must not notarize artifacts", "notarytool"),
        LiteralRequirement("workflow must not sign artifacts", "cosign sign"),
    ),
)

WORKFLOW_SPECS: Final = (CI_SPEC, DRAFT_RELEASE_SPEC)


def usage() -> str:
    return "Usage: python scripts/validate-github-workflows.py [--workflow-dir PATH]"


def parse_args(argv: Sequence[str]) -> Path:
    workflow_dir = DEFAULT_WORKFLOW_DIR
    index = 0
    while index < len(argv):
        arg = argv[index]
        if arg in ("--help", "-h"):
            raise HelpRequested
        if arg == "--workflow-dir":
            next_index = index + 1
            if next_index >= len(argv):
                raise UsageError("--workflow-dir requires a path")
            workflow_dir = Path(argv[next_index])
            index += 2
            continue
        raise UsageError(f"unknown argument: {arg}")
    return workflow_dir


def validate_workflow(spec: WorkflowSpec, workflow_dir: Path) -> list[ValidationFailure]:
    path = workflow_dir / spec.file_name
    if not path.is_file():
        return [ValidationFailure(spec.file_name, f"missing workflow file: {path}")]

    text = strip_yaml_comments(path.read_text(encoding="utf-8"))
    failures: list[ValidationFailure] = []

    for requirement in spec.literals:
        if requirement.needle not in text:
            failures.append(ValidationFailure(spec.file_name, f"missing {requirement.label}: {requirement.needle}"))

    for requirement in spec.regexes:
        if requirement.pattern.search(text) is None:
            failures.append(ValidationFailure(spec.file_name, f"missing {requirement.label}"))

    permission_text = "\n".join(top_level_block(text, "permissions"))
    for requirement in spec.forbidden_literals:
        haystack = permission_text if requirement.needle in ("write", "id-token:") else text
        if requirement.needle in haystack:
            failures.append(ValidationFailure(spec.file_name, f"forbidden {requirement.label}: {requirement.needle}"))

    if not has_least_privilege_contents_read(text):
        failures.append(ValidationFailure(spec.file_name, "missing top-level permissions are least privilege"))
    if spec.file_name == CI_FILE_NAME and not push_branch_includes_main(text):
        failures.append(ValidationFailure(spec.file_name, "missing push branch includes main"))
    if spec.file_name == CI_FILE_NAME:
        for message in ci_semantic_failures(text):
            failures.append(ValidationFailure(spec.file_name, message))
    for message in job_level_write_permission_failures(text, spec.file_name):
        failures.append(ValidationFailure(spec.file_name, message))
    if spec.file_name == DRAFT_RELEASE_FILE_NAME:
        if not draft_release_triggers_are_exact(text):
            failures.append(ValidationFailure(spec.file_name, "draft release triggers must be exactly workflow_dispatch and push tags v*"))
        if not push_tags_only_v_star(text):
            failures.append(ValidationFailure(spec.file_name, "push trigger must contain only v* tags"))
        if not release_job_has_contents_write_only(text):
            failures.append(ValidationFailure(spec.file_name, "release job permissions must be contents: write only"))
        for message in draft_release_semantic_failures(text):
            failures.append(ValidationFailure(spec.file_name, message))

    return failures


def workflow_job_names(text: str) -> list[str]:
    return [
        job_key[0]
        for line in top_level_block(text, "jobs")
        if (job_key := yaml_key_at_indent(line, 2)) is not None
    ]


def job_level_write_permission_failures(text: str, file_name: str) -> list[str]:
    failures: list[str] = []
    for job_name in workflow_job_names(text):
        for permission in job_level_write_permissions(workflow_job_block(text, job_name)):
            if file_name == DRAFT_RELEASE_FILE_NAME and job_name == "release" and permission == "contents: write":
                continue
            failures.append(f"job must not request write permission: {job_name} {permission}")
    return failures


def job_level_write_permissions(lines: Sequence[str]) -> list[str]:
    writes: list[str] = []
    in_permissions = False
    for line in lines:
        permission_heading = yaml_key_at_indent(line, 4)
        if permission_heading is not None and permission_heading[0] == "permissions":
            value = permission_heading[1]
            if value and "write" in value:
                writes.append(f"permissions: {value}")
            in_permissions = value == ""
            continue
        if in_permissions and line.startswith("    ") and not line.startswith("      "):
            return writes
        permission = yaml_key_at_indent(line, 6)
        if in_permissions and permission is not None and permission[1] == "write":
            writes.append(f"{permission[0]}: {permission[1]}")
    return writes


def validate(workflow_dir: Path) -> list[ValidationFailure]:
    failures: list[ValidationFailure] = []
    for spec in WORKFLOW_SPECS:
        failures.extend(validate_workflow(spec, workflow_dir))
    return failures


def print_success(workflow_dir: Path) -> None:
    print("GitHub workflow validation: PASS")
    print(f"workflow_dir={workflow_dir}")
    for spec in WORKFLOW_SPECS:
        print(f"validated={spec.file_name} name={spec.display_name}")


def print_failures(failures: Sequence[ValidationFailure]) -> None:
    print("GitHub workflow validation: FAIL")
    for failure in failures:
        print(f"{failure.workflow}: {failure.message}")


def main(argv: Sequence[str]) -> int:
    try:
        config = parse_args(argv)
    except HelpRequested:
        print(usage())
        return 0
    except UsageError as error:
        print(usage(), file=sys.stderr)
        print(f"error: {error}", file=sys.stderr)
        return 2

    failures = validate(config)
    if failures:
        print_failures(failures)
        return 1
    print_success(config)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
