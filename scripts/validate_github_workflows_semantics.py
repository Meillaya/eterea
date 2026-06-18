from __future__ import annotations

import re
from collections.abc import Sequence
from dataclasses import dataclass
from typing import Final

from scripts.validate_github_workflows_commands import commands_from_step, commands_from_steps

UPLOAD_ARTIFACT_ACTION: Final = "actions/upload-artifact@v7"
REQUIRED_ACTIONS: Final = (
    ("real checkout action step", "actions/checkout@v6"),
    ("real Nix installer action step", "cachix/install-nix-action@v31"),
    ("real Cargo cache action step", "actions/cache@v5"),
)
GH_TOKEN_RE: Final = re.compile(r"^GH_TOKEN:\s*\$\{\{\s*github\.token\s*\}\}\s*$")
YAML_KEY_RE: Final = re.compile(r"^(?P<indent> *)(?P<key>[A-Za-z0-9_-]+|'[^']+'|\"[^\"]+\"):(?P<value>.*)$")
CI_COMMANDS: Final = (("real cargo fmt command", "nix develop -c cargo fmt --all -- --check"), ("real cargo clippy command", "nix develop -c cargo clippy --workspace --all-targets -- -D warnings"), ("real cargo test command", "nix develop -c cargo test --workspace"), ("real Dioxus release build command", "nix develop -c cargo build -p eterea-dioxus --release"), ("real performance baseline command", "nix develop -c scripts/perf-baseline.sh"))


@dataclass(frozen=True, slots=True)
class Step:
    lines: tuple[str, ...]


def strip_yaml_comments(text: str) -> str:
    return "\n".join(filter(None, (line.partition("#")[0].rstrip() for line in text.splitlines())))


def top_level_block(text: str, heading: str) -> list[str]:
    lines = text.splitlines()
    collected: list[str] = []
    in_block = False
    for line in lines:
        if line == f"{heading}:":
            in_block = True
            continue
        if in_block and line and not line.startswith(" "):
            break
        if in_block:
            collected.append(line)
    return collected


def push_branch_includes_main(text: str) -> bool:
    on_lines = top_level_block(text, "on")
    in_push = False
    for line in on_lines:
        event_key = yaml_key_at_indent(line, 2)
        if event_key is not None and event_key[0] == "push":
            in_push = True
            continue
        if in_push and event_key is not None:
            break
        if in_push and line.strip() == "- main":
            return True
    return False


def push_tags_only_v_star(text: str) -> bool:
    on_lines = top_level_block(text, "on")
    in_push = False
    in_tags = False
    tag_values: list[str] = []
    for line in on_lines:
        event_key = yaml_key_at_indent(line, 2)
        if event_key is not None and event_key[0] == "push":
            in_push = True
            continue
        if in_push and event_key is not None:
            break
        if not in_push:
            continue
        stripped = line.strip()
        push_key = yaml_key_at_indent(line, 4)
        if push_key is not None:
            if push_key[0] != "tags" or in_tags:
                return False
            in_tags = True
            continue
        if in_tags and line.startswith("      - "):
            tag_values.append(stripped.removeprefix("-").strip().strip("\"'"))
            continue
        if in_tags and stripped:
            return False
    return tag_values == ["v*"]


def workflow_job_block(text: str, job_name: str) -> list[str]:
    target = strip_yaml_scalar_quotes(job_name)
    collected: list[str] = []
    in_block = False
    for line in top_level_block(text, "jobs"):
        job_key = yaml_key_at_indent(line, 2)
        if job_key is not None:
            if in_block:
                break
            if job_key[0] == target:
                in_block = True
                continue
        if in_block:
            collected.append(line)
    return collected


def release_job_has_contents_write_only(text: str) -> bool:
    lines = workflow_job_block(text, "release")
    permission_lines: list[str] = []
    in_permissions = False
    for line in lines:
        permission_heading = yaml_key_at_indent(line, 4)
        if permission_heading is not None and permission_heading[0] == "permissions":
            in_permissions = permission_heading[1] == ""
            continue
        if in_permissions and line.startswith("    ") and not line.startswith("      "):
            break
        permission = yaml_key_at_indent(line, 6)
        if in_permissions and permission is not None:
            permission_lines.append(f"{permission[0]}: {permission[1]}")
    return permission_lines == ["contents: write"]


def has_least_privilege_contents_read(text: str) -> bool:
    permission_lines = [
        f"{permission[0]}: {permission[1]}"
        for line in top_level_block(text, "permissions")
        if (permission := yaml_key_at_indent(line, 2)) is not None
    ]
    return permission_lines == ["contents: read"]


