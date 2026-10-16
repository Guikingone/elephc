# Scalar local promotion to SSA

- [x] Identify PHP scalar locals whose only observable accesses are ordinary loads and stores.
- [x] Replace eligible local loads and stores with SSA values, adding block parameters at joins and loop back edges.
- [x] Allocate promoted block parameters and branch arguments to registers when safe on every supported target.
- [x] Add IR and executable regression coverage for loops, joins, and excluded volatile or aliased slots.
- [x] Verify optimized and unoptimized behavior, emitted IR and assembly, and focused target coverage.
- [x] Mark the roadmap item complete after all checks pass.

## Implementation notes

Promotion runs after lowering and before register allocation in the EIR pass driver. Only ordinary PHP locals with single-word, non-heap scalar storage qualify. An address escape, implicit scope access, reference operation, exceptional handler edge, or nonstandard local-slot operation excludes the affected slot or function. A must-definition analysis keeps loads that can observe an uninitialized slot in memory.

The pass uses existing EIR block parameters for values live into CFG joins and loop headers. Branches pass the predecessor's current SSA value. Register allocation then runs on the transformed graph, and edge copies must support register homes as well as stack homes.

Unused block parameters keep distinct stack homes because edge copies still write them after their scalar uses are folded away. Functions with exception handlers use stack allocation until liveness includes implicit exception edges, including for scalar return parameters introduced by inlining.
