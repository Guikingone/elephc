//! Purpose:
//! Validates function call validation semantics for the checker.
//! Keeps call diagnostics and return-flow analysis consistent with signatures and inferred expression types.
//!
//! Called from:
//! - `crate::types::checker::functions`
//!
//! Key details:
//! - Diagnostics should map shared planner errors back to source spans without duplicating call semantics.

use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::call_args::{self, CallArgPlanError};
use crate::types::{FunctionSig, PhpType, TypeEnv};

use super::super::Checker;

/// Maps a `CallArgPlanError` from the shared call-argument planner to a typed `CompileError`
/// with a human-readable message that references the callee description and parameter names.
fn call_arg_plan_error(
    sig: &FunctionSig,
    callee_desc: &str,
    err: CallArgPlanError,
) -> CompileError {
    match err {
        CallArgPlanError::UnknownNamed { span, name } => {
            CompileError::new(span, &format!("{} has no parameter ${}", callee_desc, name))
        }
        CallArgPlanError::Duplicate {
            span,
            param_idx,
            name,
        } => {
            let param_name = sig
                .params
                .get(param_idx)
                .map(|(name, _)| name.as_str())
                .unwrap_or(name.as_str());
            CompileError::new(
                span,
                &format!(
                    "{} parameter ${} is already assigned",
                    callee_desc, param_name
                ),
            )
        }
        CallArgPlanError::PositionalAfterNamed { span } => CompileError::new(
            span,
            &format!(
                "{} cannot use positional arguments after named arguments",
                callee_desc
            ),
        ),
        CallArgPlanError::PositionalAfterSpread { span } => CompileError::new(
            span,
            &format!(
                "{} cannot use positional arguments after spread arguments",
                callee_desc
            ),
        ),
        CallArgPlanError::SpreadAfterNamed { span } => {
            spread_after_named_error(span, callee_desc)
        }
        CallArgPlanError::MissingRequired { span, param_idx } => {
            let param_name = sig
                .params
                .get(param_idx)
                .map(|(name, _)| name.as_str())
                .unwrap_or("arg");
            CompileError::new(
                span,
                &format!("{} missing required parameter ${}", callee_desc, param_name),
            )
        }
    }
}

/// Builds the diagnostic for PHP's compile-time
/// "Cannot use argument unpacking after named arguments" fatal, prefixed with
/// the callee description used by the rest of the call diagnostics.
fn spread_after_named_error(span: crate::span::Span, callee_desc: &str) -> CompileError {
    CompileError::new(
        span,
        &format!(
            "{} cannot use argument unpacking after named arguments",
            callee_desc
        ),
    )
}

/// Returns a boolean vector indicating which argument positions contain assoc-spread sources
/// (arrays with string keys that map to named arguments after spread expansion).
fn assoc_spread_sources(args: &[Expr], env: &TypeEnv) -> Vec<bool> {
    call_args::expand_static_assoc_spread_args(args)
        .iter()
        .map(|arg| match &arg.kind {
            ExprKind::Spread(inner) => is_assoc_spread_source(inner, env),
            _ => false,
        })
        .collect()
}

/// Returns true if the expression is or expands to an assoc-array at runtime,
/// which means spread arguments from it should be treated as named arguments.
fn is_assoc_spread_source(expr: &Expr, env: &TypeEnv) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => matches!(env.get(name), Some(PhpType::AssocArray { .. })),
        ExprKind::ArrayLiteralAssoc(_) => true,
        _ => matches!(
            crate::types::checker::infer_expr_type_syntactic(expr),
            PhpType::AssocArray { .. }
        ),
    }
}

/// Returns whether an expression denotes writable storage accepted by a by-reference call.
///
/// Named arguments delegate to their value. Container elements remain writable when their
/// base is itself a stable local, property, static property, or container place. Nullsafe and
/// computed-property reads are excluded because they do not guarantee one stable write target.
fn is_writable_call_place(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Variable(_) | ExprKind::StaticPropertyAccess { .. } => true,
        ExprKind::PropertyAccess { object, .. } => is_stable_place_receiver(object),
        ExprKind::ArrayAccess { array, .. } => is_stable_place_receiver(array),
        ExprKind::NamedArg { value, .. } => is_writable_call_place(value),
        _ => false,
    }
}

/// Returns whether a receiver chain can be read and written without changing its root.
fn is_stable_place_receiver(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Variable(_) | ExprKind::This | ExprKind::StaticPropertyAccess { .. } => true,
        ExprKind::PropertyAccess { object, .. } => is_stable_place_receiver(object),
        ExprKind::ArrayAccess { array, .. } => is_stable_place_receiver(array),
        _ => false,
    }
}

