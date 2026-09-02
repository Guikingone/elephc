//! Purpose:
//! Validates statement control flow behavior.
//! Keeps control-flow and assignment effects synchronized with expression inference and return analysis.
//!
//! Called from:
//! - `crate::types::checker::stmt_check`
//!
//! Key details:
//! - Branch and loop handling must preserve PHP execution order and conservative type environments.
//! - A `foreach` over a PHP-visible non-iterable (`int`, `string`, `bool`, `null`, `float`,
//!   `resource`) is a WARNING, not an error: php-src raises
//!   `foreach() argument must be of type array|object, <type> given` at runtime and keeps
//!   going, so rejecting it would make elephc refuse a program PHP runs. Codegen emits the
//!   matching runtime warning and skips the loop
//!   (`IteratorSourceKind::NonIterable` in `crate::codegen::lower_inst::iterators`).
//!   Compiler-internal types with no PHP spelling stay a hard error.

use crate::errors::CompileError;
use crate::parser::ast::{BinOp, Expr, ExprKind, StaticReceiver, Stmt, StmtKind};
use crate::termination::{block_terminal_effect_with_divergence, TerminalEffect};
use crate::types::{PhpType, TypeEnv};

use super::super::Checker;

const FS_CURRENT_AS_SELF: i64 = 16;
const FS_CURRENT_AS_PATHNAME: i64 = 32;
const FS_CURRENT_MODE_MASK: i64 = 240;
const FS_SKIP_DOTS: i64 = 4096;

/// Computes and records fixed-point array storage contracts before checking a loop body.
///
/// The shared analysis iterates over rebinds and growth sites with an evolving environment, so
/// cascading promotions, non-literal RHSs, and raw-to-raw element changes converge before any
/// header/body read is checked. EIR lowering later consumes the recorded contract for the same
/// loop span rather than repeating expression inference.
fn stabilize_loop_storage(
    checker: &mut Checker,
    loop_span: crate::span::Span,
    body: &[Stmt],
    update: Option<&Stmt>,
    env: &mut TypeEnv,
) {
    let key = (checker.current_loop_storage_scope.clone(), loop_span);
    if let Some(recorded) = checker.loop_storage_types.get(&key).cloned() {
        for (name, storage_type) in recorded {
            env.insert(name, storage_type);
        }
        return;
    }
    let snapshot = env.clone();
    let mut call_types: std::collections::HashMap<crate::span::Span, PhpType> =
        std::collections::HashMap::new();
    let contracts = crate::types::checker::loop_carried_storage_types(
        body,
        update,
        &snapshot,
        &mut |expr, analysis_env| {
            let is_call = matches!(
                expr.kind,
                ExprKind::FunctionCall { .. }
                    | ExprKind::MethodCall { .. }
                    | ExprKind::StaticMethodCall { .. }
                    | ExprKind::ClosureCall { .. }
                    | ExprKind::ExprCall { .. }
            );
            // The memo is keyed by span, so it may only answer for a span that names one call.
            // Under `Span::dummy()` every method / static / closure call in a prelude loop body
            // shared a single entry and the FIRST call's inferred type was handed to all of
            // them — 42 times while checking one `new PDO("sqlite::memory:")` program.
            let memoizable = is_call && expr.span.identifies_a_node();
            if memoizable {
                if let Some(cached) = call_types.get(&expr.span) {
                    return Some(cached.clone());
                }
            }
            let inferred = checker.infer_type(expr, analysis_env).ok()?;
            if memoizable {
                call_types.insert(expr.span, inferred.clone());
            }
            Some(inferred)
        },
    );
    let recorded = checker.loop_storage_types.entry(key).or_default();
    for (name, storage_type) in contracts {
        recorded.insert(name.clone(), storage_type.clone());
        env.insert(name, storage_type);
    }
}

/// Restores a narrowed variable in the environment to its previously saved type after a guarded
/// branch, removing it when it had no prior type. Used to keep `if`/`else` type narrowing scoped
/// to its branch.
fn restore_narrowed_var(env: &mut TypeEnv, var: &str, saved: &Option<PhpType>) {
    match saved {
        Some(ty) => {
            env.insert(var.to_string(), ty.clone());
        }
        None => {
            env.remove(var);
        }
    }
}

