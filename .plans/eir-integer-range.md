# EIR integer range and induction-variable analysis

- [x] Define an inclusive integer interval domain and edge-sensitive EIR dataflow analysis.
- [x] Propagate ranges through constants, comparisons, loop-carried block parameters, masks, shifts, and integer arithmetic.
- [x] Recognize bounded induction variables on natural loops and prove safe checked updates.
- [x] Rewrite only proven `ICheckedAdd` / `ICheckedSub` / `ICheckedMul` operations and their integer-sink forms to unchecked scalar EIR.
- [x] Preserve boxed PHP overflow-to-float behavior for every operation without a complete proof.
- [x] Add unit, optimizer-on/off runtime, EIR-shape, and all-supported-target compile coverage.
- [x] Add an example, update optimizer documentation and the roadmap, then run focused verification.

## Implementation notes

The pass runs after `mem2reg` and checked-integer sink specialization, before checked numeric chain fusion. Its forward state is path-sensitive at `ICmp` branches and maps SSA integer values to inclusive `i64` intervals. Natural-loop information identifies loop-carried header parameters and their constant-step recurrences so loop bounds can constrain both body values and checked updates without unrolling the abstract interpretation.

Arithmetic transfer uses wider intermediate calculations. A checked operation is rewritten only when every endpoint calculation remains inside the signed 64-bit range. Boxed checked results are narrowed to scalar `I64` only when their complete use shape can consume the narrowed representation; otherwise they remain checked even when a local range fact exists. Unknown inputs, unsupported CFG shapes, invalid shift counts, exceptional control flow, and any range merge that loses the needed bound fail closed.

Runtime tests compare optimizer-on and optimizer-off behavior for both proven-safe loops and deliberately overflowing expressions. Target tests emit both optimized and unoptimized assembly for `macos-aarch64`, `ios-arm64`, `ios-sim-arm64`, `linux-aarch64`, and `linux-x86_64`, proving the optimization stays target-neutral while unproven overflow paths retain the checked helper.

## PR review and CI follow-up

- [x] Integrate the current mem2reg dependency and current main without rewriting published commits.
- [x] Reproduce all three Greptile findings with valid hand-built EIR regressions.
- [x] Invalidate facts absent from an incoming range state, preserve unsupported comparison edges, and check every parallel loop back edge.
- [x] Reduce repeated full-function work in range specialization and split composite ownership fixtures responsible for CI timeouts without reducing coverage.
- [x] Run focused EIR, optimizer-on/off, all-target emission, and ownership regressions.
- [x] Commit and push the fixes, and inspect the new CI run.

Splitting native and eval ownership fixtures preserves both repetition counts and
all assertions. The scalar eval-only fixture still takes 79 seconds with warm
bridges locally, so only the four affected eval tests receive a 120-second
Nextest budget on Linux x86_64; native tests, the existing ARM64 family budget,
and the global 60-second budget are unchanged.

## Deep implementation audit

- [x] Audit transfer semantics, loop proofs, boxed consumers, and convergence limits.
- [x] Reproduce and fix boolean-cast interval corruption, including boolean sinks in adjacent passes.
- [x] Preserve boxed representation for direct static-local assignments.
- [x] Bound static loop-expression analysis on deep and shared expression graphs.
- [x] Validate focused regressions and update documentation.

The deep audit reproduced boolean-cast miscompilation in native optimizer-on/off
execution and isolated regressions in the range, integer-sink, and numeric-chain
passes. It also exposed unsafe narrowing into direct static-local stores and
missing memoization of unknown static expressions. Focused validation passed:
198 EIR unit tests, 10 range-related codegen tests, 11 adjacent checked-arithmetic
codegen tests, all five target emitters in both modes, and a warning-free build.
Sampled domain checks cover arithmetic, bitwise operations, shifts, comparison
refinements, and induction summaries, including signed 64-bit boundary values.

## Second deep audit

- [x] Check every boxed consumer against actual backend conversion and storage contracts.
- [x] Reproduce and correct consumer-sensitive narrowing regressions.
- [x] Exercise generated cyclic CFGs and composed numeric expressions differentially.
- [x] Validate focused tests, all targets, and document the revised safety boundary.

