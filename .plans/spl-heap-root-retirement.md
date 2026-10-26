# Ownership-preserving root retirement for SPL next

## Verified prerequisite

.artifacts/integration/borrowed-cell-probe is an artifact-only C/AOT experiment. A raw 24-byte borrowed copy (tag/lo/hi), cloned with existing __rt_mixed_clone during a destructor, gives an independent owned value and supports resurrection. After creating another object, saved id remains 17 and __rt_heap_kind confirms the saved payload is live kind 4; heap allocation/free count is balanced. Complete current runtime and user assembly are linked without heap stubs. This is not yet a production feature. Source helpers use C only for the experiment; no C library is to be shipped for the solution.

## Candidate implementation to verify

- Add typed internal runtime primitives to borrow-snapshot data+priority (48 raw bytes), clone one saved value into an owned Mixed cell, and free the scratch. Buffer owns no PHP payload; creating it must not retain the retiring element. A compiler-private opaque handle or pointer lives only across one mutation.
- Source next retains the array_shift return(s), snapshots raw data, publishes a retiring view, then releases data before priority while count is still old and the write lock is set. Capture destructor errors, finish native bookkeeping and mark corruption before rethrowing. Extract must retain its output and therefore must not take this discard path.
- Public current/top/debug reads during destruction must still see the original root and original logical layout. Physical arrays may be compacted by shift; an explicit phase and physical-length mapping are needed until a null root hole is restored. Scalars must remain visible as the old root during comparisons, not be replaced by a convenient null. Clear the old-root view when the first actual root write publishes its replacement. Read borrowed values only while valid; PHP raw freed-object behavior must not become a claimed safety proof.
- Free scratch and clear the retiring phase through a finally path, including destructor and comparison throws, with no escaping raw pointer. Hold the local scratch handle independently from mutable object fields.
- Cloning from inside a destructor or comparison must materialize the logical view into owned storage before any user __clone hook. Default COW copying of a scratch pointer would escape its lifetime. Use existing clone metadata/runtime mechanisms for a native pre-hook, preserve native lock/corruption flags, and reset transient phase/error state on the clone.
- Cover root identity inside __destruct, event ordering, weak references/resurrection, multi-entry and single-entry heaps, queue data/priority destructor order, pending exceptions and previous chaining, snapshots/clones during callbacks, GC, both optimizer modes and all supported targets.

No representation choice is accepted yet. A native inline heap buffer with GC tracing is an alternative if the scoped borrowed snapshot cannot preserve all required behavior. Do not alter normative PHPT matching or mark the heap surface complete until the next regression and these lifetime gates pass.

## Scoped primitive checkpoint, 2026-10-20

The native borrowed-value primitive is implemented independently of the heap mutation. A scratch allocation contains 32 bytes per captured value (original concrete cell plus logical tag/lo/hi), has raw heap kind 0, and acquires no PHP payload owner. Read acquires a real owner, retaining the canonical original cell for resources; free retires scratch only. Three typed RuntimeFnId targets lower through one target-aware backend group. There is no new builtin contract or PHP method surface in the final candidate.

M20 is an intermediate, preserved artifact: its private method wrapper prototype is REJECTED. ReflectionMethod('SplHeap', '__elephcHeapSnapshot') printed visible with a clean heap, demonstrating that private is insufficient to hide raw integer handles. The declarations, SPL whitelist additions and wrapper lowering were removed from source. The rejected wrapper implementation is retained only under .artifacts/integration/main-m20-borrow-primitives/reserved-method-candidate-rejected.rs. Future binding must keep scratch inside an instance/lifetime boundary and preserve PHP-visible metadata.

Primitive evidence before the final source build: main-m20-borrow-primitives/targets-results-3.json has 12 actual native controls on macOS ARM64 and Linux ARM64/x86_64 (two fixtures times two optimizer modes) and eight iOS device/simulator user/runtime assembly controls. All executed cases have exact expected output and clean heap summaries. C checks real runtime ownership, including resource cell identity and resurrection after a later object allocation. The native helper set is complete; there are no ownership or heap allocator stubs. The final regression fixture additionally checks container child refcounts at capture and PHP method invisibility. Final Rust validation and the M21 source guard are recorded separately; do not substitute the earlier results for those gates.

