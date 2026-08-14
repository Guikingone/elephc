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

use super::super::super::null_probe;
use super::super::super::Checker;
use super::{merge_match_arm_result_type, merge_null_coalesce_result_type};

impl Checker {
    /// Installs assignment effects that are guaranteed when a short-circuit condition is true.
    ///
    /// The ordinary expression pass keeps the right side of `&&` isolated because it may never
    /// execute. Inside the true branch, however, every right side in the successful `&&` chain
    /// has executed. This pass exposes only those branch-local effects and returns the previous
    /// bindings so the statement checker can restore the fallthrough environment afterwards.
    pub(crate) fn install_truthy_short_circuit_effects(
        &mut self,
        condition: &Expr,
        env: &mut TypeEnv,
    ) -> Result<Vec<(String, Option<PhpType>)>, CompileError> {
        self.install_short_circuit_outcome_effects(condition, true, env)
    }

    /// Installs effects and narrowing facts guaranteed when a short-circuit condition is false.
    ///
    /// Every operand of a false `||` chain has executed and evaluated false. This is used after a
    /// terminal `if` body, where reaching the following statement proves the condition was false.
    pub(crate) fn install_falsy_short_circuit_effects(
        &mut self,
        condition: &Expr,
        env: &mut TypeEnv,
    ) -> Result<Vec<(String, Option<PhpType>)>, CompileError> {
        self.install_short_circuit_outcome_effects(condition, false, env)
    }

    /// Installs effects and branch facts guaranteed by one short-circuit outcome.
    fn install_short_circuit_outcome_effects(
        &mut self,
        condition: &Expr,
        truthy: bool,
        env: &mut TypeEnv,
    ) -> Result<Vec<(String, Option<PhpType>)>, CompileError> {
        let before = env.clone();
        self.infer_short_circuit_outcome_effects(condition, truthy, false, env)?;
        let changed = env
            .iter()
            .filter_map(|(name, ty)| {
                let previous = before.get(name);
                (previous != Some(ty)).then(|| (name.clone(), previous.cloned()))
            })
            .collect();
        Ok(changed)
    }

    /// Infers operands and narrowing facts that must hold for one short-circuit outcome.
    fn infer_short_circuit_outcome_effects(
        &mut self,
        condition: &Expr,
        truthy: bool,
        nested: bool,
        env: &mut TypeEnv,
    ) -> Result<(), CompileError> {
        match &condition.kind {
            ExprKind::BinaryOp {
                left,
                op: BinOp::And,
                right,
            } if truthy => {
                self.infer_short_circuit_outcome_effects(left, true, true, env)?;
                self.infer_type_with_assignment_effects(right, env)?;
                self.infer_short_circuit_outcome_effects(right, true, true, env)
            }
            ExprKind::BinaryOp {
                left,
                op: BinOp::Or,
                right,
            } if !truthy => {
                self.infer_short_circuit_outcome_effects(left, false, true, env)?;
                self.infer_type_with_assignment_effects(right, env)?;
                self.infer_short_circuit_outcome_effects(right, false, true, env)
            }
            ExprKind::FunctionCall { name, args }
                if truthy && php_symbol_key(name.trim_start_matches('\\')) == "isset" =>
            {
                self.install_truthy_isset_root_facts(args, env);
                Ok(())
            }
            _ => {
                if nested {
                    let Some(narrowing) = self.guard_narrowing(condition, env)? else {
                        return Ok(());
                    };
                    env.insert(
                        narrowing.var,
                        if truthy {
                            narrowing.then_ty
                        } else {
                            narrowing.else_ty
                        },
                    );
                }
                Ok(())
            }
        }
    }

