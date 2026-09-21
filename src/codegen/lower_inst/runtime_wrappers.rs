//! Purpose:
//! Builds inline descriptor wrappers for runtime builtin and extern calls.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.

use super::*;

/// Emits a descriptor invoker inline and branches around its global entry body.
pub(super) fn emit_runtime_callable_invoker_inline(
    ctx: &mut FunctionContext<'_>,
    sig: &FunctionSig,
    captures: &[(String, PhpType, bool)],
) -> String {
    emit_runtime_callable_invoker_inline_with_boundary(ctx, sig, captures, false)
}

/// Emits a descriptor invoker inline, optionally bounded against native throws.
///
/// WHEN TO ASK FOR THE BOUNDARY. A descriptor the INTERPRETER calls needs one: a Throwable raised
/// inside the invoker otherwise reaches `__rt_throw_current` and unwinds natively past the
/// interpreted PHP `try` that wraps the call, which never gets to look at it. Symfony's
/// `default:` env processor is exactly that shape -- `try { $getEnv($next); } catch
/// (EnvNotFoundException) {}` where `$getEnv` is the first-class callable
/// `$container->getEnv(...)` -- and every `%env(default:...)%` in the app died on it.
///
/// WHEN NOT TO. Do NOT bound a descriptor that AOT code dispatches, which is why this is opt-in
/// per call site rather than the default: the boundary turns the native throw into a returned
/// status for EVERY caller, and AOT dispatch needs it to keep propagating. Bounding the shared
/// builtin/extern descriptors regressed
/// `codegen::runtime_reachability::test_shared_mixed_callable_dispatch_preserves_results_and_exceptions`
/// ("not-reached" where PHP prints "caught:boom:3"), because a callable that throws stopped
/// reaching its `catch`.
///
/// The two variants are cached separately: one label per (sig, captures) is not enough once the
/// same signature can be emitted both bounded and unbounded, so the bounded form takes a fresh
/// label and is not shared with the unbounded cache.
pub(in crate::codegen) fn emit_runtime_callable_invoker_inline_with_boundary(
    ctx: &mut FunctionContext<'_>,
    sig: &FunctionSig,
    captures: &[(String, PhpType, bool)],
    catch_native_throws: bool,
) -> String {
    if !catch_native_throws {
        if let Some(label) = ctx.shared.runtime_callable_invoker(sig, captures) {
            return label;
        }
    }
    // An invoker shared through the cache takes its name from the cache key, so every worker
    // that reaches it spells the same symbol. One that is NOT shared (the exception-boundary
    // form) keeps a host-scoped name, because nothing else may reference it.
    let key = format!("callable_invoker:{:?}:{:?}", sig, captures);
    let label = if catch_native_throws {
        ctx.next_global_label("callable_invoker")
    } else {
        ctx.keyed_global_label(&key, "callable_invoker")
    };
    let done_label = ctx.next_label("callable_invoker_done");
    let invoker = super::super::runtime_callable_invoker::RuntimeCallableInvoker {
        label: &label,
        sig,
        captures,
    };
    // The thunk's global entry opens its own `.text` section on ELF; put the
    // enclosing function back before continuing it, or its tail lands in there.
    let enclosing = ctx.emitter.current_text_section();
    abi::emit_jump(ctx.emitter, &done_label);
    if catch_native_throws {
        // The boundary form is NOT shared: its symbol is host-scoped and nothing caches it, so
        // every instance is a distinct body. Marking it as a keyed helper would make the merge
        // treat two different invokers as copies of one and drop a definition still referenced.
        super::super::runtime_callable_invoker::emit_runtime_callable_invoker_with_exception_boundary(
            ctx.emitter,
            ctx.data,
            &invoker,
        );
    } else {
        ctx.emit_keyed_helper(&key, |ctx| {
            super::super::runtime_callable_invoker::emit_runtime_callable_invoker(ctx.emitter, ctx.data, &invoker);
        });
    }
    ctx.emitter.reopen_text_section(enclosing);
    ctx.emitter.label(&done_label);
    if !catch_native_throws {
        ctx.shared
            .cache_runtime_callable_invoker(sig, captures, &label);
    }
    label
}

/// Emits a synthetic EIR builtin wrapper so callable descriptors can use the PHP ABI.
pub(in crate::codegen) fn emit_runtime_builtin_wrapper_inline(
    ctx: &mut FunctionContext<'_>,
    name: &str,
    sig: &FunctionSig,
    strict_php: bool,
) -> Result<String> {
    emit_runtime_call_wrapper_inline(
        ctx,
        name,
        sig,
        RuntimeCallWrapperKind::Builtin { strict_php },
    )
}

