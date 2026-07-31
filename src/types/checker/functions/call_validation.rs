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
use crate::parser::ast::{CallableTarget, Expr, ExprKind};
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
        CallArgPlanError::SpreadAfterNamed { span } => CompileError::new(
            span,
            &format!(
                "{} cannot use argument unpacking after named arguments",
                callee_desc
            ),
        ),
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

/// Returns `true` if `ty` is `PhpType::Callable` or a `Union` containing `Callable` as a member
/// (e.g. a nullable `?callable` parameter). Used to scope the string-callable coercion in
/// `Checker::coerce_callable_string_args` to genuinely `callable`-typed parameter positions.
fn type_accepts_callable(ty: &PhpType) -> bool {
    match ty {
        PhpType::Callable => true,
        PhpType::Union(members) => members.iter().any(type_accepts_callable),
        _ => false,
    }
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

impl Checker {
    /// Returns true when an argument expression is an l-value supported by by-reference calls.
    ///
    /// A non-nullsafe, non-dynamic `$obj->prop` fetch (`ExprKind::PropertyAccess`) also
    /// qualifies — mirrors `super::is_by_ref_property_arg`, which the same call-validation
    /// paths already use to skip `require_boxed_by_ref_storage` for these args, and
    /// `crate::ir_lower::expr::lower_by_ref_property_arg_with_signature`, which already
    /// lowers this exact shape through a hidden-temp copy-in/copy-out (any by-ref parameter
    /// type, not just arrays — the property is never widened; the callee writes into the
    /// temp, and the temp is copied back into the property on the normal-return edge). Only
    /// the checker gate was missing this case, rejecting programs the rest of the pipeline
    /// already supports (e.g. `Helper::fill($this->refs)` for `fill(array &$refs)`).
    pub(crate) fn is_by_ref_argument_lvalue(
        &mut self,
        arg: &Expr,
        env: &TypeEnv,
    ) -> Result<bool, CompileError> {
        match &arg.kind {
            ExprKind::Variable(_) => Ok(true),
            ExprKind::PropertyAccess { .. } => Ok(true),
            ExprKind::ArrayAccess { array, .. } if matches!(array.kind, ExprKind::Variable(_)) => {
                Ok(matches!(
                    self.infer_type(array, env)?.codegen_repr(),
                    PhpType::Array(_)
                ))
            }
            _ => Ok(false),
        }
    }

    /// Normalizes arguments for a user-defined function call, allowing unknown named arguments
    /// to be collected into the variadic parameter.
    pub(crate) fn normalize_named_call_args(
        &self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        callee_desc: &str,
        env: &TypeEnv,
    ) -> Result<Vec<Expr>, CompileError> {
        self.normalize_call_args(sig, args, span, callee_desc, false, true, env)
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
        let (normalized_args, _) = self.normalize_call_args_with_default_mask(
            sig,
            args,
            span,
            callee_desc,
            trim_trailing_defaults,
            allow_unknown_named_variadic,
            env,
        )?;
        Ok(normalized_args)
    }

    /// Normalizes call arguments and records which regular parameter slots were synthesized from
    /// declaration defaults rather than supplied by the caller.
    #[allow(clippy::too_many_arguments)]
    fn normalize_call_args_with_default_mask(
        &self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        callee_desc: &str,
        trim_trailing_defaults: bool,
        allow_unknown_named_variadic: bool,
        env: &TypeEnv,
    ) -> Result<(Vec<Expr>, Vec<bool>), CompileError> {
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
        let default_regular_args = plan
            .regular_args
            .iter()
            .map(|arg| matches!(arg, call_args::PlannedRegularArg::Default(_)))
            .collect();
        let mut normalized_args = plan.normalized_args();
        // Only plain positional call sites (no named/spread arguments) are eligible: the real
        // AST-level rewrite in `crate::optimize::callable_coercion` (which runs post-check, since
        // this checker never mutates the real program AST) only handles that same shape — see its
        // module docs. Accepting a wider shape HERE than that pass can actually rewrite would
        // type-check cleanly but reach codegen with an uncoerced string literal in a
        // `callable`-typed ABI slot, which is a real, previously-hit segfault class, not a
        // hypothetical one. Both sides must stay in lockstep.
        if !args
            .iter()
            .any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. } | ExprKind::Spread(_)))
        {
            self.coerce_callable_string_args(sig, &mut normalized_args);
        }
        Ok((normalized_args, default_regular_args))
    }

    /// Coerces literal string-callable arguments at `Callable`-typed regular-parameter
    /// positions into their first-class-callable equivalent (`ExprKind::FirstClassCallable`)
    /// at COMPILE TIME, reusing the `funcname(...)` first-class-callable machinery so the
    /// coerced argument gets the exact same arity/support validation as writing `funcname(...)`
    /// directly at the call site (including the arity-hungry `func_get_args()` rejection —
    /// `infer_type` on the rewritten node runs `resolve_first_class_callable_sig`, which already
    /// enforces that).
    ///
    /// PHP accepts a bare function-name string (`'strtoupper'`) anywhere a `callable` is
    /// expected; elephc's `callable`-typed-parameter checking otherwise rejects a `string`
    /// argument outright ("expects Callable, got Str"). Scope for this cycle (JURY ADDENDUM #2
    /// on the N1 checker-bundle spec — explicit decision, documented, not silent):
    /// - Only a LITERAL string (`ExprKind::StringLiteral`) naming a KNOWN function — user-
    ///   declared (resolved or forward-declared), `extern`, or a builtin with a first-class-
    ///   callable signature — is coerced. A non-literal string (e.g. a variable holding a name)
    ///   is left alone: dynamic name resolution is out of scope, so the existing "expects
    ///   Callable, got Str" diagnostic still fires, honestly, instead of silently miscompiling.
    /// - `'Class::method'` static-method strings are NOT coerced this cycle: PHP's visibility
    ///   rules for a static-method callable string depend on the CALLING scope, and this seam
    ///   has no access to that context here — coercing without enforcing visibility would
    ///   silently under-check, which this project's campaign law forbids. Left loud (unchanged)
    ///   rather than risked.
    /// - `[$obj, 'method']` array-form callables are NOT coerced: unlike a bare string,
    ///   resolving this form needs the receiver's runtime type and a bound-method closure,
    ///   which does not drop out trivially from this seam. Left loud (unchanged).
    ///
    /// Called from `normalize_call_args`, the single choke point shared by every user-function,
    /// method, builtin, and extern call-argument normalization path (see
    /// `normalize_named_call_args`/`normalize_builtin_call_args` above), so this one seam covers
    /// every `callable`-typed parameter uniformly — including the shutdown-function prelude's
    /// `register_shutdown_function(callable $callback, ...)` — without a per-call-site opt-in.
    fn coerce_callable_string_args(&self, sig: &FunctionSig, args: &mut [Expr]) {
        let regular_param_count = call_args::regular_param_count(sig);
        for (idx, arg) in args.iter_mut().enumerate().take(regular_param_count) {
            let ExprKind::StringLiteral(name) = &arg.kind else {
                continue;
            };
            let is_callable_param = sig
                .params
                .get(idx)
                .is_some_and(|(_, ty)| type_accepts_callable(ty));
            if !is_callable_param {
                continue;
            }
            if let Some(canonical) = self.coercible_callable_string_target(name) {
                arg.kind = ExprKind::FirstClassCallable(CallableTarget::Function(
                    crate::names::Name::unqualified(canonical),
                ));
            }
        }
    }

    /// Resolves `name` (case-insensitively) to a function elephc's first-class-callable
    /// machinery already knows how to target: a user-declared function (resolved or forward-
    /// declared via `fn_decls`), an `extern` declaration, or a builtin with a
    /// `first_class_callable_builtin_sig`. Returns the canonical name to embed in the coerced
    /// `FirstClassCallable` node, or `None` if `name` does not resolve to anything usable — the
    /// caller then leaves the string argument untouched so the ordinary type-mismatch
    /// diagnostic still fires.
    ///
    /// Read-only: does not trigger signature resolution as a side effect (a forward-declared
    /// function is matched by name only here). Full resolution — and rejection of targets the
    /// first-class-callable ABI cannot dispatch, such as arity-hungry functions — happens when
    /// the coerced node is itself type-checked immediately afterward, exactly as it would for
    /// literal `name(...)` syntax (see `resolve_first_class_callable_sig`).
    fn coercible_callable_string_target(&self, name: &str) -> Option<String> {
        if let Some(canonical) = self.canonical_function_name_folded(name) {
            return Some(canonical);
        }
        if let Some(canonical) = self.canonical_extern_function_name_folded(name) {
            return Some(canonical);
        }
        if crate::name_resolver::is_builtin_function(name) {
            let canonical = crate::types::checker::builtins::canonical_builtin_function_name(name)?;
            if crate::types::first_class_callable_builtin_sig(&canonical).is_some() {
                return Some(canonical);
            }
        }
        None
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
            // The backend has explicit boxed-Mixed cast funnels for scalar boundaries.
            // Refcounted/object boundaries do not yet validate the runtime tag, so keep
            // those statically rejected instead of treating Mixed as universally safe.
            (PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::Str, PhpType::Mixed) => true,
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
        if Self::types_compatible(expected, actual)
            || self.type_accepts(expected, actual)
            || Self::array_family_gradual_accepts(expected, actual)
            || self.gradual_boundary_accepts(expected, actual)
        {
            Ok(())
        } else {
            Err(CompileError::new(
                span,
                &format!("{} expects {:?}, got {:?}", context, expected, actual),
            ))
        }
    }

    /// Like `require_compatible_arg_type`, but additionally admits the PHP weak-mode value-boundary
    /// coercions the call-argument codegen realizes (`weak_boundary_coercion_accepts`): a concrete
    /// scalar or Stringable object into a `string` parameter, and a scalar/Stringable/union into a
    /// boxed-`Mixed` union parameter — plus the base→derived checked downcast
    /// (`call_arg_downcast_guardable`). Used ONLY at call-argument positions whose codegen lowers
    /// through `lower_args_with_signature` (which coerces the argument, and emits the downcast
    /// guard, at the boundary); property and static-property stores keep using the strict
    /// `require_compatible_arg_type` because they emit neither.
    ///
    /// `param_is_declared` is the parameter's `FunctionSig::declared_params` bit. It gates ONLY
    /// the downcast fallback, and it must: for an UNDECLARED parameter `expected` is a type
    /// elephc INFERRED, and PHP performs no check there at all, so enforcing it at runtime would
    /// throw a `TypeError` on a program PHP runs (`function pick($x) { if ($x instanceof A) …}`
    /// infers `A` and then rejects every non-`A` caller). Lowering gates its emission on the same
    /// bit, so acceptance and enforcement stay in step.
    pub(crate) fn require_compatible_call_arg_type(
        &self,
        expected: &PhpType,
        actual: &PhpType,
        span: crate::span::Span,
        context: &str,
        param_is_declared: bool,
    ) -> Result<(), CompileError> {
        match self.require_compatible_arg_type(expected, actual, span, context) {
            Ok(()) => Ok(()),
            Err(err) => {
                if self.weak_boundary_coercion_accepts(expected, actual)
                    || (param_is_declared && self.call_arg_downcast_guardable(expected, actual))
                {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }

    /// Returns true when a call ARGUMENT may flow into `expected` behind the runtime checked
    /// downcast `crate::ir_lower::expr::coerce_operands_to_params` emits at this same boundary.
    ///
    /// Deliberately a thin delegation to the shared `checked_downcast_guardable` rather than a
    /// second hierarchy walk: the return boundary uses the same predicate, and a safety predicate
    /// with two implementations is two predicates whose permissiveness unions — the shape that
    /// already cost this codebase a double free once.
    pub(crate) fn call_arg_downcast_guardable(&self, expected: &PhpType, actual: &PhpType) -> bool {
        self.checked_downcast_guardable(
            expected,
            actual,
            crate::types::checked_downcast::GuardPosition::Argument,
        )
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

    /// Validates a user-defined callback while allowing it to ignore surplus positional values.
    ///
    /// PHP passes the callback surface's full argument list, but user functions and closures read
    /// only their declared parameters. Internal builtins keep their own strict arity validation.
    pub(crate) fn check_known_user_callback_call(
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
            true,
        )
    }

    /// Shared implementation for known callable call validation.
    fn check_known_callable_call_with_options(
        &mut self,
        sig: &FunctionSig,
        args: &[Expr],
        span: crate::span::Span,
        caller_env: &TypeEnv,
        callee_desc: &str,
        allow_by_ref_spread: bool,
        allow_extra_positional: bool,
    ) -> Result<PhpType, CompileError> {
        let (normalized_args, default_regular_args) = self.normalize_call_args_with_default_mask(
            sig,
            args,
            span,
            callee_desc,
            false,
            true,
            caller_env,
        )?;
        let args = normalized_args.as_slice();
        let effective_arg_count = args
            .iter()
            .filter(|a| !matches!(a.kind, ExprKind::Spread(_)))
            .count();
        let has_spread = args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)));
        let required = sig.defaults.iter().filter(|d| d.is_none()).count();
        // A callable-variable invocation (`$cb(...)`) is scoped precisely by the `callee_desc`
        // format string `"callable ${name}"` (see `infer_closure_call_type`/`infer_expr_call_type`
        // in `inference/ops.rs`). PHP never errors on extra positional args to a non-variadic
        // function/closure invoked through a variable — only "too few" is a real PHP error
        // (`ArgumentCountError`). The runtime invoker for this path forwards the full argument
        // vector and the callee reads only its declared params, so surplus args are ABI-safe.
        // Direct user-function/method calls are NOT covered by this prefix and keep the strict
        // upper bound below (elephc's direct-call ABI materializes exact params).
        let is_callable_var_invocation =
            allow_extra_positional || callee_desc.starts_with("callable $");

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
            } else if effective_arg_count < required
                || (!is_callable_var_invocation && effective_arg_count > sig.params.len())
            {
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

        let regular_param_count = if sig.variadic.is_some() {
            sig.params.len().saturating_sub(1)
        } else {
            sig.params.len()
        };
        let variadic_elem_ty = sig.variadic.as_ref().and_then(|_| {
            sig.params.last().and_then(|(_, ty)| match ty {
                PhpType::Array(elem) => Some((**elem).clone()),
                _ => None,
            })
        });

        let mut param_idx = 0usize;
        for arg in args {
            let actual_ty = self.infer_type(arg, caller_env)?;
            if matches!(arg.kind, ExprKind::Spread(_)) {
                continue;
            }
            if param_idx < regular_param_count {
                let uses_default = default_regular_args
                    .get(param_idx)
                    .copied()
                    .unwrap_or(false);
                if !uses_default
                    && sig.ref_params.get(param_idx).copied().unwrap_or(false)
                    && !self.is_by_ref_argument_lvalue(arg, caller_env)?
                {
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
                if let Some((param_name, expected_ty)) = sig.params.get(param_idx) {
                    if !uses_default
                        && sig.declared_params.get(param_idx).copied().unwrap_or(false)
                        && sig.ref_params.get(param_idx).copied().unwrap_or(false)
                        && !super::is_by_ref_property_arg(arg)
                    {
                        self.require_boxed_by_ref_storage(
                            expected_ty,
                            &actual_ty,
                            arg.span,
                            &format!("{} parameter ${}", callee_desc, param_name),
                        )?;
                    }
                    self.require_compatible_call_arg_type(
                        expected_ty,
                        &actual_ty,
                        arg.span,
                        &format!("{} parameter ${}", callee_desc, param_name),
                        sig.declared_params.get(param_idx).copied().unwrap_or(false),
                    )?;
                }
            } else if let (Some(vname), Some(expected_ty)) =
                (sig.variadic.as_ref(), variadic_elem_ty.as_ref())
            {
                self.require_compatible_arg_type(
                    expected_ty,
                    &actual_ty,
                    arg.span,
                    &format!("{} variadic parameter ${}", callee_desc, vname),
                )?;
            }
            param_idx += 1;
        }

        Ok(sig.return_type.clone())
    }
}
