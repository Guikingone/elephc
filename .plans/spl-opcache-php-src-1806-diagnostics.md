# Startup diagnostic destination checkpoint

- [x] Honor startup display/log destinations for ordinary warnings and user diagnostics.
- [x] Validate stdout/stderr routing, buffers, user handlers and last-wins overrides.
- [x] Replay all 794 SPL PHPTs and compare with the frozen baseline.
- [x] Revalidate all 17 new passes on the committed candidate.
- [ ] Close dynamic INI, default/log-format/fatal/eval compatibility gates.

The SPL corpus moved from **318 to 335 passes** with **17 gains and zero lost passes**.
All 794 tests remain in the inventory; the 916 OPcache cases were not replayed in this slice.

Candidate source: `4d9c5f034ab5f0831b974dc442f97c55559aeabc`.
Candidate binary SHA-256: `34478f77719a40d8d3542a737cc81a6b58f5a41e0772524bf522fa995d8f97dd`.
Focused candidate replay: 17/17 matches, with 1,693 cases explicitly unselected.

Artifacts: `.artifacts/php-src-1806/diagnostic-startup-spl` and `diagnostic-startup-final-gains`.

## SPL outcomes

| Outcome | Count |
|---|---:|
| adapter_error | 1 |
| compile_error | 187 |
| compile_timeout | 6 |
| mismatch | 250 |
| oracle_error | 2 |
| oracle_skip | 2 |
| pass | 335 |
| runtime_crash | 9 |
| runtime_timeout | 2 |

## New matches

- `ext/spl/tests/ArrayObject/ArrayObject_get_object_vars.phpt`
- `ext/spl/tests/ArrayObject/ArrayObject_modify_shared_object_properties.phpt`
- `ext/spl/tests/ArrayObject/arrayObject___construct_basic1.phpt`
- `ext/spl/tests/ArrayObject/arrayObject_exchangeArray_basic2.phpt`
- `ext/spl/tests/ArrayObject/arrayObject_getFlags_basic1.phpt`
- `ext/spl/tests/ArrayObject/array_010.phpt`
- `ext/spl/tests/ArrayObject/array_012.phpt`
- `ext/spl/tests/ArrayObject/array_028.phpt`
- `ext/spl/tests/ArrayObject/bug31346.phpt`
- `ext/spl/tests/ArrayObject/bug41691.phpt`
- `ext/spl/tests/ArrayObject/bug73209.phpt`
- `ext/spl/tests/ArrayObject/gh16646_2.phpt`
- `ext/spl/tests/SplFileObject/SplFileObject_fgetcsv_escape_default.phpt`
- `ext/spl/tests/SplFileObject/SplFileObject_fputcsv_002.phpt`
- `ext/spl/tests/SplFileObject/SplFileObject_setCsvControl_variation001.phpt`
- `ext/spl/tests/recursive_tree_iterator_001.phpt`
- `ext/spl/tests/recursive_tree_iterator_006.phpt`

## Verification and limits

Four native CLI tests, two selector/seed unit tests, the five-target warning emitter test,
and the previous 13 dedicated regressions pass. Both native Linux probes show one
displayed user/ordinary warning and no stderr copy at log_errors=0. Apple runtime
assembly checks passed; they are not iOS linking or execution proof.

This slice preserves existing default destinations. It does not implement dynamic
ini_get/ini_set reporting or mutation, full PHP logging format, fatal destinations,
or all eval paths. The larger SPL/OPcache campaign remains incomplete.
