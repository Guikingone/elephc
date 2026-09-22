"""Reject movable third-party GitHub Action refs in workflows and composite actions."""

from __future__ import annotations

import re
import sys
from pathlib import Path


USES_LINE = re.compile(r"^\s*(?:-\s*)?uses\s*:\s*(.*?)\s*$")
ACTION_REF = re.compile(r"^[^/\s@]+/[^@\s]+@[0-9a-f]{40}$")


def check_text(text: str, path: Path) -> list[str]:
    """Return diagnostics for non-GitHub actions that lack a full commit SHA."""
    errors = []
    for number, line in enumerate(text.splitlines(), 1):
        match = USES_LINE.match(line)
        if not match:
            continue
        value = match.group(1).split("#", 1)[0].strip().strip("'\"")
        if value.startswith("./") or value.startswith("actions/"):
            continue
        if not ACTION_REF.fullmatch(value):
            errors.append(f"{path}:{number}: third-party action must use a full 40-character commit SHA: {value}")
    return errors


def check_repository(root: Path) -> list[str]:
    """Check workflow and local composite-action YAML sources in the repository."""
    paths = [
        *root.glob(".github/workflows/*.yml"),
        *root.glob(".github/workflows/*.yaml"),
        *root.glob(".github/actions/**/action.yml"),
        *root.glob(".github/actions/**/action.yaml"),
    ]
    errors = []
    for path in sorted(paths):
        errors.extend(check_text(path.read_text(), path.relative_to(root)))
    return errors


def main() -> int:
    """Print violations and return nonzero so CI fails on a movable third-party ref."""
    root = Path(__file__).resolve().parents[2]
    errors = check_repository(root)
    for error in errors:
        print(error, file=sys.stderr)
    if not errors:
        print("GitHub Action refs are pinned")
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
