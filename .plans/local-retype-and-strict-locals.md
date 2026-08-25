# Local Retype (unset + implicit) and `--strict-locals` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let an undeclared-type local variable change type — explicitly via `unset($a); $a = "ciao";`, implicitly via `$a = 0; $a = "ciao";` (fresh slot), and branch-divergently via `if (...) { $a = 0; } else { $a = "ciao"; }` (whole-frame boxed Mixed storage) — all warning-by-default, with a new `--strict-locals` flag restoring today's hard error. Declared types (typed locals, typed params, properties) stay strict in both modes.

**Architecture:** The checker is the single decision point: it decides which `unset()` calls kill a local binding, which incompatible straight-line assignments re-bind to a fresh binding, and which branch-divergently assigned locals get whole-frame boxed Mixed storage (via a per-body syntactic pre-scan); it records those decisions as span-keyed sets in `CheckResult` and emits the warning/error. The lowering never re-derives eligibility — it consults the recorded spans and, at those sites, abandons the old frame slot (after releasing its value) and mints a fresh slot, or pre-declares the slot as boxed `Mixed` before the first store. This keeps checker and lowering in lock-step by construction (no independent classifiers to drift).

**Tech Stack:** Rust (elephc compiler), existing test harnesses `tests/error_tests.rs` (`expect_error` / `expect_no_error` / `expect_warning` / `expect_no_warning`) and `tests/codegen/*` (`compile_and_run`).

**Spec:** The Design section below (this document is self-contained; the design was agreed in conversation on 2026-08-21).

## Global Constraints

- NEVER run `cargo nextest` (it hard-locks this machine). Verify with filtered `cargo test --test <name> <filter>`; the user runs full passes.
- Git commits: concise conventional messages (`feat(checker): …`, `test(codegen): …`, `docs: …`). Do NOT add `Co-Authored-By` lines.
- Default mode is **permissive** (retype allowed with a warning). `--strict-locals` restores the error. This flips today's de-facto default — CHANGELOG is handled at release time by the release-audit skill; do not touch `CHANGELOG.md` in this work.
- The strict-mode error message must keep the exact prefix `Type error: cannot reassign $<name> from <old> to <new>` (existing tests assert the `cannot reassign $x` substring).
- Warning message format for a retype (exact): `$<name> changes type from <old> to <new>; the previous value is discarded (compile with --strict-locals to make this an error)`.
- Warning message format for a mixed-storage local (exact): `$<name> is assigned incompatible types (<t1> and <t2>); it is compiled as boxed mixed storage (compile with --strict-locals to make this an error)`.
- Declared types are exempt from ALL of this in BOTH modes: `TypedAssign` locals (`int $x = …`), parameters with a declared type hint, and class properties keep today's strict behavior.
- e2e PHP fixtures must derive values from `$argc` where a constant would let AST folding erase the construct under test.

## Design

### Eligibility: when may a binding be killed?

A local binding may be killed (by `unset`) or re-bound to an incompatible type (by assignment) only when ALL hold:

1. The kill/retype site is at **conditional depth 0** of the current body (not inside any `if`/`elseif`/`else`/`switch`/`while`/`do-while`/`for`/`foreach`/`try`/`catch`/`finally`).
2. The binding itself was **created at conditional depth 0** (so the original store dominates the kill site and the old slot is definitely initialized — releasing it is unconditionally safe).
3. The local is **not reference-aliased**: not a by-ref parameter, not the target OR source of `$x =& $y`, not captured by-ref (`use (&$x)`), not passed as an argument to a by-ref parameter anywhere in the body, not bound via `global`, not a `static` local.
4. The local is **not declared-typed**: not a `TypedAssign` local (`int $x = …`) and not a parameter with a declared type hint. A declaration is a programmer contract and stays strict in both modes. (Class properties never reach the local paths at all — property assignments validate against the declared property type in `src/types/checker/stmt_check/assignments/properties.rs` — so `public string $variable` is strict by construction; pin with a test, no code change needed.)

Rationale: the checker's if/loop handling shares one mutable `TypeEnv` with targeted save/restore (`src/types/checker/stmt_check/control_flow.rs:292`), not per-branch fork+merge, so a kill inside one branch would leak into sibling branches and post-join code. Depth-0-only is the sound v1; ineligible sites keep today's behavior exactly (unset stays a null-store no-op for typing; incompatible reassignment stays an error even in permissive mode).

### Decision recording

- `CheckResult.local_bind_kill_sites: HashSet<Span>` — spans of `unset()` **arguments** (each plain `$var` arg has its own span) whose binding the checker killed.
- `CheckResult.local_retype_sites: HashSet<Span>` — spans of **statement-form** assignments (`StmtKind::Assign`) the checker re-bound. Expression-form assignments (`while (($a = f()))`, `$b = ($a = "x")`) are NOT eligible in v1 and keep the error, so span-keying is unambiguous: the checker and `lower_assign` both see `stmt.span`.
- `CheckResult.mixed_storage_store_sites: HashSet<Span>` — spans of every statement-form assignment to a mixed-storage local (see next section).

### Branch-divergent assignments → mixed-storage locals

`if (...) { $a = 0; } else { $a = "ciao"; }` — and single-branch retype of an outer binding, and heterogeneous loop-carried locals — are supported by giving `$a` **boxed Mixed storage for the whole frame**. This is the mechanism the string-incdec contract already uses end-to-end: the checker types the local `Mixed` and the lowering boxes the slot from the first store (cf. `boxed_incdec_storage_type`, `src/ir_lower/context.rs:677`). No env fork/merge is needed — the join problem is sidestepped because the slot holds either value everywhere.

A per-body **syntactic pre-scan** (run before statement checking) marks a name as mixed-storage when ALL hold:

1. The body contains ≥2 statement-form assignments (`StmtKind::Assign`) to the name whose syntactic value types (via `infer_expr_type_syntactic`, `src/types/checker/inference/syntactic.rs:282`) fail `merged_assignment_type`, with **at least one at syntactic conditional depth > 0**. Depth-0-only conflicts are left to the kill/rebind path, which produces two unboxed slots — strictly better codegen.
2. Every write to the name in the body is a statement-form `StmtKind::Assign`: no `++`/`--`, no compound assignment, no foreach target, no `list()` target, no expression-form assignment, no by-ref call argument.
3. The name is not otherwise excluded: not mentioned in any `unset()` in the body, not reference-aliased (syntactically: `=&` target/source, `use (&$x)`, by-ref param), not `global`, not `static`, not declared-typed.

**Marking dominates:** a marked name's first assignment binds `Mixed`, so `merge_local_assignment_type` always succeeds and the kill/rebind path never fires for it. In permissive mode the pre-scan emits one warning per marked name (at the first conflicting assignment's span); under `--strict-locals` the pre-scan is disabled and the divergent assignment errors exactly as today. The performance cost (every read of a marked local goes through Mixed dispatch) is what the warning signals.

### Mode

`CheckOptions { strict_locals: bool }` threads CLI → pipeline → checker. In strict mode the retype path errors (current message); the `unset` kill is mode-independent (killing a binding is PHP-truthful in both modes — reading after `unset` becomes "Undefined variable", assigning after `unset` binds fresh). The lowering needs no mode flag: it only ever sees programs the checker accepted, and rebinds exactly at recorded spans.