/// Returns the registry/runtime-descriptor ABI used by builtin callable wrappers.
pub(in crate::codegen) fn runtime_builtin_wrapper_sig(name: &str, sig: &FunctionSig) -> FunctionSig {
    let mut sig = sig.clone();
    if let Some(def) = crate::builtins::registry::lookup(name) {
        if let crate::builtins::semantics::BuiltinRuntimeFunctions::One(runtime_fn) =
            def.spec.semantics.runtime_functions
        {
            runtime_fn.refine_runtime_callable_wrapper_sig(&mut sig);
        }
    }
    sig
}

/// Emits an EIR extern wrapper inline so descriptors can point at PHP-ABI code.
pub(in crate::codegen) fn emit_runtime_extern_wrapper_inline(
    ctx: &mut FunctionContext<'_>,
    name: &str,
    sig: &FunctionSig,
) -> Result<String> {
    emit_runtime_call_wrapper_inline(ctx, name, sig, RuntimeCallWrapperKind::Extern)
}

/// Kind of call instruction used by a descriptor entry wrapper.
#[derive(Clone, Copy)]
enum RuntimeCallWrapperKind {
    Builtin { strict_php: bool },
    Extern,
}

/// Emits a synthetic EIR wrapper that forwards PHP-ABI descriptor entry calls.
fn emit_runtime_call_wrapper_inline(
    ctx: &mut FunctionContext<'_>,
    name: &str,
    sig: &FunctionSig,
    kind: RuntimeCallWrapperKind,
) -> Result<String> {
    let cached = match kind {
        RuntimeCallWrapperKind::Builtin { strict_php } => {
            ctx.shared.runtime_builtin_wrapper(name, sig, strict_php)
        }
        RuntimeCallWrapperKind::Extern => ctx.shared.runtime_extern_wrapper(name, sig),
    };
    if let Some(label) = cached {
        return Ok(label);
    }
    let label_prefix = match kind {
        RuntimeCallWrapperKind::Builtin { .. } => "callable_builtin",
        RuntimeCallWrapperKind::Extern => "callable_extern",
    };
    let strictness = match kind {
        RuntimeCallWrapperKind::Builtin { strict_php } => strict_php,
        RuntimeCallWrapperKind::Extern => false,
    };
    let key = format!("{}:{}:{:?}:{}", label_prefix, name, sig, strictness);
    let label = ctx.keyed_global_label(&key, label_prefix);
    let done_label = ctx.next_label(&format!("{}_done", label_prefix));
    let mut wrapper_module = ctx.module.clone();
    let wrapper = build_runtime_call_wrapper_function(&mut wrapper_module, &label, name, sig, kind)?;
    let enclosing = ctx.emitter.current_text_section();
    abi::emit_jump(ctx.emitter, &done_label);
    ctx.emit_keyed_helper(&key, |ctx| {
        super::super::block_emit::emit_synthetic_function_with_label(
            &wrapper_module,
            &wrapper,
            &label,
            ctx.emitter,
            ctx.data,
            ctx.shared,
            false,
        )
    })?;
    ctx.emitter.reopen_text_section(enclosing);
    ctx.emitter.label(&done_label);
    match kind {
        RuntimeCallWrapperKind::Builtin { strict_php } => {
            ctx.shared
                .cache_runtime_builtin_wrapper(name, sig, strict_php, &label)
        }
        RuntimeCallWrapperKind::Extern => {
            ctx.shared.cache_runtime_extern_wrapper(name, sig, &label)
        }
    }
    Ok(label)
}

/// Builds the EIR body for a PHP-ABI wrapper around a builtin or extern call.
fn build_runtime_call_wrapper_function(
    module: &mut Module,
    label: &str,
    name: &str,
    sig: &FunctionSig,
    kind: RuntimeCallWrapperKind,
) -> Result<Function> {
    let return_php_type = wrapper_return_php_type(&sig.return_type);
    let mut function = Function::new(
        label.to_string(),
        wrapper_return_ir_type(&return_php_type),
        return_php_type.clone(),
    );
    function.signature = Some(sig.clone());
    let params = wrapper_function_params(sig);
    function.params = params.clone();
    for param in params {
        function.add_local(
            Some(param.name.clone()),
            param.ir_type,
            param.php_type.clone(),
            LocalKind::PhpLocal,
        );
    }

    let data = module.data.intern_function_name(name);
    let mut builder = Builder::new(&mut function);
    let entry = builder.create_named_block("entry", Vec::new());
    builder.set_entry(entry);
    builder.position_at_end(entry);
    let operands = wrapper_param_operands(&mut builder, sig);
    let result = match kind {
        RuntimeCallWrapperKind::Builtin { strict_php } => {
            let def = crate::builtins::registry::lookup(name).ok_or_else(|| {
                CodegenIrError::invalid_module(format!(
                    "callable wrapper {} is not registry-backed",
                    name,
                ))
            })?;
            let operands = coerce_gradual_wrapper_operands(&mut builder, name, sig, operands);
            let mut lowering = WrapperBuiltinLoweringContext {
                builder: &mut builder,
                strict_php,
            };
            Some(crate::builtins::semantics::lower_registry_call(
                &mut lowering,
                def,
                &operands,
                &return_php_type,
                crate::span::Span::dummy(),
            )
            .map_err(|error| {
                CodegenIrError::invalid_module(format!(
                    "callable wrapper lowering for {} failed: {}",
                    name, error,
                ))
            })?
            .value)
        }
        RuntimeCallWrapperKind::Extern => builder.emit(
            Op::ExternCall,
            operands,
            Some(Immediate::Data(data)),
            wrapper_return_ir_type(&return_php_type),
            return_php_type.clone(),
            Ownership::for_php_type(&return_php_type),
        ),
    };
    let result = persist_scratch_backed_wrapper_string(&mut builder, result, &return_php_type);
    builder.terminate(Terminator::Return { value: result });
    Ok(function)
}