/// Returns one flag per regular parameter indicating that the shared planner
/// filled the slot from its declaration default rather than a caller expression.
fn planned_omitted_default_slots(
    sig: &FunctionSig,
    args: &[Expr],
    span: crate::span::Span,
    callee_desc: &str,
    env: &TypeEnv,
) -> Result<Vec<bool>, CompileError> {
    let assoc_spreads = assoc_spread_sources(args, env);
    let allow_unknown_named_variadic = !crate::func_args::sig_collects_surplus_args(sig);
    let plan = call_args::plan_call_args_with_regular_param_count_and_assoc_spreads(
        sig,
        args,
        span,
        call_args::regular_param_count(sig),
        false,
        allow_unknown_named_variadic,
        &assoc_spreads,
    )
    .map_err(|error| call_arg_plan_error(sig, callee_desc, error))?;
    Ok(plan
        .regular_args
        .iter()
        .map(|arg| matches!(arg, call_args::PlannedRegularArg::Default(_)))
        .collect())
}

/// Returns whether a normalized slot still carries the exact declaration
/// default inserted by an earlier planner pass.
fn is_injected_declared_default(
    sig: &FunctionSig,
    param_idx: usize,
    arg: &Expr,
) -> bool {
    sig.defaults
        .get(param_idx)
        .and_then(|default| default.as_ref())
        .is_some_and(|default| default.span == arg.span && default == arg)
}

impl Checker {
    /// Enforces PHP's syntactic "no argument unpacking after named arguments"
    /// rule on a call surface whose callee is not resolvable at compile time
    /// (string callables, `new $class(...)`), where no signature is available to
    /// run the shared planner against.
    ///
    /// The rule itself stays in `crate::types::call_args`; this only maps its
    /// error onto the same diagnostic the planner-backed surfaces produce.
    pub(crate) fn require_no_spread_after_named_args(
        &self,
        args: &[Expr],
        callee_desc: &str,
    ) -> Result<(), CompileError> {
        call_args::validate_no_spread_after_named(args).map_err(|err| match err {
            CallArgPlanError::SpreadAfterNamed { span } => {
                spread_after_named_error(span, callee_desc)
            }
            // The rule only reports `SpreadAfterNamed`; the remaining variants
            // need a signature to plan against and cannot originate here. Keep
            // the match exhaustive so a future rule still reports a real span.
            CallArgPlanError::UnknownNamed { span, .. }
            | CallArgPlanError::Duplicate { span, .. }
            | CallArgPlanError::PositionalAfterNamed { span }
            | CallArgPlanError::PositionalAfterSpread { span }
            | CallArgPlanError::MissingRequired { span, .. } => {
                CompileError::new(span, &format!("{} has invalid arguments", callee_desc))
            }
        })
    }

    /// Returns true when an argument expression is a writable PHP l-value.
    pub(crate) fn is_by_ref_argument_lvalue(
        &mut self,
        arg: &Expr,
        _env: &TypeEnv,
    ) -> Result<bool, CompileError> {
        Ok(is_writable_call_place(arg))
    }

    /// Returns whether an argument can be bound to a BUILTIN's by-reference parameter.
    ///
    /// Deliberately separate from `is_by_ref_argument_lvalue`, which answers the same question
    /// for a USER function and must stay narrower: that path writes its result back to a LOCAL
    /// SLOT (`RefArgWriteback::source_slot`), and a property has no slot, so widening the shared
    /// predicate would let the checker accept what the backend cannot lower — the exact
    /// "checker accepts, backend refuses" trade this codebase treats as a false win.
    ///
    /// Builtins reach their by-reference argument through the storage itself, which is why
    /// `array_push($this->items, 9)` already compiles and runs today. What PHP refuses, and
    /// what this rejects, is an argument with NO storage to write back to: a literal, an array
    /// literal, or a call result.
    pub(crate) fn is_builtin_by_ref_argument_lvalue(&self, arg: &Expr) -> bool {
        matches!(
            arg.kind,
            ExprKind::Variable(_)
                | ExprKind::ArrayAccess { .. }
                | ExprKind::PropertyAccess { .. }
                | ExprKind::DynamicPropertyAccess { .. }
                | ExprKind::NullsafePropertyAccess { .. }
                | ExprKind::NullsafeDynamicPropertyAccess { .. }
                | ExprKind::StaticPropertyAccess { .. }
        )
    }

