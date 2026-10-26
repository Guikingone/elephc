# SPL heap serialization family - pinned PHP 8.5.10

Normative php-src: 34308a6666b2d489c509541ea9befea9e2b42348, ext/spl/spl_heap.c.
Fresh M23 PHPT evidence: main-m23-heap-scopes/phpt-heaps and phpt-triage.json.
Ten failing serialization PHPT, plus an independent by-reference compare failure.
Do not change upstream tests or count filtered coverage as whole-corpus compliance.

## Required behavior

- Public __serialize(): array on SplHeap and SplPriorityQueue, inherited by concrete heaps.
  Validate corruption before write lock; locked message is Cannot serialize heap while it
  is being modified. Return numeric [properties, state], where state has ordered keys
  flags and heap_elements. Regular heap flags are 0; queue flags are extraction flags.
  Capture physical binary heap order, queue entries ordered data then priority.
- Properties must use existing serialization/hydration authority: preserve user properties,
  declaring-class private mangling and dynamic fields; never expose or hydrate native
  hidden storage, and never call a child override to implement native parent behavior.
- Public __unserialize(array $data): void requires exactly two outer entries with numeric
  indices 0 and 1. Load properties after validating [0], before validating state [1].
  Invalid data throws Exception: Invalid serialization data for <actual class> object.
- State requires integer flags. Regular heaps accept 0 only. Queue masks flags with 3,
  rejects zero; flags publish before heap_elements validation. Elements must be an array
  (keys ignored). Queue elements require exactly data and priority keys, accepting null
  payloads; validate key existence, not isset. Extra state keys are accepted.
- Restoration inserts onto existing heap, invoking the native comparator and maintaining
  physical order, rather than replacing raw backing storage or dispatching user insert().
  Partial writes and exceptions must match php-src; no transactional rollback assumption.
- Regular heap __unserialize validates write lock before malformed input; queue source has
  a different initial ordering. Preserve source differences and final consistency checks.

## Implementation boundaries and gates

Start with dedicated heaps/serialization.rs AST builders and actual public methods in
heaps.rs. Reuse the existing object property serialization/hydration contract and the
private native insertion body; inspect call graph before changing shared helpers.
Do not add fake PHP helper methods to bypass Reflection or capability checks.
Add focused normal, malformed, append-to-nonempty, private-property inheritance, queue
flag, throwing comparator, corrupted/locked and alias/reference regressions. Run unchanged
ten PHPT after focused tests. All supported target codegen and ownership paths must be
validated; by-reference compare and full 1710 corpus remain separate open gates.

## Native property-loader quirk found before implementation

The direct PHP 8.5.10 control property_type_direct.php stores string "bad" into an
existing public int property through inherited native __unserialize and reads it back
as string:bad. Zend/zend_API.c object_properties_load copies the incoming zval without
zend_verify_property_type, though it enforces initialized readonly properties. Therefore
blindly reusing the checked __elephc_object_restore_property writer is insufficient for
full native SPL parity. Preserve the typed property contract for ordinary PHP writes and
generic serialized-property decoding; do not weaken those paths globally. Native SPL
loading needs an explicit authority/representation design, or this remains an open gate.
Oracles are under .artifacts/integration/heap-serialization-oracles; property_type_error
was an initial false expectation (no exception), retained rather than reinterpreted.
append_without_insert_override.php emits a real PHP deprecation because insert returns
true, not bool; the corrected separate _true fixture is the normative silent control.

## Write-side implementation after accepted M25

M26 adds __serialize AST bodies and four clean PHP controls (formats, properties,
write lock, corruption). Its Rust build passes, but the public compiler smoke fails:
serialize still emits ordinary user properties and direct hooks call address zero.
These are not accepted results. The methods were absent from the EIR supported builtin
SPL whitelist; generic serialization reachability already scans magic hooks. M27 adds
the two whitelist entries and a fifth literal ReflectionMethod regression. The original
foreach/dynamic ReflectionMethod constructor fixture is retained as a separate existing
unsupported-lookup gate. No native property loader or __unserialize method was added.
M27 build/verification is pending; M25 remains the accepted checkpoint.

