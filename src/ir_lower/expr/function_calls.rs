//! Purpose:
//! Direct function and builtin call lowering.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers a direct function, builtin, or extern call.
pub(super) fn lower_function_call(ctx: &mut LoweringContext<'_, '_>, name: &Name, args: &[Expr], expr: &Expr) -> LoweredValue {
    constants::register_static_define_call(ctx, name, args);
    if let Some(value) = constants::lower_static_defined_call(ctx, name, args, expr) {
        return value;
    }
    if let Some(value) = constants::lower_static_constant_call(ctx, name, args, expr) {
        return value;
    }
    let canonical = name.as_str();
    if canonical == crate::names::DYNAMIC_INCLUDE_FUNCTION {
        let [path, once, required] = args else {
            panic!("checked internal dynamic include must carry three operands");
        };
        let ExprKind::BoolLiteral(once) = &once.kind else {
            panic!("checked internal dynamic include once flag must be a boolean literal");
        };
        let ExprKind::BoolLiteral(required) = &required.kind else {
            panic!("checked internal dynamic include required flag must be a boolean literal");
        };
        let path = lower_expr(ctx, path);
        let call = ctx.emit_value(
            Op::RuntimeCall,
            vec![path.value],
            Some(Immediate::RuntimeCall(crate::ir::RuntimeCallTarget::DynamicInclude {
                once: *once,
                required: *required,
                strict_php: crate::strict_php::is_enabled(),
            })),
            PhpType::Mixed,
            effects_lookup::runtime_effects(),
            Some(expr.span),
        );
        release_owned_call_arg_temporaries(
            ctx,
            &[path.value],
            Some(call.value),
            &ReturnArgAlias::None,
            expr.span,
        );
        ctx.mark_eval_executed();
        ctx.apply_eval_barrier();
        return call;
    }
    if let Some(value) = lower_lazy_isset(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_lazy_empty(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_desugared_dynamic_method_call(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_static_call_user_func(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_dynamic_call_user_func(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_dynamic_call_user_func_array(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_literal_pattern_array_preg_replace_callback(
        ctx, canonical, args, expr,
    ) {
        return value;
    }
    // A call whose by-reference array argument is a property, static property, or container
    // element is rewritten to `$tmp = <place>; f($tmp, ...); <place> = $tmp;` before argument
    // materialization, so copy-on-write separation is stored back into the caller's place.
    if let Some(value) = ref_place_args::lower_ref_place_function_call(ctx, name, args, expr) {
        return value;
    }
    if let Some(value) = lower_gradual_local_krsort(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_static_array_map(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) =
        compat_preludes::lower_single_arg_assert(ctx, canonical, args, expr)
    {
        return value;
    }
    if let Some(value) =
        compat_preludes::lower_default_initial_array_reduce(ctx, canonical, args, expr)
    {
        return value;
    }
    if let Some(value) =
        compat_preludes::lower_backend_gap_builtin_shape(ctx, canonical, args, expr)
    {
        return value;
    }
    if let Some(value) = lower_static_array_reduce(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_static_array_walk(ctx, canonical, args, expr) {
        return value;
    }
    if php_symbol_key(canonical.trim_start_matches('\\')) == "unset" {
        if let Some(value) = lower_unset_locals(ctx, args, expr) {
            return value;
        }
    }
    if let Some(value) = lower_static_settype(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_static_array_push(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) =
        compat_preludes::lower_gradual_array_merge(ctx, canonical, args, expr)
    {
        return value;
    }
    if let Some(value) = lower_array_internal_pointer(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_static_is_callable(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_eval_function_probe(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_eval_class_probe(ctx, canonical, args, expr) {
        return value;
    }
    if let Some(value) = lower_static_filter_var(ctx, canonical, args, expr) {
        return value;
    }
    let extension_builtin = source_prefers_extension_builtin(canonical);
    let sig = call_signature(ctx, canonical, extension_builtin);
    let is_extern = ctx.extern_functions.contains_key(canonical);
    let is_user_function = ctx.functions.contains_key(canonical) && !extension_builtin;
    let is_builtin =
        crate::types::checker::builtins::is_supported_builtin_function(canonical);
    if !is_extern && !is_user_function && !is_builtin {
        // A web handler can load a PHP file dynamically during request bootstrap. Such a file may
        // declare a function that a separately lowered AOT method calls later in the same request,
        // so preserve the call for the eval function-owner bridge instead of freezing an undefined
        // function Error before that file has executed. Non-web programs retain the eager PHP
        // undefined-call path unless their own frame already crossed an eval barrier.
        if !ctx.can_resolve_runtime_dynamic_functions() && !ctx.has_eval_barrier() {
            if let Some(value) =
                super::late_bound_call::lower_late_bound_undefined_call(ctx, canonical, expr)
            {
                return value;
            }
        }
    }
    if (ctx.has_eval_barrier() || ctx.can_resolve_runtime_dynamic_functions())
        && dynamic_function_owner_fallback_candidate(canonical)
        && crate::builtins::registry::lookup(canonical).is_none()
    {
        if let Some(value) = lower_dynamic_function_owner_spread_call(ctx, canonical, args, expr) {
            return value;
        }
    }
    let operands = if is_extern || is_user_function {
        lower_args_with_signature(ctx, sig.as_ref(), args)
    } else {
        lower_builtin_call_args(ctx, canonical, sig.as_ref(), args)
    };
    let php_type = if is_extern || is_user_function {
        call_return_type(ctx, canonical, &operands)
    } else if let Some(php_type) =
        registry_builtin_result_type(ctx, canonical, args, &operands, expr.span)
    {
        php_type
    } else {
        call_return_type(ctx, canonical, &operands)
    };
    if is_extern {
        let data = ctx.intern_function_name(canonical);
        let call = ctx.emit_value(
            Op::ExternCall,
            operands.clone(),
            Some(Immediate::Data(data)),
            php_type,
            Op::ExternCall.default_effects(),
            Some(expr.span),
        );
        // Plain extern calls release owned argument temporaries the same way method
        // and builtin calls do, so a fresh owned temporary passed as an argument is
        // not leaked once per call. The alias guard keeps a pass-through result alive.
        release_owned_call_arg_temporaries(
            ctx,
            &operands,
            Some(call.value),
            &ReturnArgAlias::Unknown,
            expr.span,
        );
        return call;
    }
    if is_user_function {
        let data = ctx.intern_function_name(canonical);
        let call = ctx.emit_value(
            Op::Call,
            operands.clone(),
            Some(Immediate::Data(data)),
            php_type,
            effects_lookup::user_call_effects(canonical),
            Some(expr.span),
        );
        // Plain user calls release owned argument temporaries the same way method and
        // builtin calls do. The alias guard keeps a passthrough result (e.g. a function
        // that returns its own array argument typed `iterable`) from being freed.
        let return_alias = ctx
            .return_alias_summaries
            .function(canonical)
            .cloned()
            .unwrap_or(ReturnArgAlias::Unknown);
        release_owned_call_arg_temporaries_with_signature(
            ctx,
            &operands,
            Some(call.value),
            &return_alias,
            sig.as_ref(),
            expr.span,
        );
        return call;
    }
    if (ctx.has_eval_barrier() || ctx.can_resolve_runtime_dynamic_functions())
        && plain_positional_call_args(args)
        && dynamic_function_owner_fallback_candidate(canonical)
        && crate::builtins::registry::lookup(canonical).is_none()
    {
        let dynamic_name = php_symbol_key(canonical.trim_start_matches('\\'));
        let data = ctx.intern_function_name(&dynamic_name);
        return ctx.emit_value(
            Op::EvalFunctionCall,
            operands,
            Some(Immediate::Data(data)),
            PhpType::Mixed,
            Op::EvalFunctionCall.default_effects(),
            Some(expr.span),
        );
    }
    let eval_literal = eval_literal_fragment(canonical, args);
    emit_builtin_call_value(ctx, canonical, operands, php_type, expr.span, eval_literal)
}

/// Returns whether a name can resolve through a function declared by runtime include/eval.
///
/// A checker-recognized PHP function is not necessarily backed by a native EIR implementation:
/// compatibility declarations such as `trigger_deprecation()` may instead arrive from an included
/// PHP polyfill. Keep true language constructs on their dedicated lowering paths, but let every
/// other unresolved/registry-free name consult the request-local dynamic function owner first.
fn dynamic_function_owner_fallback_candidate(name: &str) -> bool {
    !matches!(
        php_symbol_key(name.trim_start_matches('\\')).as_str(),
        "eval"
            | "empty"
            | "unset"
            | "isset"
            | "exit"
            | "die"
            | "filter_var$default"
            | "filter_var$default_nof"
            | "filter_var$int"
            | "filter_var$int_nof"
            | "filter_var$int_range"
            | "filter_var$int_range_nof"
            | "filter_var$float"
            | "filter_var$float_nof"
            | "filter_var$bool"
            | "filter_var$bool_nof"
            | "filter_var$ip"
            | "filter_var$ip_nof"
            | "filter_var$ip4"
            | "filter_var$ip4_nof"
            | "filter_var$ip6"
            | "filter_var$ip6_nof"
    )
}

/// Lowers a single dynamic positional spread through Magician's function-array ABI.
///
/// `$function(...$args)` preserves the source array's positional/named keys, so it cannot use the
/// fixed native argument ABI. A runtime-declared function receives the original boxed container
/// and applies PHP's call-argument rules inside the eval owner context.
fn lower_dynamic_function_owner_spread_call(
    ctx: &mut LoweringContext<'_, '_>,
    canonical: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    let [Expr {
        kind: ExprKind::Spread(container),
        ..
    }] = args
    else {
        return None;
    };
    let dynamic_name = php_symbol_key(canonical.trim_start_matches('\\'));
    let data = ctx.intern_function_name(&dynamic_name);
    let container = lower_expr(ctx, container);
    let container = super::callable_probes::coerce_eval_function_arg_array(ctx, container, expr.span);
    Some(ctx.emit_value(
        Op::EvalFunctionCallArray,
        vec![container.value],
        Some(Immediate::Data(data)),
        PhpType::Mixed,
        Op::EvalFunctionCallArray.default_effects(),
        Some(expr.span),
    ))
}

/// Promotes a boxed gradual local before descending key sort and republishes it afterwards.
///
/// A descending key order cannot live in packed storage. `MixedToHash` produces the mutable hash
/// passed to the runtime sort; after the call, the same SSA value contains any COW-adjusted hash
/// pointer and is boxed back into the local's original gradual slot. This also lets the enclosing
/// ref-place adapter write a valid boxed value back to a property or nested element.
fn lower_gradual_local_krsort(
    ctx: &mut LoweringContext<'_, '_>,
    canonical: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(canonical.trim_start_matches('\\')) != "krsort" || args.len() != 1 {
        return None;
    }
    let receiver = match &args[0].kind {
        ExprKind::Variable(name) => Some((name.as_str(), args[0].span)),
        ExprKind::NamedArg { name, value } if name == "array" => match &value.kind {
            ExprKind::Variable(local) => Some((local.as_str(), value.span)),
            _ => None,
        },
        _ => None,
    }?;
    let (name, span) = receiver;
    if !ctx.has_local_slot(name)
        || !matches!(
            ctx.local_type(name).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_)
        )
    {
        return None;
    }

    let source = ctx.load_local(name, Some(span));
    let hash_type = PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(PhpType::Mixed),
    };
    let hash = ctx.emit_value(
        Op::MixedToHash,
        vec![source.value],
        None,
        hash_type,
        Op::MixedToHash.default_effects(),
        Some(span),
    );
    if ctx.value_is_owning_temporary(source) {
        crate::ir_lower::ownership::release_if_owned(ctx, source, Some(span));
    }

    let definition = crate::builtins::registry::lookup("krsort")?;
    let lowered = crate::builtins::semantics::lower_registry_call(
        ctx,
        definition,
        &[hash.value],
        &PhpType::Bool,
        expr.span,
    )
    .ok()?;
    let call = LoweredValue {
        value: lowered.value,
        ir_type: ctx.builder.value_type(lowered.value),
    };
    ctx.store_local(name, hash, PhpType::Mixed, Some(span));
    Some(call)
}

/// Expands a literal string-pattern array into sequential callback replacements.
///
/// PHP applies array patterns in source order while reusing the same callback and evolving
/// subject. Hoisting both values into synthetic locals preserves single evaluation and callback
/// identity. The optional replacement count is deliberately left to the general runtime path,
/// because its total must accumulate across patterns rather than expose the last call's count.
fn lower_literal_pattern_array_preg_replace_callback(
    ctx: &mut LoweringContext<'_, '_>,
    canonical: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(canonical.trim_start_matches('\\')) != "preg_replace_callback"
        || !(3..=4).contains(&args.len())
        || crate::types::call_args::has_named_args(args)
        || args.iter().any(is_spread_arg)
    {
        return None;
    }
    let patterns = match &args[0].kind {
        ExprKind::ArrayLiteral(patterns)
            if patterns
                .iter()
                .all(|pattern| matches!(pattern.kind, ExprKind::StringLiteral(_))) =>
        {
            patterns.iter().collect::<Vec<_>>()
        }
        ExprKind::ArrayLiteralAssoc(patterns)
            if patterns
                .iter()
                .all(|(_, pattern)| matches!(pattern.kind, ExprKind::StringLiteral(_))) =>
        {
            patterns.iter().map(|(_, pattern)| pattern).collect::<Vec<_>>()
        }
        _ => return None,
    };
    if matches!(
        materialized_expr_type_for_merge(ctx, &args[2]).codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. }
    ) {
        return None;
    }

    let callback = if matches!(args[1].kind, ExprKind::Closure { .. }) {
        lower_preg_replace_callback_closure(ctx, &args[1])?
    } else {
        lower_expr(ctx, &args[1])
    };
    let callback_type = ctx.builder.value_php_type(callback.value);
    let callback_temp = ctx.declare_synthetic_php_local(callback_type.clone());
    ctx.store_local(
        &callback_temp,
        callback,
        callback_type,
        Some(args[1].span),
    );

    let subject = lower_expr(ctx, &args[2]);
    let subject = persist_call_arg_if_string(ctx, subject, args[2].span);
    let subject_type = ctx.builder.value_php_type(subject.value);
    let subject_temp = ctx.declare_synthetic_php_local(subject_type.clone());
    ctx.store_local(&subject_temp, subject, subject_type, Some(args[2].span));

    let limit_temp = args.get(3).map(|limit| {
        let value = lower_expr(ctx, limit);
        let value_type = ctx.builder.value_php_type(value.value);
        let temp = ctx.declare_synthetic_php_local(value_type.clone());
        ctx.store_local(&temp, value, value_type, Some(limit.span));
        temp
    });

    for pattern in patterns {
        let pattern = lower_expr(ctx, pattern);
        let callback = ctx.load_local(&callback_temp, Some(expr.span));
        let subject = ctx.load_local(&subject_temp, Some(expr.span));
        let mut operands = vec![pattern.value, callback.value, subject.value];
        if let Some(limit_temp) = &limit_temp {
            operands.push(ctx.load_local(limit_temp, Some(expr.span)).value);
        }
        let replaced = emit_builtin_call_value(
            ctx,
            canonical,
            operands,
            PhpType::Str,
            expr.span,
            None,
        );
        ctx.store_local(
            &subject_temp,
            replaced,
            PhpType::Str,
            Some(expr.span),
        );
    }

    Some(ctx.load_local(&subject_temp, Some(expr.span)))
}

/// Emits a builtin call and releases owned temporary arguments after the call consumes them.
pub(super) fn emit_builtin_call_value(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    operands: Vec<crate::ir::ValueId>,
    php_type: PhpType,
    span: Span,
    eval_literal: Option<&str>,
) -> LoweredValue {
    if eval_literal.is_none() {
        if let Some(def) = crate::builtins::registry::lookup(name) {
            let lowered = crate::builtins::semantics::lower_registry_call(
                ctx,
                def,
                &operands,
                &php_type,
                span,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "checked builtin {} failed backend-neutral EIR lowering at {}:{}: {}",
                    def.name,
                    span.line,
                    span.col,
                    error,
                )
            });
            let call = LoweredValue {
                value: lowered.value,
                ir_type: ctx.builder.value_type(lowered.value),
            };
            let return_alias = match def.spec.semantics.result_ownership {
                crate::builtins::semantics::BuiltinResultOwnership::NonHeap
                | crate::builtins::semantics::BuiltinResultOwnership::Fresh
                | crate::builtins::semantics::BuiltinResultOwnership::Independent => {
                    ReturnArgAlias::None
                }
                crate::builtins::semantics::BuiltinResultOwnership::Aliases(indexes) => {
                    ReturnArgAlias::Parameters(indexes.iter().copied().collect())
                }
                crate::builtins::semantics::BuiltinResultOwnership::Borrowed
                | crate::builtins::semantics::BuiltinResultOwnership::MayAliasArguments => {
                    ReturnArgAlias::Unknown
                }
            };
            release_owned_call_arg_temporaries(
                ctx,
                &operands,
                Some(call.value),
                &return_alias,
                span,
            );
            if php_symbol_key(name.trim_start_matches('\\')) == "extract" {
                ctx.mark_eval_executed();
                ctx.apply_eval_barrier();
            }
            return call;
        }
    }
    let (op, immediate, effects) = if let Some(fragment) = eval_literal {
        (
            Op::EvalLiteralCall,
            Some(Immediate::ProfiledData {
                data: ctx.intern_string(fragment),
                strict_php: crate::strict_php::is_enabled(),
            }),
            Op::EvalLiteralCall.default_effects(),
        )
    } else {
        let data = ctx.intern_function_name(name);
        let immediate = if php_symbol_key(name.trim_start_matches('\\')) == "eval" {
            Immediate::ProfiledData {
                data,
                strict_php: crate::strict_php::is_enabled(),
            }
        } else {
            Immediate::Data(data)
        };
        (
            Op::LanguageConstructCall,
            Some(immediate),
            effects_lookup::language_construct_effects(name),
        )
    };
    let call = ctx.emit_value(
        op,
        operands.clone(),
        immediate,
        php_type,
        effects,
        Some(span),
    );
    release_owned_call_arg_temporaries(
        ctx,
        &operands,
        Some(call.value),
        &ReturnArgAlias::Unknown,
        span,
    );
    let eval_needs_barrier = match eval_literal {
        Some(fragment) => eval_literal_needs_barrier(ctx, fragment),
        None => true,
    };
    if php_symbol_key(name.trim_start_matches('\\')) == "eval" {
        ctx.mark_eval_executed();
        if eval_needs_barrier {
            ctx.apply_eval_barrier();
        } else if let Some(write_names) = eval_literal
            .and_then(|fragment| eval_literal_scope_barrier_writes(ctx, fragment))
        {
            ctx.apply_eval_scope_barrier(&write_names);
        }
    }
    call
}

/// Resolves a migrated registry builtin's result type from the same descriptor as the checker.
///
/// Used for a builtin lowered at its own call site, so the checker's per-span result type is
/// authoritative and is passed through to the resolver.
pub(super) fn registry_builtin_result_type(
    ctx: &LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    operands: &[crate::ir::ValueId],
    span: Span,
) -> Option<PhpType> {
    // Synthetic builtin-class and prelude AST nodes share the dummy 0:0
    // span, so the checker map cannot identify an individual call there.
    // Use the typed runtime target's representation-safe fallback instead
    // of accepting whichever synthetic call last occupied that key.
    let checked = if span.line != 0 {
        ctx.builtin_call_types
            .get(&(ctx.loop_storage_scope.clone(), span))
            .map(|checked| normalize_value_php_type(checked.clone()))
    } else {
        None
    };
    resolve_registry_builtin_result_type(ctx, name, args, operands, span, checked)
}

/// Is a builtin's declared return type an acceptable stand-in for what the checker inferred?
///
/// Narrowing is fine in one direction only. A checked `int` falling back to a declared `mixed`
/// costs precision: `mixed` is the universal boxed representation of a PHP *value*, and lowering
/// knows how to carry any value in one. A checked `pointer` or `buffer` falling back to `mixed`
/// is not a loss of precision but a change of REPRESENTATION — a raw descriptor is not a boxed
/// cell — and codegen has no way to notice. Six builtins declared `mixed` while checking as
/// `Pointer` or `Callable`, which stayed harmless only for as long as every one of their call
/// sites could be found in the checker's per-span map; the day the PDO prelude stopped being
/// parsed, none of them could be, and all 276 PDO tests failed at once.
fn declared_type_is_a_safe_fallback(declared: &PhpType, checked: Option<&PhpType>) -> bool {
    let Some(checked) = checked else {
        return true;
    };
    fn is_a_php_value(ty: &PhpType) -> bool {
        !matches!(
            ty,
            PhpType::Pointer(_) | PhpType::Buffer(_) | PhpType::Packed(_) | PhpType::Callable
        )
    }
    is_a_php_value(checked) == is_a_php_value(declared)
}

/// Resolves a registry builtin's result type from its descriptor, its lowered operands, and the
/// checker's result type for this very call when one is available.
///
/// `checked` must be `None` whenever the caller cannot prove the checker examined *this* builtin
/// at `span`; the resolver then derives a representation-safe type from the typed runtime target
/// instead of trusting a type that may describe a different call.
pub(super) fn resolve_registry_builtin_result_type(
    ctx: &LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    operands: &[crate::ir::ValueId],
    span: Span,
    checked: Option<PhpType>,
) -> Option<PhpType> {
    let def = crate::builtins::registry::lookup(name)?;
    debug_assert!(
        declared_type_is_a_safe_fallback(&def.return_type, checked.as_ref()),
        "builtin `{name}` checks as {:?} but declares {:?}. Every path that cannot name this \
         call — a callable dispatched at runtime, a span the checker never filed — falls back to \
         the DECLARED type, so a declaration of a different representation is a miscompile \
         waiting for one of them.",
        checked,
        def.return_type,
    );
    let arg_types = operands
        .iter()
        .map(|operand| ctx.builder.value_php_type(*operand))
        .collect::<Vec<_>>();
    let input = crate::builtins::semantics::BuiltinSemanticInput {
        name: def.name,
        args,
        arg_types: &arg_types,
        span,
    };
    let resolved = match def.spec.semantics.result_type {
        crate::builtins::semantics::BuiltinResultType::Checked => {
            let crate::builtins::semantics::BuiltinLowering::Runtime(
                crate::ir::RuntimeCallTarget::Function(target),
            ) = def.spec.semantics.lowering
            else {
                return checked;
            };
            // The checker types an untyped user-function parameter from its call sites, while EIR
            // gives it the dynamic boxed-Mixed ABI contract, so a checked type can describe a
            // narrower element layout than the operands actually carry. A runtime target that
            // copies an argument's layout rejects such a type and re-derives its own.
            if let Some(checked) = checked {
                if target.checked_result_type_fits_operands(&arg_types, &checked) {
                    return Some(checked);
                }
            }
            target.fallback_result_type(&arg_types, &def.return_type)
        }
        crate::builtins::semantics::BuiltinResultType::Declared => def.return_type.clone(),
        crate::builtins::semantics::BuiltinResultType::Shared(resolve) => resolve(&input),
    };
    Some(normalize_value_php_type(resolved))
}
