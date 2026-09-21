//! Purpose:
//! Materializes method and by-reference call arguments with writeback.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.
//! - EVERY BY-REFERENCE ARGUMENT NEEDS AN ADDRESS, and there are four sources for one:
//!   the caller local's own storage, an array element's slot, a caller-side stack cell that
//!   is WRITTEN BACK into a scalar local afterwards (a scalar local passed to a `mixed`
//!   by-reference parameter), and — for an argument with no caller variable at all, i.e. an
//!   OMITTED optional by-reference argument — a caller-side stack cell that is simply
//!   discarded. The last two share one pushed cell block, planned before any argument is
//!   staged and released once after the call.
//! - THE DISCARDED CELL USED TO BE A HEAP ALLOCATION THAT NOTHING FREED. `f($x)` against
//!   `f($x, int &$out = null)` leaked 16 bytes PER CALL — unbounded in a loop, and PHP's
//!   documented `while ($info = curl_multi_info_read($mh))` loop is exactly that shape.
//!   Moving it into the existing cell block makes the release automatic.
//! - THE HEAP PATH IS STILL LOAD-BEARING, not a leftover. A CONSTRUCTOR that promotes a
//!   by-reference parameter binds a property which BORROWS the argument's cell for the whole
//!   life of the object, so every constructor call is planned as
//!   [`RefArgCellLifetime::MayOutliveCall`] and deliberately routed to
//!   [`materialize_temporary_ref_arg_cell`]; a caller-stack cell there is a use-after-free.
//!   See that function's own doc comment for both kinds of caller that reach it.

use super::*;

/// Loads method call arguments for lexical `self::`/`parent::` instance calls using local `this`.
pub(super) fn materialize_method_call_args_with_receiver_local_and_refs(
    ctx: &mut FunctionContext<'_>,
    receiver_slot: LocalSlotId,
    receiver_ty: &PhpType,
    operands: &[ValueId],
    param_types: &[PhpType],
    ref_params: &[bool],
    lifetime: RefArgCellLifetime,
) -> Result<CallArgMaterialization> {
    if operands.len() + 1 != param_types.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "lexical instance call materialization received {} operands for {} params",
            operands.len(),
            param_types.len()
        )));
    }
    if ref_params.len() != param_types.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "lexical instance call materialization received {} ref flags for {} params",
            ref_params.len(),
            param_types.len()
        )));
    }
    let visible_param_types = &param_types[1..];
    let visible_ref_params = &ref_params[1..];
    let mut ref_writebacks =
        plan_ref_arg_writebacks(ctx, operands, visible_param_types, visible_ref_params)?;
    let mut ref_temp_cells = plan_ref_arg_temp_cells(
        ctx,
        operands,
        visible_param_types,
        visible_ref_params,
        &ref_writebacks,
        lifetime,
    )?;
    emit_ref_arg_cell_block(ctx, &mut ref_writebacks, &mut ref_temp_cells)?;
    // A boxed temporary staged for a by-value parameter is owned by this call and nothing else
    // releases it, so the method-call paths plan the same temp cleanups the direct-call paths do.
    let cleanup_slots =
        plan_call_arg_temp_cleanups(ctx, operands, visible_param_types, visible_ref_params, &[])?;
    let cleanup_bytes = cleanup_slots.len() * 16;
    if cleanup_bytes > 0 {
        abi::emit_reserve_temporary_stack(ctx.emitter, cleanup_bytes);
    }
    let abi_param_types = abi_param_types_for_refs(param_types, ref_params);
    let assignments =
        abi::build_outgoing_arg_assignments_for_target(ctx.emitter.target, &abi_param_types, 0);
    ctx.load_local_to_result(receiver_slot)?;
    abi::emit_push_result_value(ctx.emitter, receiver_ty);
    let mut arg_temp_bytes = call_arg_temp_slot_size(&abi_param_types[0]);
    for (index, (value, param_ty)) in operands.iter().zip(visible_param_types.iter()).enumerate() {
        if visible_ref_params[index] {
            materialize_ref_arg_address(
                ctx,
                *value,
                index,
                param_ty,
                arg_temp_bytes,
                &ref_writebacks,
                &ref_temp_cells,
                0,
            )?;
            abi::emit_push_result_value(ctx.emitter, &PhpType::Int);
        } else {
            ctx.load_value_to_result(*value)?;
            let source_ty = ctx.raw_value_php_type(*value)?;
            let push_ty = materialize_direct_call_arg_for_param(ctx, &source_ty, param_ty)?;
            if let Some(cleanup) = cleanup_slots
                .iter()
                .find(|cleanup| cleanup.param_index == index)
            {
                save_call_arg_temp_cleanup(ctx, cleanup, arg_temp_bytes);
            }
            abi::emit_push_result_value(ctx.emitter, &push_ty);
        }
        arg_temp_bytes += call_arg_temp_slot_size(&abi_param_types[index + 1]);
    }
    Ok(CallArgMaterialization {
        overflow_bytes: abi::materialize_outgoing_args(ctx.emitter, &assignments),
        ref_writebacks,
        ref_temp_cells,
        cleanup_slots,
        cleanup_bytes,
        borrowed_stack_arg_bytes: 0,
    })
}

