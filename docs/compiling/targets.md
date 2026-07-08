---
title: "Targets and cross-compilation"
description: "The supported target matrix, how to select a target with --target, and the accepted target spellings."
sidebar:
  order: 4
---

elephc compiles to native machine code for a fixed set of first-class targets.
All supported targets are equal: a feature is not considered done until it works
on every one of them.

## Supported target matrix

| Target | Platform | Architecture |
|---|---|---|
| `macos-aarch64` | macOS | ARM64 (Apple Silicon) |
| `linux-aarch64` | Linux | ARM64 |
| `linux-x86_64` | Linux | x86-64 |

By default the compiler targets the **host** it runs on, detected automatically.

## Selecting a target

```bash
elephc --target linux-aarch64 hello.php
elephc --target linux-x86_64 hello.php
elephc --target=macos-aarch64 hello.php
```

Both the spaced (`--target VALUE`) and inline (`--target=VALUE`) forms work.

## Accepted spellings

Each target accepts several spellings, including the LLVM-style triple, so build
scripts written for other toolchains keep working:

| Canonical | Also accepted |
|---|---|
| `macos-aarch64` | `macos-arm64`, `aarch64-apple-darwin` |
| `linux-aarch64` | `linux-arm64`, `aarch64-unknown-linux-gnu` |
| `linux-x86_64` | `x86_64-unknown-linux-gnu` |

## Cross-compilation notes

Selecting a target different from the host produces assembly and an object file
for that target. Producing a final linked binary still depends on having a
linker and any target libraries available for that platform; the elephc test
suite uses the Docker scripts under `scripts/` to build and run the Linux
targets from a macOS host.

For the target-aware ABI and runtime details behind each platform, see
[Architecture](../internals/architecture.md) and
[The Code Generator](../internals/the-codegen.md).

## Windows codegen parity gate

`windows-x86_64` is an experimental cross-compilation target, not yet a
first-class supported target. CI cross-compiles every codegen fixture to
`windows-x86_64` and runs it under Wine to measure how much of the suite already
behaves correctly on Windows. To let that parity grow without silently
regressing, CI enforces a **curated no-regression gate**.

### How the gate works

Two lists live in the repository as the source of truth:

- `tests/codegen/support/windows_codegen_allowlist.txt` — the codegen tests that
  currently **pass** on `windows-x86_64` under Wine (the known-good set).
- `tests/codegen/support/windows_codegen_known_failures.txt` — the companion list
  of tests that currently **fail** on Windows.

Together they partition the `ci`-profile *runnable* codegen tests:

```
allow_list     = (ci-profile runnable codegen tests) - known_failures
known_failures = (ci-profile runnable codegen tests) - allow_list
```

The sharded `windows-codegen-parity` CI job runs the full suite under Wine and,
per shard, computes `regressions = actual_failures ∩ allow_list`. The gate rule
is exact:

> **The gate fails if and only if a test in the allow-list failed on Windows.**

Tests that are **not** in the allow-list — the known failures **and** any
brand-new or native-only fixtures — never fail the gate. This protects the
known-good set while letting Windows-incompatible tests exist freely: parity can
only improve, never regress. The aggregating `windows-codegen-gate` job is green
only when all 16 shards are green, and it is the single Windows-codegen
dependency of the top-level `test` gate. Each shard also prints its
passed / failed / parity% to the job summary, so the informational parity picture
stays visible alongside the gate.

### Refreshing the allow-list as parity grows

When Windows fixes land and previously-failing tests start passing, move them
from the known-failures list into the allow-list by regenerating both files from
a real parity run:

1. Download the 16 `windows-codegen-junit-<shard>` artifacts from a
   `windows-codegen-parity` CI run (each is that shard's `junit.xml`).
2. Produce the current runnable set:

   ```bash
   cargo nextest list --profile ci --test codegen_tests \
     --message-format json > nextest_list.json
   ```

3. Regenerate both lists deterministically:

   ```bash
   python3 scripts/gen_windows_codegen_allowlist.py generate \
     --list-json nextest_list.json \
     --junit path/to/windows-codegen-junit-*/junit.xml
   ```

The script writes both files sorted and locale-independent, so the same inputs
always reproduce byte-identical lists. It errors if a supplied failing test is
not in the runnable set (a sign the inputs came from a different revision). Never
hand-edit the lists — always regenerate. The same script's `gate` subcommand is
what the CI job runs to perform the intersection check.
