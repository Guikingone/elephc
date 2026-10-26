# SPL and OPcache PHP-src compliance — PR 1806

- [x] Preserve both original worktrees and identify the exact PR head.
- [x] Create an isolated continuation branch from PR 1806.
- [x] Integrate current upstream main and verify overlapping fixes.
- [x] Pin php-src and inventory the entire SPL and OPcache PHPT corpora.
- [x] Add a reproducible, fail-closed PHPT replay and retain every result.
- [ ] Resolve existing review findings with differential regressions.
- [x] Replay all 1,710 PHPTs without filename exclusions; cluster failures.
- [ ] Close all SPL and OPcache behavior/signature/INI/ownership gaps.
- [ ] Validate dynamic eval as well as AOT, and every supported target.
- [ ] Regenerate affected builtin docs and complete exact-head CI/review gates.

## Provenance

Original SPL worktree: `../elephc-spl-compliance`, branch
`audit/spl-php-src-compliance`, head `5125ea8abf72f83fda67ed5cc18556824b0fe781`.
Its five modified tracked files and untracked `class_relation.rs` remain untouched.
Original OPcache worktree: `../elephc-opcache-coverage`, branch
`audit/opcache-php-src`, head `5843201cde` (clean at discovery).
Continuation: `../elephc-spl-opcache-1806`, branch `feat/spl-opcache-php-src-1806`.
Upstream main was refreshed before integration. No publication is part of the
initial checkpoint; original PR is ready for review and must not be silently pushed.

Normative oracle: php-src tag `php-8.5.10`, commit
`34308a6666b2d489c509541ea9befea9e2b42348`. Host PHP CLI is 8.5.10.
The clone lives in `../php-src-8.5.10-oracle`.
There are **794 SPL** and **916 OPcache** PHPTs, including nested directories,
`bug*`, `gh*`, JIT, and file-cache tests. The historical 265-pass subtotal
excludes regression filenames and is not the acceptance denominator.

## Replay and acceptance

Use unchanged upstream FILE/FILE_EXTERNAL sections, sibling fixtures, INI,
ENV, SKIPIF, CLEAN, and EXPECT/EXPECTF. Use the pinned upstream EXPECTF matcher.
Every inventory entry remains in the report, including oracle skips, unsupported
sections, reference failures, compiler errors, timeouts, crashes and mismatches.
An upstream XFAIL is recorded rather than silently turned into a passing result.
Keep raw compiler diagnostics and native stdout/stderr alongside each case.
Reference-only skips are environmental evidence, not implemented API coverage.
Pin the compiler tree, executable hash and source corpus manifest for each replay.

Full compliance requires zero unexplained exclusions, blocked adapters, compiler
errors, crashes or observable mismatches, followed by AOT/eval and target gates.
OPcache's AOT implementation must prove PHP-visible API and INI semantics;
passing language regressions alone does not prove PHP JIT/cache infrastructure.

## Initial root-cause priorities

1. Resolve integration conflicts: main's subclass-aware DLL mode seeding,
   PR's Traversable acceptance, and widening untyped mixed-receiver writes.
2. Canonical numeric SplFixedArray keys: PHP integer-key parsing, overflow and
   sign preservation on both architectures (Zend/zend_hash.c and ext/spl).
3. Constructor call-site locations for ObjectNewReserved.
4. Runtime arity errors must evaluate supplied arguments in source order.
5. Missing returns for nullable/mixed declared types must still throw; preserve
   the throw in both optimizer effect and exception summaries.
6. Buffer-aware diagnostics, x86 exception snapshots, nested/object-backed
   storage mutation/sorting, CSV controls, constructor overrides, tentative
   return typing, recursive hook guards and capped file reads.

Inherited class-relation WIP is preserved for later inspection; it is not copied
into the baseline or counted as a proven fix.

## Resumption checkpoint (2026-10-14)

The initial full replay is retained under `.artifacts/php-src-1806/baseline`.
It reported 616 stdout matches, 594 compile errors, 350 mismatches, 100 oracle
errors, 22 oracle skips, 15 crashes, 10 compile timeouts and 3 runtime timeouts.
These are **not a compliance percentage**: the first adapter lacked upstream
startup INI defaults and placed its sandboxes outside the native project.
The corrected replay uses the pinned runner's defaults, project-contained
sandboxes, the installed locked PCRE2 package, a frozen compiler executable,
and separate stdout/stderr artifacts. Its manifest pins the runner as well.

Implemented review families: canonical fixed-array textual keys and their
owned-box lifetime, refused-call argument effects, nullable/mixed fallthrough
throws in EIR and both optimizer summaries, output-buffer-aware diagnostics,
x86 uncaught frame snapshots, and ObjectNewReserved source context. Synthetic
trigger_error now reads an effectful LoadDiagnosticLine EIR operation rather
than a constant zero, covering default output and user error-handler arguments.
All 11 new differential regressions passed on macOS ARM64. A fresh allocated
computed-key regression covers read/write/unset, not just persistent literals.
The 335-test SPL replay passed 334 tests; its remaining mode-word failure
exposed specialized stack/queue setters bypassing the shared masked/FIX setter.
Both architectures now tail-dispatch those setters through the shared routine;
focused revalidation is in progress.

Runtime assembly was generated from the current Rust library. macOS ARM64,
iOS device and Simulator runtime assembly succeeded. The iOS assembler used
Apple clang with a macOS sysroot warning: this is assembly evidence only,
not an iOS linking or execution gate. The Linux x86_64 numeric-key probe was
assembled, linked and executed in Docker and printed int(7).

Remaining review findings include CSV control forwarding, live object-backed
sorting and requested sort flags, subclass constructors, typed-array property
unset, tentative-return attribute boundaries, union covariance, recursive hook
reentry and capped line-read tails. The inherited class-relation WIP remains
in the original worktree. No 100% claim, push or PR publication has occurred.

OPcache continuation is now integrated from PR 1793, exact head
`5843201cde34219f8343b4891d0a5ba1e2077409`, without conflicts. Two confirmed
review findings are being closed: startup refusal logs must retain stderr until
error_log registration, and the AOT reset logger must fall back to stderr on
the native file writer's negative failure result. PHP 8.5.10 independently
confirmed startup stderr routing with verbosity 2, memory_consumption 7 and a
configured file error_log (the log file remained absent).

The runner's authoritative default is now merged descriptor output. Earlier
stdout-only inventories remain exploratory artifacts. A subprocess guard also
prevents fixture-launched reference PHP from becoming a silent native pass.
Five adapter checks pass, including exact cross-descriptor write ordering.
The corrected stdout-only full inventory reported 633 matches, 579 compiler
errors, 364 mismatches, 84 oracle errors, 22 skips, 16 crashes, 8 compiler
and 3 runtime timeouts, plus one adapter error. It preceded OPcache integration
and the final specialized mode setter fix; its manifest is authoritative.

The 34 class tests passed after the mode setter repair. Linux ARM64's numeric-key
probe also assembled, linked and printed int(7). Builtin exporter/regeneration,
module sections, site and boundary audits succeeded. The comparison generator
requires Python 3.11; on this host it ran through the installed tomli compatibility
module under Python 3.9 without changing the generator source.

An additional heap-debug regression exposed 120 live blocks over 20 iterations
when a later refused-call argument threw and the receiver was freshly created.
The recorded-arity path now uses the standard call-operand owner roots and
argument-evaluation scope, rather than releasing each argument immediately.
A destructor-order regression also pins that a first argument stays alive while
later argument side effects run. Revalidation is pending. Error-trace ownership
for refused internal calls remains a separate gate: PHP retains argument objects
through the caught exception's trace; the static throw currently has no refused
callee frame with those arguments.

