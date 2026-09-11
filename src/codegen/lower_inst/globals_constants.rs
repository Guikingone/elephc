//! Purpose:
//! Lowers globals, extern globals, constants, Mixed boxing, and invoker ref markers.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.

use super::*;

/// Lowers a global storage load into the result register and SSA destination slot.
pub(super) fn lower_load_global(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let data = expect_global_name(inst)?;
    let name = ctx.global_name_data(data)?;
    let symbol = ir_global_symbol(name);
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("load_global missing result value"))?;
    let ty = ctx.value_php_type(result)?;
    if crate::superglobals::uses_shared_ref_cell(ctx.module, name) {
        load_shared_global_to_result(ctx, &symbol);
        return store_if_result(ctx, inst);
    }
    ctx.data
        .add_comm(symbol.clone(), ty.codegen_repr().stack_size().max(8));
    abi::emit_load_symbol_to_result(ctx.emitter, &symbol, &ty);
    store_if_result(ctx, inst)
}

/// Lowers a global storage store from one SSA operand.
pub(super) fn lower_store_global(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let data = expect_global_name(inst)?;
    let name = ctx.global_name_data(data)?.to_string();
    let symbol = ir_global_symbol(&name);
    let value = expect_operand(inst, 0)?;
    let ty = ctx.load_value_to_result(value)?;
    let store_ty = if ctx.module.web && crate::superglobals::is_superglobal(&name) {
        ty.codegen_repr()
    } else {
        let source_ty = ty.codegen_repr();
        if source_ty != PhpType::Mixed {
            if ctx.value_can_transfer_ownership_to_consumer(value)? {
                emit_box_current_owned_value_as_mixed(ctx.emitter, &source_ty);
            } else {
                emit_box_current_value_as_mixed(ctx.emitter, &source_ty);
            }
        }
        PhpType::Mixed
    };
    if crate::superglobals::uses_shared_ref_cell(ctx.module, &name) {
        return lower_store_shared_global(ctx, &symbol, &store_ty);
    }
    ctx.data
        .add_comm(symbol.clone(), store_ty.codegen_repr().stack_size().max(8));
    abi::emit_store_result_to_symbol(ctx.emitter, &symbol, &store_ty, true);
    Ok(())
}

/// Removes the name's global owner while existing aliases retain their cells.
pub(super) fn lower_unset_global(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let name = ctx.global_name_data(expect_global_name(inst)?)?.to_string();
    unset_global_name(ctx, &name);
    Ok(())
}

/// Removes the symbol's owner before invoking any payload destructor.
pub(in crate::codegen) fn unset_global_name(ctx: &mut FunctionContext<'_>, name: &str) {
    let symbol = ir_global_symbol(name);
    ctx.data.add_comm(symbol.clone(), 8);
    abi::emit_load_symbol_to_reg(ctx.emitter, abi::int_result_reg(ctx.emitter), &symbol, 0);
    abi::emit_store_zero_to_symbol(ctx.emitter, &symbol, 0);
    let release = if crate::superglobals::uses_shared_ref_cell(ctx.module, name) {
        "__rt_global_ref_cell_decref"
    } else {
        "__rt_decref_mixed"
    };
    abi::emit_call_label(ctx.emitter, release);
}

/// Acquires a durable reference-cell share before replacing a local binding.
pub(super) fn lower_global_ref_cell(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let data = expect_global_name(inst)?;
    let name = ctx.global_name_data(data)?.to_string();
    if crate::superglobals::is_superglobal(&name) {
        return Err(CodegenIrError::unsupported("global_ref_cell requires boxed global storage"));
    }
    let symbol = ir_global_symbol(&name);
    ctx.data.add_comm(symbol.clone(), 8);
    let ready = ctx.next_label("global_ref_cell_ready");
    abi::emit_load_symbol_to_reg(ctx.emitter, abi::int_result_reg(ctx.emitter), &symbol, 0);
    abi::emit_branch_if_int_result_nonzero(ctx.emitter, &ready);
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Void);
    lower_store_shared_global(ctx, &symbol, &PhpType::Mixed)?;
    ctx.emitter.label(&ready);
    abi::emit_load_symbol_to_reg(ctx.emitter, abi::int_result_reg(ctx.emitter), &symbol, 0);
    abi::emit_call_label(ctx.emitter, "__rt_global_ref_cell_incref");
    store_if_result(ctx, inst)
}