/// Loads method call arguments with by-reference parameter support for local operands.
pub(super) fn materialize_method_call_args_with_receiver_reg_and_refs(
    ctx: &mut FunctionContext<'_>,
    receiver_reg: &str,
    receiver_ty: &PhpType,
    operands: &[ValueId],
    param_types: &[PhpType],
    ref_params: &[bool],
    lifetime: RefArgCellLifetime,
) -> Result<CallArgMaterialization> {
    if operands.len() > param_types.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "method call materialization received {} operands for {} params",
            operands.len(),
            param_types.len()
        )));
    }
    if ref_params.len() != param_types.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "method call materialization received {} ref flags for {} params",
            ref_params.len(),
            param_types.len()
        )));
    }
    let ref_writebacks = plan_ref_arg_writebacks(ctx, operands, param_types, ref_params)?;
    if !ref_writebacks.is_empty() {
        return Err(CodegenIrError::unsupported(
            "receiver-register method call with scalar-to-mixed by-reference writebacks",
        ));
    }
    // `lifetime` IS NOT ALWAYS `CallOnly` HERE. `new $cls(...)` with a runtime class string
    // reaches a CONSTRUCTOR through this materializer
    // (`objects::dynamic_mixed_candidates::emit_dynamic_new_mixed_constructor_call`), and a
    // constructor may promote a by-reference parameter into a property that borrows the
    // cell for the object's whole life — so that caller passes `MayOutliveCall` and gets the
    // heap cell, exactly like the non-dynamic `new X()` path.
    let mut ref_temp_cells =
        plan_ref_arg_temp_cells(ctx, operands, param_types, ref_params, &ref_writebacks, lifetime)?;
    // THE RECEIVER IS ALREADY IN A REGISTER, AND THE CELL BLOCK IS ALLOWED TO DESTROY
    // CALLER-SAVED ONES: materializing a cell's value runs `load_value_to_result` and may
    // call runtime helpers, so anything the caller left in a caller-saved register — or in
    // the integer result register — is gone by the time the receiver is staged below.
    //
    // Every caller that can reach a non-empty block therefore hands the receiver over in the
    // reserved CALLEE-SAVED nested-call register (`abi::nested_call_reg`, x19/r12), which
    // `crate::codegen::frame` reserves a save slot for. This check makes that a hard
    // contract instead of a coincidence: a future caller that passes a scratch register and
    // a by-reference argument needing a cell gets a compile-time backend error rather than a
    // constructor entered with the cell's value as `$this`.
    if !ref_temp_cells.is_empty() && receiver_reg != abi::nested_call_reg(ctx.emitter) {
        return Err(CodegenIrError::unsupported(format!(
            "receiver-register method call staging a by-reference cell with the receiver in \
             {receiver_reg} (needs the callee-saved nested-call register)"
        )));
    }
    let mut no_writebacks: Vec<RefArgWriteback> = Vec::new();
    emit_ref_arg_cell_block(ctx, &mut no_writebacks, &mut ref_temp_cells)?;
    let cleanup_slots = plan_call_arg_temp_cleanups(ctx, operands, param_types, ref_params, &[])?;
    let cleanup_bytes = cleanup_slots.len() * 16;
    if cleanup_bytes > 0 {
        abi::emit_reserve_temporary_stack(ctx.emitter, cleanup_bytes);
    }
    let abi_param_types = abi_param_types_for_refs(param_types, ref_params);
    let assignments =
        abi::build_outgoing_arg_assignments_for_target(ctx.emitter.target, &abi_param_types, 0);
    move_reg_to_int_result(ctx, receiver_reg);
    abi::emit_push_result_value(ctx.emitter, receiver_ty);
    let mut arg_temp_bytes = call_arg_temp_slot_size(&abi_param_types[0]);
    for (index, (value, param_ty)) in operands
        .iter()
        .skip(1)
        .zip(param_types.iter().skip(1))
        .enumerate()
    {
        let param_index = index + 1;
        if ref_params[param_index] {
            materialize_ref_arg_address(
                ctx,
                *value,
                param_index,
                &param_types[param_index],
                arg_temp_bytes,
                &ref_writebacks,
                &ref_temp_cells,
                0,
            )?;
            abi::emit_push_result_value(ctx.emitter, &PhpType::Int);
        } else {
            ctx.load_value_to_result(*value)?;
            let source_ty = ctx.raw_value_php_type(*value)?;
            let push_ty = materialize_direct_call_arg_for_param(ctx, &source_ty, param_ty)?;
            if let Some(cleanup) = cleanup_slots
                .iter()
                .find(|cleanup| cleanup.param_index == param_index)
            {
                save_call_arg_temp_cleanup(ctx, cleanup, arg_temp_bytes);
            }
            abi::emit_push_result_value(ctx.emitter, &push_ty);
        }
        arg_temp_bytes += call_arg_temp_slot_size(&abi_param_types[param_index]);
    }
    for param_index in operands.len()..param_types.len() {
        if param_index == 0 {
            return Err(CodegenIrError::unsupported(
                "receiver-register method call with no receiver operand",
            ));
        }
        if ref_params[param_index] {
            materialize_omitted_ref_arg_address(ctx, param_index, arg_temp_bytes, &ref_temp_cells)?;
            abi::emit_push_result_value(ctx.emitter, &PhpType::Int);
            arg_temp_bytes += call_arg_temp_slot_size(&abi_param_types[param_index]);
            continue;
        }
        let param_ty = param_types[param_index].codegen_repr();
        match param_ty {
            PhpType::Mixed => {
                objects::emit_boxed_null(ctx);
                abi::emit_push_result_value(ctx.emitter, &PhpType::Mixed);
            }
            PhpType::TaggedScalar => {
                crate::codegen::sentinels::emit_tagged_scalar_null(ctx.emitter);
                abi::emit_push_result_value(ctx.emitter, &PhpType::TaggedScalar);
            }
            _ => {
                return Err(CodegenIrError::unsupported(format!(
                    "receiver-register method call missing default for ABI type {:?}",
                    param_ty
                )));
            }
        }
        // THE CELL BLOCK SITS BELOW EVERY STAGED ARGUMENT, so a by-reference address staged
        // after this point is `arg_temp_bytes + cell_offset`. The loop used to push without
        // advancing the count because nothing read it afterwards; an omitted by-reference
        // parameter does.
        arg_temp_bytes += call_arg_temp_slot_size(&abi_param_types[param_index]);
    }
    Ok(CallArgMaterialization {
        overflow_bytes: abi::materialize_outgoing_args(ctx.emitter, &assignments),
        ref_writebacks,
        ref_temp_cells,
        cleanup_slots,
        cleanup_bytes,
        borrowed_stack_arg_bytes: 0,
    })
}