The owner-root correction passed all 13 dedicated regressions, including the
20-iteration fresh-receiver/argument-throw probe with a clean heap. The earlier
120-block result is retained above as invalid implementation evidence.
The 36 OPcache INI tests and all 8 OPcache PHP-src parity tests passed, including
both new logging regressions. The merged error-test fixture needed a diagnostic
spelling refresh (`Traversable, got int` rather than Rust Debug type spelling).

Final stream scoring must use the merged-output mode, not the earlier stdout-only
subtotals. A full merged-output replay is the next measurement on the joined
SPL/OPcache implementation. No failing result is waived by selecting a smaller
corpus; remaining API, language, ownership, oracle and subprocess-adapter work
stays on the campaign checklist.

## Final measured checkpoint

The authoritative host AOT replay is `.artifacts/php-src-1806/merged-final`,
source head `551fcb97cb7bcf718474df7a8fe981a146df6e3e`, with the frozen compiler
and bridge hashes in its manifest. All **1,710** tests were recorded without
filename exclusions: **606 pass**, **595 compile errors**, **437 mismatches**,
**16 crashes**, **10 compile timeouts**, **3 runtime timeouts**, **20 oracle
errors**, **22 oracle skips**, and **1 adapter error**. No full-compliance claim
is justified. The tracked summary is `spl-opcache-php-src-1806-checkpoint.md`;
the complete case ledger is `merged-final/failure-inventory.json`.

Native owner-root probes passed on both Linux x86_64 and Linux ARM64, each with
220 allocations, 220 frees and zero live blocks. The SPL storage example compiled
and executed successfully. Assembly comment alignment, builtin docs/site/boundary
audits, the focused constructor diagnostic test and git diff hygiene passed.
All code changes are committed locally. Neither original PR branch nor either
original worktree was modified or pushed.

## Next diagnostic destination slice

The frozen merged replay exposes interleaved duplicate display/log fragments when
php-src requests `log_errors=0`. This slice seeds startup `display_errors` and
`log_errors` in main/request entry and keeps the shared runtime object neutral.
Warnings and user diagnostics consult those destinations independently of
handler invocation. Display-to-stderr bypasses output buffers; stdout display
keeps the shared buffer/web capture path. Numeric display modes follow PHP's
uint8 projection. Existing default destinations remain unchanged in this slice.

Open gates: dynamic ini_get/ini_set reporting and mutation, normative default
configuration, full PHP log formatting, fatal destinations and eval adapters.
Object-backed ArrayObject sorting was reproduced separately (array_keys called
on the raw object); it remains the subsequent storage root-cause family.

Diagnostic startup checkpoint: all 794 SPL PHPTs were replayed in merged-output
mode using `.artifacts/php-src-1806/diagnostic-startup-spl`. SPL matches moved
from **318 to 335**, with **17 new passes and zero lost passes**. Remaining
outcomes: 187 compile errors, 250 mismatches, 9 crashes, 6 compile timeouts,
2 runtime timeouts, 2 oracle failures, 2 oracle skips and 1 adapter blocker.
The 916 OPcache PHPTs were not replayed in this slice; the old 1,710-case result
remains a historical checkpoint and must not be silently updated by aggregation.

Validation: four native CLI destination tests, two seed/parser unit tests, the
five-target warning emitter gate, and the previous 13 dedicated regressions pass.
Native Linux ARM64/x86_64 probes display both user and ordinary warnings once
and leave stderr empty at log_errors=0. Apple runtime assembly checks succeeded;
these remain assembly evidence, not iOS linking/execution proof. Two inherited
ClassInfo unit fixtures were refreshed to initialize their new deprecation flag.

The measurement binary was frozen before the later web-SAPI projection and
unit-fixture-only refinements. Its binary hash is authoritative. Current code
preserves identical CLI mode routing; focused revalidation should pin the final
committed source before any publication. No full-compliance claim is justified.

Final source verification: all 17 gained PHPTs passed again on committed source
`4d9c5f034a` (full SHA and executable hash in the focused manifest). The current
CLI destination tests also passed after the web projection and fixture refresh.
The tracked slice report is `spl-opcache-php-src-1806-diagnostics.md`.

## Active goal: rebase and TDD to full compliance

The user now explicitly requires continuation until 100%, rebasing against
origin/main and using TDD for every implementation family. This does not narrow
the original SPL/OPcache, PHPT, eval, signature/ownership or target scope.
`git fetch origin main` resolved to `27c8164f43985a6a17c43daeab30a20df6a63cdb`.
A backup branch was created, `git rebase --rebase-merges origin/main` completed,
and HEAD/tree stayed equal to the backup because that main was already integrated.

Return-contract TDD evidence is under `.artifacts/php-src-1806/tdd-return-contracts`.
The first red suite had five failing tests: invalid source contracts were accepted,
invalid union arms were accepted, valid self-to-concrete-parent covariance was
rejected, and an inherited tentative-return marker was lost. All five became
passing after owner-aware validation and every-arm union checking; all fifteen
native compliance regressions passed with ELEPHC_PHP_CHECK=1.

A separate red test then proved mandatory internal SPL returns were also being
waived by the marker (SplObjectStorage::seek and SplFixedArray getIterator/jsonSerialize).
The shared neutral contract now exposes original-owner tentative-return metadata,
reproducibly generated from the pinned PHP 8.5.10 Reflection oracle and the
shared class inventory. Unknown keys remain strict. Hard/soft contract revalidation
is in progress; no full-goal completion claim is justified.

Return-contract slice is green: six TDD error-test groups, two neutral catalogue
tests and 37 native SPL-interface tests pass, alongside the earlier 15 dedicated
regressions. Catalogue generation is stable across repeated runs. Builtin docs,
site and EIR-boundary audits passed after regeneration.
The 17 affected upstream PHPTs retain exactly their previous outcomes (one pass,
five output mismatches and eleven compile errors); no case was dropped or waived.
These remaining failures belong to additional language/runtime families.
The active goal remains the full original 100% objective.

## Nested-storage TDD slice (in progress)

The branch was refreshed against origin/main, still
`27c8164f43985a6a17c43daeab30a20df6a63cdb`, already an ancestor of HEAD.
The dedicated worktree and both inherited worktrees remain preserved.

Two new tests were red before production changes: nested writes/unsets modified
wrapper properties instead of inner entries, and projection of a nested
self-backed ArrayObject looped until SIGALRM. Both behavioral tests are now green.
The storage helpers stop at a self-backed inner identity and forward keyed
mutations through the wrappers' internal helpers, preserving input-array COW.

A new 20-iteration heap test then exposed 160 live blocks / 28,912 bytes.
Isolated controls show construction and writes clean; copies and array-rebuilding
removals leak even with a single wrapper. The evidence is under
`.artifacts/php-src-1806/tdd-nested-storage` (plus the initial red/green logs one
level above). A manually patched assembly experiment, explicitly not acceptance
proof, fixes the Mixed-to-hash reference ledger and frees 380/380 allocations.
The production correction and rooted reconstruction operands are being retested.

The first host replay selected all 108 tests in `ext/spl/tests/ArrayObject/`:
33 pass, 47 mismatch, 26 compile errors, one crash, one oracle error; all statuses
are unchanged versus the prior SPL startup replay. This is the behavioral fix
before the additional ownership changes, not a final-source compliance result.
All five targets compiled the behavior probe and assembled its program/runtime.
Linux x86_64 and ARM64 executed the expected projection/write/remove outputs,
but both still reported 171 live blocks including the self-backed probe; their
pre-ownership-fix evidence remains retained rather than counted as clean.