/// Stores a global through its shared reference cell, preserving live aliases.
pub(in crate::codegen) fn lower_store_shared_global(
    ctx: &mut FunctionContext<'_>,
    symbol: &str,
    ty: &PhpType,
) -> Result<()> {
    let store_ty = ty.codegen_repr();
    if !matches!(store_ty, PhpType::AssocArray { .. } | PhpType::Mixed) {
        return Err(CodegenIrError::unsupported(format!(
            "global reference-cell store for PHP type {:?}",
            store_ty
        )));
    }
    ctx.data.add_comm(symbol.to_string(), 8);
    let cell_ready = ctx.next_label("web_superglobal_cell_ready");
    abi::emit_push_result_value(ctx.emitter, &store_ty);
    abi::emit_load_symbol_to_reg(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        symbol,
        0,
    );
    abi::emit_branch_if_int_result_nonzero(ctx.emitter, &cell_ready);

    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 16);
    abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
    let kind_reg = abi::secondary_scratch_reg(ctx.emitter);
    let kind_word = match ctx.emitter.target.arch {
        Arch::AArch64 => crate::codegen_support::sentinels::GLOBAL_REF_CELL_HEAP_KIND as i64,
        Arch::X86_64 => crate::codegen_support::sentinels::x86_64_heap_kind_word(
            crate::codegen_support::sentinels::GLOBAL_REF_CELL_HEAP_KIND.into(),
        ) as i64,
    };
    abi::emit_load_int_immediate(ctx.emitter, kind_reg, kind_word);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!(
            "str {}, [{}, #-8]",
            kind_reg,
            abi::int_result_reg(ctx.emitter)
        )),
        Arch::X86_64 => ctx.emitter.instruction(&format!(
            "mov QWORD PTR [{} - 8], {}",
            abi::int_result_reg(ctx.emitter),
            kind_reg
        )),
    }
    abi::emit_store_zero_to_address(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
    abi::emit_store_zero_to_address(ctx.emitter, abi::int_result_reg(ctx.emitter), 8);
    abi::emit_store_reg_to_symbol(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        symbol,
        0,
    );

    ctx.emitter.label(&cell_ready);
    let cell_reg = abi::secondary_scratch_reg(ctx.emitter);
    abi::emit_reg_move(ctx.emitter, cell_reg, abi::int_result_reg(ctx.emitter));
    abi::emit_push_reg(ctx.emitter, cell_reg);
    abi::emit_load_from_address(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        cell_reg,
        0,
    );
    let release_payload = if store_ty == PhpType::Mixed {
        "__rt_decref_mixed"
    } else {
        "__rt_decref_hash"
    };
    abi::emit_call_label(ctx.emitter, release_payload);
    abi::emit_pop_reg(ctx.emitter, cell_reg);
    abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    abi::emit_store_to_address(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        cell_reg,
        0,
    );
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        runtime_value_tag(&store_ty) as i64,
    );
    abi::emit_store_to_address(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        cell_reg,
        8,
    );
    Ok(())
}

/// Loads the pointer-sized payload held by a shared global reference cell.
pub(super) fn load_shared_global_to_result(
    ctx: &mut FunctionContext<'_>,
    symbol: &str,
) {
    ctx.data.add_comm(symbol.to_string(), 8);
    let done = ctx.next_label("load_web_superglobal_done");
    abi::emit_load_symbol_to_reg(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        symbol,
        0,
    );
    abi::emit_branch_if_int_result_zero(ctx.emitter, &done);
    let cell_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_load_from_address(ctx.emitter, cell_reg, cell_reg, 0);
    ctx.emitter.label(&done);
}

