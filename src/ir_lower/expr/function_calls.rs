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
    // A mutating builtin whose by-reference array argument is a property, static property, or
    // container element is rewritten to `$tmp = <place>; f($tmp, ...); <place> = $tmp;` before
    // any builtin fast path runs, so the rewritten call reaches the local-variable
    // by-reference lowering that actually stores the copy-on-write result back.
    if let Some(value) = ref_place_args::lower_builtin_ref_place_call(ctx, name, args, expr) {
        return value;
    }
    if let Some(value) = lower_static_array_map(ctx, canonical, args, expr) {
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
    let extension_builtin = source_prefers_extension_builtin(canonical);
    // Synthetic EIR bodies are constructed after name resolution, so their generated `Name`
    // nodes can retain a root marker or non-canonical casing. Match known extern/user names by
    // PHP symbol key before deciding whether this is a builtin fallback.
    let requested = canonical.trim_start_matches('\\');
    let extern_name = lookup_folded_name(ctx.extern_functions.keys(), requested);
    let user_name = (!extension_builtin)
        .then(|| lookup_folded_name(ctx.functions.keys(), requested))
        .flatten();
    let callee = extern_name
        .as_deref()
        .or(user_name.as_deref())
        .unwrap_or(canonical);
    let sig = call_signature(ctx, callee, extension_builtin);
    let is_extern = extern_name.is_some();
    let is_user_function = user_name.is_some();
    let operands = if is_user_function {
        lower_args_with_eir_user_function_signature(ctx, callee, sig.as_ref(), args)
    } else if is_extern {
        lower_args_with_signature(ctx, sig.as_ref(), args)
    } else {
        lower_builtin_call_args(ctx, callee, sig.as_ref(), args)
    };
    let php_type = if is_extern || is_user_function {
        call_return_type(ctx, callee, &operands)
    } else if let Some(php_type) =
        registry_builtin_result_type(ctx, callee, args, &operands, expr.span)
    {
        php_type
    } else {
        call_return_type(ctx, callee, &operands)
    };
    if is_extern {
        let data = ctx.intern_function_name(callee);
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
        if matches!(
            php_symbol_key(canonical.trim_start_matches('\\')).as_str(),
            "elephc\\async\\run" | "elephc\\parallel\\run"
        ) {
            return lower_exception_safe_user_call(
                ctx,
                callee,
                operands,
                php_type,
                sig.as_ref(),
                expr,
            );
        }
        if call_has_owning_temporary_arg(ctx, &operands) {
            return lower_exception_safe_user_call(ctx, callee, operands, php_type, sig.as_ref(), expr);
        }
        let data = ctx.intern_function_name(callee);
        let call = ctx.emit_value(
            Op::Call,
            operands.clone(),
            Some(Immediate::Data(data)),
            php_type,
            effects_lookup::user_call_effects(callee),
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
    if ctx.has_eval_barrier()
        && plain_positional_call_args(args)
        && canonical_builtin_function_name(canonical).is_none()
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

/// Returns whether a user call needs a cleanup handler for an owning argument temporary.
pub(super) fn call_has_owning_temporary_arg(
    ctx: &LoweringContext<'_, '_>,
    operands: &[crate::ir::ValueId],
) -> bool {
    operands.iter().any(|value| {
        let php_type = ctx.builder.value_php_type(*value);
        ctx.value_is_owning_temporary(LoweredValue {
            value: *value,
            ir_type: value_ir_type(&php_type),
        })
    })
}

/// Lowers a user call with symmetric call-argument cleanup on throw.
///
/// A caller-owned closure, array, string, or Mixed box can be a temporary. Ordinary post-call
/// cleanup cannot execute when a user callee throws through `longjmp`, so a tiny internal handler
/// releases those argument owners before rethrowing the current exception. The successful path
/// performs the same cleanup and moves non-void results through an owned hidden temporary so both
/// control-flow edges preserve the ordinary call contract.
pub(super) fn lower_exception_safe_user_call(
    ctx: &mut LoweringContext<'_, '_>,
    canonical: &str,
    operands: Vec<crate::ir::ValueId>,
    php_type: PhpType,
    signature: Option<&FunctionSig>,
    expr: &Expr,
) -> LoweredValue {
    if matches!(&php_type, PhpType::Void) {
        return lower_exception_safe_void_user_call(ctx, canonical, operands, php_type, signature, expr);
    }
    let handler = ctx
        .builder
        .create_named_block("structured.run.call_cleanup", Vec::new());
    let after = ctx
        .builder
        .create_named_block("structured.run.call_after", Vec::new());
    let handler_token = handler.as_raw() as i64;
    let result_temp = ctx.declare_owned_hidden_temp(php_type.clone());
    let return_alias = ctx.return_alias_summaries.function(canonical).cloned()
        .unwrap_or(ReturnArgAlias::Unknown);
    ctx.emit_void(
        Op::TryPushHandler,
        Vec::new(),
        Some(Immediate::I64(handler_token)),
        Op::TryPushHandler.default_effects(),
        Some(expr.span),
    );
    let data = ctx.intern_function_name(canonical);
    let call = ctx.emit_value(
        Op::Call,
        operands.clone(),
        Some(Immediate::Data(data)),
        php_type.clone(),
        effects_lookup::user_call_effects(canonical),
        Some(expr.span),
    );
    store_value_into_temp(
        ctx,
        &result_temp,
        php_type,
        call,
        expr.span,
    );
    release_owned_call_arg_temporaries_with_signature(
        ctx,
        &operands,
        Some(call.value),
        &return_alias,
        signature,
        expr.span,
    );
    emit_structured_run_call_handler_pop(ctx, handler_token, expr.span);
    branch_to(ctx, after);

    ctx.builder.position_at_end(handler);
    ctx.clear_static_callable_locals();
    emit_structured_run_call_handler_pop(ctx, handler_token, expr.span);
    release_owned_call_arg_temporaries_with_signature(
        ctx,
        &operands,
        None,
        &ReturnArgAlias::None,
        signature,
        expr.span,
    );
    // The cleanup handler rethrows rather than observes the exception. Take the runtime slot's
    // owner before transferring it into the terminator; borrowing with `CatchCurrent` leaves a
    // second owner behind whenever a structured Async/Parallel root propagates a failure.
    let current = ctx.emit_owned_value(
        Op::CatchBind,
        Vec::new(),
        None,
        PhpType::Object("Throwable".to_string()),
        Op::CatchBind.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::Throw {
        value: current.value,
    });

    ctx.builder.position_at_end(after);
    ctx.clear_static_callable_locals();
    take_owned_temp(ctx, &result_temp, expr.span)
}

/// Lowers a void user call with argument cleanup on both the return and throw paths.
fn lower_exception_safe_void_user_call(
    ctx: &mut LoweringContext<'_, '_>,
    canonical: &str,
    operands: Vec<crate::ir::ValueId>,
    php_type: PhpType,
    signature: Option<&FunctionSig>,
    expr: &Expr,
) -> LoweredValue {
    let handler = ctx
        .builder
        .create_named_block("user.call_cleanup", Vec::new());
    let after = ctx
        .builder
        .create_named_block("user.call_after", Vec::new());
    let handler_token = handler.as_raw() as i64;
    ctx.emit_void(
        Op::TryPushHandler,
        Vec::new(),
        Some(Immediate::I64(handler_token)),
        Op::TryPushHandler.default_effects(),
        Some(expr.span),
    );
    let data = ctx.intern_function_name(canonical);
    let call = ctx.emit_value(
        Op::Call,
        operands.clone(),
        Some(Immediate::Data(data)),
        php_type,
        effects_lookup::user_call_effects(canonical),
        Some(expr.span),
    );
    release_owned_call_arg_temporaries_with_signature(
        ctx,
        &operands,
        Some(call.value),
        &ReturnArgAlias::None,
        signature,
        expr.span,
    );
    emit_structured_run_call_handler_pop(ctx, handler_token, expr.span);
    branch_to(ctx, after);

    ctx.builder.position_at_end(handler);
    ctx.clear_static_callable_locals();
    emit_structured_run_call_handler_pop(ctx, handler_token, expr.span);
    release_owned_call_arg_temporaries_with_signature(
        ctx,
        &operands,
        None,
        &ReturnArgAlias::None,
        signature,
        expr.span,
    );
    let current = ctx.emit_owned_value(
        Op::CatchBind,
        Vec::new(),
        None,
        PhpType::Object("Throwable".to_string()),
        Op::CatchBind.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::Throw {
        value: current.value,
    });

    ctx.builder.position_at_end(after);
    ctx.clear_static_callable_locals();
    call
}

/// Pops the synthetic handler installed around one structured `run()` call.
fn emit_structured_run_call_handler_pop(
    ctx: &mut LoweringContext<'_, '_>,
    handler_token: i64,
    span: Span,
) {
    ctx.emit_void(
        Op::TryPopHandler,
        Vec::new(),
        Some(Immediate::I64(handler_token)),
        Op::TryPopHandler.default_effects(),
        Some(span),
    );
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
            let cleanup = if call_has_owning_temporary_arg(ctx, &operands) {
                let handler = ctx.builder.create_named_block("builtin.arg_cleanup", Vec::new());
                let after = ctx.builder.create_named_block("builtin.arg_after", Vec::new());
                ctx.emit_void(
                    Op::TryPushHandler, Vec::new(), Some(Immediate::I64(handler.as_raw() as i64)),
                    Op::TryPushHandler.default_effects(), Some(span),
                );
                Some((handler, after))
            } else {
                None
            };
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
            let result_temp = cleanup.and_then(|_| {
                if call.ir_type == IrType::Void {
                    None
                } else {
                    let result_type = ctx.builder.value_php_type(call.value);
                    let temp = ctx.declare_owned_hidden_temp(result_type.clone());
                    store_value_into_temp(ctx, &temp, result_type, call, span);
                    Some(temp)
                }
            });
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
            if let Some((handler, after)) = cleanup {
                emit_structured_run_call_handler_pop(ctx, handler.as_raw() as i64, span);
                branch_to(ctx, after);
                ctx.builder.position_at_end(handler);
                emit_structured_run_call_handler_pop(ctx, handler.as_raw() as i64, span);
                release_owned_call_arg_temporaries(ctx, &operands, None, &ReturnArgAlias::None, span);
                let current = ctx.emit_owned_value(
                    Op::CatchBind, Vec::new(), None, PhpType::Object("Throwable".to_string()),
                    Op::CatchBind.default_effects(), Some(span),
                );
                ctx.builder.terminate(Terminator::Throw { value: current.value });
                ctx.builder.position_at_end(after);
                if let Some(temp) = result_temp {
                    return take_owned_temp(ctx, &temp, span);
                }
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
            .get(&span)
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
