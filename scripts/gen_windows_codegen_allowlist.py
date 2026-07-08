#!/usr/bin/env python3
"""Manage the Windows (windows-x86_64) codegen no-regression parity gate.

This single tool owns every piece of the Windows codegen parity gate that is
expressed in data rather than YAML:

  * ``generate`` — (re)builds the two in-repo source-of-truth lists from a
    ``cargo nextest list`` JSON dump (the *runnable* set) and a set of failing
    test names (JUnit reports and/or plain-text lists):

        allow_list      = runnable_ci_tests - known_windows_failures
        known_failures  = the failing set (sorted)

  * ``gate`` — the CI post-step. Given the allow-list and the *actual* failing
    tests of a Windows-under-wine run (per-shard JUnit reports), it computes
    ``regressions = actual_failures ∩ allow_list`` and exits non-zero iff that
    intersection is non-empty. Tests that are NOT in the allow-list (the known
    failures and any brand-new / native-only tests) can never fail the gate.

Both subcommands share the same JUnit / plain-text failure parsing, so the
"what counts as a failing test name" definition lives in exactly one place.

Determinism: every emitted list is sorted with Python's default (Unicode
code-point) ordering, independent of the host locale, so regenerating from the
same inputs always produces byte-identical files.

Usage examples::

    # Regenerate both lists from a fresh nextest list + the failures dump.
    python3 scripts/gen_windows_codegen_allowlist.py generate \\
        --list-json nextest_list.json \\
        --failures win_failing.txt

    # Regenerate from the 16 per-shard JUnit artifacts of a parity CI run.
    python3 scripts/gen_windows_codegen_allowlist.py generate \\
        --list-json nextest_list.json \\
        --junit artifacts/*/junit.xml

    # CI gate step for one shard.
    python3 scripts/gen_windows_codegen_allowlist.py gate \\
        --allowlist tests/codegen/support/windows_codegen_allowlist.txt \\
        --junit target/nextest/ci/junit.xml \\
        --shard 3/16
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

# Default in-repo locations, resolved relative to the repository root (this
# script lives in `<repo>/scripts/`).
REPO_ROOT = Path(__file__).resolve().parent.parent
SUPPORT_DIR = REPO_ROOT / "tests" / "codegen" / "support"
DEFAULT_ALLOWLIST = SUPPORT_DIR / "windows_codegen_allowlist.txt"
DEFAULT_KNOWN_FAILURES = SUPPORT_DIR / "windows_codegen_known_failures.txt"

# The nextest test binary whose fixtures make up the parity suite.
CODEGEN_SUITE_ID = "elephc::codegen_tests"


def _local_tag(tag: str) -> str:
    """Return an XML element tag without its ``{namespace}`` prefix, if any."""
    return tag.rsplit("}", 1)[-1]


def parse_junit_failures(path: Path) -> tuple[set[str], int]:
    """Parse a nextest JUnit report into (failing_test_names, tests_run).

    A ``<testcase>`` counts as failing iff it has a direct child element named
    ``failure`` or ``error`` (nextest emits ``<failure>`` for panics, non-zero
    exits, and slow-timeout terminations). Passing tests carry no such child;
    skipped/ignored tests are not emitted at all. The returned name is the
    testcase ``name`` attribute, which nextest sets to the full test path
    (e.g. ``codegen::arrays::callbacks::test_array_all``) — the exact form used
    throughout these lists.
    """
    root = ET.parse(path).getroot()
    failures: set[str] = set()
    ran = 0
    for testcase in root.iter("testcase"):
        name = testcase.attrib.get("name")
        if name is None:
            continue
        ran += 1
        if any(_local_tag(child.tag) in ("failure", "error") for child in testcase):
            failures.add(name)
    return failures, ran


def read_name_list(path: Path) -> set[str]:
    """Read a plain-text list of test names, ignoring blank and ``#`` lines."""
    names: set[str] = set()
    for raw in Path(path).read_text().splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        names.add(line)
    return names


def collect_failures(junit_paths: list[str], failure_paths: list[str]) -> tuple[set[str], int | None]:
    """Union the failing test names from JUnit reports and plain-text lists.

    Returns (failures, tests_run). ``tests_run`` is the total number of executed
    testcases seen across the JUnit reports (used for the parity summary), or
    ``None`` when only plain-text failure lists were supplied (a bare name list
    carries no notion of how many tests ran).
    """
    failures: set[str] = set()
    ran_total = 0
    saw_junit = False
    for jp in junit_paths:
        saw_junit = True
        f, ran = parse_junit_failures(Path(jp))
        failures |= f
        ran_total += ran
    for fp in failure_paths:
        failures |= read_name_list(Path(fp))
    return failures, (ran_total if saw_junit else None)