Still open: final ownership regression, refreshed PHPT/target evidence, indexed
append delegation, object-backed sort, broader SPL/OPcache language/runtime gaps,
full corpus/eval/target acceptance. The 100% goal remains active.

Ownership iteration v2 passes 17 behavioral tests but still fails the heap gate:
80 live blocks / 4,160 bytes remain. Single and nested getArrayCopy controls now
free all allocations (380/380 and 720/720), while reconstruction-based removals
retain 80 blocks. Explicit synthetic operand locals are optimized away and do
not resolve the reconstruction's ownership defects; that unsuccessful attempt
is retained in the v2 evidence rather than treated as acceptance.

The current main already has an authoritative `MixedArrayUnset` EIR path.
A declared-mixed standalone removal/COW control passes and frees 380/380 blocks.
The SPL helper now reuses that path rather than rebuilding arrays. A separate
red differential control confirms numeric-string offsets were not normalized
by the old strict-key comparison; its tracked regression also checks key holes.
The complete focused suite is being rerun on this implementation.
The next append control is also red: nested array append bypasses inner public
overrides in PHP and produces three entries, but Elephc exits on SIGSEGV.


Iteration v3 passes all 19 dedicated tests, including the nested heap gate and
numeric-string removal. The existing storage test group was also replayed.
A further indexed getArrayCopy control then fails on SIGBUS before any removal:
the generic Mixed-to-associative conversion assumes a hash payload. Its new
heap/COW regression is now tracked. The conversion now clones the Mixed cell,
reuses the shared indexed/hash promotion helper, retains its returned hash and
releases the private cell. The original source remains borrowed and the owning
method-call cell is still released by EIR return coercion. This handles both
architectures through their actual runtime register contracts. v4 validation
is pending; v3 target emission in `final-matrix` is not final acceptance evidence.


Iteration v4 passes all 20 dedicated regressions and all 23 existing storage
tests. Current generated runtime matches the earlier runtime byte-for-byte.
The expanded target probe exposes an x86_64 ABI defect in MixedArrayUnset:
SysV supplies the receiver in rdi, but native mixed_deref reads rax. Adding that
register transfer to a retained assembly experiment changes Linux x86_64 from
exit 255 (non-array refusal) to the expected behavior. Production now contains
that transfer, and all 20 focused regressions pass again in v5.
All previously missing inline branch comments in the touched runtime emitter
were documented; both changed assembly-emitting files pass comment alignment.

The expanded combined probe still retains six blocks (608 bytes ARM64, 544
bytes x86_64). Isolated self-backed construction, declared/dynamic properties,
named/temporary projection and removal/isset probes are clean. A self-backed
keyed write retains one 48-byte block even when followed by removal; reusing
loop variable names in the combined program additionally affects its lifetime
ledger. These are open ownership gates, not clean target acceptance.
The original nested and indexed array-backed heap tests are clean. v5 target
probes now separate these exact workloads while preserving the combined failure.

No full compliance claim, push or external PR update has been made. The active
100% goal continues through the remaining self-write/combined ownership controls,
nested append crash, object-backed sorting and the complete SPL/OPcache corpus.

### Native matrix v5 checkpoint (2026-10-14)

- Current production runtime: matrix-v5 regenerated from the v5 compiler/rlib with the x86 mixed-array-unset ABI correction. Both array-only and combined probes assembled on all five supported targets. macOS, Linux ARM64 and Linux x86_64 executed both probes successfully. iOS evidence is assembly only, not native execution or full library acceptance.
- Array-backed workload: 1,784 allocations and 1,784 frees on all three executable targets, zero live blocks/bytes. Includes nested writes, removals, indexed copies and COW.
- Combined self-backed workload preserves expected PHP output but retains six blocks (608 bytes ARM64, 544 bytes x86_64). Isolated dynamic self-backed write retains one 48-byte block. These remain open TDD work; no full compliance claim.
- Provenance and raw logs: `.artifacts/php-src-1806/tdd-nested-storage/matrix-v5/manifest.json`, `results.json`, native stdout/stderr and assembler logs.

### Dynamic-property ownership TDD in progress (2026-10-14)

- `11461a470d` freezes nested SPL storage and array-copy fixes. The 108 upstream ArrayObject PHPTs replayed on that clean commit: 33 pass, 47 mismatch, 26 compile_error, one runtime_crash (`bug54323`), one oracle_error (upstream XFAIL `gh10519`). No status delta versus diagnostic-startup-spl. Evidence: `tdd-nested-storage/phpt-v5`, including manifest and delta.json.
- Self-backed dynamic writes red: 20 retained strings / 960 bytes over 20 iterations. Fresh receiver/computed-name write red: 80 retained blocks / 8,800 bytes; retiring name and receiver reduced it to 20 boxed scalar cells / 800 bytes.
- New refusal regression exposed 40 retained blocks / 3,696 bytes. EIR showed two RHS arrays evaluated, with the first release after the throwing store. An independent source-order oracle emits `onv`; native baseline emits `vonv`.
- Current WIP makes dynamic assignment own one RHS evaluation, roots receiver/name while later operands execute, retains its assignment-expression result independently, and retires adopted scalar boxes through the existing property-store helper. This slice is not committed or declared green yet.
- Pending follow-up: nested append still crashes in the saved red reproducer; object-backed dimension append differs from public append in php-src (dimension append allowed, method append refused). Typed-property weak-coercion assignment results must retain PHP behavior. Full corpus and target gates remain open.

### Dynamic-property ownership and x86 exception checkpoint (2026-10-14)

- 24 compliance regressions pass with PHP checking after the final codegen fix (`dynamic-write-green-v4.log`). The dynamic assignment RHS now executes once, in receiver/name/value source order; fresh receiver, computed name, successful scalar store and refused array store are heap-clean.
- Broader property-access check: 152 pass, seven failures. All seven reproduce on clean production sources from `11461a470d`; baseline compiler SHA, original fixtures and output comparison are in `baseline-property-check/`. They concern inherited/private slot reads and compound updates, and remain open pre-existing blockers.
- x86 refused-write red was a Throwable creation-line bug: fallback diagnostic loading replaced the fresh exception pointer in rax with line 4, then wrote to address 0x24. The production emitter now uses a distinct ABI scratch register. Native assembly experiment first confirmed the cause; matrix-v7 was then regenerated from the corrected compiler.
- matrix-v7: all five targets assemble the runtime and four probes. macOS/Linux ARM64/Linux x86_64 execute successfully. Each isolated probe is heap-clean: self-backed writes 740/740 allocations/frees, fresh scalar writes 240/240, refused writes 140/140. iOS evidence remains assembly only.
- Combined probe has five retained blocks: 528 bytes ARM64 / 504 bytes x86_64. Isolation shows a two-iteration loop reusing `$inner`/`$outer` before a self-backed subtype reproduces the five blocks; renaming the prior loop variables is clean. `heap-retype-loop.ir` vs `heap-retype-rename.ir` shows Mixed storage for the reused local. This is the next TDD ownership slice, not a waived gate.

### Static receiver ownership TDD in progress (2026-10-14)

- `96a9cd2fcf` freezes the dynamic-assignment and x86 Throwable-pointer slice.
- New tracked loop-widened receiver regression is red: 89 allocations / 84 frees, five blocks / 528 bytes. StoreLocal correctly consumes its owned object source; the retained reference instead appears when a named property write unboxes its receiver from the final Mixed frame slot. Unlike the raw-local case, this load owns an extra payload reference. The property write emitted no receiver release.
- Current WIP roots fresh named-write receivers during later operand/store evaluation and retires receiver ownership after ordinary/magic-dispatch paths merge. Backend ownership filtering preserves borrowed raw locals while retiring unboxed Mixed-local loads. Validation is pending.