/// Copies a scratch-backed string out of the concat arena before the wrapper hands it back.
///
/// A wrapper body is built here rather than lowered from PHP, so it never reaches
/// `persist_scratch_return_string` — and the descriptor invoker that calls it REWINDS `_concat_off`
/// to the pre-call value the moment the wrapper returns, on the documented understanding that a
/// `Str` result is already owned (`emit_boxed_invoker_return`). Returning the arena pointer instead
/// left every result of `array_map('strtoupper', …)` pointing at one address, each with its own
/// length, so the last element overwrote the others in place.
///
/// The question asked is the one return lowering asks: does the DEFINING op write to the shared
/// scratch storage? A builtin that already returns heap-owned bytes is left alone, because copying
/// it here would orphan the copy the invoker does not know about.
fn persist_scratch_backed_wrapper_string(
    builder: &mut Builder<'_>,
    result: Option<ValueId>,
    return_php_type: &PhpType,
) -> Option<ValueId> {
    let value = result?;
    if return_php_type.codegen_repr() != PhpType::Str {
        return result;
    }
    let scratch_backed = builder
        .value_defining_instruction(value)
        .is_some_and(|inst| crate::ir_lower::string_op_uses_scratch_storage(inst.op));
    if !scratch_backed {
        return result;
    }
    builder.emit(
        Op::StrPersist,
        vec![value],
        None,
        IrType::Str,
        PhpType::Str,
        Ownership::for_php_type(&PhpType::Str),
    )
}

/// Casts a GRADUAL wrapper operand to the parameter type the builtin's own lowering needs.
///
/// A callable wrapper's parameters ARE the source element types: the descriptor case is
/// specialized to the array it will walk. A `Mixed` element therefore reached, say, `strtolower`'s
/// lowering, which needs a concrete string -- so the only way to keep the wrapper lowerable was to
/// refuse every gradual source up front (`callable_accepts_string_source`), and
/// `array_map('strtolower', $gradual)` compiled into a runtime abort:
/// `callback string does not name a supported callable`. php casts there, and so does this.
///
/// Only a SCALAR target is cast. A container or object parameter has no single cast, and the
/// builtins that take one already accept a gradual source through their own policy.
fn coerce_gradual_wrapper_operands(
    builder: &mut Builder<'_>,
    name: &str,
    sig: &FunctionSig,
    operands: Vec<ValueId>,
) -> Vec<ValueId> {
    let Some(declared) = crate::types::first_class_callable_builtin_sig(name) else {
        return operands;
    };
    operands
        .into_iter()
        .enumerate()
        .map(|(idx, operand)| {
            let source = sig.params.get(idx).map(|(_, ty)| ty.codegen_repr());
            let target = declared.params.get(idx).map(|(_, ty)| ty.codegen_repr());
            let (Some(PhpType::Mixed), Some(target)) = (source, target) else {
                return operand;
            };
            if !matches!(
                target,
                PhpType::Str | PhpType::Int | PhpType::Float | PhpType::Bool
            ) {
                return operand;
            }
            builder
                .emit(
                    Op::RuntimeCall,
                    vec![operand],
                    None,
                    wrapper_value_ir_type(&target),
                    target.clone(),
                    Ownership::for_php_type(&target),
                )
                .unwrap_or(operand)
        })
        .collect()
}

/// EIR construction adapter used by synthetic builtin callable wrappers.
struct WrapperBuiltinLoweringContext<'a, 'f> {
    builder: &'a mut Builder<'f>,
    strict_php: bool,
}

