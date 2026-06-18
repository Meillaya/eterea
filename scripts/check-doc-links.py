#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

# ─── How to run ───
# 1. Install uv (optional): curl -LsSf https://astral.sh/uv/install.sh | sh
# 2. Run directly with Python and one or more markdown paths/globs:
#      python scripts/check-doc-links.py README.md docs/**/*.md
# 3. Or make executable and run:
#      chmod +x scripts/check-doc-links.py && ./scripts/check-doc-links.py README.md
# ──────────────────

from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime
from html import unescape
from pathlib import Path
from typing import Final
from urllib.parse import unquote, urlsplit
import glob
import re
import sys

EXTERNAL_SCHEMES: Final[frozenset[str]] = frozenset({"http", "https", "mailto"})
MARKDOWN_SUFFIXES: Final[frozenset[str]] = frozenset({".md", ".markdown"})
USAGE: Final[str] = "Usage: python scripts/check-doc-links.py <markdown-file-or-glob> [...]"

INLINE_LINK_RE: Final[re.Pattern[str]] = re.compile(r"(!?)\[[^\]\n]*\]\(([^)\n]*)\)")
REF_USE_RE: Final[re.Pattern[str]] = re.compile(r"(!?)\[([^\]\n]+)\]\[([^\]\n]*)\]")
REF_DEF_RE: Final[re.Pattern[str]] = re.compile(r"^ {0,3}\[([^\]]+)\]:\s*(.+?)\s*$")
HEADING_RE: Final[re.Pattern[str]] = re.compile(r"^ {0,3}(#{1,6})\s+(.+?)\s*#*\s*$")
HTML_ANCHOR_RE: Final[re.Pattern[str]] = re.compile(
    r"<a\s+[^>]*(?:id|name)=(?:\"([^\"]+)\"|'([^']+)'|([^\s>]+))",
    re.IGNORECASE,
)
HTML_TAG_RE: Final[re.Pattern[str]] = re.compile(r"<[^>]+>")
NON_SLUG_RE: Final[re.Pattern[str]] = re.compile(r"[^a-z0-9 _-]")
SPACE_RE: Final[re.Pattern[str]] = re.compile(r"\s+")


@dataclass(frozen=True, slots=True)
class LinkCandidate:
    source: Path
    line: int
    target: str
    kind: str


@dataclass(frozen=True, slots=True)
class LinkIssue:
    source: Path
    line: int
    target: str
    kind: str
    reason: str


@dataclass(frozen=True, slots=True)
class ScanReport:
    scanned: tuple[Path, ...]
    skipped_inputs: tuple[str, ...]
    ignored_external: tuple[str, ...]
    issues: tuple[LinkIssue, ...]


def has_glob_meta(value: str) -> bool:
    """Return whether a CLI token should be expanded by Python glob."""
    return "*" in value or "?" in value or "[" in value


def expand_inputs(tokens: list[str]) -> tuple[tuple[Path, ...], tuple[str, ...]]:
    """Expand CLI globs and skip literal missing inputs with a visible note."""
    paths: list[Path] = []
    skipped: list[str] = []
    for token in tokens:
        if has_glob_meta(token):
            matches = [Path(match) for match in glob.glob(token, recursive=True)]
            files = sorted(path for path in matches if path.is_file())
            if files:
                paths.extend(files)
            else:
                skipped.append(f"{token} (unmatched glob input skipped)")
            continue
        path = Path(token)
        if path.is_file():
            paths.append(path)
        elif path.exists():
            skipped.append(f"{token} (not a regular file; skipped)")
        else:
            skipped.append(f"{token} (missing literal input skipped)")
    return tuple(dict.fromkeys(paths)), tuple(skipped)


def label_key(value: str) -> str:
    """Normalize a Markdown reference label."""
    return SPACE_RE.sub(" ", value.strip()).casefold()


def split_destination(raw_value: str) -> str:
    """Parse the destination portion before an optional Markdown title."""
    value = raw_value.strip()
    if not value:
        return ""
    if value.startswith("<"):
        end = value.find(">")
        if end >= 0:
            return value[1:end].strip()
    return value.split(maxsplit=1)[0].strip()


def line_is_fence(line: str) -> bool:
    """Return whether a line opens or closes a fenced code block."""
    stripped = line.lstrip()
    return stripped.startswith("```") or stripped.startswith("~~~")


