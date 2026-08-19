# Whole-program declaration reachability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make unused declaration elimination a single, conservative compiler pass so every prelude (PDO, image, hash, tz, web, …) and ordinary user code pay only for what is reachable, instead of each prelude inventing its own prune.

**Architecture:** After AST DCE and before EIR lowering, run one whole-program reachability pass. A shared usage/hazard scanner builds a declaration graph. A fixed-point marks reachable functions, classes, methods, enums, interfaces, traits, and externs from explicit roots. The pass then rewrites the AST and reconciles `CheckResult` so lowering and codegen never see dead declarations. Existing one-off prunes (web session functions, OPcache per-function inject) become clients of this pass, not parallel implementations.

**Tech Stack:** Rust, existing AST (`src/parser/ast.rs`), AST optimizer (`src/optimize/`), `CheckResult` (`src/types/result.rs`), EIR lowering (`src/ir_lower/`), codegen tests (`tests/codegen/`).

## Global Constraints

- Supported targets stay first-class: `macos-aarch64`, `linux-aarch64`, `linux-x86_64`. No ARM64-only lowering or link assumptions.
- PHP-derived syntax must stay PHP-compatible. Dropping a declaration must not change the result of any program that can observe that declaration (static call, dynamic call, `function_exists` / `method_exists` / `class_exists` with a literal, first-class callable, `eval`, `unserialize`, Reflection, or declared-class/interface enumeration).
- Soundness over precision. A false positive (keeping extra code) is allowed. A false negative (dropping something observable) is a blocker.
- EIR is the only backend. The pass must run before `ir_lower::lower_program_*` and before `--emit-ir`.
- `ir_lower` prefers `check_result.classes[name].method_decls` over the AST method list. Pruning the AST without reconciling `CheckResult` is a silent no-op.
- Never run `cargo fmt`. Never add `Co-Authored-By`. Keep new Rust files under the 500-line cohesion rule; split by responsibility.
- Every new `.rs` file needs a `//!` preamble. Every function needs a `///` docblock.
- Local verification is focused tests only. Do not run the full suite unless the user asks.
- Plans and code comments are English. ROADMAP only gains a new item under the current planning version (`v0.26.x`); do not rewrite history.

---

## Task checklist

- [x] Task 1: Shared usage/hazard scanner and declaration graph types
- [x] Task 2: Fixed-point reachability + AST prune + CheckResult reconcile
- [x] Task 3: Wire the pass into the pipeline after DCE
- [x] Task 4: Prelude inventory and `--with-<crate>` force-keep
- [x] Task 5: Method-level pruning on live classes, including vtable rebuild
- [x] Task 6: Retire the web-only function prune; keep OPcache inject-time pay-for-use
- [x] Task 7: Validate linker dead-strip boundaries and keep macOS user assembly intact
- [x] Task 8: Docs, ROADMAP, and README

---

## Why this exists

Today the compiler already has three incomplete, inconsistent “pay for what you use” mechanisms:

| Mechanism | What it does | What it misses |
|---|---|---|
| Prelude `inject_if_used` | If the program never mentions PDO / hash / tz / image / `var_export`, the whole prelude stays out | One `new PDO` injects the entire 7k-line PDO surface |
| AST DCE (`eliminate_dead_code`) | Drops unreachable *statements* inside bodies | Never deletes `FunctionDecl` / `ClassDecl` / methods |
| One-off prunes | `--web` has `prune_unreachable_prelude_functions`; OPcache injects per function; SPL/datetime/reflection methods lower on demand | PDO, image, hash, tz, user functions, live-class methods have no equivalent |

EIR lowering then emits every remaining declaration:

- `src/ir_lower/program/function_declarations.rs` — every `FunctionDecl`
- `src/ir_lower/program/class_methods.rs` — every method with a body
- `src/codegen/block_emit.rs` `emit_module` — every function, method, and closure

A used class also pins every method through its vtable (`_class_vtable_*` takes the address of each slot). Linker `--gc-sections` / `-dead_strip` can drop unreferenced *runtime* `__rt_*` helpers, but not unused PDO methods that the vtable still names.

The `--web` prune in `src/web_prelude.rs` is the prototype to generalize. It is function-only, runs at inject time (before name resolution), does not record method or class names, and treats any dynamic call as “keep everything”. That is the right conservatism. It is the wrong scope.

---

## Design

### Where the pass lives

New module `src/optimize/reachability/`, exported from `src/optimize.rs` as:

```rust
pub fn prune_unreachable_declarations(
    program: Program,
    check_result: &mut crate::types::CheckResult,
    options: reachability::PruneOptions<'_>,
) -> Program
```

Do **not** put this in individual prelude crates. The asymmetry is exactly that each prelude grew a private policy.

Do **not** put this in EIR. Dead methods should never be lowered. `--emit-ir` must show the pruned program.

### When it runs

Insert it in `src/pipeline.rs` **after** `optimize::eliminate_dead_code` and **before** `--emit-ir` / `ir_lower`:

```text
opt-prop → opt-post → opt-norm → dce → decl-reach → ir-lower
```

Why after DCE, not before typecheck:

- Typecheck still sees the full AST, so unused user functions still get diagnostics.
- DCE has already deleted `if (false) { unused(); }` and similar, so those calls do not keep declarations alive.
- `--check` exits before DCE today and should stay that way. Declaration prune is a codegen-size pass, not a diagnostic pass.

Do **not** prune before typecheck in v1. A later optional “trusted prelude bodies skip the checker” can be a follow-up; it is not required to close the assembler gap.

Add a progress phase `"decl-reach"` / `"Pruning unreachable declarations"` in `src/progress.rs`.

### What is a declaration

The pass understands these AST kinds as graph nodes:

- free functions (`StmtKind::FunctionDecl`)
- classes / enums / interfaces / traits (`ClassDecl`, `EnumDecl`, `InterfaceDecl`, `TraitDecl`)
- methods (instance and static), keyed `(class_key, method_key, is_static)`
- extern functions / extern classes / extern globals
- packed classes (keep if referenced; do not invent a packed-specific policy)

Closures are not declarations. They stay attached to the surviving parent function.

Property-init thunks (`_class_propinit_*`) stay iff their class stays.

### Roots

A declaration is a root if any of the following holds:

1. It is referenced from top-level executable statements (the implicit `main`).
2. It is `#[Export]` (already collected by `exports::collect`; pass those names in).
3. It is a compiler-required bootstrap symbol for the current mode (the `--web` handler wrapper, session auto-start core, exception wrap). These are listed explicitly, not guessed.
4. Its injected prelude group is force-kept (`--with-pdo`, `--with-tz`, `--with-image`); `--with-eval` separately keeps every user-visible declaration. `--with-crypto` force-links its bridge but does not force-inject the hash prelude.
5. `--with-eval` / an actual `eval()` call / Magician bridge requirement: treat *all* user-visible declarations as roots (see Hazards).

`--web` is **not** a force-keep. `--web` must still drop unused `session_*` wrappers. `--with-web` is an alias for `--web`, not “keep the whole session API”.

### Edges

From a live declaration body (and from top-level statements), record:

| Source shape | Edge |
|---|---|
| `foo()` / first-class `foo(...)` | function `foo` |
| call to an include-loaded `FunctionVariantGroup` | every concrete function variant selectable by its runtime dispatcher |
| `new C` / `C::class` / `instanceof C` / `catch (C)` / type hints on live callables | class/interface/enum `C` |
| `new static(...)` inside a live method | the lexical class and every instantiable descendant, including each possible constructor body and argument signature |
| `$obj->m()` / `C::m()` / first-class `C::m(...)` / `[$obj, 'm']` / `['C', 'm']` | method `(C, m)` plus class `C` |
| `foreach`, `count()`, object offset access, `json_encode()`, iterator builtins, `new IteratorIterator(...)` | behavioral edges to the compiler-invoked `Iterator` / `IteratorAggregate` / `Countable` / `ArrayAccess` / `JsonSerializable` methods; class-qualified for known receivers and method-name wildcard for opaque or recursively encoded receivers |
| Registry builtin parameter with a callable type or structural callback-slot metadata | callable edge from the argument bound by the shared call-argument planner; unknown/spread values widen callable hazards; registry validation checks every declared slot and a registry-wide test pins the current callback-consuming set |
| attributes on a live declaration, including fixed and variadic parameters | attribute class construction plus declaration edges from every attribute argument |
| `function_exists('foo')` / `is_callable('foo')` / `call_user_func('foo')` with a **string literal** | function `foo` |
| `method_exists($x, 'm')` / `class_exists('C')` / `property_exists('C', 'p')` with a **string literal** | method `m` on every live type that could be `$x`, or class `C`; named arguments are normalized before selecting the target |
| `elephc_pdo_*()` / any extern call | that extern function |
| `extends` / `implements` / trait use on a live class | parent, interfaces, used traits |
| `parent::m()` / `$this->m()` inside a live method | implementation owner selected by `ClassInfo.method_impl_classes` / `static_method_impl_classes` |

Method bodies indexed for classes and enums come from the checker's flattened
`method_decls`, not only from methods written directly in the class AST. This is
required for trait-imported and aliased methods because that same flattened list
is what EIR lowering emits.

Local receiver refinement is a conservative may-analysis. Reassigning a variable to a known
class unions that class with every class already recorded for the variable; it never overwrites
the previous fact. A typed local contributes its concrete named class/interface members even
when its initializer is `null`; generic `mixed`/`object`/`callable`/`array` hints do not refine
the receiver. An assignment from an opaque value permanently forgets the receiver for the
remainder of that declaration scan, so later calls become wildcard method edges. `foreach`
key/value bindings, `list()`/array destructuring, reference assignments (both alias endpoints),
`global`, static-local bindings, catch variables, and increment/decrement writes also invalidate
the affected local. This deliberately gives up precision rather than selecting one control-flow
path and dropping a method reachable through another.

Interprocedural aliases use the same conservative rule. A `global $name` in a behaviorally
reachable function or method makes calls through `$name` wildcard across scanned scopes. Direct
function, method, static-method, and constructor calls use checker-validated signatures plus the
shared call-argument planner to forget the actual caller variable passed to any by-reference
parameter, including named arguments and cases where the callee parameter has a different name.
`$GLOBALS['literal']` aliases that literal top-level variable name; a computed `$GLOBALS[$key]`
makes every tracked variable name opaque. Dynamic expression calls remain intentionally broad:
they may denote a function, closure, or callable array, so retaining methods of live classes is a
precision cost rather than a correctness defect.

This `$GLOBALS` rule is a declaration-retention model, not an implementation of PHP's special
runtime alias storage. Native AOT lowering does not currently make `$GLOBALS['name']` and `$name`
the same runtime cell, so coverage for this pass asserts that the candidate method symbol survives
instead of asserting unsupported runtime output. If `$GLOBALS` storage semantics are implemented,
the conservative reachability edge is already in place and the fixture can gain a runtime assertion.

`__elephc_initialize_pdo_statement()` is an explicit compiler-internal coupling: its backend
lowering directly calls `PDOStatement::__elephcInitialize()`, so the scanner records that method
edge even though it is not visible as an AST call. Keep this edge synchronized with the PDO
initializer lowering until the dependency is represented in builtin registry metadata.

Name keys use `crate::names::php_symbol_key` (case-insensitive, canonical after name resolution).

The scan must be exhaustive on `StmtKind` / `ExprKind`. Follow `src/pdo_prelude/detect.rs`: no wildcard arm. Adding an AST node must fail to compile until the scanner is updated.

### Hazards (keep more, never drop too much)

Hazards are boolean flags on the scan result. They do **not** disable the pass. They widen the keep-set.

| Hazard | Trigger examples | Keep |
|---|---|---|
| `dynamic_function` | `eval()`, `$fn()`, `call_user_func($x)`, `function_exists($x)`, `is_callable($x)`, `ExprCall`, `ClosureCall` used as a name lookup | every remaining free function |
| `dynamic_method` | `$obj->$m()`, `$class::$m()`, computed `ExprCall` / `ClosureCall`, `method_exists($o, $x)`, `ReflectionMethod` / `ReflectionClass` method APIs, `call_user_func([$o, $x])` | every method of every **live** class |
| `dynamic_class` | `new $c`, `class_exists($x)`, `property_exists($x, 'p')`, `unserialize()`, `get_declared_classes()`, `ReflectionClass` by variable name | every class / interface / enum |
| `eval_bridge` | `eval` present, `--with-eval`, Magician already required | all user-visible declarations (functions, classes, methods) |

Literal introspection is **not** a hazard. `function_exists('pdo_drivers')` is a normal edge to `pdo_drivers`.

Do not invent hazards for functions that the AOT builtin registry does not expose. In particular,
`get_defined_functions()` and `get_class_methods()` are not supported AOT builtins today; the
checker rejects them before declaration reachability runs. If they are added later, their registry
work must add the corresponding reachability hazards and tests in the same change.

If a live class declares `__call` / `__callStatic`, that is not automatically `dynamic_method` for *other* classes. It does mean the class itself must keep `__call` / `__callStatic`.

`unserialize` is `dynamic_class`. PHP can reconstruct any declared class from the payload. Dropping `Pdo\Sqlite` would change `unserialize('O:9:"Pdo\\Sqlite":...')`.

### What v1 prunes

**Always (no hazard):**

- Unused free functions, including unused prelude helpers (`pdo_drivers` if never referenced).
- Unused classes / enums / interfaces / traits and all of their methods. This is the cheap PDO win for `Pdo\Dblib`, `Pdo\Firebird`, `PDORow`, unused image backends, unused hash helpers, unused web session wrappers.

**When `dynamic_method` is false:**

- Unused methods on a *live* class, including unused `PDO::beginTransaction` when the program only calls `query`.

**When `dynamic_method` is true:**

- Keep every method of every live class. Still drop unused sibling classes.

**Always keep on a live instantiated class, even if never named:**

- `__construct` if any `new` of that class (or a subclass) is reachable
- `__destruct`, `__clone`
- `__toString` if the class is used in a string context or `echo`
- `__get` / `__set` / `__isset` / `__unset` / `__invoke` / `__serialize` / `__unserialize` / `__sleep` / `__wakeup` / `__debugInfo` / `__set_state` if the matching operation exists **or** the class is live and we cannot prove the operation is absent. v1: if the class is instantiated, keep the whole magic-method set that the class actually declares.
- Every method required by a live interface (`IteratorAggregate::getIterator`, `ArrayAccess::*`, …)
- Every method that a live subclass still dispatches to through the vtable

**Never prune:**

- `main` / top-level
- Packed-class layouts that are still referenced
- Runtime `__rt_*` helpers (already handled by runtime dead-strip)
- Builtin catalog entries (they are not AST declarations)

### CheckResult reconcile (mandatory)

After the AST rewrite, mutate `CheckResult` in place:

1. Drop `functions` / `function_attribute_*` entries only when their names belong to indexed
   AST `FunctionDecl` declarations that are not kept. Preserve checker/builtin-only entries.
2. Drop `classes` / `enums` / `interfaces` only when the symbol is backed by an indexed AST
   class-like declaration that is not kept. Synthetic checker entries such as `Exception`,
   `Error`, `TypeError`, Reflection, SPL, and DateTime metadata are outside the prune domain.
3. For a kept AST-backed class, drop `method_decls`, `methods`, `static_methods`,
   visibility/impl maps, and callable-return maps for methods that are not kept. Do not rewrite
   the method metadata of checker-only classes.
4. Rebuild `vtable_methods` + `vtable_slots` and the static counterparts from the methods that remain, preserving inheritance order of the *kept* methods. Compact slots are fine: every later consumer reads the updated `ClassInfo`.
5. Drop AST-declared `extern_functions` that no remaining AST node calls. Preserve synthetic
   extern metadata not represented by the declaration index. Recompute `required_libraries` as the union of:
   - libraries still named by remaining `extern_functions`
   - libraries still required by remaining builtin call sites after shared named/spread argument normalization (`builtin_call_types` / program walk)
   Do **not** keep `elephc_pdo` only because the prelude once declared the extern block.
6. Leave `warnings` and `throw_access_sites` alone. They already fired.

`ir_lower::program::class_methods::lower_class_like_methods` must then see a `method_decls` list that matches the AST. Add a debug-only assertion (cfg(debug_assertions)) that every `method_decls` entry still exists in the AST for that class.

### Prelude inventory

Each `inject_if_used` records what it added into a `PreludeInventory` threaded through `pipeline::compile`:

```rust
pub struct PreludeInventory {
    pub groups: HashMap<String, PreludeGroup>,
}

pub struct PreludeGroup {
    pub id: &'static str, // "pdo", "tz", "image", "hash", "web", "opcache", "var_export", "list_id", "version"
    pub functions: HashSet<String>,
    pub classes: HashSet<String>,
    pub methods: HashSet<(String, String, bool)>,
    pub externs: HashSet<String>,
}
```

`--with-pdo` does **not** skip the pass. It marks every name in group `"pdo"` as a root, so `eval('new PDO')` still works when the user asked for a forced surface.

`--with-eval` marks every user-visible declaration as a root.

Injectors stay responsible for *whether* to inject the prelude at all (`inject_if_used`). The new pass is responsible for *how much* of an injected prelude survives.

### Relationship to existing one-off prunes

- **`--web` `prune_unreachable_prelude_functions`:** delete it once Task 3+5 have equivalent tests green. Keep the inject-time skip of `__ElephcCallableSessionHandler` (that avoids parsing a huge unused block). The general pass is the backstop for `session_start` vs `__elephc_session_start_core`.
- **OPcache per-function inject:** keep it. Not tokenizing unused OPcache functions is cheaper than injecting them and pruning later. The general pass is the backstop if a helper is injected as a dependency and then becomes dead.
- **SPL / datetime / reflection on-demand lowering:** keep it. Those methods are not AST declarations; they are synthetic. Do not try to express them in this pass.
- **Runtime `__rt_*` dead-strip:** keep it. Orthogonal. This pass reduces *user* assembly and the references that pin bridge objects.

### Linker dead-strip boundary (Task 7, after the prune)

Linker dead stripping remains a safety net, not a substitute for the AST pass.
Linux user functions already use `.section .text.<name>` from
`Emitter::label_global`. On macOS, only the runtime object opts into
`.subsections_via_symbols`: generated user metadata contains contiguous tables,
and callable descriptors may reach internal labels through data rather than a
direct call relocation. Treating every global user symbol as an independent atom
can therefore leave a descriptor pointing at stripped code.

The macOS user object and cdylib stay intact. Declaration reachability removes
dead user declarations before assembly, while the existing runtime-object path
continues to discard independent `__rt_*` helpers.

### Non-goals (v1)

- Inlining or interprocedural DCE of statement bodies (already exists separately).
- Pruning unused *properties* / class constants (data, not the assembler blow-up).
- Changing prelude *injection* thresholds (`inject_if_used` stays).
- A CLI flag to turn the pass off. Add `--no-decl-prune` later only if bisect needs it.
- Typecheck-time skip of trusted prelude bodies.
- Whole-program EIR function DCE. If the AST + CheckResult are pruned, EIR never sees the dead functions.

---

## File map

Create:

- `src/optimize/reachability.rs` — module root + `PruneOptions` + `prune_unreachable_declarations`
- `src/optimize/reachability/usage.rs` — exhaustive AST scan (refs + hazards)
- `src/optimize/reachability/graph.rs` — declaration index + edges + fixed point
- `src/optimize/reachability/prune.rs` — AST rewrite
- `src/optimize/reachability/reconcile.rs` — `CheckResult` mutation + vtable rebuild
- `src/optimize/reachability/inventory.rs` — `PreludeInventory` / `PreludeGroup`
- `src/optimize/reachability/tests.rs` — unit tests (scanner, graph, prune, reconcile)
- `tests/codegen/optimizer/declaration_reachability.rs` — end-to-end assembler and runtime tests

Modify:

- `src/optimize.rs` — `mod reachability;` and re-export
- `src/pipeline.rs` — inventory thread, `decl-reach` phase, pass invocation
- `src/progress.rs` — phase label
- `src/pdo_prelude.rs`, `src/tz_prelude.rs`, `src/image_prelude.rs`, `src/hash_prelude.rs`, `src/var_export_prelude.rs`, `src/list_id_prelude.rs`, `src/opcache_prelude/injection.rs`, `src/web_prelude.rs`, `src/version_prelude.rs` — fill inventory (and web: delete function prune once replaced)
- `src/ir_lower/program/class_methods.rs` — debug assertion that `method_decls` ⊆ AST
- `src/codegen_support/emit.rs`, `src/codegen/mod.rs` / user-asm finalize — preserve the runtime-only macOS `.subsections_via_symbols` boundary
- `tests/codegen/cli.rs` — existing web prune test must keep passing
- `tests/codegen/mod.rs` — register the new test file
- `docs/internals/the-optimizer.md`, `docs/compiling/compilation-pipeline.md`, `docs/compiling/optimization.md`, `README.md`, `ROADMAP.md`

---

## Interfaces

These names are authoritative for later tasks. Do not invent synonyms.

```rust
// src/optimize/reachability.rs

pub struct PruneOptions<'a> {
    pub inventory: &'a PreludeInventory,
    pub forced_groups: &'a HashSet<String>,
    pub exported_functions: &'a HashSet<String>,
    pub eval_forced: bool,
}

pub fn prune_unreachable_declarations(
    program: Program,
    check_result: &mut crate::types::CheckResult,
    options: PruneOptions<'_>,
) -> Program;

// src/optimize/reachability/inventory.rs

#[derive(Clone, Debug, Default)]
pub struct PreludeInventory {
    pub groups: HashMap<String, PreludeGroup>,
}

#[derive(Clone, Debug, Default)]
pub struct PreludeGroup {
    pub id: String,
    pub functions: HashSet<String>,
    pub classes: HashSet<String>,
    pub methods: HashSet<(String, String, bool)>,
    pub externs: HashSet<String>,
}

impl PreludeInventory {
    pub fn new() -> Self;
    pub fn group_mut(&mut self, id: &str) -> &mut PreludeGroup;
    pub fn record_program(&mut self, id: &str, prelude: &[Stmt]);
}

// src/optimize/reachability/usage.rs

#[derive(Clone, Debug, Default)]
pub struct Usage {
    pub functions: HashSet<String>,
    pub classes: HashSet<String>,
    pub methods: HashSet<(String, String, bool)>,
    pub externs: HashSet<String>,
    pub hazards: Hazards,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Hazards {
    pub dynamic_function: bool,
    pub dynamic_method: bool,
    pub dynamic_class: bool,
}

pub fn scan_program(program: &[Stmt]) -> Usage;
pub fn scan_stmt(stmt: &Stmt) -> Usage;

// src/optimize/reachability/graph.rs

#[derive(Clone, Debug, Default)]
pub struct Reachability {
    pub functions: HashSet<String>,
    pub classes: HashSet<String>,
    pub methods: HashSet<(String, String, bool)>,
    pub externs: HashSet<String>,
    pub hazards: Hazards,
}

pub fn compute(
    program: &[Stmt],
    options: &PruneOptions<'_>,
) -> Reachability;
```

