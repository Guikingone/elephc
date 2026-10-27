# Loop counter integer fast paths

- [x] Recover task 2f4c2d30 and plan 199f174d; confirm Part A is on the base branch.
- [x] Part B: give dead-between-region nested counters independent bindings.
- [x] Part C: avoid unobservable seed overflow on descending counters.
- [x] Part D: sink cast-only index locals without changing intermediate float semantics.
- [x] Part E: guard integer loop versions and retain the generic fallback.
- [x] Validate focused EIR, runtime, ownership and boundary tests; update docs and example.

Code Journal task: `2f4c2d30`. Design plan: `199f174d`.

The implementation must preserve PHP overflow and integer/float comparison semantics.
Binding decisions stay keyed to the original AST nodes; loop duplication belongs in
EIR, after lowering. Aliased, dynamically addressed and exceptional local scopes
are excluded conservatively. Every supported target shares the same EIR changes.

Region analysis treats a for initializer as the start of a region ending at that
loop's exit. A statement assignment can start a region lasting to the end of its
statement list. Regions never nest for the same name, and every occurrence must
belong to a region. Incoming bindings and escaping names remain shared.

Implementation and validation completed on `feat/task-2f4c2d30`, based on
`777b1024e2`. Review and CI publication are tracked separately in Code Journal.

Part B propagates unknown writers and possible reference escapes through the
numeric dependency graph. Declared numeric properties and typed buffer reads
reuse authoritative checker types. Part C uses an EIR seed guard, preserving the
single lowering of each AST node. Part D follows reaching writers and can remove
a fast copy's numeric store while preserving generic boxed storage. Part E adds
bounded EIR copies with integer entry guards and final-value synchronization.

Validation: warning-free `cargo build`; six numeric-region tests; six counter,
seed and index EIR tests; five codegen tests with PHP 8.5.12, both IR modes and
strict heap checking; assembly emission for all five supported targets; existing
counter-binding/static-proof regressions and the Fiber frame-budget regression.
The combined counter/index allocation fixture has eight allocations and eight
frees at both 10 and 1000 iterations. `git diff --check` passes.

A separate existing returned-string concat leak, also present with EIR
optimization disabled, is recorded as Code Journal task `f00b3705`.
