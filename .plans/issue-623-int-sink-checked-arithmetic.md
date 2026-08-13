# Issue 623: Non-Boxing Checked Integer Arithmetic for Int Sinks

## Task Checklist

- [x] Add scalar checked-to-int EIR operations with PHP-preserving overflow semantics.
- [x] Add target-aware AArch64 and x86_64 lowering without Mixed allocation.
- [x] Add an EIR pass that specializes checked arithmetic only when every consumer is an int sink or ownership cleanup.
- [x] Classify scalar loads from provably immutable local slots so CSE can canonicalize them and LICM can hoist them.
- [x] Register the pass before constant folding, CSE, and LICM.
- [x] Add unit tests for accepted and rejected consumer graphs.
- [x] Add structural CSE and LICM integration regressions.
- [x] Pin optimized/unoptimized overflow parity and mixed-observer behavior.
- [x] Prove per-iteration Mixed allocations disappear under heap debug.
- [x] Correct the buffer documentation's overly literal single-instruction claim.
- [x] Run focused macOS, Linux x86_64, and Linux AArch64 validation plus hygiene checks.

## Semantic Contract

Non-constant integer `+`, `-`, and `*` lower to `ICheckedAdd`, `ICheckedSub`,
and `ICheckedMul` when PHP overflow promotion can make the result a float. The
existing operations return an owned boxed `Mixed`, which correctly preserves the
dynamic result for mixed-observing consumers but prevents CSE and LICM.

The optimization must not replace these operations with wrapping `IAdd`, `ISub`,
or `IMul`. Current `main` implements PHP 8.4 float-to-int conversion through
`__rt_php_float_to_int`, including modulo-2^64 conversion for finite out-of-range
doubles. Optimized and `--no-ir-opt` programs must remain behaviorally identical.

Introduce `ICheckedAddToInt`, `ICheckedSubToInt`, and `ICheckedMulToInt`. Each
operation returns `I64` / `PhpType::Int` / `Ownership::NonHeap`, has pure semantic
effects, and computes the same value as:

1. the corresponding boxed checked arithmetic operation;
2. followed by PHP's integer cast of that dynamic result.

The in-range fast path returns the native integer result directly. The overflow
path converts the original operands to double, performs the floating-point
operation, and routes the result through `__rt_php_float_to_int`. No Mixed cell is
allocated in either path.

## Pass Design

Add a dedicated `CheckedIntSink` EIR pass after `Peephole` and before
`ConstFold`, CSE, and LICM. This is a whole-function use analysis rather than a
local box/unbox peephole.

For each checked add/sub/mul result, collect every instruction and terminator use.
The candidate is eligible only when every path observes the value as an integer:

- `Cast` with `Immediate::CastTarget(IrType::I64)` is an int sink. Its result is
  redirected to the specialized scalar producer and the cast is neutralized.
- `StoreLocal`, `StoreStaticLocal`, or `InitStaticLocal` is an int sink only when
  its referenced local slot has integer storage.
- `StoreRefCell` is an int sink only when its carried alias target type is Int.
- A normal, unmarked `Acquire` is transparent ownership scaffolding only when
  every use of its result recursively reaches an accepted int sink or cleanup.
  Its result is redirected to the scalar producer and the acquire is neutralized.
- `Release` of the checked or transparent acquired result is cleanup and is
  neutralized after specialization.
- Lifetime-pin acquires, terminator uses, mixed stores, output, calls,
  comparisons, and every unknown consumer reject the candidate conservatively.

The commit phase updates both instruction and value metadata, applies dominance-
safe use replacement with the shared rewrite helpers, and neutralizes the now
dead casts and ownership operations. The fixed-point driver handles cleanup
exposed in later sweeps.

Current-main baseline EIR also exposes a second gate that was not visible in the
original issue description: repeated expressions reload their scalar operands
through distinct `LoadLocal` values, and an invariant `$argc` load is emitted
inside the loop body. Add a conservative `ImmutableLocalLoads` analysis pass:

- consider only `I64` locals with concrete PHP `Int` storage;
- reject slots named by ref-cell, alias, unset, cleanup, or any other indirect
  mutation operation;
- accept zero explicit stores only for initialized function/main inputs, and
  accept one `StoreLocal` in the predecessor-free entry block only when it
  dominates every load by instruction order;
- mark only the proven loads pure.

CSE's existing constant-operand canonicalization can then key equivalent pure
nullary loads by `(LoadLocal, type, slot immediate)`. LICM may treat a proven-pure
`LoadLocal` as the one permitted nullary hoist candidate, moving the load and its
dependent checked-to-int operation together. All mutable and address-taken slots
retain `READS_LOCAL` and remain ineligible.

## Backend Design

Lower the new operations through a cohesive target-aware leaf under
`src/codegen/lower_inst/`:

- AArch64 uses `adds`/`subs` or `mul`+`smulh` for overflow detection.
- x86_64 uses the arithmetic overflow flag from `add`, `sub`, or `imul`.
- Both targets preserve the original operands for the overflow slow path,
  reproduce the existing int-to-double arithmetic, and call the shared PHP
  float-to-int helper.
- The operations stay outside the volatile-safe allowlist because the overflow
  path may call a helper, even though their semantic effects are pure.
- Every emitted instruction carries an aligned explanatory assembly comment.

## Test Plan

Unit tests build EIR directly and validate the function after every rewrite:

- add/sub/mul specialization through an integer cast;
- integer local and ref-cell stores, including current acquire/release scaffolding;
- multiple integer sinks for one producer;
- rejection for output, mixed store, function return, unknown consumer, and
  lifetime-pin acquire;
- pass idempotence and exact result metadata.

Integration tests use runtime-unknown operands so AST folding cannot erase the
targeted EIR:

- repeated buffer index arithmetic becomes one scalar checked-to-int operation
  under CSE while `--no-ir-opt` keeps two boxed checked operations;
- loop-invariant buffer index arithmetic moves from the loop body to its
  preheader under LICM;
- a hot loop compiled with heap debug no longer allocates one Mixed cell per
  integer-only arithmetic evaluation;
- optimized and unoptimized overflow casts agree for `MAX + 1`, `MIN - 1`, and
  `MAX * 3`;
- `echo PHP_INT_MAX + 1` remains a mixed-observing checked operation and keeps
  PHP float promotion.

The existing CSE fixture `($n + 1) * ($n + 1)` remains a negative case: its
outer arithmetic observes the operands dynamically, so its two checked adds must
not be specialized.

## Validation

Run only focused checks:

```bash
cargo build
cargo test --lib checked_int_sink
cargo test --test codegen_tests checked_int_sink
./scripts/check_asm_comments.py <touched-codegen-files>
./scripts/test-linux-x86_64.sh checked_int_sink
./scripts/test-linux-arm64.sh checked_int_sink
git diff --check
```

Finally inspect optimized and unoptimized `--emit-ir`, generated assembly, and a
release build of the issue benchmark to confirm the structural and performance
outcomes.