### Static receiver checkpoint (2026-10-14)

- Both new reds are closed: loop-widened self-backed receiver (89/89 allocations/frees), and fresh named receiver (20/20). All 26 compliance regressions pass with PHP checking.
- matrix-v8 freshly emitted from the current compiler: all five supported targets assemble runtime plus combined/loop/fresh probes. macOS and both Linux targets execute all three, with matching outputs and zero retained blocks. The expanded combined workload is now 1,895/1,895 allocations/frees on all three executable targets.
- Real compiler iOS static-library attempts were made for device and Simulator, and refused because the selected Command Line Tools lack iphoneos/iphonesimulator SDKs. Raw diagnostics and nonzero statuses are preserved. Assembly evidence must not be advertised as iOS archive/link/native acceptance; that SDK gate remains open.
- Artifact provenance: `tdd-nested-storage/matrix-v8/manifest.json`, `results.json`, per-target stdout/stderr, assembler and archive logs.

- Repeated focused property-access suite after named receiver cleanup: 152 pass, the same seven pre-existing inherited/private-slot failures reproduce (no new failing test names). Raw log: `property-access-static-receiver.log`. The full suite was not run.

### Rebase provenance checkpoint (2026-10-14)

- origin/main advanced from `27c8164f4398` to `62cfeb85efb1` (eval array/curl callback receiver ownership). Backup: `backup/spl-opcache-before-rebase-3601d13a`. Rebase with merge topology completed at `0fe70df4c9d4`, after restoring unchanged historical conflict resolutions from their original merge/commit trees.
- Exact tree comparison against the backup detected five lost merge resolutions outside the 27 changed upstream paths: internal arity operand roots, startup accelerator log destination, native reset log fallback, their OPcache regressions, and the Traversable error spelling assertion. All five files are restored verbatim from the pre-rebase backup; upstream changed files are preserved. This comparison is mandatory because `--rebase-merges` does not preserve every manual merge hunk automatically.
- Compiler/Magician rebuild and focused revalidation follow before the next frozen full PHPT replay. Remote PR 1806 remains open/ready/conflicting at its original `5125ea8abf72` head; no remote mutation or push occurred. Local continuation is isolated and contains current main.

- Independent `git merge-tree --write-tree` of the pre-rebase backup plus current origin/main produced expected source tree `7c5b5eb6c669ad71a344ee3224f1e7966c025017`. The restored working tree differs only by this new six-line provenance note; no code/test resolution or upstream change was lost. Evidence: `rebase-expected-merge.txt` and `rebase-tree-delta.txt`.

- Post-rebase validation: compiler plus Magician rebuilt without warnings; all 26 compliance regressions, three imported eval callback receiver tests, and eight OPcache parity tests pass. Assembly comment checks and git diff hygiene pass. Raw logs: `rebase-build.log`, `rebase-compliance.log`, `rebase-eval-callbacks.log`, `rebase-opcache.log`. The next full PHPT snapshot will freeze this rebuilt source and its native bridge archives.

### Fresh full corpus checkpoint a7364c782a (2026-10-14)

- Frozen clean source worktree: `../elephc-1806-replay-frozen`, detached at `a7364c782ac81c7016e8c5301b8e3c5da3c2a442`. Compiler and eight native bridge archives copied independently; pinned build manifest preserved. Managed PCRE2 10.47 is installed for macOS ARM64.
- All 1,710 upstream PHPTs were selected and completed using the pinned PHP 8.5.10 oracle, merged output and unchanged upstream payloads. Results: 635 pass, 595 compile_error, 414 mismatch, 12 runtime_crash, eight compile_timeout, three runtime_timeout, 20 oracle_error, 22 oracle_skip and one adapter_error. `complete=true` means inventory completion; `full_compliance=false` remains authoritative.
- SPL (794): 337 pass, 187 compile_error, 254 mismatch, five crashes, four compile timeouts, two runtime timeouts, two oracle errors, two skips, one adapter error. OPcache (916): 298 pass, 408 compile_error, 160 mismatch, seven crashes, four compile timeouts, one runtime timeout, 18 oracle errors, 20 skips.
- Exact comparison with the earlier merged-final replay: 29 newly passing PHPTs, zero lost passes, 33 total status changes. These gains cover the accumulated committed slices and the updated main, not only the latest ownership fix.
- Evidence: frozen worktree `.artifacts/php-src-1806/full-a7364c782a/{manifest.json,results.jsonl,summary.json,delta-from-merged-final.json}`; baseline build provenance in `.artifacts/php-src-1806/frozen-build.json`.
- Next append TDD slice is deliberately outside this snapshot: three tracked regressions are red for nested storage forwarding, outer override dispatch and terminal object-store method refusal. Object-backed dimension append semantics remain a separate open capability.

### Append TDD checkpoint in progress (2026-10-14)

- Initial three regressions were red (nested appends silently lost, wrong outer override order, missing object-store Error). The first prototype's new synthetic helper produced a null virtual entry on derived wrappers; it was removed. Null-offset appends now reuse the established internal keyed-write helper, forwarding through nested wrappers without inner public overrides.
- EIR object dimension append now supplies a null key to offsetSet. Existing native SplFixedArray/DLL base-family append diagnostics/helpers remain isolated. Public ArrayObject/ArrayIterator append resolves the terminal store, refuses object properties with the outer runtime class name, then dispatches the outer offsetSet override.
- Five append regressions pass with PHP checking, including a user ArrayAccess class that the frozen baseline rejected as an unknown append method. Entire focused SPL codegen surface: 355/355 pass. A sixth ownership control passes: explicit unset after 20 self-backed append refusals is clean (260/260).
- Native matrix-append-v1: all five targets assemble; macOS and both Linux targets execute all five probes with expected behavior. Nested/outer/interface are heap-clean, and the append loop is 1,640/1,640 allocations/frees. No iOS archive/native claim: SDK gate remains open.
- Refusal probe at implicit main exit retains five blocks. Controls isolate this as a pre-existing shutdown-cycle issue: self-backed construction alone at main exit retains five blocks/520 bytes, and a plain Node self-cycle retains two blocks/80 bytes. Explicit unset collects both; the append method's exception frame is balanced. Shutdown collection/order is a separate required TDD gate and is not waived.
- Object-backed dimension append is also still open: PHP permits it while public append refuses it. The current array-only helper must not be advertised as covering numeric object properties.

- Append slice follow-up verification: 41 ArrayAccess-filtered codegen regressions pass, covering named/union/interface/nullable receivers beyond the storage fixtures. Sixth method-refusal ownership control is green; it intentionally unsets the last error and receiver before collecting cycles. No shutdown-cycle fix was made or claimed.

### Append PHPT checkpoint d7ba30e3ed (2026-10-14)

- Fresh replay of all 108 upstream ArrayObject PHPTs: 34 pass, 46 mismatch, 26 compile_error, one runtime_crash and one oracle_error. Exact delta versus frozen full a7364c782a: `array_002.phpt` changes mismatch to pass, no lost pass. Evidence: `tdd-nested-storage/phpt-append/{manifest.json,results.jsonl,summary.json,delta-from-full.json}`.
- The last complete 1,710-case inventory remains 635 passes at a7364c782a. Do not present a fabricated 636/1,710 current result by adding this subset delta.

### Shutdown cycles TDD in progress (2026-10-14)