The C/PHP test boundary must use an acquired return: a raw call to a PHP global getter may return a borrow via its hidden ownership convention. The fixture returns array_pop from a fresh local array before C releases it. Its high-argc reachability branch consumes return values with var_dump so function bodies survive source optimization. iOS CLI emission uses --emit staticlib.

Open next gate: attach this primitive to next while preserving the old count and write lock during root destruction, data-before-priority release, pending exceptions, logical root views and clone materialization. Reads require a still-valid captured payload; primitive success does not prove arbitrary reads after destruction. No next body was changed at this checkpoint. M19's destructor-order failure remains a failure. Full 1710-PHPT compliance has not been replayed or achieved.

Additional generic gate discovered by the PHP-visibility regression: method_exists($class, '__elephcHeapSnapshot') inside a foreach over string class names fails at EIR codegen with "builtin requires a literal string name". Evidence: .artifacts/integration/borrow-primitives-validation-3/native.log. The focused invisibility check now uses nine literal class/method pairs; the calculated-class-name failure remains a separate, unresolved compiler gate and must be checked on clean main before changing generic lookup behavior. No normative PHPT was altered.

The calculated-class-name method_exists gate is reproduced on the preserved clean-main base-0 CLI, SHA-256 ffddabddb3570fdcc9a57b71bb6d3d3bd8d3dc1b2590f7987dc97b64e373fb6a. Pinned PHP emits 111 for foreach over the three SPL class names with method 'count'; base-0 EIR emission exits 1 with the same literal-name diagnostic. Evidence: .artifacts/integration/borrow-primitives-validation-4/dynamic-method-exists/result.json. It predates these primitive changes and needs a dedicated main-baseline fix. The corrected primitive regressions now pass: one unit test covers 15 typed target lowerings and three integration tests execute both optimizer modes, including the nine literal PHP-visibility checks. Source guards passed for both test commands.

Architecture correction for the next candidate (M22): the full builtin/EIR boundary audit correctly rejected the three RuntimeFnId prototype variants because they had no builtin semantic home. They were removed completely; src/ir/runtime_fn.rs and the builtin runtime-function dispatcher are restored to their prior bytes. The operations now use dedicated RuntimeCallTarget variants with fixed storage signatures, lowered by src/codegen/lower_inst/runtime_calls/borrowed_value.rs. No placeholder PHP builtin was added. The M21 build was explicitly interrupted after that audit failure and is not an accepted checkpoint. M22 source validation supersedes the earlier typed-ID prototype results.

M22 final evidence is accepted for the internal primitives only: .artifacts/integration/main-m22-borrow-primitives/checkpoint.json. Architecture audit 0, four focused Rust tests, 15 typed target lowerings, 12 exact/clean native controls and eight iOS compile-only cases; all guarded source bytes are unchanged. The CLI/archive hashes and open gates are in that checkpoint. The full campaign remains active and the next destructor-order failure is unchanged.

## Receiver-bound scope candidate after M22 (unaccepted WIP)

M22 remains the last accepted immutable checkpoint. The current source adds typed compiler calls carried by a private Name annotation; their diagnostic text does not grant a PHP capability. A native roundtrip through the real parser/rewrite/check/EIR pipeline passes with both optimizer modes, including owned object/resource arguments and clean heaps. Test hook: tests/codegen/support/ast_transform.rs; regression: runtime_gc/compiler_calls.rs. No fake PHP method or builtin was registered.

Native heap scopes now use an object-keyed linked list in heap_scopes/. They borrow destructor-bearing values, stabilize only inert strings, invalidate freed payloads before allocator reuse, support out-of-order scope completion, and materialize logical clones before user hooks through an optional callback. The canonical SPL property prefix is shared under types/spl_heap_layout.rs. heap_scopes is runtime feature bit 16; plain-runtime heap_free keeps its old public API and omits the observer. Feature propagation and cache-key coverage were updated. Frame shutdown drains abandoned raw scopes after PHP cleanup.

