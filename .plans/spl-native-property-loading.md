# Native SPL property loading - active work after M28

Goal remains full SPL/OPcache compliance, pinned PHP 8.5.10 and php-src
34308a6666b2d489c509541ea9befea9e2b42348. M28 remains last accepted checkpoint.

## Current authoritative work

- Added ClassInfo.native_property_contracts keyed by physical index, with Some(type)
  for an original typed contract and None for an untyped field. Non-reference heap user
  fields become Mixed storage; reserved NUL fields keep their native ABI. Reference
  fields retain their existing typed shared-cell representation. This is a prerequisite,
  not an implementation/acceptance of native reference detachment.
- ClassInfo helper methods expose declaration, write and value types separately.
  Checker native-slot reads stay Mixed; ordinary writes use the original contract.
  Metadata refinement runs before/after body checking and after final global inference.
  Native slots resist ordinary inferred-type narrowing.
- Backend weak property guard reads native declaration contracts and explicitly boxes
  coerced scalars when physical storage is Mixed. Literal false has its own accepted
  boolean-payload action. Reflection, eval declared-type registration, strict decoder
  masks/object checks and uninitialized diagnostic names retain original types.
- Four ClassInfo construction sites carry empty defaults; initial too-broad edits in an
  enum helper signature were corrected. Library cargo check attempt 3 passed.
- M29 built with guarded sources and default archive. CLI
  7441d30e1202f6d0d76d89c68bd80f35fe81bcfb1c4d3abbe8e25d9870f05237;
  rlib 36334f081f3cd286ff6baf40c034d153ae8d1a90030a277955f8903d6902c4b3;
  Magician ce982d79308bce60a16f8871c50a2da24e8b31c41e2fba898da2700253f008a2.
  It matches PHP exactly/clean for five scalar/private/reference/Stringable controls,
  including a newly compiling false-default property. Its array/container control
  fails compilation: Array push requires an array property, got mixed.
- M30 fixes that checker admission only for compiler-marked native boxed fields and
  lowers boxed container writes through existing PropGetForWrite + MixedArrayAppend /
  PHP-array indexed mutation paths. Added key/COW and nullable-array controls. Build
  is in flight at initial writing, exec session 10673, output build-m30.log. Revalidate
  the actual handle/process before deciding it stopped. Do not edit Rust during builds.

## Evidence and tests

Eight new runtime_gc/native_heap_property_storage.rs tests cover scalar coercion,
rejected values/literal false, array/object writes, private shadowing, promoted reference
aliases, Stringable, keyed COW writes and nullable arrays, both IR modes with clean heaps.
Run together with the previous 41 heap/compiler-call/borrowed-receiver tests: filters
spl_heap_, borrowed_method_receiver, test_compiler_runtime_calls, native_heap_property_storage.
No such Rust lot has run yet for this foundation.

Frozen controls: .artifacts/integration/native-heap-property-oracles; original inventory
plus extra-inventory.json. M28 baseline: native-heap-property-baseline-m28/results.json.
M28 already fails assignment-expression normalization (string:7 versus PHP integer:7),
dynamic readonly ordering (TypeError versus PHP readonly Error), and compiling a false
default. M29 fixes the false default but leaves the first two baseline failures unchanged.
Verify latest main before any generic fixes; do not overwrite the separate dirty receiver
main-baseline worktree's unaccepted Reflection/property experiments.

## Native loader must preserve these measured semantics

The existing plan spl-heap-serialization.md records full validation/insert ordering.
Three additional clean PHP controls under heap-serialization-oracles prove:
- property_destructor_view: old|new;
- array_property_destructor_view: array|new;
- property_reference_detach: integer:1|string:bad.
The normal property getter retains the stored Mixed box in
materialize_loaded_property_result, so a release-before-publish native loader needs
an explicit borrowed-view lifetime design. An ordinary early property assignment changes
the observed state; a payload pin delays destructors. Do not claim those as native parity.
Readonly initialization/error chaining, unknown/mangled/numeric keys, dynamic-property
diagnostics, partial writes, comparator exceptions and native reference detachment all
remain requirements. No __unserialize body or native property-load compiler call has been
added yet. Native-only capabilities must use private CompilerCall annotations and typed
RuntimeCallTarget entries, not PHP-visible wrappers or orphan RuntimeFnId variants.

## Remaining campaign gates

M28 fresh selection is 27/36: eight loading failures, one typed by-reference compare
representation failure. The compare explicitly declares &: int with untyped parameters.
Full Reflection tentative returns/temporary strings and dynamic ReflectionMethod class
lookup remain reproduced gates. 1674 tests remain unselected at M28; eval, supported
target campaign, main reconciliation (113 commits ahead at resume), exact candidate CI,
limited Trail check reconciliation and publication are all open. No commit/push occurred.

