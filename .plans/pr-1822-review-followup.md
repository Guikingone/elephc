# PR #1822 review follow-up

- [x] Freeze the reviewed head and inspect all review threads and CI.
- [x] Reproduce the four open findings against PHP and the starting compiler.
- [x] Guard false arguments on instance, static, nullable-receiver, and parent calls.
- [x] Preserve null-only property declarations separately from void.
- [x] Diagnose unsupported true-only properties, including promoted properties.
- [x] Name the actual object class in property and argument TypeErrors.
- [x] Protect partially constructed receivers on exceptions and model parameter guards.
- [x] Complete regression, parser, error, optimizer, and target checks after the last code edit.
- [x] Publish the follow-up on the existing branch and inspect exact-SHA CI.
- [x] Consolidate the verified review response and resolve the four treated threads.

## Provenance

Reviewed head: `889c6c55770f9480ee8d56e875b0b135e882ac6b`.
Observed base: `4fdabfb8b1cd0b3bc58e789c846cf5970e86490e`.
Branch: `feat/generics-optimizations`; worktree was clean at entry.
All existing head CI checks had succeeded. Three older threads were resolved;
the four new threads are the current correction scope.

## Changes and decisions

False parameter guards consume declared signatures before ABI bool coercion.
The same mixed tag/payload check is used for instance, static, nullable receiver,
and parent constructor calls. Error composition uses the runtime class-name table.
False property diagnostics retain the declared name `false` rather than `bool`.

Null-only properties retain an explicit named null annotation, which resolves to
the existing null runtime representation. Void declarations remain invalid.
True-only property storage is not implemented: the review explicitly permits
rejecting these declarations until the correct guard exists. Ordinary, promoted,
nullable, and union declarations therefore receive an explicit diagnostic.

Late argument guards can throw even for pure method bodies. Both conservative
effects and the exact exception-flow summaries must retain that TypeError, or a
catch can be pruned. Constructor validation happens before allocation; real
constructor failure uses the existing unconstructed-object ownership guard,
which releases fields and suppresses premature destructor invocation.

## Evidence

The initial follow-up exposed pruned static catches and leaked failed constructors.
Temporary logs from the interrupted session were cleared; the final evidence was
regenerated from the unchanged source before publication.

Fresh checks: six new heap-debug regressions pass with optimization on and off;
37 optimizer-effect tests, seven EIR-effect tests, one parser test, two diagnostic
tests, and four existing generic-method/variadic regressions pass. The compiler
build is warning-free. Earlier same-source generics runs passed 199 tests in each
optimization mode; these are separate from the fresh focused results.

All six new PHP fixtures also match PHP on Linux ARM64 and Linux x86_64, on/off:
24 Docker executions with clean heaps. x86_64 executes under QEMU on the ARM64
host. Only PHP's argument-error caller-location suffix is normalized; complete
property values and object-variable arrays are compared. A probe covering the
four fixes emits and assembles on all five supported targets, on/off: ten checks.
iOS evidence is compilation/assembly rather than device execution.

The example matches PHP on/off with clean heaps. Its first version exposed the
separate unsupported `false` declaration-default path; it now initializes the
properties in its constructor. That limitation is documented, and no unrelated
default-value implementation is added here. Assembly-comment alignment and
`git diff --check` pass. No full local suite was run.

## Publication

Code commit: `15b977c4e17d5cf7354f970a4f46f4b03996c094`, pushed to the existing
`origin/feat/generics-optimizations` branch. GitHub confirmed the exact head;
the PR is mergeable and its required CI checks were queued/running, with no
reported failure at inspection. Full CI completion remains pending.

Verified response: https://github.com/illegalstudio/elephc/pull/1822#issuecomment-6149367830.
All seven threads are resolved: the three older threads plus these four fixes.
This subsequent plan-only update does not alter the validated compiler source.

## Second review round — 2026-10-13

Reviewed head: `2d14195d242ec585960867c5df836b8f2181b874`.
The worktree was clean. GitHub was refreshed before publication: the same five
new open threads were present; seven previous threads remained resolved.