Injector signature change (every prelude, same shape):

```rust
// before
pub fn inject_if_used(program: Program, force: bool) -> Program

// after
pub fn inject_if_used(
    program: Program,
    force: bool,
    inventory: &mut crate::optimize::reachability::PreludeInventory,
) -> Program
```

`force` still controls *injection*. Inventory recording happens only when the prelude is actually prepended. `pipeline.rs` then inserts the group id into `forced_groups` when `with_crates` contains that crate.

---

### Task 1: Shared usage/hazard scanner and declaration graph types

**Files:**
- Create: `src/optimize/reachability.rs`
- Create: `src/optimize/reachability/inventory.rs`
- Create: `src/optimize/reachability/usage.rs`
- Create: `src/optimize/reachability/graph.rs`
- Create: `src/optimize/reachability/tests.rs`
- Modify: `src/optimize.rs` (add `pub(crate) mod reachability;`)

**Interfaces:**
- Consumes: `crate::parser::ast::{Program, Stmt, StmtKind, Expr, ExprKind, ...}`, `crate::names::php_symbol_key`
- Produces: `Usage`, `Hazards`, `scan_program`, `PreludeInventory`, `compute` (can return empty keep-sets until Task 2 fills prune)

- [x] **Step 1: Write failing unit tests for the scanner**

In `src/optimize/reachability/tests.rs`, cover at least:

```rust
#[test]
fn scan_records_function_and_method_and_class() {
    let program = parse(
        "<?php
        $pdo = new PDO('sqlite::memory:');
        $pdo->query('select 1');
        PDO::getAvailableDrivers();
        pdo_drivers();
        ",
    );
    let usage = scan_program(&program);
    assert!(usage.classes.contains(&php_symbol_key("PDO")));
    assert!(usage.methods.contains(&(php_symbol_key("PDO"), php_symbol_key("query"), false)));
    assert!(usage.methods.contains(&(php_symbol_key("PDO"), php_symbol_key("getAvailableDrivers"), true)));
    assert!(usage.functions.contains(&php_symbol_key("pdo_drivers")));
    assert!(!usage.hazards.dynamic_function);
}

#[test]
fn scan_literal_function_exists_is_a_reference_not_a_hazard() {
    let program = parse("<?php echo function_exists('pdo_drivers') ? 'y' : 'n';");
    let usage = scan_program(&program);
    assert!(usage.functions.contains(&php_symbol_key("pdo_drivers")));
    assert!(!usage.hazards.dynamic_function);
}

#[test]
fn scan_eval_is_dynamic_function() {
    let program = parse("<?php eval('echo 1;');");
    let usage = scan_program(&program);
    assert!(usage.hazards.dynamic_function);
}

#[test]
fn scan_dynamic_method_and_unserialize_hazards() {
    let program = parse(
        "<?php
        $m = 'query';
        $pdo->$m();
        unserialize($argv[1]);
        ",
    );
    let usage = scan_program(&program);
    assert!(usage.hazards.dynamic_method);
    assert!(usage.hazards.dynamic_class);
}
```

`parse` helper: `lexer::tokenize` + `parser::parse`. Do not go through the full pipeline.

- [x] **Step 2: Run the unit tests and confirm they fail to compile / fail**

Run:

```bash
cargo test --lib optimize::reachability::tests -- --nocapture
```

Expected: module missing or assertions fail.

- [x] **Step 3: Implement the scanner**

Port the walk from `src/web_prelude/usage.rs`, then **extend** it. The web scanner is incomplete on purpose (functions only). The new scanner must additionally record:

- `NewObject` / `NewScopedObject` / `InstanceOf` named targets / `catch` types / class type hints
- `MethodCall` / `NullsafeMethodCall` / `StaticMethodCall` method names
- `FirstClassCallable` function, instance method, and static method
- array-callable literals `[$obj, 'm']` and `['C', 'm']` (string elements only)
- `function_exists` / `is_callable` / `call_user_func` / `call_user_func_array` / `method_exists` / `class_exists` literal vs non-literal
- `get_declared_classes` / `get_declared_interfaces` / `get_declared_traits` as class hazards
- `eval` as `dynamic_function`
- `unserialize` as `dynamic_class`
- `ExprCall` / variable-function / dynamic method / `new $c` as the matching hazard
- Reflection class names (`ReflectionClass`, `ReflectionMethod`, `ReflectionFunction`, `ReflectionObject`) as `dynamic_method` + `dynamic_function` when constructed or used. v1 may treat any reference to those classes as both hazards. That is conservative and acceptable.

Match `StmtKind` / `ExprKind` exhaustively.

Implement `PreludeInventory::record_program` by walking a prelude AST and recording every `FunctionDecl`, `ClassDecl` (plus methods), `EnumDecl`, `InterfaceDecl`, and `ExternFunctionDecl`.

Implement `graph::compute` as a stub that only unions `scan_program` top-level usage into `Reachability` (no body-of-callee edges yet) so Task 2 can fill the fixed point without renaming types.

- [x] **Step 4: Re-run the unit tests**

```bash
cargo test --lib optimize::reachability::tests
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/optimize/reachability.rs src/optimize/reachability src/optimize.rs
git commit -m "feat: add declaration-reachability usage scanner"
```

---

### Task 2: Fixed-point reachability + AST prune + CheckResult reconcile

**Files:**
- Create: `src/optimize/reachability/prune.rs`
- Create: `src/optimize/reachability/reconcile.rs`
- Modify: `src/optimize/reachability.rs` (public `prune_unreachable_declarations`)
- Modify: `src/optimize/reachability/graph.rs` (real fixed point)
- Modify: `src/optimize/reachability/tests.rs`
- Modify: `src/optimize.rs` (re-export)

**Interfaces:**
- Consumes: Task 1 types, `crate::types::CheckResult`, `ClassInfo::{method_decls, methods, static_methods, vtable_*}`
- Produces: `prune_unreachable_declarations`, working `compute`

- [x] **Step 1: Write failing unit tests for prune + reconcile**