## M31 functional verification completed

M30's boxed array mutation fix passes the original array/object control plus keyed COW
and nullable-array controls through the public pipeline. M31 changes only tests from
M30; CLI, rlib and default Magician archive hashes are identical. It adds an exact-tag
write control, including successful literal false assignment and full PHP value types.
All 50 selected native Rust regressions pass with source guards true. All 90 property
matrix cases pass: nine controls in both IR modes across five targets, with 54 actual
exact/clean native executions and 36 iOS assembly-only cases. Fresh unchanged PHPT is
still 27/36, five mismatches/four compile errors.

Final hygiene found two pre-existing trailing assembly-comment columns in touched helper
files. Each was corrected by one space after code; original files/hashes are archived in
M31/style-followup and style-followup.json. No Rust tokens or emitted instruction text
changed. Current preambles, assembly comments and staged/unstaged diff checks pass.
M31's functional source guards were true before these exact two comment-only edits;
do not present its raw source manifest as identical to the current tree. Repin them with
the next semantic build rather than repeating the whole compiler for two comment spaces.

Native loading remains unimplemented, so this is a verified property-storage prerequisite,
not completion of the loader/campaign. The next action is its separate native property-load
operation and borrowed old-value view, followed by real __unserialize validation/insertion
and reference detachment. The complete goal remains active.

Release-order inspection found that mixed_free_deep invokes payload release before freeing
the Mixed cell itself; do not assume the box is already freed inside its payload destructor.
The cell's refcount is nevertheless being retired, and ordinary property reads incref that
cell, so the owned borrowed-view path still needs explicit modeling. Inspect array_free_deep
for the exact live-slot state before implementing array views; use measured PHP controls.

## Property-loop exception oracle

Fresh PHP controls property_throw_continues and property_throw_readonly_chain prove
that a destructor exception does not stop later ordinary property loads. A later
initialized readonly property stops the loop, and the final chain is outer invalid-data
Exception -> readonly Error -> original destructor Exception. A per-slot helper must
therefore capture destructor errors, publish the incoming value, and return the owned
pending Throwable to the generated loop rather than immediately aborting that loop.
A readonly error can stop immediately; chain its earlier pending Throwable before the
outer invalid-data wrapper. These facts are in the frozen JSON controls.

## M32 private replacement primitive in verification

Added CompilerCall/RuntimeCallTarget NativePropertyReplace(Object, I64, Mixed) -> Mixed
(pending Throwable or null), with fixed ABI, receiver native-prefix validation and a
literal physical slot restricted to the compiler-marked non-reference user map. No PHP
name/builtin/method is registered. The runtime uses a separate raw native-property
scope list and optional owned-read callback. Native getters now accept PropertySlot
metadata and read a logical snapshot instead of incref'ing the retiring Mixed box.
Allocator invalidation, final scope draining and web reset include the new records.

The replacement reserves a 128-byte frame, input slots 0/8/16, old cell 24, scope 32,
pending raw Throwable at sp+64 (fp-48 on both architectures), GC suppression at sp+72
and returned Mixed at sp+80. It captures the view, retains the incoming slot owner,
invokes guarded old cleanup, publishes the incoming pointer and clears the initialization
marker, unlinks the scope, boxes the pending exception/null and restores suppression.
The primitive is trusted low level: readonly/name/ref validation belongs to the upcoming
class-specific native loader. Reference storage is not supported by this primitive.

Build M32 passed with guarded sources/default archive: CLI
473f5931efb5b24867100b65250700f5b1426a8c19f5b0d70a82e8c5ff3eee8a,
rlib e3cff24c7ee51afdadc7fac3a3ac47c930c439d52cc5c960288e694717363025.
A first cargo check required the new textual as_eir target name; corrected. Five new
trusted-AST native tests use a real PHP __unserialize wrapper as their reference source.
All five frozen PHP controls pass: string:bad, old|new, drop|new, drop|alive|gone, array|new.
Their native execution is currently running in exec session 2820 via verify-m32.log;
per-target runtime assembly checks run in 98131. Revalidate handles before restarting.

Still required: verify/fix the five native cases, especially zero-ref array views; route
clone/Mixed projections through logical property views before any retiring-box retain;
implement full class-specific native property loading and public __unserialize state
validation/insertion; native reference detachment remains mandatory. No M32 acceptance
or full-compliance claim. Current PHP-src classification remains M31's 27/36.

M32 runtime assembly verification is terminal and successful on all five supported
targets; results are under M32/runtime-targets/results.json (assembly only). The
focused native test command is still confirmed live via exec session 2820; its log
currently says Compiling elephc. Poll that exact handle and inspect native-views.log
before changing Rust or restarting anything. No native-view result is established yet.

## M32 result and M33 array-view correction