- Promoted `null`: preserve the named-null declaration exception in parameter
  checking, allow boxed value arguments at the user method boundary, check their
  runtime tag before allocation/coercion, and pass a canonical null ABI word.
  `void` declarations and by-reference mixed storage keep their existing gates.
- Interface `false`: inspect the concrete implementation before argument
  materialization. The guard uses its declaring class/signature, including
  inheritance, and does not reject wider contravariant `bool` implementations.
  Nullable receivers perform their null check first. Throwable intrinsics retain
  their existing path.
- Assembly folding: both normalization passes preserve indented `.ascii` data.
  The unit regression keeps label references canonical but distinguishes literal
  payload bytes that happen to spell those labels.
- Restore complete immediate docblocks for the CLI native fixture runner and
  mixed property guard. Document the EIR `shape()` / `has_nominal_use` approach
  as rejected history and the assembly comparison as the active mechanism.

Baseline probes reproduced the promoted-null declaration error and acceptance
of integer zero through an interface false parameter. An intermediate regression
then caught the additional mixed-to-null checker rejection; that failure is
retained in the temporary evidence rather than counted as a passing result.

Final execution evidence: 203 generics tests passed with optimization on and 203
with optimization off, including ten heap-debug review regressions (four new).
Four new PHP fixtures also match complete PHP stdout, except the existing PHP
argument-error caller-location suffix normalization, on both Linux architectures
in both modes: 16 executions, clean heaps. x86_64 runs under QEMU. All four
fixtures emit and assemble for all five supported targets, on/off: 40 checks.
iOS evidence is compile/assembly only. The updated example demonstrates promoted
null with a default and matches PHP with clean heaps on/off.

Compiler SHA-256 used for executable and target probes:
`58eab4b1ef979f1950111075b8937fa5260ef04b3510c6f60c41a459474ac25b`.
Evidence directory: `/private/tmp/elephc-1822-round2`.
Assembly-comment alignment and diff hygiene pass. No full local suite was run.
The final normalizer unit regression, three review diagnostics, and the existing
ordinary void-parameter rejection pass. The dev build is warning-free. Because
the test and dev builds produce different compiler artifacts, all 16 Linux
executions and 40 assembly checks were replayed with the final dev compiler;
the four fixtures plus the example also match PHP with clean heaps on macOS,
on/off (ten executions). Publication follows on the existing branch.


## Third review round — mixed receiver literal arguments

Reviewed head: `cf91e46213977197c408b14a3aa9c1a604355d2a`.
The generics worktree was clean. One new thread remained open:
`PRRT_kwDORs6r_s6sCqjR` / comment `4263494962`.

The PHP oracle rejects integer/string/object arguments to a promoted null
constructor when it is explicitly invoked through a mixed receiver. The
starting compiler accepted each argument, replaced it with null, and executed
the constructor body again. A counter rose from two to five instead of staying
two. The baseline is retained in `/private/tmp/elephc-1822-round3/baseline.json`.

The mixed candidate path now runs the same declared literal parameter guard as
concrete/interface calls before preparing ABI arguments. The receiver register
is preserved across guard calls. Throwable compact intrinsics retain their
existing bypass. Three heap-debug regressions cover null/false constructor
re-entry and a pure false-parameter method. They compare body counters and
stored values as well as complete error messages.

All 13 review regressions pass on/off after the source edit. Seven existing
mixed dispatch tests, the owned-argument return control and the omitted by-ref
default-argument control pass. The source change is five lines and reuses the
existing target-aware guard. Assembly comments and diff hygiene pass.
The final dev build is warning-free. Three fixtures match complete PHP stdout
(with only the argument-error caller-location suffix normalized) on macOS and
both Linux targets, on/off: six macOS plus twelve Linux executions, clean heaps.
All three probes emit and assemble on all five targets on/off (30 checks).
Linux x86_64 runs under QEMU; iOS evidence is compile/assembly only.
Final compiler SHA-256: `051470ebf8a74f70d33144b91684d7cb0dd5d43b90122d2a27f7b0308b30545f`.
Evidence directory: `/private/tmp/elephc-1822-round3`. No full local suite was run.
Publication uses the existing generics branch; full CI remains its readiness gate.

## Fourth review round — argument conversion documentation

