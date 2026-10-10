# Scheduler branch integration with origin/main

## Checklist

- [x] Publish WIP checkpoint ae2c6bd and verified null-default follow-up 516dea0b36.
- [x] Fetch origin/main and record exact integration parents.
- [x] Preserve inherited vendor deletions in a targeted recoverable stash.
- [x] Start an ordinary no-commit merge without rewriting published history.
- [x] Reconcile non-generated conflicts, preserving scheduler/ownership/ABI work
      and upstream behavior/docblocks rather than choosing one side wholesale.
- [x] Regenerate builtin documentation from the reconciled contracts and bindings.
- [ ] Compile and run focused scheduler, binding/ownership, native/eval and target
      controls on the integrated source; previous-parent checks are historical.
- [ ] Commit and push the validated merge, then verify PR #958's exact head/CI state.
- [ ] Restore the inherited local vendor state without including it in the merge.

## Exact state and recovery

The maintainer requested merge/rebase of origin/main on 2026-10-09. An ordinary
merge was chosen because the branch is published and a force-push is unnecessary.
Our pre-merge parent is 516dea0b36 (full ID available from HEAD); the incoming
parent is afe32d3efe27209b87eb68d4b2adcda8dbc5eefd. There were 55 branch-only and
1,719 incoming-only commits before integration. PR #958 is draft and currently
conflicting; no CI runtime success is inferred from its label classifier.

The seven inherited example-vendor deletions are stored in stash
46b10c28d5f6c08e10fa3fb8a78f317dde039fdc, named
scheduler-main-merge-516dea0b36-20261009-inherited-vendor-only. Apply by exact SHA,
not by a moving stash index, after the merge is committed. Keep this recovery
entry until restoration is verified. The local .codex/, .kioku/, .mcp.json and
scratchpad/ remain outside publication; incoming main tracks none of those paths.

The initial merge reported 962 unresolved paths: 840 generated builtin pages
and registry artifacts, plus 122 other paths. Generated files may use incoming
main as a provisional baseline but MUST be regenerated from reconciled source
before acceptance. The other conflicts include build/package wiring, native/eval
ABI, codegen/runtime, EIR lowering, checker/effects, test infrastructure and manual
docs. This is an integration task, not a request to discard either implementation.

The incoming guide adds mandatory Rust function docblocks and pinned third-party
GitHub Actions; preserve these while carrying scheduler-specific behavior. Do not
change Cargo release versions or changelog content during ordinary development.
The pre-merge 83/24/19 focused results and Linux/iOS probes do not prove this merge.

## Initial resolutions

Generated builtin pages/registries now use the incoming baseline provisionally,
including deletion of obsolete web/trigger_error pages; regenerate after source
reconciliation. Build/package lists retain both upstream mbstring and branch
Parallel contracts/bridge. Third-party action pins from main remain intact.

ABI collision: incoming main already uses invoker argument 3 for lexical scope
(i64, -1 global), while the scheduler branch used argument 3 for scalar strictness
(u64, 0 weak / 1 strict). The integrated ABI must retain both: descriptor,
boxed arguments, lexical scope, then binding policy. Magician announces version 4
and its prebound native-call boundary passes (-1, 0). Emitted AOT/internals,
wrappers, Fiber, curl/PDO/notification and eval producers must all migrate to the
fourth policy argument before this merge can build or be accepted. Native test
stubs are updated to the composed four-argument signature, not one side's ABI.

Runtime feature-bit collision: main assigns bits 12-15 to handler state, object
clone, mbregex and MIME support. Preserve those incoming assignments; the
integrated ctx-register, Async reactor and Parallel bits append at 16-18. Keep
ctx-register active in RuntimeFeatures::none(), since this branch's codegen
requires it. Old mb_strlen detection must move to main's mbstring family, not
drop its new bridge/provider behavior. Runtime cache tests need the composed map.

Incoming main also introduces mutable mbstring runtime cells absent from the
current ctx layout: catalog array, deferred capture head/tail/draining and native
INI activation. They must be routed/isolated or safely refused for workers before
Parallel acceptance; do not retain raw process-global heap pointers by merely
choosing main's .comm definitions. Preserve immutable diagnostic data separately.

After this first pass, 105 paths remain unmerged. The read-only Rustdoc classifier
found no remaining file whose incoming delta is purely Rustdoc, so whole-file
ours selection is not justified. The generated baseline is provisional only;
840 generated conflict resolutions are not source/behavior validation. No merge
commit or merge push has been made. HEAD remains the published follow-up
516dea0b36250ec9cc83e09040cbe296a3defb57 and MERGE_HEAD is the incoming main SHA.

## Integrated source checkpoint (2026-10-09)

All Git conflict stages are now reconciled, including three modify/delete cases that
the stage-3-only inventory did not list. The obsolete array_filter,
array_filter_refcounted and array_find_any_all emitters were removed after verifying
that incoming main's boxed predicate helpers own their production routes. They are
recoverable from the pre-merge parents. A residual marker in array_udiff_uintersect
was found by Cargo and removed; its shared boxed helper retains lexical scope and
now explicitly forwards weak binding policy to the owned-argument boundary.

The integrated library passes cargo check --lib. This followed a real compiler
check reporting 33 API/structure errors, then 5, then zero. The remaining three
warnings identify an obsolete static-closure lifetime helper superseded by main's
prepublished result and operand-owner machinery. No executable scheduler, binding,
ownership or target result is implied by a successful library check.

Changes carried through the merge:

- Descriptor invokers and their consuming caller boundaries independently spill
  scope and binding policy. Magician uses ABI 4; prebound eval arguments are not
  coerced a second time. Every AOT producer still needs executable validation.
- Invoker argument owners, managed reference cells, resolved defaults, PHP return
  ownership status and backtrace state use incoming main's mechanisms. Parallel
  Fiber-suspend guards and physical parameter-order validation remain enabled.
- x86_64 indexed invokers reload argc without borrowing r14; associative argc uses
  rbx. The eval builtin dispatcher spills its eval context while r15 retains
  result-out. Other automatically merged r14 writes remain an audit requirement.
- Internal Fiber/curl/PDO/notification/error/exception callbacks explicitly supply
  global lexical scope and weak binding. Array callback helpers preserve their
  caller scope and explicitly supply weak policy to the composed ABI.
- Descriptor result unboxing retains main's ownership checks and scalar/nullable
  cleanup while preserving direct transfers for owned strings and heap payloads.
  Mixed-to-string conversion consumes the explicit owned-return status rather
  than persisting a Stringable result twice.
- C lifecycle/free entries were ported into main's new lifecycle leaf. They save,
  publish and restore the host context register; repeated init preserves a live
  arena and the incoming startup/mbstring request initialization contract.
- EIR calls use main's source-evaluation roots, result staging and reference-return
  lifetimes instead of legacy run()/method cleanup handlers. Branch snapshot,
  scalar-binding, worker-safety and callback guards are retained. Variadic inferred
  return-carrier normalization is shared with metadata through dynamic_params.
- GC recounting keeps the worker arena base, property GC metadata uses main's
  managed-cell representation, and bounded concat/implode/inet_pton and ASCII
  origin-tracking paths were retained. Runtime feature tests and Linux build
  scripts combine Async/Parallel with the mbstring family, without the removed
  mb_strlen feature field.

Current integrated checks:

- elephc-builtin-contract library tests: 52/52, both default and curl feature.
- Magician native_scope tests: 27/27; ffi native-function registration: 3/3.
- Three isolated extraction tests (PHP control-byte defaults, neutral unions,
  namespaced prelude declarations) pass. Four edited Python files parse; both
  edited Linux shell scripts pass bash -n.
- The abi::tests Magician filter matched ZERO tests and is not validation evidence.
- Targeted diff hygiene passes. Local rustfmt checks are not compile/test evidence
  because the repository disables formatting.

Open integration gates before commit/push:

1. Compile and execute the affected AOT/invoker/Async/Parallel/GC regressions, with
   current supported-target evidence. New emitter tests are not yet executed.
2. Route or safely isolate incoming mutable mbstring, handler, diagnostic, GC,
   resource-inventory and magic-set state; review direct heap-base/r14 uses. Merely
   retaining their main .comm baseline is not safe worker isolation.
3. Verify Fiber boxed arguments on ABI-v4 escape, generator last-key slot sharing,
   pending cleanup exception chains, result retirement and descriptor release.
4. Audit typed by-reference variadic preservation and source-evaluation vs binding
   order across the composed planner/owner mechanisms. Focused historical tests
   do not settle those full ABI contracts.
5. Regenerate builtin docs and registries, run their full extraction/audit/boundary
   gates, and complete Rustdoc/assembly-comment hygiene on the integrated code.
6. Commit/push the validated merge normally, restore stash
   46b10c28d5f6c08e10fa3fb8a78f317dde039fdc by SHA after the merge commit, then verify
   PR #958 head/CI. Full array/I/O implementation and exact-artifact independent
   Codex/Kimi/GLM/DeepSeek consensus remain separate, open acceptance gates.