M32 native lot is terminal: four pass / one fail, guarded source bytes unchanged.
Declared-type bypass, object destructor view, pending error publication and object
resurrection pass both IR modes. The array destructor view fails with bad refcount.
Logs: M32/native-views.log and verification.json. This is not accepted native parity.

M33 adds properties/arrays.rs. Before releasing the slot, a raw memo records each
reachable packed array once by source identity, including raw logical snapshots of
its elements. A shared memo prevents cycles and repeated shared subgraphs. Only inert
string stabilizers have PHP owners; temporary typed-element boxing is retired before
destruction starts. A source array with positive refcount is read normally; a zero-ref
array observed during a child destructor is materialized as a fresh Mixed-element array
from its snapshots. Memo invalidation marks freed child payloads null before address reuse.
The allocator observer now preserves the complete volatile integer register set used by
that flat traversal. Memo nodes and inert captures are dropped after scope unlinking.

Two additional clean PHP controls prove full values, not only is_array: array_contents
expects old:text:7:4|new; nested_array expects nested:text|new. They are native regressions
in native_property_views.rs. Library cargo check and assembly-comment checks pass.
A guarded M33 build is live at writing: exec session 99844, with output under
build-m33.log/build.json. Poll that actual handle before deciding it stopped.
After it completes, run all seven native_property_view tests with source guards, then
the full relevant regression lot and actual cross-target executions. Do not edit Rust
while the guarded build/test is live.

Still unimplemented/unverified: hash-container borrowed views, native property clone
interception before raw Mixed-box retain, readonly/name/dynamic-key loading validation,
public __unserialize and native insertion, native reference detachment, full corpus/CI.
The full goal remains active and no M33 acceptance is claimed.

A new pinned PHP associative-array destructor control hash_contents now requires
old:text:4|new with string keys and integer key 7. It is frozen under native-property-view-
oracles and is not a native pass. Trail located __rt_hash_iter_next_value: its borrowed
cursor interface returns key pair and value pair/tag, suitable for snapshot capture
without retaining PHP payloads. Preserve insertion order (head+24, entry successor+56).
M33 verification script is prepared as verify-native-property-views-m33.py.

M33 build is terminal and successful: CLI
ea3951b7c76a0d74e74dffeea3b14b33a45ac02ce82f3561efa1e2252d2d10a1,
rlib b65ebb12b6ed184da49e0ccaa73e57e3d5715349f4564da34c222532194909de,
default Magician ce982d79308bce60a16f8871c50a2da24e8b31c41e2fba898da2700253f008a2.
Guarded seven-test native verification is now live at exec session 49808, with output
verify-m33.log and M33/native-views.log. Do not restart the completed build handle 99844.

M33 runtime assembly checks are now terminal/successful on all five supported targets
(M33/runtime-targets/results.json); these are assembly-only, not execution. The seven
native-view tests remain confirmed live at exec 49808. Hash iterator discovery names
hash_entry_reference.rs::emit_iter_next_value for its borrowed cursor/key/value ABI;
layout evidence is saved in native-property-hash-iterator.txt.

M33 native verification is terminal/successful: seven tests pass, both IR modes with
clean heaps and source guards true. Test log M33/native-views.log; verification.json
records exit 0. The packed-array refcount failure is fixed for simple, full-content and
nested views. Five runtime assembly-only targets also pass. Next required evidence is
actual Linux execution through the trusted AST test bridge, not just assembly.

M34 adds a test-only ELEPHC_NATIVE_VIEW_ASM_DIR export branch to the oracle helper.
With it unset, the same native assertions run; with it set, the trusted AST is lowered
for ELEPHC_TEST_TARGET and user/runtime assembly plus source/expected output are saved
without claiming execution. Production CLI/rlib/default archive are identical to M33.
The verifier verify-native-view-targets-m34.py builds/pins the current integration test
binary, reruns the normal host native cases, exports each target in a fresh process
(target() is cached per process), then executes macOS/Linux binaries and assembles iOS.
This verifier is live at exec session 87886; preserve its source guard and inspect the
actual process/log before edits or restart. The complete goal remains active.

M34 target verifier is terminal/successful: all 70 cases pass with guarded source bytes
unchanged (42 exact/clean native executions on macOS/Linux ARM64/Linux x86_64; 28 iOS
assembly-only cases). The normal host native seven-case branch also passes. The combined
focused regression run is terminal/successful: 57 tests, zero failures, source guard true.
Fresh unchanged heap PHPT is running in exec 53299, preserving actual failure classification.
All M33/M34 production artifacts are identical; M34 changes test-only artifact export.

## M35 native property clone interception in progress