/// Names a `foreach` source that PHP accepts but can never iterate, or `None` when the type
/// has no PHP-visible spelling and must stay a hard compile error.
///
/// The names match what php-src prints in `foreach() argument must be of type array|object,
/// <name> given` (captured from PHP 8.5.6): `int`, `float`, `string`, `null`, `resource`, and
/// `true`/`false` for booleans. Only `PhpType::False` pins the boolean value at compile time;
/// a general `bool` is reported as `bool` here, while the RUNTIME warning emitted by
/// `__rt_warn_foreach_non_iterable` always prints the real `true`/`false`.
fn non_iterable_foreach_argument_name(ty: &PhpType) -> Option<&'static str> {
    match ty {
        PhpType::Int => Some("int"),
        PhpType::Float => Some("float"),
        PhpType::Str => Some("string"),
        PhpType::False => Some("false"),
        PhpType::Bool => Some("bool"),
        PhpType::Void => Some("null"),
        PhpType::Resource(_) => Some("resource"),
        _ => None,
    }
}

/// Returns the synthetic constructor default flags for filesystem iterators.
fn filesystem_iterator_default_flags(class_name: &str) -> Option<i64> {
    match class_name {
        "FilesystemIterator" => Some(FS_SKIP_DOTS),
        "GlobIterator" | "RecursiveDirectoryIterator" => Some(0),
        _ => None,
    }
}

/// Joins the bindings from every branch that can reach the statement after a conditional.
///
/// A real PHP local absent from one reachable branch remains a possible runtime binding but has no
/// safe precise type, so it is widened to `Mixed`. Synthetic property-narrowing keys are different:
/// absence means the fact is not common to every path, so those keys are omitted. Bindings present
/// everywhere are widened to the normalized union of their exit types.
fn join_fallthrough_type_envs(checker: &Checker, branches: &[TypeEnv]) -> Option<TypeEnv> {
    branches.first()?;
    let mut names = branches
        .iter()
        .flat_map(|branch| branch.keys().cloned())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();

    let mut joined = TypeEnv::new();
    for name in names {
        let mut types = Vec::with_capacity(branches.len());
        for branch in branches {
            if let Some(ty) = branch.get(&name) {
                types.push(ty.clone());
            }
        }

        if types.len() != branches.len() {
            if !name.starts_with('\u{1}') {
                joined.insert(name, PhpType::Mixed);
            }
        } else {
            joined.insert(name, join_fallthrough_binding_types(checker, &types));
        }
    }
    Some(joined)
}

/// Joins one binding's types across reachable branch exits without losing container shape.
///
/// A compatible supertype wins when it accepts every observed value. Differing array element
/// types widen inside the same array container, because representing them as a union of whole
/// arrays would collapse to boxed `Mixed` storage in codegen and make subsequent element access
/// or reference binding lose the fact that the runtime value is definitely an array.
fn join_fallthrough_binding_types(checker: &Checker, types: &[PhpType]) -> PhpType {
    let mut joined = types.first().cloned().unwrap_or(PhpType::Never);
    for next in types.iter().skip(1) {
        joined = if checker.type_accepts(&joined, next) {
            joined
        } else if checker.type_accepts(next, &joined) {
            next.clone()
        } else {
            match (&joined, next) {
                (PhpType::Array(left), PhpType::Array(right)) => PhpType::Array(Box::new(
                    PhpType::widen_array_branch_element((**left).clone(), (**right).clone()),
                )),
                (
                    PhpType::AssocArray {
                        key: left_key,
                        value: left_value,
                    },
                    PhpType::AssocArray {
                        key: right_key,
                        value: right_value,
                    },
                ) => PhpType::AssocArray {
                    key: Box::new(PhpType::widen_array_branch_element(
                        (**left_key).clone(),
                        (**right_key).clone(),
                    )),
                    value: Box::new(PhpType::widen_array_branch_element(
                        (**left_value).clone(),
                        (**right_value).clone(),
                    )),
                },
                _ => checker.normalize_union_type(vec![joined, next.clone()]),
            }
        };
    }
    joined
}

