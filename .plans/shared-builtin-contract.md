# Shared builtin contract and runtime dispatch

- [x] Add a dependency-leaf `elephc-builtin-contract` crate with the canonical PHP builtin catalog.
- [x] Move neutral name, signature, default, by-reference, visibility, documentation, and requirement metadata into the shared catalog.
- [x] Make the compiler registry join AOT-only checker/EIR semantics onto shared contract IDs.
- [x] Make the Magician registry join eval-only direct/value hooks onto the same shared contract IDs.
- [x] Replace parity allowlists with explicit backend support records and validated transitional signature overrides.
- [x] Add a typed `RuntimeBuiltinId` boundary for runtime-helper-backed builtins.
- [x] Add generated-runtime C-ABI wrappers that accept boxed eval cells and dispatch supported `RuntimeBuiltinId` values.
- [x] Migrate type and math builtins to shared runtime dispatch where their dynamic contracts agree.
- [x] Migrate string builtins to shared runtime dispatch where their dynamic contracts agree.
- [x] Migrate array builtins to shared runtime dispatch where their dynamic contracts agree.
- [x] Migrate I/O builtins to shared runtime dispatch where their dynamic contracts agree.
- [x] Keep explicit Magician adapters only for by-reference/lvalue handling, dynamic callables, reflection, eval declarations, and other genuinely interpreter-specific behavior.
- [x] Regenerate builtin documentation, run focused AOT/Magician/parity tests, and audit every catalog entry and implementation binding.

## Objective

Elephc currently has two declarative builtin registries. The compiler registry is the
single source for static checking, EIR lowering, ownership, effects, runtime requirements,
and generated documentation. Magician independently repeats PHP-visible names, parameter
names, defaults, variadic shape, by-reference flags, extension classification, and runtime
dispatch hooks. Integration tests compare both registries and carry explicit exception
lists for static-only, eval-only, and intentionally extended signatures.

The target architecture has one dependency-neutral catalog and two independent execution
bindings:

```text
                        elephc-builtin-contract
                     canonical PHP surface metadata
                         /                  \
          compiler AOT binding       Magician eval binding
          checker + typed EIR        direct/value hooks
                         \                  /
                 generated runtime C-ABI dispatch
                    for shareable runtime helpers
```

Magician must remain independent from the compiler crate. The shared contract must not
depend on compiler AST, type-checker, EIR, codegen, target ABI, Magician EvalIR, or runtime
cell implementations.

## Shared crate

Create `crates/elephc-builtin-contract` as a normal workspace library and dependency leaf.
It owns:

- a stable `BuiltinId` for every catalog entry;
- canonical PHP name and case-insensitive lookup;
- area and surface kind (`Function`, `LanguageConstruct`, `DedicatedSyntax`, or
  `PreludeProvided`);
- parameter names, neutral PHP type shapes, defaults, passing mode, variadic shape, and
  arity overrides;
- return declaration and by-reference return flag;
- PHP/extension/internal visibility;
- neutral bridge/runtime capability requirements;
- documentation summary, examples, manual link, and deprecation metadata;
- explicit backend support status and a reason for intentionally unsupported entries;
- narrowly scoped transitional signature overrides required to preserve current behavior.

The catalog data is static and deterministic. Lookup rejects duplicate names and IDs. A
backend implementation registration without a catalog contract, or more than one binding
for the same backend and ID, is a hard initialization/test failure.

Backend support is derived from implementation bindings whenever possible. An intentional
absence is represented as `Unsupported { reason }`, not a global name allowlist. Transitional
signature differences are stored next to the affected contract with a reason, so both
registries still derive their complete signature from the shared crate without silently
changing behavior.

## Compiler binding

The compiler `BuiltinSpec` becomes an AOT binding keyed by `BuiltinId`. It retains only
compiler-specific behavior:

- optional checker and result-type refinement hooks;
- backend-neutral EIR lowering or typed runtime target;
- effects and ownership contracts consumed by EIR;
- source-dependent requirement resolution when it needs compiler expressions/types;
- callable policy and target support details that cannot be represented neutrally.

Name, area, parameters, defaults, arity, declared return, visibility, and documentation are
read from the shared contract. Existing registry consumers should receive an assembled view
so the migration does not duplicate lookup or change behavior.

Compiler-resident constructs remain dedicated lowering paths, but their PHP surface belongs
to the shared catalog through `BuiltinKind`. Prelude-provided functions likewise remain
implemented by their preludes while using the same canonical catalog identity.

## Magician binding