/// Converts declared parameter types to the ABI-visible shape for by-reference args.
pub(super) fn abi_param_types_for_refs(param_types: &[PhpType], ref_params: &[bool]) -> Vec<PhpType> {
    param_types
        .iter()
        .zip(ref_params.iter())
        .map(|(ty, is_ref)| {
            if *is_ref {
                PhpType::Int
            } else {
                ty.codegen_repr()
            }
        })
        .collect()
}

/// Returns the temporary stack slot size used by outgoing-argument staging.
pub(super) fn call_arg_temp_slot_size(ty: &PhpType) -> usize {
    if matches!(ty.codegen_repr(), PhpType::Void | PhpType::Never) {
        0
    } else {
        16
    }
}

/// Plans temporary by-reference cells for caller/callee representation boundaries.
pub(super) fn plan_ref_arg_writebacks(
    ctx: &FunctionContext<'_>,
    args: &[ValueId],
    param_types: &[PhpType],
    ref_params: &[bool],
) -> Result<Vec<RefArgWriteback>> {
    let mut writebacks = Vec::new();
    for (param_index, value) in args.iter().enumerate() {
        if !ref_params[param_index] {
            continue;
        }
        let source_ty = ctx.raw_value_php_type(*value)?.codegen_repr();
        // A non-local by-reference argument has no caller variable to update. It is materialized
        // as a throwaway ref cell below; only local sources participate in writeback planning.
        let Ok(source) = local_ref_arg_source(ctx, *value) else {
            continue;
        };
        let cell_ty = param_types[param_index].codegen_repr();
        let kind = match (&source_ty, &cell_ty) {
            (source_ty, PhpType::Mixed) if !matches!(source_ty, PhpType::Mixed | PhpType::Union(_)) => {
                reject_unsupported_mixed_ref_writeback_source(
                    source_ty,
                    &ctx.function.name,
                    source.slot,
                )?;
                RefArgWritebackKind::ConcreteToMixed
            }
            (PhpType::Mixed, PhpType::Array(element))
                if element.codegen_repr() == PhpType::Mixed =>
            {
                RefArgWritebackKind::MixedArrayToRawArray
            }
            _ => continue,
        };
        writebacks.push(RefArgWriteback {
            param_index,
            source_value: *value,
            source_slot: source.slot,
            source_ty,
            cell_ty,
            kind,
            cell_offset: 0,
        });
    }
    Ok(writebacks)
}

/// Rejects Mixed ref-cell writebacks whose concrete caller slot cannot consume one word safely.
pub(super) fn reject_unsupported_mixed_ref_writeback_source(
    source_ty: &PhpType,
    function_name: &str,
    source_slot: LocalSlotId,
) -> Result<()> {
    if matches!(
        source_ty.codegen_repr(),
        PhpType::Int | PhpType::Bool | PhpType::Void | PhpType::Never
    ) {
        return Ok(());
    }
    if matches!(
        source_ty.codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Object(_)
    ) {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "by-reference Mixed parameter writeback to PHP type {:?} in {} for local slot {:?}",
        source_ty, function_name, source_slot
    )))
}

/// Plans caller-side stack cells for by-reference arguments that have NO caller variable
/// behind them — the OMITTED optional by-reference argument, above all.
///
/// The predicates below mirror [`materialize_ref_arg_address`]'s own order exactly: an
/// argument that already has a writeback cell, a local slot, or an array-element address
/// needs no cell of its own. Everything else would otherwise reach the heap fallback, whose
/// allocation nothing frees.
pub(super) fn plan_ref_arg_temp_cells(
    ctx: &FunctionContext<'_>,
    args: &[ValueId],
    param_types: &[PhpType],
    ref_params: &[bool],
    writebacks: &[RefArgWriteback],
    lifetime: RefArgCellLifetime,
) -> Result<Vec<RefArgTempCell>> {
    let mut cells = Vec::new();
    // A callee that may KEEP the reference needs storage that outlives this frame, so it
    // keeps the heap cell (see `RefArgCellLifetime`). Planning a stack cell there would hand
    // a constructor-promoted property a pointer into a frame that is gone by its first use.
    if lifetime == RefArgCellLifetime::MayOutliveCall {
        return Ok(cells);
    }
    for (param_index, value) in args.iter().enumerate() {
        if !ref_params[param_index] {
            continue;
        }
        if writebacks
            .iter()
            .any(|writeback| writeback.param_index == param_index)
        {
            continue;
        }
        if local_ref_arg_source(ctx, *value).is_ok() {
            continue;
        }
        if value_is_array_element_address(ctx, *value)? {
            continue;
        }
        cells.push(RefArgTempCell {
            param_index,
            source_value: Some(*value),
            cell_ty: param_types[param_index].codegen_repr(),
            cell_offset: 0,
        });
    }
    // PARAMETERS THE CALLER NEVER WROTE. Every other materializer asserts one argument per
    // parameter, so this range is empty for them; the receiver-register path is the one that
    // can be short, because `lower_mixed_method_candidate` builds `param_types` from EACH
    // CANDIDATE's own signature and one candidate's third parameter is another's second — no
    // caller-side padding satisfies them all at once.
    //
    // The callee still writes through the pointer, so the omitted position gets a discarded
    // cell exactly like an argument with no caller variable: seeded to null below, released
    // with the rest of the block by `emit_ref_arg_writebacks`.
    for param_index in args.len()..param_types.len() {
        if !ref_params[param_index] {
            continue;
        }
        cells.push(RefArgTempCell {
            param_index,
            source_value: None,
            cell_ty: param_types[param_index].codegen_repr(),
            cell_offset: 0,
        });
    }
    Ok(cells)
}