```rust
#[test]
fn prune_drops_unused_function_and_keeps_called_one() {
    let program = parse(
        "<?php
        function used(): int { return 1; }
        function unused(): int { return 2; }
        echo used();
        ",
    );
    let mut check = empty_check_result(&program);
    let pruned = prune_unreachable_declarations(program, &mut check, empty_options());
    assert!(has_function(&pruned, "used"));
    assert!(!has_function(&pruned, "unused"));
    assert!(check.functions.contains_key(&php_symbol_key("used")));
    assert!(!check.functions.contains_key(&php_symbol_key("unused")));
}

#[test]
fn prune_follows_callee_body_edges() {
    let program = parse(
        "<?php
        function inner(): int { return 3; }
        function outer(): int { return inner(); }
        function unused(): int { return inner(); }
        echo outer();
        ",
    );
    let mut check = empty_check_result(&program);
    let pruned = prune_unreachable_declarations(program, &mut check, empty_options());
    assert!(has_function(&pruned, "outer"));
    assert!(has_function(&pruned, "inner"));
    assert!(!has_function(&pruned, "unused"));
}

#[test]
fn prune_drops_unused_class_keeps_used_class() {
    let program = parse(
        "<?php
        class Keep { public function f(): int { return 1; } }
        class Drop { public function g(): int { return 2; } }
        echo (new Keep())->f();
        ",
    );
    let mut check = empty_check_result(&program);
    let pruned = prune_unreachable_declarations(program, &mut check, empty_options());
    assert!(has_class(&pruned, "Keep"));
    assert!(!has_class(&pruned, "Drop"));
}

#[test]
fn eval_keeps_otherwise_unused_function() {
    let program = parse(
        "<?php
        function hidden(): int { return 1; }
        eval('echo hidden();');
        ",
    );
    let mut check = empty_check_result(&program);
    let pruned = prune_unreachable_declarations(program, &mut check, empty_options());
    assert!(has_function(&pruned, "hidden"));
}
```

`empty_check_result` for these unit tests can be a minimal `CheckResult` built by running `types::check(&program).unwrap()` — that is the cheapest honest fixture.

- [x] **Step 2: Run the tests; they must fail**

```bash
cargo test --lib optimize::reachability::tests::prune_
```

- [x] **Step 3: Implement the fixed point**

`graph::compute`:

1. Index every declaration in the program (walk `If` / `Switch` / `NamespaceBlock` / `Synthetic` / `IncludeOnceGuard` the same way `lower_function_declarations` does).
2. Seed the worklist with roots (top-level executable usage + `exported_functions` + forced inventory groups + `eval_forced` ⇒ all declarations).
3. If executable-root usage has `dynamic_function`, mark every function reachable. Scan hazards in a declaration body only after an executable edge reaches that declaration; dead bodies and methods retained solely for structural metadata do not widen the graph.
4. Same for `dynamic_class` → every class-like. `dynamic_method` is handled in Task 5; in this task, treat `dynamic_method` as “keep all methods of classes that are already live” and do **not** yet drop methods of live classes.
5. While the worklist is not empty, scan the body of each newly-live declaration and union its edges.

Task 2 may keep all methods of a live class. That is enough to delete unused *sibling* classes and unused free functions. Task 5 tightens methods.

- [x] **Step 4: Implement AST prune**

`prune.rs`: walk statement lists and `retain` declarations that `Reachability` keeps. Recurse into grouping / control-flow nodes so a function declared inside an `if` is still removable. Do not delete non-declaration statements.

- [x] **Step 5: Implement CheckResult reconcile**

Drop only unused AST-backed function / class / enum / interface / extern entries. Preserve every
synthetic checker entry that has no declaration-index owner. Leave vtables of kept source classes
unchanged in this task (all methods still kept). Recompute `required_libraries` from remaining
`extern_functions` plus a walk of remaining builtin calls if that walk is cheap; if not, leave
`required_libraries` as a superset here and fix it in Task 5 when unused methods stop referencing
externs.

- [x] **Step 6: Re-run unit tests**

```bash
cargo test --lib optimize::reachability::tests
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/optimize/reachability src/optimize.rs
git commit -m "feat: prune unreachable functions and unused classes"
```

---

### Task 3: Wire the pass into the pipeline after DCE

**Files:**
- Modify: `src/pipeline.rs`
- Modify: `src/progress.rs`
- Create: `tests/codegen/optimizer/declaration_reachability.rs`
- Modify: `tests/codegen/mod.rs` (or `tests/codegen/optimizer.rs` module list)

**Interfaces:**
- Consumes: `optimize::prune_unreachable_declarations`, `PruneOptions` with empty inventory / empty forced groups for this task
- Produces: every full compile after DCE runs the pass

- [x] **Step 1: Write a failing codegen test**

```rust
//! Purpose:
//! End-to-end assembler tests for whole-program declaration reachability.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Asserts symbols in user assembly, not only stdout, so a keep-everything
//!   pass cannot silently satisfy the tests.

#[test]
fn test_unused_user_function_is_absent_from_assembly() {
    let dir = make_cli_test_dir("elephc_decl_reach_unused_fn");
    let (user_asm, _, _) = compile_source_to_asm_with_options(
        "<?php
        function used(): int { return 1; }
        function unused(): int { return 2; }
        echo used();
        ",
        &dir,
        8_388_608,
        false,
        false,
    );
    assert!(user_asm.contains(&function_symbol("used")) || user_asm.contains("_fn_used"));
    assert!(
        !user_asm.contains(".globl _fn_unused\n"),
        "unused function must not be emitted: {user_asm}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_unused_user_function_program_still_runs() {
    let out = compile_and_run(
        "<?php
        function used(): int { return 1; }
        function unused(): int { return 2; }
        echo used();
        ",
    );
    assert_eq!(out, "1");
}
```

Use `crate::names::function_symbol` / `method_symbol` for assertions. Look at `tests/codegen/cli.rs::test_cli_web_prunes_unused_session_surface_from_assembly` for the `.globl` pattern.

- [x] **Step 2: Run the assembler test; it must fail**

```bash
cargo test --test codegen_tests test_unused_user_function_is_absent_from_assembly
```

Expected: FAIL because the unused function is still emitted.

- [x] **Step 3: Wire the pipeline**

In `src/pipeline.rs`, immediately after `optimize::eliminate_dead_code(ast)`:

```rust
crate::progress::phase("decl-reach");
let phase_started = Instant::now();
let mut check_result = check_result;
let ast = optimize::prune_unreachable_declarations(
    ast,
    &mut check_result,
    optimize::reachability::PruneOptions {
        inventory: &inventory,          // empty until Task 4
        forced_groups: &forced_groups,  // empty until Task 4
        exported_functions: &exported_function_names,
        eval_forced: with_crates.contains("eval"),
    },
);
timings.record_since("decl-reach", phase_started);
```

`check_only` still returns before this phase. `--emit-ir` must run *after* this phase so dumped IR is pruned.

`exported_function_names` is `exported_functions.keys()` collected into a `HashSet<String>`.

Add `"decl-reach" => "Pruning unreachable declarations"` to `src/progress.rs`.

If `pipeline.rs` currently moves `check_result` by value into `backend::emit_and_link` / `eir_output::emit`, keep using the mutated value.

- [x] **Step 4: Re-run the new tests plus the existing web prune test**

```bash
cargo test --test codegen_tests test_unused_user_function
cargo test --test codegen_tests test_cli_web_prunes_unused_session_surface_from_assembly
```

Expected: PASS. The web test already passes today; it must not regress when the general pass starts deleting user functions.

- [ ] **Step 5: Commit**

```bash
git add src/pipeline.rs src/progress.rs tests/codegen/optimizer/declaration_reachability.rs tests/codegen
git commit -m "feat: run declaration reachability after AST DCE"
```

---

### Task 4: Prelude inventory and `--with-<crate>` force-keep

**Files:**
- Modify: every `inject_if_used` listed in the file map
- Modify: `src/pipeline.rs` (create inventory, pass `&mut inventory`, fill `forced_groups` from `with_crates`)
- Modify: `tests/codegen/optimizer/declaration_reachability.rs`
- Modify: codegen test harnesses that call injectors directly (grep `inject_if_used(`)