def load_runnable(list_json_path: Path) -> set[str]:
    """Extract the CI-profile *runnable* codegen tests from a nextest list dump.

    Reads ``cargo nextest list --profile ci --test codegen_tests
    --message-format json`` output. A test is runnable when it is not
    ``#[ignore]``d and matches the profile's filter (``filter-match.status ==
    "matches"``). Returns the set of full test names.
    """
    data = json.loads(Path(list_json_path).read_text())
    suites = data.get("rust-suites", {})
    suite = suites.get(CODEGEN_SUITE_ID)
    if suite is None:
        raise SystemExit(
            f"error: nextest list JSON has no '{CODEGEN_SUITE_ID}' suite "
            f"(found: {sorted(suites)})"
        )
    runnable: set[str] = set()
    for name, tc in suite.get("testcases", {}).items():
        if tc.get("ignored"):
            continue
        fm = tc.get("filter-match")
        matched = fm.get("status") == "matches" if isinstance(fm, dict) else bool(fm)
        if matched:
            runnable.add(name)
    return runnable


def _write_list(path: Path, header: list[str], names: list[str]) -> None:
    """Write a sorted name list with a ``#`` comment header to ``path``."""
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = [f"# {h}" if h else "#" for h in header]
    lines.extend(names)
    path.write_text("\n".join(lines) + "\n")


ALLOWLIST_HEADER = [
    "Windows (windows-x86_64) codegen no-regression allow-list.",
    "",
    "One full nextest test name per line (sorted, comments start with '#').",
    "These codegen fixtures currently PASS when cross-compiled to",
    "windows-x86_64 and run under wine64. The CI gate fails iff any test in",
    "THIS list regresses (starts failing) on Windows; tests NOT listed here",
    "(the known failures and any brand-new / native-only fixtures) never fail",
    "the gate, so Windows parity can only improve, never regress.",
    "",
    "allow_list = (ci-profile runnable codegen tests) - (known failures)",
    "",
    "Regenerate with:",
    "  cargo nextest list --profile ci --test codegen_tests \\",
    "    --message-format json > nextest_list.json",
    "  python3 scripts/gen_windows_codegen_allowlist.py generate \\",
    "    --list-json nextest_list.json \\",
    "    --junit <per-shard junit.xml from a windows-codegen-parity run>",
    "See docs/compiling/targets.md ('Windows codegen parity gate').",
    "DO NOT hand-edit; regenerate so it stays in sync with the suite.",
]

KNOWN_FAILURES_HEADER = [
    "Windows (windows-x86_64) codegen KNOWN-FAILURES list (companion to the",
    "allow-list). One full nextest test name per line (sorted, '#' comments).",
    "These codegen fixtures currently FAIL under windows-x86_64 + wine64. They",
    "are the complement of the allow-list within the ci-profile runnable set:",
    "",
    "  known_failures = (ci-profile runnable codegen tests) - allow_list",
    "",
    "Kept in-repo as the source of truth for how the allow-list was derived and",
    "to track Windows parity progress. As failures get fixed, refresh both files",
    "together with scripts/gen_windows_codegen_allowlist.py (a fixed test moves",
    "from here into the allow-list). DO NOT hand-edit; regenerate.",
]


def cmd_generate(args: argparse.Namespace) -> int:
    """Regenerate the allow-list and known-failures files from source inputs."""
    runnable = load_runnable(Path(args.list_json))
    failures, _ = collect_failures(args.junit, args.failures)
    if not failures:
        raise SystemExit(
            "error: no failing test names were supplied "
            "(pass --junit and/or --failures)."
        )

    stale = sorted(failures - runnable)
    if stale:
        preview = "\n  ".join(stale[:20])
        more = "" if len(stale) <= 20 else f"\n  ... and {len(stale) - 20} more"
        msg = (
            f"error: {len(stale)} failing test name(s) are not in the runnable "
            f"ci set (stale/renamed inputs?):\n  {preview}{more}\n"
            "The failure inputs and the nextest list JSON must come from the "
            "same revision. Re-run `cargo nextest list` on the same commit."
        )
        if args.allow_stale_failures:
            print("WARNING: " + msg, file=sys.stderr)
            failures &= runnable
        else:
            raise SystemExit(msg + "\n(Use --allow-stale-failures to drop them.)")

    allow = sorted(runnable - failures)
    known = sorted(failures)

    _write_list(Path(args.out_allowlist), ALLOWLIST_HEADER, allow)
    _write_list(Path(args.out_known_failures), KNOWN_FAILURES_HEADER, known)

    print(f"runnable ci codegen tests : {len(runnable)}")
    print(f"known Windows failures     : {len(known)}")
    print(f"allow-list (known-good)    : {len(allow)}")
    print(f"wrote {args.out_allowlist}")
    print(f"wrote {args.out_known_failures}")
    assert not (set(allow) & set(known)), "allow-list and known-failures overlap"
    return 0