    /// Binds roots that must exist when a successful `isset()` condition is true.
    ///
    /// A chained probe such as `isset($value['key'])` may be the first syntactic read of a local
    /// that was created along only one earlier short-circuit path. The true branch proves that
    /// the root is initialized, while its exact runtime shape remains gradual, so `Mixed` is the
    /// sound branch-local storage fact. The caller restores the previous binding after the branch.
    fn install_truthy_isset_root_facts(&self, args: &[Expr], env: &mut TypeEnv) {
        let roots = args
            .iter()
            .filter_map(|arg| {
                null_probe::undefined_probe_root_variable(arg, env).map(str::to_string)
            })
            .collect::<Vec<_>>();
        for root in roots {
            env.insert(root, PhpType::Mixed);
        }
    }

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
                // `int` can overflow to float and `string` can become int/float
                // (`"9"++` is `int(10)`), so the local is dynamically typed afterwards.
                if matches!(old_ty, Some(PhpType::Int) | Some(PhpType::Str)) {
                    env.insert(name.clone(), PhpType::Mixed);
                }
                Ok(result_ty)
            }
            ExprKind::PostIncrement(name) | ExprKind::PostDecrement(name) => {
                let old_ty = env.get(name).cloned();
                let result_ty = self.infer_type(expr, env)?;
                // Same retype as the pre-form: only the RESULT differs, and it was already
                // computed above against the type the local held before the update.
                if matches!(old_ty, Some(PhpType::Int) | Some(PhpType::Str)) {
                    env.insert(name.clone(), PhpType::Mixed);
                }
                Ok(result_ty)
            }
            ExprKind::BinaryOp { left, op, right } => {
                self.infer_type_with_assignment_effects(left, env)?;
                if matches!(op, BinOp::And | BinOp::Or) {
                    let mut right_env = env.clone();
                    if *op == BinOp::And {
                        self.install_truthy_short_circuit_effects(left, &mut right_env)?;
                    } else {
                        self.install_falsy_short_circuit_effects(left, &mut right_env)?;
                    }
                    self.infer_type_with_assignment_effects(right, &mut right_env)?;
                    merge_conditional_storage_effects(env, &right_env);
                    Ok(PhpType::Bool)
                } else {
                    self.infer_type_with_assignment_effects(right, env)?;
                    self.infer_type(expr, env)
                }
            }
            ExprKind::NullCoalesce { value, default } => {
                // `$neverDefined ?? $d` is a null probe: PHP answers `$d` without an
                // undefined-variable warning, so the left chain root reads as `null`.
                let probe = null_probe::begin_null_probe_root(self, value, env);
                let value_ty = self.infer_null_probe_operand_with_effects(value, env);
                null_probe::end_null_probe_root(probe, env);
                let value_ty = value_ty?;
                let default_ty = if value_ty == PhpType::Void {
                    self.infer_type_with_assignment_effects(default, env)?
                } else {
                    let mut default_env = env.clone();
                    let default_ty =
                        self.infer_type_with_assignment_effects(default, &mut default_env)?;
                    merge_conditional_storage_effects(env, &default_env);
                    default_ty
                };
                let non_null_value = if Self::union_contains_void(&value_ty) {
                    self.strip_void_from_union(&value_ty)
                } else {
                    value_ty
                };
                Ok(merge_null_coalesce_result_type(
                    non_null_value,
                    default_ty,
                ))
            }
            ExprKind::ShortTernary { value, default } => {
                let value_ty = self.infer_type_with_assignment_effects(value, env)?;
                if value_ty == PhpType::Void {
                    self.infer_type_with_assignment_effects(default, env)?;
                } else {
                    let mut default_env = env.clone();
                    self.infer_type_with_assignment_effects(default, &mut default_env)?;
                    merge_conditional_storage_effects(env, &default_env);
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
                self.install_truthy_short_circuit_effects(condition, &mut then_env)?;
                self.install_falsy_short_circuit_effects(condition, &mut else_env)?;
                if let Some(guard) = guard {
                    then_env.insert(guard.var.clone(), guard.then_ty);
                    else_env.insert(guard.var, guard.else_ty);
                }
                let then_ty = self.infer_type_with_assignment_effects(then_expr, &mut then_env)?;
                merge_conditional_storage_effects(env, &then_env);
                merge_conditional_storage_effects(&mut else_env, &then_env);
                let else_ty = self.infer_type_with_assignment_effects(else_expr, &mut else_env)?;
                merge_conditional_storage_effects(env, &else_env);
                Ok(merge_match_arm_result_type(self, then_ty, else_ty))
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
                let mut fallthrough_env = env.clone();
                let mut result_ty: Option<PhpType> = None;
                for (conditions, result) in arms {
                    let mut arm_env = fallthrough_env.clone();
                    for condition in conditions {
                        self.infer_type_with_assignment_effects(condition, &mut arm_env)?;
                    }
                    fallthrough_env = arm_env.clone();
                    let mut result_env =
                        self.match_arm_narrowed_env(subject, conditions, &arm_env)?;
                    let ty = self.infer_type_with_assignment_effects(result, &mut result_env)?;
                    result_ty = Some(match result_ty {
                        Some(acc) => merge_match_arm_result_type(self, acc, ty),
                        None => ty,
                    });
                    merge_conditional_storage_effects(env, &result_env);
                }
                if let Some(default) = default {
                    let mut default_env = fallthrough_env;
                    let ty = self.infer_type_with_assignment_effects(default, &mut default_env)?;
                    result_ty = Some(match result_ty {
                        Some(acc) => merge_match_arm_result_type(self, acc, ty),
                        None => ty,
                    });
                    merge_conditional_storage_effects(env, &default_env);
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
            ExprKind::InstanceOf { value, target } => {
                self.infer_type_with_assignment_effects(value, env)?;
                if let crate::parser::ast::InstanceOfTarget::Expr(target) = target {
                    self.infer_type_with_assignment_effects(target, env)?;
                }
                self.infer_type(expr, env)
            }
            ExprKind::Closure { capture_refs, .. } => {
                // Capturing an undefined local by reference creates the caller-visible null cell
                // before the closure value is constructed. Use gradual storage because the
                // closure can replace that cell with any PHP value when it executes.
                for name in capture_refs {
                    env.entry(name.clone()).or_insert(PhpType::Mixed);
                }
                self.infer_type(expr, env)
            }
            ExprKind::FunctionCall { name, args } => {
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                let builtin_name = name.trim_start_matches('\\');
                if let Some((signature, internal_callable)) =
                    self.direct_function_call_signature(builtin_name)
                {
                    self.prepare_by_ref_variable_storage(
                        &signature,
                        args,
                        expr.span,
                        env,
                        internal_callable,
                    )?;
                } else if self.direct_call_is_late_bound_undefined(builtin_name) {
                    self.prepare_late_bound_call_argument_storage(args, env);
                }
                // `isset`/`unset` are lazy language constructs: an operand may be
                // an undeclared property routed to `__isset`/`__unset`, which must
                // not be inferred as a bare property access here. The call's own
                // inference handles the operands (with magic routing).
                if matches!(
                    php_symbol_key(builtin_name).as_str(),
                    "isset" | "unset"
                ) {
                    // `isset($never)` / `unset($never)` are exactly the constructs PHP provides
                    // for probing storage that may never have been declared, so a never-declared
                    // chain root reads as `null` for the operand.
                    for arg in &expanded_args {
                        let probe = null_probe::begin_null_probe_root(self, arg, env);
                        let effects = self.infer_non_reading_arg_assignment_effects(arg, env);
                        null_probe::end_null_probe_root(probe, env);
                        effects?;
                    }
                } else if !builtin_name.eq_ignore_ascii_case("unset") {
                    // `empty()` shares that tolerance, but its operand is still read (PHP
                    // consults `__isset` then `__get`), so it stays on the eager path with only
                    // the probe binding added.
                    let is_empty = php_symbol_key(builtin_name) == "empty";
                    // An array-callback builtin types its callback's unannotated parameters
                    // from the array element/key inside `check_builtin`. Skip that argument
                    // here: the eager pass would otherwise check the closure body against the
                    // unhinted parameter fallback and reject valid PHP such as
                    // `array_filter($strings, fn($v) => strlen($v) > 5)`.
                    let contextual_callbacks =
                        crate::types::checker::builtins::contextual_callback_arg_positions(
                            builtin_name,
                        );
                    for (idx, arg) in expanded_args.iter().enumerate() {
                        if contextual_callbacks.contains(&idx) {
                            continue;
                        }
                        if (builtin_name.eq_ignore_ascii_case("preg_match") && idx == 2)
                            || (builtin_name.eq_ignore_ascii_case("openssl_encrypt")
                                && is_openssl_encrypt_tag_arg(arg, idx))
                        {
                            continue;
                        }
                        if is_empty {
                            let probe = null_probe::begin_null_probe_root(self, arg, env);
                            let effects = self.infer_null_probe_operand_with_effects(arg, env);
                            null_probe::end_null_probe_root(probe, env);
                            effects?;
                            continue;
                        }
                        self.infer_call_argument_with_assignment_effects(arg, env)?;
                    }
                }
                let ty = self.infer_type(expr, env)?;
                // The callee may mutate any reachable object; drop property narrowings. (The
                // call's own argument checking above still saw them.)
                Self::purge_property_narrowings(env);
                if builtin_name.eq_ignore_ascii_case("preg_match") {
                    if let Some(arg) = expanded_args.get(2) {
                        if let Some(name) = output_variable(arg) {
                            env.insert(name.clone(), PhpType::Array(Box::new(PhpType::Str)));
                        }
                    }
                }
                if builtin_name.eq_ignore_ascii_case("openssl_encrypt") {
                    if let Some(arg) = openssl_encrypt_tag_arg(&expanded_args) {
                        if let Some(name) = output_variable(arg) {
                            env.insert(name.clone(), PhpType::Str);
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
            ExprKind::NewObject { args, .. } => {
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                for arg in &expanded_args {
                    self.infer_call_argument_with_assignment_effects(arg, env)?;
                }
                let ty = self.infer_type(expr, env)?;
                Self::purge_property_narrowings(env);
                Ok(ty)
            }
            ExprKind::StaticMethodCall {
                receiver,
                method,
                args,
            } => {
                let target = CallableTarget::StaticMethod {
                    receiver: receiver.clone(),
                    method: method.clone(),
                };
                let signature = self
                    .resolve_first_class_callable_sig(&target, expr.span, env)
                    .ok()
                    .or_else(|| self.static_call_effect_signature(receiver, method));
                if let Some(signature) = signature {
                    self.prepare_by_ref_variable_storage(
                        &signature,
                        args,
                        expr.span,
                        env,
                        false,
                    )?;
                }
                let closure_bind = matches!(
                    receiver,
                    crate::parser::ast::StaticReceiver::Named(name)
                        if name.as_str().trim_start_matches('\\') == "Closure"
                ) && php_symbol_key(method) == "bind";
                if !closure_bind {
                    let expanded_args =
                        crate::types::call_args::expand_static_assoc_spread_args(args);
                    for arg in &expanded_args {
                        self.infer_call_argument_with_assignment_effects(arg, env)?;
                    }
                }
                // `Closure::bind` owns inference of its closure operand so it can install the
                // literal visibility scope before checking the body. Eager inference here would
                // check that body in its lexical scope first and reject valid private access.
                let ty = self.infer_type(expr, env)?;
                Self::purge_property_narrowings(env);
                Ok(ty)
            }
            ExprKind::ClosureCall { var, args } => {
                if let Some(signature) = self.callable_sigs.get(var).cloned() {
                    self.prepare_by_ref_variable_storage(
                        &signature,
                        args,
                        expr.span,
                        env,
                        false,
                    )?;
                }
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                let skip_contextual_callback =
                    self.variable_targets_preg_replace_callback(var.as_str());
                for (idx, arg) in expanded_args.iter().enumerate() {
                    if skip_contextual_callback && idx == 1 {
                        continue;
                    }
                    self.infer_call_argument_with_assignment_effects(arg, env)?;
                }
                let ty = self.infer_type(expr, env)?;
                Self::purge_property_narrowings(env);
                Ok(ty)
            }
            ExprKind::ExprCall { callee, args } => {
                self.infer_type_with_assignment_effects(callee, env)?;
                if let Some(signature) = self.resolve_expr_callable_sig(callee, env)? {
                    self.prepare_by_ref_variable_storage(
                        &signature,
                        args,
                        expr.span,
                        env,
                        false,
                    )?;
                }
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                let skip_contextual_callback = self
                    .expr_targets_preg_replace_callback(callee);
                for (idx, arg) in expanded_args.iter().enumerate() {
                    if skip_contextual_callback && idx == 1 {
                        continue;
                    }
                    self.infer_call_argument_with_assignment_effects(arg, env)?;
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
                let target = CallableTarget::Method {
                    object: object.clone(),
                    method: method.clone(),
                };
                let signature = self
                    .resolve_first_class_callable_sig(&target, expr.span, env)
                    .ok()
                    .or_else(|| self.instance_call_effect_signature(&object_type, method));
                if let Some(signature) = signature {
                    self.prepare_by_ref_variable_storage(
                        &signature,
                        args,
                        expr.span,
                        env,
                        false,
                    )?;
                }
                let expanded_args = crate::types::call_args::expand_static_assoc_spread_args(args);
                for arg in &expanded_args {
                    self.infer_call_argument_with_assignment_effects(arg, env)?;
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
                    self.infer_call_argument_with_assignment_effects(arg, env)?;
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
                    self.infer_call_argument_with_assignment_effects(arg, env)?;
                }
                let ty = self.infer_type(expr, env)?;
                Self::purge_property_narrowings(env);
                Ok(ty)
            }
            _ => self.infer_type(expr, env),
        }
    }

    /// Infers one call argument while preserving assignment effects in a gradual spread source.
    fn infer_call_argument_with_assignment_effects(
        &mut self,
        arg: &Expr,
        env: &mut TypeEnv,
    ) -> Result<PhpType, CompileError> {
        let ExprKind::Spread(inner) = &arg.kind else {
            return self.infer_type_with_assignment_effects(arg, env);
        };
        match self.infer_type_with_assignment_effects(inner, env)? {
            PhpType::Array(elem_ty) => Ok(*elem_ty),
            PhpType::AssocArray { value, .. } => Ok(*value),
            PhpType::Mixed | PhpType::Union(_) => Ok(PhpType::Mixed),
            _ => Err(CompileError::new(
                arg.span,
                "Spread operator requires an array",
            )),
        }
    }

    /// Returns the statically known signature for a direct named call and whether it uses
    /// internal-function named-argument rules.
    pub(crate) fn direct_function_call_signature(
        &self,
        name: &str,
    ) -> Option<(crate::types::FunctionSig, bool)> {
        let key = php_symbol_key(name);
        if !crate::types::checker::builtins::strict_php_hidden_builtin(&key) {
            if let Some(signature) = crate::types::builtin_call_sig(&key) {
                return Some((signature, true));
            }
        }
        let canonical = self
            .canonical_function_name_folded(name)
            .unwrap_or_else(|| name.to_string());
        self.functions
            .get(&canonical)
            .cloned()
            .or_else(|| self.unresolved_direct_function_effect_signature(&canonical))
            .map(|signature| (signature, false))
    }

    /// Builds the caller-visible reference contract for a declared function not yet resolved.
    ///
    /// Call-effect inference runs before ordinary call inference. A forward call can therefore
    /// encounter only its parsed declaration, but its by-reference arguments still need their
    /// post-call storage before later statements are checked. This lightweight signature keeps
    /// declared parameter types authoritative without resolving or checking the function body.
    fn unresolved_direct_function_effect_signature(
        &self,
        canonical: &str,
    ) -> Option<crate::types::FunctionSig> {
        let declaration = self.fn_decls.get(canonical)?;
        let params = declaration
            .params
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let ty = declaration
                    .param_types
                    .get(index)
                    .and_then(|type_expr| type_expr.as_ref())
                    .and_then(|type_expr| {
                        self.resolve_declared_param_type_hint(
                            type_expr,
                            declaration.span,
                            "Function parameter",
                        )
                        .ok()
                    })
                    .unwrap_or(PhpType::Int);
                (name.clone(), ty)
            })
            .chain(declaration.variadic.iter().map(|name| {
                let element = declaration
                    .variadic_type
                    .as_ref()
                    .and_then(|type_expr| {
                        self.resolve_declared_param_type_hint(
                            type_expr,
                            declaration.span,
                            "Variadic function parameter",
                        )
                        .ok()
                    })
                    .unwrap_or(if declaration.variadic_by_ref {
                        PhpType::Mixed
                    } else {
                        PhpType::Int
                    });
                (name.clone(), PhpType::Array(Box::new(element)))
            }))
            .collect();
        Some(crate::types::FunctionSig {
            params,
            param_type_exprs: declaration
                .param_types
                .iter()
                .cloned()
                .chain(
                    declaration
                        .variadic
                        .iter()
                        .map(|_| declaration.variadic_type.clone()),
                )
                .collect(),
            param_attributes: declaration.param_attributes.clone(),
            defaults: declaration.defaults.clone(),
            return_type: PhpType::Int,
            declared_return: declaration.return_type.is_some(),
            by_ref_return: declaration.by_ref_return,
            ref_params: declaration.ref_params.clone(),
            declared_params: declaration
                .param_types
                .iter()
                .map(|type_expr| type_expr.is_some())
                .chain(
                    declaration
                        .variadic
                        .iter()
                        .map(|_| declaration.variadic_type.is_some()),
                )
                .collect(),
            variadic: declaration.variadic.clone(),
            deprecation: None,
        })
    }

    /// Returns whether a direct call has no compile-time declaration and will use runtime lookup.
    fn direct_call_is_late_bound_undefined(&self, name: &str) -> bool {
        let canonical = self
            .canonical_function_name_folded(name)
            .unwrap_or_else(|| name.to_string());
        !self.functions.contains_key(&canonical)
            && !self.fn_decls.contains_key(&canonical)
            && !self.function_variant_groups.contains_key(&canonical)
            && self.canonical_extern_function_name_folded(name).is_none()
            && crate::types::checker::builtins::is_late_bound_undefined_function(&canonical)
    }

    /// Provides gradual recovery storage for undefined variables passed to an unresolved call.
    ///
    /// PHP resolves a direct function before evaluating its arguments, so the compiler's current
    /// late-bound path throws before any such variable is read. Binding the names as `Mixed` keeps
    /// later unreachable statements typeable without pretending the unknown runtime signature is
    /// by-value or by-reference. Calls with any known signature stay on the precise path above.
    fn prepare_late_bound_call_argument_storage(&self, args: &[Expr], env: &mut TypeEnv) {
        let names = args
            .iter()
            .filter_map(by_ref_output_variable)
            .filter(|name| !env.contains_key(name.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        for name in names {
            env.insert(name, PhpType::Mixed);
        }
    }

    /// Resolves an instance method signature for call effects without callable-syntax policy.
    fn instance_call_effect_signature(
        &self,
        object_type: &PhpType,
        method: &str,
    ) -> Option<crate::types::FunctionSig> {
        let class_name = match object_type.codegen_repr() {
            PhpType::Object(class_name) => class_name,
            _ => return None,
        };
        let key = php_symbol_key(method);
        self.classes
            .get(class_name.trim_start_matches('\\'))?
            .methods
            .get(&key)
            .cloned()
    }

    /// Resolves a static method signature for call effects without callable-syntax policy.
    fn static_call_effect_signature(
        &self,
        receiver: &crate::parser::ast::StaticReceiver,
        method: &str,
    ) -> Option<crate::types::FunctionSig> {
        let class_name = match receiver {
            crate::parser::ast::StaticReceiver::Named(name) => name.as_str().to_string(),
            crate::parser::ast::StaticReceiver::Self_
            | crate::parser::ast::StaticReceiver::Static => self.current_class.clone()?,
            crate::parser::ast::StaticReceiver::Parent => self
                .classes
                .get(self.current_class.as_ref()?)?
                .parent
                .clone()?,
        };
        let info = self.classes.get(class_name.trim_start_matches('\\'))?;
        let key = php_symbol_key(method);
        info.static_methods
            .get(&key)
            .or_else(|| info.methods.get(&key))
            .cloned()
    }

    /// Prepares caller storage for variables supplied to by-reference parameters.
    ///
    /// PHP treats an undefined variable at an untyped reference boundary as a writable null
    /// cell. The compiler uses boxed `Mixed` storage for that cell so the callee can replace it
    /// with any PHP value. Declared reference parameters instead see a null value and retain
    /// ordinary parameter validation, which rejects null when the declaration does not accept it.
    fn prepare_by_ref_variable_storage(
        &mut self,
        signature: &crate::types::FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        env: &mut TypeEnv,
        internal_callable: bool,
    ) -> Result<(), CompileError> {
        let normalized = if internal_callable {
            self.normalize_builtin_call_args(signature, args, span, "call", env)?
        } else {
            self.normalize_named_call_args(signature, args, span, "call", env)?
        };
        let regular_param_count = crate::types::call_args::regular_param_count(signature);
        let mut param_index = 0usize;
        for argument in &normalized {
            if matches!(argument.kind, ExprKind::Spread(_)) {
                continue;
            }
            if param_index >= regular_param_count {
                break;
            }
            if signature
                .ref_params
                .get(param_index)
                .copied()
                .unwrap_or(false)
            {
                if let Some(name) = by_ref_output_variable(argument) {
                    let declared = signature
                        .declared_params
                        .get(param_index)
                        .copied()
                        .unwrap_or(false);
                    let expected = signature
                        .params
                        .get(param_index)
                        .map(|(_, ty)| ty)
                        .unwrap_or(&PhpType::Mixed);
                    let current = env.get(name).cloned();
                    let storage = if internal_callable {
                        if *expected == PhpType::Mixed {
                            current.clone().or(Some(PhpType::Mixed))
                        } else {
                            Some(expected.clone())
                        }
                    } else if declared
                        && matches!(
                            expected,
                            PhpType::Array(_) | PhpType::AssocArray { .. }
                        )
                        && current.as_ref().is_some_and(|current| {
                            matches!(
                                current,
                                PhpType::Array(_) | PhpType::AssocArray { .. }
                            )
                        })
                    {
                        // A declared `array` reference may replace elements, keys, or the whole
                        // array while retaining the same PHP-visible contract. The caller's
                        // pre-call shape is therefore no longer sound after the call; expose the
                        // generic declared shape while preserving the shared array ABI storage.
                        Some(expected.clone())
                    } else if declared
                        && current.as_ref().is_some_and(|current| {
                            current.codegen_repr() != expected.codegen_repr()
                                && (matches!(
                                    current.codegen_repr(),
                                    PhpType::Mixed | PhpType::Union(_)
                                ) || matches!(
                                    expected.codegen_repr(),
                                    PhpType::Mixed | PhpType::Union(_) | PhpType::TaggedScalar
                                ))
                        })
                    {
                        // A declared reference parameter validates and updates the variable in
                        // place. Keep a gradual caller boxed for a checked concrete adapter;
                        // otherwise adopt the parameter's nullable/gradual representation so
                        // every value the callee may write fits the caller's frame slot.
                        let required_storage = if current.as_ref().is_some_and(|current| {
                            matches!(current.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
                        }) {
                            PhpType::Mixed
                        } else {
                            expected.clone()
                        };
                        self.by_ref_local_storage_types.insert(
                            (self.current_loop_storage_scope.clone(), name.clone()),
                            required_storage,
                        );
                        Some(expected.clone())
                    } else if !declared || *expected == PhpType::Mixed {
                        Some(PhpType::Mixed)
                    } else if let Some(current) = current.clone() {
                        if current != PhpType::Mixed
                            && expected.codegen_repr() == PhpType::Mixed
                            && (Self::types_compatible(expected, &current)
                                || self.type_accepts(expected, &current))
                        {
                            Some(expected.clone())
                        } else {
                            Some(current)
                        }
                    } else if Self::types_compatible(expected, &PhpType::Void)
                        || self.type_accepts(expected, &PhpType::Void)
                    {
                        Some(expected.clone())
                    } else {
                        Some(PhpType::Void)
                    };
                    if let Some(storage) = storage {
                        if !internal_callable
                            && storage.codegen_repr() == PhpType::Mixed
                            && current
                                .as_ref()
                                .is_some_and(|current| current.codegen_repr() != PhpType::Mixed)
                        {
                            self.by_ref_local_storage_types.insert(
                                (self.current_loop_storage_scope.clone(), name.clone()),
                                storage.clone(),
                            );
                        }
                        env.insert(name.clone(), storage);
                    }
                }
            }
            param_index += 1;
        }
        Ok(())
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

/// Merges storage effects from an expression arm that may not execute into its outer environment.
///
/// Array layout changes retain their common converted representation. Any other conditional
/// creation, removal, or retyping loses its precise fact and becomes `Mixed`; synthetic property
/// narrowing keys are omitted because they describe flow facts rather than runtime bindings.
fn merge_conditional_storage_effects(env: &mut TypeEnv, branch_env: &TypeEnv) {
    let mut names = env
        .keys()
        .chain(branch_env.keys())
        .cloned()
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();

    for name in names {
        let outer_ty = env.get(&name).cloned();
        let branch_ty = branch_env.get(&name).cloned();
        if outer_ty == branch_ty {
            continue;
        }
        if name.starts_with('\u{1}') {
            env.remove(&name);
            continue;
        }
        if let (Some(outer_ty), Some(branch_ty)) = (&outer_ty, &branch_ty) {
            if let Some(converted) =
                crate::types::array_storage_conversion(Some(outer_ty), branch_ty)
            {
                env.insert(name, converted);
                continue;
            }
        }
        env.insert(name, PhpType::Mixed);
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

/// Returns the plain caller variable targeted by a positional or named by-reference output.
fn by_ref_output_variable(arg: &Expr) -> Option<&String> {
    match &arg.kind {
        ExprKind::Variable(name) => Some(name),
        ExprKind::NamedArg { value, .. } => by_ref_output_variable(value),
        _ => None,
    }
}

/// Returns the variable name used by a builtin output argument.
fn output_variable(arg: &Expr) -> Option<&String> {
    match &arg.kind {
        ExprKind::Variable(name) => Some(name),
        ExprKind::NamedArg { value, .. } => output_variable(value),
        _ => None,
    }
}

/// Returns true when one source-order argument is OpenSSL encrypt's by-reference tag.
fn is_openssl_encrypt_tag_arg(arg: &Expr, index: usize) -> bool {
    match &arg.kind {
        ExprKind::NamedArg { name, .. } => php_symbol_key(name) == "tag",
        _ => index == 5,
    }
}

/// Finds OpenSSL encrypt's tag argument across positional and reordered named calls.
fn openssl_encrypt_tag_arg(args: &[Expr]) -> Option<&Expr> {
    args.iter()
        .find(|arg| {
            matches!(
                &arg.kind,
                ExprKind::NamedArg { name, .. } if php_symbol_key(name) == "tag"
            )
        })
        .or_else(|| {
            args.get(5)
                .filter(|arg| !matches!(arg.kind, ExprKind::NamedArg { .. }))
        })
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
