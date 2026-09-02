//! Purpose:
//! Creates eval contexts and seeds top-level declared-symbol metadata.
//!
//! Called from:
//! - The eval lowering facade and sibling eval support modules.
//!
//! Key details:
//! - Context creation also preserves regex-provider and PHP-profile registration.

use super::*;

/// Ensures a persistent eval context exists and stores its handle in the scratch frame.
pub(super) fn ensure_eval_context(ctx: &mut FunctionContext<'_>) -> Result<()> {
    let slot = eval_context_slot(ctx)?;
    let offset = ctx.local_offset(slot)?;
    let ready = ctx.next_label("eval_context_ready");
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::load_at_offset(ctx.emitter, result_reg, offset);
    abi::emit_branch_if_int_result_nonzero(ctx.emitter, &ready);
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_context_new");
    abi::emit_call_label(ctx.emitter, &symbol);
    abi::store_at_offset(ctx.emitter, result_reg, offset);
    let context_arg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    abi::load_at_offset(ctx.emitter, context_arg, offset);
    let metadata_sync = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_context_try_sync_aot_metadata");
    abi::emit_call_label(ctx.emitter, &metadata_sync);
    abi::emit_branch_if_int_result_nonzero(ctx.emitter, &ready);
    abi::load_at_offset(ctx.emitter, context_arg, offset);
    if let Some(label) = ctx.shared.eval_registration_helper() {
        abi::emit_call_label(ctx.emitter, &label);
    } else {
        let label = ctx.next_label("eval_register_module");
        ctx.shared.cache_eval_registration_helper(label.clone());
        abi::emit_call_label(ctx.emitter, &label);
        let helper_done = ctx.next_label("eval_register_module_done");
        abi::emit_jump(ctx.emitter, &helper_done);
        emit_eval_registration_helper(ctx, &label)?;
        ctx.emitter.label(&helper_done);
    }
    ctx.emitter.label(&ready);
    abi::load_at_offset(ctx.emitter, result_reg, offset);
    abi::emit_store_to_sp(ctx.emitter, result_reg, EVAL_CONTEXT_HANDLE_OFFSET);
    Ok(())
}

/// Loads this function's eval context, or a null handle for request-global fallback lookup.
///
/// A statically typed function can be compiled without an eval operation of its own while a
/// different frame owns the request's registered SPL callbacks.  The dynamic construction ABI
/// accepts a null handle for that case and resolves the retained callback-owner contexts instead
/// of forcing every AOT function to reserve a persistent eval frame slot.
pub(super) fn load_eval_context_or_null(ctx: &mut FunctionContext<'_>) -> Result<()> {
    if ctx
        .function
        .locals
        .iter()
        .any(|local| local.kind == LocalKind::EvalContext)
    {
        return ensure_eval_context(ctx);
    }
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_load_int_immediate(ctx.emitter, result_reg, 0);
    abi::emit_store_to_sp(ctx.emitter, result_reg, EVAL_CONTEXT_HANDLE_OFFSET);
    Ok(())
}

/// Emits the single module-wide body that registers all generated eval metadata.
///
/// The helper is called once after each distinct context allocation, but its
/// assembly body is emitted only at the first eval site. A dedicated ABI frame
/// preserves both the incoming context handle and the caller's return address.
fn emit_eval_registration_helper(ctx: &mut FunctionContext<'_>, label: &str) -> Result<()> {
    ctx.emitter.label_shared(label);
    abi::emit_frame_prologue(ctx.emitter, EVAL_CONTEXT_HELPER_FRAME_SIZE);
    let context_arg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    abi::store_at_offset(ctx.emitter, context_arg, EVAL_CONTEXT_HELPER_LOCAL_OFFSET);
    register_eval_regex_provider(ctx);
    register_eval_declared_symbols(ctx, EVAL_CONTEXT_HELPER_LOCAL_OFFSET);
    register_eval_native_functions(ctx, EVAL_CONTEXT_HELPER_LOCAL_OFFSET)?;
    register_eval_native_method_signatures(ctx, EVAL_CONTEXT_HELPER_LOCAL_OFFSET);
    publish_eval_aot_metadata(ctx, EVAL_CONTEXT_HELPER_LOCAL_OFFSET);
    abi::emit_frame_restore(ctx.emitter, EVAL_CONTEXT_HELPER_FRAME_SIZE);
    abi::emit_return(ctx.emitter);
    Ok(())
}

/// Publishes the fully registered AOT metadata for bridge calls made without a generated context.
fn publish_eval_aot_metadata(ctx: &mut FunctionContext<'_>, context_offset: usize) {
    load_eval_context_local_to_arg(ctx, context_offset, 0);
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_context_publish_aot_metadata");
    abi::emit_call_label(ctx.emitter, &symbol);
}

/// Registers managed PCRE2 shim callbacks when regex is enabled for this binary.
pub(super) fn register_eval_regex_provider(ctx: &mut FunctionContext<'_>) {
    if !ctx.module.required_runtime_features.regex {
        return;
    }
    for (index, provider_symbol) in [
        "elephc_pcre2_v1_compile",
        "elephc_pcre2_v1_exec",
        "elephc_pcre2_v1_free",
    ]
    .into_iter()
    .enumerate()
    {
        let arg_reg = abi::int_arg_reg_name(ctx.emitter.target, index);
        let symbol = ctx.emitter.target.extern_symbol(provider_symbol);
        abi::emit_symbol_address(ctx.emitter, arg_reg, &symbol);
    }
    let register = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_register_regex_provider");
    abi::emit_call_label(ctx.emitter, &register);
}