def _emit_summary(shard: str, ran: int | None, failed_count: int, regressions: list[str]) -> None:
    """Append the per-shard parity picture to $GITHUB_STEP_SUMMARY, if set."""
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not summary_path:
        return
    lines = [f"### Windows codegen parity — shard {shard}", ""]
    if ran is not None:
        passed = ran - failed_count
        parity = (passed / ran * 100.0) if ran else 100.0
        lines += [
            "| metric | value |",
            "| --- | --- |",
            f"| tests run | {ran} |",
            f"| passed | {passed} |",
            f"| failed | {failed_count} |",
            f"| parity | {parity:.1f}% |",
            f"| allow-list regressions | {len(regressions)} |",
            "",
        ]
    else:
        lines += [f"failed tests: {failed_count}, allow-list regressions: {len(regressions)}", ""]
    if regressions:
        lines.append("**Regressed (allow-listed) tests:**")
        lines.append("")
        lines += [f"- `{name}`" for name in regressions]
        lines.append("")
    with open(summary_path, "a") as fh:
        fh.write("\n".join(lines) + "\n")


def cmd_gate(args: argparse.Namespace) -> int:
    """Fail iff any allow-listed test failed in this Windows-under-wine run.

    Loads the allow-list and the actual failing tests (JUnit and/or plain-text),
    intersects them, emits the parity summary, and returns 1 when the
    intersection is non-empty (printing the regressed names) or 0 otherwise.
    """
    allow = read_name_list(Path(args.allowlist))
    if not allow:
        raise SystemExit(f"error: allow-list '{args.allowlist}' is empty — refusing to run a no-op gate.")
    failures, ran = collect_failures(args.junit, args.failures)

    regressions = sorted(failures & allow)
    _emit_summary(args.shard, ran, len(failures), regressions)

    if regressions:
        print(
            f"WINDOWS CODEGEN REGRESSION: {len(regressions)} allow-listed "
            f"test(s) failed on windows-x86_64 (shard {args.shard}):",
            file=sys.stderr,
        )
        for name in regressions:
            print(f"  {name}", file=sys.stderr)
        print(
            "\nThese tests are in the known-good allow-list "
            f"({args.allowlist}); a Windows failure here is a regression.\n"
            "If a test legitimately can no longer pass on Windows, refresh the "
            "allow-list with scripts/gen_windows_codegen_allowlist.py.",
            file=sys.stderr,
        )
        return 1

    print(f"OK: no allow-listed Windows codegen regressions (shard {args.shard}); {len(failures)} non-gated failure(s).")
    return 0


def build_parser() -> argparse.ArgumentParser:
    """Construct the argparse CLI with the ``generate`` and ``gate`` subcommands."""
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="command", required=True)

    g = sub.add_parser("generate", help="regenerate the allow-list and known-failures files")
    g.add_argument("--list-json", required=True, help="cargo nextest list --message-format json output")
    g.add_argument("--junit", action="append", default=[], help="nextest JUnit report(s) with the failing tests")
    g.add_argument("--failures", action="append", default=[], help="plain-text failing-test list(s), one name per line")
    g.add_argument("--out-allowlist", default=str(DEFAULT_ALLOWLIST))
    g.add_argument("--out-known-failures", default=str(DEFAULT_KNOWN_FAILURES))
    g.add_argument("--allow-stale-failures", action="store_true", help="drop (don't error on) failures missing from the runnable set")
    g.set_defaults(func=cmd_generate)

    t = sub.add_parser("gate", help="fail iff an allow-listed test regressed on Windows")
    t.add_argument("--allowlist", default=str(DEFAULT_ALLOWLIST))
    t.add_argument("--junit", action="append", default=[], help="this run's nextest JUnit report(s)")
    t.add_argument("--failures", action="append", default=[], help="plain-text failing-test list(s), one name per line")
    t.add_argument("--shard", default="?", help="shard label for logs/summary, e.g. 3/16")
    t.set_defaults(func=cmd_gate)
    return p


def main(argv: list[str]) -> int:
    """CLI entry point: parse arguments and dispatch to the chosen subcommand."""
    args = build_parser().parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
