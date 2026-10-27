# Pinned SPL / OPcache PHPT replay

The oracle is php-src `php-8.5.10` at
`34308a6666b2d489c509541ea9befea9e2b42348`, with an installed PHP 8.5.10 CLI.
The oracle checkout must be clean. Prepare the compiler and bridge archives,
then install the project's locked native packages before measuring.

```sh
cargo build
./target/debug/elephc native install --locked --target macos-aarch64
python3 scripts/php_src/replay.py \
  --php-src ../php-src-8.5.10-oracle \
  --elephc target/debug/elephc \
  --output .artifacts/php-src-1806/replay --jobs 2 --timeout 30
```

The inventory contains all 794 SPL and 916 OPcache PHPTs. `--filter` selects a
focused replay and explicitly records the unselected remainder. No test names
are excluded automatically. The runner extracts unchanged FILE sections and
uses the pinned upstream EXPECTF function through PHP/PCRE. It copies sibling
fixtures and their directory hierarchy, honors INI/ENV/SKIPIF/CLEAN and records
reference failures independently. A source or fixture requiring a PHP subprocess
is an adapter blocker; native tests never silently invoke reference PHP through
`TEST_PHP_EXECUTABLE`. Unsupported sections remain explicit adapter errors.

The default comparison merges stdout and stderr in kernel write order, as
php-src's runner does. Duplicate native diagnostics are mismatches, not stripped
output. `--output-channel stdout` is an exploratory mode with stderr retained
but unscored; its matches are labeled `stdout_pass`, never full passes.

A PHPT whose INI sets a non-zero `opcache.opt_debug_level` compares php's Zend
optimizer opcode dumps, which an AOT compiler has no equivalent for. It is
declared out of scope with the status `out_of_scope_opcode_dump`: reported on
its own, never counted as a pass.

`manifest.json` pins the compiler binary, tracked source diff, reference PHP,
runner, matcher, test hashes and settings. `results.jsonl` is flushed after each
case. Per-case streams live under `cases/`; `summary.json` records every outcome.
Choose a new output directory for each run. Evidence directories under
`.artifacts/` are local and ignored by `.artifacts/.gitignore`.

This host AOT adapter is a campaign measurement, not a 100% certificate. It does
not validate dynamic eval or the supported target matrix. Reference-only skips,
XFAIL matches, CLEAN execution through PHP and oracle build/configuration gaps
remain visible. The command exits nonzero while these acceptance gates remain
open, even if a focused subset matches.

Run the adapter's checks with `python3 scripts/php_src/test_replay.py`.

## Tentative method return contracts

After exporting the shared symbol registry, regenerate the neutral tentative
method-return inventory with:

```sh
python3 scripts/php_src/gen_tentative_method_returns.py \
  --php-src ../php-src-8.5.10-oracle
```

The command verifies the pinned source SHA and PHP 8.5.10 runtime, joins original
method owners to the shared class catalog, and generates sorted canonical keys.
An absent key never grants ReturnTypeWillChange permission to relax a return.
