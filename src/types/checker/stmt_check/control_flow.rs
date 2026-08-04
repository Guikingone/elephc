//! Purpose:
//! Validates statement control flow behavior.
//! Keeps control-flow and assignment effects synchronized with expression inference and return analysis.
//!
//! Called from:
//! - `crate::types::checker::stmt_check`
//!
//! Key details:
//! - Branch and loop handling must preserve PHP execution order and conservative type environments.
//! - `if`/elseif/else chains and `switch (true)` case bodies apply flow-sensitive type-guard
//!   narrowing via `crate::types::checker::stmt_check::narrowing`; case-body narrowing is gated on
//!   fall-in safety so a case reachable by fall-through from a non-terminating case is not narrowed.

use crate::errors::CompileError;
use crate::parser::ast::{BinOp, Expr, ExprKind, StaticReceiver, Stmt, StmtKind};
use crate::types::{PhpType, TypeEnv};
use std::collections::BTreeMap;

use super::super::type_compat::type_is_gradual_object_family;
use super::super::Checker;

const FS_CURRENT_AS_SELF: i64 = 16;
const FS_CURRENT_AS_PATHNAME: i64 = 32;
const FS_CURRENT_MODE_MASK: i64 = 240;
const FS_SKIP_DOTS: i64 = 4096;

