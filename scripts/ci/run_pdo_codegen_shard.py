#!/usr/bin/env python3
"""Run one deterministic PDO shard inside a single libtest process."""

from __future__ import annotations

import hashlib
import subprocess
import sys
from pathlib import Path


PDO_TEST_PREFIX = "codegen::pdo"
TEST_THREADS = "4"


def test_names(test_binary: Path) -> list[str]:
    """Return every PDO test exposed by the archived codegen test binary."""
    result = subprocess.run(
        [str(test_binary), "--list"],
        check=True,
        capture_output=True,
        text=True,
    )
    return sorted(
        line.removesuffix(": test")
        for line in result.stdout.splitlines()
        if line.startswith(PDO_TEST_PREFIX) and line.endswith(": test")
    )


def shard_for(test_name: str, shard_count: int) -> int:
    """Map a test name to a stable one-based shard number."""
    digest = hashlib.sha256(test_name.encode()).digest()
    return int.from_bytes(digest[:8], "big") % shard_count + 1


def main() -> int:
    """Select this shard's exact tests and execute them in one cached process."""
    if len(sys.argv) != 4:
        print(
            "usage: run_pdo_codegen_shard.py <test-binary> <shard> <shard-count>",
            file=sys.stderr,
        )
        return 2

    test_binary = Path(sys.argv[1]).resolve()
    shard = int(sys.argv[2])
    shard_count = int(sys.argv[3])
    if not test_binary.is_file() or not 1 <= shard <= shard_count:
        print("invalid test binary or shard selection", file=sys.stderr)
        return 2

    selected = [
        name for name in test_names(test_binary) if shard_for(name, shard_count) == shard
    ]
    if not selected:
        print(f"PDO shard {shard}/{shard_count} selected no tests", file=sys.stderr)
        return 2

    print(f"Running {len(selected)} PDO tests in shard {shard}/{shard_count}", flush=True)
    command = [
        str(test_binary),
        "--exact",
        *selected,
        "--test-threads",
        TEST_THREADS,
    ]
    return subprocess.run(command, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