Kioku acknowledged decision-scheduler-main-integration-composes-lexical-scop-416b820a
after a long CLI capture. Its fact records the earlier first-pass state (105
conflicts), not the current result. Reverify its source after this ledger update;
do not promote that historical decision into an integrated measurement packet.

### Focused invoker test compilation

The root library test build found one remaining synthetic enum-transcriber pattern
that omitted main's `generics` field. It now explicitly refuses unsupported generic
enum transcription rather than dropping that metadata. The obsolete
static_closure_lifetime module/file was removed after confirming it had no remaining
caller; main's result/descriptor roots supersede it. Both it and the earlier three
array helper removals are recoverable from the published pre-merge parent.

The first executable emitter-test run was 26/28. The two failures expected legacy
global _concat_off/_exc_call_frame_top names despite correct context-relative stores
in the emitted assembly. Assertions now check the exact ctx fields and pre/post-call
store placement, not merely a comment or symbol name. The targeted root-package
rerun is still pending at this log point. Future root unit runs must use `-p elephc`
as well as a filter: omitting the package rebuilds unrelated workspace test crates.

That rerun subsequently completed: 28/28 invoker tests pass, including the composed
scope/policy frame, defaults, managed reference owners and all-five-target emitter
controls. The root library test build is warning-free after retiring the obsolete
closure helper. This still does not execute generated PHP programs or prove worker
runtime isolation, I/O completion, target linking or model consensus.

The semantic Kioku recall for scheduler/main/ABI4/context was launched once via the
local CLI and has not yet returned. Do not claim a semantic hit or a successful
integrated measurement capture from a live, empty process.

Four additional focused all-target emitter controls pass: descriptor payload-owner
transfer, per-tag Mixed/string owned-return handling, shared variadic return ABI,
and the consuming argument boundary's independent scope/policy spills. The new C
lifecycle ordering test first failed on Linux AArch64 because its expected publish
sequence ignored ELF PIC/GOT materialization. Its assertion now uses the exact
target-aware publication sequence while still requiring save-before-publish and
restore-before-return for init/shutdown/free. That rerun is pending at this point.

### Request-state isolation and worker teardown (2026-10-09)

The C lifecycle ordering rerun subsequently passed its all-five-target emitter
control. Semantic Kioku recall also completed, returning three relevant packets
with explicit warnings: moved citations/context not reverified, and an expired
Candidate62 measurement. They are historical context, not current integration
acceptance. The CLI exposes a code graph as well as verified memory; prefer that
surface for subsequent discovery when the MCP tools are absent.

Forty-five incoming request cells now have distinct context offsets, defaults,
symbol routing and owner verification. The two initial root tests pass (layout
and initialization/verification on all five emitter targets). Initialization
preserves the pool's state pointer and uses an immutable default reporting mask.
These tests do not execute workers or prove that every consumer is routed.

The current cleanup additions are implemented but their rerun is pending:

- Handler pop drains preceding registrations through the bounded exception
  preservation boundary; feature gates match diagnostic helper emission.
- Native mbstring reset still runs after a caught deferred PHP cleanup throw.
- Stream, Buffer, cycle and inventory cleanup preserve accumulated exceptions
  rather than abandoning sibling cleanup on the first destructor throw.
- Stream teardown marks a native inventory incarnation closed before closing
  its descriptor. Otherwise inventory cleanup could close the same descriptor
  twice, potentially closing a new descriptor reused by another worker.
- Worker inventory draining no longer invokes request-cursor reset. The existing
  resource-id registry is intentionally process-wide and lock-protected; resetting
  its cursor during one worker's cleanup would interfere with other workers.
- The C pool fixture accounts for the inserted 360-byte block, rendezvous-tests
  all 45 new cells with two threads, restores their defaults and arena base, and
  passes/verifies the fifth worker-entry release-status argument.
- Its fatal stream probe allocates a real inventory node, observes incarnation
  retirement before the TLS/native close hook, and checks the global cursor is
  unchanged. This executable probe has not run on the integrated source yet.

Final owner verification still quarantines live owners and pending Throwables.
Later destructors can republish owners; never erase those pointers merely to
make a context reusable. Direct-symbol/register audits, actual native/e2e probes,
all supported targets, docs regeneration and exact-artifact consensus remain open.
No merge commit or merge push has been made; the inherited stash is not restored.

The expanded root context run completed 22/26. All three new request-state tests
and the ordered cleanup/verification control passed. Four broader controls failed:
reserved-register scratch on both architectures, kitchen-sink legacy-state access,
and default-arena materialization count. The x86 findings were real: new hash
reference markers, automatic-key overflow handling and exception-chain traversal
borrowed r14. The write-through return also loaded an r15 spill into r14. Corrected
paths use preserved r15/rbx or the declared r11 scratch. GC status/destructor scans,
managed reference-cell lookup and heap-debug reports now load the active context's
arena/extent instead of raw main-arena symbols.

The ARM findings were paired context saves/restores in worker release and native
callback frames, not scratches. The audit is extracted into a cohesive test leaf:
frame reloads require a matching context save in the same global helper. Negative
controls reject another register's frame slot, unmatched ARM pairs, cross-helper
reloads and a read-through-r14 instruction that overwrites r14 itself. This also
closes the old x86 audit's overly broad allowance for any memory load into r14.
The corrected expanded rerun is active; do not call these four gates green yet.

Kioku usage feedback, observed under concurrent build load: semantic recall returned
useful stale/moved-citation warnings. Code-graph callers reported no references to
per_context_symbol_offset although ABI source has qualified ctx:: calls; this is
insufficient discovery evidence, not proof of no consumers. A learn capture spent
over 21 minutes in its duplicate guard's recall/citation-relocation pipeline, with
an rg scan for an old PHP TaskGroup::spawn signature. Its own process group was
verified and terminated (exit 143), with no matching packet written. Retrying this
one capture uses a precise source range and KIOKU_FORCE=1 to bypass duplicate lookup;
no Kioku source/configuration or other worktree process was changed.

The retry acknowledged decision-integrated-worker-teardown-preserves-request-own-f789481f,
pinned to lines 214-236 rather than the entire ledger. The packet was inspected;
it explicitly leaves native and full target/model acceptance open.

The expanded context gate subsequently passed **30/30**, after routing incoming
serializer/key-conversion concat ABI accesses, migrating GC cache-bin addressing,
and recognizing Linux AArch64's exact :lo12: main-context publication relocation.
The legacy-state scanner now checks exact symbol tokens regardless of the address
register or relocation form; negative controls cover x19 and GOT cases that the
old x9/RIP-only check missed. The all-feature reserved-register audit passes all
five emitter targets. This is not native execution or all-five-target linking.

Against that built unit artifact, focused controls passed: stream ownership 2/2,
heap-debug live reports 1/1, boxed invoker-marker write-through 1/1, PIC ABI 7/7,
and the broader invoker filter 40/40 (overlaps the marker control; do not sum these
as unique tests). Inventory controls first passed 3/5: two assertions assumed legacy
symbol names or the wrong helper emission order. They now require actual ctx head/
tail addresses and delimit drain before insert on both architectures; rerun active.

The C probe passes host syntax checking. Its wrapper is named run_worker_entry to
avoid ELF collision/recursion with the runtime's __rt_parallel_worker_entry label;
invalid-input testing calls the raw five-argument entry and checks that a rejected
call does not alter its release-status sentinel. Native execution is still pending.

Additional inspection point before any full-isolation claim: the Fiber entry leaf
still publishes the main _rt_ctx after a zeroed fake frame. Determine whether worker
Fiber creation is safely diagnosed or must inherit the worker context in its initial
frame; reproduce before treating this as a closed or confirmed executable defect.
The builtin-docs skill has been read for the required regeneration gates; exporter
build/render/audits have not been rerun on this integrated source yet.

The inventory rerun is now **5/5**. All changed-helper controls are green at the
unit/emitter level; the actual staticlib/C context-pool test has been launched
with the exact test name on the macOS host. Its Cargo process must be polled to
terminal before any other heavyweight build or claim of native success. Source
compilation, generated PHP, C host link and execution are separate possible failure
stages; a live Cargo job is not a passing native result.

### Native pool execution and Fiber follow-up

The first native pool execution failed INVENTORY-INCARNATION-NOT-RETIRED. Added
failure-only C snapshots showed registration was correct (id 99, fd 3, kind 1),
but none of the native close hooks ran. The earlier simple worker probe had already
consumed/released its context under the composed five-argument entry ABI. Reusing
that pointer without a fresh acquire made release a no-op. The fixture now reacquires
after the simple worker, and production worker entry rejects a zero acquisition
back-pointer before publishing the ctx register, on both architectures. A native
negative control checks that rejected reuse leaves release-status output untouched.