/// Seeds a discarded cell that stands in for an OMITTED by-reference argument.
///
/// PHP's default for such a parameter is `null` in every case elephc can reach here: a
/// by-reference parameter may not carry a non-null constant default and still be omitted by a
/// caller that means to read the write back. The refusal is kept for representations with no
/// null this runtime can spell, rather than pushing a zero that
/// [`emit_ref_arg_writebacks`] would then hand to a refcount helper.
fn emit_omitted_ref_cell_seed(ctx: &mut FunctionContext<'_>, cell_ty: &PhpType) -> Result<()> {
    match cell_ty.codegen_repr() {
        PhpType::Mixed => {
            objects::emit_boxed_null(ctx);
            Ok(())
        }
        PhpType::TaggedScalar => {
            crate::codegen::sentinels::emit_tagged_scalar_null(ctx.emitter);
            Ok(())
        }
        PhpType::Int | PhpType::Bool | PhpType::False => {
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
            Ok(())
        }
        other => Err(CodegenIrError::unsupported(format!(
            "omitted by-reference parameter cell for ABI type {other:?}"
        ))),
    }
}

/// Loads the address of the discarded cell standing in for an OMITTED by-reference argument.
fn materialize_omitted_ref_arg_address(
    ctx: &mut FunctionContext<'_>,
    param_index: usize,
    arg_temp_bytes: usize,
    temp_cells: &[RefArgTempCell],
) -> Result<()> {
    let cell = temp_cells
        .iter()
        .find(|cell| cell.param_index == param_index)
        .ok_or_else(|| {
            // `plan_ref_arg_temp_cells` plans no cells at all for `MayOutliveCall`, where a
            // caller-stack cell would be a use-after-free. Refusing is the right answer there.
            CodegenIrError::unsupported(
                "receiver-register method call with an unplanned omitted by-reference parameter",
            )
        })?;
    abi::emit_temporary_stack_address(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        arg_temp_bytes + cell.cell_offset,
    );
    Ok(())
}

/// Emits the caller-side by-reference cell block: the Mixed writeback cells first, then the
/// discarded cells, as one contiguous run of 16-byte stack slots.
///
/// Offsets are assigned across the WHOLE block (the last cell pushed sits at the current
/// stack pointer), which is what lets [`materialize_ref_arg_address`] address either kind
/// the same way and [`emit_ref_arg_writebacks`] release them in one step.
pub(super) fn emit_ref_arg_cell_block(
    ctx: &mut FunctionContext<'_>,
    writebacks: &mut [RefArgWriteback],
    temp_cells: &mut [RefArgTempCell],
) -> Result<()> {
    let total = writebacks.len() + temp_cells.len();
    for (index, writeback) in writebacks.iter_mut().enumerate() {
        match writeback.kind {
            RefArgWritebackKind::ConcreteToMixed => {
                ctx.load_value_to_result(writeback.source_value)?;
                emit_box_current_value_as_mixed(ctx.emitter, &writeback.source_ty);
            }
            RefArgWritebackKind::MixedArrayToRawArray => {
                let arg_reg = abi::int_arg_reg_name(ctx.emitter.target, 0);
                ctx.load_value_to_reg(writeback.source_value, arg_reg)?;
                abi::emit_call_label(ctx.emitter, "__rt_mixed_to_owned_hash");
            }
        }
        abi::emit_push_result_value(ctx.emitter, &writeback.cell_ty);
        writeback.cell_offset = (total - index - 1) * 16;
    }
    let pushed = writebacks.len();
    for (index, cell) in temp_cells.iter_mut().enumerate() {
        match cell.source_value {
            Some(source_value) => {
                let source_ty = ctx.load_value_to_result(source_value)?;
                coerce_ref_cell_store_value(ctx, source_value, &source_ty, &cell.cell_ty)?;
            }
            None => emit_omitted_ref_cell_seed(ctx, &cell.cell_ty)?,
        }
        abi::emit_push_result_value(ctx.emitter, &cell.cell_ty);
        // A push writes ONE word for every representation except `Str`/`TaggedScalar`, so
        // the cell's second word is whatever the stack happened to hold. The heap path this
        // replaces zeroed it, and a callee that reads the cell as a two-word value (a
        // string, a tagged scalar) must not see garbage there.
        if !matches!(cell.cell_ty.codegen_repr(), PhpType::Str | PhpType::TaggedScalar) {
            let scratch = abi::symbol_scratch_reg(ctx.emitter);
            abi::emit_temporary_stack_address(ctx.emitter, scratch, 0);
            abi::emit_store_zero_to_address(ctx.emitter, scratch, 8);
        }
        cell.cell_offset = (total - pushed - index - 1) * 16;
    }
    Ok(())
}