def collect_reference_defs(path: Path) -> dict[str, str]:
    """Collect Markdown reference definitions from a file."""
    refs: dict[str, str] = {}
    in_fence = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if line_is_fence(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        match = REF_DEF_RE.match(line)
        if match:
            refs[label_key(match.group(1))] = split_destination(match.group(2))
    return refs


def collect_candidates(path: Path) -> tuple[LinkCandidate, ...]:
    """Collect inline and reference Markdown link/image candidates."""
    refs = collect_reference_defs(path)
    candidates: list[LinkCandidate] = []
    in_fence = False
    for index, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if line_is_fence(line):
            in_fence = not in_fence
            continue
        if in_fence or REF_DEF_RE.match(line):
            continue
        for match in INLINE_LINK_RE.finditer(line):
            kind = "image" if match.group(1) else "link"
            candidates.append(LinkCandidate(path, index, split_destination(match.group(2)), kind))
        for match in REF_USE_RE.finditer(line):
            kind = "image" if match.group(1) else "link"
            label = match.group(3) or match.group(2)
            target = refs.get(label_key(label), "")
            candidates.append(LinkCandidate(path, index, target, kind))
    return tuple(candidates)


def slugify_heading(text: str) -> str:
    """Create a GitHub-style slug for a Markdown heading."""
    without_tags = HTML_TAG_RE.sub("", unescape(text)).strip().casefold()
    without_punctuation = NON_SLUG_RE.sub("", without_tags)
    return SPACE_RE.sub("-", without_punctuation.strip())


def anchors_for(path: Path) -> frozenset[str]:
    """Return generated heading anchors and explicit HTML anchors for a Markdown file."""
    anchors: set[str] = set()
    seen_slugs: dict[str, int] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        heading = HEADING_RE.match(line)
        if heading:
            base_slug = slugify_heading(heading.group(2))
            count = seen_slugs.get(base_slug, 0)
            seen_slugs[base_slug] = count + 1
            anchors.add(base_slug if count == 0 else f"{base_slug}-{count}")
        for match in HTML_ANCHOR_RE.finditer(line):
            value = next(group for group in match.groups() if group is not None)
            anchors.add(value)
            anchors.add(value.casefold())
    return frozenset(anchors)


def target_path(root: Path, source: Path, raw_path: str) -> Path:
    """Resolve a local Markdown target relative to source or repository root."""
    decoded = unquote(raw_path)
    if decoded.startswith("/"):
        return root / decoded.lstrip("/")
    return source.parent / decoded


def check_candidate(root: Path, candidate: LinkCandidate, anchor_cache: dict[Path, frozenset[str]]) -> LinkIssue | str | None:
    """Check one link, returning an issue, ignored external marker, or success."""
    if not candidate.target:
        return LinkIssue(candidate.source, candidate.line, candidate.target, candidate.kind, "empty target")
    parsed = urlsplit(candidate.target)
    scheme = parsed.scheme.casefold()
    if scheme in EXTERNAL_SCHEMES:
        return f"{candidate.source}:{candidate.line}: {candidate.kind} {candidate.target}"
    if scheme or parsed.netloc:
        return f"{candidate.source}:{candidate.line}: {candidate.kind} {candidate.target}"
    local_path = target_path(root, candidate.source, parsed.path) if parsed.path else candidate.source
    if not local_path.exists():
        noun = "image target" if candidate.kind == "image" else "link target"
        return LinkIssue(candidate.source, candidate.line, candidate.target, candidate.kind, f"missing {noun}: {local_path}")
    fragment = unquote(parsed.fragment)
    if fragment and local_path.suffix.casefold() in MARKDOWN_SUFFIXES:
        anchors = anchor_cache.setdefault(local_path, anchors_for(local_path))
        wanted = fragment.casefold()
        if fragment not in anchors and wanted not in anchors:
            return LinkIssue(candidate.source, candidate.line, candidate.target, candidate.kind, f"missing anchor #{fragment} in {local_path}")
    return None


def scan(paths: tuple[Path, ...], skipped: tuple[str, ...], root: Path) -> ScanReport:
    """Scan Markdown inputs and return all broken local targets."""
    issues: list[LinkIssue] = []
    external: list[str] = []
    anchor_cache: dict[Path, frozenset[str]] = {}
    for path in paths:
        for candidate in collect_candidates(path):
            result = check_candidate(root, candidate, anchor_cache)
            match result:
                case LinkIssue():
                    issues.append(result)
                case str():
                    external.append(result)
                case None:
                    continue
    return ScanReport(paths, skipped, tuple(external), tuple(issues))


def print_report(report: ScanReport) -> None:
    """Print a deterministic, grep-friendly scan report."""
    print(f"DOC_LINK_CHECK generated_at={datetime.now(UTC).isoformat()}")
    print(f"SCANNED markdown_files={len(report.scanned)}")
    for path in report.scanned:
        print(f"SCAN {path}")
    for skipped in report.skipped_inputs:
        print(f"SKIP {skipped}")
    print(f"IGNORED_EXTERNAL count={len(report.ignored_external)}")
    for external in report.ignored_external:
        print(f"EXTERNAL {external}")
    if report.issues:
        print(f"FAIL broken_local_targets={len(report.issues)}")
        for issue in report.issues:
            print(f"BROKEN {issue.source}:{issue.line}: {issue.kind} {issue.target!r} - {issue.reason}")
    else:
        print("PASS broken_local_targets=0")


def main(argv: list[str]) -> int:
    """Run the Markdown local-link checker."""
    if not argv or argv == ["--help"] or argv == ["-h"]:
        print(USAGE)
        print("Checks local Markdown links, anchors, and image targets; ignores external URLs.")
        return 0 if argv else 2
    paths, skipped = expand_inputs(argv)
    report = scan(paths, skipped, Path.cwd())
    print_report(report)
    return 1 if report.issues else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
