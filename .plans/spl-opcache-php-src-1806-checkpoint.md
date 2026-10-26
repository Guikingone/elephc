# SPL / OPcache PHP-src replay checkpoint

- [x] Freeze php-src, compiler and bridge provenance.
- [x] Replay all 1,710 PHPTs with merged stdout/stderr.
- [x] Retain every result and diagnostic family.
- [ ] Resolve all failures and complete eval/native/CI acceptance.

Oracle: `php-8.5.10`, `34308a6666b2d489c509541ea9befea9e2b42348`.
Compiler source: `551fcb97cb7bcf718474df7a8fe981a146df6e3e`; executable SHA-256: `deaf22778cf2f3fed4dfbbc3a334b7b3a4afeef1d1d1b561a64648c2042dd044`.

Merged stdout/stderr is compared with unchanged upstream EXPECT/EXPECTF.
Skips, oracle failures, timeouts, crashes and unsupported adapters remain open gates.
Host AOT evidence does not certify eval, the target matrix, signatures or all PHP-visible APIs.

| Outcome | SPL | OPcache | Total |
|---|---:|---:|---:|
| adapter_error | 1 | 0 | 1 |
| compile_error | 187 | 408 | 595 |
| compile_timeout | 6 | 4 | 10 |
| mismatch | 267 | 170 | 437 |
| oracle_error | 2 | 18 | 20 |
| oracle_skip | 2 | 20 | 22 |
| pass | 318 | 288 | 606 |
| runtime_crash | 9 | 7 | 16 |
| runtime_timeout | 2 | 1 | 3 |

Recorded: 1710 / 1710 tests. No filename exclusion.

## Compiler failure clusters

These are diagnostic families for triage, not independently proven root causes.

- 34: error: Unexpected token: Ampersand
- 18: error: Unexpected token: RBracket
- 15: error: Undefined variable: $a
- 12: error: Undefined variable: $x
- 11: error: Invalid assignment target
- 11: error: Undefined variable: $GLOBALS
- 10: error: Undefined constant: y
- 9: error: exit() argument must be integer
- 9: error: Cannot index non-array
- 6: error: Undefined variable: $pipes
- 6: EIR backend error: unsupported EIR backend feature: unset target shape with 1 lowered operands (supported: variables, array/hash elements, ArrayAccess offsets, __unset()-backed properties, declared fixed object properties, and dynamic object properties). Packed fields, by-reference property slots, and runtime-dependent magic property shapes are not supported. An element of a by-reference INDEXED array (`function f(array &$a) { unset($a[1]); }`) would leave a key hole, so the local must become a hash, and the caller's slot still says `array<T>`; the associative form (`unset($a["k"])`) is supported
- 6: error: Constructor 'Exception::__construct' parameter $message expects string, got int
- 6: error: Unexpected token: Plus
- 5: error: Unknown type: var
- 5: error: Bitwise operators require integer operands
- 5: error: Unexpected token at statement position: Plus
- 5: error: Reference elements in array literals (`[&$x]`) are not supported: an array element cannot alias a variable's storage. Assign the value instead, or use `$b =& $a[0]` to alias an existing array element
- 5: error: Undefined variable: $y
- 4: error: Variable variables (`$$name`) are not supported: variable names must be known at compile time
- 4: error: Duplicate function declaration: foo
- 4: error: Cannot override method C::getTimezone without declaring a compatible return type (parent returns DateTimeZone)
- 4: error: Expected ';'
- 4: EIR backend error: unsupported EIR backend feature: constructor call to SplFixedArray::__construct without an emitted EIR method body
- 3: error: Runtime-dynamic include/require path expressions are not supported: variable `$file` is resolved at runtime. Include paths must be compile-time-constant strings (string literals, concatenations of foldable strings, or `const`/`define()` string constants)
- 3: error: Array push requires an object or typed pointer

## Remaining acceptance gates

- Resolve the full failure inventory; do not exclude bug/gh, JIT or file-cache tests.
- Close duplicate/incorrect diagnostic streams and source locations.
- Audit signatures, tentative return contracts, constants and reflection against the pinned stubs.
- Complete eval, ownership/destructor/trace, native target and exact-head CI/review gates.
- Implement subprocess/environment adapters and re-run oracle failures with an in-tree PHP build.

Artifacts: `manifest.json`, `results.jsonl`, `summary.json`, `failure-inventory.json` and `cases/`.
