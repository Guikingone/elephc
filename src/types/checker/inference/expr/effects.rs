//! Purpose:
//! Infers expression effects forms for the checker.
//! Handles type facts and diagnostics for expression shapes that need more than scalar/operator inference.
//!
//! Called from:
//! - `crate::types::checker::inference::expr`
//!
//! Key details:
//! - Expression inference shares environments with statement checking, so variable and effect updates must stay synchronized.
//! - Short-circuit `&&`/`||` chains thread a `chain_env` through their flattened operands so a
//!   recognized type guard (`$v instanceof C`, `is_int/...($v)`, optionally `!`-negated) narrows the
//!   guarded variable for the *subsequent* operands: guard-true (`then_ty`) for `&&`, guard-false
//!   (`else_ty`) for `||`. Narrowing stays inside `chain_env` and never leaks past the chain.

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
    /// - Binary `&&`/`||` flatten the same-operator chain and thread a cloned `chain_env` through
    ///   the operands so an earlier operand's assignments are visible to later ones without leaking
    ///   past the chain. Each operand's recognized type guard also narrows its variable in
    ///   `chain_env` for the subsequent operands (`then_ty` for `&&`, `else_ty` for `||`).
    /// - Ternary, null coalesce, and match clone the environment per branch; the result type is
    ///   the wider of all branch types via `wider_type_syntactic`.
    /// - `preg_replace_callback` argument at index 1 is skipped (special handling for capture groups).
    pub(crate) fn infer_type_with_assignment_effects(
        &mut self,
        expr: &Expr,
        env: &mut TypeEnv,
    ) -> Result<PhpType, CompileError> {
        match &expr.kind {
            ExprKind::Assignment {
                target,
                value,
                result_target,
                prelude,
                ..
            } => {
                self.check_assignment_expression(
                    target,
                    value,
                    result_target.as_deref(),
                    prelude,
                    expr.span,
                    env,
                )
            }
            ExprKind::ListUnpack { vars, value } => {
                // `[$a, $b] = EXPR` binds each positional target as a local (so later code,
                // e.g. an `if` body, sees them defined) and evaluates to EXPR. Each target takes
                // the source's element type, or `Mixed` when the source is not a statically-known
                // array (e.g. `$pairs ?? null`, which PHP permits — targets become `null`).
                let value_ty = self.infer_type_with_assignment_effects(value, env)?;
                let elem_ty = match &value_ty {
                    PhpType::Array(elem) => (**elem).clone(),
                    PhpType::AssocArray { value: elem, .. } => (**elem).clone(),
                    _ => PhpType::Mixed,
                };
                for var in vars {
                    env.insert(var.clone(), elem_ty.clone());
                }
                Ok(value_ty)
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
                // By-reference call outputs in the (possibly cloned) default branch define their
                // out-parameters for later code, mirroring PHP's undefined-variable behavior.
                self.define_nested_by_ref_outputs(default, env);
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
                let default_ty = if value_ty == PhpType::Void {
                    self.infer_type_with_assignment_effects(default, env)?
                } else {
                    let mut default_env = env.clone();
                    self.infer_type_with_assignment_effects(default, &mut default_env)?
                };
                // By-reference call outputs in the (possibly cloned) default branch define their
                // out-parameters for later code, mirroring PHP's undefined-variable behavior.
                self.define_nested_by_ref_outputs(default, env);
                Ok(wider_type_syntactic(&value_ty, &default_ty))
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.infer_type_with_assignment_effects(condition, env)?;
                // Flow-sensitive narrowing across the ternary branches, mirroring the
                // if/else narrowing in `control_flow.rs` and the `infer_type` ternary arm.
                // This is the statement-level path (a ternary directly under `return`,
                // an assignment RHS, etc.), so it is where the `is_countable($x) ? count($x)
                // : null` shape is checked. When the condition is a recognized type guard,
                // the guarded variable is narrowed to its then-type in the then-branch and
                // its else-type in the else-branch. Both branch envs are already per-branch
                // clones, so the narrowing never leaks past the ternary.
                let guard = self.type_guard_narrowing(condition, env);
                let mut then_env = env.clone();
                let mut else_env = env.clone();
                if let Some(guard) = &guard {
                    then_env.insert(guard.var.clone(), guard.then_ty.clone());
                    else_env.insert(guard.var.clone(), guard.else_ty.clone());
                }
                if let Some((key, then_ty)) = self.member_path_guard_then(condition) {
                    // then-branch only (see WIN B): the else-branch keeps the declared type.
                    then_env.insert(key, then_ty);
                }
                let then_ty = self.infer_type_with_assignment_effects(then_expr, &mut then_env)?;
                let else_ty = self.infer_type_with_assignment_effects(else_expr, &mut else_env)?;
                // By-reference call outputs in the cloned branches define their out-parameters for
                // later code, mirroring PHP's undefined-variable behavior.
                self.define_nested_by_ref_outputs(then_expr, env);
                self.define_nested_by_ref_outputs(else_expr, env);
                Ok(wider_type_syntactic(&then_ty, &else_ty))
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
                let mut result_ty = None;
                // PHP evaluates match-arm conditions top-to-bottom in a single shared scope, so an
                // assignment in one arm's condition (e.g. `!$length = strlen(...) => ...`) is visible
                // to the conditions of every later arm (`$length < 4 => ...`). Thread one
                // `condition_env` through all arm conditions in source order to model that, instead
                // of giving each arm a fresh clone of the outer env. Each arm body then sees the
                // conditions evaluated up to and including its own arm, but body assignments stay in
                // a per-arm clone so they do not leak to sibling arms (only one body ever runs) or
                // past the match. Ordinary condition assignments are likewise not surfaced to the
                // outer `env`, keeping post-match definite-assignment conservative.
                let mut condition_env = env.clone();
                for (conditions, result) in arms {
                    for condition in conditions {
                        self.infer_type_with_assignment_effects(condition, &mut condition_env)?;
                        // By-reference call outputs in arm conditions define their out-parameters
                        // for later code, mirroring PHP's undefined-variable behavior.
                        self.define_nested_by_ref_outputs(condition, env);
                    }
                    let mut arm_env = condition_env.clone();
                    let arm_ty = self.infer_type_with_assignment_effects(result, &mut arm_env)?;
                    self.define_nested_by_ref_outputs(result, env);
                    result_ty = Some(match result_ty {
                        Some(current) => wider_type_syntactic(&current, &arm_ty),
                        None => arm_ty,
                    });
                }
                if let Some(default) = default {
                    let mut default_env = condition_env.clone();
                    let default_ty =
                        self.infer_type_with_assignment_effects(default, &mut default_env)?;
                    self.define_nested_by_ref_outputs(default, env);
                    result_ty = Some(match result_ty {
                        Some(current) => wider_type_syntactic(&current, &default_ty),
                        None => default_ty,
                    });
                }
                Ok(result_ty.unwrap_or(PhpType::Void))
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
                // variable, just like the builtin `preg_match` out-parameter handling below.
                // Define such variables before inferring the arguments so the (otherwise
                // undefined) by-reference variable is not reported as "Undefined variable".
                for (var, ty) in self.function_call_by_ref_outputs(name, &expanded_args, env) {
                    env.entry(var).or_insert(ty);
                }
                // Promote already-defined caller variables whose storage cannot hold the
                // boxed/nullable value a by-reference parameter may write back.
                for (var, ty) in self.function_call_by_ref_boxed_promotions(name, &expanded_args, env)
                {
                    env.insert(var, ty);
                }
                let builtin_name = name.trim_start_matches('\\');
                // A builtin by-reference out-parameter auto-vivifies the caller's variable (PHP
                // definite-assignment semantics), mirroring the user-function path above. Define
                // such variables BEFORE inferring the call so the variable stays defined even when
                // the call's own inference recovers from an error (e.g. an unrecognized builtin
                // routed through `check_function_call`), which would otherwise skip a post-call
                // define and leave the next read reported as "Undefined variable".
                //
                // `builtin_call_by_ref_outputs` returns only entries that must be applied: an
                // as-yet-undefined var (any by-ref param) OR an already-defined var bound to an
                // OUT-ONLY param (preg_match/preg_match_all `$matches`, preg_replace/
                // preg_replace_callback `$count`), which PHP overwrites wholesale. Using `insert`
                // (not `or_insert`) therefore re-types the aliased subject-as-out-param shape
                // `preg_match_all(…, $s, $s)` while leaving IN-OUT params untouched (they are never
                // returned for an already-defined var, so their existing type is preserved).
                for (var, out_ty) in
                    Checker::builtin_call_by_ref_outputs(builtin_name, &expanded_args, env)
                {
                    env.insert(var, out_ty);
                }
                // Builtin by-reference out-parameter positions come from the canonical signature's
                // `ref_params`. Such parameters (preg_match/preg_match_all &$matches, preg_replace
                // &$count, parse_str &$result, proc_open &$pipes) auto-vivify the caller's variable,
                // so the argument is not eagerly inferred here (it may be as-yet undefined) and is
                // defined in the caller scope after the call returns.
                let builtin_sig = crate::types::builtin_call_sig(builtin_name);
                let is_builtin_by_ref =
                    |idx: usize| builtin_sig.as_ref().map_or(false, |sig| sig.ref_params.get(idx).copied().unwrap_or(false));
                // `isset`/`unset` are lazy language constructs: an operand may be
                // an undeclared property routed to `__isset`/`__unset`, which must
                // not be inferred as a bare property access here. The call's own
                // inference handles the operands (with magic routing).
                let is_lazy_construct = builtin_name.eq_ignore_ascii_case("isset")
                    || builtin_name.eq_ignore_ascii_case("unset");
                if is_lazy_construct {
                    // The operand chain itself is still lazy (an undeclared property must
                    // route through `__isset`/`__unset` magic, handled by the call's own
                    // inference below), but PHP always evaluates the INDEX expression of
                    // every `ArrayAccess` link in the chain regardless of whether the base
                    // exists — `isset($connections[$h = $redis->_target($id)])` defines `$h`
                    // even when `$connections` has no such key yet (php-verified: PHP still
                    // evaluates a nested index when an outer index does not exist, and an
                    // undefined base array triggers no error). Walk just those
                    // always-evaluated index sub-expressions for assignment effects and
                    // nested by-reference outputs so later reads see the definition.
                    for arg in &expanded_args {
                        self.walk_isset_unset_operand_assignment_effects(arg, env)?;
                    }
                } else {
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
                if builtin_name.eq_ignore_ascii_case("unset") {
                    for arg in &expanded_args {
                        promote_indexed_local_for_element_unset(arg, env);
                    }
                }
                Ok(ty)
            }
            ExprKind::NewObject { args, .. } => {
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                for arg in &expanded_args {
                    self.infer_type_with_assignment_effects(arg, env)?;
                }
                self.infer_type(expr, env)
            }
            ExprKind::StaticMethodCall {
                receiver,
                method,
                args,
            } => {
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                // `Closure::bind($closure, $newThis [, $scope])`: the generic per-arg pre-pass
                // below would type-check the closure LITERAL argument out of context — before
                // `infer_static_method_call_type_with_options`'s Closure::bind handling ever
                // gets a chance to relax property-access visibility for a JURY-safe scope
                // rebind — and reject it. Route through the same shared
                // `check_closure_bind_call_args` helper instead, so this `??=`-style
                // assignment-effects pre-pass agrees with the main inference path on whether
                // the rebind applies.
                if matches!(receiver, crate::parser::ast::StaticReceiver::Named(name) if name.as_str().trim_start_matches('\\') == "Closure")
                    && php_symbol_key(method) == "bind"
                {
                    if let Some(closure_arg) = expanded_args.first() {
                        let rest: Vec<&Expr> = expanded_args.get(1..2).unwrap_or(&[]).iter().collect();
                        let scope_arg = expanded_args.get(2);
                        super::super::check_closure_bind_call_args(self, closure_arg, &rest, scope_arg, env)?;
                    }
                    return self.infer_type(expr, env);
                }
                // A static method with a by-reference parameter (e.g. the yaml
                // `Parser::preg_match($re, $value, $match)` shape) defines the caller's argument
                // variable. Define such variables before inferring the arguments so the
                // by-reference variable is not reported as "Undefined variable".
                for (var, ty) in
                    self.static_method_call_by_ref_outputs(receiver, method, &expanded_args, env)
                {
                    env.entry(var).or_insert(ty);
                }
                // Promote already-defined caller variables whose storage cannot hold the
                // boxed/nullable value a by-reference parameter may write back.
                for (var, ty) in self.static_method_call_by_ref_boxed_promotions(
                    receiver,
                    method,
                    &expanded_args,
                    env,
                ) {
                    env.insert(var, ty);
                }
                for arg in &expanded_args {
                    self.infer_type_with_assignment_effects(arg, env)?;
                }
                self.infer_type(expr, env)
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
                self.infer_type(expr, env)
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
                self.infer_type(expr, env)
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
            ExprKind::MethodCall {
                object,
                method,
                args,
            }
            | ExprKind::NullsafeMethodCall {
                object,
                method,
                args,
            } => {
                let object_type = self.infer_type_with_assignment_effects(object, env)?;
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                // An instance method with a by-reference parameter defines the caller's argument
                // variable. Define such variables before inferring the arguments so the
                // by-reference variable is not reported as "Undefined variable".
                for (var, ty) in
                    self.method_call_by_ref_outputs(&object_type, method, &expanded_args, env)
                {
                    env.entry(var).or_insert(ty);
                }
                // Promote already-defined caller variables whose storage cannot hold the
                // boxed/nullable value a by-reference parameter may write back.
                for (var, ty) in
                    self.method_call_by_ref_boxed_promotions(&object_type, method, &expanded_args, env)
                {
                    env.insert(var, ty);
                }
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
                self.infer_type(expr, env)
            }
            _ => self.infer_type(expr, env),
        }
    }

    /// Walks the always-evaluated index sub-expressions of an `isset`/`unset` operand for
    /// assignment effects and nested by-reference outputs, without inferring the lazy base
    /// chain itself.
    ///
    /// PHP always evaluates the offset expression of every `ArrayAccess` link in the operand —
    /// `isset($a[$i][$j = f()])` evaluates `$j = f()` even when `$a[$i]` does not exist
    /// (php-verified: PHP does not short-circuit a nested index on a missing outer key) — so an
    /// assignment or by-reference call inside an index must still define its target for code
    /// that runs after the `isset`/`unset`. `PropertyAccess`/`NullsafePropertyAccess` links are
    /// only recursed into (to reach further nested indices under them, e.g.
    /// `isset($this->arr[$k = f()])`); the property access itself is never passed to
    /// `infer_type_with_assignment_effects`, preserving the `__isset`/`__unset` property-magic
    /// skip this whole path exists for (regression: `isset($obj->undeclaredProp)` must stay
    /// clean, and an undefined base array/object must not be reported as undefined here either
    /// — both are php-verified as producing no error/warning).
    fn walk_isset_unset_operand_assignment_effects(
        &mut self,
        arg: &Expr,
        env: &mut TypeEnv,
    ) -> Result<(), CompileError> {
        match &arg.kind {
            ExprKind::ArrayAccess { array, index } => {
                self.infer_type_with_assignment_effects(index, env)?;
                self.walk_isset_unset_operand_assignment_effects(array, env)?;
            }
            ExprKind::PropertyAccess { object, .. }
            | ExprKind::NullsafePropertyAccess { object, .. }
            | ExprKind::DynamicPropertyAccess { object, .. }
            | ExprKind::NullsafeDynamicPropertyAccess { object, .. } => {
                self.walk_isset_unset_operand_assignment_effects(object, env)?;
            }
            _ => {}
        }
        Ok(())
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
}

/// Returns true when a first-class callable target is PHP `preg_replace_callback`.
fn callable_target_is_preg_replace_callback(target: &CallableTarget) -> bool {
    matches!(
        target,
        CallableTarget::Function(name) if php_symbol_key(name.as_str()) == "preg_replace_callback"
    )
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

/// Promotes a packed indexed-array local to an associative array when one of its elements is
/// removed via `unset($arr[$key])`.
///
/// PHP's `unset()` removes a key without renumbering the remaining elements, so the array can no
/// longer be a contiguous packed list (e.g. `unset([1,2,3][1])` leaves keys `0` and `2`). Re-typing
/// the local as `AssocArray<Int, T>` makes its literal build as a hash, so the element removal
/// lowers through `HashUnset`. Only plain `$var[$key]` targets on a currently-packed array are
/// affected; associative arrays, objects, and non-variable receivers are left unchanged.
fn promote_indexed_local_for_element_unset(arg: &Expr, env: &mut TypeEnv) {
    let ExprKind::ArrayAccess { array, .. } = &arg.kind else {
        return;
    };
    let ExprKind::Variable(name) = &array.kind else {
        return;
    };
    let Some(PhpType::Array(elem_ty)) = env.get(name).cloned() else {
        return;
    };
    env.insert(
        name.clone(),
        PhpType::AssocArray {
            key: Box::new(PhpType::Int),
            value: elem_ty,
        },
    );
}