A fresh check at this turn confirmed the current Rust source exactly matched M34;
origin/main was still 149 commits ahead at that observation. The new changes intercept
Mixed non-reference fields before clone retains the copied physical cell. Static clone
collects Mixed offsets and calls the optional native owned-read callback, falling back
to ordinary retain when absent. Both boxed wrappers add a tag-7-only path that supplies
the original receiver/slot offset, installs the acquired logical value and skips the raw
retain. Reference-cell cloning keeps its existing branch/ABI.

Two actual PHP controls clone_static/clone_boxed (in native-property-view-oracles) both
expect old|new:object. They are now trusted-AST native tests, giving nine view tests.
The full static-clone caller graph was inspected before the multi-file change. Assembly
comments align and focused library cargo check passes. A guarded M35 build is live at
exec session 14496, with output under build-m35.log/build.json. Poll this handle before
deciding it stopped. Run verify-native-property-views-m35.py after it finishes,
then cross-target execution with the export verifier updated from 14 to 18 cases/target.
No M35 native or full-compliance result yet. Hash views and all public-loader/reference
gates remain open. No commit/push.

M35 guarded build is terminal/successful: CLI
88a1d71a522e5543862a1c8d97601a96fe22e4d6cb91d91a8ebdb52cd8d70436,
rlib be5d91528263b61d07ebe0f0e8e6d022c455c5de95ee40508910c13f47913deb,
verified default archive ce982d79308bce60a16f8871c50a2da24e8b31c41e2fba898da2700253f008a2.
The nine-view native verifier is now launched; inspect verify-m35.log and
M35/native-views.log before claiming execution results.

M35 native verifier is confirmed live at exec session 1416; do not edit Rust or restart
while it compiles/runs. Cross-target verifier verify-native-view-targets-m35.py is
prepared for nine controls / 18 cases per target; it validates the new guarded build
rather than incorrectly assuming identity with M33. Run after the current Cargo phase
ends. No native clone result yet.

M35 nine-view native verification is terminal/successful: 9/9 tests pass both IR modes
with clean heaps and source guard true, including static and boxed destructor-time clone.
The actual target execution verifier is now launched via verify-native-view-targets-m35.py;
inspect its process handle and log before editing Rust or claiming all-target execution.

The M35 target verifier is confirmed live at exec 73701; native execution already passes
all 54 cases on macOS/Linux ARM64/Linux x86_64, with 18 iOS device cases assembled.
Simulator assembly is completing. The earlier native Cargo phase record is archived
separately as M35/native-verification.json because the target verifier writes its final
summary to verification.json. No Rust edits occurred during either guarded phase.

M35 final focused lot is terminal/successful: 59 tests, source guard true; 54 native
executions and 36 iOS assembly cases pass. Fresh unchanged 36-case PHPT remains
27 pass / 4 compile errors / 5 mismatches (1674 unselected). checkpoint.json records
this bounded scope, not full compliance. No commit/push.

## M36 native hash property views

The raw container memo is now shared across packed arrays and hashes:
[next, source identity, count, tag] followed by 16-byte (key snapshot, value snapshot)
entries. Packed keys are null, hash keys own only inert scalar/string snapshots.
Hash values use the dereferencing borrowed iterator; this is a value-view protocol,
not proof of writable reference identity. Hash reconstruction consumes newly acquired
Mixed values through hash_set_value in insertion order. Invalidation/drop use the
same shared layout. Reference alias and cyclic acquired-graph reconstruction remain
explicitly open, as do name/readonly/native loading and the public __unserialize body.

Five PHP controls freeze hash contents, nested hash/array combinations, integer/string
key insertion order, and destructor-time clone. The unversioned hash_contents oracle
filename was refreshed with the M36 variant (same old:text:4|new output); use the M36
source manifest/test fixtures for precise new-case provenance. Guarded build is live
at exec 61335. Do not edit Rust while live. Next run native view verification, then
actual target executions using verify-native-view-targets-m36.py (28 cases per target).

M36 guarded production build is terminal/successful (source guards true, verified default
Magician archive). CLI bac6cc56c4b6e450dd4fc3e8648c69dc545ac0acac2657701cb6644f9f55760b;
rlib ec79172ead19919ccdf5cbdd2bf9835b809279d7cf59be20d9e2ac830758e3e4.
Native verification is live at exec 18407 (14 controls, each both optimizer modes).
Do not edit Rust while the Cargo verifier is live. Hygiene checks pass with required
module preambles and assembly comments at column 81 or one space after longer code.

M36 14/14 native views and the combined 64-test focused regression lot pass, each
with guarded source bytes. Fresh 36-case PHPT remains 27/4/5. Actual target verifier
is live at exec 92069; no Rust changes until its guard completes.