def draft_release_semantic_failures(text: str) -> list[str]:
    steps = release_steps(text)
    failures: list[str] = []
    for label, action in REQUIRED_ACTIONS:
        if not has_uses_step(steps, action):
            failures.append(f"missing {label}: {action}")
    if not has_upload_artifact_contract(steps):
        failures.append("missing real upload-artifact step with name, path, and retention-days")
    if not has_real_build_command(steps):
        failures.append("missing real Dioxus release build command")
    if not has_real_checksum_command(steps):
        failures.append("missing real checksum file generation command")
    if not has_real_provenance_write(steps):
        failures.append("missing real PROVENANCE.md file write command")
    if not has_real_draft_release_create(steps):
        failures.append("missing real gh release create --draft command with GH_TOKEN env")
    return failures


def release_steps(text: str) -> list[Step]:
    return job_steps(text, "release")


def ci_semantic_failures(text: str) -> list[str]:
    steps = job_steps(text, "rust-workspace")
    failures = [f"missing {label}: {action}" for label, action in REQUIRED_ACTIONS if not has_uses_step(steps, action)]
    if not has_cache_key_contract(steps):
        failures.append("missing real Cargo cache key")
    failures.extend(f"missing {label}: {command}" for label, command in CI_COMMANDS if command not in commands_from_steps(steps))
    return failures


def job_steps(text: str, job_name: str) -> list[Step]:
    job_lines = workflow_job_block(text, job_name)
    step_blocks: list[Step] = []
    current: list[str] = []
    in_steps = False
    for line in job_lines:
        if line == "    steps:":
            in_steps = True
            continue
        if in_steps and line.startswith("    ") and not line.startswith("      "):
            break
        if not in_steps:
            continue
        if line.startswith("      - "):
            if current:
                step_blocks.append(Step(tuple(current)))
            current = [line]
            continue
        if current:
            current.append(line)
    if current:
        step_blocks.append(Step(tuple(current)))
    return step_blocks


def has_uses_step(steps: Sequence[Step], action: str) -> bool:
    return any(any(line.strip() == f"uses: {action}" for line in step.lines) for step in steps)


def has_upload_artifact_contract(steps: Sequence[Step]) -> bool:
    for step in steps:
        if not any(line.strip() == f"uses: {UPLOAD_ARTIFACT_ACTION}" for line in step.lines):
            continue
        with_lines = mapping_block(step.lines, "with")
        has_name = has_mapping_key(with_lines, "name")
        has_path = has_mapping_key(with_lines, "path")
        has_retention = has_mapping_key(with_lines, "retention-days")
        if has_name and has_path and has_retention:
            return True
    return False


def has_cache_key_contract(steps: Sequence[Step]) -> bool:
    return any(any(line.strip() == "uses: actions/cache@v5" for line in step.lines) and any(line == "key: ${{ runner.os }}-cargo-${{ hashFiles('Cargo.lock') }}" for line in mapping_block(step.lines, "with")) for step in steps)


def has_real_build_command(steps: Sequence[Step]) -> bool:
    return any("nix develop -c cargo build -p eterea-dioxus --release" in command for command in commands_from_steps(steps))


def has_real_checksum_command(steps: Sequence[Step]) -> bool:
    return any("sha256sum" in command and ">" in command for command in commands_from_steps(steps))


def has_real_provenance_write(steps: Sequence[Step]) -> bool:
    return any("PROVENANCE.md" in command and ">" in command for command in commands_from_steps(steps))


def has_real_draft_release_create(steps: Sequence[Step]) -> bool:
    for step in steps:
        if not any(GH_TOKEN_RE.match(line.strip()) is not None for line in mapping_block(step.lines, "env")):
            continue
        for command in commands_from_step(step.lines):
            if command.startswith("gh release create ") and "--draft" in command.split() and not any(token.startswith("--dry-run") for token in command.split()):
                return True
    return False


def mapping_block(lines: Sequence[str], key: str) -> list[str]:
    block: list[str] = []
    in_block = False
    heading = f"        {key}:"
    for line in lines:
        if line == heading:
            in_block = True
            continue
        if in_block and line.startswith("        ") and not line.startswith("          "):
            break
        if in_block and line.strip():
            block.append(line.strip())
    return block


def has_mapping_key(lines: Sequence[str], key: str) -> bool:
    return any(line.startswith(f"{key}:") for line in lines)


def yaml_key_at_indent(line: str, indent: int) -> tuple[str, str] | None:
    match = YAML_KEY_RE.match(line)
    if match is None or len(match.group("indent")) != indent:
        return None
    return (strip_yaml_scalar_quotes(match.group("key")), strip_yaml_scalar_quotes(match.group("value").strip()))


def strip_yaml_scalar_quotes(value: str) -> str:
    if len(value) < 2 or value[0] != value[-1] or value[0] not in ("'", '"'):
        return value
    return value[1:-1]