impl Checker {
    /// Validates control-flow statements and updates the type environment for their assignment effects.
    ///
    /// Dispatches to specific handlers for `foreach`, `switch`, `if`, `do-while`, `while`, `for`,
    /// `throw`, and `try` constructs. Each handler infers expression types, binds loop/scoped
    /// variables to their PHP-determined types, tracks `break`/`continue` depth, and accumulates
    /// errors for malformed or incompatible constructs. Returns `Ok(())` only when all checks pass.
    pub(crate) fn check_control_flow_stmt(
        &mut self,
        stmt: &crate::parser::ast::Stmt,
        env: &mut TypeEnv,
    ) -> Result<(), CompileError> {
        match &stmt.kind {
            StmtKind::Foreach {
                array,
                key_var,
                value_var,
                value_by_ref,
                body,
            } => {
                // `foreach ($arr as &$v)` takes a reference into each element, so BOTH names it
                // touches are reference-aliased for the rest of the body and neither binding can
                // be killed or re-bound — releasing or abandoning that storage would strand the
                // element references.
                //
                // The ITERABLE is aliased because the loop holds references into its elements.
                // `$v` is aliased because it IS one of those references: PHP leaves it bound to
                // the last element after the loop ends, so a post-loop `$v = "changed"` writes
                // through into `$arr`. The conditional-depth rule does NOT cover it — a `$v`
                // already assigned at depth 0 ABOVE the loop keeps that depth-0 binding, so
                // `local_binding_is_killable` answered true and the permissive path re-bound a
                // name lowering had ref-bound (`mark_ref_bound_local` in
                // `crate::ir_lower::stmt::typed_foreach`). Lowering then refuses the re-bind and
                // degrades to a store through the ref cell at the CELL's type, which for
                // `int` cell + `string` value has no coercion at all: a program the strict
                // checker rejects became one that miscompiles. Marking `$v` here is the same
                // permanent marking a `=&` target receives, and restores the hard error.
                //
                // Recorded before the iterable is inferred so an iterable that fails to type is
                // still treated as aliased.
                if *value_by_ref {
                    self.record_reference_alias_root(array);
                    self.ref_aliased_locals.insert(value_var.clone());
                }
                let arr_ty = self.infer_type_with_assignment_effects(array, env)?;
                if let PhpType::Array(elem_ty) = &arr_ty {
                    if let Some(k) = key_var {
                        // A genuinely packed array has int keys; an UNKNOWN-element array (an
                        // `array`-hinted param/property, elements known only to phpdoc) may be
                        // associative at runtime, so its keys are Mixed (ward-http's
                        // `foreach ($headers as $name => $values)` with string keys).
                        let key_ty = if matches!(elem_ty.as_ref(), PhpType::Mixed) {
                            PhpType::Mixed
                        } else {
                            PhpType::Int
                        };
                        env.insert(k.clone(), key_ty);
                        self.clear_foreach_callable_metadata(k);
                    }
                    let value_ty = *elem_ty.clone();
                    env.insert(value_var.clone(), value_ty.clone());
                    self.update_foreach_callable_metadata(value_var, array, &value_ty);
                } else if let PhpType::AssocArray { key, value } = &arr_ty {
                    if let Some(k) = key_var {
                        env.insert(k.clone(), *key.clone());
                        self.clear_foreach_callable_metadata(k);
                    }
                    let value_ty = *value.clone();
                    env.insert(value_var.clone(), value_ty.clone());
                    self.update_foreach_callable_metadata(value_var, array, &value_ty);
                } else if let PhpType::Object(class_name) = &arr_ty {
                    let is_iter = self.class_implements_interface(class_name, "Iterator")
                        || self.interface_extends_interface(class_name, "Iterator");
                    let is_iter_agg = self
                        .class_implements_interface(class_name, "IteratorAggregate")
                        || self.interface_extends_interface(class_name, "IteratorAggregate");
                    let is_traversable_marker = class_name
                        .trim_start_matches('\\')
                        .eq_ignore_ascii_case("Traversable")
                        || self.interface_extends_interface(class_name, "Traversable");
                    let (key_ty, value_ty) = if !is_iter && !is_iter_agg && !is_traversable_marker {
                        if !self.current_class.as_deref().is_some_and(|current| {
                            crate::names::php_symbol_key(current)
                                == crate::names::php_symbol_key(class_name)
                        }) {
                            return Err(CompileError::new(
                                stmt.span,
                                &format!(
                                    "foreach over object requires {} to implement Iterator or IteratorAggregate outside its declaring scope",
                                    class_name
                                ),
                            ));
                        }
                        (PhpType::Str, PhpType::Mixed)
                    } else {
                        self.foreach_object_key_value_types(class_name, array)
                    };
                    if let Some(k) = key_var {
                        env.insert(k.clone(), key_ty);
                        self.clear_foreach_callable_metadata(k);
                    }
                    env.insert(value_var.clone(), value_ty);
                    self.clear_foreach_callable_metadata(value_var);
                } else if matches!(
                    arr_ty,
                    PhpType::Iterable | PhpType::Mixed | PhpType::Union(_)
                ) {
                    if let Some(k) = key_var {
                        env.insert(k.clone(), PhpType::Mixed);
                        self.clear_foreach_callable_metadata(k);
                    }
                    env.insert(value_var.clone(), PhpType::Mixed);
                    self.clear_foreach_callable_metadata(value_var);
                } else if let Some(type_name) = non_iterable_foreach_argument_name(&arr_ty) {
                    // php-src does NOT reject this: `ZEND_FE_RESET_R` raises
                    // `foreach() argument must be of type array|object, <type> given`
                    // (E_WARNING), skips the loop body, and execution continues. Mirroring
                    // that as a hard error would make elephc reject a program PHP runs, so
                    // the diagnostic is a compile warning and codegen emits the same
                    // runtime warning (`IteratorSourceKind::NonIterable` in
                    // `src/codegen/lower_inst/iterators.rs`). Compiler-internal types
                    // (`Packed`, `Pointer`, `Buffer`, `Never`, `Callable`, `TaggedScalar`)
                    // have no PHP-visible spelling and stay a hard error below.
                    self.warnings.push(crate::errors::CompileWarning::new(
                        stmt.span,
                        &format!(
                            "foreach() argument must be of type array|object, {} given; the loop body will never run",
                            type_name
                        ),
                    ));
                    // The body is still type-checked, so bind both loop variables the way
                    // the `Mixed` source path does.
                    if let Some(k) = key_var {
                        env.insert(k.clone(), PhpType::Mixed);
                        self.clear_foreach_callable_metadata(k);
                    }
                    env.insert(value_var.clone(), PhpType::Mixed);
                    self.clear_foreach_callable_metadata(value_var);
                } else {
                    return Err(CompileError::new(
                        stmt.span,
                        "foreach requires an array, iterable, or an object implementing Iterator/IteratorAggregate",
                    ));
                }
                // A foreach key is a boxed `Mixed` cell at runtime regardless of
                // the source array's key type, so record the bound name so that a
                // `$dst[$k] = $v` write under it defers to `Op::ArraySetMixedKey`
                // instead of promoting the destination to `AssocArray`.
                if let Some(k) = key_var {
                    self.foreach_key_locals.insert(k.clone());
                }
                if *value_by_ref && matches!(arr_ty, PhpType::Object(_) | PhpType::Iterable) {
                    return Err(CompileError::new(
                        stmt.span,
                        "by-reference foreach over Iterator/IteratorAggregate objects or iterable-typed values is not supported; use an array source or remove &",
                    ));
                }
                // Widen after the key/value bindings are in the environment so a push of
                // the foreach value variable joins with its real element type.
                stabilize_loop_storage(self, stmt.span, body, None, env);
                let errors = self.check_break_continue_target_body(body, env);
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(CompileError::from_many(errors))
                }
            }
            StmtKind::Switch {
                subject,
                cases,
                default,
            } => {
                self.infer_type_with_assignment_effects(subject, env)?;
                let mut errors = Vec::new();
                for (values, _) in cases {
                    for v in values {
                        self.infer_type_with_assignment_effects(v, env)?;
                    }
                }
                let direct_entry_env = env.clone();
                let mut switch_exit_envs = Vec::new();
                let mut fallthrough_env = None;
                self.break_continue_depth += 1;
                for (_, body) in cases {
                    let mut entry_envs = vec![direct_entry_env.clone()];
                    if let Some(previous) = fallthrough_env.take() {
                        entry_envs.push(previous);
                    }
                    let mut branch_env = join_fallthrough_type_envs(self, &entry_envs)
                        .unwrap_or_else(|| direct_entry_env.clone());
                    errors.extend(self.check_body(body, &mut branch_env));
                    match block_terminal_effect_with_divergence(body, &|expr| {
                        self.expr_is_declared_never_call(expr)
                    }) {
                        TerminalEffect::FallsThrough => fallthrough_env = Some(branch_env),
                        TerminalEffect::Breaks | TerminalEffect::TerminatesMixed => {
                            switch_exit_envs.push(branch_env);
                        }
                        TerminalEffect::ExitsCurrentBlock => {}
                    }
                }
                if let Some(body) = default {
                    let mut entry_envs = vec![direct_entry_env.clone()];
                    if let Some(previous) = fallthrough_env.take() {
                        entry_envs.push(previous);
                    }
                    let mut branch_env = join_fallthrough_type_envs(self, &entry_envs)
                        .unwrap_or_else(|| direct_entry_env.clone());
                    errors.extend(self.check_body(body, &mut branch_env));
                    match block_terminal_effect_with_divergence(body, &|expr| {
                        self.expr_is_declared_never_call(expr)
                    }) {
                        TerminalEffect::FallsThrough => switch_exit_envs.push(branch_env),
                        TerminalEffect::Breaks | TerminalEffect::TerminatesMixed => {
                            switch_exit_envs.push(branch_env);
                        }
                        TerminalEffect::ExitsCurrentBlock => {}
                    }
                } else {
                    switch_exit_envs.push(direct_entry_env);
                    if let Some(previous) = fallthrough_env.take() {
                        switch_exit_envs.push(previous);
                    }
                }
                self.break_continue_depth -= 1;
                if let Some(joined) = join_fallthrough_type_envs(self, &switch_exit_envs) {
                    *env = joined;
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(CompileError::from_many(errors))
                }
            }
            StmtKind::If {
                condition,
                then_body,
                elseif_clauses,
                else_body,
            } => {
                let mut errors = Vec::new();

                // Each clause body receives its own environment. The shared `env` carries only the
                // accumulated false-condition path into the next elseif/else; mutating it while
                // checking a truthy sibling would leak assignments between mutually exclusive
                // branches. Reachable exits are joined after the whole chain.
                let mut clauses: Vec<(&Expr, &Vec<Stmt>)> = vec![(condition, then_body)];
                clauses.extend(elseif_clauses.iter().map(|(c, b)| (c, b)));
                let mut fallthrough_clause_envs = Vec::new();

                for (cond, body) in &clauses {
                    self.infer_type_with_assignment_effects(cond, env)?;
                    let condition_env = env.clone();
                    let mut branch_env = condition_env.clone();
                    self.install_truthy_short_circuit_effects(cond, &mut branch_env)?;

                    if let Some(guard) = self.guard_narrowing(cond, &branch_env)? {
                        branch_env.insert(guard.var.clone(), guard.then_ty);
                        *env = condition_env;
                        env.insert(guard.var, guard.else_ty);
                    } else {
                        *env = condition_env;
                    }

                    for s in *body {
                        if let Err(error) = self.check_stmt(s, &mut branch_env) {
                            errors.extend(error.flatten());
                        }
                    }
                    if !self.body_cannot_fall_through(body) {
                        fallthrough_clause_envs.push(branch_env);
                    }
                    self.install_falsy_short_circuit_effects(cond, env)?;
                }

                // The final else receives the accumulated complements without mutating the
                // already-recorded truthy exits. With no explicit else, that false path itself is
                // a reachable exit.
                if let Some(body) = else_body {
                    let mut branch_env = env.clone();
                    for s in body {
                        if let Err(error) = self.check_stmt(s, &mut branch_env) {
                            errors.extend(error.flatten());
                        }
                    }
                    if !self.body_cannot_fall_through(body) {
                        fallthrough_clause_envs.push(branch_env);
                    }
                } else {
                    fallthrough_clause_envs.push(env.clone());
                }
                if let Some(joined) = join_fallthrough_type_envs(self, &fallthrough_clause_envs) {
                    *env = joined;
                }

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(CompileError::from_many(errors))
                }
            }
            StmtKind::DoWhile { body, condition } => {
                stabilize_loop_storage(self, stmt.span, body, None, env);
                let errors = self.check_break_continue_target_body(body, env);
                self.infer_type_with_assignment_effects(condition, env)?;
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(CompileError::from_many(errors))
                }
            }
            StmtKind::While { condition, body } => {
                stabilize_loop_storage(self, stmt.span, body, None, env);
                self.infer_type_with_assignment_effects(condition, env)?;
                let truthy_short_circuit_bindings =
                    self.install_truthy_short_circuit_effects(condition, env)?;
                // The condition is re-evaluated before every iteration, so a guard on it
                // holds on entry to each one: `while (($row = fgetcsv($h)) !== false)`
                // leaves `$row` an array inside the body. The narrowing is dropped again
                // afterwards, because the loop exits precisely when the guard is false.
                // Installed INSIDE the short-circuit effects and undone before them, so the
                // two restores nest instead of overwriting one another.
                let guard = self.guard_narrowing(condition, env)?;
                let saved = guard
                    .as_ref()
                    .map(|g| (g.var.clone(), env.get(&g.var).cloned()));
                if let Some(g) = &guard {
                    env.insert(g.var.clone(), g.then_ty.clone());
                }
                let errors = self.check_break_continue_target_body(body, env);
                if let Some((var, previous)) = saved {
                    match previous {
                        Some(ty) => {
                            env.insert(var, ty);
                        }
                        None => {
                            env.remove(&var);
                        }
                    }
                }
                for (name, previous) in &truthy_short_circuit_bindings {
                    restore_narrowed_var(env, name, previous);
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(CompileError::from_many(errors))
                }
            }
            StmtKind::For {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(s) = init {
                    self.check_stmt(s, env)?;
                }
                stabilize_loop_storage(self, stmt.span, body, update.as_deref(), env);
                if let Some(c) = condition {
                    self.infer_type_with_assignment_effects(c, env)?;
                }
                if let Some(s) = update {
                    self.check_stmt(s, env)?;
                }
                let errors = self.check_break_continue_target_body(body, env);
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(CompileError::from_many(errors))
                }
            }
            StmtKind::Throw(expr) => {
                let thrown_ty = self.infer_type_with_assignment_effects(expr, env)?;
                match thrown_ty {
                    PhpType::Object(type_name)
                        if self.object_type_implements_throwable(&type_name)
                            || self.unresolved_new_object_defers_to_runtime(expr, &type_name) =>
                    {
                        Ok(())
                    }
                    PhpType::Object(_) => Err(CompileError::new(
                        stmt.span,
                        "Type error: throw requires an object implementing Throwable",
                    )),
                    ref ty
                        if crate::types::checker::type_compat::type_is_gradual_object_family(ty) =>
                    {
                        Ok(())
                    }
                    _ => Err(CompileError::new(
                        stmt.span,
                        "Type error: throw requires an object value",
                    )),
                }
            }
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                let mut errors = Vec::new();
                for s in try_body {
                    if let Err(error) = self.check_stmt(s, env) {
                        errors.extend(error.flatten());
                    }
                }
                for catch_clause in catches {
                    let mut resolved_types = Vec::new();
                    for raw_exception_type in &catch_clause.exception_types {
                        let exception_type =
                            self.resolve_catch_type_name(raw_exception_type, stmt.span)?;
                        let catch_type_is_known = self.classes.contains_key(&exception_type)
                            || self.interfaces.contains_key(&exception_type);
                        if catch_type_is_known
                            && !self.object_type_implements_throwable(&exception_type)
                        {
                            return Err(CompileError::new(
                                stmt.span,
                                &format!(
                                    "Catch type must extend or implement Throwable: {}",
                                    exception_type
                                ),
                            ));
                        }
                        if !catch_type_is_known {
                            self.unresolved_catch_types.insert(exception_type.clone());
                        }
                        // PHP permits unresolved catch names: if the class never loads, that arm
                        // simply cannot match. Keep the declared nominal type so unreachable-arm
                        // analysis does not erase callback and collection element precision.
                        resolved_types.push(exception_type);
                    }
                    if let Some(variable) = &catch_clause.variable {
                        env.insert(
                            variable.clone(),
                            PhpType::Object(self.common_catch_type_name(&resolved_types)),
                        );
                    }
                    for s in &catch_clause.body {
                        if let Err(error) = self.check_stmt(s, env) {
                            errors.extend(error.flatten());
                        }
                    }
                }
                if let Some(body) = finally_body {
                    self.finally_break_continue_bases
                        .push(self.break_continue_depth);
                    errors.extend(self.check_body(body, env));
                    self.finally_break_continue_bases.pop();
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(CompileError::from_many(errors))
                }
            }
            _ => unreachable!("non-control-flow statement routed to control-flow checker"),
        }
    }

    /// Returns the static key and value types exposed by foreach over an object iterator.
    ///
    /// Concrete `Iterator` implementations can narrow `key()`/`current()` from the
    /// interface's `mixed` contract, so foreach should expose those narrower types inside
    /// the loop. IteratorAggregate sources are resolved through their `getIterator()`
    /// return type when that type is statically known.
    fn foreach_object_key_value_types(
        &self,
        class_name: &str,
        source: &Expr,
    ) -> (PhpType, PhpType) {
        let value_override = self.foreach_object_value_type_override(class_name, source);
        if self.class_implements_interface(class_name, "Iterator")
            || self.interface_extends_interface(class_name, "Iterator")
        {
            return (
                self.iterator_method_return_type(class_name, "key"),
                value_override.unwrap_or_else(|| {
                    self.iterator_method_return_type(class_name, "current")
                }),
            );
        }

        let get_iterator_ty = self.iterator_method_return_type(class_name, "getIterator");
        if let PhpType::Object(iterator_name) = get_iterator_ty {
            return (
                self.iterator_method_return_type(&iterator_name, "key"),
                value_override.unwrap_or_else(|| {
                    self.iterator_method_return_type(&iterator_name, "current")
                }),
            );
        }

        (PhpType::Mixed, PhpType::Mixed)
    }

    /// Returns a narrower foreach value type for SPL filesystem iterators when flags are static.
    fn foreach_object_value_type_override(
        &self,
        class_name: &str,
        source: &Expr,
    ) -> Option<PhpType> {
        if class_name == "DirectoryIterator" {
            return Some(PhpType::Object("DirectoryIterator".to_string()));
        }
        let flags = self.filesystem_iterator_source_flags(class_name, source)?;
        match flags & FS_CURRENT_MODE_MASK {
            FS_CURRENT_AS_PATHNAME => None,
            FS_CURRENT_AS_SELF => Some(PhpType::Object(class_name.to_string())),
            _ => Some(PhpType::Object("SplFileInfo".to_string())),
        }
    }

    /// Returns constructor flags for statically constructed filesystem iterators.
    fn filesystem_iterator_source_flags(&self, class_name: &str, source: &Expr) -> Option<i64> {
        if !matches!(
            class_name,
            "FilesystemIterator" | "GlobIterator" | "RecursiveDirectoryIterator"
        ) {
            return None;
        }
        let ExprKind::NewObject {
            class_name: source_class,
            args,
        } = &source.kind
        else {
            return None;
        };
        if source_class.as_str() != class_name {
            return None;
        }
        args.get(1)
            .and_then(|expr| self.eval_static_int_expr(expr))
            .or_else(|| filesystem_iterator_default_flags(class_name))
    }

    /// Evaluates a side-effect-free integer expression used for SPL flag constants.
    fn eval_static_int_expr(&self, expr: &Expr) -> Option<i64> {
        match &expr.kind {
            ExprKind::IntLiteral(value) => Some(*value),
            ExprKind::Negate(inner) => self.eval_static_int_expr(inner).map(|value| -value),
            ExprKind::BitNot(inner) => self.eval_static_int_expr(inner).map(|value| !value),
            ExprKind::BinaryOp { left, op, right } => {
                let left = self.eval_static_int_expr(left)?;
                let right = self.eval_static_int_expr(right)?;
                match op {
                    BinOp::BitOr => Some(left | right),
                    BinOp::BitAnd => Some(left & right),
                    BinOp::BitXor => Some(left ^ right),
                    BinOp::Add => Some(left + right),
                    BinOp::Sub => Some(left - right),
                    _ => None,
                }
            }
            ExprKind::ScopedConstantAccess { receiver, name } => {
                self.class_constant_int_value(receiver, name)
            }
            _ => None,
        }
    }

    /// Resolves a class constant integer value from checker metadata.
    fn class_constant_int_value(&self, receiver: &StaticReceiver, name: &str) -> Option<i64> {
        let StaticReceiver::Named(class_name) = receiver else {
            return None;
        };
        self.classes
            .get(class_name.as_str())
            .and_then(|class_info| class_info.constants.get(name))
            .and_then(|expr| self.eval_static_int_expr(expr))
    }

    /// Looks up an iterator-related method return type on either a class or an interface.
    ///
    /// Missing metadata falls back to `mixed`, matching PHP's loose iterator contracts and
    /// preserving the previous conservative behavior for dynamic or unknown iterator shapes.
    fn iterator_method_return_type(&self, type_name: &str, method: &str) -> PhpType {
        let method_key = crate::names::php_symbol_key(method);
        if type_name == "DirectoryIterator" && method_key == "current" {
            return PhpType::Object("DirectoryIterator".to_string());
        }
        if let Some(class_info) = self.classes.get(type_name) {
            return class_info
                .methods
                .get(&method_key)
                .map(|sig| sig.return_type.clone())
                .unwrap_or(PhpType::Mixed);
        }
        self.interfaces
            .get(type_name)
            .and_then(|interface_info| interface_info.methods.get(&method_key))
            .map(|sig| sig.return_type.clone())
            .unwrap_or(PhpType::Mixed)
    }

    /// Checks a loop body with `break`/`continue` target tracking.
    ///
    /// Increments `break_continue_depth` before checking the body and decrements it after,
    /// so that `break`/`continue` validation knows the correct nesting level. Returns all
    /// errors accumulated while checking the body; the caller decides whether to propagate them.
    fn check_break_continue_target_body(
        &mut self,
        body: &[Stmt],
        env: &mut TypeEnv,
    ) -> Vec<CompileError> {
        // A loop body may observe a property write made by an earlier iteration (the write site
        // is *after* the read site in source order), so property narrowings from outside the
        // loop cannot be trusted inside it.
        Self::purge_property_narrowings(env);
        self.break_continue_depth += 1;
        let errors = self.check_body(body, env);
        self.break_continue_depth -= 1;
        errors
    }

    /// Updates callable metadata for a foreach value variable.
    ///
    /// Homogeneous arrays that store callable descriptors keep their signature
    /// and capture metadata under the source array variable name. A foreach value
    /// binding from that array must expose the same metadata to calls emitted in
    /// the loop body.
    fn update_foreach_callable_metadata(
        &mut self,
        dest: &str,
        source_array: &Expr,
        value_ty: &PhpType,
    ) {
        if value_ty != &PhpType::Callable {
            self.clear_foreach_callable_metadata(dest);
            return;
        }
        if let ExprKind::Variable(src_name) = &source_array.kind {
            self.copy_foreach_callable_metadata(dest, src_name);
        } else {
            self.clear_foreach_callable_metadata(dest);
        }
    }

    /// Copies callable signature, capture, first-class target, and callable-array metadata.
    fn copy_foreach_callable_metadata(&mut self, dest: &str, src: &str) {
        if let Some(return_ty) = self.closure_return_types.get(src).cloned() {
            self.closure_return_types.insert(dest.to_string(), return_ty);
        } else {
            self.closure_return_types.remove(dest);
        }
        if let Some(sig) = self.callable_sigs.get(src).cloned() {
            self.callable_sigs.insert(dest.to_string(), sig);
        } else {
            self.callable_sigs.remove(dest);
        }
        if let Some(captures) = self.callable_captures.get(src).cloned() {
            self.callable_captures.insert(dest.to_string(), captures);
        } else {
            self.callable_captures.remove(dest);
        }
        if let Some(target) = self.callable_array_targets.get(src).cloned() {
            self.callable_array_targets
                .insert(dest.to_string(), target);
        } else {
            self.callable_array_targets.remove(dest);
        }
        if let Some(target) = self.first_class_callable_targets.get(src).cloned() {
            self.first_class_callable_targets
                .insert(dest.to_string(), target);
        } else {
            self.first_class_callable_targets.remove(dest);
        }
    }

    /// Clears callable metadata for a foreach key or value binding.
    fn clear_foreach_callable_metadata(&mut self, dest: &str) {
        self.closure_return_types.remove(dest);
        self.callable_sigs.remove(dest);
        self.callable_captures.remove(dest);
        self.callable_array_targets.remove(dest);
        self.first_class_callable_targets.remove(dest);
    }

    /// Checks each statement in a body sequentially, collecting all errors.
    ///
    /// Unlike `check_break_continue_target_body`, this does not update `break_continue_depth`.
    /// Used for `switch` cases, `if` branches, `try` blocks, and other bodies where the
    /// break/continue level is managed at a higher level.
    fn check_body(&mut self, body: &[Stmt], env: &mut TypeEnv) -> Vec<CompileError> {
        let mut errors = Vec::new();
        for stmt in body {
            if let Err(error) = self.check_stmt(stmt, env) {
                errors.extend(error.flatten());
            }
        }
        errors
    }
}