- Two initial tracked reds prove missing main-exit collection: a native self-cycle prints only body rather than PHP's body/cycle destructor output, and a self-backed ArrayObject retains five blocks. A third frozen-baseline red includes gc_disable(), global/static aliases and explicit return; PHP still invokes the cyclic destructor.
- Current WIP forces explicit cycle collection after process local/static/global/class-property owners retire, before instrument/frame teardown and heap reporting. The web-handler path similarly collects after request-local cleanup while persistent roots remain registered. This is target-neutral emission through the existing explicit collector helper.
- All three new cases and 21 existing COW/cycle cases pass (24/24), including gc_disable() and global/static roots. git diff hygiene and full frame emitter assembly-comment checks pass. Destructor/resurrection and native target validation are still pending; no closure claim yet.

- Shutdown validation evidence: 24 COW/cycle cases green; destructor filter 64 pass and one pre-existing refcount failure (reentrant by-ref property replacement). The failure reproduces identically on frozen a7364c782a and the current CLI before any body output; it is not introduced by the final collector. Baseline/current logs preserved as `reentrant-property-*`.
- Native matrix-shutdown-v2: all five supported targets assemble; all four process probes run on macOS/Linux ARM64/Linux x86_64 with expected destructor output and clean heap. Refusal is now 44/44 allocations/frees. iOS uses host-consumable library emission, which does not have standalone process EOF semantics; missing SDK/archive gates remain open. The earlier matrix-v1 assertion incorrectly required the process collector in iOS library assembly and is preserved as invalid validation, superseded by v2.
- `web_request_teardown_collects_cycles_with_gc_disabled` passes two requests on a one-worker native server.
- The final-collection omission is present in origin/main's emitter. Per the standing team learning, its independent fix is being prepared on `fix/main-shutdown-cycle-collection`, worktree `../elephc-shutdown-cycle-collection`, cut from main 62cfeb85efb1. Its compiler/test target is temporarily shared and must be rebuilt for the SPL branch before further branch-specific replay. No push/publication is authorized.
- User corrected tool naming: Trail-code is installed and used through trail ask/grep/skeleton/callers/check. Personal preference saved via trail note; no Codex memory file was edited.

### Main shutdown-cycle defect isolated and integrated (2026-10-14)

The generic EOF and gc_disable/global/static-cycle regressions reproduced missing destructors on clean origin/main `62cfeb85ef`; the SPL self-backed constructor case requires this feature branch and was excluded from the standalone main fix. Dedicated branch `fix/main-shutdown-cycle-collection` commit `22d9ee086f` adds the shared explicit cycle pass after root-owner retirement and before frame exit, with two main regressions and a two-request web regression. Main validation: 23 COW/cycle tests and the web teardown regression passed. Cherry-picked here as `27019d52ef`. Trail check reported a lexical false positive against the intentional cyclic test fixture; the commit explains it and a new immutable learning clarifies it. The self-backed SPL regression remains here. No push or publication performed.

The integrated shutdown checkpoint passed 24 COW/cycle regressions with `ELEPHC_PHP_CHECK=1`. Reusing the primary Cargo target directory for the isolated main test left stale main builtin-contract artifacts; the first integration build failed with missing `method_return_is_tentative`. A package-scoped clean of `elephc-builtin-contract`, followed by rebuilding `elephc` and `elephc-magician`, restored branch-local artifacts. Only these two crates differ from main; do not silently reuse build outputs across source worktrees.

### Descendant object-backed storage preparation / php-src bug54323 (2026-10-14)

The pinned `ArrayObject/bug54323.phpt` crashed after offsetUnset when `$c->prop` read an untyped default string slot that still had its concrete string representation. A direct `unset($c->prop)` control already warned and returned NULL. Constructor and exchangeArray inference reserved boxed mutable backing storage only for exact builtin receiver names; descendants missed both paths. The shared predicate now includes the existing checker ancestry relation for ArrayObject and ArrayIterator.

Three TDD regressions reproduced SIGSEGV before the change and passed after it with PHP cross-check. They cover descendant construction for both classes and descendant exchangeArray. A subsequent heap-debug version restores the temporary error handler and passes all three with a clean heap. The initial probe retained the registered handler (three blocks / 184 bytes); explicit restore releases those owners, so this is preserved as an incomplete heap measurement rather than reported as clean. Native validation artifacts keep the first invalid Docker image name and the next incomplete runtime-feature configuration; the current driver enables `handler_state` because restore_error_handler requires the generated pop helper.

The focused whole SPL codegen checkpoint passed 359/359 with PHP cross-check. A fresh unchanged ArrayObject PHPT replay completed 108 cases: 35 pass, 46 mismatch, 26 compile_error, 1 oracle_error (upstream XFAIL gh10519). Compared with the preceding append replay, only bug54323 changed: runtime_crash -> pass, no lost passes. This does not replace the frozen full 1710-case result at a7364c782a. Latest origin/main fetch still points to 62cfeb85ef, which is already an ancestor of this branch.

Artifacts: `.artifacts/php-src-1806/tdd-nested-storage/bug54323/`, `phpt-storage-subclasses/`, and `matrix-storage-subclasses-v3/`. Internal deprecation severity currently uses E_USER_DEPRECATED in the synthetic storage body, while php-src reports E_DEPRECATED; this newly identified semantic gap remains to fix. The warning-filtered subclass regressions do not establish severity parity.

The final descendant matrix v3 passed all nine native executions (three cases on macOS ARM64, Linux ARM64 and Linux x86_64), with matching PHP-visible outputs and balanced heap counts 94/94, 63/63 and 69/69 respectively. All three programs and the runtime assembled for all five supported targets. The real iOS SDK/archive gate remains open as previously recorded. Trail check against origin/main was truncated at 400 KB and reported 20 lexical matches against regression narratives; an untruncated check of the new increment against HEAD reported 9 such matches. Actual ancestry checks, regression fixtures and feature/main branch ownership follow the learnings; these false positives are explained in the commit and in a new immutable learning. Neither check is reported as automatically clean.

### Internal diagnostic severity and common argument-owner defect (2026-10-14)

The SPL synthetic storage body now lowers a compiler-internal engine deprecation with E_DEPRECATED (8192), rather than relaxing public trigger_error validation. Native severity, reporting-mask and public-validation regressions pass. Two eval tests pass after explicit shared-contract eval support is declared; the three contract-support gates pass. The display regression captures the test harness's default stderr channel; the first stdout assertion and unsupported ini_set control were invalid and are retained in the raw logs.

Throwing handlers exposed an existing main defect: backend-owned diagnostic arguments were released only after normal return. Clean main reds retained 24 blocks across three throws and eight blocks when the callback installed its replacement. Dedicated branch fix/main-error-handler-invocation-owners at 31d7a18b7c repairs this through a new owned runtime entry, preserving borrowed eval callers. Six registration-owner tests, a warning-free build and six native executions (two cases on macOS/Linux ARM64/Linux x86_64) pass with clean heaps; all five targets assemble. Cherry-picked as 87f69b552b with the SPL WIP stashed and restored; only the shared dispatch source changed during integration.

The SPL constructor throw regression remains red: the argument-container repair reduces 11 live blocks / 1120 bytes to three blocks / 432 bytes (backing object, fresh receiver and backing storage). This separate constructor-unwind ownership gate remains open. No current full-compliance result, push or publication is claimed. Evidence: primary tdd-nested-storage/bug54323/deprecation-{red,green-v2,green-v3,eval-red,eval-green-v2,contract-green}.log; standalone .artifacts/error-handler-invocation-owners/{red.log,green.log,build.log,matrix-v1/}. Trail check's two lexical false positives are explained in the dedicated commit and immutable learning.

### Engine diagnostic and incomplete receiver gates closed (2026-10-14)