impl crate::builtins::semantics::BuiltinLoweringContext
    for WrapperBuiltinLoweringContext<'_, '_>
{
    /// Returns PHP metadata attached to one synthetic-wrapper operand.
    fn value_php_type(&self, value: ValueId) -> PhpType {
        self.builder.value_php_type(value)
    }

    /// Emits one backend-neutral operation into the synthetic wrapper body.
    fn emit_value(
        &mut self,
        op: Op,
        operands: Vec<ValueId>,
        immediate: Option<Immediate>,
        php_type: PhpType,
        effects: crate::ir::Effects,
        span: Option<crate::span::Span>,
    ) -> crate::builtins::semantics::LoweredBuiltinValue {
        let value = self
            .builder
            .emit_with_effects(
                op,
                operands,
                immediate,
                wrapper_value_ir_type(&php_type),
                php_type.clone(),
                Ownership::for_php_type(&php_type),
                effects,
                span,
            )
            .expect("builtin wrapper operation produces a value");
        crate::builtins::semantics::LoweredBuiltinValue { value }
    }

    /// Emits one typed runtime operation into the synthetic wrapper body.
    fn emit_runtime_call(
        &mut self,
        target: crate::ir::RuntimeCallTarget,
        operands: Vec<ValueId>,
        php_type: PhpType,
        effects: crate::ir::Effects,
        span: Option<crate::span::Span>,
    ) -> crate::builtins::semantics::LoweredBuiltinValue {
        let target = match target {
            crate::ir::RuntimeCallTarget::Function(target) => {
                crate::ir::RuntimeCallTarget::ProfiledFunction {
                    target,
                    strict_php: self.strict_php,
                }
            }
            target => target,
        };
        self.emit_value(
            Op::RuntimeCall,
            operands,
            Some(Immediate::RuntimeCall(target)),
            php_type,
            effects,
            span,
        )
    }

    /// A synthetic wrapper has no function-name pool to intern a callee into, and no
    /// registry builtin that composes user calls admits a runtime-selected callable
    /// (`BuiltinCallablePolicy::StaticOnly`), so this is never reached; it answers a
    /// null constant rather than a call so a future policy change fails visibly in tests
    /// instead of emitting an unresolved symbol.
    fn emit_user_call(
        &mut self,
        _name: &str,
        _operands: Vec<ValueId>,
        php_type: PhpType,
        span: Option<crate::span::Span>,
    ) -> crate::builtins::semantics::LoweredBuiltinValue {
        self.emit_value(
            Op::ConstNull,
            Vec::new(),
            None,
            php_type,
            Op::ConstNull.default_effects(),
            span,
        )
    }

    /// A wrapper's operands are its own parameters, never a caller's locals, so there is
    /// no by-reference output to write; the composing builtin reports the failure.
    fn store_operand_local(
        &mut self,
        _operand: ValueId,
        _value: ValueId,
        _php_type: PhpType,
        _span: Option<crate::span::Span>,
    ) -> bool {
        false
    }
}

/// Converts callable signature params into EIR function params with matching ABI/local slots.
pub(super) fn wrapper_function_params(sig: &FunctionSig) -> Vec<FunctionParam> {
    sig.params
        .iter()
        .enumerate()
        .map(|(idx, (name, php_type))| FunctionParam {
            name: name.clone(),
            ir_type: wrapper_value_ir_type(php_type),
            php_type: php_type.clone(),
            by_ref: sig.ref_params.get(idx).copied().unwrap_or(false),
            variadic: sig.variadic.as_deref() == Some(name.as_str()),
        })
        .collect()
}

/// Emits `LoadLocal` operands for every wrapper parameter.
pub(super) fn wrapper_param_operands(builder: &mut Builder<'_>, sig: &FunctionSig) -> Vec<ValueId> {
    sig.params
        .iter()
        .enumerate()
        .map(|(idx, (_, php_type))| {
            builder.emit_load_local(
                LocalSlotId::from_raw(idx as u32),
                wrapper_value_ir_type(php_type),
                php_type.clone(),
            )
        })
        .collect()
}

/// Returns a materializable PHP type for wrapper return values.
pub(super) fn wrapper_return_php_type(php_type: &PhpType) -> PhpType {
    match php_type.codegen_repr() {
        PhpType::Never => PhpType::Void,
        other => other,
    }
}

/// Returns EIR return storage for a wrapper function signature.
pub(super) fn wrapper_return_ir_type(php_type: &PhpType) -> IrType {
    match php_type.codegen_repr() {
        PhpType::Void | PhpType::Never => IrType::Void,
        other => IrType::from_php(&other),
    }
}

/// Returns EIR value storage for wrapper params and call results.
pub(super) fn wrapper_value_ir_type(php_type: &PhpType) -> IrType {
    match php_type.codegen_repr() {
        PhpType::Void | PhpType::Never => IrType::I64,
        other => IrType::from_php(&other),
    }
}