/// Loads the address that should be passed for a by-reference argument.
pub(super) fn materialize_ref_arg_address(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    param_index: usize,
    param_ty: &PhpType,
    arg_temp_bytes: usize,
    writebacks: &[RefArgWriteback],
    temp_cells: &[RefArgTempCell],
    ref_cell_base_offset: usize,
) -> Result<()> {
    if let Some(writeback) = writebacks
        .iter()
        .find(|writeback| writeback.param_index == param_index)
    {
        let cell_offset = arg_temp_bytes + ref_cell_base_offset + writeback.cell_offset;
        abi::emit_temporary_stack_address(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            cell_offset,
        );
        return Ok(());
    }
    if local_ref_arg_source(ctx, value).is_ok() {
        return materialize_local_ref_arg_address(ctx, value);
    }
    if value_is_array_element_address(ctx, value)? {
        ctx.load_value_to_reg(value, abi::int_result_reg(ctx.emitter))?;
        return Ok(());
    }
    if let Some(cell) = temp_cells
        .iter()
        .find(|cell| cell.param_index == param_index)
    {
        let cell_offset = arg_temp_bytes + ref_cell_base_offset + cell.cell_offset;
        abi::emit_temporary_stack_address(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            cell_offset,
        );
        return Ok(());
    }
    materialize_temporary_ref_arg_cell(ctx, value, param_ty)
}

/// Allocates a heap ref-cell for a by-reference argument that is not a local variable.
///
/// LOAD-BEARING, NOT DEAD. Two kinds of caller reach it, and only one of them is a leftover:
///
/// - A call whose cells were planned as [`RefArgCellLifetime::MayOutliveCall`] — every
///   CONSTRUCTOR call, static or dynamic. A constructor that promotes a by-reference
///   parameter binds a property that BORROWS this cell for the whole life of the object, so
///   it has to be heap storage that outlives the frame. Nothing frees it, which is a
///   narrower pre-existing defect (one cell per constructed object) that only the object
///   model can fix — but replacing this allocation with a stack cell is a use-after-free,
///   and that is exactly what a "clean up the dead fallback" edit would do.
/// - A call path that plans no cells at all. Those would leak one 16-byte block per call, so
///   every path that stages by-reference arguments today plans them
///   ([`plan_ref_arg_temp_cells`]).
pub(super) fn materialize_temporary_ref_arg_cell(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    param_ty: &PhpType,
) -> Result<()> {
    let source_ty = ctx.load_value_to_result(value)?;
    let target_ty = param_ty.codegen_repr();
    coerce_ref_cell_store_value(ctx, value, &source_ty, &target_ty)?;
    abi::emit_push_result_value(ctx.emitter, &target_ty);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 16);
    abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
    let cell_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    abi::emit_pop_reg(ctx.emitter, cell_reg);
    store_pushed_value_to_ref_cell(ctx, cell_reg, &target_ty);
    move_reg_to_int_result(ctx, cell_reg);
    Ok(())
}

/// Stores the pushed argument value into a freshly allocated by-reference cell.
pub(super) fn store_pushed_value_to_ref_cell(ctx: &mut FunctionContext<'_>, cell_reg: &str, val_ty: &PhpType) {
    let temp_reg = if cell_reg == abi::temp_int_reg(ctx.emitter.target) {
        abi::symbol_scratch_reg(ctx.emitter)
    } else {
        abi::temp_int_reg(ctx.emitter.target)
    };
    match val_ty.codegen_repr() {
        PhpType::Str => {
            let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
            abi::emit_pop_reg_pair(ctx.emitter, ptr_reg, len_reg);
            abi::emit_store_to_address(ctx.emitter, ptr_reg, cell_reg, 0);
            abi::emit_store_to_address(ctx.emitter, len_reg, cell_reg, 8);
        }
        PhpType::TaggedScalar => {
            let tag_reg = crate::codegen::sentinels::tagged_scalar_tag_reg(ctx.emitter);
            abi::emit_pop_reg_pair(ctx.emitter, abi::int_result_reg(ctx.emitter), tag_reg);
            abi::emit_store_to_address(ctx.emitter, abi::int_result_reg(ctx.emitter), cell_reg, 0);
            abi::emit_store_to_address(ctx.emitter, tag_reg, cell_reg, 8);
        }
        PhpType::Float => {
            abi::emit_pop_float_reg(ctx.emitter, abi::float_result_reg(ctx.emitter));
            abi::emit_store_to_address(
                ctx.emitter,
                abi::float_result_reg(ctx.emitter),
                cell_reg,
                0,
            );
        }
        _ => {
            abi::emit_pop_reg(ctx.emitter, temp_reg);
            abi::emit_store_to_address(ctx.emitter, temp_reg, cell_reg, 0);
            abi::emit_store_zero_to_address(ctx.emitter, cell_reg, 8);
        }
    }
}

/// Writes temporary Mixed by-reference cells back into the original caller locals, releases
/// whatever a discarded cell ended up holding, and frees the whole cell block.
///
/// One function for both kinds because they share one pushed block: releasing them
/// separately would need two stack adjustments and two chances to get the order wrong.
pub(super) fn emit_ref_arg_writebacks(
    ctx: &mut FunctionContext<'_>,
    call_args: &CallArgMaterialization,
) -> Result<()> {
    for writeback in &call_args.ref_writebacks {
        abi::emit_load_temporary_stack_slot(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            writeback.cell_offset,
        );
        match writeback.kind {
            RefArgWritebackKind::ConcreteToMixed => {
                if ctx.local_php_type(writeback.source_slot)?.codegen_repr() == PhpType::Mixed {
                    emit_mixed_ref_writeback_to_gradual_local(ctx, writeback)?;
                    continue;
                }
                abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
                abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
                move_reg_to_int_result(ctx, mixed_unbox_low_payload_reg(ctx));
                store_current_scalar_result_to_ref_source(ctx, writeback)?;
                abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
                abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
            }
            RefArgWritebackKind::MixedArrayToRawArray => {
                emit_box_current_owned_value_as_mixed(ctx.emitter, &writeback.cell_ty);
                emit_mixed_ref_writeback_to_gradual_local(ctx, writeback)?;
            }
        }
    }
    for cell in &call_args.ref_temp_cells {
        // A discarded cell has no caller variable to write back to, but a REFCOUNTED one
        // still holds a value the caller owns — the default the caller materialized, or
        // whatever the callee left in its place, which `store_ref_cell` retained on the way
        // in. Releasing it is the same ownership rule the writeback loop above applies to
        // its own cells. `emit_decref_if_refcounted` is the canonical dispatcher and is
        // deliberately a no-op for scalars AND for `Str`, whose ownership is not refcounted
        // in this runtime; a string left in a discarded cell therefore keeps whatever
        // behaviour the surrounding string-return path already has.
        let cell_ty = cell.cell_ty.codegen_repr();
        if !matches!(
            cell_ty,
            PhpType::Mixed
                | PhpType::Union(_)
                | PhpType::Array(_)
                | PhpType::AssocArray { .. }
                | PhpType::Object(_)
                | PhpType::Iterable
                | PhpType::Callable
        ) {
            continue;
        }
        abi::emit_load_temporary_stack_slot(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            cell.cell_offset,
        );
        abi::emit_decref_if_refcounted(ctx.emitter, &cell_ty);
    }
    let block_cells = call_args.ref_writebacks.len() + call_args.ref_temp_cells.len();
    abi::emit_release_temporary_stack(ctx.emitter, block_cells * 16);
    Ok(())
}

