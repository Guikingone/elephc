#!/usr/bin/env python3
"""Verify that the PDO_DBLIB bridge uses FreeTDS's portable open symbol.

The optional ``dbopen`` compatibility export is absent from FreeTDS builds that
do not enable Sybase ABI compatibility. Inspecting the built bridge catches a
regression without making CI depend on one distribution's FreeTDS build flags.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def undefined_symbols(archive: Path, nm: str) -> tuple[set[str], str]:
    """Return normalized undefined symbols and diagnostics reported by ``nm``."""
    result = subprocess.run(
        [nm, "-u", str(archive)],
        capture_output=True,
        text=True,
        check=False,
    )
    symbols: set[str] = set()
    for line in result.stdout.splitlines():
        fields = line.split()
        if not fields or fields[-1].endswith(":"):
            continue
        symbol = fields[-1].split("@", 1)[0]
        symbols.add(symbol.removeprefix("_"))
    return symbols, result.stderr.strip()


def main() -> int:
    """Check for ``tdsdbopen`` and reject the optional ``dbopen`` dependency."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path, help="built libelephc_pdo static archive")
    parser.add_argument("--nm", default="nm", help="nm-compatible executable")
    args = parser.parse_args()

    if not args.archive.is_file():
        print(f"error: archive not found: {args.archive}", file=sys.stderr)
        return 1

    symbols, diagnostics = undefined_symbols(args.archive, args.nm)
    if "dbopen" in symbols:
        print(
            "error: PDO_DBLIB references optional FreeTDS symbol `dbopen`; "
            "use `tdsdbopen(..., 0)` for portable Sybase semantics",
            file=sys.stderr,
        )
        return 1
    if "tdsdbopen" not in symbols:
        print(
            "error: PDO_DBLIB archive does not reference expected symbol `tdsdbopen`",
            file=sys.stderr,
        )
        if diagnostics:
            print(diagnostics, file=sys.stderr)
        return 1

    print("PDO_DBLIB portable-symbol audit passed: tdsdbopen present, dbopen absent.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