The second audit reproduced a native crash when narrowing arithmetic consumed by
an array cast and a compile failure when assigning it to typed static properties.
Unit regressions also cover typed reference-cell conversions and folded floating
overflow stored into an integer slot. IntegerRange and ConstFold now share a
consumer contract, so constant folding cannot bypass the representation guard.
Validation passed: 202 EIR unit tests, 13 range codegen tests, 11 adjacent checked
arithmetic tests, 6 constant-propagation tests, and a warning-free build. Generated
checks cover 972 cyclic CFGs and 180 composed-expression results; target emission
checks cover all five targets with optimization enabled and disabled.
The previous head's CI still reports unrelated fixture timeouts. A preexisting
by-reference float-parameter discrepancy was recorded separately for follow-up.

## Third deep audit

- [x] Recheck dataflow, induction, and direct consumer contracts after the shared guard change.
- [x] Reproduce precision loss in Mixed/int loose equality and preserve exact integer payloads.
- [x] Preserve runtime-tagged relational operands and bool/null spaceship coercions.
- [x] Add concrete expected-output, generated observer, unit, and all-target regressions.
- [x] Complete focused validation and record the local-only commit.

The audit reproduced three further failures: loose equality rounded integer
payloads above the exact double range, PhpRelCmp rejected two narrowed scalar
operands, and spaceship narrowing changed bool/null comparisons from truthiness
to numeric ordering. Exact integer-tag equality now has a target-aware path;
the shared consumer guard retains boxed ordering inputs where required.
The observer matrix checks 675 cast, predicate, and comparison results plus
echo/print_r output with optimization enabled and disabled. Separate expected
outputs cover 36 large-integer equality comparisons and bool/null ordering,
including the zero-mask constant-folding path.
Focused validation passed: 203 EIR unit tests, 17 range codegen tests, 15 loose
comparison codegen tests, all five target emitters in both modes, a warning-free
build, and assembly-comment alignment. The changes remain local because the PR
is ready for review and the contribution policy prohibits further pushes.

## Fourth deep audit

- [x] Recheck simultaneous narrowing, nullable storage, and ownership contracts.
- [x] Reproduce legacy null-sentinel collisions in boxed narrowing and scalar null folding.
- [x] Preserve boxed collision payloads and ambiguous scalar null predicates.
- [x] Add representation-specific unit, runtime, heap-debug, and all-target coverage.
- [x] Complete focused validation and record the local-only commit.

The audit reproduced optimizer-on/off divergence under `--null-repr=sentinel`:
range and constant specialization could reinterpret an ordinary boxed integer
payload as null, and scalar null-predicate folding could discard the legacy
storage interpretation. Boxed interval proofs and exact integer folds now share
a payload-compatibility check. Null predicates over ambiguous raw integer or float
sentinel bits remain runtime operations. Neighboring integer values and tagged
mode still specialize, as do existing scalar checked integer sinks.
Validation passed: 206 EIR unit tests, 21 range codegen tests, all five target
emitters with both null representations and optimizer modes, reference/array
ownership with heap debugging, a warning-free build, and `git diff --check`.
The commit remains local under the ready-for-review contribution policy.

## Latest rebased CI repair

- [x] Inspect the failed jobs on head `5d590ee1` and compare focused optimizer-on/off runs.
- [x] Separate native/eval ownership, output-buffer clean/flush, and multisort null-representation cases.
- [x] Restore the array-argument eval fixture named by the existing timeout override.
- [x] Verify the resulting cases under the CI profile in both optimizer modes and prepare the fix for publication.

The failed run terminated six composite fixtures at the 60-second outer limit.
The information-result fixture passed locally in 55.93 seconds with optimization
enabled and 32.41 seconds with it disabled. Independent cases now have independent
timeouts without removing PHP source operations, allocation comparisons, heap
checks, null representations, or repetition counts. The three additional
two-compilation eval ownership probes receive the existing 120-second Linux
x86_64 budget; native cases and the global 60-second guard remain unchanged.

Focused validation passed with zero retries: all 11 affected cases with the
optimizer enabled and disabled, seven integer-range all-target tests, a
warning-free build, configuration filter-name checks, and `git diff --check`.
The full supported-target execution matrix remains the CI publication gate.

