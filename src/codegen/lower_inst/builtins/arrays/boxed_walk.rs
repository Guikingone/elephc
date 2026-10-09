//! Purpose:
//! Lowers mutating walks over boxed PHP arrays through a storage-neutral callback runtime.
//!
//! Called from:
//! - `lower_array_walk()` and `lower_array_walk_recursive()`.
//!
//! Key details:
//! - The receiver is separated before callback normalization so element references mutate only
//!   the caller-visible array owner.
//! - The runtime consumes one retained callable descriptor and preserves actual key/value tags.

use super::*;

const CALLBACK_STACK_BYTES: usize = 64;

/// Lowers a mutating walk over a concrete Mixed-valued hash.
///
/// The boxed walker mutates a borrowed Mixed cell in place; a concrete hash has no cell, so the
/// hash is boxed for the call and its mutated payload moved back into the receiver afterwards. The
/// hash's own owner is handed to the box first so the walker's promote step sees a unique payload
/// and mutates the receiver's storage instead of a copy-on-write split.
pub(super) fn lower_assoc_array_walk(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    array: ValueId,
    callback: ValueId,
    name: &str,
    recursive: bool,
) -> Result<()> {
    let receiver =
        crate::codegen::lower_inst::receiver_place::ReceiverPlace::resolve(ctx, array)?;
    receiver.require_writable(name)?;
    receiver.prepare_consuming_storeback(ctx, array)?;
    super::sort_dispatch::ensure_unique_hash_sort_source(ctx, array)?;
    abi::emit_reserve_temporary_stack(ctx.emitter, CALLBACK_STACK_BYTES);
    super::boxed_membership::store_borrowed_cell(ctx, callback, 32)?;
    let callback_error = if recursive {
        "array_walk_recursive(): Argument #2 ($callback) must be a valid callback"
    } else {
        "array_walk(): Argument #2 ($callback) must be a valid callback"
    };
    super::boxed_predicates::acquire_callback_descriptor(
        ctx,
        inst,
        callback,
        name,
        callback_error,
    )?;
    let result = abi::int_result_reg(ctx.emitter);
    let arg0 = abi::int_arg_reg_name(ctx.emitter.target, 0);
    let arg1 = abi::int_arg_reg_name(ctx.emitter.target, 1);
    let arg2 = abi::int_arg_reg_name(ctx.emitter.target, 2);
    let arg3 = abi::int_arg_reg_name(ctx.emitter.target, 3);
    abi::emit_push_reg(ctx.emitter, result);                                    // park the owned descriptor at stack offset 0
    let hash_ty = ctx.value_php_type(array)?.codegen_repr();
    ctx.load_value_to_result(array)?;
    crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &hash_ty);     // box the borrowed hash into a cell
    abi::emit_push_reg(ctx.emitter, result);                                    // park the cell: cell at 0, descriptor at 16
    // Hand the hash's remaining owner to the box. The walker promotes its borrowed cell to a hash
    // with a COW split, so a payload the receiver still owned would be copied and the callback's
    // writes lost; a unique payload is mutated in place instead.
    ctx.load_value_to_result(array)?;
    abi::emit_reg_move(ctx.emitter, arg0, result);
    abi::emit_call_label(ctx.emitter, "__rt_decref_hash");
    abi::emit_load_temporary_stack_slot(ctx.emitter, arg0, 16);                 // pass the callable descriptor
    abi::emit_load_temporary_stack_slot(ctx.emitter, arg1, 0);                  // pass the borrowed Mixed cell
    abi::emit_load_int_immediate(ctx.emitter, arg2, i64::from(recursive));
    abi::emit_load_int_immediate(ctx.emitter, arg3, ctx.lexical_class_id());
    abi::emit_call_label(ctx.emitter, "__rt_array_walk_boxed");
    // Detach the mutated hash from the box and republish it as the receiver.
    abi::emit_load_temporary_stack_slot(ctx.emitter, result, 0);                // result = boxed cell
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x9, [x0, #8]");                        // take the mutated hash out of the cell payload
            ctx.emitter.instruction("str xzr, [x0, #8]");                       // detach so releasing the box cannot free it
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov r10, QWORD PTR [rax + 8]");            // take the mutated hash out of the cell payload
            ctx.emitter.instruction("mov QWORD PTR [rax + 8], 0");              // detach so releasing the box cannot free it
        }
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => abi::emit_store_to_sp(ctx.emitter, "x9", 16),
        Arch::X86_64 => abi::emit_store_to_sp(ctx.emitter, "r10", 16),
    }
    abi::emit_load_temporary_stack_slot(ctx.emitter, result, 0);                // release the emptied box
    abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
    abi::emit_load_temporary_stack_slot(ctx.emitter, result, 16);               // republish the mutated hash
    ctx.store_result_value(array)?;
    receiver.store_back_value(ctx, array)?;
    abi::emit_release_temporary_stack(ctx.emitter, CALLBACK_STACK_BYTES + 32);
    store_void_builtin_result(ctx, inst)?;
    Ok(())
}

/// Lowers a boxed `array_walk` variant and returns whether this path handled the operand.
pub(super) fn lower_boxed_array_walk(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    array: ValueId,
    callback: ValueId,
    name: &str,
    recursive: bool,
) -> Result<bool> {
    if ctx.value_php_type(array)?.codegen_repr() != PhpType::Mixed {
        return Ok(false);
    }

    super::boxed_mutation::prepare_boxed_array_receiver(ctx, array, name)?;
    abi::emit_reserve_temporary_stack(ctx.emitter, CALLBACK_STACK_BYTES);
    super::boxed_membership::store_borrowed_cell(ctx, callback, 32)?;
    let callback_error = if recursive {
        "array_walk_recursive(): Argument #2 ($callback) must be a valid callback"
    } else {
        "array_walk(): Argument #2 ($callback) must be a valid callback"
    };
    super::boxed_predicates::acquire_callback_descriptor(
        ctx,
        inst,
        callback,
        name,
        callback_error,
    )?;
    let result = abi::int_result_reg(ctx.emitter);
    abi::emit_reg_move(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 0),
        result,
    );
    ctx.load_value_to_reg(array, abi::int_arg_reg_name(ctx.emitter.target, 1))?;
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 2),
        i64::from(recursive),
    );
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 3),
        ctx.lexical_class_id(),
    );
    abi::emit_call_label(ctx.emitter, "__rt_array_walk_boxed");
    abi::emit_release_temporary_stack(ctx.emitter, CALLBACK_STACK_BYTES);
    store_void_builtin_result(ctx, inst)?;
    Ok(true)
}
