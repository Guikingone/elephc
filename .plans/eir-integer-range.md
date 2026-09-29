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