Static generic object construction missed the unconstructed-object guard already used by dynamic construction. The feature prototype reuses that guard in the refactored generic emitter, covering ObjectNew and ObjectNewReserved without changing handle order. Four SPL diagnostic regressions are green, including throwing-handler ownership (24/24 allocations/frees). Nine native severity/throw/public-validation executions pass on macOS/Linux ARM64/Linux x86_64 with clean heaps (65/65, 24/24, 1/1). The default-mask matrix v2 also matches the PHP oracle byte-for-byte on all three executable targets, with 9/9 allocations/frees. Runtime and programs assemble on all five targets. Matrix-mask v1's stdout-only expectation was invalid: the real CLI displays the default diagnostic, while the integration harness captures its log on stderr. Its failed assertion remains preserved.

Dedicated main branch fix/main-constructor-unwind-owners at 06e65838ee guards the old main emitter, with three constructor ownership tests, a warning-free build and nine native successes (12/12, 5/5, 2/2). Clean-main repeated-constructor red retained three 24-byte receivers; the revised property-owner fixture also reproduced two live objects on frozen a7364c782a. The original fresh-argument fixture exposed a separate Throwable trace-argument lifetime gap: PHP keeps Child in getTrace()[0]['args'][0] until unset(error), while native destruction happens before catch. The original fixture, oracle control and failing output are retained; this trace gate remains open. Receiver tests isolate property allocation inside the constructor. A verification accidentally run before applying the patch is invalid evidence, retained as invalid-green-before-patch.log.

Cherry-picked main constructor fix as fbdbacf379, adapting the conflict to the feature's validated refactored emitter. WIP source and generated docs were stashed/restored with SHA256 comparison: zero content differences. Both dedicated main fixes remain unpublished. Trail check's sole constructor fixture conflict is a lexical false positive explained in the commit and immutable learning.

Builtin documentation workflow completed: exporter plus CLI built with curl, registries/pages regenerated, module and comparison pages regenerated, builtin audit / site compatibility / target-enforced EIR boundary audit all passed. The machine's Python 3.9 lacks tomllib; the comparison generator ran through its already-installed tomli module, without editing repository scripts. The 66 existing internal-page diffs only shift sidebar order after the new internal helper; the compatibility page and module function blocks have no behavioral diff. Broad SPL revalidation and a fresh unchanged ArrayObject PHPT replay follow; the frozen full 1710-case count remains 635 passes until replaced by a full replay.

The integrated whole focused SPL checkpoint passed 363/363 with ELEPHC_PHP_CHECK=1 and curl enabled (374 seconds). Runtime-driven eval cases rebuilt their current branch archives during the pass; no source changes were made during execution. Assembly-comment and git diff hygiene checks pass. Raw log: bug54323/deprecation-spl-suite.log. This is a focused codegen gate, not the php-src corpus denominator.

Final diagnostic-family PHPT checkpoint: all 108 upstream ArrayObject tests completed unchanged; 35 pass, 46 mismatch, 26 compile_error and one oracle_error. Exact comparison against phpt-storage-subclasses has zero changed statuses, zero new passes and zero lost passes. These fixes are demonstrated by the new severity/reporting/ownership regressions and native matrices; they do not add a fabricated corpus pass. Artifact: phpt-internal-deprecation/{manifest.json,results.jsonl,summary.json,delta-from-storage-subclasses.json}. Seven compiler/eval builtin parity tests also pass. The last complete full inventory remains 635/1710 at frozen a7364c782a.

Next family oracle probes are in .artifacts/php-src-1806/tdd-object-property-append/: object-backed language [] and offsetSet(null) insert numeric property buckets. Such buckets are invisible to named property/offset lookup, and unset(wrapper[0]) does not remove them. Explicit offsetSet(0, value) creates a distinct string-key property; getArrayCopy exposes both integer 0 and string "0", whereas casting the object to array collapses them. Preserve these key-kind distinctions rather than converting [] to a named property write. No implementation has been changed for this next family yet.

### Object-backed numeric property append TDD in progress (2026-10-14)

Engine-deprecation family committed at 059869c044. Latest origin/main still 62cfeb85ef, already an ancestor. Only task WIP now: tests/codegen/spl/object_property_append.rs plus its module entry in tests/codegen/spl.rs. Three new regressions are red on this exact source: basic [] drops both writes; []/offsetSet(null) with numeric unset visibility drops all writes; integer/string key distinction loses the original numeric bucket in getArrayCopy. All red executions are heap-clean. No production implementation for this family has changed. Raw log and PHP fixtures/oracles: .artifacts/php-src-1806/tdd-object-property-append/.

Pinned php-src spl_array_write_dimension_ex calls zend_hash_next_index_insert on the backing object's actual property hash for null offsets. Explicit offset writes use get_hash_key; numeric object property buckets are therefore invisible to named property access and unset. The PHP control shows `(array)object`, get_object_vars and get_mangled_object_vars normalize numeric string property names, while ArrayObject::getArrayCopy duplicates the raw table and preserves both integer 0 and string "0". Existing storage_array_of_body uses get_object_vars, so append and raw-copy semantics both need correction. PHP-src checkout has no Trail wiring graph; only known ext/spl/spl_array.c was read as normative fallback.

Mapped implementation points: storage_offset_append_body at storage.rs:857 forwards nested wrappers correctly, but terminal path array_push on object drops the write. storage_array_of_body at :528 normalizes via get_object_vars. Runtime stdclass_set uses __rt_property_hash_set and __rt_hash_set, which preserve actual key kinds. __rt_hash_append uses persistent integer index and owned input values. object_vars.rs __rt_object_to_hash projects declared fields and copies the dynamic tail through __rt_hash_project_spread, which normalizes keys. _class_object_dynamic_prop_flags plus _class_object_payload_sizes locate the dynamic property-hash slot. hash_spread.rs currently has spread mode and normalized projection mode; a raw preserving mode or equivalent helper will be needed. GetObjectVars RuntimeFnId lowers through group_02.rs to builtins/types.rs emit_object_hash_projection(value, cast_mode, scope_id). Keep public views normalized while adding the SPL raw view.

Before implementation, Trail callers was run for the append body (no indexed edges despite a known call from storage_offset_write_body), object projection emitter (full closure) and GetObjectVars lowerer. No heavyweight process is live from this family after the red run. Need map any new RuntimeFnId/contract support inventory and all target siblings before editing. Do not replace raw integer buckets with named "0" properties. The fresh-argument Throwable trace retention gap remains separately open on main; the earlier constructor repair does not fix it.

Object-append production prototype now adds a raw copy mode to hash_spread.rs (__rt_hash_raw_project_spread) and a raw object projection entry to object_vars.rs (__rt_object_raw_properties), preserving both integer/string key kinds for dynamic and incomplete-object tails. Existing public/cast paths keep their normalization. This is not yet bound to SPL or a builtin contract; the three new end-to-end regressions remain red. The narrow object_vars emitter tests compile and pass on the prototype, with assembly-comment and git diff hygiene green. The bare --lib command matched the intended compiler unit tests but also built/filtered the other workspace libraries; use -p elephc for subsequent focused unit checks. No builtin/raw-copy or numeric append binding has been implemented yet.

### Raw SPL object projection binding checkpoint (2026-10-14)

Added internal __elephc_spl_object_vars_raw contract/AOT home backed by RuntimeFnId::ElephcSplObjectVarsRaw. Its checker and fallback agree on an associative hash with Mixed keys/Mixed values; result ownership is Fresh and effects mirror GetObjectVars. The runtime entry uses raw mode 2, preserving integer/string key kinds while retaining visibility-mangled declared properties. The normal get_object_vars/cast paths keep normalization. Export safety proof inventory and support counts include the new helper; eval support remains explicitly internal/unsupported, consistent with the other compiled-only property helpers.

