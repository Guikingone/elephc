#!/usr/bin/env python3
"""Snapshot the local PHP binary's internal function list into php_baseline.json.

Run manually when pinning or bumping the PHP baseline version (requires a local
PHP binary). CI never runs this: the snapshot is committed, so the comparison
generator works without PHP installed.

Usage:
    python3 scripts/docs/extract_php_baseline.py [--php /path/to/php]
"""
from __future__ import annotations

import argparse
import datetime
import json
import subprocess
import sys
from pathlib import Path

OUTPUT = Path(__file__).resolve().parent / "php_baseline.json"

# Extensions loaded on a contributor's machine that are NOT bundled with
# php-src (PECL / third-party / vendor tooling). Dropped from the snapshot so
# the baseline represents PHP itself, whichever local PHP produced it.
# (imap moved from php-src to PECL in PHP 8.4.)
NON_BUNDLED_EXTENSIONS = frozenset({
    "herd", "igbinary", "imagick", "imap", "mongodb", "pdo_sqlsrv",
    "redis", "sqlsrv", "swephp", "zstd",
})

PHP_PROGRAM = r"""
$map = [];
foreach (get_defined_functions()["internal"] as $f) {
    $r = new ReflectionFunction($f);
    $ext = $r->getExtensionName();
    $map[strtolower($f)] = $ext === false ? "core" : strtolower($ext);
}
ksort($map);
$exts = array_map("strtolower", get_loaded_extensions());
sort($exts);
echo json_encode([
    "php_version" => PHP_VERSION,
    "extensions" => $exts,
    "functions" => $map,
], JSON_UNESCAPED_SLASHES);
"""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--php", default="php", help="PHP binary to snapshot (default: php)")
    args = parser.parse_args()

    try:
        proc = subprocess.run(
            [args.php, "-r", PHP_PROGRAM], capture_output=True, text=True, check=False
        )
    except FileNotFoundError:
        print(f"error: PHP binary '{args.php}' not found; install PHP or pass --php", file=sys.stderr)
        return 1
    if proc.returncode != 0:
        print(proc.stderr, file=sys.stderr)
        print(f"error: '{args.php} -r' exited with {proc.returncode}", file=sys.stderr)
        return 1

    raw = json.loads(proc.stdout)

    # Count what we're dropping
    original_funcs = len(raw["functions"])
    original_exts = len(raw["extensions"])

    # Filter out non-bundled extensions and their functions
    raw["extensions"] = [e for e in raw["extensions"] if e not in NON_BUNDLED_EXTENSIONS]
    raw["functions"] = {
        name: ext for name, ext in raw["functions"].items()
        if ext not in NON_BUNDLED_EXTENSIONS
    }

    n_dropped = original_funcs - len(raw["functions"])

    data = {
        "php_version": raw["php_version"],
        "generated_at": datetime.date.today().isoformat(),
        "extensions": raw["extensions"],
        "functions": raw["functions"],
    }
    OUTPUT.write_text(json.dumps(data, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"wrote {OUTPUT} (PHP {data['php_version']}, {len(data['functions'])} functions, dropped {n_dropped} functions from non-bundled extensions)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