Reviewed head: `8309f9de5072ceb1ffdb67edb23bf94c7917d383`.
One new thread remained open: `PRRT_kwDORs6r_s6sD8kG`, comment `4264031250`.
The immediate Rustdoc for `materialize_direct_call_arg_for_param` was missing.
Restore a concise description of its ABI conversion and the ordering constraint
that literal false/null guards must run first. This changes documentation only.

The reviewed head's CI completed with 184 successful checks and one skipped check,
including the aggregate Build & Test gate. The earlier eval shard timeout did
not recur on this head. Verification for this documentation-only change checks
diff hygiene and confirms that removing Rustdoc lines yields identical Rust
source to the reviewed head. No additional runtime suite is required.

## Fifth review round — deep inheritance and ordinary call lookup cost

- [x] Freeze the new upstream head and inspect all current reviews and CI.
- [x] Reproduce the deep hierarchy argument bypass against PHP.
- [x] Remove the fixed ancestor depth limit and cache parameter-only candidates.
- [x] Verify focused generics/review tests in both IR modes and all supported targets.
- [x] Rebase/squash and prepare the corrected version for fork publication.

Reviewed head: 9568ffa2e55f217becf15a8ebc524fa99f8bbd7d.
Its CI had 184 successful checks and one skipped check. Two threads remained:
PRRT_kwDORs6r_s6s04Wg (deep ancestry) and PRRT_kwDORs6r_s6s04Wn
(whole-program searches at ordinary virtual calls).

At 70 parent links, the reviewed compiler accepted true through inherited false
instance and late-static methods, while PHP threw TypeError. Traversal now
follows parents with a visited set instead of a 64-iteration cutoff.
The guard path skips ordinary signatures, caches declared literal parameter
candidates by method/staticness/argument count, and no longer resolves emitted
method bodies just to discover a guard. Runtime implementation diagnostics,
widened overrides, nullable receiver precedence and existing Mixed dispatch
remain covered.

Final focused verification: 158 generics tests and 18 review tests pass with IR
optimization on and off (176 distinct end-to-end tests, 352 executions).
The standalone harnesses also pass their 17 shared support checks.
The final dev build is warning-free. Both new fixtures match normalized PHP
stdout and have clean heaps on macOS ARM64, Linux ARM64 and Linux x86_64 in both
modes (12 executions). Linux x86_64 runs under QEMU. Both fixtures emit on all
five supported targets in both modes (20 combinations); iOS device and simulator
also pass static-library emission and assembly of program/runtime objects
(eight combinations, compile/assembly evidence only).
Assembly-comment alignment, module preambles and diff hygiene pass.
No full local suite was run.

The first in-process test run hit storage-full while writing a runtime assembly
cache, poisoning subsequent cache users; that failed log is retained separately
and is not counted as passing evidence. A standalone null method parameter is
also rejected on the reviewed baseline. The null regression therefore uses the
already supported promoted-null constructor surface; the declaration gap is
not claimed fixed here.

Trail check reports eight keyword collisions: docs/index/dispatch mention the
static signature only to skip unnecessary guard discovery, while actual checks
still select the runtime implementation; regression tests assert rejection of
the baseline bypass; block_emit's isolated compiler cache is unrelated to the
standing rule against renaming the graph cache or overwriting dirty worktrees.
The plan also recounts the failing baseline rather than reintroducing it.
These are not semantic conflicts. The learning is preserved unchanged.

Evidence directory: /private/tmp/elephc-1822-review, including frozen compiler
hashes, exact fixtures, complete logs and target-results.json.

The ordinary-call probe (129 implementations, 512 virtual call sites, IR
optimization disabled) emits byte-identical assembly before and after.
Three alternating warm trials measured medians of 198.7 ms and 193.8 ms;
this small local observation is informational, not a general speedup claim.

Rebased on origin/main 132b4da6f7. The aggregate Rust/Cargo source fingerprint
is unchanged after rebase: da739e2e32ea32972d3e2ee19c10f4d420c386a5fb348fe80727dd054c755ece.
The upstream head remains 9568ffa2e55f217becf15a8ebc524fa99f8bbd7d.