A fourth TDD regression on this exact branch showed getArrayCopy dropping protected/private properties while public get_object_vars and cast remained correct. After routing ordinary object stores through the raw helper, the regression passes with a clean heap: 3:1:2:3 / 1:3. Self-backed ArrayObject/ArrayIterator stores retain their prior accessible-property projection and filtering of four synthetic state keys. Raw copy red/green logs are in tdd-object-property-append/raw-copy-{red,green}.log. The two existing object_vars emitter unit tests pass. All current assembly comments and diff hygiene pass. Numeric object property append remains unimplemented and the original three append regressions remain red; no full or family completion claim is made.

Current WIP spans catalog_data/support, callables module/new __elephc_spl_object_vars_raw home, runtime_fn enum/descriptor, exports safety, runtime function group_02, builtins/types projection lowerer, storage.rs projection selection, raw hash/object projection helpers and four object_property_append regressions. No process remains live after raw-copy green. Next action: implement typed object property hash append, ensuring borrowed/owned Mixed argument handling and dynamic hash-slot metadata on every supported target, then run all four controls, wrapper/self-backed regressions and native matrices. Regenerate builtin docs again for this new helper before commit. Goal remains active until all original SPL/OPcache, corpus and target gates are verified.

### Public iteration view regression found and semantic repair in progress (2026-10-15)

Replay from tdd-object-property-append/phpt completed 108 cases: 34 pass, 47 mismatch, 26 compile_error, one oracle_error. Exact delta from the preceding deprecation replay has one lost pass: array_012.phpt (ArrayIterator::count over public/protected/private properties). No restart is required; the process is terminal and summary/results are present. The raw copy change incorrectly fed the complete property table to iteration/count.

PHP oracle controls establish three distinct views: getArrayCopy includes all visibility-mangled declared properties and both integer/string key kinds; offset lookup includes named properties but excludes integer buckets; iteration/count include public properties and preserve both key kinds. Added __elephc_spl_object_vars_iter / RuntimeFnId::ElephcSplObjectVarsIter, runtime object projection mode 4 (global public visibility plus raw dynamic keys), and StorageProjectionResult::Iteration. __elephcStorageArrayOf stays a full raw copy for exchangeArray/getArrayCopy; __elephcStorageArray now projects the public iteration view. Metadata/support counts and export proof inventory include the fourth helper (AOT registry 742, eval internal 115, unsupported 468). Builtin docs must be regenerated again for this fourth helper.

New focused regression iteration_uses_public_raw_view was red for count/iteration and exposed 32 retained heap blocks (array literal of two wrappers, nested foreach, inline count(getArrayCopy), then local reuse). After mode 4, stdout matches PHP exactly, but the same 32 live blocks / 5136 bytes remain. Latest log iteration-green.log: nine append-family tests pass, the new iteration case fails only on heap-clean assertion. Do not call the family complete or delete that assertion. Need isolate temporary foreach-container ownership versus inline method-call result ownership and late-widened local cleanup, then repair the responsible root cause; main defects require their own branch. The lost upstream array_012 status needs a fresh narrow replay after the repair.

Earlier valid evidence before mode 4: 372/372 focused SPL, 41/41 ArrayAccess, 9/9 new family controls, 27 native executions with clean heaps on macOS/Linux ARM64/Linux x86_64, and all programs/runtime assembled on all five targets. matrix-v1 lacked object_clone; matrix-v2 enabled eval_bridge unnecessarily and required mbstring hooks; matrix-v3 correctly enables handler_state and object_clone. Keep both invalid configurations. Mode 4 is newer than that matrix and requires revalidation.

Generated docs for the first three helpers passed all audits; their source/registry counts are stale after the fourth helper. Example spl-storage/main.php demonstrates legacy object-backed numeric append and compiled/executed successfully. Trail's query in the php-src checkout created an untracked graft/ cache; it was moved intact to tdd-object-property-append/oracle-graft-cache, restoring the pinned oracle's clean state at 34308a6666b2. No tracked oracle source was changed. No task-owned process remains live after iteration-green terminal status. No publication or goal completion claimed.

### Iteration recovery and common ownership isolation (2026-10-15)

Fresh unchanged array_012.phpt now passes after the public/raw iteration split (phpt-iteration-narrow/summary.json); this is one selected test, not a replacement aggregate. CLI controls in iteration-controls show identical PHP stdout: single-wrapper and homogeneous wrapper arrays are heap-clean; mixed ArrayObject/ArrayIterator getArrayCopy calls retain 19 blocks. A generic A::values associative array / B::values indexed array control reproduces four live blocks / 1240 bytes on both current source and frozen a736. A separate local-reuse/indexed-write control retains 13 blocks and is clean when fresh local names are used; append-only is clean. That second ownership boundary remains open.

Dedicated fix/main-mixed-method-result-owners is cut from freshly fetched origin/main a1a8428a894bf4cb66650ab682834c328758eae0 in the existing main worktree. Three new GC regressions are red on that main: arrays/hash returns, destructor timing, and shared property returns. The first shared-property fixture was rejected by count type inference; its corrected explicit array property reproduces the ownership failure. store_call_result used borrowed boxing for an already-owned concrete return. The prototype uses the existing owned-result boxing helper; green validation is running. The first compile used the helper's private module path and failed; the public re-export corrects that. No commit/integration yet.

origin/main advanced four commits from 62cfeb85ef (workflow scheduling, rustls bump, repository stats). Plain rebase attempted to replay 100 historical feature commits and conflicted at an old resolved SPL error test. Aborted without changing source; all 46 WIP files were stashed and restored with zero SHA256 differences. Preserve stash 4d8440dc93ba5e35c90fddcfc6cf77a271fe20b3 and artifact hashes until a clean rebasing strategy retains the historical merge resolutions. The feature branch remains at 059869c044; it has not silently absorbed current main. No publication.

The common return-box fix is committed on dedicated main branch fix/main-mixed-method-result-owners at c91a580c449d123cc4308f2aaa4f0ed2a5a40e24, integrated here as e887d37762. Three GC controls, 14 related tests, warning-free build, nine PHP-matching native executions with clean heaps and all-five-target assembly passed. Integration again restored all 46 WIP files with zero SHA256 content differences. Trail check's two lexical matches were explained in the commit and a second immutable learning.

The remaining reused-local failure is now reproduced independently on fresh main a1a8428a89 in fix/main-container-write-local-owners. Indexed append leaves two blocks / 112 bytes because backend writeback overwrites the old Mixed box without retiring it; indexed ArrayAccess receiver writes leave six / 800 bytes. The prototype uses release_mutated_source_local_owner before language ArrayPush, and roots the concrete object receiver plus key/value temporaries in PHP evaluation order. Receiver-only green left a four-block / 1192-byte RHS tree and a one-block / 80-byte key when RHS throws; both new regressions are green with complete operand roots. Eight GC controls pass (fresh RHS destructor timing, callee/key/RHS exceptions, exact key ordering, native SPL setters). Twenty-four native runs and all-five-target assembly passed on matrix-v2. The array-source cleanup was subsequently narrowed from the helper shared by variadic array_push to the language ArrayPush entry: repeated variadic publications require separate lease retaking. Final validation of this narrower placement is running; do not treat matrix-v2 as that exact source.

