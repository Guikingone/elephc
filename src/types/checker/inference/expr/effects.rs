//! Purpose:
//! Infers expression effects forms for the checker.
//! Handles type facts and diagnostics for expression shapes that need more than scalar/operator inference.
//!
//! Called from:
//! - `crate::types::checker::inference::expr`
//!
//! Key details:
//! - Expression inference shares environments with statement checking, so variable and effect updates must stay synchronized.

use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::{BinOp, CallableTarget, Expr, ExprKind};
use crate::types::{PhpType, TypeEnv};

use super::super::super::Checker;
use super::super::syntactic::wider_type_syntactic;

impl Checker {
    /// Infers the type of an expression while tracking assignment effects through the environment.
    ///
    /// Handles expression forms where variable assignments within sub-expressions must be
    /// visible to later parts of the same expression (e.g., `$a = 1, $a + 2` in ternary/loop contexts).
    /// For most expressions, simply delegates to `infer_type`; for control-flow expressions
    /// (ternary, null coalesce, match), clones the environment to isolate branch-specific bindings
    /// from influencing other branches.
    ///
    /// # Arguments
    /// * `expr` - The expression to infer
    /// * `env` - The type environment, mutated in-place for side-effectful sub-expressions
    ///
    /// # Returns
    /// The inferred `PhpType` on success, or a `CompileError` if type checking fails.
    ///
    /// # Key details
    /// - Assignment expressions call `check_assignment_expression` to properly register the binding.
    /// - Binary `&&`/`||` clone the environment before the right branch to prevent assignments
    ///   in the left branch from leaking into the right branch (PHP semantics).
    /// - Ternary, null coalesce, and match clone the environment per branch so assignments
    ///   do not leak across arms; match/ternary result types then reuse `infer_type`'s
    ///   Mixed-aware branch merge (not the Str-absorbing syntactic join).
    /// - `preg_replace_callback` argument at index 1 is skipped (special handling for capture groups).
    pub(crate) fn infer_type_with_assignment_effects(
        &mut self,
        expr: &Expr,
        env: &mut TypeEnv,
    ) -> Result<PhpType, CompileError> {
        match &expr.kind {
            ExprKind::Variable(name) if self.eval_barrier_active && !env.contains_key(name) => {
                env.insert(name.clone(), PhpType::Mixed);
                Ok(PhpType::Mixed)
            }
            ExprKind::Assignment {
                target,
                value,
                result_target,
                prelude,
                ..
            } => {
                let ty = self.check_assignment_expression(
                    target,
                    value,
                    result_target.as_deref(),
                    prelude,
                    expr.span,
                    env,
                )?;
                // A write through to a property (directly or via one of its elements)
                // invalidates every property narrowing.
                match &target.kind {
                    ExprKind::Variable(name) => {
                        Self::purge_property_narrowings_for_root(env, name)
                    }
                    _ if assignment_may_write_property(target) => {
                        Self::purge_property_narrowings(env)
                    }
                    _ => {}
                }
                Ok(ty)
            }
            ExprKind::PreIncrement(name) | ExprKind::PreDecrement(name) => {
                let old_ty = env.get(name).cloned();
                let result_ty = self.infer_type(expr, env)?;
                if matches!(old_ty, Some(PhpType::Int)) {
                    env.insert(name.clone(), PhpType::Mixed);
                }
                Ok(result_ty)
            }
            ExprKind::PostIncrement(name) | ExprKind::PostDecrement(name) => {
                let old_ty = env.get(name).cloned();
                let result_ty = self.infer_type(expr, env)?;
                if matches!(old_ty, Some(PhpType::Int)) {
                    env.insert(name.clone(), PhpType::Mixed);
                }
                Ok(result_ty)
            }
            ExprKind::BinaryOp { left, op, right } => {
                if matches!(op, BinOp::And | BinOp::Or) {
                    // PHP evaluates a short-circuit chain left-to-right, so an assignment in an
                    // earlier operand is visible to every later operand of the same chain (e.g.
                    // `... && ($w = strspn(...)) < n && '#' !== $line[$w]`). The chain is
                    // left-associative, so the naive "clone the env for the right operand" approach
                    // hides a nested operand's assignments from the operands that run after it.
                    //
                    // Flatten the chain into its source-order operands instead. Only operands joined
                    // by the *same* operator are flattened: in a pure `&&` chain every earlier
                    // operand definitely ran when a later one runs, and likewise for a pure `||`
                    // chain (each later operand runs only after the earlier ones evaluated to
                    // false). Mixing `&&` and `||` is left as a nested boundary, handled by the
                    // recursive call, so we never treat a conditionally-skipped operand's
                    // assignment as visible.
                    //
                    // The first operand always runs, so it is processed into `env` and its ordinary
                    // assignments stay definitely-assigned past the chain. The remaining operands
                    // are threaded through a single cloned `chain_env` for left-to-right visibility
                    // without leaking their (conditionally evaluated) assignments to the outer
                    // scope. By-reference call outputs in later operands are still surfaced to `env`,
                    // matching PHP's non-flow-sensitive undefined-variable behavior for out-params.
                    let mut operands = Vec::new();
                    flatten_short_circuit_operands(expr, op, &mut operands);
                    // Thread `chain_env` left-to-right so each operand is checked with the
                    // assignments AND the type-guard narrowings implied by the operands that ran
                    // before it. The first operand runs unconditionally, so it is inferred into
                    // `env` (its definite assignments survive the chain); `chain_env` is then
                    // re-cloned from `env` and every later operand mutates only `chain_env`.
                    let mut chain_env = env.clone();
                    for (i, &operand) in operands.iter().enumerate() {
                        if i == 0 {
                            self.infer_type_with_assignment_effects(operand, env)?;
                            chain_env = env.clone();
                        } else {
                            self.infer_type_with_assignment_effects(operand, &mut chain_env)?;
                            self.define_nested_by_ref_outputs(operand, env);
                        }
                        // Short-circuit type-guard narrowing: in a pure `&&` chain a later operand
                        // runs only when this one was truthy, so a recognized type guard here
                        // narrows its variable to the guard-true (`then_ty`) type for the operands
                        // that follow; in a pure `||` chain a later operand runs only when this one
                        // was falsy, so the guard-false (`else_ty`) complement applies. The narrowing
                        // is read from and written to `chain_env` so repeated guards refine
                        // cumulatively (e.g. `$x instanceof A && $x instanceof B`) and a variable
                        // reassigned by an earlier operand is seen at its post-assignment type. This
                        // runs for the FIRST operand too, which is frequently the guard itself
                        // (`$q instanceof CQ && $q->m()`). Narrowing stays inside `chain_env`; the
                        // `or_insert` surfacing below never overwrites an outer-scope type, so it
                        // does not leak past the chain.
                        if let Some(g) = self.type_guard_narrowing(operand, &chain_env) {
                            let narrowed = if matches!(op, BinOp::And) {
                                g.then_ty
                            } else {
                                g.else_ty
                            };
                            chain_env.insert(g.var, narrowed);
                        }
                    }
                    // Surface ordinary assignments made in later (conditionally-evaluated) operands
                    // to the outer scope, mirroring the by-ref-output surfacing above and PHP's
                    // non-flow-sensitive undefined-variable behavior: a variable first assigned in a
                    // `&&`/`||` operand is usable after the chain (e.g. `... && ($u = 5) > 0` then
                    // read `$u`). `or_insert` only DEFINES a currently-undefined variable — it never
                    // overwrites an existing type, so the flow-sensitive narrowing threaded through
                    // `chain_env` for variables that already existed does not leak to the outer scope.
                    for (var, ty) in chain_env {
                        env.entry(var).or_insert(ty);
                    }
                    Ok(PhpType::Bool)
                } else {
                    self.infer_type_with_assignment_effects(left, env)?;
                    self.infer_type_with_assignment_effects(right, env)?;
                    self.infer_type(expr, env)
                }
            }
            ExprKind::NullCoalesce { value, default } => {
                let value_ty = self.infer_type_with_assignment_effects(value, env)?;
                let default_ty = if value_ty == PhpType::Void {
                    self.infer_type_with_assignment_effects(default, env)?
                } else {
                    let mut default_env = env.clone();
                    self.infer_type_with_assignment_effects(default, &mut default_env)?
                };
                if Self::union_contains_void(&value_ty) {
                    Ok(wider_type_syntactic(
                        &self.strip_void_from_union(&value_ty),
                        &default_ty,
                    ))
                } else {
                    Ok(wider_type_syntactic(&value_ty, &default_ty))
                }
            }
            ExprKind::ShortTernary { value, default } => {
                let value_ty = self.infer_type_with_assignment_effects(value, env)?;
                if value_ty == PhpType::Void {
                    self.infer_type_with_assignment_effects(default, env)?;
                } else {
                    let mut default_env = env.clone();
                    self.infer_type_with_assignment_effects(default, &mut default_env)?;
                }
                // Result type comes from the Mixed-aware short-ternary merge in `infer_type`.
                self.infer_type(expr, env)
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.infer_type_with_assignment_effects(condition, env)?;
                // Flow-narrowing across the branches (see guard_narrowing): `$x instanceof X`
                // and simple `$x->prop instanceof X` guards narrow the branch envs. A ternary is a
                // single expression, so branch narrowing is write-invalidation-safe.
                let guard = self.guard_narrowing(condition, env)?;
                let mut then_env = env.clone();
                let mut else_env = env.clone();
                if let Some(guard) = guard {
                    then_env.insert(guard.var.clone(), guard.then_ty);
                    else_env.insert(guard.var, guard.else_ty);
                }
                let then_ty =
                    self.infer_type_with_assignment_effects(then_expr, &mut then_env)?;
                let else_ty =
                    self.infer_type_with_assignment_effects(else_expr, &mut else_env)?;
                // By-reference call outputs in the cloned branches define their out-parameters
                // for later code, mirroring PHP's undefined-variable behavior.
                self.define_nested_by_ref_outputs(then_expr, env);
                self.define_nested_by_ref_outputs(else_expr, env);
                // Merge with the same Mixed-aware join as `infer_type`'s ternary arm, computed
                // directly from the (narrowed) branch inferences: re-inferring the whole ternary
                // through the plain path would drop the short-circuit chain narrowing threaded
                // through the condition and falsely reject guarded member calls.
                Ok(super::merge_match_arm_result_type(self, then_ty, else_ty))
            }
            ExprKind::ArrayLiteral(elems) => {
                for elem in elems {
                    self.infer_type_with_assignment_effects(elem, env)?;
                }
                self.infer_type(expr, env)
            }
            ExprKind::ArrayLiteralAssoc(pairs) => {
                for (key, value) in pairs {
                    self.infer_type_with_assignment_effects(key, env)?;
                    self.infer_type_with_assignment_effects(value, env)?;
                }
                self.infer_type(expr, env)
            }
            ExprKind::Match {
                subject,
                arms,
                default,
            } => {
                self.infer_type_with_assignment_effects(subject, env)?;
                for (conditions, result) in arms {
                    let mut arm_env = env.clone();
                    for condition in conditions {
                        self.infer_type_with_assignment_effects(condition, &mut arm_env)?;
                    }
                    self.infer_type_with_assignment_effects(result, &mut arm_env)?;
                }
                if let Some(default) = default {
                    let mut default_env = env.clone();
                    self.infer_type_with_assignment_effects(default, &mut default_env)?;
                }
                // Result type comes from the Mixed-aware match merge in `infer_type`
                // (assignment effects must not reintroduce the Str-absorbing syntactic join).
                self.infer_type(expr, env)
            }
            ExprKind::ArrayAccess { array, index } => {
                self.infer_type_with_assignment_effects(array, env)?;
                self.infer_type_with_assignment_effects(index, env)?;
                self.infer_type(expr, env)
            }
            ExprKind::Negate(inner)
            | ExprKind::Not(inner)
            | ExprKind::BitNot(inner)
            | ExprKind::Throw(inner)
            | ExprKind::ErrorSuppress(inner)
            | ExprKind::Print(inner)
            | ExprKind::Spread(inner) => {
                self.infer_type_with_assignment_effects(inner, env)?;
                self.infer_type(expr, env)
            }
            ExprKind::Cast { expr: inner, .. } | ExprKind::PtrCast { expr: inner, .. } => {
                self.infer_type_with_assignment_effects(inner, env)?;
                self.infer_type(expr, env)
            }
            ExprKind::FunctionCall { name, args } => {
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                // A user function with a by-reference parameter defines the caller's argument
                // variable, just like the builtin out-parameter handling below. Define such
                // variables before inferring the arguments so the (otherwise undefined)
                // by-reference variable is not reported as "Undefined variable".
                for (var, ty) in self.function_call_by_ref_outputs(name, &expanded_args, env) {
                    env.entry(var).or_insert(ty);
                }
                // Promote already-defined caller variables whose storage cannot hold the
                // boxed/nullable value a by-reference parameter may write back.
                for (var, ty) in
                    self.function_call_by_ref_boxed_promotions(name, &expanded_args, env)
                {
                    env.insert(var, ty);
                }
                let builtin_name = name.trim_start_matches('\\');
                // A builtin by-reference out-parameter auto-vivifies the caller's variable (PHP
                // definite-assignment semantics), mirroring the user-function path above. Define
                // such variables BEFORE inferring the call so the variable stays defined even when
                // the call's own inference recovers from an error.
                //
                // `builtin_call_by_ref_outputs` returns only entries that must be applied: an
                // as-yet-undefined var (any by-ref param) OR an already-defined var bound to an
                // OUT-ONLY param (preg_match/preg_match_all `$matches`, preg_replace/
                // preg_replace_callback `$count`), which PHP overwrites wholesale. Using `insert`
                // (not `or_insert`) therefore re-types the aliased subject-as-out-param shape
                // `preg_match_all(…, $s, $s)` while leaving IN-OUT params untouched.
                for (var, out_ty) in
                    Checker::builtin_call_by_ref_outputs(builtin_name, &expanded_args, env)
                {
                    env.insert(var, out_ty);
                }
                // Builtin by-reference out-parameter positions come from the canonical
                // signature's `ref_params` (preg_match/preg_match_all &$matches, preg_replace
                // &$count, parse_str &$result, proc_open &$pipes, ...): such arguments are not
                // eagerly inferred here (they may be as-yet undefined) — they were defined in
                // the caller scope above.
                let builtin_sig = crate::types::builtin_call_sig(builtin_name);
                let is_builtin_by_ref = |idx: usize| {
                    builtin_sig
                        .as_ref()
                        .map_or(false, |sig| sig.ref_params.get(idx).copied().unwrap_or(false))
                };
                // `isset`/`unset` are lazy language constructs: an operand may be
                // an undeclared property routed to `__isset`/`__unset`, which must
                // not be inferred as a bare property access here. The call's own
                // inference handles the operands (with magic routing).
                if matches!(
                    php_symbol_key(builtin_name).as_str(),
                    "isset" | "unset"
                ) {
                    for arg in &expanded_args {
                        self.infer_non_reading_arg_assignment_effects(arg, env)?;
                    }
                } else if !builtin_name.eq_ignore_ascii_case("unset") {
                    for (idx, arg) in expanded_args.iter().enumerate() {
                        if is_builtin_by_ref(idx) {
                            continue;
                        }
                        if builtin_name.eq_ignore_ascii_case("preg_replace_callback") && idx == 1 {
                            continue;
                        }
                        // The user-sort comparator is type-checked by `check_builtin`
                        // with its parameters typed from the array element (so an
                        // unannotated object comparator type-checks). Skip the eager
                        // pass here, which would otherwise check the comparator body
                        // with default `Int` parameters and reject object access.
                        if idx == 1
                            && (builtin_name.eq_ignore_ascii_case("usort")
                                || builtin_name.eq_ignore_ascii_case("uasort")
                                || builtin_name.eq_ignore_ascii_case("uksort"))
                        {
                            continue;
                        }
                        self.infer_type_with_assignment_effects(arg, env)?;
                    }
                }
                let ty = self.infer_type(expr, env)?;
                // The callee may mutate any reachable object; drop property narrowings. (The
                // call's own argument checking above still saw them.)
                Self::purge_property_narrowings(env);
                if builtin_name.eq_ignore_ascii_case("preg_match") {
                    if let Some(arg) = expanded_args.get(2) {
                        if let Some(name) = preg_match_output_var(arg) {
                            env.insert(name.clone(), PhpType::Array(Box::new(PhpType::Str)));
                        }
                    }
                }
                if builtin_name.eq_ignore_ascii_case("unset") {
                    for arg in &expanded_args {
                        promote_indexed_local_for_element_unset(arg, env);
                    }
                }
                if builtin_name.eq_ignore_ascii_case("eval") {
                    self.mark_eval_barrier(env);
                }
                Ok(ty)
            }
            ExprKind::NewObject { args, .. } | ExprKind::StaticMethodCall { args, .. } => {
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                for arg in &expanded_args {
                    self.infer_type_with_assignment_effects(arg, env)?;
                }
                let ty = self.infer_type(expr, env)?;
                Self::purge_property_narrowings(env);
                Ok(ty)
            }
            ExprKind::ClosureCall { var, args } => {
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                let skip_contextual_callback =
                    self.variable_targets_preg_replace_callback(var.as_str());
                for (idx, arg) in expanded_args.iter().enumerate() {
                    if skip_contextual_callback && idx == 1 {
                        continue;
                    }
                    self.infer_type_with_assignment_effects(arg, env)?;
                }
                let ty = self.infer_type(expr, env)?;
                Self::purge_property_narrowings(env);
                Ok(ty)
            }
            ExprKind::ExprCall { callee, args } => {
                self.infer_type_with_assignment_effects(callee, env)?;
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                let skip_contextual_callback = self
                    .expr_targets_preg_replace_callback(callee);
                for (idx, arg) in expanded_args.iter().enumerate() {
                    if skip_contextual_callback && idx == 1 {
                        continue;
                    }
                    self.infer_type_with_assignment_effects(arg, env)?;
                }
                let ty = self.infer_type(expr, env)?;
                Self::purge_property_narrowings(env);
                Ok(ty)
            }
            ExprKind::NamedArg { value, .. } => {
                self.infer_type_with_assignment_effects(value, env)?;
                self.infer_type(expr, env)
            }
            ExprKind::PropertyAccess { object, .. }
            | ExprKind::NullsafePropertyAccess { object, .. } => {
                self.infer_type_with_assignment_effects(object, env)?;
                self.infer_type(expr, env)
            }
            ExprKind::DynamicPropertyAccess { object, property }
            | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
                self.infer_type_with_assignment_effects(object, env)?;
                self.infer_type_with_assignment_effects(property, env)?;
                self.infer_type(expr, env)
            }
            ExprKind::MethodCall { object, args, .. }
            | ExprKind::NullsafeMethodCall { object, args, .. } => {
                self.infer_type_with_assignment_effects(object, env)?;
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                for arg in &expanded_args {
                    self.infer_type_with_assignment_effects(arg, env)?;
                }
                let ty = self.infer_type(expr, env)?;
                Self::purge_property_narrowings(env);
                Ok(ty)
            }
            ExprKind::NullsafeDynamicMethodCall {
                object,
                method,
                args,
            } => {
                self.infer_type_with_assignment_effects(object, env)?;
                self.infer_type_with_assignment_effects(method, env)?;
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                for arg in &expanded_args {
                    self.infer_type_with_assignment_effects(arg, env)?;
                }
                self.infer_type(expr, env)
            }
            ExprKind::BufferNew { len, .. } => {
                self.infer_type_with_assignment_effects(len, env)?;
                self.infer_type(expr, env)
            }
            ExprKind::NewScopedObject { args, .. } => {
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                for arg in &expanded_args {
                    self.infer_type_with_assignment_effects(arg, env)?;
                }
                let ty = self.infer_type(expr, env)?;
                Self::purge_property_narrowings(env);
                Ok(ty)
            }
            _ => self.infer_type(expr, env),
        }
    }

    /// Infers effects for a language-construct operand without treating properties as reads.
    fn infer_non_reading_arg_assignment_effects(
        &mut self,
        arg: &Expr,
        env: &mut TypeEnv,
    ) -> Result<(), CompileError> {
        match &arg.kind {
            ExprKind::PropertyAccess { object, .. }
            | ExprKind::NullsafePropertyAccess { object, .. } => {
                self.infer_type_with_assignment_effects(object, env)?;
                Ok(())
            }
            ExprKind::DynamicPropertyAccess { object, property }
            | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
                self.infer_type_with_assignment_effects(object, env)?;
                self.infer_type_with_assignment_effects(property, env)?;
                Ok(())
            }
            ExprKind::ArrayAccess { array, index } => {
                self.infer_type_with_assignment_effects(array, env)?;
                self.infer_type_with_assignment_effects(index, env)?;
                Ok(())
            }
            ExprKind::NamedArg { value, .. } => {
                self.infer_non_reading_arg_assignment_effects(value, env)
            }
            _ => {
                self.infer_type_with_assignment_effects(arg, env)?;
                Ok(())
            }
        }
    }

    /// Returns true when an expression call target is first-class `preg_replace_callback`.
    fn expr_targets_preg_replace_callback(&self, callee: &Expr) -> bool {
        match &callee.kind {
            ExprKind::FirstClassCallable(target) => callable_target_is_preg_replace_callback(target),
            ExprKind::Variable(var_name) => {
                self.variable_targets_preg_replace_callback(var_name.as_str())
            }
            _ => false,
        }
    }

    /// Returns true when a variable stores first-class `preg_replace_callback`.
    fn variable_targets_preg_replace_callback(&self, var_name: &str) -> bool {
        self.first_class_callable_targets
            .get(var_name)
            .is_some_and(callable_target_is_preg_replace_callback)
    }

    /// Marks the active statement stream as having crossed eval and widens local facts.
    fn mark_eval_barrier(&mut self, env: &mut TypeEnv) {
        self.eval_barrier_active = true;
        let local_names = env.keys().cloned().collect::<Vec<_>>();
        for ty in env.values_mut() {
            *ty = PhpType::Mixed;
        }
        for name in local_names {
            self.closure_return_types.remove(&name);
            self.callable_sigs.remove(&name);
            self.callable_captures.remove(&name);
            self.callable_array_targets.remove(&name);
            self.first_class_callable_targets.remove(&name);
        }
    }
}