/// Lowers a C extern global load into the EIR result slot.
pub(super) fn lower_extern_global_load(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let data = expect_global_name(inst)?;
    let name = ctx.global_name_data(data)?;
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("extern_global_load missing result value"))?;
    let ty = ctx.value_php_type(result)?;
    let symbol = ctx.emitter.target.extern_symbol(name);
    match ty.codegen_repr() {
        PhpType::Bool
        | PhpType::Int
        | PhpType::Resource(_)
        | PhpType::Pointer(_)
        | PhpType::Buffer(_)
        | PhpType::Packed(_)
        | PhpType::Callable => {
            abi::emit_load_extern_symbol_to_reg(
                ctx.emitter,
                abi::int_result_reg(ctx.emitter),
                &symbol,
                0,
            );
        }
        PhpType::Float => {
            abi::emit_load_extern_symbol_to_reg(
                ctx.emitter,
                abi::float_result_reg(ctx.emitter),
                &symbol,
                0,
            );
        }
        PhpType::Str => {
            abi::emit_load_extern_symbol_to_reg(
                ctx.emitter,
                abi::int_result_reg(ctx.emitter),
                &symbol,
                0,
            );
            abi::emit_call_label(ctx.emitter, "__rt_cstr_to_str");
        }
        other => {
            ctx.emitter.comment(&format!(
                "WARNING: unsupported extern global load for ${} with PHP type {:?}",
                name, other
            ));
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers a C extern global store from one SSA operand.
pub(super) fn lower_extern_global_store(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let data = expect_global_name(inst)?;
    let name = ctx.global_name_data(data)?.to_string();
    let value = expect_operand(inst, 0)?;
    let ty = ctx.load_value_to_result(value)?.codegen_repr();
    let symbol = ctx.emitter.target.extern_symbol(&name);
    match ty {
        PhpType::Bool
        | PhpType::Int
        | PhpType::Resource(_)
        | PhpType::Pointer(_)
        | PhpType::Buffer(_)
        | PhpType::Packed(_)
        | PhpType::Callable => {
            abi::emit_store_reg_to_extern_symbol(
                ctx.emitter,
                abi::int_result_reg(ctx.emitter),
                &symbol,
                0,
            );
        }
        PhpType::Float => {
            abi::emit_store_reg_to_extern_symbol(
                ctx.emitter,
                abi::float_result_reg(ctx.emitter),
                &symbol,
                0,
            );
        }
        PhpType::Str => {
            abi::emit_call_label(ctx.emitter, "__rt_str_to_cstr");
            abi::emit_store_reg_to_extern_symbol(
                ctx.emitter,
                abi::int_result_reg(ctx.emitter),
                &symbol,
                0,
            );
        }
        other => {
            ctx.emitter.comment(&format!(
                "WARNING: unsupported extern global store for ${} with PHP type {:?}",
                name, other
            ));
        }
    }
    Ok(())
}

/// Lowers an integer constant into the canonical integer result register and slot.
pub(super) fn lower_const_i64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_i64(inst)?;
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), value);
    store_if_result(ctx, inst)
}

/// Lowers a boolean constant into the canonical integer result register and slot.
pub(super) fn lower_const_bool(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = i64::from(expect_bool(inst)?);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), value);
    store_if_result(ctx, inst)
}

/// Lowers a null constant to the selected one-word or tagged-scalar representation.
pub(super) fn lower_const_null(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.result_php_type.codegen_repr() == PhpType::TaggedScalar {
        crate::codegen::sentinels::emit_tagged_scalar_null(ctx.emitter);
    } else {
        abi::emit_load_int_immediate(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            0x7fff_ffff_ffff_fffe,
        );
    }
    store_if_result(ctx, inst)
}

/// Lowers explicit Mixed boxing for scalar, string, object, and existing Mixed operands.
pub(super) fn lower_mixed_box(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let source_ty = ctx.load_value_to_result(value)?;
    let raw_source_ty = ctx.raw_value_php_type(value)?;
    let box_ty = if matches!(raw_source_ty, PhpType::Resource(_)) {
        raw_source_ty
    } else {
        source_ty
    };
    emit_box_current_value_as_mixed(ctx.emitter, &box_ty);
    store_if_result(ctx, inst)
}