    /// Normalizes arguments for a user-defined function call, allowing unknown named arguments
    /// to be collected into the variadic parameter.
    ///
    /// The one exception is the hidden variadic added by `crate::func_args` to collect the
    /// surplus positional arguments `func_get_args()` exposes: the callee declares no
    /// variadic of its own, so PHP still rejects an unknown named argument there.
    pub(crate) fn normalize_named_call_args(
        &self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        callee_desc: &str,
        env: &TypeEnv,
    ) -> Result<Vec<Expr>, CompileError> {
        let allow_unknown_named_variadic = !crate::func_args::sig_collects_surplus_args(sig);
        self.normalize_call_args(
            sig,
            args,
            span,
            callee_desc,
            false,
            allow_unknown_named_variadic,
            env,
        )
    }

    /// Normalizes arguments for a builtin or extern function call, rejecting unknown named
    /// arguments and not allowing unknown named arguments to flow into the variadic parameter.
    pub(crate) fn normalize_builtin_call_args(
        &self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        callee_desc: &str,
        env: &TypeEnv,
    ) -> Result<Vec<Expr>, CompileError> {
        self.normalize_call_args(sig, args, span, callee_desc, true, false, env)
    }

    /// Shared argument normalization for both user-defined and builtin calls. Delegates to the
    /// shared call-argument planner and converts planner errors to `CompileError`.
    fn normalize_call_args(
        &self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        callee_desc: &str,
        trim_trailing_defaults: bool,
        allow_unknown_named_variadic: bool,
        env: &TypeEnv,
    ) -> Result<Vec<Expr>, CompileError> {
        let assoc_spread_sources = assoc_spread_sources(args, env);
        let plan = call_args::plan_call_args_with_regular_param_count_and_assoc_spreads(
            sig,
            args,
            span,
            call_args::regular_param_count(sig),
            trim_trailing_defaults,
            allow_unknown_named_variadic,
            &assoc_spread_sources,
        )
        .map_err(|err| call_arg_plan_error(sig, callee_desc, err))?;
        Ok(plan.normalized_args())
    }

    /// Returns true if `expected` and `actual` are compatible according to PHP assignment
    /// coercion rules (e.g., int is compatible with float, float is compatible with int/bool/void,
    /// `Mixed` is compatible with everything, `Never` is compatible with everything).
    pub(crate) fn types_compatible(expected: &PhpType, actual: &PhpType) -> bool {
        if expected == actual {
            return true;
        }
        match (expected, actual) {
            (PhpType::Mixed, _) => true,
            (_, PhpType::Never) => true, // never is the bottom type — compatible with any expected type
            (PhpType::Bool, PhpType::False) => true,
            // The backend has explicit boxed-gradual cast funnels for scalar boundaries. Most
            // unions use the same boxed representation as `Mixed`; compact nullable integers use
            // `TaggedScalar` instead and deliberately remain excluded until their ABI conversion
            // is explicit.
            (
                PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::Str,
                actual @ (PhpType::Mixed | PhpType::Union(_)),
            ) if actual.codegen_repr() == PhpType::Mixed => true,
            // Boxed unions keep the same representation. Concrete heap boundaries below use
            // shared EIR narrowing with runtime checks before their payload is exposed.
            (PhpType::Union(_), PhpType::Mixed) => true,
            (
                PhpType::Object(_) | PhpType::Callable,
                actual @ (PhpType::Mixed | PhpType::Union(_)),
            ) if actual.codegen_repr() == PhpType::Mixed => true,
            (PhpType::Iterable, actual @ (PhpType::Mixed | PhpType::Union(_)))
                if actual.codegen_repr() == PhpType::Mixed => true,
            (PhpType::Array(element), actual @ (PhpType::Mixed | PhpType::Union(_)))
                if element.codegen_repr() == PhpType::Mixed
                    && actual.codegen_repr() == PhpType::Mixed => true,
            (
                PhpType::AssocArray { key, value },
                actual @ (PhpType::Mixed | PhpType::Union(_)),
            )
                if key.codegen_repr() == PhpType::Mixed
                    && value.codegen_repr() == PhpType::Mixed
                    && actual.codegen_repr() == PhpType::Mixed => true,
            (PhpType::Iterable, PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable) => true,
            (PhpType::Union(expected_members), PhpType::Union(actual_members)) => actual_members
                .iter()
                .all(|actual_member| {
                    expected_members.iter().any(|expected_member| {
                        Self::types_compatible(expected_member, actual_member)
                    })
                }),
            (PhpType::Union(members), _) => members
                .iter()
                .any(|member| Self::types_compatible(member, actual)),
            (
                PhpType::AssocArray { key, value },
                PhpType::Array(_) | PhpType::AssocArray { .. },
            ) if **key == PhpType::Mixed && **value == PhpType::Mixed => true,
            (PhpType::Float, PhpType::Int | PhpType::Bool | PhpType::False | PhpType::Void) => true,
            (PhpType::Int, PhpType::Bool | PhpType::False | PhpType::Void) => true,
            (PhpType::Bool, PhpType::Int | PhpType::Void) => true,
            (PhpType::Pointer(_), PhpType::Pointer(_) | PhpType::Void) => true,
            (PhpType::Resource(_), PhpType::Resource(_)) => {
                PhpType::resource_types_compatible(expected, actual)
            }
            (PhpType::Callable, PhpType::Callable) => true,
            _ => false,
        }
    }

