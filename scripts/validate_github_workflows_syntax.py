#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# Import-only helper for scripts/validate-github-workflows.py.

from __future__ import annotations

from typing import Final

from scripts.validate_github_workflows_semantics import push_tags_only_v_star, top_level_block, yaml_key_at_indent

DRAFT_RELEASE_EVENTS: Final = frozenset(("workflow_dispatch", "push"))


def draft_release_triggers_are_exact(text: str) -> bool:
    event_values: dict[str, str] = {}
    for line in top_level_block(text, "on"):
        event_key = yaml_key_at_indent(line, 2)
        if event_key is None:
            continue
        event_name, value = event_key
        if event_name not in DRAFT_RELEASE_EVENTS or event_name in event_values:
            return False
        event_values[event_name] = value
    return event_values == {"workflow_dispatch": "", "push": ""} and push_tags_only_v_star(text)