Related main checks passed 39 array_access and 40 array_push tests. PHP-check logging on the eval array_push/unshift dispatcher still reports existing by-reference warning differences; these test-harness greens are not a full equivalence claim. Two exploratory variadic controls were rejected by the frontend (mixed foreach receiver, and disallowed array-to-object retype); retain those compile logs and do not report them as executed checks. Main container-write fix is not yet committed or integrated. Primary ten-case append/public-iteration target matrix-v4 is prepared but not executed.

Main container ownership correction committed as f1c271be0010bc3cd044a03391577505aca44e59 and integrated as 4431199ed2. The sole cherry-pick conflict appended two runtime GC modules at the same file tail; both module declarations are retained. The 46 SPL WIP files were stashed/restored with zero SHA256 differences. Final source passes eight GC controls, 40 array_push tests, warning-free build and matrix-v3's 24 native executions with clean heaps on all executable targets; all five targets assemble. The unchanged object-setter portion retains its earlier 39/39 ArrayAccess checkpoint. Trail final check reports three lexical narrative/preamble matches; commit and immutable learning explain them, without claiming an automatic green.

Primary append/public-iteration revalidation and the two integrated main regression families are now running on HEAD 4431199ed2 plus the preserved four-helper WIP. No publication or current full-corpus claim.

Integrated append family is now green: 10/10 object-property controls (including the original 32-block iteration regression), 3/3 mixed-method return controls, and 8/8 container-write controls. All assertions retain exact PHP stdout and heap-clean checks. docs-build-v4 compiled the exporter/CLI; all six docs generation/audit commands passed with the fourth internal helper. Contract support passes 3/3 and builtin parity 7/7 on current source. Rust preamble, assembly-comment and diff checks pass.

Current matrix-v4 passes all 30 native executions (ten fixtures on macOS ARM64, Linux ARM64, Linux x86_64), including public-view 211 allocations / 211 frees. Every program and runtime also assembles for all five supported targets. Manifest includes compiler/rlib, test source hashes and changed Rust file hashes, including untracked production helpers. Generated docs were reviewed; other existing internal-page diffs are ordering shifts after the new internal entries. Full focused SPL and the unchanged 108-case ArrayObject replay follow. Frozen full aggregate remains 635/1710 at a736; no current full-compliance claim.

Final ArrayObject replay phpt-v4 completed all 108 cases: 35 pass, 46 mismatch, 26 compile_error, 1 oracle_error. Direct status comparison against tdd-nested-storage/phpt-internal-deprecation has zero changed statuses, zero gained passes and zero lost passes. Against the invalid raw-iteration prototype, only array_012 recovers mismatch -> pass. This family adds no fabricated upstream corpus gain.

Whole focused SPL passes 373/373; a repeat with libtest --show-output associates 18 PHP-check output differences with exact test names in spl-php-check-notes.json. Four disappear only under the analysis-only /private/var/folders path canonicalization; fourteen retain semantic/diagnostic differences (autoload state, iterator behavior, missing deprecations/warnings, Serializable metadata, regex flags and disabled-function scope). Raw outputs are preserved and no assertion/oracle is weakened. Thus a Rust test pass is not a universal PHP equivalence pass. Both suite runs remain distinct evidence. The grouped run rebuilt dependency artifacts but did not change tracked manifests or source; the working HEAD remains 4431199ed2. Example recompiles and executes with the expected real object-backing deprecation and numeric append line.

Next proven PHPT root: ArrayObject_proptable_canonicalization.php currently matches every output except `(array)$ao`, which exposes three compiler storage/flags/iterator-class properties. PHP exposes the normalized backing data instead. Need oracle probes for ArrayObject/ArrayIterator flags, subclasses, nested/self stores and numeric-string collisions, then TDD the cast view. No cast implementation changed yet. Also audit ignored offsetSet return-value ownership as a separate hypothesis before describing the generic setter family as exhaustive.

Before the numeric projection checkpoint, Trail check --base HEAD reports 20 lexical matches in plan history, neutral catalog prose and generated helper titles. Source and tests keep distinct raw/named/public projections, descendant layout preparation and dedicated-main provenance. Both invalid matrices and the historical rebase abort remain recorded; the old graft cache name is an artifact description, not a command choice. New immutable learning and commit explain these matches without changing fixtures or prior notes. The check is not claimed as automatically clean.

### History-preserving main rebase (2026-10-15)

Numeric property checkpoint committed as 7441615e9399f4e4a3ba0d4e3bd75a393834ca97, with clean tracked worktree. Fresh origin/main is dbea9357500166b6d1b3b137136528909a0eb263 and adds concat-reset optimization plus earlier rustls/workflow/stats changes. A clean isolated worktree rebase/spl-opcache-1806-main-20261015 uses --rebase-merges=no-rebase-cousins. It preserves the original feature cousin history and stops only while recreating merge 33678cea93 with three old resolution conflicts. Each conflict is outside the 28-file new upstream delta; restoring those three files from that original merge preserves its resolutions, then all remaining commits replay cleanly.

Rebased HEAD is 978a0810f109d2a67a96b63786a6837c4263b5d7. Its tree d551240bd7a9a4b41db4762c134bb17f611692a9 exactly equals the clean git merge-tree result of pre-rebase 7441615e93 and new main dbea935750: zero unexpected paths or lost feature changes. Artifacts rebase-merge-reconciliation.json and rebase-verification.json retain the proof. Primary was updated only after verifying no tracked edits; backup/spl-opcache-1806-before-main-20261015 preserves the old tip. README cache and all own stashes remain. The isolated rebase worktree is retained, with no Cargo target or publication.

Because new main changes the EIR optimizer, the previous source/compiler evidence remains a pre-rebase checkpoint. Fresh focused regressions, SPL and concat-reset integration tests, then a fresh native/PHPT replay are required on 978a0810f1. No current post-rebase pass count yet; old binaries are not treated as current builds.

Pinned next-cast oracle artifacts: next-object-cast/oracle.php and oracle.stdout. For both ArrayObject and ArrayIterator, flags 0/2 cast their normalized backing table (including visibility-mangled backing fields); flags 1/3 cast the wrapper's own complete user property table. get_object_vars always exposes only the wrapper's own public properties and get_mangled_object_vars its own complete properties, independent of flags; compiler state must be hidden. getArrayCopy remains the complete raw backing table and iteration remains public backing data for all four flags. Casting an outer wrapper follows an inner wrapper's cast flags (outer -> inner STD_PROP_LIST casts to the inner own table, empty for a plain wrapper), while self-backed subclass casts expose its own public/protected/private fields without compiler state. Numeric dual int0/string0 buckets collapse to the later named value in the cast but remain distinct in getArrayCopy. These probes are PHP-only; no next-family production code or tests have been added while post-rebase validation runs.

Post-rebase validation on 978a0810f1 passes 10 append controls, 3 return-owner controls, 8 container-owner controls and 373 SPL tests. The newly merged concat-reset suite initially has 5 pass / 2 fail, only on hardcoded clean stdout expectations: optimized and unoptimized outputs are identical, and pinned PHP emits the same default warnings to stdout. Test fixtures now explicitly choose display_errors=stderr, compare both streams between optimizer modes, and require the expected float-key/undefined-key diagnostics. All original IR scratch-state assertions and stdout values remain. All 7 concat-reset tests pass; production diagnostics are not suppressed or changed. Oracle default/stderr captures are in post-rebase/diagnostic-controls/.

Fresh matrix-v5 passes 30/30 PHP-matching native runs with clean heaps on the three executable targets; all five targets assemble. It records the rebuilt post-rebase compiler and runtime library. Fresh unchanged ArrayObject phpt-v5 replay is running; no final status asserted yet. Rebase and diagnostic fixture learnings retain the old failure evidence and the exact merge-tree proof.