    /// Returns `Ok(())` if `actual` is compatible with `expected` (via `types_compatible` or
    /// `type_accepts`), otherwise returns a `CompileError` describing the type mismatch.
    pub(crate) fn require_compatible_arg_type(
        &self,
        expected: &PhpType,
        actual: &PhpType,
        span: crate::span::Span,
        context: &str,
    ) -> Result<(), CompileError> {
        if Self::types_compatible(expected, actual) || self.type_accepts(expected, actual) {
            Ok(())
        } else {
            Err(CompileError::new(
                span,
                &format!("{} expects {:?}, got {:?}", context, expected, actual),
            ))
        }
    }

    /// Formats a parameter-count range as a human-readable string, e.g. `3` or `2 to 5`.
    pub(crate) fn format_fixed_or_range_arity(min_args: usize, max_args: usize) -> String {
        if min_args == max_args {
            format!("{}", min_args)
        } else {
            format!("{} to {}", min_args, max_args)
        }
    }

    /// Validates and type-checks a call to a known callable (user function or builtin with
    /// a known signature): arity constraints, named/spread arguments, ref-param requirements,
    /// and argument-type compatibility. Returns the signature's return type on success.
    pub(crate) fn check_known_callable_call(
        &mut self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        caller_env: &TypeEnv,
        callee_desc: &str,
    ) -> Result<PhpType, CompileError> {
        self.check_known_callable_call_with_options(
            sig,
            args,
            span,
            caller_env,
            callee_desc,
            false,
            false,
        )
    }

    /// Validates a direct call to a method or constructor of `owner_class`, applying PHP's
    /// coercive parameter binding when that class is declared in user source.
    ///
    /// Only surfaces whose arguments reach EIR through `lower_args_with_signature` may opt in:
    /// that is where the matching argument rewrite runs, and accepting a binding without it
    /// would hand raw storage to a differently typed parameter slot. Compiler-injected classes
    /// (SPL, `Exception`, reflection, …) lower several of their members through bespoke
    /// emitters instead, so they stay on the strict path.
    pub(crate) fn check_user_declared_call(
        &mut self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        caller_env: &TypeEnv,
        callee_desc: &str,
        owner_class: &str,
    ) -> Result<PhpType, CompileError> {
        let coercive = self.class_supports_coercive_param_binding(owner_class);
        self.check_known_callable_call_with_options(
            sig,
            args,
            span,
            caller_env,
            callee_desc,
            false,
            coercive,
        )
    }

    /// `check_user_declared_call` for a callee that also accepts spread arguments into
    /// by-reference parameters materialized by descriptor invokers.
    pub(crate) fn check_user_declared_call_allowing_by_ref_spread(
        &mut self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        caller_env: &TypeEnv,
        callee_desc: &str,
        owner_class: &str,
    ) -> Result<PhpType, CompileError> {
        let coercive = self.class_supports_coercive_param_binding(owner_class);
        self.check_known_callable_call_with_options(
            sig,
            args,
            span,
            caller_env,
            callee_desc,
            true,
            coercive,
        )
    }

    /// Returns true when `class_name` is a class-like symbol declared in user source.
    ///
    /// Compiler-injected classes carry `Span::dummy()` as their declaration span, which is the
    /// only marker distinguishing them from user declarations. An unknown name is treated as
    /// not user-declared so an unresolved receiver never silently gains coercive binding.
    fn class_is_user_declared(&self, class_name: &str) -> bool {
        self.classes
            .get(class_name)
            .is_some_and(|info| info.declaration_span != crate::span::Span::dummy())
    }