/// Publishes a mutated by-reference Mixed cell back into gradual caller storage.
///
/// The callee may replace the cell's runtime value with a different array kind or scalar, so
/// unboxing it according to the caller's pre-call narrowed type would discard that runtime tag.
/// The local takes its own cell reference instead; the temporary argument owner is then released.
fn emit_mixed_ref_writeback_to_gradual_local(
    ctx: &mut FunctionContext<'_>,
    writeback: &RefArgWriteback,
) -> Result<()> {
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_call_label(ctx.emitter, "__rt_incref");
    abi::emit_push_reg(ctx.emitter, result_reg);
    let target_ty = ctx.local_php_type(writeback.source_slot)?.codegen_repr();
    let offset = ctx.local_offset(writeback.source_slot)?;
    super::super::frame::emit_owned_local_cleanup(
        ctx,
        writeback.source_slot,
        offset,
        &target_ty,
    );
    abi::emit_pop_reg(ctx.emitter, result_reg);
    ctx.store_current_result_to_local(writeback.source_slot)?;
    abi::emit_pop_reg(ctx.emitter, result_reg);
    abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
    Ok(())
}

/// Returns the low payload register produced by `__rt_mixed_unbox` on the active target.
pub(super) fn mixed_unbox_low_payload_reg(ctx: &FunctionContext<'_>) -> &'static str {
    match ctx.emitter.target.arch {
        Arch::AArch64 => "x1",
        Arch::X86_64 => "rdi",
    }
}

/// Unboxes a boxed Mixed/Union payload and retains it for an owned concrete heap result.
pub(super) fn emit_unbox_mixed_to_owned_refcounted_result(ctx: &mut FunctionContext<'_>, result_ty: &PhpType) {
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    move_reg_to_int_result(ctx, mixed_unbox_low_payload_reg(ctx));
    abi::emit_incref_if_refcounted(ctx.emitter, result_ty);
}

/// Unboxes a guarded Mixed value into an owned concrete heap representation.
///
/// Flow-sensitive checking proves the value has the requested type before this op is emitted;
/// the runtime helper extracts its payload and this result takes its own reference so retaining
/// stores and later cleanup have a balanced ownership ledger.
pub(super) fn lower_mixed_unbox(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    load_value_to_first_int_arg(ctx, value)?;
    let result_ty = inst.result_php_type.codegen_repr();
    match (&result_ty, &inst.immediate) {
        (PhpType::Object(_), Some(Immediate::Bool(true))) => {
            emit_mixed_tag_unbox_guard(ctx, 6, &|given| {
                format!(
                    "clone(): Argument #1 ($object) must be of type object, {} given",
                    given
                )
            });
        }
        (PhpType::Object(_), Some(Immediate::Data(_))) => {
            emit_mixed_nominal_object_unbox_guard(ctx, inst)?;
        }
        (PhpType::Callable, Some(Immediate::I64(10))) => {
            emit_mixed_tag_unbox_guard(ctx, 10, &|given| {
                format!("Value must be of type callable, {} given", given)
            });
        }
        (PhpType::Array(_), Some(Immediate::I64(4))) => {
            emit_mixed_tag_unbox_guard(ctx, 4, &|given| {
                format!("Only arrays and Traversables can be unpacked, {} given", given)
            });
        }
        (PhpType::Iterable, _) => emit_mixed_iterable_unbox_guard(ctx),
        _ => abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox"),
    }
    move_reg_to_int_result(ctx, mixed_unbox_low_payload_reg(ctx));
    if matches!(
        &result_ty,
        PhpType::Array(element) if element.codegen_repr() == PhpType::Mixed
    ) {
        let result_reg = abi::int_result_reg(ctx.emitter).to_string();
        callable_invoker_args::emit_clone_indexed_array_for_invoker_with_runtime_tag(
            &result_reg,
            ctx.emitter,
        );
    } else {
        abi::emit_incref_if_refcounted(ctx.emitter, &result_ty);
    }
    store_if_result(ctx, inst)
}

/// Unboxes a gradual iterable after validating arrays or Traversable object payloads.
fn emit_mixed_iterable_unbox_guard(ctx: &mut FunctionContext<'_>) {
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let accepted_label = ctx.next_label("mixed_iterable_accepted");
    let object_label = ctx.next_label("mixed_iterable_object");
    let object_accepted_label = ctx.next_label("mixed_iterable_object_accepted");
    let wrong_label = ctx.next_label("mixed_iterable_wrong_type");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #4");                              // runtime tag 4 identifies an indexed array iterable
            ctx.emitter.instruction(&format!("b.eq {}", accepted_label));       // accept indexed array payloads without object checks
            ctx.emitter.instruction("cmp x0, #5");                              // runtime tag 5 identifies an associative array iterable
            ctx.emitter.instruction(&format!("b.eq {}", accepted_label));       // accept associative array payloads without object checks
            ctx.emitter.instruction("cmp x0, #6");                              // runtime tag 6 identifies an object candidate
            ctx.emitter.instruction(&format!("b.eq {}", object_label));         // validate object candidates against Traversable contracts
            ctx.emitter.instruction(&format!("b {}", wrong_label));             // reject scalar and resource payloads at the boundary
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 4");                              // runtime tag 4 identifies an indexed array iterable
            ctx.emitter.instruction(&format!("je {}", accepted_label));         // accept indexed array payloads without object checks
            ctx.emitter.instruction("cmp rax, 5");                              // runtime tag 5 identifies an associative array iterable
            ctx.emitter.instruction(&format!("je {}", accepted_label));         // accept associative array payloads without object checks
            ctx.emitter.instruction("cmp rax, 6");                              // runtime tag 6 identifies an object candidate
            ctx.emitter.instruction(&format!("je {}", object_label));           // validate object candidates against Traversable contracts
            ctx.emitter.instruction(&format!("jmp {}", wrong_label));           // reject scalar and resource payloads at the boundary
        }
    }

    ctx.emitter.label(&object_label);
    let interface_ids = builtins::type_predicates::traversable_interface_ids(ctx);
    if interface_ids.is_empty() {
        abi::emit_jump(ctx.emitter, &wrong_label);
    } else {
        match ctx.emitter.target.arch {
            Arch::AArch64 => abi::emit_push_reg(ctx.emitter, "x1"),
            Arch::X86_64 => abi::emit_push_reg(ctx.emitter, "rdi"),
        }
        for interface_id in interface_ids {
            builtins::type_predicates::emit_saved_object_interface_check(
                ctx,
                interface_id,
                &object_accepted_label,
            );
        }
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                abi::emit_pop_reg(ctx.emitter, "x1");
                abi::emit_load_int_immediate(ctx.emitter, "x0", 6);
            }
            Arch::X86_64 => {
                abi::emit_pop_reg(ctx.emitter, "rdi");
                abi::emit_load_int_immediate(ctx.emitter, "rax", 6);
            }
        }
        abi::emit_jump(ctx.emitter, &wrong_label);
        ctx.emitter.label(&object_accepted_label);
        match ctx.emitter.target.arch {
            Arch::AArch64 => abi::emit_pop_reg(ctx.emitter, "x1"),
            Arch::X86_64 => abi::emit_pop_reg(ctx.emitter, "rdi"),
        }
        abi::emit_jump(ctx.emitter, &accepted_label);
    }

    builtins::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(
        ctx,
        &wrong_label,
        &|given| format!("Value must be of type iterable, {} given", given),
    );
    ctx.emitter.label(&accepted_label);
}