Additional native loader key controls are frozen at native-property-name-oracles-m36.
PHP 8.5.10 accepts empty names, converts integer keys to decimal names, restores valid
unknown mangled keys dynamically, and uses plain protected names to address declared
protected slots. Plain private names become separate dynamic fields; mangled private
names target the physical declaring class. An invalid NUL name emits a notice but is
still published and later fields continue. Existing __rt_object_restore_property is
not reusable unchanged: it rejects empty/unknown-mangled names, only matches mangled
protected names and uses ordinary checked writers. Preserve this behavior for its
existing users; the native SPL loader needs an explicit private protocol.

M36 target verifier is terminal/successful: all 140 cases pass, source guard true.
84 actual native executions on macOS/Linux ARM64/Linux x86_64 match pinned PHP output
and report clean heaps; 56 iOS device/simulator cases assemble (no execution claim).
The focused native lot passes 64/64; the unchanged 36-case PHPT remains 27/4/5.
checkpoint.json freezes this bounded scope with exact production/test hashes and
full_compliance=false. Production and test source bytes stayed unchanged throughout
build/native/target/combined phases. All process handles for M36 are terminal.
Next implement the private native name/property loading protocol using the new PHP
key controls, followed by exact public __unserialize validation/insertion/error chain.
Existing restore.rs is checked generic hydration and must keep its existing contract.
No commit or push. The 1710-case goal remains active.

M36 Trail check reports the same 20 lexical conflicts in historical campaign plans,
limited to the first 400 KB / 500 changed files. Log: M36/trail-check.log. This is not
whole-branch approval, and no commit/push is authorized by this result. The wrapper
printed excerpts after Trail; it did not preserve Trail's numeric exit status.
Additional *_no_allow controls prove dynamic loading emits deprecations without
AllowDynamicProperties. The plain private-name diagnostic unexpectedly displays H::$
while publishing the priv key; retain complete oracle bytes rather than normalizing it.

## M37 name-based native property restoration in progress

M36 source equality was freshly verified before edits. Foreign Cargo belongs to build-perf
and does not use this campaign target; it was left untouched. The new compiler-only
NativePropertyRestore operation has fixed (Object, Str, Mixed) -> Mixed storage semantics;
the backend expands the string pair into four C ABI words. Runtime feature discovery
enables heap scopes. The user-assembly native_restore helper resolves the actual runtime
class and exact physical user slots, including private ancestor names and both protected
spellings, then calls the guarded M36 replacement primitive. Initialized readonly fields
throw Error before retirement; an uninitialized readonly slot accepts the raw incoming tag.

The helper currently routes unmatched keys through the object's property hash. Heap
metadata reserves a name-addressable GC-visible tail without granting dynamic permission.
This is not a complete dynamic-property loader: diagnostics, borrowed destructor views
of hash entries, and publication after dynamic destructor errors remain open. Reference
slots explicitly reject detachment until their per-instance representation is implemented;
never claim reference compatibility from these value-only controls.

Nine oracle controls live under native-property-restore-oracles-m37 with exact source
hashes/output: public type bypass, protected aliases, private ancestors, initialized and
uninitialized readonly, destructor view/pending error, dynamic empty and unknown mangled
names. Only the adapter is replaced in the native test AST. PHP still executes real SPL
__unserialize, returning destructor errors but rethrowing readonly validation errors.
The tests can export user/runtime assembly through ELEPHC_NATIVE_RESTORE_ASM_DIR;
normal unset-environment runs still execute native binaries and require clean heaps.

The initial library check caught a private-module import; fixed via the existing public
runtime re-export, no new runtime API. A guarded M37 build is now live at exec 55640.
Do not edit Rust during guarded build/test phases. After build, run
verify-native-property-restore-m37.py, then verify-native-restore-targets-m37.py
(18 cases per target), combined regressions, and unchanged heap PHPT. Public __unserialize,
reference detachment, exact dynamic diagnostics/publication and complete corpus remain open.

M37 production build is terminal/successful: source guards true, current default archive
verified, CLI 3f146e41ec1168452336130cb09a1dd1f861abb566e49986ae5dc2e7e575f4dd,
rlib 161e413201c060589ce5c8c9c65e031d97b405e3b7682bc0903e925526fa421d.
Nine-control native verifier is now live at exec 92938; do not edit Rust until terminal.

Further pinned PHP name controls under native-property-name-oracles-m37 found an important
remaining mismatch in the initial exact-key resolver: mangled class names compare without
case sensitivity for private slots; ANY valid mangling scope can select a matching public
or protected slot by its unmangled suffix. A wrong scope for a private field remains a
separate dynamic key. Next normalize valid mangled key suffixes before public/protected
dispatch, and compare private declaring prefixes case-insensitively (property bytes remain
case-sensitive). Do not call the initial resolver complete or run the full target acceptance
until these controls have been implemented. The full public native loader remains open.