/// Writes the physical eval call site's strict profile before every runtime dispatch.
///
/// Writing both true and false prevents a strict eval from leaking its profile into
/// a later LFC eval that reuses the same persistent bridge context.
pub(super) fn mark_eval_strict_php(ctx: &mut FunctionContext<'_>, inst: &Instruction) {
    let strict_php = matches!(
        inst.immediate,
        Some(Immediate::ProfiledData {
            strict_php: true,
            ..
        })
    );
    mark_eval_strict_php_value(ctx, strict_php);
}

/// Writes an explicitly preserved source-profile flag into the active eval context.
pub(super) fn mark_eval_strict_php_value(ctx: &mut FunctionContext<'_>, strict_php: bool) {
    let arg_reg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    abi::emit_load_int_immediate(ctx.emitter, arg_reg, i64::from(strict_php));
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_set_strict_php");
    abi::emit_call_label(ctx.emitter, &symbol);
}

/// Writes the compilation's PHP profile before every runtime dispatch.
///
/// Without this, `PHP_VERSION` and its siblings fork at the eval boundary: a binary compiled
/// `--php-version 8.2` would report `8.2.0` natively and `8.5.0` from inside `eval()`. The
/// bridge defaults to the newest profile, so this call is what makes the older ones true.
pub(super) fn mark_eval_php_version(ctx: &mut FunctionContext<'_>) {
    let version_id = i64::from(crate::codegen::compile_php_version().version_id());
    let arg_reg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    abi::emit_load_int_immediate(ctx.emitter, arg_reg, version_id);
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_set_php_version_id");
    abi::emit_call_label(ctx.emitter, &symbol);
}

/// Returns the hidden frame slot that owns this function's persistent eval context.
pub(super) fn eval_context_slot(ctx: &FunctionContext<'_>) -> Result<LocalSlotId> {
    ctx.function
        .locals
        .iter()
        .find(|local| local.kind == LocalKind::EvalContext)
        .map(|local| local.id)
        .ok_or_else(|| CodegenIrError::invalid_module("eval call missing eval context local"))
}

/// Registers eligible AOT global functions with a newly allocated eval context.
pub(super) fn register_eval_native_functions(
    ctx: &mut FunctionContext<'_>,
    context_offset: usize,
) -> Result<()> {
    let registrations = eval_native_function_registrations(ctx);
    for registration in registrations {
        register_eval_native_function(ctx, context_offset, &registration)?;
    }
    Ok(())
}

/// Registers eligible AOT method and constructor signatures with a newly allocated eval context.
pub(super) fn register_eval_native_method_signatures(ctx: &mut FunctionContext<'_>, context_offset: usize) {
    for registration in eval_native_method_registrations(ctx) {
        register_eval_native_method(ctx, context_offset, &registration);
    }
    for registration in eval_native_constructor_registrations(ctx) {
        register_eval_native_constructor(ctx, context_offset, &registration);
    }
    for registration in eval_native_property_type_registrations(ctx) {
        register_eval_native_property_type(ctx, context_offset, &registration);
    }
    for registration in eval_native_abstract_property_registrations(ctx) {
        register_eval_native_abstract_property(ctx, context_offset, &registration);
    }
    for registration in eval_native_interface_property_registrations(ctx) {
        register_eval_native_interface_property(ctx, context_offset, &registration);
    }
    for registration in eval_native_property_default_registrations(ctx) {
        register_eval_native_property_default(ctx, context_offset, &registration);
    }
    for registration in eval_native_member_attribute_registrations(ctx) {
        register_eval_native_member_attribute(ctx, context_offset, &registration);
    }
    register_eval_native_class_parents(ctx, context_offset);
}

/// Registers generated declared-name metadata with a newly allocated eval context.
pub(super) fn register_eval_declared_symbols(ctx: &mut FunctionContext<'_>, context_offset: usize) {
    let class_names = ctx.module.declared_class_names.clone();
    let interface_names = ctx.module.declared_interface_names.clone();
    let trait_names = ctx.module.declared_trait_names.clone();
    for name in class_names {
        register_eval_declared_symbol_name(
            ctx,
            context_offset,
            "__elephc_eval_register_declared_class_name",
            &name,
        );
    }
    for name in interface_names {
        register_eval_declared_symbol_name(
            ctx,
            context_offset,
            "__elephc_eval_register_declared_interface_name",
            &name,
        );
    }
    for name in trait_names {
        register_eval_declared_symbol_name(
            ctx,
            context_offset,
            "__elephc_eval_register_declared_trait_name",
            &name,
        );
    }
}

/// Emits one declared-name metadata registration call into the eval context.
pub(super) fn register_eval_declared_symbol_name(
    ctx: &mut FunctionContext<'_>,
    context_offset: usize,
    symbol_name: &str,
    name: &str,
) {
    load_eval_context_local_to_arg(ctx, context_offset, 0);
    let (name_label, name_len) = ctx.data.add_string(name.as_bytes());
    abi::emit_symbol_address(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 1),
        &name_label,
    );
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 2),
        name_len as i64,
    );
    let symbol = ctx.emitter.target.extern_symbol(symbol_name);
    abi::emit_call_label(ctx.emitter, &symbol);
}