**Interfaces:**
- Consumes: Task 1 `PreludeInventory::record_program`
- Produces: inventory filled per injected prelude; `--with-pdo` roots the whole PDO group

Mapping from CLI crate to inventory id:

| `with_crates` entry | inventory id | injector |
|---|---|---|
| `pdo` | `pdo` | `pdo_prelude` |
| `tz` | `tz` | `tz_prelude` |
| `image` | `image` | `image_prelude` |
| `crypto` | `hash` | `hash_prelude` (only if you decide hash is forceable this way; today `inject_if_used(..., false)`) |
| `eval` | _(not a group)_ | `eval_forced: true` |
| `web` | `web` | **do not** force-keep |

`hash_prelude::inject_if_used` currently hardcodes `force: false` from the pipeline. Leave that. Inventory still records the group when it injects.

- [x] **Step 1: Write failing tests**

```rust
#[test]
fn test_pdo_query_only_drops_unused_sibling_classes() {
    let dir = make_cli_test_dir("elephc_decl_reach_pdo_siblings");
    let (user_asm, _, libs) = compile_source_to_asm_with_options(
        "<?php
        $pdo = new PDO('sqlite::memory:');
        echo $pdo->query('select 1')->fetchColumn();
        ",
        &dir,
        8_388_608,
        false,
        false,
    );
    assert!(
        !user_asm.contains("_method_Pdo_ns_Dblib")
            && !user_asm.contains("Pdo\\\\Dblib"),
        "unused optional PDO driver class must not be emitted"
    );
    assert!(
        libs.iter().any(|library| library == "elephc_pdo"),
        "used PDO program must still link the PDO bridge"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_with_pdo_keeps_unreferenced_pdo_function() {
    // Compile with --with-pdo and no PDO syntax. Assembly must still
    // contain pdo_drivers / PDO so eval/dynamic use can see them.
}
```

The `--with-pdo` test should go through the CLI helper in `tests/codegen/cli.rs` (same style as the web prune test), because `compile_source_to_asm_with_options` does not take `with_crates` today. If that is awkward, extend the compiler test helper with an optional `with_crates: &[&str]` in this task rather than inventing a second compile path.

Also add:

```rust
#[test]
fn test_program_without_pdo_still_does_not_link_bridge() {
    let dir = make_cli_test_dir("elephc_decl_reach_no_pdo");
    let (_, _, libs) = compile_source_to_asm_with_options(
        "<?php echo 1;",
        &dir,
        8_388_608,
        false,
        false,
    );
    assert!(libs.iter().all(|library| library != "elephc_pdo"));
    let _ = fs::remove_dir_all(&dir);
}
```

- [x] **Step 2: Run; sibling-class test should fail until inventory + prune see prelude decls**

Task 3 already drops unused *user* classes. Prelude classes should already drop if they are ordinary `ClassDecl`s in the AST and nothing references them. This task exists because:

- `--with-pdo` must *prevent* that drop
- injectors must record groups so force-keep is not a hardcoded name list
- tests lock the policy

If the sibling-class test already passes after Task 3, keep it as a regression and still implement inventory + `--with-pdo` roots.

- [x] **Step 3: Change injector signatures and record inventory**

Pattern for each injector:

```rust
pub fn inject_if_used(
    program: Program,
    force: bool,
    inventory: &mut crate::optimize::reachability::PreludeInventory,
) -> Program {
    if !force && !detect::program_uses_pdo(&program) {
        return program;
    }
    let prelude = parsed_prelude_for_version(php_version);
    inventory.record_program("pdo", &prelude);
    let mut combined = prelude;
    combined.extend(program);
    combined
}
```

Grep for every `inject_if_used` call site (pipeline, codegen test harness, prelude unit tests) and pass a local `PreludeInventory`. Tests that only care about injection can use `&mut PreludeInventory::new()`.

In `pipeline.rs`:

```rust
let mut inventory = optimize::reachability::PreludeInventory::new();
let mut forced_groups = HashSet::new();
if with_crates.contains("pdo") { forced_groups.insert("pdo".into()); }
if with_crates.contains("tz") { forced_groups.insert("tz".into()); }
if with_crates.contains("image") { forced_groups.insert("image".into()); }
```

Do not insert `"web"`.

- [x] **Step 4: Make forced groups roots in `graph::compute`**

If `options.forced_groups` contains `"pdo"`, mark every name in `inventory.groups["pdo"]` reachable before the fixed point.

`--with-eval` remains `eval_forced: true` (keep everything user-visible).

- [x] **Step 5: Run focused tests**

```bash
cargo test --test codegen_tests test_pdo_query_only_drops_unused_sibling_classes
cargo test --test codegen_tests test_with_pdo_keeps_unreferenced_pdo_function
cargo test --test codegen_tests test_program_without_pdo_still_does_not_link_bridge
cargo test --lib pdo_prelude
cargo test --test codegen_tests test_cli_web_prunes_unused_session_surface_from_assembly
```

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: record prelude inventory and honor --with-* force-keep"
```

---

### Task 5: Method-level pruning on live classes, including vtable rebuild

This is the PDO-sized win: `new PDO` + `query` must not emit `PDO::beginTransaction`.

**Files:**
- Modify: `src/optimize/reachability/graph.rs`
- Modify: `src/optimize/reachability/prune.rs` (strip methods inside a kept `ClassDecl`)
- Modify: `src/optimize/reachability/reconcile.rs` (strip `method_decls` / rebuild vtables)
- Modify: `src/ir_lower/program/class_methods.rs` (debug assertion)
- Modify: `src/optimize/reachability/tests.rs`
- Modify: `tests/codegen/optimizer/declaration_reachability.rs`

**Interfaces:**
- Consumes: `Hazards.dynamic_method`, live class set, method edges
- Produces: kept method set; compacted `vtable_methods` / `vtable_slots`

- [x] **Step 1: Write failing tests**

Unit:

```rust
#[test]
fn prune_drops_unused_method_on_live_class() {
    let program = parse(
        "<?php
        class T {
            public function keep(): int { return 1; }
            public function drop(): int { return 2; }
        }
        echo (new T())->keep();
        ",
    );
    let mut check = types::check(&program).unwrap();
    let pruned = prune_unreachable_declarations(program, &mut check, empty_options());
    assert!(has_method(&pruned, "T", "keep"));
    assert!(!has_method(&pruned, "T", "drop"));
    let class = check.classes.get("T").unwrap();
    assert!(class.methods.contains_key(&php_symbol_key("keep")));
    assert!(!class.methods.contains_key(&php_symbol_key("drop")));
    assert!(!class.vtable_methods.iter().any(|m| php_symbol_key(m) == php_symbol_key("drop")));
}

#[test]
fn prune_keeps_interface_method() {
    let program = parse(
        "<?php
        interface I { public function need(): int; }
        class C implements I {
            public function need(): int { return 1; }
            public function extra(): int { return 2; }
        }
        function take(I $x): int { return $x->need(); }
        echo take(new C());
        ",
    );
    let mut check = types::check(&program).unwrap();
    let pruned = prune_unreachable_declarations(program, &mut check, empty_options());
    assert!(has_method(&pruned, "C", "need"));
    assert!(!has_method(&pruned, "C", "extra"));
}

