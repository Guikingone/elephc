# Issue #1805: nested by-reference foreach

- [x] Reproduce the corruption and premature termination against the starting compiler.
- [x] Preserve loop-carried aliases without a synthetic null store.
- [x] Retire raw owners only at successful bindings and declare destructor effects.
- [x] Preserve tombstone chains when the hash header and entry allocation are stable.
- [x] Add 11 heap-debug regressions, an example, and ownership documentation.
- [x] Validate focused tests on macOS and both Linux architectures, with optimization on/off.
- [x] Compare PHP output and emit/assemble all five supported targets.
- [ ] Verify the published revision in PR #1831 CI.

## Scope and provenance

Issue: https://github.com/illegalstudio/elephc/issues/1805.
Branch: `fix/byref-foreach-family`; starting HEAD:
`44c0716a878c36b016b8d088753d3828795a1ce0`.

A re-entered inner loop previously wrote a synthetic boxed null through its last
entry alias and released that payload without clearing the entry. This corrupted
array contents and could print freed-memory poison even with balanced final heap
counts. Reference value locals now receive no null seed. A successful binding
retires a raw local owner and relinquishes the previous managed-cell lease without
releasing an aliased payload. Empty walks preserve existing values and references.
`IterCurrentValueRef` counts as a store for frame initialization and parameter
ownership, and has conservative effects because retirement may invoke destructors.

Zend foreach_008 also exposed premature termination after both saved successor
anchors were deleted. The raw walker can follow multiple tombstones when physical
entry identities remain valid. Cursor validation therefore compares both the hash
header and its separate entry allocation before accepting a tombstone. Rehashing
can move entries while preserving the header. Relocated or reused live slots still
use key-identity resynchronization; snapshots advance after validation.

## Focused validation

Command: `cargo test -p elephc --test codegen_tests codegen::runtime_gc::by_ref_foreach`.
Repeat with `ELEPHC_IR_OPT=off`; the harness defaults to optimization on.

| Executed target | Optimization on | Optimization off |
| --- | --- | --- |
| macOS ARM64 | 89 passed | 89 passed |
| Linux ARM64, Docker | 89 passed | 89 passed |
| Linux x86_64, Docker/QEMU | 89 passed | 89 passed |

The Linux runs execute the same 89 names, including all 11 new regressions.
ARM64 uses Rust 1.95.0; x86_64 uses Rust 1.98.1 under QEMU on an ARM64 host.
The x86_64 prerequisite build took 70m09s; its on/off tests took 94.09s/83.92s.
Existing musl deprecation warnings in probe/Magician are recorded separately
from the warning-free macOS build. No full local suite was run.

Additional evidence:

- PHP 8.5.10: 50 native macOS replays match exact stdout and have clean heaps.
- Linux x86_64: all 11 regression fixtures and 16 capacity fixtures, each on/off,
  match PHP and have clean heaps: 54 cross-compiled ELF executions under QEMU.
- Zend foreach_005, foreach_008, and foreach_009 pass; extracted PHPT bodies omit
  only the terminal PHP closing tag for Elephc input.
- A nested-deletion probe emits and assembles on macOS ARM64, iOS ARM64,
  iOS Simulator ARM64, Linux ARM64, and Linux x86_64, each on/off: 10 checks.
  iOS evidence is compilation/assembly, not device execution.
- The final macOS build replays the issue reproducer and three Zend fixtures
  on/off: 8 exact PHP matches with clean heaps. The example also matches PHP.
- `git diff --check`, assembly-comment alignment, PHP example syntax, and touched
  Rust module preambles pass. Source and new-test hashes remain unchanged after
  the Linux runs; temporary build volumes and containers were cleaned up.

Linux test binary SHA-256 values:

- ARM64: `46fbac3bb35229ad2269f353e97298fcaeb3f691b9d1c608d9ee59fcaf8f3386`.
- x86_64: `faa06938343101646cc8f71f3ebe4202f3bd25aab8e05b61ec3a87c5bcaae8c4`.

Detailed logs, source manifests, and compiler artifacts are retained locally by
the author. The earlier interrupted x86_64 prerequisite builds are excluded
from these test results. The pre-existing issue #1803 plan remains untouched.