The existing next_destructor regression passed once with early source-level root assignment. Broader focused run: 19 passed / 4 failed. Failures were callback_order, drop_throw and queue_two_errors (bad refcount), plus a static count() typing limitation for the global holding a clone. LLDB retained the callback binary and proved the failing incref targets an already-dying backing array; the destructor is invoked from array_free_deep during property writeback. The array had refs=3 entering scope_begin and refs=3 on return, so the two begin argument releases balance their two getter owners.

The current correction replaces source-level root assignment with HeapScopeReleaseRoot: ensure_unique moves the property owner, the new backing pointer and null element are published first, then the old Mixed element is released. This is native SPL container behavior, not a waiver of the generic property-writeback issue. Current verification is live in exec session 83011, cargo PID 74736 at inspection; do not restart it just because output is quiet. .artifacts/integration/heap-scopes-focused-2.log. Cargo.lock, Cargo.toml and build.rs still match M22. The target directory's dependency cache was rebuilt while free disk space increased to about 400 GiB; no cache was cleaned by this campaign.

The clone fixture now guards its global result with instanceof H; pinned PHP output is unchanged. Preserve the original non-narrowed fixture, which remains evidence of the existing typing gap rather than a passing case. Oracle inventory: heap-scopes-oracles/inventory.json, plus four extra lifetime oracles in lifetime-inventory.json. No upstream PHPT was modified.

Frozen clean-main base-0 checks (CLI ffddab...): self_temporary has exact clean behavior; array_throw and property_array_throw have exact stdout but leak respectively 40 and 72 bytes. Evidence: destructor-array-baseline/results.json. Verify latest main before implementing any dedicated generic fix. Do not confuse those baseline leaks with the new receiver scope's refcount failure.

Pending gates: finish the current seven-case run, debug remaining failures, add the four extra lifetime regressions, rerun all heap controls, validate the scope flag/cache propagation and non-forgeable names, pin current CLI/archive/source, execute focused Linux ARM64/x86_64 and iOS assembly checks, then replay relevant unchanged PHPT. Neither M22 nor this candidate proves full 1710-PHPT compliance.


### Resume: receiver-unset boundary (2026-10-20)

- `heap-scopes-focused-3.log` is terminal: 26 passed, one failed (27 selected).
  The new array/resource views and debug resurrection controls passed, alongside the seven
  previously green retirement cases. Each retirement fixture runs IR optimization on and off.
- The remaining `receiver_unset` case prints `data|heap:1|` and faults in
  `__rt_array_ensure_unique` while removeTop continues on the freed receiver.
  PHP 8.5.10 expects `data|heap:0|after`. This is not an accepted checkpoint.
- A separate branch/worktree `fix/main-borrowed-method-receiver` /
  `elephc-borrowed-method-receiver` was created from main `09c634061a` to verify an ordinary
  class reproducer before making a generic lifetime repair. No inherited worktree was reset.
- Candidate hygiene: ObjectCloneShallow now declares READS_GLOBAL for the optional logical-view
  clone hook. Web request reset clears the borrowed scope head and clone hook before callbacks
  and before the heap arena reset. Assembly comment checks pass for the adjusted scope helpers
  and static clone path. No final candidate build or cross-target acceptance yet.

- Generic receiver verification completed: main 09c634061a reproduces a poisoned receiver
  property read despite a clean heap summary. Six fixed cases pass in both IR modes in the
  separate worktree (`.artifacts/borrowed-receiver/fix-2.log`). The minimal helper/call-site
  changes and regression module are now integrated.
- First integration rebuild (`heap-scopes-focused-4.log`) did not run tests: Cargo reused the
  other checkout's builtin-contract archive and could not find `method_return_is_tentative`.
  This is a build provenance failure, not a test result. All current-checkout crate roots are
  being refreshed without content changes via build-refreshed-checkpoint.py, then pinned as
  main-m23-heap-scopes. No cache deletion and no edits during the guarded build.