/// Returns true when a first-class callable target is PHP `preg_replace_callback`.
fn callable_target_is_preg_replace_callback(target: &CallableTarget) -> bool {
    matches!(
        target,
        CallableTarget::Function(name) if php_symbol_key(name.as_str()) == "preg_replace_callback"
    )
}

/// Returns true when an assignment target can write through to an object property — directly
/// (`$obj->p = …`) or via an element of one (`$obj->p[0] = …`) — invalidating property
/// narrowings. Plain variables (and elements of plain variables) cannot.
fn assignment_may_write_property(target: &Expr) -> bool {
    match &target.kind {
        ExprKind::Variable(_) => false,
        ExprKind::ArrayAccess { array, .. } => assignment_may_write_property(array),
        _ => true,
    }
}

/// Returns the variable name used as `preg_match()`'s output `$matches` argument.
fn preg_match_output_var(arg: &Expr) -> Option<&String> {
    match &arg.kind {
        ExprKind::Variable(name) => Some(name),
        ExprKind::NamedArg { value, .. } => preg_match_output_var(value),
        _ => None,
    }
}

/// Promotes a packed indexed-array local to an associative array when one of its elements is
/// removed via `unset($arr[$key])`.
///
/// PHP's `unset()` removes a key without renumbering the remaining elements, so the array can no
/// longer be a contiguous packed list (e.g. `unset([1,2,3][1])` leaves keys `0` and `2`). Re-typing
/// the local as `AssocArray<Int, T>` makes its literal build as a hash, so the element removal
/// lowers through `HashUnset`. Only plain `$var[$key]` targets on a currently-packed array are
/// affected; associative arrays, objects, and non-variable receivers are left unchanged.
fn promote_indexed_local_for_element_unset(arg: &Expr, env: &mut TypeEnv) {
    let ExprKind::ArrayAccess { array, index, .. } = &arg.kind else {
        return;
    };
    let ExprKind::Variable(name) = &array.kind else {
        return;
    };
    let Some(PhpType::Array(elem_ty)) = env.get(name).cloned() else {
        return;
    };
    let idx_ty = crate::types::array_keys::normalized_array_key_type(
        index,
        super::super::syntactic::infer_expr_type_syntactic(index),
    );
    let key_ty = if idx_ty == PhpType::Int {
        PhpType::Int
    } else {
        PhpType::Mixed
    };
    let value_ty = if *elem_ty == PhpType::Never {
        PhpType::Mixed
    } else {
        *elem_ty
    };
    env.insert(
        name.clone(),
        PhpType::AssocArray {
            key: Box::new(key_ty),
            value: Box::new(value_ty),
        },
    );
}

/// Collects, in source order, the operands of a left-associative short-circuit chain joined by `op`.
///
/// `op` is the chain's logical operator (`&&` or `||`). The function recurses into nested
/// `BinaryOp` nodes only while they use the *same* operator, appending every other expression as a
/// leaf operand. Mixing `&&` and `||` therefore stops the flattening at the operator boundary: a
/// differently-joined sub-expression becomes a single leaf, so a conditionally-skipped operand's
/// assignments are never threaded into operands that run after it. The resulting order matches
/// PHP's left-to-right evaluation order, which the caller relies on for definite-assignment.
fn flatten_short_circuit_operands<'a>(expr: &'a Expr, op: &BinOp, out: &mut Vec<&'a Expr>) {
    if let ExprKind::BinaryOp {
        left,
        op: inner_op,
        right,
    } = &expr.kind
    {
        if inner_op == op {
            flatten_short_circuit_operands(left, op, out);
            flatten_short_circuit_operands(right, op, out);
            return;
        }
    }
    out.push(expr);
}
