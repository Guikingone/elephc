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