/// Unboxes a dynamic value and raises a catchable type error unless its tag matches.
fn emit_mixed_tag_unbox_guard(
    ctx: &mut FunctionContext<'_>,
    expected_tag: i64,
    message_for: &impl Fn(&str) -> String,
) {
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let accepted_label = ctx.next_label("mixed_unbox_tag_accepted");
    let wrong_label = ctx.next_label("mixed_unbox_wrong_tag");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cmp x0, #{}", expected_tag));     // compare the boxed value's runtime tag with the boundary contract
            ctx.emitter.instruction(&format!("b.eq {}", accepted_label));       // continue only when the runtime representation matches
            ctx.emitter.instruction(&format!("b {}", wrong_label));             // report the dynamic value's concrete runtime type
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp rax, {}", expected_tag));     // compare the boxed value's runtime tag with the boundary contract
            ctx.emitter.instruction(&format!("je {}", accepted_label));         // continue only when the runtime representation matches
            ctx.emitter.instruction(&format!("jmp {}", wrong_label));           // report the dynamic value's concrete runtime type
        }
    }
    builtins::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(
        ctx,
        &wrong_label,
        message_for,
    );
    ctx.emitter.label(&accepted_label);
}

/// Validates a boxed object against a named class or interface before unboxing it.
fn emit_mixed_nominal_object_unbox_guard(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let expected = objects::class_name_immediate(ctx, inst)?.to_string();
    if expected.is_empty() || php_symbol_key(&expected) == "object" {
        emit_mixed_tag_unbox_guard(ctx, 6, &|given| {
            format!("Value must be of type object, {} given", given)
        });
        return Ok(());
    }
    let Some((target_id, target_kind)) = objects::classify_named_target(ctx, &expected) else {
        return Err(CodegenIrError::invalid_module(format!(
            "missing runtime type metadata for gradual boundary {:?}",
            expected
        )));
    };
    let source_reg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    abi::emit_push_reg(ctx.emitter, source_reg);
    objects::emit_match_call(ctx, target_id, target_kind, "__rt_mixed_instanceof");
    let accepted_label = ctx.next_label("mixed_unbox_nominal_accepted");
    let wrong_label = ctx.next_label("mixed_unbox_nominal_wrong");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbnz x0, {}", accepted_label));   // a true matcher result satisfies the named boundary
            ctx.emitter.instruction(&format!("b {}", wrong_label));             // reject a scalar or unrelated object payload
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // did the boxed object satisfy the named boundary?
            ctx.emitter.instruction(&format!("jne {}", accepted_label));        // continue when class or interface matching succeeded
            ctx.emitter.instruction(&format!("jmp {}", wrong_label));           // reject a scalar or unrelated object payload
        }
    }
    ctx.emitter.label(&wrong_label);
    abi::emit_pop_reg(ctx.emitter, source_reg);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let type_error_label = ctx.next_label("mixed_unbox_nominal_type_error");
    builtins::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(
        ctx,
        &type_error_label,
        &|given| format!("Value must be of type {}, {} given", expected, given),
    );
    ctx.emitter.label(&accepted_label);
    abi::emit_pop_reg(ctx.emitter, source_reg);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    Ok(())
}