/// Clones a boxed Mixed zval cell so later mutation cannot rewrite an aliased source cell.
pub(super) fn lower_mixed_clone(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    load_value_to_first_int_arg(ctx, value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_clone");
    store_if_result(ctx, inst)
}

/// Lowers an invoker-only by-reference argument marker for descriptor calls.
pub(super) fn lower_invoker_ref_arg(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if let Some(Immediate::GlobalName(data)) = inst.immediate {
        return lower_global_array_ref_marker(ctx, inst, data);
    }
    let slot = expect_local_slot(inst)?;
    let source_ty = ctx.local_php_type(slot)?.codegen_repr();
    let ref_cell_reg = abi::secondary_scratch_reg(ctx.emitter);
    let marker_tag_reg = abi::tertiary_scratch_reg(ctx.emitter);
    let source_tag_reg = abi::symbol_scratch_reg(ctx.emitter);
    ctx.materialize_local_storage_address(slot, ref_cell_reg)?;
    abi::emit_load_int_immediate(
        ctx.emitter,
        marker_tag_reg,
        callable_invoker_args::INVOKER_ARG_REF_CELL_TAG,
    );
    abi::emit_load_int_immediate(
        ctx.emitter,
        source_tag_reg,
        crate::codegen::runtime_value_tag(&source_ty) as i64,
    );
    ctx.emitter.comment("cufa_invoker_ref_cell");
    emit_box_runtime_payload_as_mixed(ctx.emitter, marker_tag_reg, ref_cell_reg, source_tag_reg);
    store_if_result(ctx, inst)
}

/// Lowers an owning array marker for a promoted local reference cell.
pub(super) fn lower_array_local_ref_cell(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let slot = expect_local_slot(inst)?;
    let source_ty = ctx.local_php_type(slot)?.codegen_repr();
    let ref_cell_reg = abi::secondary_scratch_reg(ctx.emitter);
    ctx.materialize_local_storage_address(slot, ref_cell_reg)?;
    abi::emit_reg_move(ctx.emitter, abi::int_result_reg(ctx.emitter), ref_cell_reg);
    abi::emit_call_label(ctx.emitter, "__rt_ref_cell_incref");
    abi::emit_reg_move(ctx.emitter, ref_cell_reg, abi::int_result_reg(ctx.emitter));
    let marker_tag_reg = abi::tertiary_scratch_reg(ctx.emitter);
    let source_tag_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_load_int_immediate(
        ctx.emitter,
        marker_tag_reg,
        callable_invoker_args::ARRAY_LOCAL_REF_CELL_TAG,
    );
    abi::emit_load_int_immediate(
        ctx.emitter,
        source_tag_reg,
        crate::codegen::runtime_value_tag(&source_ty) as i64,
    );
    ctx.emitter.comment("array_local_ref_cell");
    emit_box_runtime_payload_as_mixed(ctx.emitter, marker_tag_reg, ref_cell_reg, source_tag_reg);
    store_if_result(ctx, inst)
}

/// Lowers an owning array-reference marker for a request-global ref-cell.
fn lower_global_array_ref_marker(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    data: crate::ir::DataId,
) -> Result<()> {
    let name = ctx.global_name_data(data)?;
    if !ctx.module.web || !crate::superglobals::is_superglobal(name) {
        return Err(CodegenIrError::unsupported(format!(
            "array reference to non-web global ${name}"
        )));
    }
    let symbol = ir_global_symbol(name);
    ctx.data.add_comm(symbol.clone(), 8);
    abi::emit_load_symbol_to_reg(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        &symbol,
        0,
    );
    abi::emit_call_label(ctx.emitter, "__rt_global_ref_cell_incref");
    let cell_reg = abi::secondary_scratch_reg(ctx.emitter);
    let marker_tag_reg = abi::tertiary_scratch_reg(ctx.emitter);
    let source_tag_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_reg_move(ctx.emitter, cell_reg, abi::int_result_reg(ctx.emitter));
    abi::emit_load_int_immediate(
        ctx.emitter,
        marker_tag_reg,
        callable_invoker_args::ARRAY_GLOBAL_REF_CELL_TAG,
    );
    abi::emit_load_int_immediate(
        ctx.emitter,
        source_tag_reg,
        runtime_value_tag(&crate::superglobals::superglobal_type()) as i64,
    );
    emit_box_runtime_payload_as_mixed(
        ctx.emitter,
        marker_tag_reg,
        cell_reg,
        source_tag_reg,
    );
    store_if_result(ctx, inst)
}