## Remaining x86_64 timeout envelopes

- [x] Inspect the seven new 60-second timeouts on head `22accea8d`.
- [x] Reproduce redundant range facts and compare the pass's prelude analysis cost.
- [x] Keep states sparse and use constant-time fact-availability queries without weakening overflow proofs.
- [x] Split catalog ownership and MIME weak/strict native/eval cases, preserving every original assertion.
- [x] Apply a bounded eval-ownership family budget instead of adding one exception per failed member.
- [x] Validate the final focused optimizer-on/off cases and prepare publication.

The preceding repairs passed; this run failed on different members of the same
multi-compilation eval cohort. A native-named nested-property ownership helper
also compiles two opaque eval programs, so it receives the existing core-eval
budget. Ordinary native tests and the global guard retain their 60-second limit.

The range analysis previously copied full-domain scalar facts and immutable
literal facts through CFG edges. Regression fixtures reproduced 1,028 redundant
entries in a state that can be empty. Sparse states recover those facts from
types and literal definitions, while full-domain boxed facts remain explicit
because they prove the integer tag. Iterative dominator-tree intervals avoid a
full ancestor walk for each retained fact. Tests compare these queries with the
original analysis on deep, unreachable, and 972 generated cyclic CFGs.

Final focused validation passed: 211 EIR unit tests, 23 integer-range runtime and
all-target regressions, and all 11 timeout replacement cases in both optimizer
modes under the CI profile with zero retries. The eval-budget selector matches
34 explicitly named eval ownership cases and excludes native `evaluation` and
`spread_evaluation` cases. Build output is warning-free, temporary profiling was
removed, and the global/compiled-program 60-second guards remain intact.

## Integration with current main

- [x] Resolve the overlapping MIME validation split using main's four test names.
- [x] Reuse main's mbstring-family budget instead of retaining a duplicate x86_64 override.
- [x] Revalidate the merged range pass, timeout regressions, and configuration.
- [x] Prepare the integration commit and exact-head CI verification.
- [ ] Confirm the complete supported-target CI matrix on the published head.

Main now includes the same weak/strict native/eval MIME split and a bounded
mbstring-family budget. The merge preserves these changes and every original
assertion. The redundant earlier x86_64-only eval ownership override is removed;
the nested-default opaque-eval budget remains scoped to its actual helper users.

Post-merge validation passed with zero retries: 211 EIR unit tests, 34 codegen
cases with the optimizer enabled (23 integer-range and 11 timeout regressions),
and the same 11 timeout regressions with the optimizer disabled. The four slow
eval fixtures finished in approximately 64 seconds within their bounded budget.
The compiler and updated Magician library build without warnings, Nextest parses
the merged configuration, and `git diff --check` is clean.

## Final macOS non-codegen timeout repair

- [x] Verify the fresh 5/5 review and isolate the three remaining macOS timeout envelopes.
- [x] Schedule each target independently for the two five-target ownership fixtures.
- [x] Give the single combined PDO/mysqli compilation the existing exact PDO budget.
- [x] Validate all ten target cases and the combined extension probe (optimizer on/off).
- [ ] Publish the CI-only repair and confirm fresh exact-head CI.

Run `37336788502` on head `63d737f6f` passed every Linux and iOS job and all
but one macOS shard. Two tests enclosed five complete target compilations each;
the third injected both database preludes into one cold-cache executable. All
three were terminated twice at exactly 60 seconds, with no assertion failure.
The target split preserves both original PHP sources, every ownership assertion,
and assembly emission for all five supported targets. The combined extension
probe keeps its original source and output check and receives a bounded
180-second override in the default and CI profiles. The global guard is unchanged.

Focused CI-profile validation passed with zero retries: all ten target cases
both in parallel and serial scheduling, and the combined extension probe with
optimization on (17.78 seconds) and off (15.04 seconds). Each serial target case
finished in 7.94-12.22 seconds. The compiler build is warning-free, Nextest lists
exactly ten replacement cases, both profile overrides match only the combined
extension probe, and `git diff --check` is clean. Task-owned temporary files
were removed after validation. Fresh full CI remains the readiness gate.