/// Stores an unboxed scalar Mixed payload back through the original by-reference source.
pub(super) fn store_current_scalar_result_to_ref_source(
    ctx: &mut FunctionContext<'_>,
    writeback: &RefArgWriteback,
) -> Result<()> {
    if matches!(
        writeback.source_ty.codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Object(_)
    ) {
        return emit_heap_ref_writeback_store(ctx, writeback);
    }
    ctx.store_current_result_to_local(writeback.source_slot)
}

/// Retains a heap payload before releasing the previous caller-slot owner, then stores it.
///
/// Retaining first is required for alias-safe replacement: the old container graph may already
/// reach the incoming payload, so releasing it first could destroy the value being assigned.
fn emit_heap_ref_writeback_store(
    ctx: &mut FunctionContext<'_>,
    writeback: &RefArgWriteback,
) -> Result<()> {
    let result_reg = abi::int_result_reg(ctx.emitter);
    let slot_ty = ctx.local_php_type(writeback.source_slot)?.codegen_repr();
    let offset = ctx.local_offset(writeback.source_slot)?;
    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_call_label(ctx.emitter, "__rt_incref");
    super::super::frame::emit_owned_local_cleanup(
        ctx,
        writeback.source_slot,
        offset,
        &slot_ty,
    );
    abi::emit_pop_reg(ctx.emitter, result_reg);
    ctx.store_current_result_to_local(writeback.source_slot)
}

/// Loads a local variable's address for a by-reference method-call argument.
pub(super) fn materialize_local_ref_arg_address(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    let source = local_ref_arg_source(ctx, value)?;
    ctx.materialize_local_storage_address(source.slot, abi::int_result_reg(ctx.emitter))
}

/// Returns true when a value already holds a direct pointer to an array element slot.
pub(super) fn value_is_array_element_address(ctx: &FunctionContext<'_>, value: ValueId) -> Result<bool> {
    let Some(value_ref) = ctx.function.value(value) else {
        return Err(CodegenIrError::missing_entry("value", value.as_raw()));
    };
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Ok(false);
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    Ok(inst_ref.op == Op::ArrayElemAddr)
}

/// Resolves the PHP value type stored behind an indexed-array element address.
pub(super) fn array_element_address_value_type(
    ctx: &FunctionContext<'_>,
    value: ValueId,
) -> Result<Option<PhpType>> {
    let Some(value_ref) = ctx.function.value(value) else {
        return Err(CodegenIrError::missing_entry("value", value.as_raw()));
    };
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Ok(None);
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    if inst_ref.op != Op::ArrayElemAddr {
        return Ok(None);
    }
    let array = inst_ref
        .operands
        .first()
        .copied()
        .ok_or_else(|| CodegenIrError::invalid_module("array_elem_addr missing array operand"))?;
    let PhpType::Array(element) = ctx.value_php_type(array)?.codegen_repr() else {
        return Err(CodegenIrError::invalid_module(
            "array_elem_addr source does not use indexed-array storage",
        ));
    };
    Ok(Some(element.codegen_repr()))
}

/// Describes a local operand used as a by-reference call argument.
struct LocalRefArgSource {
    slot: LocalSlotId,
}

/// Resolves an EIR value back to a local slot and whether it already stores a ref-cell pointer.
fn local_ref_arg_source(ctx: &FunctionContext<'_>, value: ValueId) -> Result<LocalRefArgSource> {
    let Some(value_ref) = ctx.function.value(value) else {
        return Err(CodegenIrError::missing_entry("value", value.as_raw()));
    };
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Err(CodegenIrError::unsupported(
            "by-reference method call argument from non-local value",
        ));
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    match inst_ref.op {
        Op::LoadLocal | Op::LoadRefCell => {}
        _ => {
            return Err(CodegenIrError::unsupported(format!(
                "by-reference method call argument from opcode {}",
                inst_ref.op.name()
            )))
        }
    };
    let Some(Immediate::LocalSlot(slot)) = inst_ref.immediate else {
        return Err(CodegenIrError::invalid_module(
            "by-reference load argument has no local slot",
        ));
    };
    Ok(LocalRefArgSource { slot })
}

/// Resolves an EIR value back to a `load_local` source slot for by-reference calls.
pub(super) fn local_slot_for_loaded_value(ctx: &FunctionContext<'_>, value: ValueId) -> Result<LocalSlotId> {
    local_ref_arg_source(ctx, value).map(|source| source.slot)
}

/// Returns true when a local slot stores a ref-cell pointer instead of a raw value.
pub(super) fn local_slot_stores_ref_cell_pointer(ctx: &FunctionContext<'_>, slot: LocalSlotId) -> bool {
    ctx.local_stores_ref_cell_pointer(slot)
}

/// Moves a scratch integer register into the canonical integer result register.
pub(super) fn move_reg_to_int_result(ctx: &mut FunctionContext<'_>, source_reg: &str) {
    let result_reg = abi::int_result_reg(ctx.emitter);
    if source_reg == result_reg {
        return;
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("mov {}, {}", result_reg, source_reg)); // move the unboxed receiver pointer into the normal argument staging register
        }
        Arch::X86_64 => {
            ctx.emitter
                .instruction(&format!("mov {}, {}", result_reg, source_reg)); // move the unboxed receiver pointer into the normal argument staging register
        }
    }
}