M37 guarded native verifier is terminal/successful: all nine controls pass with clean
heaps in both modes; sources stayed unchanged. M37 is bounded evidence only: name
normalization gaps were already found by the next PHP controls, so no all-target or
whole-loader acceptance is claimed. Production CLI also warns that the newly introduced
compiler-call variant is not constructed outside tests until public AST generation lands.

## M38 native key normalization

The name dispatcher now parses a valid NUL-delimited scope once, keeps original key
bytes for dynamic insertion, and matches declared public/protected names by the unmangled
suffix. Private matching additionally checks the declaring-class prefix without ASCII
case sensitivity, leaving property bytes case-sensitive. New tests freeze foreign scopes
on public/protected fields, lowercase private scopes, plain private shadow keys and wrong
private scopes. There are 14 restoration controls total.

Guarded M38 build is live at exec 54359. Do not edit Rust until terminal. Next run
verify-native-property-restore-m38.py, then verify-native-restore-targets-m38.py
(28 cases per target), 78 combined focused regressions and fresh unchanged heap PHPT.
The dynamic diagnostics/reference/dynamic-destructor/public-loader gates remain open.

Fresh direct php-src inspection reconfirms public method ordering: regular __unserialize
starts with consistency(write=true); queue does not. Both validate count/[0], load all
properties, reject pending errors, then validate [1]/flags/elements. Internal insertion
failure wraps the comparator exception in Invalid serialization data; both finish with
consistency(write=false). Inline the existing insertion body without its public entry
assertion or final return: queue must not acquire regular heap's early guard, and later
property callbacks must not add a consistency check that php-src does not perform.

M38's first production build was preserved at main-m38-native-property-name-before-hygiene.
Assembly-comment alignment was then corrected (the new emitter and one existing missing
space in the touched shared file), and a fresh guarded build was completed at the final
main-m38-native-property-name-normalization path. CLI
2c30c6f733b0388f3936fd9c8966b315d44d7b2e0ac64737b6dee6d123049b00;
rlib b3ffc3dc0d444bc0c9db1b192533c9fecde74e932427c2974706521dcc4d3d76.
All 14 native restoration tests pass in both modes with clean heaps and guarded sources;
the combined relevant lot passes 78/78. Fresh unchanged heap PHPT stays 27/4/5, 1674
unselected. The target execution verifier is live at exec 12281. Do not edit Rust until
its source guard completes. Preserve native-verification.json separately from the final
cross-target verification.json summary. All other M38 handles are terminal.

M38 actual target execution found a new x86_64 bug after 80 completed passing cases:
readonly_initialized/off remained live in its own Docker container. Runtime assembly
shows __rt_str_persist consumes/returns the internal rax/rdx string pair, whereas the
new Error emitter supplied rdi/rsi. The object-handle helper correctly uses rax; it was
not the cause. Process/mount evidence and failed qemu ptrace attempt are preserved in
M38/readonly-x86-*.log, runtime.s/user.s and target-abort.json. The own verifier PID30433
was deliberately stopped, then only its verified own container c9c71d5e343b was killed.
All foreign containers were preserved. M38 target handle12281 is terminal143; no full
M38 target acceptance. Source guard before termination was true.

M39 changes only Error string materialization to abi::string_result_regs and adds a
60-second bounded native verification runner with named own-container cleanup on timeout.
Run the full 14 native controls and all 140 target cases again, plus combined regressions.
The M38 78/78 host regression and 27/4/5 PHPT results remain valid for their old source;
never transfer target acceptance to this new artifact without execution.

M39 target scope is deliberately expanded to BOTH 14 old-value view controls and 14
name-restoration controls: scope_dynamic_property_storage adds a GC-visible tail to
heap layouts, which can affect cloning as well as loading. Export both test families
into the same target directory, 56 cases/target, 280 total. This gives 168 actual native
executions and 112 iOS assembly cases when all pass. x86_64 and readonly_initialized
run first to revalidate the observed failure without waiting through other architectures.
A fresh live GitHub read confirms PR1806 is still OPEN at head5125ea8abf72f83fda67ed5cc18556824b0fe781,
base5d96278997e970e007be6950f3a36ca9c96e2cce. No publication occurred.

M39 guarded build is terminal/successful; CLI
a6154abcda63405987c578f9392e36f37640bf5d5269e22d2db31cd068cf1870,
rlib dbccc582df7b56afc939c38cf5c531672e594f69e5ba694bd9aef7d36e0875a6.
Native 14-control verification passes, source guard true; combined relevant lot78/78
also passes. The actual readonly_initialized x86_64 case that hung on M38 now passes
both optimizer modes with exact output and clean heaps. The expanded 280-case target
verifier remains live at exec87970; all other M39 verification handles are terminal.
Do not edit Rust until the target source guard completes. New native timeout applies
only to named own containers, leaving all foreign containers untouched.