/// Computes and records fixed-point array storage contracts before checking a loop body.
///
/// The shared analysis iterates over rebinds and growth sites with an evolving environment, so
/// cascading promotions, non-literal RHSs, and raw-to-raw element changes converge before any
/// header/body read is checked. This includes entry-empty roots autovivified by nested writes: a
/// read earlier in the body can observe the container created by a prior iteration, so the
/// single-pass checker must use the back-edge's boxed element representation from the start. EIR
/// lowering later consumes the recorded contract for the same loop span rather than repeating
/// expression inference.
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
            if is_call {
                if let Some(cached) = call_types.get(&expr.span) {
                    return Some(cached.clone());
                }
            }
            let inferred = checker.infer_type(expr, analysis_env).ok()?;
            if is_call {
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

/// Collects variable names bound by a simple `$var = <expr>` assignment appearing directly in an
/// `if`/`elseif` condition (`if (!$x = f())`, `false === $y = g()`, `$a && $b = h()`). Such a
/// variable takes the condition-assigned type on the frame slot, which then persists past the whole
/// construct as a member that no reaching branch actually leaves live once a later branch reassigns
/// it — the post-`if` join must recompute its type from the branches instead. Only the assignment
/// target is collected; assignment right-hand sides are not descended into.
fn collect_condition_assigned_vars(cond: &Expr, out: &mut Vec<String>) {
    match &cond.kind {
        ExprKind::Not(inner) | ExprKind::Negate(inner) => {
            collect_condition_assigned_vars(inner, out);
        }
        ExprKind::BinaryOp { left, right, .. } => {
            collect_condition_assigned_vars(left, out);
            collect_condition_assigned_vars(right, out);
        }
        ExprKind::Assignment { target, .. } => {
            if let ExprKind::Variable(name) = &target.kind {
                if !out.iter().any(|existing| existing == name) {
                    out.push(name.clone());
                }
            }
        }
        _ => {}
    }
}

/// Records a variable's pre-`if` type the first time the chain touches it, so the post-`if`
/// restore (and the branch-union join) has the un-narrowed type to fall back to. Later touches are
/// ignored: by then `env` already carries an accumulated complement from an earlier clause.
fn remember_pre_if_type(
    saved_vars: &mut Vec<(String, Option<PhpType>)>,
    var: &str,
    env: &TypeEnv,
) {
    if !saved_vars.iter().any(|(existing, _)| existing == var) {
        saved_vars.push((var.to_string(), env.get(var).cloned()));
    }
}

/// Returns true when `ty` carries an object member: an `Object`, or a union with an object member.
/// The post-`if` branch-union join is scoped to object-bearing variables because the dead-member
/// staleness it fixes is an object member-access concern; a scalar/`Mixed` local (e.g. an
/// `int` exit code narrowed by `is_numeric` and reassigned to `int` in both arms) keeps its
/// conservative type, so the join never tightens a scalar or gradual slot into a strict union.
fn type_is_object_bearing(ty: &PhpType) -> bool {
    match ty {
        PhpType::Object(_) => true,
        PhpType::Union(members) => members.iter().any(type_is_object_bearing),
        _ => false,
    }
}

/// Collects the names of variables directly reassigned by a top-level `$var = <expr>` statement in
/// a branch body. Used to scope the post-`if` branch-union join to variables a branch actually
/// rebinds: a variable only *narrowed* across the branches (each clause tests it but none reassigns
/// it) rejoins to its pre-`if` type, and reconstructing it as the union of its per-branch narrowings
/// would needlessly re-partition e.g. `\Throwable` into `FatalError|Error|ErrorException|Throwable`.
fn collect_body_assigned_vars(body: &[Stmt], out: &mut Vec<String>) {
    for stmt in body {
        if let StmtKind::Assign { name, .. } = &stmt.kind {
            if !out.iter().any(|existing| existing == name) {
                out.push(name.clone());
            }
        }
    }
}

/// Snapshots a branch's exit environment for the post-`if` join. Clones the current flow types and
/// re-applies the precise per-path types a conditional body widens away in `env`: each surviving
/// direct overwrite (e.g. a `new Concrete()` store reverted to its conservative frame-slot type on
/// exit) and a trailing `$name = <expr>` assignment (whose value the widened slot would otherwise
/// mask). A later union over the reaching branches then sees the precise per-path type.
/// Captures a branch's ENTRY environment when that branch cannot fall through.
///
/// `check_conditional_path_body` type-checks a clause body against the shared `env` and writes each
/// assignment straight into it — the recorded `overwrites` are what the JOIN consumes, not what the
/// body did to `env`. For a branch that ends in `return`/`throw`/`continue`/`break`, nothing it
/// wrote can be observed after the `if`, so leaving those writes in `env` types the following code
/// off a path that never reaches it:
///
/// ```php
/// private static function createRequestFromFactory(array $query = [], array $request = [], …) {
///     if (self::$requestFactory) {
///         $request = (self::$requestFactory)(…);   // $request becomes a Request OBJECT
///         if (!$request instanceof self) { throw new \LogicException(…); }
///         return $request;                          // … and this path RETURNS
///     }
///     return new static($query, $request, …);       // yet $request read as Object, not array
/// }
/// ```
///
/// That is `Symfony\Component\HttpFoundation\Request::createRequestFromFactory`, reported as
/// `parameter $request expects Array(Never), got Object("…\Request")` on a program PHP runs.
/// Returning `Some(env.clone())` here, and restoring it once the body has been checked, keeps the
/// branch's diagnostics while discarding its unreachable type facts. `None` — the fall-through
/// case — costs nothing.
fn diverging_branch_entry_env(
    checker: &Checker,
    body: &[Stmt],
    env: &TypeEnv,
) -> Option<TypeEnv> {
    checker.body_cannot_fall_through(body).then(|| env.clone())
}

fn snapshot_branch_exit(
    env: &TypeEnv,
    overwrites: &BTreeMap<String, PhpType>,
    final_assignment: &Option<(String, PhpType)>,
) -> TypeEnv {
    let mut snapshot = env.clone();
    for (name, ty) in overwrites {
        snapshot.insert(name.clone(), ty.clone());
    }
    if let Some((name, ty)) = final_assignment {
        snapshot.insert(name.clone(), ty.clone());
    }
    snapshot
}

/// Returns the synthetic constructor default flags for filesystem iterators.
fn filesystem_iterator_default_flags(class_name: &str) -> Option<i64> {
    match class_name {
        "FilesystemIterator" => Some(FS_SKIP_DOTS),
        "GlobIterator" | "RecursiveDirectoryIterator" => Some(0),
        _ => None,
    }
}

impl Checker {
    /// Promotes an untyped parameter of the method currently being checked to `Mixed` when
    /// `foreach` proves that the legacy `Int` fallback is not a usable static representation.
    ///
    /// Method schemas initially use `Int` for untyped parameters until a call site specializes
    /// them. Uncalled framework methods have no such call site, but their bodies are still
    /// validated and emitted. Recording `Mixed` on the authoritative method signature keeps the
    /// checker environment, EIR parameter ABI, and runtime `foreach` guard aligned.
    fn promote_untyped_foreach_method_param(
        &mut self,
        iterable: &Expr,
        iterable_ty: &PhpType,
        env: &mut TypeEnv,
    ) -> bool {
        if !matches!(iterable_ty, PhpType::Int) {
            return false;
        }
        let ExprKind::Variable(param_name) = &iterable.kind else {
            return false;
        };
        let (Some(class_name), Some(method_name)) =
            (self.current_class.clone(), self.current_method.clone())
        else {
            return false;
        };
        let Some(class_info) = self.classes.get_mut(&class_name) else {
            return false;
        };
        let sig = if self.current_method_is_static {
            class_info.static_methods.get_mut(&method_name)
        } else {
            class_info.methods.get_mut(&method_name)
        };
        let Some(sig) = sig else {
            return false;
        };
        let Some(param_index) = sig.params.iter().position(|(name, _)| name == param_name) else {
            return false;
        };
        if sig
            .declared_params
            .get(param_index)
            .copied()
            .unwrap_or(false)
        {
            return false;
        }
        sig.params[param_index].1 = PhpType::Mixed;
        env.insert(param_name.clone(), PhpType::Mixed);
        true
    }

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
                let mut arr_ty = self.infer_type_with_assignment_effects(array, env)?;
                if self.promote_untyped_foreach_method_param(array, &arr_ty, env) {
                    arr_ty = PhpType::Mixed;
                }
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
                    let normalized_class_name = class_name.trim_start_matches('\\');
                    let is_iter = normalized_class_name == "Iterator"
                        || self.class_implements_interface(class_name, "Iterator")
                        || self.interface_extends_interface(class_name, "Iterator");
                    let is_iter_agg = normalized_class_name == "IteratorAggregate"
                        || self.class_implements_interface(class_name, "IteratorAggregate")
                        || self.interface_extends_interface(class_name, "IteratorAggregate");
                    let is_traversable = normalized_class_name == "Traversable"
                        || self.interface_extends_interface(class_name, "Traversable");
                    if is_iter || is_iter_agg || is_traversable {
                        let (key_ty, value_ty) =
                            self.foreach_object_key_value_types(class_name, array);
                        if let Some(k) = key_var {
                            env.insert(k.clone(), key_ty);
                            self.clear_foreach_callable_metadata(k);
                        }
                        env.insert(value_var.clone(), value_ty);
                        self.clear_foreach_callable_metadata(value_var);
                    } else {
                        // A plain object implementing none of Iterator/IteratorAggregate/
                        // Traversable iterates its accessible declared properties in PHP:
                        // property-name string keys with values in declaration order. The
                        // EIR backend snapshots the object's public properties into a hash
                        // and drives the hash-iteration path (see `lower_iter_start`), so
                        // bind the key to `Str` and the value to `Mixed` accordingly.
                        if let Some(k) = key_var {
                            env.insert(k.clone(), PhpType::Str);
                            self.clear_foreach_callable_metadata(k);
                        }
                        env.insert(value_var.clone(), PhpType::Mixed);
                        self.clear_foreach_callable_metadata(value_var);
                    }
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
                } else {
                    return Err(CompileError::new(
                        stmt.span,
                        &format!(
                            "foreach requires an array, iterable, or an object implementing Iterator/IteratorAggregate, got {:?}",
                            arr_ty
                        ),
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
                self.break_continue_depth += 1;
                // `switch (true)` runs a case body when the case guard is truthy, so the body should
                // see that guard's narrowing (mirroring the if-then narrowing above). Only narrow a
                // case body that cannot be reached by falling through from a previous, non-terminating
                // case: otherwise the case's own guard may be false at runtime when control fell in.
                // `fall_in_safe` is true for the first case and for any case whose predecessor
                // terminates (break/continue/return/throw/diverge). The narrowing is inserted into a
                // saved-and-restored case environment, so it never leaks past the case body.
                //
                // Every case is also a direct dispatch target. A terminated predecessor therefore
                // cannot feed its assignments into the next case; start that sibling from the
                // switch-entry environment. Non-terminated predecessors keep their environment to
                // model real fallthrough. Exit environments are joined separately for statements
                // after the switch, so isolating sibling entry does not lose conservative storage
                // widening across the construct.
                let subject_is_true = matches!(&subject.kind, ExprKind::BoolLiteral(true));
                let switch_entry_env = env.clone();
                let mut switch_exit_env = switch_entry_env.clone();
                let mut previous_case_env: Option<TypeEnv> = None;
                let mut prev_terminates = true;
                for (values, body) in cases {
                    let fall_in_safe = prev_terminates;
                    let mut case_env = if prev_terminates {
                        switch_entry_env.clone()
                    } else {
                        previous_case_env
                            .take()
                            .unwrap_or_else(|| switch_entry_env.clone())
                    };
                    if subject_is_true && fall_in_safe && values.len() == 1 {
                        let narrowings =
                            self.and_chain_then_narrowings(&values[0], &case_env);
                        let saved: Vec<(String, Option<PhpType>)> = narrowings
                            .iter()
                            .map(|(var, _)| (var.clone(), case_env.get(var).cloned()))
                            .collect();
                        for (var, then_ty) in &narrowings {
                            case_env.insert(var.clone(), then_ty.clone());
                        }
                        errors.extend(self.check_body(body, &mut case_env));
                        for (var, original) in &saved {
                            restore_narrowed_var(&mut case_env, var, original);
                        }
                    } else {
                        errors.extend(self.check_body(body, &mut case_env));
                    }
                    self.merge_switch_path_env(&mut switch_exit_env, &case_env);
                    prev_terminates = self.case_body_terminates(body);
                    previous_case_env = Some(case_env);
                }
                if let Some(body) = default {
                    let mut default_env = if prev_terminates {
                        switch_entry_env.clone()
                    } else {
                        previous_case_env
                            .take()
                            .unwrap_or_else(|| switch_entry_env.clone())
                    };
                    errors.extend(self.check_body(body, &mut default_env));
                    self.merge_switch_path_env(&mut switch_exit_env, &default_env);
                }
                *env = switch_exit_env;
                self.break_continue_depth -= 1;
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

                // Flow-sensitive type narrowing across the if / elseif* / else chain.
                //
                // Each recognized guard narrows its variable to the guarded type while checking
                // that branch's body. The fallthrough env for the remaining clauses (and the final
                // else) accumulates the complement, which is sound because reaching a later clause
                // means every earlier condition was false.
                //
                // After the whole construct we restore every variable we narrowed, so code after
                // the `if` sees the joined view. The single exception is an exhaustively divergent
                // chain (no else and *every* clause body diverges): there the only way to fall
                // through is with all conditions false, so the accumulated complement is sound for
                // the statements after the `if`.
                let mut clauses: Vec<(&Expr, &Vec<Stmt>)> = vec![(condition, then_body)];
                clauses.extend(elseif_clauses.iter().map(|(c, b)| (c, b)));

                // Pre-`if` type of every variable we narrow, captured the first time we touch it,
                // so each one can be restored after the construct.
                let mut saved_vars: Vec<(String, Option<PhpType>)> = Vec::new();
                let mut applied_any_guard = false;
                // Set when the clause loop already persisted a top-level `||` condition's De Morgan
                // complement on the fall-through edge, so the single-clause block after the join
                // does not apply the same narrowing a second time against an already-narrowed
                // environment.
                let mut or_complement_applied = false;
                let track_single_guard_convergence =
                    clauses.len() == 1 && else_body.is_none();
                let mut single_guard_then_exit: Option<(String, PhpType, bool)> = None;
                let mut branch_overwrites = Vec::new();
                // Exit environment of every branch that can fall through past the `if`. The post-`if`
                // type of a joined variable is the union of its type across them, which is precise
                // only when every path reaching the following code is one of the recorded snapshots.
                //
                // With an explicit `else` those are exactly the branch bodies. Without one, the
                // implicit false edge also reaches, so it is recorded too — as the accumulated
                // complement env, which is what a value satisfying no condition actually carries.
                // A single guard with no `else` is excluded: `single_guard_then_exit` below already
                // computes that same two-path union and additionally refines it when the false edge
                // is impossible, so re-deriving it here would only widen it back.
                let record_branch_exits = else_body.is_some() || clauses.len() > 1;
                let mut branch_exit_snapshots: Vec<TypeEnv> = Vec::new();
                // Variables bound inside a clause condition (`if (!$x = f())`). Their condition type
                // otherwise persists past the construct even when a later branch reassigns them, so
                // they take the branch-union at the join, alongside the guard-narrowed `saved_vars`.
                let mut condition_assigned_vars: Vec<String> = Vec::new();
                let mut precise_object_overwrites = if let Some(else_branch) = else_body {
                    let mut branch_bodies: Vec<&[Stmt]> =
                        clauses.iter().map(|(_, body)| body.as_slice()).collect();
                    branch_bodies.push(else_branch.as_slice());
                    Self::convergent_direct_object_overwrites(&branch_bodies)
                } else {
                    BTreeMap::new()
                };
                precise_object_overwrites.retain(|name, _| {
                    matches!(
                        env.get(name),
                        Some(
                            PhpType::Str
                                | PhpType::Int
                                | PhpType::Float
                                | PhpType::Bool
                                | PhpType::False
                                | PhpType::Void
                        )
                    )
                });

                for (cond, body) in &clauses {
                    self.infer_type_with_assignment_effects(cond, env)?;
                    if else_body.is_some() {
                        collect_condition_assigned_vars(cond, &mut condition_assigned_vars);
                    }

                    if let Some(guard) = self.guard_narrowing(cond, env)? {
                        applied_any_guard = true;
                        // Remember the variable's pre-`if` type the first time we narrow it.
                        remember_pre_if_type(&mut saved_vars, &guard.var, env);

                        // Check the guarded body with the "then" type.
                        let saved = env.get(&guard.var).cloned();
                        env.insert(guard.var.clone(), guard.then_ty.clone());
                        // Assignment-receiver aliases (`$f = $src`) hold the same value on branch
                        // entry, so they take the same narrowing in the guarded body. This is a
                        // branch-entry fact; a later reassignment inside the body overwrites it.
                        for alias in &guard.aliases {
                            env.insert(alias.clone(), guard.then_ty.clone());
                        }
                        let diverging_entry_env = diverging_branch_entry_env(self, body, env);
                        let (body_errors, overwrites, final_assignment) = self
                            .check_conditional_path_body(
                            body,
                            &precise_object_overwrites,
                            env,
                        );
                        errors.extend(body_errors);
                        if let Some(entry_env) = diverging_entry_env {
                            *env = entry_env;
                        }
                        if !self.body_cannot_fall_through(body) {
                            if track_single_guard_convergence {
                                if let Some(exit_ty) = final_assignment
                                    .as_ref()
                                    .filter(|(name, _)| name == &guard.var)
                                    .map(|(_, ty)| ty.clone())
                                    .or_else(|| overwrites.get(&guard.var).cloned())
                                    .or_else(|| env.get(&guard.var).cloned())
                                {
                                    let false_edge_impossible =
                                        saved.as_ref() == Some(&guard.then_ty);
                                    single_guard_then_exit = Some((
                                        guard.var.clone(),
                                        exit_ty,
                                        false_edge_impossible,
                                    ));
                                }
                            }
                            if record_branch_exits {
                                branch_exit_snapshots.push(snapshot_branch_exit(
                                    env,
                                    &overwrites,
                                    &final_assignment,
                                ));
                            }
                            branch_overwrites.push(overwrites);
                        }
                        restore_narrowed_var(env, &guard.var, &saved);

                        // The fallthrough env for the rest of the chain (next elseif or else)
                        // sees the complement.
                        env.insert(guard.var.clone(), guard.else_ty.clone());
                        // Assignment-receiver aliases (`$f = $src`) share the value, so the
                        // complement applies to them on the fallthrough edge too.
                        for alias in &guard.aliases {
                            env.insert(alias.clone(), guard.else_ty.clone());
                        }
                    } else {
                        // A pure `&&` condition can prove several independent facts in its
                        // true branch. Its false-side complement is a disjunction, which this
                        // environment can represent only when every conjunct constrains the same
                        // binding (`and_chain_else_narrowings`); a top-level `||` condition
                        // contributes its De Morgan complement to the same fall-through edge.
                        // Anything else leaves the fall-through environment unchanged.
                        let narrowings = self.and_chain_then_narrowings(cond, env);
                        let saved: Vec<(String, Option<PhpType>)> = narrowings
                            .iter()
                            .map(|(var, _)| (var.clone(), env.get(var).cloned()))
                            .collect();
                        for (var, then_ty) in &narrowings {
                            remember_pre_if_type(&mut saved_vars, var, env);
                            env.insert(var.clone(), then_ty.clone());
                        }
                        let diverging_entry_env = diverging_branch_entry_env(self, body, env);
                        let (body_errors, overwrites, final_assignment) =
                            self.check_conditional_path_body(
                                body,
                                &precise_object_overwrites,
                                env,
                            );
                        errors.extend(body_errors);
                        if let Some(entry_env) = diverging_entry_env {
                            *env = entry_env;
                        }
                        if !self.body_cannot_fall_through(body) {
                            if record_branch_exits {
                                branch_exit_snapshots.push(snapshot_branch_exit(
                                    env,
                                    &overwrites,
                                    &final_assignment,
                                ));
                            }
                            branch_overwrites.push(overwrites);
                        }
                        for (var, original) in &saved {
                            restore_narrowed_var(env, var, original);
                        }
                        // Falling past this clause proves its condition false. A `&&` chain whose
                        // conjuncts all constrain one binding, and a top-level `||` chain, both
                        // have a false edge this environment can represent, so persist it for the
                        // remaining clauses, the `else` body, and — when no clause falls through —
                        // the statements after the `if`.
                        let mut complements = self.and_chain_else_narrowings(cond, env);
                        let or_complements = self.or_chain_complement_narrowings(cond, env);
                        or_complement_applied |= !or_complements.is_empty();
                        complements.extend(or_complements);
                        if !complements.is_empty() {
                            applied_any_guard = true;
                        }
                        for (var, else_ty) in complements {
                            remember_pre_if_type(&mut saved_vars, &var, env);
                            env.insert(var, else_ty);
                        }
                    }
                }

                // Final else body (if present) is checked with the accumulated complement.
                if let Some(body) = else_body {
                    let diverging_entry_env = diverging_branch_entry_env(self, body, env);
                    let (body_errors, overwrites, final_assignment) =
                        self.check_conditional_path_body(
                            body,
                            &precise_object_overwrites,
                            env,
                        );
                    errors.extend(body_errors);
                    if let Some(entry_env) = diverging_entry_env {
                        *env = entry_env;
                    }
                    if !self.body_cannot_fall_through(body) {
                        branch_exit_snapshots.push(snapshot_branch_exit(
                            env,
                            &overwrites,
                            &final_assignment,
                        ));
                        branch_overwrites.push(overwrites);
                    }
                } else {
                    // The implicit false edge reaches the code after the `if` carrying the
                    // accumulated complement — the type a value satisfying no clause condition
                    // actually has. Recording it as a reaching path keeps the union below sound
                    // for a chain that has no `else`.
                    if record_branch_exits {
                        branch_exit_snapshots.push(env.clone());
                    }
                    // The implicit false edge performs no unconditional overwrite.
                    branch_overwrites.push(BTreeMap::new());
                }

                // Keep the accumulated complement for the statements after the `if` only when no
                // guarded clause can fall through: there is no else and every clause ends in a
                // non-fallthrough statement, so reaching the following code implies all conditions
                // were false. Otherwise restore every narrowed variable to its pre-`if` type.
                let keep_complement_after_if = applied_any_guard
                    && else_body.is_none()
                    && clauses
                        .iter()
                        .all(|(_, body)| self.body_cannot_fall_through(body));
                if !keep_complement_after_if {
                    for (var, original) in &saved_vars {
                        let converged = single_guard_then_exit
                            .as_ref()
                            .and_then(|(then_var, then_ty, false_edge_impossible)| {
                                if then_var != var {
                                    return None;
                                }
                                if *false_edge_impossible {
                                    return Some(then_ty.clone());
                                }
                                let false_ty = env.get(var)?;
                                if false_ty == then_ty {
                                    return Some(then_ty.clone());
                                }
                                if Self::array_family_gradual_accepts(false_ty, then_ty) {
                                    if let Some(merged) =
                                        self.merged_assignment_type(false_ty, then_ty)
                                    {
                                        return Some(merged);
                                    }
                                }
                                // Single guard, no `else`: exactly two paths reach the code after
                                // the `if` — the guard-true body exit (`then_ty`) and the
                                // guard-false complement (`false_ty`). The precise post-`if` type
                                // is their union. Falling through to restore the pre-`if` type
                                // needlessly widened the variable back to members the complement
                                // had eliminated (`if ($x instanceof C) { $x = ...; }` over a
                                // `string|C|null` value kept `C` in the join, so a following use
                                // saw the object member the `instanceof` had just ruled out).
                                Some(self.normalize_union_type(vec![
                                    then_ty.clone(),
                                    false_ty.clone(),
                                ]))
                            });
                        if let Some(converged) = converged {
                            env.insert(var.clone(), converged);
                        } else {
                            restore_narrowed_var(env, var, original);
                        }
                    }

                    // Every path that reaches the code after the `if` is one of the recorded branch
                    // exits: a clause body that falls through, the `else` body, or — for a chain
                    // with no `else` — the implicit false edge carrying the accumulated complement
                    // (a clause that diverges via `return`/`continue`/`throw` records nothing).
                    // A variable that a branch *reassigns* therefore has, after the `if`, the union of
                    // its type across those reaching branches. This is the precise flow join, scoped
                    // to variables that are both (a) narrowed by a guard or bound in a clause
                    // condition and (b) reassigned by some branch: it recovers an `else`-branch (or
                    // `elseif`-condition) reassignment the path-insensitive frame-slot join would
                    // otherwise leave carrying a value only live on a path that reassigned or diverged
                    // away from it — e.g. Symfony's `ResolveAutowireInlineAttributesPass`, where
                    // `$attribute` is bound in an `elseif` condition, that clause `continue`s, and the
                    // `else` rebinds it via `newInstance()`, so `ReflectionAttribute` never reaches
                    // the following `$attribute->value`. A variable only narrowed across the branches
                    // (`if ($e instanceof Error) ... elseif ... else ...` over a `\Throwable`, no
                    // clause rebinding `$e`) is excluded: it rejoins to its pre-`if` type rather than
                    // a re-partitioned union, and gradual/`Mixed` slots are never tightened here.
                    if !branch_exit_snapshots.is_empty() {
                        let mut reassigned_in_branch: Vec<String> = Vec::new();
                        for (_, body) in &clauses {
                            collect_body_assigned_vars(body, &mut reassigned_in_branch);
                        }
                        if let Some(else_branch) = else_body {
                            collect_body_assigned_vars(else_branch, &mut reassigned_in_branch);
                        }
                        let mut joined: Vec<(String, PhpType)> = Vec::new();
                        let join_names = saved_vars
                            .iter()
                            .map(|(name, _)| name)
                            .chain(condition_assigned_vars.iter());
                        let mut seen: Vec<&String> = Vec::new();
                        for name in join_names {
                            if seen.iter().any(|existing| *existing == name) {
                                continue;
                            }
                            seen.push(name);
                            if !reassigned_in_branch.iter().any(|v| v == name) {
                                continue;
                            }
                            // Two reasons a name reaches the join, and they need different scoping.
                            //
                            // The original one is an OBJECT member going stale in the frame slot,
                            // which is why a scalar/gradual local used to be left untouched here.
                            //
                            // The second is the one that gate silently swallowed: the restore above
                            // puts every narrowed variable back to its PRE-`if` type, which is right
                            // for the narrowing and wrong for an ASSIGNMENT made under it. `$loop =
                            // null; if (null !== $loop) {} elseif (…) { $loop = []; }` came out of
                            // the construct as `null` — PHP's own `array|null` was thrown away, and
                            // only because the guard happened to test `$loop` itself (guarding any
                            // OTHER variable kept the assignment). Symfony's `PhpDumper` builds its
                            // circular-reference path in exactly that shape.
                            //
                            // So the join now also covers a narrowed variable whose branch-exit
                            // types are not all the restored type. That is strictly the union over
                            // the reaching branches, which is what the restored value was standing
                            // in for.
                            let restored = env.get(name);
                            let assignment_changed_it = branch_exit_snapshots
                                .iter()
                                .any(|snapshot| snapshot.get(name) != restored);
                            if !restored.is_some_and(type_is_object_bearing) && !assignment_changed_it
                            {
                                continue;
                            }
                            let mut members = Vec::with_capacity(branch_exit_snapshots.len());
                            let mut bound_on_every_branch = true;
                            for snapshot in &branch_exit_snapshots {
                                match snapshot.get(name) {
                                    Some(ty) => members.push(ty.clone()),
                                    // Not bound on this reaching branch: possibly-undefined after the
                                    // `if`, so leave its env entry untouched rather than narrow it.
                                    None => {
                                        bound_on_every_branch = false;
                                        break;
                                    }
                                }
                            }
                            if bound_on_every_branch && !members.is_empty() {
                                joined.push((name.clone(), self.normalize_union_type(members)));
                            }
                        }
                        for (name, ty) in joined {
                            self.assign_post_if_flow_type(env, name, ty);
                        }
                    }
                }

                // A local directly overwritten to the same type on every continuing branch has
                // that precise flow type after the `if`, even though its frame slot remains
                // conservatively widened while each conditional store is checked.
                if let Some(first) = branch_overwrites.first() {
                    let precise: Vec<(String, PhpType)> = first
                        .iter()
                        .filter(|(name, ty)| {
                            branch_overwrites
                                .iter()
                                .all(|path| path.get(*name) == Some(*ty))
                        })
                        .map(|(name, ty)| (name.clone(), ty.clone()))
                        .collect();
                    for (name, ty) in precise {
                        self.assign_post_if_flow_type(env, name, ty);
                    }
                }

                // De Morgan complement for a diverging single-clause `if (A || B) { <diverges> }`:
                // reaching the statements after the `if` proves the whole `||` condition was false,
                // hence every disjunct false. Persist each disjunct's guard-false narrowing so a
                // receiver proven by `!(cond || !$x instanceof T)` (i.e. `$x instanceof T`) is usable
                // afterward. Gated on a body that cannot fall through (the only way past the `if` is
                // with the condition false) and on there being no elseif/else clause — a
                // fall-through clause would leave another path to the following code.
                //
                // Skipped when the clause loop already persisted the same complement: with no
                // `elseif` the loop's only clause *is* this `if`, so re-deriving the disjuncts'
                // guard-false facts here would run them against an environment they have already
                // narrowed. That second pass is not guaranteed to be a no-op — a guard whose
                // false type is computed from the receiver's current type would be applied twice —
                // so the duplicate is removed rather than assumed idempotent.
                if else_body.is_none()
                    && elseif_clauses.is_empty()
                    && !or_complement_applied
                    && self.body_cannot_fall_through(then_body)
                {
                    for (var, then_ty) in self.or_chain_complement_narrowings(condition, env) {
                        env.insert(var, then_ty);
                    }
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
                // Entering the body proves every guard in a pure `&&` condition true. Keep these
                // facts scoped to the body: a while may execute zero times, so they cannot refine
                // the environment after the loop.
                let narrowings = self.and_chain_then_narrowings(condition, env);
                let saved: Vec<(String, Option<PhpType>)> = narrowings
                    .iter()
                    .map(|(var, _)| (var.clone(), env.get(var).cloned()))
                    .collect();
                for (var, then_ty) in &narrowings {
                    env.insert(var.clone(), then_ty.clone());
                }
                let errors = self.check_break_continue_target_body(body, env);
                for (var, original) in &saved {
                    restore_narrowed_var(env, var, original);
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
            } => self.check_for_stmt(
                stmt.span,
                init.as_deref(),
                condition.as_ref(),
                update.as_deref(),
                body,
                env,
            ),
            StmtKind::Throw(expr) => {
                let thrown_ty = self.infer_type_with_assignment_effects(expr, env)?;
                match thrown_ty {
                    PhpType::Object(type_name)
                        if self.object_type_implements_throwable(&type_name) =>
                    {
                        Ok(())
                    }
                    PhpType::Object(_) => Err(CompileError::new(
                        stmt.span,
                        "Type error: throw requires an object implementing Throwable",
                    )),
                    // Gradual operands — `Mixed` (e.g. `throw new <class not in the closed
                    // world>`, which degrades to `Mixed`), `?Object`/`Object|null`, an
                    // object-plus-scalar union, or any union containing an object/`Mixed`
                    // member — cannot have their Throwable contract proven statically. PHP
                    // only enforces it at runtime, so accept here (the EIR backend still
                    // faults a non-Throwable at throw time). A proven non-object stays loud.
                    ref ty if type_is_gradual_object_family(ty) => Ok(()),
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
                    if let Err(error) = self.check_conditional_stmt(s, env) {
                        errors.extend(error.flatten());
                    }
                }
                for catch_clause in catches {
                    let mut resolved_types = Vec::new();
                    for raw_exception_type in &catch_clause.exception_types {
                        let exception_type =
                            self.resolve_catch_type_name(raw_exception_type, stmt.span)?;
                        if !self.classes.contains_key(&exception_type)
                            && !self.interfaces.contains_key(&exception_type)
                        {
                            // An exception type that is unknown everywhere in the closed world is
                            // an absent optional dependency (e.g. `ProcessFailedException`): warn
                            // and skip it rather than erroring. The class does not exist at runtime,
                            // so the clause simply never matches; `$e` is bound below from whatever
                            // types did resolve (falling back to `Throwable` when none did).
                            if !self.class_like_exists(&exception_type) {
                                self.warn_absent_class(stmt.span, &exception_type);
                                continue;
                            }
                            return Err(CompileError::new(
                                stmt.span,
                                &format!("Undefined class: {}", exception_type),
                            ));
                        }
                        if !self.object_type_implements_throwable(&exception_type) {
                            return Err(CompileError::new(
                                stmt.span,
                                &format!(
                                    "Catch type must extend or implement Throwable: {}",
                                    exception_type
                                ),
                            ));
                        }
                        resolved_types.push(exception_type);
                    }
                    if let Some(variable) = &catch_clause.variable {
                        env.insert(
                            variable.clone(),
                            PhpType::Object(self.common_catch_type_name(&resolved_types)),
                        );
                    }
                    for s in &catch_clause.body {
                        if let Err(error) = self.check_conditional_stmt(s, env) {
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

    /// Checks a `for` statement in PHP execution order while keeping its initializer precise.
    ///
    /// A nested `for` can itself sit on a conditional path, but once its body is entered the
    /// initializer has definitely executed. Temporarily removing the enclosing conditional depth
    /// lets that initializer overwrite stale flow types for the body; the completed loop path is
    /// joined back with the entry environment before returning to the enclosing conditional path.
    fn check_for_stmt(
        &mut self,
        loop_span: crate::span::Span,
        init: Option<&Stmt>,
        condition: Option<&Expr>,
        update: Option<&Stmt>,
        body: &[Stmt],
        env: &mut TypeEnv,
    ) -> Result<(), CompileError> {
        let enclosing_conditional_depth = self.conditional_assignment_depth;
        let conditional_entry_env =
            (enclosing_conditional_depth > 0).then(|| env.clone());
        if conditional_entry_env.is_some() {
            self.conditional_assignment_depth = 0;
        }

        let result = (|| {
            if let Some(stmt) = init {
                self.check_stmt(stmt, env)?;
            }
            stabilize_loop_storage(self, loop_span, body, update, env);
            if let Some(expr) = condition {
                self.infer_type_with_assignment_effects(expr, env)?;
            }
            // Check in PHP execution order: on the first iteration the body observes the types
            // established by `init`, while `update` runs only after that body. This is especially
            // important for integer `++`/`--`: their post-update type is `Mixed` because overflow
            // may promote to float, but that future type must not infect the preceding body.
            let errors = self.check_break_continue_target_body(body, env);
            if let Some(stmt) = update {
                self.check_conditional_stmt(stmt, env)?;
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(CompileError::from_many(errors))
            }
        })();

        self.conditional_assignment_depth = enclosing_conditional_depth;
        if let Some(entry_env) = conditional_entry_env {
            let path_env = std::mem::replace(env, entry_env);
            self.merge_switch_path_env(env, &path_env);
        }
        result
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

    /// Joins one switch path into the conservative environment used after the switch.
    /// Writes a post-`if` flow type, honouring an ENCLOSING conditional.
    ///
    /// Both post-`if` writers compute a type that is exact for every branch continuing past THIS
    /// `if`. That is not the same as exact after the ENCLOSING construct: in
    /// `if ($tag) { if ($a) { $x = new T(); } else { throw; } }` the inner chain does make `T`
    /// definite — every arm that continues assigns it — but the OUTER branch may be skipped
    /// entirely, so after the outer `if` the type is `T` joined with what `$x` already was.
    ///
    /// A plain assignment already obeys that rule: `check_assign` unions with the existing type
    /// whenever `conditional_assignment_depth > 0`. These two writers used a bare `env.insert` and
    /// replaced it outright, so `$x` left the outer `if` as `T` alone. The damage was SILENT — a
    /// later `$x['k']` read compiled against an object and produced `0` instead of the element,
    /// with no diagnostic. Symfony's `ContentLoaderTrait::resolveServices` is exactly this shape,
    /// which is why the same root also surfaced as `Cannot index non-array` there once the wrong
    /// type reached a stricter position.
    ///
    /// A throwing arm is what exposes it: without one, the chain's implicit false edge contributes
    /// the original type to the join and the union comes out right by accident.
    fn assign_post_if_flow_type(&self, env: &mut TypeEnv, name: String, ty: PhpType) {
        if self.conditional_assignment_depth == 0 {
            env.insert(name, ty);
            return;
        }
        let Some(existing) = env.get(&name).cloned() else {
            env.insert(name, ty);
            return;
        };
        if existing == ty {
            return;
        }
        let resolved = self
            .merged_assignment_type(&existing, &ty)
            .unwrap_or_else(|| self.normalize_union_type(vec![existing, ty]));
        env.insert(name, resolved);
    }

    fn merge_switch_path_env(&self, merged: &mut TypeEnv, path: &TypeEnv) {
        for (name, path_ty) in path {
            let Some(existing) = merged.get(name).cloned() else {
                merged.insert(name.clone(), path_ty.clone());
                continue;
            };
            if existing == *path_ty {
                continue;
            }
            merged.insert(
                name.clone(),
                self.normalize_union_type(vec![existing, path_ty.clone()]),
            );
        }
    }

    /// Checks each statement in a body sequentially, collecting all errors.
    ///
    /// Unlike `check_break_continue_target_body`, this does not update `break_continue_depth`.
    /// Used for `switch` cases, `if` branches, `try` blocks, and other bodies where the
    /// break/continue level is managed at a higher level.
    fn check_body(&mut self, body: &[Stmt], env: &mut TypeEnv) -> Vec<CompileError> {
        self.conditional_assignment_depth += 1;
        let mut errors = Vec::new();
        for stmt in body {
            if let Err(error) = self.check_stmt(stmt, env) {
                errors.extend(error.flatten());
            }
        }
        self.conditional_assignment_depth -= 1;
        errors
    }

    /// Checks one conditional path while retaining precise direct-overwrite types between its
    /// statements, then restores the conservative storage joins before returning.
    ///
    /// The returned map contains locals whose precise overwrite survived to the path exit. The
    /// enclosing `if` may keep such a type only when every continuing branch reports the same
    /// overwrite. The third element is the `(name, type)` of a trailing `$name = <expr>` statement,
    /// inferred with the pre-statement environment: nothing runs after it on this path, so it is the
    /// variable's precise flow type at the branch exit — used both for single-guard convergence and
    /// for the post-`if` branch join (a conditional store otherwise widens the local's env entry
    /// back to its conservative frame slot, discarding the assigned value).
    fn check_conditional_path_body(
        &mut self,
        body: &[Stmt],
        precise_object_overwrites: &BTreeMap<String, PhpType>,
        env: &mut TypeEnv,
    ) -> (
        Vec<CompileError>,
        BTreeMap<String, PhpType>,
        Option<(String, PhpType)>,
    ) {
        let mut errors = Vec::new();
        let mut precise_overwrites = BTreeMap::new();
        let mut conservative_types = BTreeMap::new();
        let mut final_assignment = None;
        for (index, stmt) in body.iter().enumerate() {
            if index + 1 == body.len() {
                if let StmtKind::Assign { name, value } = &stmt.kind {
                    final_assignment = self
                        .infer_type(value, env)
                        .ok()
                        .map(|ty| (name.clone(), ty));
                }
            }
            let direct_overwrite = match &stmt.kind {
                StmtKind::Assign { name, value }
                    if precise_object_overwrites.contains_key(name)
                        && !matches!(
                            &value.kind,
                            ExprKind::NullCoalesce { value: current, .. }
                                if matches!(&current.kind, ExprKind::Variable(current_name) if current_name == name)
                        ) =>
                {
                    self.infer_type(value, env)
                        .ok()
                        .or_else(|| self.assignment_recovery_call_return_type(value))
                        .filter(|ty| precise_object_overwrites.get(name) == Some(ty))
                        .map(|ty| (name.clone(), ty))
                }
                // Flow-sensitive concrete-object assignment: `$x = new Concrete()` types `$x` as
                // that exact class for the reads that follow it on THIS path, even when a sibling
                // branch assigns a different (e.g. base) class and the storage-wide join is the
                // common base. The join is restored at branch exit (via `conservative_types`), so
                // the type after the whole `if` stays the conservative least-upper-bound — a
                // Derived-only call past the merge still errors.
                StmtKind::Assign { name, value }
                    if matches!(&value.kind, ExprKind::NewObject { .. }) =>
                {
                    match self.infer_type(value, env) {
                        Ok(PhpType::Object(class)) if !class.is_empty() => {
                            Some((name.clone(), PhpType::Object(class)))
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
            let result = self.check_conditional_stmt(stmt, env);
            if let Err(error) = result {
                errors.extend(error.flatten());
            }

            if let Some((name, ty)) = direct_overwrite {
                if let Some(conservative) = env.get(&name).cloned() {
                    conservative_types.insert(name.clone(), conservative);
                    env.insert(name.clone(), ty.clone());
                    precise_overwrites.insert(name, ty);
                };
            } else {
                precise_overwrites.retain(|name, ty| env.get(name) == Some(ty));
            }
        }
        for name in precise_overwrites.keys() {
            if let Some(conservative) = conservative_types.get(name).cloned() {
                env.insert(name.clone(), conservative);
            }
        }
        (errors, precise_overwrites, final_assignment)
    }

    /// Returns direct local overwrites by concrete `new` expressions that survive to a branch's
    /// end; a later direct assignment to the same local invalidates the earlier object fact.
    fn direct_object_overwrites(body: &[Stmt]) -> BTreeMap<String, PhpType> {
        let mut overwrites = BTreeMap::new();
        for stmt in body {
            if let StmtKind::Assign { name, value } = &stmt.kind {
                if let ExprKind::NewObject { class_name, .. } = &value.kind {
                    overwrites.insert(
                        name.clone(),
                        PhpType::Object(class_name.as_str().trim_start_matches('\\').to_string()),
                    );
                } else {
                    overwrites.remove(name);
                }
            }
        }
        overwrites
    }

    /// Intersects direct object overwrites across every explicit branch, retaining only locals
    /// assigned the same concrete class on all paths.
    fn convergent_direct_object_overwrites(
        branches: &[&[Stmt]],
    ) -> BTreeMap<String, PhpType> {
        let Some(first) = branches.first() else {
            return BTreeMap::new();
        };
        let mut convergent = Self::direct_object_overwrites(first);
        for branch in branches.iter().skip(1) {
            let branch_overwrites = Self::direct_object_overwrites(branch);
            convergent.retain(|name, ty| branch_overwrites.get(name) == Some(ty));
        }
        convergent
    }

    /// Checks one statement whose writes may be skipped by the surrounding control-flow edge.
    fn check_conditional_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut TypeEnv,
    ) -> Result<(), CompileError> {
        self.conditional_assignment_depth += 1;
        let result = self.check_stmt(stmt, env);
        self.conditional_assignment_depth -= 1;
        result
    }
}