### Fresh M23 validation and remaining heap families (2026-10-20)

- Guarded M23 build passed; 35 native Rust regressions passed, three library tests passed,
  builtin/EIR architecture audit passed. The cache-key test belongs to the CLI binary; its
  separate focused invocation passed (cache-key-m23.log).
- Fresh unchanged php-src regex `(?i)(heap|priorityqueue|bug69264)` selects 36 cases: 25 pass, six mismatches, five compile errors. bug69264 and
  heap_next_write_lock pass. This is a filtered run, complete=false/full_compliance=false.
  Ten failing cases form the missing heap serialization family; the remaining by-reference
  compare case rejects the declared reference-cell payload representation. No PHPT was altered.
- Cross-target custom retirement matrix: 22 exact/clean executions per macOS ARM64, Linux
  ARM64 and Linux x86_64; 24 assembly checks per iOS device/Simulator. The six initial
  clone links lacked regex/object-clone runtime families; they are failures, preserved.
- A corrected clone-only harness emits regex and object-clone runtime features and links
  the actual PCRE2 shim with host/container test providers. Static and boxed clones pass
  both optimizer modes on macOS and Linux ARM64, and assemble on both iOS targets.
  Linux x86_64 exposes a real new helper ABI bug: clone_array loads capacity into rax
  and stride into rdi, whereas __rt_array_new requires rdi/rsi. Both clone controls fail
  with heap memory exhausted. Fix and rerun before accepting the candidate.
- M24 is a guarded archive-refresh/comment-alignment checkpoint in flight; it precedes
  the x86 ABI fix and cannot close the candidate. M22 remains last accepted scope until
  the replacement checkpoint and complete focused validation are recorded.

### M24 superseded, M25 final focused verification

M24 builds and restores the verified default Magician archive, but is superseded because
its native heap clone-array helper uses the wrong x86 argument registers. M25 replaces
both array_new argument registers with abi::int_arg_reg_name, adds a boxed clone receiver
regression, and preserves all other scope semantics. M25's guarded build passes, with
CLI SHA-256 21d4d4ea9062dadb6fd67d730049aaad44f3fe15a84be7b32de8a6421bddd781,
rlib c1874b8ff6537b94da83c77fd86a49a8c8648de853c8e183809a683fe855fedd,
and verified default Magician ce982d79308bce60a16f8871c50a2da24e8b31c41e2fba898da2700253f008a2.
All 36 focused Rust native tests pass; hygiene covers 51 preambles, aligned assembly
comments and both staged/unstaged diff whitespace. Source guards pass for native tests.
The fresh M25 unchanged PHPT replay is exactly 25 pass, six mismatch, five compile errors
out of 36 selected; the 1674 unselected tests carry no fresh result.

The M25 Trail check is limited to 400 KB/500 files and exits 1: all twenty lexical
alerts refer to the two historical campaign checkpoint/diagnostics plans, which retain
failed experiments and oracle results as history. This does not reintroduce rejected
RuntimeFnId prototypes or resource snapshot ownership. There is no whole-branch Trail
approval; reconcile these records and branch overlap before publication. No commit or
push was made. The new learning records the target ABI fix and native hydration quirk.

Next family: .plans/spl-heap-serialization.md, backed by seven clean frozen PHP controls
under heap-serialization-oracles/accepted-inventory.json. Native property loading bypasses
ordinary property type checks but honors initialized readonly fields; preserve this fact
as an open representation/authority design gate rather than weakening generic hydration.

M25 accepted for the receiver-bound root-retirement slice only: checkpoint.json under
.artifacts/integration/main-m25-heap-clone-abi. All 130 cross-target cases pass: 78
executed exact/clean native cases (26 per macOS ARM64/Linux ARM64/Linux x86_64), and
52 iOS device/Simulator assembly-only cases. Both final verification commands exited 0
with source guards true. This supersedes M22 for this slice; PHP-src compliance remains
25/36 for the freshly selected heap family and full_compliance=false.