New pinned PHP controls for the next implementation step:
- native-dynamic-property-oracles-m39: dynamic destructor old|new; a destructor failure
  still publishes new and writes the later field before wrapping Invalid serialization;
  reference detachment leaves integer:1|string:new; reentrant write is callback|new;
  reentrant unset is absent|absent (do not reinsert a bucket the callback deleted).
- native-property-edge-oracles-m39: mangled unknown scopes invoke autoload (Unknown|bad),
  class_alias names select the original private slot, readonly clone reinitialization is
  allowed once, and throwing notice/deprecation handlers do not prevent property writes
  or later fields before the outer Invalid serialization wrapper.
- IMPORTANT correction to the initial M38 generalization: an UNREGISTERED foreign scope
  becomes NULL lookup and can select a protected field under the native caller scope;
  a REGISTERED unrelated scope cannot. Both registered Other controls preserve the
  protected value1 and publish a separate raw mangled dynamic key. Any scope beginning
  with * leaves fake scope unchanged, not just the exact one-character star spelling.
  The current partial name dispatcher does NOT implement these runtime class/scope cases.

Exact object_properties_load source from pinned Zend/zend_API.c explains these controls:
zend_lookup_class runs before property-info resolution; native scope and readonly
IS_PROP_REINITABLE flags matter; ordinary typed checks are bypassed; diagnostics can
leave a pending exception while the C property loop still writes/continues. These are
mandatory full-compliance gates, not intentionally unsupported final behavior. The
current operation remains private/test-only until public heap AST generation is added.

M39 expanded target matrix is terminal/successful: all280 cases pass, source guardtrue;
168 actual native executions exact/clean and112 iOS assembly-only cases. All M39 handles
are now terminal. The source-specific checkpoint.json records78 focused tests, fresh
27/4/5 PHPT, the M38 x86 failure/fix lineage, partial native semantics and all open gates.
Trail check exited1 with20 historical-plan lexical alerts, truncated400KB/500files;
it is not full branch approval. No commit/push. Next wire actual public __unserialize
validation/property loop/native insertion with its exact regular/queue ordering, while
retaining every native scope/reference/dynamic edge gate above as mandatory unfinished work.

## M40 public __unserialize implementation in progress

M39's guard and all280 target cases completed before these Rust edits. Public inherited
__unserialize methods are now registered for both heaps/queue with the boxed PHP array
contract and added to the EIR supported-SPL whitelist. The new unserialization.rs body
keeps pinned regular/queue order: properties first, pending-error chaining and readonly
stop, then flags/elements, inline native insertion without the public entry precondition
or return, and final read-consistency validation. Invalid validation guards stay outside
the insertion catch so they are not double-wrapped.

Eight unmodified PHP public-path oracles are frozen at heap-unserialize-public-oracles-m40:
formats/null queue payload, append without insert override, properties before invalid
state, queue flag publication/masking, comparator failure/corruption, continuing property
destructor error, readonly error chain, and wire serialize/unserialize roundtrip. Native
regressions are registered at runtime_gc/spl_heap_unserialize.rs in both optimizer modes.
A first library check found the private compiler_call builder inaccessible; the new module
now owns its trivial trusted Name constructor. No behavioral public acceptance yet.

A guarded production build is live at exec64819 under main-m40-heap-unserialize-public.
Do not edit Rust until terminal. After build, run smoke-heap-unserialize-m40.py against
the pinned CLI: it runs16 actual native cases with exact output/clean heaps and stops
on the first failure, preserving all stderr/commands. Then fix that evidence before
running the new Rust lot/full relevant lot and unchanged heap PHPT. New public behavior
still inherits all known partial name/reference/dynamic scope gates from M39; full
compliance remains false. Warning-free construction should improve once this compiler
call variant is actually used by production AST generation.

M40 production build is terminal/successful with guarded source bytes, CLI
bb4b51adb7a23de97d805460eda9cb61fe192272f8548450df151cd7ebbbb8b0,
rlib1ad8ec188f30a995b326ec0c071d846c693cd8ed04aced2be835275c6d19d7fa.
Its public CLI smoke is terminal/FAILED: append_override/on panics during EIR lowering.
ExceptionChainMixed operand0 is a physical Mixed propertyError slot despite the preceding
instanceof guard; the typed operation requires Heap(Object). Full compile.stderr and
source guardtrue are preserved in M40/smoke; no public acceptance or PHPT claim.

M41 changes only the generated error-chain expression to cast that proven Throwable
value to Object before the private raw-object operation. This uses the ordinary object
cast's existing identity/ownership protocol and avoids weakening the typed signature or
changing existing M25 chain callers. The next guarded build is main-m41-heap-unserialize-object-chain;
after terminal success run smoke-heap-unserialize-m41.py before any broader validation.