#[test]
fn prune_keeps_all_methods_when_dynamic_method_hazard() {
    let program = parse(
        "<?php
        class T {
            public function keep(): int { return 1; }
            public function drop(): int { return 2; }
        }
        $t = new T();
        $m = $argv[1];
        echo $t->$m();
        ",
    );
    let mut check = types::check(&program).unwrap();
    let pruned = prune_unreachable_declarations(program, &mut check, empty_options());
    assert!(has_method(&pruned, "T", "drop"));
}
```

Codegen:

```rust
#[test]
fn test_pdo_query_only_omits_begin_transaction_method() {
    let dir = make_cli_test_dir("elephc_decl_reach_pdo_methods");
    let (user_asm, _, _) = compile_source_to_asm_with_options(
        "<?php
        $pdo = new PDO('sqlite::memory:');
        echo $pdo->query('select 1')->fetchColumn();
        ",
        &dir,
        8_388_608,
        false,
        false,
    );
    assert!(
        !user_asm.contains(&method_symbol("PDO", "beginTransaction")),
        "unused PDO::beginTransaction must not be emitted"
    );
    assert!(
        user_asm.contains(&method_symbol("PDO", "__construct"))
            || user_asm.contains(&method_symbol("PDO", "query")),
        "used PDO methods must remain"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_pdo_query_only_still_runs() {
    let out = compile_and_run(
        "<?php
        $pdo = new PDO('sqlite::memory:');
        $pdo->exec('create table t(id integer)');
        echo $pdo->query('select 1')->fetchColumn();
        ",
    );
    assert_eq!(out, "1");
}

#[test]
fn test_method_exists_literal_keeps_method() {
    let dir = make_cli_test_dir("elephc_decl_reach_method_exists");
    let (user_asm, _, _) = compile_source_to_asm_with_options(
        "<?php
        class T {
            public function hidden(): int { return 1; }
        }
        $t = new T();
        echo method_exists($t, 'hidden') ? 'y' : 'n';
        ",
        &dir,
        8_388_608,
        false,
        false,
    );
    assert!(user_asm.contains(&method_symbol("T", "hidden")));
    let _ = fs::remove_dir_all(&dir);
}
```

Also add a runtime test that `method_exists($t, 'hidden')` prints `y`.

- [x] **Step 2: Run; method-omission tests must fail**

```bash
cargo test --lib optimize::reachability::tests::prune_drops_unused_method
cargo test --test codegen_tests test_pdo_query_only_omits_begin_transaction_method
```

- [x] **Step 3: Extend the fixed point**

When `!hazards.dynamic_method`:

- Methods of a live class start *dead*.
- Seed: referenced methods, declared magic methods (v1: all magic methods the class actually declares, if the class is instantiated), interface-required methods, `__construct` if `new` is reachable.
- Then follow edges from those method bodies (they may call other methods of the same class, `parent::`, or free functions).
- Track interface-required methods as structural roots: retain their symbols and static dependencies for vtables, but propagate their dynamic hazards only after an executable call edge upgrades the method to behaviorally reachable.

When `hazards.dynamic_method`: every method of every live class is a root.

`method_exists($o, 'literal')` adds that method name as a reference on every live class that declares it (or, cheaper and still sound: on every live class).

- [x] **Step 4: Strip methods in the AST**

In a kept `ClassDecl` / `EnumDecl` / `InterfaceDecl`, `methods.retain(|m| reach.methods.contains(&(class, php_symbol_key(&m.name), m.is_static)))`.

Abstract interface methods with no body still need to stay if the interface is live and some implementor is live. If the interface itself is live only as a type hint, keep the abstract signatures (they are not assembler).

- [x] **Step 5: Rebuild ClassInfo vtables**

For each kept class:

1. `method_decls.retain(...)` to the kept methods.
2. Remove dropped keys from `methods`, `static_methods`, visibility/impl/declaring maps, late-static maps, callable-return maps.
3. Rebuild `vtable_methods` as the kept instance methods in the previous vtable order (filter, do not sort alphabetically — preserve dispatch order of survivors).
4. Re-number `vtable_slots` from `0..`.
5. Same for `static_vtable_*`.

Then drop `extern_functions` that no remaining method/function body calls. Recompute `required_libraries`. After this, a program that used PDO but whose remaining methods only call a subset of `elephc_pdo_*` will stop pulling unused bridge objects *if* those objects are not referenced. The user object will also stop referencing them because the methods that called them are gone.

- [x] **Step 6: Debug assertion in lowering**

In `lower_methods_for_class_like`, under `cfg(debug_assertions)`, panic if a `method_decls` entry has no matching AST method. This catches a missed reconcile.

- [x] **Step 7: Run focused tests**

```bash
cargo test --lib optimize::reachability::tests
cargo test --test codegen_tests test_pdo_query_only
cargo test --test codegen_tests test_method_exists_literal_keeps_method
cargo test --test codegen_tests test_unused_user_function
cargo test --test codegen_tests test_cli_web_prunes_unused_session_surface_from_assembly
```

Grep existing PDO codegen tests (`tests/codegen/pdo.rs` and driver files) and run a focused slice if the query-only fixture is too thin (transactions, fetch modes, exceptions):

```bash
cargo test --test codegen_tests pdo::
```

Only if that slice is huge, run the named tests that exercise `beginTransaction` / `prepare` / `bind` so kept-method paths still work.

- [ ] **Step 8: Commit**

```bash
git commit -m "feat: prune unused methods on live classes"
```

---

### Task 6: Retire the web-only function prune; keep OPcache inject-time pay-for-use

**Files:**
- Modify: `src/web_prelude.rs` (remove `prune_unreachable_prelude_functions` and the `user_usage` scan used only for that prune)
- Keep: inject-time skip of `__ElephcCallableSessionHandler` / `is_callable_session_handler_decl`
- Keep: `src/web_prelude/usage.rs` only if still used by the callable-handler skip; otherwise delete it and use the shared scanner
- Modify: `src/opcache_prelude/injection.rs` — no behavior change; add a comment that the general pass is the backstop
- Modify: `tests/codegen/cli.rs` (existing web prune test must remain)

**Interfaces:**
- Consumes: Task 3–5 pipeline pass
- Produces: one reachability implementation, not two

- [x] **Step 1: Confirm the existing web assembler test still specifies the contract**

`tests/codegen/cli.rs::test_cli_web_prunes_unused_session_surface_from_assembly` already requires:

- `__elephc_session_start_core` stays (auto-start)
- `session_start` public wrapper is absent
- `session_set_save_handler` is absent
- `__ElephcCallableSessionHandler` is absent

Do not weaken this test.

- [x] **Step 2: Delete the web-only prune**

Remove `prune_unreachable_prelude_functions`. The callable-handler skip stays:

```rust
if !needs_callable_session_handler {
    combined.retain(|stmt| !is_callable_session_handler_decl(&stmt.kind));
}
```

`needs_callable_session_handler` can keep using a small local scan, or switch to `optimize::reachability::scan_program(&program).hazards.dynamic_function` plus a function-name set. Prefer the shared scanner so “what is a dynamic call” cannot drift.

Web bootstrap statements (superglobal setup, auto-start `if`) are top-level executable roots. They must keep `__elephc_session_start_core`. Verify `scan_program` records that call from the prelude *after* it is prepended. The pipeline pass runs on the combined AST, so this is automatic if auto-start stays as a top-level `if` in the prelude.

- [x] **Step 3: Run web + session tests**

```bash
cargo test --test codegen_tests test_cli_web_prunes_unused_session_surface_from_assembly
cargo test --test codegen_tests session
```

If `session` is too broad, run the web/session assembler and a couple of runtime session tests that actually call `session_start()`.

- [ ] **Step 4: Commit**

```bash
git commit -m "refactor: drop web-only declaration prune in favor of the shared pass"
```

---

### Task 7: Validate linker dead-strip boundaries (all targets)

**Files:**
- Modify: `src/codegen_support/emit.rs` / user-asm finalize (`src/codegen/mod.rs` `generate_user_asm_from_ir_with_options`)
- Modify: `tests/codegen/dead_strip.rs` or `tests/codegen/optimizer/declaration_reachability.rs`

**Interfaces:**
- Consumes: existing runtime `dead_strip` + `.subsections_via_symbols` path in `src/codegen_support/driver_support.rs`
- Produces: an explicit boundary: Linux user functions remain per-section, macOS user assembly remains one linker atom, and macOS runtime helpers remain per-symbol atoms

- [x] **Step 1: Cover declaration pruning and linker-sensitive callable reachability**

Keep the declaration-level assembly shape tests. Add a managed-native regression
whose instance callable is retained through descriptor data and invoked by a
bridge callback. This catches dangling callable targets that a direct-call-only
fixture cannot expose.

```rust
#[test]
fn test_managed_regex_instance_callable_survives_dead_strip() {
    // The bridge reaches the method through a callable descriptor, not a direct call edge.
    let out = compile_cli_file_and_run_with_managed_pcre2(/* callable fixture */, &[]);
    assert_eq!(out, "descriptor:descriptor:");
}
```

- [x] **Step 2: Reproduce the unsafe macOS experiment**

Applying the runtime object's `.subsections_via_symbols` path to executable user
assembly reproduces a SIGSEGV when the managed regex bridge invokes the
address-taken instance callable. Preserving the user object as one atom restores
the descriptor target.

- [x] **Step 3: Keep user-object atom splitting disabled on macOS**

Do not set `Emitter::dead_strip`, localize internal user labels, or append
`.subsections_via_symbols` in `generate_user_asm_from_ir_with_options`. Keep
those operations in runtime-object generation only. Linux remains unchanged and
cdylib exports remain intact.

- [x] **Step 4: Run dead-strip + reachability tests**

```bash
cargo test --test codegen_tests test_user_assembly_respects_linker_dead_strip_boundaries
cargo test --test codegen_tests test_hello_world_after_dead_strip
cargo test --test codegen_tests test_pdo_query_only
cargo nextest run --profile ci --test codegen_tests \
  -E 'test(test_managed_regex_instance_callable_survives_dead_strip)'
```

- [ ] **Step 5: Commit**

```bash
git commit -m "fix: preserve address-taken user callable labels"
```

---

### Task 8: Docs, ROADMAP, and README

**Files:**
- Modify: `docs/internals/the-optimizer.md`
- Modify: `docs/compiling/compilation-pipeline.md`
- Modify: `docs/compiling/optimization.md`
- Modify: `README.md` (the optimizer paragraph that already describes DCE)
- Modify: `ROADMAP.md` under `v0.26.x` — add one unchecked item, do not edit completed items

**Interfaces:** none.

- [x] **Step 1: Pipeline docs**

In `docs/compiling/compilation-pipeline.md`, insert after `dce`:

```text
  -> decl-reach         drop unreachable functions, classes, and methods
```

Describe in the optimizer section: the pass is conservative; `eval` / dynamic call / `unserialize` / Reflection keep a wider set; `--with-pdo` (and siblings) force-keep that prelude group; `--web` is not a force-keep.

- [x] **Step 2: Optimizer internals**

In `docs/internals/the-optimizer.md`:

- Fix the stale sentence that elephc “goes straight from AST to target assembly” / “there is no middle IR”. EIR is the backend. This pass is AST-level and runs *before* EIR lowering.
- Add pass 6: `prune_unreachable_declarations`.
- Document hazards and the CheckResult reconcile requirement (`method_decls` / vtables).
- Point at `src/optimize/reachability/`.

- [x] **Step 3: Optimization CLI page**

In `docs/compiling/optimization.md`, mention declaration reachability under the AST optimizer list. No new flag.

- [x] **Step 4: README**

Update the DCE bullet so it does not claim only intra-procedural statement removal.

- [x] **Step 5: ROADMAP**

Under `## v0.26.x`, add:

```markdown
- [ ] Whole-program declaration reachability — drop unreachable functions, unused classes, and unused methods (including compiler preludes such as PDO) after AST DCE, with conservative keep-all behavior for `eval`, dynamic calls, `unserialize`, and Reflection, and `--with-<crate>` force-keep for forced prelude groups
```

Do not mark it `[x]` until the implementation lands.

- [ ] **Step 6: Commit**

```bash
git commit -m "docs: describe whole-program declaration reachability"
```

---

## Soundness appendix

These are the cases that must stay green. If a later simplification would break one, keep the extra code.

| Program | Must keep |
|---|---|
| `echo used(); function unused(){}` | `used` only |
| `new PDO` + `query` | `PDO`, `PDOStatement` as needed, `query` / fetch methods actually called, `__construct`, exception classes if thrown |
| `function_exists('pdo_drivers')` | `pdo_drivers` |
| `method_exists($pdo, 'beginTransaction')` | `PDO::beginTransaction` |
| `method_exists(method: 'run', object_or_class: 'Worker')` | `Worker::run` |
| `new static()` reached through `Child::factory()` | the runtime-selected child constructor and its signature dependencies |
| live `fopen(mode: 'rb', filename: 'https://...')` after a dead positional `fopen` call | `elephc_tls` |
| `$pdo->$m()` | every method of live PDO classes |
| `eval('...')` or `--with-eval` | every user-visible declaration |
| `--with-pdo` and no PDO syntax | entire PDO group |
| `--web` and `echo 'ok'` | session auto-start core; **not** `session_start` / save-handler |
| `unserialize($argv[1])` | every class |
| `foreach ($stmt as $row)` | `PDOStatement::getIterator` (interface) |
| `#[Export] function f` in a cdylib | `f` and its callees |
| class with unused sibling `Pdo\Dblib` | drop `Pdo\Dblib` unless `--with-pdo` or `dynamic_class` |

## Suggested PR split

Land as four PRs so each is reviewable and green:

1. Tasks 1–3 — scanner + unused functions/classes + pipeline (user-code win, no prelude API churn yet)
2. Task 4 — inventory + `--with-*` (prelude API signature change isolated)
3. Tasks 5–6 — method prune + delete web-only prune (the PDO assembler win)
4. Tasks 7–8 — linker safety net + docs

Do not mix the method-vtable rebuild with the injector signature change in the same PR.

## Self-review

**Spec coverage**

- Asymmetry between web/OPcache and PDO/image/hash/user code → Tasks 3–6
- Unused prelude methods in the assembler → Task 5
- `--with-pdo` must not become a no-op → Task 4
- `CheckResult.method_decls` would otherwise re-lower dropped methods → Task 2 / 5
- Linker cannot save vtable-pinned methods → Task 5 (primary), Task 7 (safety net)
- PHP observability (`function_exists`, `method_exists`, `eval`, `unserialize`, Reflection) → hazards in Task 1, tests in Tasks 2 and 5
- Docs / ROADMAP → Task 8

**Placeholder scan:** no TBD / “handle edge cases later” without an explicit v1 rule.

**Type consistency:** `PruneOptions`, `PreludeInventory`, `Usage`, `Hazards`, `Reachability`, and `prune_unreachable_declarations` are named once in Interfaces and reused.

**Intentionally deferred:** typecheck-time skip of trusted prelude bodies; `--no-decl-prune`; property/constant prune; EIR-level function DCE.