After those changes the exact staticlib/C pool test passes on macOS AArch64 (1/1).
Its C compiler/linker and execution succeeded, including native fd hook retirement,
global resource cursor preservation, two-thread marker isolation, pool racing,
dirty-slot refusal and released-entry rejection. This is host-native pool evidence,
not executable Linux/iOS, complete PHP-array/I/O behavior or model consensus.

Fiber inspection confirms both constructors zero the initial saved-register frame,
then Fiber entry explicitly republishes main _rt_ctx. Generators reuse the same
constructor through codegen/block_emit.rs. A new heap-debug PHP regression creates
a Fiber inside a Parallel worker and changes its GC state; the parent must still
print 42|main-enabled. It has been added but not executed: the first test-build
command used a bare --exact name, whereas its real name is
codegen::parallel_execution::parallel_worker_local_fiber_inherits_worker_runtime_context.
Poll that build to terminal; a zero-match result is not evidence and must be followed
by the fully qualified exact name. Do not restart a still-live build.

That first codegen-test build terminated with a real fixture compile error, before
test selection: compile_and_run_with_heap_debug returns ProgramOutput, not String.
The new regression now checks success, stdout and the clean heap-debug summary,
matching existing Parallel controls. It is relaunched with the fully qualified
exact name; its current Cargo process is live. No Fiber runtime change has been
implemented and no successful Fiber-worker execution is claimed at this checkpoint.

The fully qualified Fiber regression then compiled and ran exactly one test, which
crashed on the elephc-parallel-1 thread. LLDB reproduced EXC_BAD_ACCESS at address 8
in __rt_fiber_entry+72: x19 (current Fiber) was zero, while x28 pointed at main rt_ctx.
The entry's first instructions explicitly materialized that main pointer. This is
a confirmed executable isolation defect, not merely a suspicious emitter shape.

The common constructor now seeds the creator's context in the actual switch-save
slot (AArch64 x28 at 72; SysV r14 at 8), after zeroing and before installing entry.
Entry no longer republishes main. Both constructors are updated; Generators reuse
the same path. An all-five-target emitter control checks seeding/order/slot bounds,
and the reserved-register audit has negative controls for main publication in entry.
The corrected Fiber regression rerun is active. A second heap-debug Generator
worker regression was added; neither new executable result is yet claimed green.

The corrected exact Fiber regression subsequently passed **1/1** on macOS AArch64,
including 42|main-enabled stdout and the clean heap-debug assertion. The Generator
exact control is now launched separately; its result must match one executed test.
The all-target constructor/switch unit controls and post-fix Linux/ordinary Fiber
controls still need execution. Kioku acknowledged the earlier native pool measurement
as measurement-integrated-staticlib-context-pool-passes-native--dcd8f70d, with a
20-commit freshness budget and explicit remaining gates.

The exact heap-debug Generator worker regression also passed **1/1** on macOS
AArch64, preserving main GC state and clean ownership. The target-wide coroutine
constructor/switch and context-register unit gates are now running serially.
The EIR architecture inventory passed: 741 registry AOT builtins, zero structural
errors and zero backend-dependent homes. This audit may silently run Cargo via
the exporter freshness fallback; treat it as a possible heavyweight build, not a
pure Python-only operation, and serialize it with test/bridge builds.

Linux preflight was verified broken on Docker Desktop: the daemon reports
/var/lib/docker, but host df cannot access that guest path. Both runner scripts now
measure free space through an ephemeral --rm container using the prepared test
image, before creating the Cargo target volume. The shared helper fails closed on
Docker failure or malformed measurements. Four mocked controls (valid, zero,
malformed, failed command) pass, all four shell files pass bash -n, and the actual
ARM64-image probe reports 21,801,072 KiB free (about 20.8 GiB) at this checkpoint.
No Docker image, unrelated container or volume was pruned. This is tight relative
to the 20-GiB runtime disk guard, so do not infer enough room for both target builds
or disable the guard to manufacture target evidence.

Post-coroutine-fix unit reruns completed: constructor/switch **2/2**, including
creator-context seeding on all five targets; context/register/storage controls
**30/30**, including the new negative entry-publication controls. Both focused
native coroutine regressions are green, but executable Linux coverage is not yet
proved. The full generated-docs workflow is now launched serially: canonical curl
exporter build, render/registries/module/comparison generation, docs/site audits,
EIR target-architecture audit and diff hygiene. Stop at its first actual failure;
do not treat the previously reconciled generated baseline as already regenerated.

The full canonical docs workflow subsequently completed with exit 0. It exports
1,185 builtins, 209 classes and 1,117 constants, renders 2,254 pages, updates zero
module blocks and regenerates the PHP comparison. Docs audit reports zero errors;
site compatibility validates 2,276 generated pages; EIR architecture inventory
again reports zero structural errors and zero backend-dependent homes. Whole-tree
unstaged diff hygiene passes. Inspection found 1,041 generated paths changed from
the reconciled incoming baseline (1,070 inserted and 1,070 removed lines), rather
than treating the original 840 generated conflict resolutions as sufficient.

Kioku acknowledged the coroutine measurement as
measurement-worker-local-fiber-and-generator-context-regress-b5f0f8f5. No Docker
volume matching this worktree's runner hash 8067abf45272ac3f exists, so no owned
target cache was assumed reusable or deleted to increase disk space. Linux native
validation and touched-Rust/assembly hygiene remain separate gates before publishing
the merge checkpoint; full array/I/O and absolute exact-artifact reviews remain open.

### Native Linux context-pool probes without Cargo target volumes

The explicit ignored fixture emitter emit_linux_context_pool_probe_bundles ran
exactly once and passed. It invokes the current compiler's AST/EIR --emit-asm
staticlib route for a scalar export, and generates the real feature-none runtime
in static-library boundary mode. The existing C pool host is reused verbatim.
No mock runtime helpers or borrowed bridge archive were substituted. The generated
bundles live under the test-owned temporary directory:
/var/folders/9d/f4jbjvxs0t5d94xl7cp671t00000gn/T/elephc_linux_context_pool_probes_93696_ThreadId(2)_0.

Both bundles were assembled and linked inside their corresponding existing Docker
images, then executed with exit 0 and POOL-OK. Each run uses --rm, one CPU and 512 MiB,
mounts only its own bundle, and performs cc/ar/cc, not a Cargo rebuild. This checks
the C pool/worker-entry/fatal resource cleanup/slot-race/quarantine surface on Linux
AArch64 and x86_64. It does NOT establish the Rust Parallel bridge, PHP-array ABI,
Fiber/Generator worker execution, full Async/I/O or the entire executable matrix.
The x86_64 image executes through Docker's amd64 support on the ARM64 host.

Provenance (SHA-256):

- ARM64 image: 3f506a232838c660a0258049609b99a32be19beeaff50f93bd691da0c599a96e.
- ARM64 auth.s: d29ae9ffab5ed6fd4b88e58950020226b0f194a12e0b466182e76832b9a10f90.
- ARM64 runtime.s: 350ba200fc6d03eb72737a8fac3ca4f2b0b098fc3c3c6d6c03613a5818490cd2.
- x86_64 image: db9f3dce9005b21801ae553692d7c5c69fac998efd890766faedeccfcbd47312.
- x86_64 auth.s: 2ee837b179e5a96874a0448294a46b591f9f1667facc4587f861e2e61df34d5e.
- x86_64 runtime.s: 451c4c62986baafbb072e23179d3275d1dbe1829133c7c367c1377416ace34ce.
- Shared pool-host.c: ee158d84c120c355a5cef43b5ccac2ad266abf711e6f570387a72de53948d571.

The normal Linux runner's disk guard remains intact; no Docker volumes/images from
other worktrees were deleted. The read-only Kioku recall launched for Linux probe
history is still pending; no result is inferred from silence. This thin native
probe is additional direct evidence, not a redefinition of whole-goal acceptance.

### Incoming Rustdoc policy reconciliation

The assembly-comment audit passes on the 20 currently edited ABI/runtime Rust
files, including the new request-state and register-audit leaves. A lexical Rustdoc
scan first had a syntax error, then was corrected; ordinary comments between a
Rustdoc block and item attributes are ignored rather than reported as missing docs.
The latest-edit scan found missing blocks in ctx owner-verification helpers,
Parallel execution controls, two iOS refusal controls, the stream drain helper and
one ABI test. Those functions now have responsibility-specific Rustdoc comments.
The shared Parallel contract's three sizing functions and four tests are documented,
and its focused library tests pass 4/4. A serial cargo check --tests is still active.
No function signature, attribute, assembly instruction or control-flow body was
changed by these documentation edits; the relevant alignment/diff checks pass.

The broader contribution inventory is 389 Rust paths relative to origin/main.
Its lexical scan found no missing module preambles, but reports additional candidate
missing function blocks across the Parallel bridge, preludes, binder/lowering
helpers and test modules. This is a remaining incoming-policy gate, not a reason
to weaken the acceptance checklist or claim that all 389 files are documented.
Candidates need source confirmation before edits; lexical scans are not a Rust AST
proof. Final exporter freshness/review artifacts must account for later doc edits.
Kioku acknowledged the Linux pool measurement as
measurement-current-linux-arm64-and-x86-64-context-pool-asse-cc00e990. The earlier
Linux-history recall remains a live process, with no returned result asserted.