`EvalBuiltinSpec` becomes an eval implementation binding keyed by the same `BuiltinId`. It
retains:

- direct expression hooks where source-order/lvalue behavior requires unevaluated EvalIR;
- evaluated-value hooks for normal dynamic callable dispatch;
- interpreter-specific availability predicates, such as registered regex capability;
- the Magician home-file path used by generated documentation.

Parameter binding, defaults, required count, variadic shape, extension visibility, and
documentation metadata come from the shared contract. Magician must continue to compile and
test without depending on the `elephc` compiler crate.

## Runtime builtin boundary

Introduce `RuntimeBuiltinId` for the subset of builtin contracts whose implementation can be
executed through the generated runtime using boxed `Mixed` cells. This ID is distinct from
the PHP name and from compiler-only EIR operations. The compiler maps an eligible
`RuntimeFnId`/contract pair onto the runtime builtin ID; Magician maps the same `BuiltinId`
onto it instead of a duplicate Rust algorithm.

The generated runtime exposes a versioned C ABI with explicit ownership:

```text
__elephc_runtime_builtin_call_v1(
    runtime_builtin_id,
    boxed_args,
    arg_count,
    eval_context,
    result_out,
) -> status
```

The exact ABI may use an argument-array cell if that preserves the current target helpers
more naturally. It must document who owns every input and result, distinguish success,
runtime fatal, and pending throwable, validate arity/type at the appropriate layer, and use
target-aware generated-runtime helpers. Unknown IDs fail closed.

Dynamic source is opaque at compile time, so enabling Magician must emit/register every
runtime builtin wrapper promised by its active capability set or provide a capability table
that prevents unavailable IDs from appearing in eval lookup. Optional bridge/native
dependencies must remain independently detected and must not be pulled in merely because
Magician is linked.

## Migration order

1. Types and math: prioritize scalar predicates, conversions, and numeric operations already
   exposed through `RuntimeValueOps`; keep `settype` and other by-reference operations on
   explicit adapters initially.
2. Strings: migrate helper-backed scalar operations first. Keep algorithms whose AOT helper
   has a typed/unboxed ABI on an adapter until the boxed wrapper is proven equivalent.
3. Arrays: migrate non-mutating operations before COW and callback surfaces. By-reference
   sort/mutation functions and callback-driven walkers remain explicit until lvalue/writeback
   semantics are shared.
4. I/O: migrate stateless/path helpers before resources, streams, filters, sockets, output
   buffering, and operations with Magician-owned resource metadata.

Each batch removes the superseded Magician implementation rather than layering a shared call
under dead duplicate code.

## Required invariants

- No behavior change while extracting and joining the catalog.
- Magician has no dependency on the compiler crate.
- PHP name lookup is case-insensitive and leading-backslash tolerant on both paths.
- Named/default/variadic/by-reference behavior remains byte-for-byte compatible with current
  diagnostics and evaluation order.
- Strict-PHP hides the same extension set in AOT and dynamic eval.
- Runtime capabilities and optional bridges remain fail-closed and selectively linked.
- Boxed `Mixed`, ownership, COW, throwable, and cleanup behavior remains balanced.
- Every runtime wrapper supports macOS AArch64, Linux AArch64, and Linux x86_64 through the
  existing target-aware runtime/codegen boundary.
- No implementation is selected by a PHP string after contract lookup; backends dispatch on
  typed IDs.
- Generated docs read the shared contract plus backend implementation status, never merge two
  independent signature catalogs.

## Validation

Use focused checks during each batch:

- contract registry unit tests for ID/name uniqueness, deterministic ordering, signature
  profiles, and support records;
- compiler builtin registry/parity unit tests;
- `cargo test --test builtin_parity_tests` while transitional parity checks remain;
- focused `cargo test -p elephc-magician --lib` filters for migrated families;
- focused AOT codegen/error tests for representative normal, named, variadic, by-reference,
  throwable, ownership, and strict-PHP calls;
- target-sensitive focused Linux tests when a generated runtime wrapper or ABI path changes;
- `cargo build --example gen_builtins` and the complete builtin documentation audit chain;
- `cargo build`, focused rustfmt checks, assembly-comment checks for touched emitters, and
  `git diff --check` before commits.

Completion requires a final machine-audited join over every shared contract proving that each
backend has exactly one implemented or explicitly unsupported status, no old signature source
still participates in lookup, all four requested builtin families have migrated where their
runtime contracts are shareable, and every remaining Magician adapter has a documented dynamic
reason.