M27 production smoke passes the four serializer controls and the example in both IR
modes. Fresh unchanged PHPT: 27/36 pass, five mismatches and four compile errors;
SplHeap_serialize_corrupted and SplHeap_serialize_indexed_format are newly passing.
The Rust lot is 40 pass / one fail: full Reflection getReturnType returns array instead
of PHP's null tentative type and leaks four strings. The same full fixture with count
reproduces wrong int output and four leaked strings identically on M25 and M27, so it
predates the writer. Artifacts: reflection-heap-metadata-baseline/ and M27/smoke.
This is retained as an explicit existing campaign gate, not a passed metadata check.
The dynamic ReflectionMethod constructor probe is also still unsupported.
M28 changes only the new regression to the confirmed clean arity/visibility control;
it does not claim full Reflection parity. Writer production sources stay identical
to M27. Pending: source/CLI identity verification, all 41 native controls, four writer
cases across every target, then an accepted checkpoint.

## Accepted write-side checkpoint M28

.artifacts/integration/main-m28-heap-serialize-write/checkpoint.json records acceptance
for __serialize writing only. All 41 focused native Rust regressions pass with source
guards true; all four writer oracles pass both optimizer modes on macOS ARM64 and both
Linux architectures (24 exact/clean executions), plus 16 iOS device/Simulator assembly
cases. M27 and M28 CLI, rlib and verified default Magician archive hashes are identical;
M28 changes only the new Reflection regression to arity/visibility. Therefore M27's
40 target cases, 27/36 unchanged PHPT replay and both example executions belong to the
exact accepted artifacts. Full Reflection output/leak failures are preserved, not counted
as passes. M26 and M27 partial/failed Rust lots remain historical evidence.

Remaining selected cases: eight loading protocol failures and one by-reference compare
compile error. That upstream compare explicitly declares &: int while parameters are
untyped; do not misclassify it as inherited tentative return inference. Native property
loading's type bypass and readonly Error chaining need their explicit authority design.
The 1674 unselected tests, Reflection/eval/main reconciliation/CI and full compliance
remain open. Trail check is still truncated to 400 KB/500 files with twenty lexical
alerts in historical plans only; no full branch approval, commit or push.

## Native loading storage foundation in progress after M28

The previous goal turn made verified progress (M25 root retirement and M28 writer),
not a completed campaign. Current source still matches M28 at the start of this goal
continuation; origin/main has since advanced to 113 commits ahead. Preserve all dirty
changes and worktrees; no publication/rebase is being performed during this slice.

A new per-physical-slot native_property_contracts map separates original PHP declaration
contracts from boxed user-property storage on native heap descendants. NUL backing fields
keep their ABI. Ordinary assignment checking/weak coercion, strict hydration masks,
Reflection and eval registration must keep the original contract; getters and GC use the
actual boxed value. Reference fields retain their existing shared typed-cell ABI for now;
native replacement must detach the binding with an explicit per-instance representation
path. This remains a required gate, not a declared unsupported final feature.

Two initial cargo checks rejected a too-broad initializer insertion into an enum helper
signature; that source error is corrected. The third focused library cargo check passes.
No behavioral acceptance yet. Fresh PHP controls and pinned M28 comparisons are under
native-heap-property-oracles and native-heap-property-baseline-m28. Five controls already
match PHP exactly/clean on M28 (scalar coercions, arrays/objects, private shadowing,
promoted references, Stringable). Three additional controls expose pre-existing M28
gates (dynamic false property compile, assignment expression result, dynamic readonly
error order). Preserve their full outputs and verify current main before generic repairs.

M29 storage foundation builds successfully with guarded sources and the verified default
Magician archive. Public-pipeline controls: scalar coercion, rejected values/literal false,
private shadowing, promoted reference aliases and Stringable all match PHP exactly with
clean heaps. The previously unsupported false default is now represented correctly.
One newly introduced gate remains: array_and_object fails compilation; inspect its exact
compile.stderr under main-m29-native-heap-property-storage/smoke before further edits.
The old assignment-expression result and dynamic readonly order mismatches remain
unchanged against M28. No foundation checkpoint is accepted yet.

Three additional clean pinned PHP native-load oracles confirm required semantics: property
destructor sees old|new, array-property destructor sees array|new, and reference detachment
leaves integer:1|string:bad. These are under heap-serialization-oracles. Current ordinary
property getters retain the Mixed box through materialize_loaded_property_result; native
release-before-publication needs a separate borrowed-view mechanism, not a PHP payload pin.
Do not substitute an early ordinary assignment, which changes destructor-observable state.