The documentation pass now also covers descriptor param_validation, numeric_bounds,
pending_owners and container_storage. Responsibility-specific blocks explain
borrowed tag/marker validation, independent ABI4 scope/policy, signed integer bounds,
record clearing before release/alias transfer and key-preserving container leases.
The stale ABI-v3 preamble is corrected. These four files pass the lexical function
docblock check, assembly-comment alignment and diff hygiene; only comments changed.
The pre-existing Hash-to-indexed-only ABI gap remains explicitly documented, not
silently fixed by documentation or covered by focused pool tests.

Kioku notes surfaced drifted gotcha-descriptor-parameter-preflight-must-precede-mars-49526109.
Its C61 guidance says the descriptor has no strict_types policy and implies global
preflight before conversion. Current code validates at each physical parameter
position with independent ABI4 scope/policy and pending-owner cleanup. A replacement
decision was submitted with an explicit supersedes/reason edge, retaining the useful
declared-contract and earlier-owner cleanup rules. Capture is still live: do not
claim the old packet has been retired before acknowledgment. The original serial
cargo check --tests is also live (its package identity/cwd were verified separately
from other worktrees' Cargo processes); no other worktree process was interrupted.

The next coherent Rustdoc pass covers crates/elephc-parallel/src/value.rs and
value/tests.rs. Public ABI summaries now complement the existing Safety blocks:
single-root structural completion, exact floating bits, checked container slots,
nonnull cancellation ids, writer consumption at finish, complete-envelope validation,
and blob-borrowed byte views. Internal writer/parser helpers and all seven test/helper
functions are documented. The diff adds 25 and seven comment lines respectively,
with no deletions; lexical function coverage and diff hygiene pass for both files.
Focused value-codec tests have not been rerun yet; the original serial root
cargo check --tests remains active and must finish before another heavy build.

Graft's value.rs API lookup was ambiguous between three old scratchpad review copies,
so those historical spans were not treated as the current source. Kioku notes also
marked its raw-pointer-contract citation moved/context-unverified and the old EPV1
decision drifted. Current work used the explicit owned worktree file; no historical
packet was promoted into fresh codec/native acceptance evidence. The ABI4 note's
superseding capture remains pending, rather than assumed successful.

The failure-envelope module and its eight helpers/tests are now documented as a
cohesive follow-up: checked wire lengths, optional absent/present-empty sentinels,
copied EPFL fields/frames, exact parse exhaustion, borrowed C views and explicit
raw-pointer safety contracts. These edits add 36 and eight comment lines with no
deletions; their lexical docblock and diff checks pass. No encoding, ABI layout,
failure kind, lifetime representation or control-flow body was changed.

The original serial root cargo check --tests completed with exit 0, including its
trailing whole-tree diff hygiene. Subsequent focused bridge tests passed **3/3**
for value:: and **4/4** for failure::; their parent command still needs terminal
polling for its final git diff --check. Those are codec controls, not full native
Parallel/Async or array ABI acceptance. Remaining contribution-wide Rustdoc,
callback/target gates, publication and exact-artifact consensus are still open.

The parent codec-test command subsequently completed with exit 0, including its
final whole-tree diff hygiene. No root Cargo check or codec-test process is left
running from these serial commands. The Kioku ABI4 superseding capture and the
failure.rs graph/notes lookups remain pending; their silence is not evidence of
completed lineage/index refresh.

### Bridge documentation and durable verification continuation

Current Git reads confirm HEAD 516dea0b36250ec9cc83e09040cbe296a3defb57,
MERGE_HEAD/origin/main afe32d3efe27209b87eb68d4b2adcda8dbc5eefd and an empty
unmerged-stage inventory. The merge is still uncommitted and unpublished.

The job and executor documentation pass covers owned input/result copies,
registry release, FIFO reservations, spawn publication, worker status mapping,
context-release fallback and joining the actual OS thread rather than treating a
terminal job result as proof of thread exit. The prior focused commands returned
8/8 job tests and 5/5 executor tests with exit 0; later executor test-summary edits
are comment-only and their fresh rerun is tracked separately below.

The PHP-wire leaf and its tests now document safe-subset parsing, ordered array
keys, depth/count checks, native EPV1 ownership, copied EPFL failures and the
borrowed thread-local prepared pointer's expiration. Explicit raw-pointer Safety
contracts describe readable source ranges, writable outputs and allocator-matched
buffer release. This pass adds 58 and 14 comment lines respectively, with no
deletions or executable changes; it does not close the broader PHP-array ABI gate.

Kioku lineage is now confirmed from both packet files, not inferred from the earlier
pending capture: decision-descriptor-abi4-binding-validates-each-physical--18d9f519
supersedes gotcha-descriptor-parameter-preflight-must-precede-mars-49526109, and the
old packet records the reciprocal superseded_by and reason. The obsolete no-strict-
policy/global-preflight advice must not be used for current ABI4 binding.

During tool-session continuation the focused PHP-wire/executor command's observation
handle disappeared while its own Cargo process remained live (PID 79102, cwd verified
as this worktree, parent wrapper 79064). Both exact owned processes are being stopped
so verification can restart with a durable output log. No result from that lost
output is counted as passing, and no other worktree's Cargo process is interrupted.

The two owned orphan processes were subsequently confirmed absent. The durable
replacement command ran all six modules of the focused elephc-parallel bridge
library: **44/44 passed**, 0 failed, 0 ignored, exit 0. Its complete log is
/tmp/elephc-parallel-integrated-4XZASn. This includes 12 PHP-wire, 5 executor,
8 job, 4 failure, 3 value and 12 bridge/storage tests; do not add the earlier
overlapping filtered counts to these totals. The bridge entry tests use explicit
mock callbacks and are not native emitted-context or whole-scheduler acceptance.

The bridge root now documents worker-table borrowing, exact release/quarantine
handoff, thread-local scope guards and raw transfer-pointer safety. The PHP-wire
and bridge root changes pass the lexical function-doc scan, and focused diff
hygiene passes. These are comment-only changes, including correcting the root's
stale future-bridge wording. Broad contribution Rustdoc and final exporter
freshness remain open. The next executable gate is the descriptor parameter
binding module against the integrated compiler, with a durable log.

The descriptor parameter-binding test leaf now documents its 16 runtime-selected
callback controls and no longer suggests a global pre-marshalling preflight. Its
examples cover fixed/variadic declared contracts, weak Stringable/numeric coercion,
named array COW, throwing conversion cleanup, defaults and returned payload leases.
The shared transfer classifier and checker are documented at their existing v1
boundaries: recursively copied values, direct-only Cancellation, static target
resolution, capture/effect restrictions, literal shape proofs and TaskGroup escape
diagnostics. Their function-doc and focused diff checks pass; only comments changed.

The integrated executable descriptor gate is live, serial, with durable log
/tmp/elephc-descriptor-binding-integrated-X0qWN6. Cargo first waited for the shared
package-cache lock, then began compiling this worktree's root compiler. Its own
Cargo PID 87257 and rustc child 90737 were verified live; no terminal result or
executable pass is inferred yet. A Kioku measurement capture for the 44 bridge
controls is also live, with log /tmp/elephc-kioku-bridge-4Y5KuM; acknowledgment has
not yet been observed. Existing notes/graph queries with lost observation handles
remain distinguishable from completed retrieval and are not current code evidence.

Live remote inspection now reports main at
4f79c92c9eebf8925a611e42dae6a07f6a2e1027 rather than the current merge parent
afe32d3efe27209b87eb68d4b2adcda8dbc5eefd. A fresh fetch is in progress to inspect
that delta; do not silently rewrite MERGE_HEAD or imply the newer parent was
already tested. Finish the pinned integration and account for the follow-up before
claiming current-main publication. PR #958 is OPEN and draft, with head exactly
516dea0b36250ec9cc83e09040cbe296a3defb57; GitHub reports mergeStateStatus UNKNOWN,
not a verified clean merge or passing CI. No push was performed.

The fetch subsequently completed with exit 0. origin/main and FETCH_HEAD now both
resolve to 4f79c92c9eebf8925a611e42dae6a07f6a2e1027, while MERGE_HEAD remains the
original afe32d3efe27209b87eb68d4b2adcda8dbc5eefd. The newer delta contains 13
commits and one first-parent merge, PR #1694 (monitor source names). The scoped
ABI/backend/type/bridge inventory shows src/ir_lower/tests/eval_ownership.rs as
the changed path in those trees; this is an inventory, not a semantic audit of
the complete monitoring delta. Inspect the full delta before the follow-up merge.
Use the explicit original parent SHA for pinned-integration inventories rather
than treating the now-advanced origin/main ref as that old baseline.

The full fetched delta is seven paths: src/ir_lower/tests/eval_ownership.rs,
src/monitor/{decl_ranges,exact,mod,render,tests}.rs and src/parser/mod.rs. It has
not yet been merged into this in-progress integration. The executable binding
command has advanced from compiling the compiler library to rustc compiling
tests/codegen_tests.rs (child 93644 of Cargo 87257). Keep that existing command
and its durable log; do not restart because the large test binary is slow to link.
No descriptor result or Kioku capture acknowledgment is yet counted as complete.

### Incoming monitoring delta and memory acknowledgment

The complete seven-path incoming diff was inspected against the two pinned main
SHAs. It replaces line-based declaration guessing with lexer-backed namespace and
class ranges, recognizes generator entry suffixes without changing complete PHP
names, and aggregates generic instantiations for exact-profile source attribution.
The parser change is a Rust API forwarding the existing class-name token grammar;
it does not add PHP syntax. New monitor controls cover namespaces, comments/string
braces, contextual names, generic headers, suffix ambiguity and exact totals. The
eval-ownership delta splits two all-target tests into separate target entries while
preserving their substantive assertions. None is merged or counted as passing here
yet; the follow-up still needs ordinary integration and focused monitor/target checks.

Kioku capture is now acknowledged in the durable log and its packet exists:
measurement-integrated-parallel-bridge-library-passes-44-uni-a60c7691. It pins ledger
lines 550-556 and has a 20-commit freshness budget. The body explicitly records both
merge parents and uncommitted state; measured_at is the old HEAD, not a claim that
the integrated source was committed at that SHA. No emitted native acceptance is
inferred from the 44 unit controls.

The next cohesive documentation pass adds 15 responsibility-specific comment lines
across argument snapshot cleanup, declared-string return ownership/previous chains,
return-binding effects and worker-entry effect projection. The lexical function-doc
and focused diff checks pass with no executable changes. Kioku api initially reported
no indexed file for the existing string-return leaf; a single-file scan is live to
repair that coverage rather than treating its absence as evidence about the code.
The supplementary graph query has durable log /tmp/elephc-boundary-graph-54sBQB.

The descriptor gate completed test-binary compilation in 24m30s and is now executing
its 16 selected tests. The first test's helper is building the required elephc-crypto
bridge and has waited on the shared package-cache lock. Cargo 87257, its test runner
22597 and helper Cargo 23768 were observed live; no passing test count is asserted
before the actual output. Preserve this command/log across observation-session loss.

### Named descriptor integration regressions and resumed correction

The original descriptor batch completed: **11 passed, 5 failed**, 0 ignored,
with complete diagnostics in /tmp/elephc-descriptor-binding-integrated-X0qWN6.
Three variadic tests exited 255 with Unknown named parameter ($tail, $value,
$named). Two named array-prefix tests failed static checking with callback
parameter $prefix expects array, got mixed. The batch is failed evidence, not
native acceptance; its Cargo/test processes were subsequently confirmed absent.

The interrupted correction is present after resuming on 2026-10-10. The shared
storage predicate now recognizes boxed Mixed and typed key-preserving Hash
collectors, retaining rejection of raw indexed scalar collectors. The invoker
reads a typed Hash's element contract and forces Mixed elements for a boxed
Mixed collector, rather than inheriting an unrelated input-container element.
Static checking defers an explicitly opaque Mixed value only on descriptor
by-value binding into a boxed target. Direct calls, raw indexed targets and
supplied references are not newly relaxed. The existing dynamic-spread projection
path is preserved; the larger raw Hash/indexed ABI gate remains separate.

The shared storage/source contract unit test passes **1/1** after the correction.
A new heap-debug regression requires invalid input to fail before callback entry
and a valid array to run exactly once; the PHP CLI oracle prints
TypeError|callee|2. The corrected descriptor batch now has 17 tests and is running
serially with durable log /tmp/elephc-binding-postfix-s9qvAm. No corrected native
pass is claimed yet. Assembly-comment alignment and focused diff hygiene pass.

Kioku semantic recall returned the rejected blanket-Hash C61 decision, the current
ABI4 physical-binding decision and the callable-return alias gotcha. These remain
compatible with the shape-aware correction: no uniform Hash transport was imposed.
Its single-file scans repaired the string-return leaf's missing index and the
invoker's stale ABI offsets. Supplementary graph results include historical
scratchpad review copies alongside the current owned file; only current-file
edges were used, never historical spans as current executable evidence.

The resumed post-fix pipeline completed with exit 0: storage/source contract
**1/1**, native macOS descriptor binding **17/17**, followed by whole-tree
git diff --check. Log: /tmp/elephc-binding-postfix-s9qvAm. All five previously
failing cases are green. The strengthened negative test's callee marker confirms
the invalid opaque value is rejected BEFORE target entry; its heap-debug cleanup
also passes. This is focused macOS evidence, not a full array ABI/target claim.

The freshly built library-test artifact was then run directly with the invoker
filter: **40/40**, exit 0, log /tmp/elephc-invoker-postfix-unit-ijqyZa. This checks
cache/default consumers, argument/result ownership and target-aware emitters,
including the supported-target scope/argc/cleanup controls; it is distinct from
native Linux execution. No new build or heavyweight parallel test was started
for this short emitter run. Neighboring strict dynamic and named-variadic checks
are serially running from the freshly built codegen test artifact with log
/tmp/elephc-binding-neighbors-KdEX7W; do not infer their outcome until completion.

A renewed lexical Rustdoc inventory against the ORIGINAL integration parent
still reports candidates in scheduler preludes, runtime/lowering leaves and test
modules. Completed bridge/binder documentation is not full contribution hygiene.
Canonical docs freshness, remaining focused gates, original/follow-up merge
commits, fork push, inherited vendor restoration and exact-artifact review remain
open. The Kioku packets/local scratch directories stay excluded from publication.

The neighbor batch subsequently completed with exit 101: strict dynamic binding
**3/3** and ordinary named variadics **8/8** pass, but the three user-relay controls
are **2/3**, with user_function_relays_named_variadic_arguments rejected because
call_user_func_array() sees its variadic argument list as Iterable rather than array.
This is a separate representation leak, not a reason to accept arbitrary Traversable
objects as call_user_func_array() argument arrays.

Known named/runtime-keyed user-function tails now reuse the existing shared
descriptor_variadic_container PHP-array contract in initial resolution, parameter
specialization and associative callback specialization. Source element declarations
are retained separately. Variadic slots are located by name in callback specialization
rather than assuming the final slot; forwarded dynamic array variables remain marked
as potentially keyed. A known dense array literal does not gain a keyed marker, so
its existing indexed path remains available. No blanket list-to-Hash conversion or
public API was introduced. The relay correction is pending executable verification,
with log /tmp/elephc-variadic-relay-fix-WG3O2k; the 17/17 result above predates this
additional correction and is not promoted to a latest-source acceptance claim.

The relay correction's first pipeline completed: its exact failing relay **1/1**,
named-variadic family **8/8** and descriptor binding **17/17** pass. Durable log:
/tmp/elephc-variadic-relay-fix-WG3O2k. A subsequent one-line dense-literal guard
keeps syntactically known indexed literals out of the keyed-source classifier;
those results predate that last guard. The latest-source named family, all three
relay controls and the boxed by-reference named-tail control are now being checked
serially with a new durable log; no fresh result is assumed from the earlier run.
No merge commit, fork push, vendor restoration or reviewer consensus is claimed.

The latest-source slice log is /tmp/elephc-named-final-slice-didbvC, observation
session 91064. Its Cargo 57174 and rustc child 63553 were verified live after the
dense-literal guard, so continue that command rather than starting another build.
Kioku acknowledged the named-binding fix as
bug_fix-named-descriptor-binding-recognizes-boxed-and-ha-e910e6db; its capture log
is /tmp/elephc-kioku-named-fix-DrIC4m. It cites the storage/boxed-binding code and
the bounded 17/40 evidence, explicitly retaining the relay correction's separate
gate and the open complete-ABI/I/O/target/reviewer requirements.

The final slice completed with exit 101: latest-source named variadics **8/8**
and user-function/method relays **3/3** pass. The boxed by-reference named-tail
control instead fails **0/1** with SIGSEGV. Its existing regression is
tests/codegen/callables/state_and_variadics.rs::test_by_ref_variadic_scalar_and_named_tail_writeback_use_boxed_storage.
The complete log is /tmp/elephc-named-final-slice-didbvC. Automatic LLDB replay
shows EXC_BAD_ACCESS address 0 at fn_replace+204, `ldr x9, [x0]`, with x0=0.
The first failing program call is the direct `replace($position, named: $named)`;
the later descriptor/reference/count variants have not yet executed successfully.

Retained native artifact:
/var/folders/9d/f4jbjvxs0t5d94xl7cp671t00000gn/T/elephc_test_76969_ThreadId(2)_0/test.
The callee saves incoming x0 then loads through that saved address before the
dimension update. The next diagnosis must compare direct-call variadic reference
packaging with this callee entry contract; the trace alone does not establish
whether the regression predates the current correction. Do not remove/disable
the reference test, convert references to copies, or claim full by-reference ABI
acceptance from the passing by-value cases. Cargo/test processes for this final
slice are terminal, and whole-tree diff hygiene passes. Merge publication remains
unperformed while this real runtime failure and the other recorded gates remain.

Kioku acknowledged the reference failure as
gotcha-integrated-by-reference-variadic-collector-crash-24ae769e, pinned to this
trace checkpoint and the existing regression source. The named-binding fix and
remaining reference failure are separate packets, so recall must not treat the
positive named-binding evidence as whole reference-ABI closure.

### Reference collector lifetime and scalar retyping diagnosis

Static LLDB disassembly of the retained failing binary identifies the first
failure exactly: main stores the new collector in frame slot -0x3b0, clears that
slot on successful argument-evaluation handoff at main+1200, then passes its
address to fn_replace at main+1396. The callee dereferences that emptied slot.
This is not evidence that the source-variable reference cells should be copied.

The correction acquires a stable owned SSA collector before retiring the snapshot
slot, and includes that detached synthetic reference collector in ordinary user-
call unwind roots. The backend then stages a managed temporary reference cell
instead of recovering the cleared local address. Existing actual reference places
keep their original handling. A new throwing-callee heap-debug regression verifies
caller writeback and retirement after exceptions. An all-target EIR/emission test
checks the detached Acquire, published/retired collector owner and managed cell on
macOS/iOS device/iOS Simulator/Linux ARM64/Linux x86_64; it passes **1/1** with exit 0,
log /tmp/elephc-reference-collector-unit-W6zNBz. This is emission, not native iOS/Linux.

The first expanded reference batch was **3 passed, 8 failed**, log
/tmp/elephc-reference-variadic-fix-ukAdLf. It removed the initial null-slot crash,
then exposed scalar-retyping corruption/leaks. Positional variadic preparation
was using the fixed-reference marker rather than the existing variadic Mixed-
widening marker. Reusing the correct marker advanced the batch to **10 passed,
1 failed**, log /tmp/elephc-reference-variadic-retype-gy4efR, with the new throwing
heap-debug case and the three reference GC controls green. The remaining failure
is the call_user_func segment of the mixed direct/descriptor regression.

PHP 8.5.10 CLI prints a by-reference warning and keeps the source scalar unchanged
for that call_user_func probe. Elephc's current docs/php/functions.md explicitly
promises source-variable mutation through call_user_func, and the existing test
expects it. This safety correction preserves that documented Elephc behavior;
the PHP semantic discrepancy is an explicit open parity/consensus finding, not
silently declared equivalent or resolved by changing the test expectation.

Known descriptor reference-variadic arguments now use the same widening marker
as direct positional/named tails, while fixed reference parameters retain their
existing guarded storage normalization. A separate heap-debug call_user_func
regression checks the documented retyping behavior and balanced owners. Its
expanded 12-control reference batch is running serially with durable log
/tmp/elephc-reference-variadic-final-5nNfoS, session 88575. No terminal result or
full reference-ABI acceptance is inferred yet. Original/follow-up merge, source
documentation/generation gates, fork publication and exact-artifact consensus
remain open; no history rewrite, push or vendor-state restoration occurred.

The latest reference batch completed with exit 101: **11 passed, 1 failed**,
log /tmp/elephc-reference-variadic-final-5nNfoS. Direct positional/named writeback,
methods/closures, callable retyping, repeated aliases, hidden counts, throwing
callee cleanup and the reference GC controls pass. The new isolated heap-debug
call_user_func retyping test also passes. The original combined regression still
prints a pointer-like integer for its call_user_func segment AFTER an earlier
call through the same callable; its other segments produce the expected values.
Do not report the whole reference batch as green or weaken that combined case.

The next investigation is callable metadata lifetime/unknown-signature argument
preparation. callable_descriptor_signature_for_expr consults callable parameter
signatures or flow-local static bindings; clear_static_callable_locals clears the
latter at argument-evaluation joins. The signature-driven widening marker applies
only when the caller retains a known variadic signature. This explains a possible
route difference between isolated and combined calls, but requires focused EIR/
caller evidence before attributing the remaining failure or retaining stale facts.
No blanket bypass of callable invalidation is authorized by the passing isolated test.

Kioku recorded the PHP/Elephc call_user_func compatibility finding as
gotcha-call-user-func-source-variable-writeback-is-docu-792ec083, with successful
capture log /tmp/elephc-kioku-cuf-parity-KZbiNg. The reference batches and all-target
emission test above are terminal, and whole-tree diff hygiene passes. Their native
signal/ownership findings are distinct from the open PHP semantic decision.

Kioku acknowledged bug_fix-reference-variadic-collectors-detach-evaluation--f396e27b
and the explicit superseding edge from the original null-slot crash packet. Its
fact preserves the current **11/12** result and combined-call failure rather than
claiming reference closure. Capture log:
/tmp/elephc-kioku-reference-owner-fix-bYNbC3. Both the remaining dynamic-signature
case and the documented PHP semantic divergence stay open for later reconciliation.

### Context-dependent reference marker authorization

Source inspection separates two facts: callable identity is flow-sensitive, while
the checker records per-argument boxed-reference storage authorization by scope
and source span. Both record_boxed_reference_output and unresolved-callee alias
recording contribute those sites. The runtime descriptor adapter already supports
Mixed-backed caller storage and addresses its concrete payload for typed refs.
Argument CFG joins may discard static callable identity; retaining a stale target
or return signature is not required to honor the checked storage authorization.

When descriptor argument lowering has no signature but the exact variable/span
has checker authorization, it now prepares the widening reference marker rather
than passing the old scalar slot unchanged. Known fixed reference handling and
runtime target selection remain intact. The context-dependent reference batch is
now **12/12**, exit 0, log /tmp/elephc-context-reference-fix-p8Pphu.

The strengthened heap-debug regression performs an earlier callable invocation
before call_user_func and checks cleanup. A fixed int-reference control confirms
its scalar ABI/writeback still yields 1:6. The next pipeline completed with exit 0:
fixed reference **1/1**, expanded reference family **12/12**, descriptor binding
**17/17**, then whole-tree diff hygiene. Log:
/tmp/elephc-authorized-reference-gates-YOQukf. These focused macOS results do not
close complete array/I/O/target or reviewer acceptance.

A further heap-debug control reassigns a previously invoked reference callable
to a runtime-selected by-value identity target; it requires result/source 5|5 so
storage authorization cannot preserve stale callable identity. That control and
the argument-evaluation-owner library family are being verified serially, log
/tmp/elephc-reference-identity-gates-x7HuAP, session 73843. No result from this
last pipeline is inferred yet. The explicit PHP/Elephc call_user_func behavior
discrepancy remains open, and no merge commit/push/vendor restoration occurred.

The runtime-target-selection heap-debug control passes **1/1**. The following
argument-evaluation-owner library family is **12 passed, 1 failed**, exit 101.
Its failure is not a reference retyping output: assembly emission rejects
`backtrace hidden argument collector PHP type AssocArray { key: Mixed, value: Mixed }`
in mixed_named_string_and_callable_coercions_do_not_double_retire_evaluation_owners
(argument_evaluation_owners.rs:586). The full log is
/tmp/elephc-reference-identity-gates-x7HuAP. This newly exposed collector/backtrace
transport gate must be handled shape-aware; do not drop keys or narrow the
signature merely to make the emitter accept it. The reference handoff test inside
this family remains green on its supported-target emission loop.

The authorization fix itself is current-source validated: the combined reference
case is green, the fixed-scalar reference/heap control is green, all 12 reference
controls and all 17 descriptor-binding controls pass. Callable selection remains
runtime-driven in the retargeting control; no invalidated target identity was
restored. These are bounded checkpoints, not whole array/target/review closure.

Kioku acknowledged bug_fix-descriptor-reference-storage-authorization-survi-711c76a5,
capture log /tmp/elephc-kioku-authorization-fix-i5O7sv. The fact separates storage
authorization from callable identity, cites the passing reference/retargeting
controls and explicitly retains the failing backtrace collector family.

The exact unsupported collector diagnostic is emitted by both target paths in
src/codegen/lower_inst/core_builtins/backtrace.rs (matches around 526-534 and
920-928), currently accepting only PhpType::Array. Inspect their frame metadata
and corresponding runtime readers before widening collector support; both
AArch64 and x86_64 must retain the actual shape and argument/key semantics. The
supplementary literal graph lookup is still pending (session 40680), and is not
needed to treat these direct diagnostic matches as current source evidence.
Whole-tree diff hygiene passes and conflict stages remain empty, but HEAD and
MERGE_HEAD are unchanged and the original/follow-up merges remain uncommitted.

### Backtrace positional collector representation

The AST func_args pass guarantees its HIDDEN_ARGS_PARAM snapshot is positional-
only: source variadic entry values are copied through an is_int(key) filter,
and a synthetic surplus collector does not accept unknown public named arguments.
This internal snapshot is distinct from the user variadic's original numeric/
string-key storage. PHP 8.5.10 CLI confirms debug_backtrace args preserve named
variadic entries, while func_get_args excludes them; mutations of the ordinary
variadic variable do not rewrite those entry-time backtrace values.

The hidden snapshot reader now borrows indexed storage or obtains an owned
insertion-ordered values copy for a Hash snapshot using the existing typed
array_values backend emitter. Both debug_backtrace materialization and printed
argument ranges retire that temporary copy, while the original frame collector
remains borrowed and unchanged. Optional hidden count extraction also reads the
normalized positional view, accounting for its extra owner stack slot before
retirement. This is an internal positional snapshot copy, not a blanket user
Hash-to-list transport workaround or named-key preservation claim.

The focused serial pipeline completed exit 0: argument-evaluation-owner library
**13/13**, including its supported-target emission loops; native optional/variadic
backtrace **1/1**; native printed optional/variadic backtrace **1/1**; diff hygiene.
Log /tmp/elephc-trace-collector-gates-GFL8yu. Assembly-comment alignment also passes.
The shared values emitter's visibility was widened only inside lower_inst to reuse
the existing allocation/retention implementation; no PHP builtin contract changed.

New required heap-debug regression:
test_core_backtrace_regression_named_tail_keeps_entry_snapshot_and_keys.
Its PHP CLI oracle is
{"0":11,"1":12,"named":22}|[11,12]|named|{"named":22}|[]|named|.
Current native source fails **0/1** with
[11,12]|[11,12]|bad|[]|[]|bad|, but heap cleanup is clean. The full evidence log is
/tmp/elephc-named-trace-snapshot-n3qfaN. Neither trace materialization nor printing
currently receives the named tail's entry-time snapshot. This test must stay
enabled and must pass before full backtrace/array acceptance: do not infer closure
from the repaired positional reader or read the already-mutated user variadic.

Next work must capture the original named variadic entries separately from the
positional func_* snapshot, preserve fixed/default argument-count rules and keys,
and make both frame materialization and printing consume that view with balanced
ownership on every supported target. Optional non-variadic hidden-count controls
are additionally running from the current codegen artifact; outcomes are pending.

The hidden-count follow-up completed exit 0: call_user_func-only frame-capture
gate **1/1** and eval-only optional frame-capture gate **1/1**, log
/tmp/elephc-trace-count-prefix-zoqbQ8. The latter rebuilt its required bridge
artifacts serially before native execution; it is not a whole eval parity claim.
Kioku acknowledged the missing named-entry snapshot as
gotcha-backtrace-named-variadic-arguments-require-entry-06389138, with successful
capture log /tmp/elephc-kioku-named-trace-tJDCnP. All build/test processes for the
positional/backtrace and named snapshot probes above are terminal.

Current local origin/main resolves to 91136c41cebbf1f5a0166e1bec7b7425895ae1bc,
which is newer than the previously inspected 4f79c92c9e follow-up. MERGE_HEAD
remains afe32d3efe and HEAD remains 516dea0b36. Refresh/inspect the follow-up
against its exact parent before merging it; the earlier seven-path inventory
does not describe every possible newer main change. No source stage, merge
commit, fork push or inherited vendor restoration is inferred from these tests.

### Named backtrace entry snapshot: terminal-cursor correction (2026-10-10)

The diagnostic run /tmp/elephc-named-snapshot-diagnostic-PW8KkP is terminal
(0/1, exit 101). It confirms that captureNamedTrace has an allocated snapshot
local with AssocArray<Mixed, Mixed> storage in both frame readers; the failure
is not evidence that the local itself was absent. Its native output still
omitted names and heap cleanup was clean. Compiler stderr instrumentation used
for that check has now been removed from named_arguments.rs.

The runtime hash iterator documents and emits two distinct negative cursors:
-2 accompanies the final valid entry, whereas -1 means no entry was yielded.
The new named reader incorrectly stopped on any negative result, skipping the
tail entry (including the only entry in the named-only fixture). It now stops
only on -1. Runtime hash iteration semantics are unchanged.

The structural snapshot regression now requires a StoreLocal initializer, not
only a local declaration and emitted reader label. Additional native heap-debug
regressions cover typed named strings after user-local mutation and named
reference updates whose returned trace cells must not alias caller storage.
The PHP CLI oracle for those two fixtures is respectively
{"first":"one","last":"two"} and {"named":42}|42|42.
Post-correction native and structural results are pending; no acceptance or
publication gate is closed by the source diagnosis alone.

The initialized-snapshot structural check completed 1/1 (five supported-target
lowering/emission loops), log
/tmp/elephc-named-snapshot-initialization-20261010.log. The native cursor-fix run
completed 1/3, log /tmp/elephc-named-trace-cursor-fix-20261010.log. The original
mixed positional/named and named-only fixture now matches PHP exactly, including
printed names, with clean heap-debug teardown.

Both supplementary fixtures stopped before runtime: declaring their return as
array while returning $frame["args"] produces "return type expects array, got
mixed". This is a separate required PHP-array return-boundary finding, not a
named-reader failure. Their escape/ownership controls now explicitly return
mixed so the string/reference reader paths can execute; this does not establish
typed-array return compatibility. Preserve the stronger reproducer by changing
the return annotations of namedValues and namedReferences back to array. The
return boundary remains open in the full PHP-array campaign.

The mixed-return ownership controls completed 2/3, log
/tmp/elephc-named-trace-ownership-controls-20261010.log. Typed strings preserve
both original values and final-entry order with clean heap teardown. The named
reference fixture reaches native execution but reports a bad refcount before
caller output. Keep that regression enabled.

The retained native reproducer is
/tmp/elephc-named-reference-debug.wGXVh1/main.php (compiled through the current
CLI with --heap-debug --keep-symbols). Its executable SHA-256 is
bdc66244974a500ec1b9a77b8e472b98759504471be2e15fed60c6b4d6ab7d92;
assembly SHA-256 is
5f9969a251d8d65362fe8df0bbe2c14afb2dc759a57084dc1cddf7290a3a99f1.
LLDB places the failing decref in main's call-owner cleanup, after the callee
returned normally, not after an initial PHP exception. Conditional hash-release
breakpoints show the callee's named-snapshot cleanup retires the last original
hash owner before the caller's owner guard tries to retire it again.

The shared managed-cell initializer retained only identical source/target PHP
representations. For a Hash-backed collector passed as Array(Mixed), coercion
keeps the raw pointer but that inequality omitted the cell's independent payload
retain. The initializer now retains this existing raw Hash/Iterable collector
borrow as well. It does not cast the hash to an indexed list, discard keys, or
change declared element validation. Validation is running in
/tmp/elephc-named-reference-cell-owner-fix-20261010.log; no pass is inferred yet.

That owner-fix run completed 2/3: the reference test now has clean heap teardown
(52 allocations/frees), but outputs {"named":null}|42|99 instead of PHP's
{"named":42}|42|42. Memory safety improvement is not semantic acceptance.
Private invoker marker tag 11 is not an ordinary persistent PHP reference:
mixed_reference.rs uses nested Mixed tag 7 and high word 1 for that protocol.
The plain mixed clone therefore leaves an invoker marker unresolved.

The reference-value materializer used by descriptor binding is now extracted
without changing its instruction sequence into
codegen_support/callable_invoker_args/reference_values.rs. Its old descriptor
wrapper delegates with the same label allocator. The named snapshot reader
recognizes the private marker after unboxing, uses this same source-address/tag
decoder, then clones the resolved value into independent trace storage and
retires both temporary owners. The native test keeps its PHP oracle unchanged.
Validation is pending in /tmp/elephc-named-reference-value-fix-20261010.log;
the descriptor unit suite and five-target snapshot checks must be replayed too.

Kioku operational feedback: recall and targeted scan/api remain usable, but
callers emit_ref_arg_cell_block --context 2 spent over five minutes building
its shell argument list before starting its search. lsof identified its cand
file (1,825,808 bytes) as stdin and its rawhits output still empty; this merge
checkout has 23,353 modified/untracked paths. cmd_callers appends those paths
and repeatedly executes set -- "$@" "$gr_f", making dirty-checkout argument
construction expensive. Only this agent's nonessential query process (78369)
was explicitly cancelled; no build or external process was stopped. Graft's
subsequent graph query completed, while source readback verified the current
coercion and ownership code. Do not treat the cancelled query as no references.

The decoder-fix native run completed exit 0, 3/3, with clean heap-debug teardown,
log /tmp/elephc-named-reference-value-fix-20261010.log. Both named-reference
expectations now match PHP: the trace observes the callee's value 42, and later
trace mutation leaves the caller at 42. The initial printed/mixed-key snapshot
and typed string controls also remain green. The broader backtrace family is
running separately in /tmp/elephc-core-backtrace-family-20261010.log; no result
for that family or the shared descriptor unit replay is inferred yet.

Debugger artifact provenance correction: the first CLI build was subsequently
recompiled to inspect the owner-only fix. The earlier bdc66244/5f9969a2 hashes
above describe that historical pre-owner-fix build, not current main/main.s.
The heap-clean but semantically failing owner-only executable and assembly are
now preserved as main.owner-fix and main.owner-fix.s in the same temporary
directory, with SHA-256 respectively
c69b20be7ecc236452396c9c32de3150e505d58259023e3d58c34f3b38ebe553 and
6d72cf3f8264cabaa41e6f184e4fcef190936c94b77e926c7be2a68ae5dc7972.

The broader Core backtrace regression family completed exit 0, 15/15, including
the eval/native boundary, nested eval, optional/variadic count controls,
call_user_func/array and literal/first-class/variable-callable capture gates.
Log /tmp/elephc-core-backtrace-family-20261010.log. This is focused native macOS
evidence, not complete PHP-array, supported-target or final review acceptance.

Kioku replacement bug_fix-named-backtrace-snapshots-decode-reference-marke-13a10ec4
now supersedes both gotcha-backtrace-snapshots-must-resolve-private-invoker-cf2f2ae1
and gotcha-backtrace-named-variadic-arguments-require-entry-06389138, preserving
the retired findings and their reasons rather than silently forgetting them.
Neighbor reference/descriptor native controls and the shared decoder/snapshot
unit replay are queued serially; do not infer results from the 15-test family.

The current native by-reference variadic neighbors completed exit 0, 12/12,
including throwing writeback, retyping, named tails, closure/method forwarding,
clean heap, repeated-call growth and replaced-box cleanup controls. Log
/tmp/elephc-named-reference-neighbors-20261010.log. The descriptor binding
neighbors are executing next in /tmp/elephc-named-descriptor-neighbors-20261010.log.
The same fail-fast sequential queue (exec session 58230) then runs the shared
descriptor unit suite and initialized snapshot checks before/after EIR
optimization, in /tmp/elephc-shared-marker-decoder-unit-20261010.log and
/tmp/elephc-named-snapshot-optimized-unit-20261010.log. Read their terminal
footers before promoting any pending counts. Do not restart a live queue.

Final source diff hygiene and assembly comment alignment pass for the new
reference decoder, named reader and managed-cell initializer. HEAD remains
516dea0b36 and the original MERGE_HEAD remains afe32d3efe; no merge commit,
follow-up main merge, fork push or vendor stash restoration occurred here.

### Shared-decoder validation and function documentation (2026-10-10)

The previous sequential validation queue completed exit 0: descriptor binding
neighbors 17/17, shared runtime_callable_invoker module controls 28/28, and
argument-evaluation ownership controls 14/14. The last group checks the named
snapshot initializer before and after EIR optimization and emits every supported
target. Logs are /tmp/elephc-named-descriptor-neighbors-20261010.log,
/tmp/elephc-shared-marker-decoder-unit-20261010.log and
/tmp/elephc-named-snapshot-optimized-unit-20261010.log. The 28-module filter is
not the earlier broader 40-invoker filter; keep their coverage descriptions distinct.

A fresh contribution inventory relative to the original afe32d3efe integration
parent found 395 lexical missing-function-doc candidates. Each candidate's
current source was inspected before adding responsibility-specific Rustdoc:
prelude builders and parse-parity helpers, worker-safety/effect analysis,
runtime guards and teardown/registry locks, instrumentation state accounting,
transfer diagnostics, Mixed value-copy/string boundaries, Async behavior and
heap-debug ownership fixtures. Only comments changed in this documentation pass;
signatures, attributes, bodies, assembly instructions and test expectations remain.
The old legacy itoa diagnostic now explicitly describes its fixed local artifact
directory and the absence of native execution evidence.

The same lexical inventory now reports zero candidates. Its audit source and
initial/final inventories are /tmp/elephc-function-doc-candidates-20261010.awk,
/tmp/elephc-function-doc-candidates-20261010.txt and
/tmp/elephc-function-doc-candidates-20261010-final.txt. This is a source-confirmed
lexical checklist, not a Rust AST proof or whole-repository Rustdoc acceptance.
Focused checks/build, canonical docs freshness and merge publication still need
their own terminal evidence. The full scheduler/array/I/O/review goal stays open.

The broader backend comment audit covered 238 existing contribution files and
found 534 issues: 520 existing comment-column mismatches in 23 files and 14
missing comments in the descriptor suspension guard. Existing comments were
mechanically aligned without changing instruction strings or comment text; the
14 additions describe descriptor-kind/name comparison and mismatch cleanup.
A final rerun reports all comments aligned, log
/tmp/elephc-premerge-asm-audit-final-20261010.log. File inventory is
/tmp/elephc-premerge-asm-audit-files-20261010.txt. New named-reader/reference
decoder leaves were separately checked. Kioku recorded bounded hygiene
measurement-main-integration-rustdoc-and-assembly-comment-in-6d428754.

Root cargo build -p elephc completed exit 0 without warnings, log
/tmp/elephc-premerge-build-20261010.log. Its queued Async/Parallel parse-parity
and prelude-detection library checks remain active in exec session 75149.
Subsequent assembly comment formatting changes only comments/spacing; canonical
exporter freshness and final compilation evidence still need their own run.

The latest explicit fetch now pins origin/main at
9df6d3bd7ad066484a9c21c4eb1f54dbdfa2fbf3. Beyond the earlier inspected monitor
source-name merge and traffic update, it adds PR #1808 canonical loop optimization
and associated CFG/range/liveness/LICM changes. The afe32..9df follow-up inventory
is 38 paths, 2553 insertions/475 deletions; it is not the earlier seven-file delta.
Finish the original ordinary merge without rewriting MERGE_HEAD, then inspect
and integrate this pinned follow-up through a second ordinary merge and rerun
the affected optimizer/ownership/scheduler gates. Neither merge is yet committed.

The default build/prelude validation queue completed exit 0: Async parse-parity
and injection 8/8, Parallel parse-parity/worker patches 8/8, and shared prelude
reference detection 39/39. Logs /tmp/elephc-premerge-async-prelude-20261010.log,
/tmp/elephc-premerge-parallel-prelude-20261010.log and
/tmp/elephc-premerge-prelude-detection-20261010.log. No full local suite was run.

The canonical builtin export/render/audit workflow is executing serially from
the reconciled source with the curl exporter feature enabled (exec session
21463). Its logs use /tmp/elephc-premerge-builtin-* plus premerge-php-comparison,
site-compat and eir-boundary. Generated pages are not manually rewritten.
Check the exporter and every subsequent terminal log before staging the original
integration merge. Docker guard shell syntax and mocked preflight controls pass;
the contribution function-doc inventory is still zero before publication.

GitHub readback confirms PR #958 is open/draft on Guikingone's
spike/runtime-ctx-register with exact head 516dea0b36250ec9cc83e09040cbe296a3defb57.
Both configured publication remotes and the exact inherited-vendor stash remain
unchanged; unmerged index entries remain zero. No other live collaboration agent
is editing this worktree. Untracked project additions intended for explicit staging
are the heartbeat discussion note, Docker guard test, named backtrace reader,
shared reference-value decoder and two runtime-context audit/state leaves. Local
graft graph cards are untracked too and must stay outside publication alongside
.kioku/.codex/.mcp.json/scratchpad; do not use git add -A.

The pinned loop follow-up's recognizer/CFG/driver and branch/range/LICM/liveness
diffs were inspected before integration. It restricts induction summaries to PHP
Int, retains checked overflow modes, avoids generator/handler loop rewrites,
propagates liveness deltas and limits standalone LICM materializations to blocks
dominating every latch. Its own plan explicitly leaves a pre-existing raw-slot
overflow difference between optimizer modes open; do not infer that our complete
array/ABI acceptance follows from upstream's optimizer measurements. The follow-up
also widens existing scoped Linux eval fixture budgets, not the general watchdog.
Its structural and executable tests still need replay on our integrated branch.

Canonical builtin workflow completed exit 0: curl-enabled exporter build,
1185 builtin contracts, 209 classes, 1117 constants, 2254 rendered pages, zero
builtin audit errors, 2276 site-compatible pages and zero enforced EIR structural
inventory errors. Logs /tmp/elephc-premerge-builtin-exporter-20261010.log,
premerge-builtin-render, builtin-modules, php-comparison, builtin-audit,
site-compat and eir-boundary. Generated changes versus the provisional index
remain 1041 paths, 1070 insertions/deletions, consistent with the earlier exporter
refresh; no generated page was hand-edited. Whole-tree diff hygiene passes.

The original merge can now be committed as an explicitly unaccepted WIP
integration checkpoint. Its publication does not close the follow-up 9df main
integration, typed-array return/consumer coverage, I/O provider or exact-artifact
review consensus. Preserve the reserved vendor stash until both merge commits
are finished and its inherited local state can be restored without publication.