M41 guarded production build is terminal/successful, but public smoke is terminal/FAILED:
PHP Object casts deliberately remain boxed Heap(Mixed), so the same typed chain assertion
still fails with a Cast expression. The failed smoke/source guard and pinned artifacts
remain preserved; do not count M40/M41 as successful public loading.

M42 removes that ineffective AST cast and uses the existing owned MixedUnbox protocol
through merge_temps::coerce_value_for_temp in compiler-call lowering, only for operand0
of the private ExceptionChainMixed operation. The generated property loop dominates it
with instanceof Throwable; ordinary source cannot select the capability. The helper
acquires an object payload lease and retires any owned boxed source, preserving the raw
Object signature and later operand cleanup. Existing Object catch-variable callers stay
unchanged. Next guarded build main-m42-heap-unserialize-typed-chain, then its public smoke.

M42 guarded build and public compilation succeed, proving the owned object ABI bridge
fixed the previous EIR assertion. Execution still FAILS: append_override/on segfaults.
The retained symbol/debug-info rebuild plus lldb-crash.log locate exception_matches
reading raw0x7ffffffffffffffe from SplHeap::__unserialize. emit-ir.stdout proves slot11
starts as boxed Mixed null but the catch path later stores raw Heap(Object) into it,
and subsequent reads become Heap(Object) even when the catch did not execute. This is
the new synthetic builder's unstable assignment representation, not a passing load.

M43 keeps pendingPropertyError boxed through typed_assign_stmt(mixed) for every update,
including raw caught Throwables; it does not change generic nullable-object behavior.
Two additional required public regressions freeze lock/corruption ordering, giving10
public controls/20 native smoke cases. Example spl-heap-serialization now demonstrates
actual restoration and extraction; its PHP output/source hash are frozen separately.
Next guarded build main-m43-heap-unserialize-pending-storage, then its smoke before any
broader Rust/PHPT/target acceptance. All M42 processes are terminal; no failed public
artifact has been accepted. The last accepted bounded primitive remains M39, full goal active.

M43 guarded build succeeds. Public CLI smoke now passes12 actual cases (append override,
comparator exception, corruption order, formats/null queue, lock order, properties before
state; both modes). It then FAILS property_error_continues/on: output is
Invalid serialization data for H object||new:2 instead of ...|drop|new:2, heap clean.
The wrapper's previous Throwable message is empty. No complete public acceptance yet.

Fresh unchanged M43 heap PHPT improves from27/36 to33/36: only serialize_error_handling
compile_error, by-reference compare compile_error and SplMaxHeap_serialize_complex
mismatch remain. These are specific to the M43 pinned artifact, not full1710 compliance.

M44 constructs the Invalid serialization Exception without a previous argument, then
uses the existing native ExceptionChainMixed operation before throwing. This mirrors
zend_throw_exception_ex's pending-exception chaining directly, rather than feeding a
physical Mixed pending cell to the ordinary raw previous-argument constructor path.
Catch variables and pending boxed errors share the same trusted chain operation.
Run the same20-case public smoke again after the guarded build; preserve M43 evidence.
A separate ordinary Mixed-previous constructor control should be checked against M39
and current main before classifying/fixing any generic constructor issue on its own branch.

M44 guarded build succeeds. Public smoke passes14 native cases, including the previously
wrong destructor-error chain in both modes, with exact output/clean heaps. Next case
readonly-chain fails compilation solely because the auxiliary PHP fixture reassigns its
typed catch variable to Throwable|null. The original valid PHP fixture is preserved in
M40/M43; this is a separate frontend limitation, not a native readonly result.

M45 is TEST-ONLY relative to M44 production: spell out the same three-node previous chain
with separate guarded variables and reject any extra link. Reference PHP output remains
identical. Add explicit assembly export to the public test helper; normal unset-env runs
still execute native. New source/expected controls live in public-oracles-m45. After the
fresh guarded build, verify production CLI/rlib identity with M44, run the20 public CLI
smokes, the10 Rust public regressions, then88 combined regressions and100 target cases
(60 native/40 iOS assembly). Only after public proof is complete accept this bounded stage.

Remaining M43 upstream failures were inspected: serialize_error_handling rejects a Mixed
foreach argument to the array $data contract at compile time; by-reference compare is
still its known ABI issue; serialize_complex has wrong associative-array ordering already
on the WRITER side. Two controls on pinned M39 (before public loading) independently
prove generic gaps: Mixed previous Exception constructor prints empty instead of drop
with clean heaps; Mixed hash spaceship returns0 instead of PHP1 with clean heaps. Direct
typed hash spaceship is a separate existing checker diagnostic. Artifacts are
mixed-previous-constructor-baseline-m39 and mixed-hash-compare-baseline-m39. Verify fresh
main before generic fixes; use dedicated branches if they reproduce there. These remain
mandatory campaign gates, never count them as fixed by native exception chaining.