    /// Returns whether ordinary EIR argument lowering can apply PHP scalar parameter coercions.
    fn class_supports_coercive_param_binding(&self, class_name: &str) -> bool {
        self.class_is_user_declared(class_name)
            || matches!(
                class_name.trim_start_matches('\\'),
                "DateTime" | "DateTimeImmutable"
            )
    }

    /// Validates a known callable call while allowing spread arguments for by-reference
    /// parameters that will be materialized by descriptor invokers at runtime.
    pub(crate) fn check_known_callable_call_allowing_by_ref_spread(
        &mut self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        caller_env: &TypeEnv,
        callee_desc: &str,
    ) -> Result<PhpType, CompileError> {
        self.check_known_callable_call_with_options(
            sig,
            args,
            span,
            caller_env,
            callee_desc,
            true,
            false,
        )
    }

    /// Shared implementation for known callable call validation.
    ///
    /// `coercive_param_binding` opts the callee into PHP's coercive parameter binding for its
    /// declared parameters; see `check_user_declared_call` for when that is sound.
    fn check_known_callable_call_with_options(
        &mut self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        caller_env: &TypeEnv,
        callee_desc: &str,
        allow_by_ref_spread: bool,
        coercive_param_binding: bool,
    ) -> Result<PhpType, CompileError> {
        let omitted_default_slots =
            planned_omitted_default_slots(sig, args, span, callee_desc, caller_env)?;
        let normalized_args = self.normalize_named_call_args(sig, args, span, callee_desc, caller_env)?;
        let args = normalized_args.as_slice();
        let effective_arg_count = args
            .iter()
            .filter(|a| !matches!(a.kind, ExprKind::Spread(_)))
            .count();
        let has_spread = args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)));
        let regular_param_count = if sig.variadic.is_some() {
            sig.params.len().saturating_sub(1)
        } else {
            sig.params.len()
        };
        // The variadic collector is represented as the signature's final param
        // with no default expression, but it never contributes to minimum arity.
        let required = sig
            .defaults
            .iter()
            .take(regular_param_count)
            .filter(|default| default.is_none())
            .count();

        if sig.ref_params.iter().any(|is_ref| *is_ref) && has_spread && !allow_by_ref_spread {
            return Err(CompileError::new(
                span,
                &format!(
                    "{} cannot be invoked with spread arguments when it has pass-by-reference parameters",
                    callee_desc
                ),
            ));
        }

        if !has_spread {
            if sig.variadic.is_some() {
                if effective_arg_count < required {
                    return Err(CompileError::new(
                        span,
                        &format!(
                            "{} expects at least {} arguments, got {}",
                            callee_desc, required, effective_arg_count
                        ),
                    ));
                }
            } else if effective_arg_count < required || effective_arg_count > sig.params.len() {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "{} expects {} arguments, got {}",
                        callee_desc,
                        Self::format_fixed_or_range_arity(required, sig.params.len()),
                        effective_arg_count
                    ),
                ));
            }
        }

        let variadic_elem_ty = sig.variadic.as_ref().and_then(|_| {
            sig.params.last().and_then(|(_, ty)| match ty {
                PhpType::Array(elem) => Some((**elem).clone()),
                _ => None,
            })
        });

        let mut param_idx = 0usize;
        for arg in args {
            let actual_ty = self.infer_call_argument_type(arg, caller_env)?;
            if matches!(arg.kind, ExprKind::Spread(_)) {
                continue;
            }
            if param_idx < regular_param_count {
                let omitted_default = omitted_default_slots
                    .get(param_idx)
                    .copied()
                    .unwrap_or(false)
                    || is_injected_declared_default(sig, param_idx, arg);
                if sig.ref_params.get(param_idx).copied().unwrap_or(false) {
                    // The callee holds a reference to this local from here on, and it can
                    // escape, so the local is never kill/retype eligible in this body.
                    self.record_reference_alias_root(arg);
                    // An omitted trailing default the planner materialized is not a caller
                    // expression at all, so the l-value shape has nothing to say about it.
                    if !omitted_default && !self.is_by_ref_argument_lvalue(arg, caller_env)? {
                        let param_name = sig
                            .params
                            .get(param_idx)
                            .map(|(name, _)| name.as_str())
                            .unwrap_or("arg");
                        return Err(CompileError::new(
                            arg.span,
                            &format!(
                                "{} parameter ${} must be passed a variable",
                                callee_desc, param_name
                            ),
                        ));
                    }
                }
                if let Some((param_name, expected_ty)) = sig.params.get(param_idx) {
                    if sig.declared_params.get(param_idx).copied().unwrap_or(false)
                        && sig.ref_params.get(param_idx).copied().unwrap_or(false)
                        && !omitted_default
                    {
                        self.require_boxed_by_ref_storage(
                            expected_ty,
                            &actual_ty,
                            arg.span,
                            &format!("{} parameter ${}", callee_desc, param_name),
                        )?;
                    }
                    // `strict_types` applies to every declared parameter type, including the
                    // closure and first-class-callable surfaces that stay off the coercive
                    // path. Builtin signatures carry `declared_params: false` throughout
                    // (`crate::builtins::registry`), so this never fires for an internal
                    // function whose parameter types the checker does not consume.
                    if sig.declared_params.get(param_idx).copied().unwrap_or(false) {
                        self.require_strict_types_param_binding(
                            expected_ty,
                            &actual_ty,
                            arg.span,
                            &format!("{} parameter ${}", callee_desc, param_name),
                        )?;
                    }
                    if coercive_param_binding
                        && sig.declared_params.get(param_idx).copied().unwrap_or(false)
                    {
                        self.require_bound_param_arg_type(
                            expected_ty,
                            &actual_ty,
                            arg,
                            caller_env,
                            &format!("{} parameter ${}", callee_desc, param_name),
                            None,
                            sig.ref_params.get(param_idx).copied().unwrap_or(false),
                        )?;
                    } else {
                        let declared_value_param = sig
                            .declared_params
                            .get(param_idx)
                            .copied()
                            .unwrap_or(false)
                            && !sig.ref_params.get(param_idx).copied().unwrap_or(false);
                        let has_runtime_guard = crate::types::param_binding::object_requires_runtime_nominal_guard(
                            expected_ty,
                            &actual_ty,
                        ) || crate::types::param_binding::nullable_int_requires_runtime_param_guard(
                            expected_ty,
                            &actual_ty,
                        );
                        if !declared_value_param || !has_runtime_guard
                        {
                            self.require_compatible_arg_type(
                                expected_ty,
                                &actual_ty,
                                arg.span,
                                &format!("{} parameter ${}", callee_desc, param_name),
                            )?;
                        }
                    }
                }
            } else {
                // An argument collected by a by-REFERENCE variadic (`&...$xs`) is bound by
                // reference exactly like a regular by-ref parameter's, so the local it names is
                // aliased for the rest of the body. The variadic's flag sits at
                // `regular_param_count` in `ref_params` (it is the signature's last slot).
                // Recorded outside the element-type check below because that one only runs when
                // the element type is a known array, which has nothing to do with aliasing.
                let variadic_by_ref = sig
                    .ref_params
                    .get(regular_param_count)
                    .copied()
                    .unwrap_or(false);
                if variadic_by_ref {
                    self.record_reference_alias_root(arg);
                }
                if let Some(vname) = sig.variadic.as_ref() {
                    if variadic_by_ref && !self.is_by_ref_argument_lvalue(arg, caller_env)? {
                        return Err(CompileError::new(
                            arg.span,
                            &format!(
                                "{} variadic parameter ${} must be passed a variable",
                                callee_desc, vname
                            ),
                        ));
                    }
                    let Some(expected_ty) = variadic_elem_ty.as_ref() else {
                        param_idx += 1;
                        continue;
                    };
                    // The variadic occupies the last `declared_params` slot, so gating on it keeps
                    // the strict rejection off builtin variadics, whose registry-derived parameter
                    // types the checker does not otherwise consume.
                    let declared_variadic = sig.declared_params.last().copied().unwrap_or(false);
                    if coercive_param_binding && declared_variadic {
                        self.require_bound_param_arg_type(
                            expected_ty,
                            &actual_ty,
                            arg,
                            caller_env,
                            &format!("{} variadic parameter ${}", callee_desc, vname),
                            None,
                            variadic_by_ref,
                        )?;
                    } else {
                        if declared_variadic {
                            self.require_strict_types_param_binding(
                                expected_ty,
                                &actual_ty,
                                arg.span,
                                &format!("{} variadic parameter ${}", callee_desc, vname),
                            )?;
                        }
                        self.require_compatible_arg_type(
                            expected_ty,
                            &actual_ty,
                            arg.span,
                            &format!("{} variadic parameter ${}", callee_desc, vname),
                        )?;
                    }
                }
            }
            param_idx += 1;
        }

        Ok(sig.return_type.clone())
    }
}