### Out of scope (v1)

- Expression-form assignments; compound assignments (`.=`, `+=`); heterogeneity through `++`/`--` beyond the existing incdec contract; `list()` / foreach-target retype; static properties and globals' program-wide types.
- Declared-typed locals/params/properties: strict in both modes by design (a requirement, not a limitation).
- Names both `unset` and branch-divergently assigned: the pre-scan skips them (the kill path applies where eligible; otherwise today's error).
- Precise `Int|Str` union storage instead of `Mixed` for marked locals (future refinement — union int-dispatch already exists via `type_supports_mixed_int_dispatch`).
- eval strictness: the magician interpreter is dynamically typed and already allows retype; `--strict-locals` gates static compilation only (verified by test in Task 8, documented in Task 8).

## File Structure

- Modify: `src/cli.rs` — `--strict-locals` flag, help text, `Config.strict_locals`.
- Modify: `src/types/checker/mod.rs` — `CheckOptions`, `check_types_with_options`, new `Checker` fields, `local_binding_is_killable`, `CheckResult` assembly.
- Modify: `src/types/checker/driver/mod.rs` — thread options into `check_types_impl` and the nested `check_types` call at `driver/mod.rs:458`.
- Modify: `src/types/result.rs` — `check_with_options` / `check_with_target_and_options` wrappers; `CheckResult` fields.
- Modify: `src/types/checker/stmt_check.rs` — conditional-depth increment around control-flow statements.
- Modify: `src/types/checker/stmt_check/assignments.rs` — record `static` local names (`StmtKind::StaticVar` arm, line ~151).
- Modify: `src/types/checker/stmt_check/assignments/locals.rs` — `merge_local_assignment_type` retype hook (line 749), `check_ref_assign` alias recording (line 221), binding-depth recording.
- Modify: `src/types/checker/inference/expr/effects.rs` — `unset` kill arm in `infer_type_with_assignment_effects` (line 44).
- Modify: `src/types/checker/inference/ops.rs` — by-ref call-argument aliasing (line ~1137), closure-body entry (`with_local_storage_context`, line ~406).
- Modify: `src/types/checker/driver/functions.rs` — per-body scan/reset invocation, typed-param recording.
- Create: `src/types/checker/mixed_storage_scan.rs` — syntactic pre-scan for mixed-storage locals.
- Modify: `src/ir_lower/mod.rs` — copy the two span sets from `CheckResult` into `LoweringContext`.
- Modify: `src/ir_lower/context.rs` — kill in `unset_local` (line 1834), new `rebind_local_for_retype`.
- Modify: `src/ir_lower/stmt/mod.rs` — retype hook in `lower_assign` (line 129).
- Modify: `src/ir_lower/stmt/repr_fixpoint.rs` — reset per-name repr accumulation at recorded kill/retype spans (assign arm at line 329).
- Modify: `tests/error_tests.rs` — `check_source_strict`, `expect_error_strict` helpers.
- Modify: `tests/error_tests/type_system.rs` — migrate `cannot reassign` tests, add new checker tests.
- Create: `tests/codegen/locals_retype.rs` — e2e tests (register in the codegen test module tree like its siblings).
- Modify: docs pages that document `--strict-php`-style flags and the local monomorphism rule (located in Task 6).

---

### Task 1: `--strict-locals` flag and `CheckOptions` plumbing

No semantic change yet — the option must reach the `Checker` as a field and default to `false`.

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/types/checker/mod.rs`
- Modify: `src/types/checker/driver/mod.rs`
- Modify: `src/types/result.rs`
- Modify: the pipeline call site of `check_with_target` (locate in step 4)

**Interfaces:**
- Produces: `pub struct CheckOptions { pub strict_locals: bool }` (derives `Debug, Clone, Copy, Default`) in `src/types/checker/mod.rs`, re-exported as `types::CheckOptions`.
- Produces: `pub fn check_types_with_options(program: &Program, target_platform: Platform, options: CheckOptions) -> Result<CheckResult, CompileError>`; existing `check_types(program, platform)` becomes a delegating wrapper passing `CheckOptions::default()`.
- Produces: in `src/types/result.rs`: `pub fn check_with_options(program: &Program, options: CheckOptions)` (host platform) and `pub fn check_with_target_and_options(program: &Program, target: Target, options: CheckOptions)`; existing `check` / `check_with_target` delegate with defaults.
- Produces: `Checker.strict_locals: bool` field; `Config.strict_locals: bool` in `src/cli.rs`.

- [x] **Step 1: Write the failing CLI test**

In the `#[cfg(test)]` module of `src/cli.rs`, mirroring the `strict_php` tests (around lines 713–830):

```rust
/// Verifies `--strict-locals` sets the strict_locals flag on the parsed config.
#[test]
fn strict_locals_flag_sets_strict_locals() {
    let config = parse_args_for_test(&["build", "main.php", "--strict-locals"]);
    assert!(config.strict_locals);
}

/// Verifies the absence of `--strict-locals` defaults to permissive local retyping.
#[test]
fn no_strict_locals_flag_defaults_off() {
    let config = parse_args_for_test(&["build", "main.php"]);
    assert!(!config.strict_locals);
}
```

Use the same argv-construction helper the neighbouring `strict_php` tests use (read them and copy the exact pattern — the helper name above is illustrative; keep the file's real one).

- [x] **Step 2: Run to verify failure**

Run: `cargo test --test '*' --bin elephc strict_locals 2>/dev/null || cargo test -p elephc strict_locals`
(unit tests live in the binary crate; use `cargo test strict_locals` and expect a compile error: no field `strict_locals`).

- [x] **Step 3: Implement flag parsing**

In `src/cli.rs`: add `pub(crate) strict_locals: bool` to the config struct next to `strict_php` (line ~184); `let mut strict_locals = false;` next to `strict_php` init (line ~268); parse arm `else if arg == "--strict-locals" { strict_locals = true; }` next to the `--strict-php` arm (line ~396); add the field to the struct literal (line ~498); add a `--help` line next to `--strict-php` (line ~103): `  --strict-locals         Make an incompatible local retype (e.g. int then string) a compile error instead of a warning`. Check the existing help-coverage test (line ~739 comment: a flag missing from `--help` is caught) and satisfy it.

- [x] **Step 4: Thread options to the checker**

- In `src/types/checker/mod.rs`: define `CheckOptions`; rename the body of `check_types` into `check_types_with_options(program, target_platform, options)` and make `check_types` delegate with `CheckOptions::default()`. Pass `options` into `driver::check_types_impl`.
- In `src/types/checker/driver/mod.rs`: add the `options: CheckOptions` parameter to `check_types_impl`, set `checker.strict_locals = options.strict_locals` where the `Checker` is constructed, and pass the same `options` through the nested `check_types` call at line ~458 (switch it to `check_types_with_options`).
- Add `pub strict_locals: bool` (default `false`) to the `Checker` struct in `src/types/checker/mod.rs`.
- In `src/types/result.rs`: add the two `*_with_options` wrappers; keep `check` / `check_with_target` delegating.
- Find and update the pipeline call site: `grep -rn "check_with_target(" src/ --include="*.rs"` — switch it to `check_with_target_and_options(program, target, CheckOptions { strict_locals: config.strict_locals })` (thread `config` from `src/pipeline.rs` / `src/lib.rs` as the call chain requires).

- [x] **Step 5: Run tests and build**

Run: `cargo test strict_locals` — expect the two new tests PASS.
Run: `cargo build` — expect clean.

- [x] **Step 6: Commit**

```bash
git add src/cli.rs src/types/checker/mod.rs src/types/checker/driver/mod.rs src/types/result.rs src/pipeline.rs
git commit -m "feat(cli): add --strict-locals flag threaded to the checker as CheckOptions"
```

---

### Task 2: Checker — eligibility state and `unset` kills depth-0 bindings

**Files:**
- Modify: `src/types/checker/mod.rs` (Checker fields, predicate, CheckResult field)
- Modify: `src/types/result.rs` (CheckResult field)
- Modify: `src/types/checker/stmt_check.rs` (depth counter)
- Modify: `src/types/checker/stmt_check/assignments.rs` (static names)
- Modify: `src/types/checker/stmt_check/assignments/locals.rs` (binding depth, ref-alias sources)
- Modify: `src/types/checker/inference/expr/effects.rs` (unset arm)
- Test: `tests/error_tests/type_system.rs`

**Interfaces:**
- Consumes: `Checker.strict_locals` from Task 1 (not read yet — used in Task 3).
- Produces: `Checker` fields `local_conditional_depth: u32`, `local_binding_depth: HashMap<String, u32>`, `ref_aliased_locals: HashSet<String>`, `static_local_names: HashSet<String>`, `typed_local_names: HashSet<String>`, `local_bind_kill_sites: HashSet<Span>`, `local_retype_sites: HashSet<Span>` (retype set filled in Task 3).
- Produces: `fn local_binding_is_killable(&self, name: &str) -> bool` on `Checker`.
- Produces: `CheckResult.local_bind_kill_sites: HashSet<Span>` and `CheckResult.local_retype_sites: HashSet<Span>` (assembled in `check_types_with_options`).

- [x] **Step 1: Write the failing checker tests**

In `tests/error_tests/type_system.rs`:

```rust
/// `unset` at top level kills the binding: a later incompatible assignment binds fresh.
#[test]
fn test_unset_then_retype_is_accepted() {
    expect_no_error("<?php $a = 1; unset($a); $a = \"ciao\"; echo $a;");
}

/// `unset` at top level kills the binding: a later read is an undefined variable.
#[test]
fn test_read_after_unset_is_undefined() {
    expect_error("<?php $a = 1; unset($a); echo $a;", "Undefined variable");
}

/// Multi-arg unset kills every plain-variable binding.
#[test]
fn test_multi_arg_unset_kills_all_bindings() {
    expect_no_error("<?php $a = 1; $b = 2; unset($a, $b); $a = \"x\"; $b = \"y\"; echo $a . $b;");
}

/// A conditional unset does NOT kill the binding (sound: the branch may not run).
/// (A later incompatible reassignment of the still-bound name is Task 3's business:
/// it becomes a depth-0 retype warning — do not assert an error for it here.)
#[test]
fn test_conditional_unset_keeps_binding() {
    expect_no_error("<?php $a = 1; if ($argc > 1) { unset($a); } echo $a;");
}

/// A binding created inside a branch is not killable at depth 0 (may be uninitialized).
#[test]
fn test_branch_created_binding_not_killable() {
    expect_error("<?php if ($argc > 1) { $a = 1; } unset($a); $a = \"x\"; echo $a;", "cannot reassign");
}

/// Reference-aliased locals are never killable.
#[test]
fn test_ref_aliased_local_not_killable() {
    expect_error("<?php $a = 1; $r =& $a; unset($a); $a = \"x\";", "cannot reassign");
}

/// Static locals are never killable.
#[test]
fn test_static_local_not_killable() {
    expect_error("<?php function f() { static $a = 1; unset($a); $a = \"x\"; } f();", "cannot reassign");
}

/// Global-bound locals are never killable.
#[test]
fn test_global_local_not_killable() {
    expect_error("<?php $g = 1; function f() { global $g; unset($g); $g = \"x\"; } f();", "cannot reassign");
}

/// By-ref closure captures are never killable.
#[test]
fn test_by_ref_capture_not_killable() {
    expect_error("<?php $a = 1; $f = function() use (&$a) { return $a; }; unset($a); $a = \"x\";", "cannot reassign");
}

/// A local passed to a by-ref parameter is aliased from that point on.
#[test]
fn test_by_ref_call_arg_not_killable() {
    expect_error("<?php function f(&$x) { $x = 2; } $a = 1; f($a); unset($a); $a = \"s\";", "cannot reassign");
}

/// Declared-typed locals are a contract: never killable, in both modes.
#[test]
fn test_typed_local_not_killable() {
    expect_error("<?php int $a = 1; unset($a); $a = \"x\";", "cannot reassign");
}

/// Parameters with a declared type hint are a contract: never killable.
/// (An untyped parameter stays killable — pin that too.)
#[test]
fn test_typed_param_not_killable() {
    expect_error("<?php function f(int $a) { unset($a); $a = \"x\"; } f(1);", "cannot reassign");
    expect_no_error("<?php function f($a) { unset($a); $a = \"x\"; echo $a; } f(1);");
}

/// Class properties never reach the local retype paths: pin the declared-property error.
#[test]
fn test_typed_property_stays_strict() {
    expect_error("<?php class C { public string $s = \"a\"; } $c = new C(); $c->s = 5;", "");
}
```

For `test_typed_local_not_killable`, if the plain-`<?php` harness rejects the `int $a = 1;` extension syntax, use the fixture convention the existing `TypedAssign` error tests use (find them: `grep -rn "Typed local" tests/`). For `test_typed_property_stays_strict`, fill the expected substring from the real diagnostic the property path emits today (run the fixture once; the test pins that it errors — the exact message is whatever `assignments/properties.rs` produces).

- [x] **Step 2: Run to verify failures**

Run: `cargo test --test error_tests unset_then_retype`
Expected: FAIL (`cannot reassign` error still raised — the kill does not exist yet). Also run `cargo test --test error_tests read_after_unset` — FAIL (no error today). If any message substring differs from the real diagnostic (e.g. the exact undefined-variable wording), adjust the assertion to the real message, not vice versa.

- [x] **Step 3: Add Checker state and the predicate**

In `src/types/checker/mod.rs`, add the six fields listed in Interfaces (init empty/zero where the `Checker` is constructed), plus:

```rust
/// True when `name`'s current binding may be killed (by `unset`) or re-bound to an
/// incompatible type (by assignment) at the current program point. See
/// `.plans/local-retype-and-strict-locals.md` — depth-0 + non-aliased rule.
fn local_binding_is_killable(&self, name: &str) -> bool {
    self.local_conditional_depth == 0
        && self.local_binding_depth.get(name).copied().unwrap_or(0) == 0
        && !self.active_ref_params.contains(name)
        && !self.ref_aliased_locals.contains(name)
        && !self.active_globals.contains(name)
        && !self.static_local_names.contains(name)
        && !self.typed_local_names.contains(name)
}
```

Add both span sets to `CheckResult` (`src/types/result.rs` struct, `src/types/checker/mod.rs` assembly in `check_types_with_options`, next to where `warnings` is moved out of the checker).

- [x] **Step 4: Track conditional depth and per-body reset**

- In `src/types/checker/stmt_check.rs`, in the dispatch arm that routes `Foreach | Switch | If | DoWhile | While | For | Throw | Try` to `check_control_flow_stmt` (lines 102–109): increment `self.local_conditional_depth` before the call and decrement after (conditions get depth +1 too — conservative, acceptable).
- Per-body reset: find where each function/closure body check begins (grep `active_ref_params` seeding and the driver's per-function env construction; closure bodies via `grep -rn "capture_refs" src/types/checker`). At each body entry, save `local_conditional_depth`, `local_binding_depth`, `ref_aliased_locals`, `static_local_names`, `typed_local_names` with `std::mem::take`, reset depth to 0, and restore on exit. If `active_ref_params` is already seeded per-body from `ref_params`, piggyback on the same location.

- [x] **Step 5: Record binding depth and ref-alias sources**

- In `merge_local_assignment_type` (`locals.rs:749`): in the `else` (fresh insert) branch, also `checker.local_binding_depth.insert(name.to_string(), checker.local_conditional_depth);` — this requires changing the signature from `checker: &Checker` to `checker: &mut Checker` (single caller at `locals.rs:211`). Do the same recording in `check_typed_assign`'s env insert (`locals.rs:786`+).
- In `check_ref_assign` (`locals.rs:221`): every arm that inserts the target into `active_ref_params` also inserts the target into `ref_aliased_locals`; the `Variable` source arm additionally inserts the source name.
- Where by-ref closure captures are checked (found in Step 4's grep — `src/types/checker/inference/ops.rs:379-406` builds `closure_ref_params` including `capture_refs` and passes them to `with_local_storage_context`): insert each name into `ref_aliased_locals`, and use that same entry point for the closure-body save/restore of Step 4.
- By-ref CALL arguments: `grep -rn "ref_params" src/types/checker/inference` — at every site where a plain `$var` argument is matched against a by-ref parameter (user functions `inference/ops.rs:1137`/`:1174`, methods `inference/objects/methods.rs:1359`, constructors `inference/objects/constructors.rs:1193`, plus the builtin by-ref-argument path — locate it from how `sort()`-style builtins validate their by-ref array arg), insert the variable name into `ref_aliased_locals`. A reference can escape through the callee, so the alias is permanent for the body.
- In `src/types/checker/stmt_check/assignments.rs` `StmtKind::StaticVar` arm (line ~151): insert the name into `static_local_names`.
- Declared types: in `check_typed_assign` (`locals.rs:786`), insert the name into `typed_local_names`; where function parameters are seeded into the body env (find the site in `src/types/checker/driver/functions.rs`), insert every parameter that has a declared type hint (`param type != None`) into `typed_local_names`.

- [x] **Step 6: Add the unset-kill arm**

In `infer_type_with_assignment_effects` (`src/types/checker/inference/expr/effects.rs:44`), add an arm before the fallthrough (read `src/parser/ast/expr.rs:71` for the exact `FunctionCall` field names first):

```rust
ExprKind::FunctionCall { name, args, .. }
    if crate::names::php_symbol_key(name.trim_start_matches('\\')) == "unset" =>
{
    // Full validation (arity, probe legality) through the normal path first.
    let ty = self.infer_type(expr, env)?;
    // PHP grammar only allows `unset` in statement position, so this arm always
    // runs with the statement-level mutable env.
    for arg in args {
        if let ExprKind::Variable(var) = &arg.kind {
            if env.contains_key(var) && self.local_binding_is_killable(var) {
                env.remove(var);
                self.local_bind_kill_sites.insert(arg.span);
                self.local_binding_depth.remove(var);
            }
        }
    }
    Ok(ty)
}
```

(Adapt the `php_symbol_key` import path to the one `src/ir_lower/expr/function_calls.rs:56` uses.) Also clear the checker's per-name callable/reflection metadata for killed names the same way a rebinding assignment does — reuse the existing `clear_callable_metadata`-style helpers called around `locals.rs:400-448`.

- [x] **Step 7: Run tests**

Run: `cargo test --test error_tests -- test_unset test_read_after test_multi_arg test_conditional_unset test_branch_created test_ref_aliased test_static_local test_global_local test_by_ref_capture`
Expected: all PASS. Also run `cargo test --test error_tests` (whole suite) to catch regressions from the depth counter and signature change.

- [x] **Step 8: Commit**

```bash
git add src/types tests/error_tests
git commit -m "feat(checker): unset kills eligible depth-0 local bindings and records kill sites"
```

---

### Task 3: Checker — implicit retype: warning in permissive mode, error under `--strict-locals`

**Files:**
- Modify: `src/types/checker/stmt_check/assignments/locals.rs` (`merge_local_assignment_type`, `check_assign` threading)
- Modify: `tests/error_tests.rs` (strict helpers)
- Modify: `tests/error_tests/type_system.rs`, plus triage of every test matching `cannot reassign`

**Interfaces:**
- Consumes: `Checker.strict_locals`, `local_binding_is_killable`, `local_retype_sites`, `local_binding_depth` from Tasks 1–2.
- Produces: statement-form gating — `merge_local_assignment_type(checker, name, ty, span, env, stmt_form: bool)`; `check_assign` gains the same `stmt_form: bool` parameter, `true` only from the `StmtKind::Assign` statement path.
- Produces: test helpers `check_source_strict(src) -> Result<(), String>` and `expect_error_strict(src, substr)` in `tests/error_tests.rs`.

- [x] **Step 1: Write the failing tests**

Helpers in `tests/error_tests.rs` (mirror `check_source` exactly, ending with `types::check_with_options(&ast, elephc::types::CheckOptions { strict_locals: true })`):

```rust
/// Like `check_source` but with --strict-locals semantics.
fn check_source_strict(src: &str) -> Result<(), String> { /* same pipeline as check_source, strict options */ }

/// Verifies a snippet fails under --strict-locals with the given substring.
fn expect_error_strict(src: &str, expected_substr: &str) { /* same shape as expect_error over check_source_strict */ }
```

Tests in `tests/error_tests/type_system.rs`:

```rust
/// Permissive default: an incompatible depth-0 reassignment warns and re-binds.
#[test]
fn test_implicit_retype_warns_by_default() {
    expect_warning("<?php $a = 0; $a = \"ciao\"; echo $a;", "changes type from int to string");
}

/// --strict-locals restores the hard error.
#[test]
fn test_implicit_retype_errors_under_strict_locals() {
    expect_error_strict("<?php $a = 0; $a = \"ciao\"; echo $a;", "cannot reassign $a");
}

/// A compatible reassignment stays silent.
#[test]
fn test_compatible_reassign_has_no_warning() {
    expect_no_warning("<?php $a = 1; $a = 2;", "changes type");
}

/// unset-then-assign is a fresh binding, not a retype: no warning.
#[test]
fn test_unset_then_assign_has_no_warning() {
    expect_no_warning("<?php $a = 0; unset($a); $a = \"ciao\";", "changes type");
}

/// A conditional incompatible reassignment is not kill/rebind-eligible. The `$a++`
/// write also blocks Task 6's mixed-storage marking, so this fixture stays an error
/// through the whole plan (a plain conditional retype becomes legal in Task 6).
#[test]
fn test_conditional_retype_still_errors() {
    expect_error("<?php $a = 0; if ($argc > 1) { $a = \"x\"; } $a++;", "cannot reassign");
}

/// Interplay: a conditional unset leaves the binding alive, so a later depth-0
/// incompatible reassignment is an ordinary retype (warning), and it is sound:
/// the fresh slot is written on both paths.
#[test]
fn test_retype_after_conditional_unset_warns() {
    expect_warning("<?php $a = 1; if ($argc > 1) { unset($a); } $a = \"x\"; echo $a;", "changes type");
}

/// A ref-aliased local stays an error even in permissive mode.
#[test]
fn test_ref_aliased_retype_still_errors() {
    expect_error("<?php $a = 0; $r =& $a; $a = \"x\";", "cannot reassign");
}
```

- [x] **Step 2: Run to verify failures**

Run: `cargo test --test error_tests implicit_retype`
Expected: FAIL — `test_implicit_retype_warns_by_default` errors instead of warning.

- [x] **Step 3: Implement the retype hook**

In `merge_local_assignment_type` (`locals.rs:749`, already `&mut Checker` from Task 2), replace the `merged_ty.is_none()` early-return with:

```rust
if merged_ty.is_none() {
    if !checker.strict_locals && stmt_form && checker.local_binding_is_killable(name) {
        checker.warnings.push(crate::errors::CompileWarning::new(
            span,
            &format!(
                "${} changes type from {} to {}; the previous value is discarded (compile with --strict-locals to make this an error)",
                name, existing, ty
            ),
        ));
        checker.local_retype_sites.insert(span);
        checker.local_binding_depth.insert(name.to_string(), 0);
        env.insert(name.to_string(), ty.clone());
        return Ok(());
    }
    return Err(CompileError::new(
        span,
        &format!("Type error: cannot reassign ${} from {} to {}", name, existing, ty),
    ));
}
```

Thread `stmt_form: bool` from the callers: `grep -n "check_assign(" src/types/checker` — pass `true` only from the `StmtKind::Assign` statement path (`check_assignment_like_stmt`), `false` from expression-position assignment checking. On the retype path also clear the per-name callable/reflection metadata (same helpers as Task 2 Step 6). Match `CompileWarning::new`'s real signature (`src/errors/mod.rs:73`).

- [x] **Step 4: Triage existing `cannot reassign` tests**

Run: `grep -rn "cannot reassign" tests/`. For each hit (known: `tests/error_tests/type_system.rs:153`, `tests/error_tests/misc/syntax_misc.rs`, `tests/codegen/callables/closures.rs`): if the fixture is now eligible for permissive retype (depth-0, statement-form, unaliased), convert the assertion into the `expect_warning` + `expect_error_strict` pair; if it exercises an ineligible shape, keep it as `expect_error`. Do not weaken fixtures to preserve assertions — change the assertion to match the designed behavior.

- [x] **Step 5: Run tests**

Run: `cargo test --test error_tests`
Expected: PASS. Then `cargo test --test codegen_tests closures` (or the module owning the triaged closure test) — PASS.

- [x] **Step 6: Commit**

```bash
git add src/types tests
git commit -m "feat(checker): incompatible depth-0 local reassignment re-binds with a warning; --strict-locals keeps the error"
```

---

### Task 4: Lowering — thread recorded sites; `unset` abandons the slot; e2e unset-retype

**Files:**
- Modify: `src/ir_lower/mod.rs` (`lower_program*` at lines 47/57/67 — copy `check_result.local_bind_kill_sites` / `local_retype_sites` into `LoweringContext`)
- Modify: `src/ir_lower/context.rs` (`unset_local` at 1834)
- Create: `tests/codegen/locals_retype.rs` (+ register the module where siblings are registered)

**Interfaces:**
- Consumes: `CheckResult.local_bind_kill_sites` / `local_retype_sites` (Tasks 2–3).
- Produces: `LoweringContext.bind_kill_sites: HashSet<Span>`, `LoweringContext.retype_sites: HashSet<Span>` (retype consumed in Task 5).
- Produces: on a recorded kill, the name is removed from `local_slots`, `local_types`, `local_kinds` after the old value is released — the next `declare_local` mints a fresh slot.

- [x] **Step 1: Write the failing e2e tests**

`tests/codegen/locals_retype.rs`:

```rust
//! Purpose:
//! End-to-end coverage for local binding kill (`unset`) and retype-to-fresh-slot lowering.

use crate::support::*;

/// unset kills the int binding; the string reassignment gets a fresh heap-typed slot.
#[test]
fn test_unset_then_retype_int_to_string() {
    let out = compile_and_run("<?php $a = $argc; unset($a); $a = \"ciao\"; echo $a;");
    assert_eq!(out, "ciao");
}

/// unset releases the owned heap string before the int rebind (no leak, no UAF).
#[test]
fn test_unset_then_retype_string_to_int() {
    let out = compile_and_run("<?php $a = \"ciao\" . $argc; unset($a); $a = 7; echo $a;");
    assert_eq!(out, "7");
}

/// unset without a following reassignment still compiles and runs.
#[test]
fn test_unset_without_reassignment() {
    let out = compile_and_run("<?php $a = $argc; unset($a); echo \"ok\";");
    assert_eq!(out, "ok");
}
```

Register the module exactly like a sibling (see how `array_basics` is declared in the codegen test module tree).

- [x] **Step 2: Run to verify failure**

Run: `cargo test --test codegen_tests locals_retype`
Expected: `test_unset_then_retype_*` FAIL at compile-check stage only if Task 2/3 not merged — here they should fail in lowering/codegen (type mismatch or wrong output). Record the actual failure mode.

- [x] **Step 3: Thread the span sets**

In `src/ir_lower/mod.rs` `lower_program` / `_with_source_path` / `_and_web` (lines 47/57/67): clone both sets from `check_result` into the `LoweringContext` construction. Add the two fields to `LoweringContext` in `src/ir_lower/context.rs`, defaulting empty.

- [x] **Step 4: Implement the kill in `unset_local`**

In `unset_local` (`context.rs:1834`): when `span.is_some_and(|s| self.bind_kill_sites.contains(&s))` and the local has a slot:

- Non-ref-bound path: before today's `store_local(name, null, PhpType::Void, span)` behavior, first read `store_local` (`context.rs:1374`) to identify its previous-value release path and reuse it to release the old slot's owned value (the checker guarantees the slot is definitely initialized — depth-0 rule); then instead of null-storing, remove `name` from `local_slots`, `local_types`, `local_kinds` and return `null`. Keep the existing `clear_*` metadata calls.
- Ref-bound path: keep the existing release/`Op::UnsetLocal` sequence, then additionally remove the three map entries.
- Non-recorded spans: behavior unchanged (today's null-store).

- [x] **Step 5: Run the e2e tests**

Run: `cargo test --test codegen_tests locals_retype`
Expected: all three PASS (fresh slot is minted by `declare_local` on the next assignment because the name is unmapped, with the checker-approved new type).

- [x] **Step 6: Ownership verification**

Read `tests/fresh_result_ownership_leak_tests.rs` and add one analogous leak assertion for `test_unset_then_retype_string_to_int`'s shape (heap value released at the kill). If that harness doesn't transfer cleanly, instead run the string-to-int fixture under the runtime heap-debug mode used elsewhere in the repo (`grep -rn "heap-debug\|heap_debug" src/cli.rs tests/` for the invocation) and assert no leak/UAF diagnostics in its output.

- [x] **Step 7: Commit**

```bash
git add src/ir_lower tests/codegen
git commit -m "feat(lowering): recorded unset kill sites abandon the local slot after releasing its value"
```

---

### Task 5: Lowering — retype re-binds to a fresh slot; repr fixpoint reset; e2e retype

**Files:**
- Modify: `src/ir_lower/stmt/mod.rs` (`lower_assign` at line 129)
- Modify: `src/ir_lower/context.rs` (new `rebind_local_for_retype`)
- Modify: `src/ir_lower/stmt/repr_fixpoint.rs` (assign arm at line 329; unset handling)
- Test: `tests/codegen/locals_retype.rs`

**Interfaces:**
- Consumes: `LoweringContext.retype_sites` (Task 4), release path identified in Task 4 Step 4.
- Produces: `pub(crate) fn rebind_local_for_retype(&mut self, name: &str, span: Option<Span>)` on `LoweringContext` — releases the old slot's owned value and removes `name` from `local_slots`, `local_types`, `local_kinds` (same mechanics as the Task 4 kill, factored so both call one helper).

- [x] **Step 1: Write the failing e2e tests**

Append to `tests/codegen/locals_retype.rs`:

```rust
/// Implicit retype: int local re-binds to a fresh string slot.
#[test]
fn test_implicit_retype_int_to_string() {
    let out = compile_and_run("<?php $a = $argc; $a = \"ciao\"; echo $a;");
    assert_eq!(out, "ciao");
}

/// Implicit retype: heap string local re-binds to a fresh int slot (old value released).
#[test]
fn test_implicit_retype_string_to_int() {
    let out = compile_and_run("<?php $a = \"ciao\" . $argc; $a = 7; echo $a;");
    assert_eq!(out, "7");
}

/// The RHS of a retype assignment reads the OLD binding.
#[test]
fn test_retype_rhs_reads_old_value() {
    let out = compile_and_run("<?php $a = $argc; $a = \"n=\" . $a; echo $a;");
    assert_eq!(out, "n=1");
}

/// Retype after a loop that used the old binding.
#[test]
fn test_retype_after_loop() {
    let out = compile_and_run("<?php $a = 0; for ($i = 0; $i < $argc; $i++) { $a += $i; } $a = \"done\"; echo $a;");
    assert_eq!(out, "done");
}

/// A by-value closure capture keeps the old value across a later retype.
#[test]
fn test_closure_capture_before_retype() {
    let out = compile_and_run("<?php $a = $argc; $f = function() use ($a) { return $a; }; $a = \"x\"; echo $f() . $a;");
    assert_eq!(out, "1x");
}

/// Fully-constant retype (AST folding may pre-resolve it — output must match either way).
#[test]
fn test_constant_retype() {
    let out = compile_and_run("<?php $a = 3; $a = \"ciao\"; echo $a;");
    assert_eq!(out, "ciao");
}

/// Conditional unset then depth-0 retype: the fresh slot is written on both paths and
/// the old heap value is released exactly once whichever path ran (release must be
/// null-tolerant — the unset path may have nulled the old slot already).
#[test]
fn test_retype_after_conditional_unset_of_heap_local() {
    let out = compile_and_run("<?php $a = \"s\" . $argc; if ($argc > 1) { unset($a); } $a = 7; echo $a;");
    assert_eq!(out, "7");
}
```

If `compile_and_run` fixtures observe an `$argc` different from 1, adjust the expected literals to the harness's real value at Step 4 (check a sibling test that prints `$argc`).

- [x] **Step 2: Run to verify failure**

Run: `cargo test --test codegen_tests locals_retype`
Expected: the new tests FAIL (wrong output or codegen panic — the store still targets the old slot).

- [x] **Step 3: Implement the rebind**

- Factor the Task 4 kill mechanics (release + three map removals) into `rebind_local_for_retype`; make `unset_local`'s kill path call it.
- In `lower_assign` (`src/ir_lower/stmt/mod.rs:129`): read the function; after the RHS value is lowered and before the store, insert:

```rust
if ctx.retype_sites.contains(&span) {
    ctx.rebind_local_for_retype(name, Some(span));
}
```

(RHS first: `$a = "n=" . $a` must read the old slot.) The subsequent store path then calls `declare_local` on the unmapped name and mints the fresh slot with the new type.

- [x] **Step 4: Teach the repr fixpoint about kills**

Read `src/ir_lower/stmt/repr_fixpoint.rs` (assign arm at line 329). Where the pass accumulates/merges a storage representation per name across assignments, add: if the assignment's `stmt.span` is in `retype_sites`, replace the accumulated repr for `name` with the new assignment's repr instead of merging; where the pass scans statements, treat an `unset` argument whose span is in `bind_kill_sites` as ending the name's accumulation (start fresh at the next assignment). Mirror the pass's existing structure — if it has unit tests in `src/ir_lower/tests/`, extend them with one retype case.

- [x] **Step 5: Run the tests**

Run: `cargo test --test codegen_tests locals_retype`
Expected: all PASS. Then run the neighbouring representative suites the change could disturb: `cargo test --test codegen_tests array_basics` and `cargo test --test error_tests`.

- [x] **Step 6: Commit**

```bash
git add src/ir_lower tests/codegen
git commit -m "feat(lowering): recorded retype sites re-bind locals to a fresh slot with repr-fixpoint reset"
```

---

### Task 6: Checker — mixed-storage pre-scan for branch-divergent locals

**Files:**
- Create: `src/types/checker/mixed_storage_scan.rs`
- Modify: `src/types/checker/mod.rs` (module decl, `mixed_storage_locals: HashSet<String>` per-body field, `mixed_storage_store_sites: HashSet<Span>` cumulative field, CheckResult assembly)
- Modify: `src/types/result.rs` (CheckResult field)
- Modify: `src/types/checker/stmt_check/assignments/locals.rs` (first insert honors marking)
- Modify: the per-body entry points from Task 2 Step 4 (driver function loop, `with_local_storage_context` for closures) — run the scan there
- Test: `tests/error_tests/type_system.rs`

**Interfaces:**
- Consumes: `Checker.strict_locals`, per-body exclusion sets and the depth definition from Task 2; `expect_error_strict` from Task 3; `infer_expr_type_syntactic` (`src/types/checker/inference/syntactic.rs:282`); `merged_assignment_type` (`src/types/checker/type_compat/unions.rs:170`).
- Produces: `impl Checker { fn run_mixed_storage_scan(&mut self, body: &[Stmt], params: &[FnParamInfo]) }` — fills `mixed_storage_locals` for this body, appends to `mixed_storage_store_sites`, and pushes one `CompileWarning` per marked name (permissive mode only; no-op under `strict_locals`). The exact param-info type mirrors whatever the driver already has in hand at the body entry point — the scan only needs each param's name, by-ref flag, and whether it has a type hint.
- Produces: `CheckResult.mixed_storage_store_sites: HashSet<Span>`.

- [x] **Step 1: Write the failing tests**

In `tests/error_tests/type_system.rs`:

```rust
/// Branch-divergent assignment is accepted via whole-frame Mixed storage.
#[test]
fn test_branch_divergent_assignment_is_accepted() {
    expect_no_error("<?php if ($argc > 1) { $a = 0; } else { $a = \"ciao\"; } echo $a;");
}

/// Permissive default: the marking warns once, naming the boxed compilation.
#[test]
fn test_branch_divergent_assignment_warns() {
    expect_warning("<?php if ($argc > 1) { $a = 0; } else { $a = \"ciao\"; } echo $a;", "boxed mixed storage");
}

/// --strict-locals disables the pre-scan: the divergent assignment errors as today.
#[test]
fn test_branch_divergent_assignment_errors_under_strict() {
    expect_error_strict("<?php if ($argc > 1) { $a = 0; } else { $a = \"ciao\"; } echo $a;", "cannot reassign");
}

/// Single-branch retype of an outer binding is also handled by marking.
#[test]
fn test_single_branch_retype_of_outer_binding_is_accepted() {
    expect_no_error("<?php $a = 0; if ($argc > 1) { $a = \"x\"; } echo $a;");
}

/// Heterogeneous loop-carried local is handled by marking.
#[test]
fn test_loop_carried_heterogeneous_local_is_accepted() {
    expect_no_error("<?php $a = 0; for ($i = 0; $i < $argc; $i++) { $a = \"s\"; } echo $a;");
}

/// Declared-typed locals are never marked (contract wins in both modes).
#[test]
fn test_typed_local_never_mixed() {
    expect_error("<?php int $a = 0; if ($argc > 1) { $a = \"x\"; }", "cannot reassign");
}

/// Ref-aliased locals are never marked.
#[test]
fn test_ref_aliased_never_mixed() {
    expect_error("<?php $a = 0; $r =& $a; if ($argc > 1) { $a = 1; } else { $a = \"x\"; }", "cannot reassign");
}

/// A non-Assign write (++) blocks marking: the divergent assignment stays an error.
#[test]
fn test_incdec_write_blocks_marking() {
    expect_error("<?php $a = 0; if ($argc > 1) { $a = \"x\"; } $a++;", "cannot reassign");
}

/// unset anywhere in the body blocks marking: the name stays unmarked, so the
/// else-branch assignment errors exactly as today (before the unset is even reached).
#[test]
fn test_unset_blocks_marking() {
    expect_error("<?php if ($argc > 1) { $a = 0; } else { $a = \"ciao\"; } unset($a); echo $a;", "cannot reassign");
}
```

For `test_typed_local_never_mixed`, reuse the fixture convention chosen in Task 2 for extension syntax.

- [x] **Step 2: Run to verify failures**

Run: `cargo test --test error_tests branch_divergent`
Expected: FAIL — the accept/warn cases error with `cannot reassign` today.

- [x] **Step 3: Implement the scan module**

`src/types/checker/mixed_storage_scan.rs`: a single recursive walk over `&[Stmt]` carrying a `depth: u32` (increment for the bodies of `If`/`Switch`/`While`/`DoWhile`/`For`/`Foreach`/`Try` — same depth definition as Task 2's runtime counter). Collect per name:

- `assigns: Vec<(PhpType, u32, Span)>` from `StmtKind::Assign` (type via `infer_expr_type_syntactic(value)`),
- `disqualified: bool` — set by any other write or excluded shape: `StmtKind::TypedAssign`, incdec/compound-assign statements or expressions targeting the name, expression-form `ExprKind::Assignment`, foreach value/key targets, `list()` targets, `unset()` mention, `=&` target or source, `use (&$x)` capture, `global`, `static`, and plain-variable arguments in call positions whose callee signature (from `checker.fn_decls` / method availability) declares that parameter by-ref — when the callee cannot be resolved statically, disqualify conservatively.

Enumerate the write-capable `StmtKind`/`ExprKind` variants by reading `src/parser/ast/stmt.rs` and `src/parser/ast/expr.rs` — the walk must match on every variant that can write a local, and default-recurse through the rest.

Mark `name` when: not disqualified, params say it is neither by-ref nor type-hinted (or not a param at all), and some pair in `assigns` fails `self.merged_assignment_type(a, b)` (both directions) with at least one member of a failing pair at depth > 0. On marking (permissive mode only): insert into `mixed_storage_locals`, extend `mixed_storage_store_sites` with ALL of the name's assign spans, and push the warning from Global Constraints (naming the first failing pair's two types) at the first failing pair's later span. Under `strict_locals` the scan returns without marking anything.

- [x] **Step 4: Honor the marking at bind time**

In `merge_local_assignment_type` (`locals.rs`), in the fresh-insert `else` branch: if `checker.mixed_storage_locals.contains(name)`, insert `PhpType::Mixed` instead of `ty.clone()` (binding depth recording unchanged). Compatible and incompatible reassignments then merge trivially (`Mixed` absorbs everything), so neither the retype hook nor the error fires for marked names. Invoke `run_mixed_storage_scan` at every per-body entry point from Task 2 Step 4, right after the per-body state reset; add `mixed_storage_locals` to the save/restore set. Assemble `mixed_storage_store_sites` into `CheckResult` next to the other two span sets.

- [x] **Step 5: Run tests**

Run: `cargo test --test error_tests` (whole suite — the marking must not disturb Tasks 2–3 behavior; their negative tests are the guard). Fill the two deferred assertion substrings from real diagnostics.

- [x] **Step 6: Commit**

```bash
git add src/types tests/error_tests
git commit -m "feat(checker): syntactic pre-scan compiles branch-divergent locals as boxed mixed storage"
```

---

### Task 7: Lowering — box marked locals from the first store; e2e branch-divergent

**Files:**
- Modify: `src/ir_lower/mod.rs` (thread `mixed_storage_store_sites` into `LoweringContext` next to the Task 4 sets)
- Modify: `src/ir_lower/stmt/mod.rs` (`lower_assign` pre-declare hook)
- Test: `tests/codegen/locals_retype.rs`

**Interfaces:**
- Consumes: `CheckResult.mixed_storage_store_sites` (Task 6), `LoweringContext` span-set pattern from Task 4, `has_local_slot` / `declare_local` (`src/ir_lower/context.rs:658/663`).
- Produces: `LoweringContext.mixed_storage_store_sites: HashSet<Span>`; marked locals get a `PhpType::Mixed` slot before their first store.

- [x] **Step 1: Write the failing e2e tests**

Append to `tests/codegen/locals_retype.rs`:

```rust
/// Branch-divergent local: the else arm runs (argc == 1) and prints the string.
#[test]
fn test_branch_divergent_local_runs_else_arm() {
    let out = compile_and_run("<?php if ($argc > 1) { $a = 0; } else { $a = \"ciao\"; } echo $a;");
    assert_eq!(out, "ciao");
}

/// Single-branch retype, branch taken: the Mixed slot holds the string.
#[test]
fn test_single_branch_retype_taken() {
    let out = compile_and_run("<?php $a = 41; if ($argc > 0) { $a = \"ciao\"; } echo $a;");
    assert_eq!(out, "ciao");
}

/// Single-branch retype, branch NOT taken: the Mixed slot still holds the boxed int.
/// This is the load-bearing test — both dynamic outcomes flow through one slot.
#[test]
fn test_single_branch_retype_not_taken() {
    let out = compile_and_run("<?php $a = 41; if ($argc > 5) { $a = \"ciao\"; } echo $a;");
    assert_eq!(out, "41");
}

/// Heterogeneous loop-carried local: each iteration re-boxes; previous value released.
#[test]
fn test_loop_carried_heterogeneous_local() {
    let out = compile_and_run("<?php $a = 0; for ($i = 0; $i < $argc; $i++) { $a = \"s\" . $i; } echo $a;");
    assert_eq!(out, "s0");
}
```

- [x] **Step 2: Run to verify failure**

Run: `cargo test --test codegen_tests locals_retype`
Expected: the new tests FAIL (the first store declares an unboxed slot; the divergent store then mismatches or miscompiles).

- [x] **Step 3: Implement the pre-declare hook**

Thread the set in `src/ir_lower/mod.rs` exactly like Task 4's sets. In `lower_assign` (`src/ir_lower/stmt/mod.rs:129`), next to the Task 5 retype hook, before the store:

```rust
if ctx.mixed_storage_store_sites.contains(&span) && !ctx.has_local_slot(name) {
    ctx.declare_local(name, PhpType::Mixed);
}
```

Subsequent stores find the existing Mixed slot and box the value through the existing Mixed-storage store path (the same path the string-incdec contract exercises; the previous boxed value is released by the `release_previous` branch in `store_mutated_local_impl`, `src/ir_lower/context.rs:1814-1820`).

- [x] **Step 4: Verify the repr fixpoint is stable on marked names**

Read the `repr_fixpoint.rs` assign arm with a marked-name example: the slot starts `Mixed`; confirm the pass's merge keeps `Mixed` (it must never narrow a declared slot). If its per-name accumulation starts from the assignment's value type instead of the declared slot type, seed marked names with `Mixed` using the same span-set check. Extend its unit tests (in `src/ir_lower/tests/`) with one marked-name case if the pass has them.

- [x] **Step 5: Run the tests**

Run: `cargo test --test codegen_tests locals_retype` — all PASS, including the Task 4/5 tests (no regression). Then `cargo test --test error_tests`.

- [x] **Step 6: Ownership verification**

Extend the Task 4 Step 6 leak-check approach to `test_loop_carried_heterogeneous_local` (a boxed heap string overwritten each iteration is the release-pressure case).

- [x] **Step 7: Commit**

```bash
git add src/ir_lower tests/codegen
git commit -m "feat(lowering): pre-declare mixed-storage locals as boxed slots at recorded store sites"
```

---

### Task 8: eval parity check and documentation

**Files:**
- Test: one eval-path test (locate the harness below)
- Modify: docs pages for CLI flags and the local typing rule (located below)

**Interfaces:**
- Consumes: the shipped behavior of Tasks 1–7.

- [x] **Step 1: Write the eval parity test**

Read `tests/eval_string_interpolation_tests.rs` to learn the eval test harness, then add (in that style, in the file the harness convention dictates):

```rust
/// eval'd code retypes locals dynamically; permissive AOT now matches it.
#[test]
fn test_eval_local_retype_matches_aot() {
    let out = compile_and_run("<?php eval('$a = 1; $a = \"ciao\"; echo $a;');");
    assert_eq!(out, "ciao");
}
```

Run: `cargo test eval_local_retype` — if it already passed before this plan (magician is dynamically typed), keep it as a pinned regression test; note the observed prior behavior in the test doc comment.

- [x] **Step 2: Update docs**

- Run `grep -rn "strict-php" docs/ README.md` — add `--strict-locals` to every surface that lists compiler flags, with the one-line semantics from the CLI help.
- Run `grep -rni "reassign" docs/` — find the page documenting the monomorphic-local rule; rewrite it to describe: permissive default (warning + fresh binding for straight-line retype; warning + whole-frame boxed Mixed storage for branch-divergent assignment, with its dispatch cost), the eligibility rules (depth-0/unaliased for kill/rebind; the pre-scan markability guards for Mixed), `unset` as explicit kill, `--strict-locals`, and the declared-type exemption (typed locals, typed params, and properties stay strict in both modes). State explicitly that eval'd code is interpreted dynamically and is not gated by `--strict-locals`.

- [x] **Step 3: Verification sweep**

Run the touched suites once more: `cargo test --test error_tests`, `cargo test --test codegen_tests locals_retype`, `cargo test strict_locals`. Report results; the user runs the full pass.

- [x] **Step 4: Commit**

```bash
git add tests docs README.md
git commit -m "test(eval): pin local-retype parity; docs: document --strict-locals and permissive local retyping"
```

---

## Self-Review Notes

- Spec coverage: unset-kill (Task 2 checker, Task 4 lowering), implicit retype + warning (Task 3 checker, Task 5 lowering), branch-divergent Mixed storage (Task 6 checker, Task 7 lowering), `--strict-locals` (Task 1 flag, Task 3/6 semantics), declared-type exemption (Task 2 `typed_local_names` + property pinning test), soundness exclusions (Task 2 predicate, Task 6 markability guards, negative tests throughout), eval/docs (Task 8). Ref-aliasing now includes by-ref call arguments (Task 2 Step 5).
- Mechanism precedent: the Mixed-storage path deliberately mirrors the shipped string-incdec contract (checker types the local Mixed, lowering boxes the slot from the first store) — Task 7 reuses the existing Mixed-slot store/release paths rather than inventing new ones.
- Cross-task interplay pinned by tests: a marked name never reaches the retype hook (marking dominates — Task 6 Step 4); the Task 3 conditional-retype fixture includes `$a++` so it stays an error after Task 6; conditional-unset-then-retype is a warning with a both-paths-sound fresh slot (T3 test + T5 e2e heap variant); depth-0-only conflicts stay on the fresh-slot path (better codegen) because the pre-scan requires a depth>0 member in a failing pair.
- Known open risks, each pinned to a verification step: exact undefined-variable message (T2S2), `CompileWarning::new` signature (T3S3), `$argc` value under `compile_and_run` (T5S1), release-path reuse inside `store_local` (T4S4), repr-fixpoint internals (T5S4, T7S4), per-body state reset locations (T2S4), builtin by-ref-arg site (T2S5), extension-syntax fixture convention for typed locals (T2S1), the property-error substring (T2S1). Each step says what to read and what rule to apply — none is a design unknown.
- Type consistency: field and helper names are used identically across tasks (`local_bind_kill_sites` / `local_retype_sites` / `mixed_storage_store_sites` / `mixed_storage_locals` / `typed_local_names` / `bind_kill_sites` / `retype_sites` / `rebind_local_for_retype` / `local_binding_is_killable` / `run_mixed_storage_scan` / `CheckOptions` / `check_with_options` / `check_with_target_and_options` / `expect_error_strict`).
