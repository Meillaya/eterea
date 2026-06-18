#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# Import-only helper for scripts/validate_github_workflows_semantics.py.

from __future__ import annotations

import re
from collections.abc import Sequence
from typing import Final, Protocol

FALSE_IF_PATTERN: Final = (
    r"^\s*if\s+(?:false|!\s+true|\[\s*(?:0\s+-eq\s+1|1\s+-eq\s+0)\s*\]|test\s+(?:0\s+-eq\s+1|1\s+-eq\s+0))"
    + r"(?:\s*;?\s*then\b)?"
)
FALSE_IF_RE: Final = re.compile(FALSE_IF_PATTERN)
IF_RE: Final = re.compile(r"^\s*if\b")
FI_RE: Final = re.compile(r"^\s*fi\b")
ELSE_RE: Final = re.compile(r"^\s*else\b")
ECHO_RE: Final = re.compile(r"^\s*(?:echo|printf)\b")
HEREDOC_RE: Final = re.compile(r"<<-?\s*['\"]?([A-Za-z_][A-Za-z0-9_]*)['\"]?")
SHELL_FUNCTION_RE: Final = re.compile(
    r"^(?:function\s+[A-Za-z_][A-Za-z0-9_]*(?:\s*\(\s*\))?\s*\{|[A-Za-z_][A-Za-z0-9_]*\s*\(\s*\)\s*\{)"
)


class StepLike(Protocol):
    @property
    def lines(self) -> tuple[str, ...]: ...


def commands_from_steps(steps: Sequence[StepLike]) -> list[str]:
    commands: list[str] = []
    for step in steps:
        commands.extend(commands_from_step(step.lines))
    return commands


def commands_from_step(lines: Sequence[str]) -> list[str]:
    commands: list[str] = []
    pending = ""
    for line in executable_script_lines(run_script_lines(lines)):
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.endswith("\\"):
            pending = f"{pending}{stripped[:-1]} "
            continue
        commands.append(f"{pending}{stripped}".strip())
        pending = ""
    if pending:
        commands.append(pending.strip())
    return commands


def run_script_lines(lines: Sequence[str]) -> list[str]:
    run_lines: list[str] = []
    in_run = False
    for line in lines:
        if line.startswith("        run:"):
            value = line.partition("run:")[2].strip()
            if value and value not in ("|", ">"):
                run_lines.append(value)
            in_run = value in ("", "|", ">")
            continue
        if in_run and line.startswith("        ") and not line.startswith("          "):
            break
        if in_run and line.startswith("          "):
            run_lines.append(line[10:])
    return run_lines


def executable_script_lines(lines: Sequence[str]) -> list[str]:
    executable: list[str] = []
    heredoc_marker = ""
    false_if_depth = 0
    function_depth = 0
    for line in lines:
        stripped = line.strip()
        if heredoc_marker:
            if stripped == heredoc_marker:
                heredoc_marker = ""
            continue
        if function_depth > 0:
            function_depth += shell_brace_delta(stripped)
            if function_depth < 0:
                function_depth = 0
            continue
        if false_if_depth > 0:
            false_if_depth = next_false_if_depth(false_if_depth, stripped)
            continue
        if is_ignored_script_line(stripped):
            continue
        if starts_shell_function(stripped):
            function_depth = shell_brace_delta(stripped)
            if function_depth < 0:
                function_depth = 0
            continue
        if starts_obvious_false_if(stripped):
            false_if_depth = 0 if inline_if_is_closed(stripped) else 1
            continue
        executable.append(line)
        marker = heredoc_end_marker(stripped)
        if marker:
            heredoc_marker = marker
    return executable


def next_false_if_depth(depth: int, stripped: str) -> int:
    if ELSE_RE.match(stripped) is not None and depth == 1:
        return 0
    if IF_RE.match(stripped) is not None:
        return depth + 1
    if FI_RE.match(stripped) is not None:
        return depth - 1
    return depth


def is_ignored_script_line(stripped: str) -> bool:
    return not stripped or stripped.startswith("#") or ECHO_RE.match(stripped) is not None


def starts_obvious_false_if(stripped: str) -> bool:
    return FALSE_IF_RE.match(stripped) is not None


def inline_if_is_closed(stripped: str) -> bool:
    return "; fi" in stripped or stripped.endswith(";fi")


def heredoc_end_marker(command: str) -> str:
    match = HEREDOC_RE.search(command)
    if match is None:
        return ""
    return match.group(1)


def starts_shell_function(stripped: str) -> bool:
    return SHELL_FUNCTION_RE.match(stripped) is not None


def shell_brace_delta(stripped: str) -> int:
    return stripped.count("{") - stripped.count("}")
