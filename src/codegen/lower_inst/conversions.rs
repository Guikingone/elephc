//! Purpose:
//! Lowers scalar EIR conversion opcodes, including explicit PHP casts.
//! Bridges direct coercion opcodes and `Cast` immediates to existing runtime helpers.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()`.
//!
//! Key details:
//! - Concrete scalar casts stay inline; Mixed numeric casts delegate to boxed runtime helpers.
//! - String numeric parsing delegates to shared runtime routines.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::ir::{Immediate, Instruction, IrHeapKind, IrType, ValueId};
use crate::names::{label_fragment, method_symbol};
use crate::types::PhpType;

use super::super::context::FunctionContext;
use super::{
    direct_call_stack_pad_bytes, emit_dynamic_instance_method_call,
    emit_mixed_method_class_dispatch, expect_operand, load_value_to_first_int_arg,
    lower_runtime_object_method_call, materialize_method_call_args_with_receiver_reg_and_refs,
    mixed_method_candidates, predicates, store_if_result, strings,
};
use crate::codegen::{CodegenIrError, Result};

/// Lowers a string-to-integer conversion through PHP string cast rules.
pub(super) fn lower_str_to_int(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let ty = ctx.load_value_to_result(value)?;
    if ty != PhpType::Str {
        return Err(CodegenIrError::unsupported(format!(
            "{} for PHP type {:?}",
            inst.op.name(),
            ty
        )));
    }
    abi::emit_call_label(ctx.emitter, "__rt_str_to_int");
    store_if_result(ctx, inst)
}

/// Lowers a string-to-float conversion through PHP numeric string parsing.
pub(super) fn lower_str_to_float(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let ty = ctx.load_value_to_result(value)?;
    if ty != PhpType::Str {
        return Err(CodegenIrError::unsupported(format!(
            "{} for PHP type {:?}",
            inst.op.name(),
            ty
        )));
    }
    abi::emit_call_label(ctx.emitter, "__rt_str_to_number");
    store_if_result(ctx, inst)
}

/// Lowers explicit scalar casts based on the target storage immediate and result PHP type.
pub(super) fn lower_cast(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    match expect_cast_target(inst)? {
        IrType::I64 if inst.result_php_type == PhpType::Bool => predicates::lower_is_truthy(ctx, inst),
        IrType::I64 => lower_cast_to_int(ctx, inst),
        IrType::F64 => lower_cast_to_float(ctx, inst),
        IrType::Str => lower_cast_to_string(ctx, inst),
        IrType::Heap(IrHeapKind::Array) => lower_cast_to_array(ctx, inst),
        IrType::Heap(IrHeapKind::Hash) => lower_hash_cast(ctx, inst),
        IrType::Heap(IrHeapKind::Object) => lower_cast_to_object(ctx, inst),
        IrType::Heap(IrHeapKind::Mixed) if inst.result_php_type == PhpType::Mixed => {
            lower_mixed_array_cast(ctx, inst)
        }
        target => Err(CodegenIrError::unsupported(format!(
            "cast to EIR type {:?}",
            target
        ))),
    }
}

/// Routes a hash-producing cast to the projection its EIR producer asked for.
///
/// Two unrelated lowerings emit `Cast` with a `Heap(Hash)` target and an object operand, and
/// they are NOT the same projection. `(array) $obj` mangles private and protected property
/// names (`\0Class\0prop`, `\0*\0prop`); an in-scope `foreach ($this as $k => $v)` exposes the
/// very same properties with BARE names. `ir_lower` keeps them apart through the declared
/// result key type: the explicit cast declares `AssocArray { key: Str, .. }`
/// (`expr::cast_php_type`), the `foreach` conversion `AssocArray { key: Mixed, .. }`
/// (`stmt::typed_foreach`).
fn lower_hash_cast(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if matches!(
        &inst.result_php_type,
        PhpType::AssocArray { key, .. } if key.codegen_repr() == PhpType::Mixed
    ) {
        return lower_object_to_foreach_array(ctx, inst);
    }
    super::builtins::types::lower_object_array_cast(ctx, inst)
}

/// Converts a concrete object to a property hash with bare names for in-scope `foreach`.
fn lower_object_to_foreach_array(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let PhpType::Object(_) = ctx.value_php_type(value)?.codegen_repr() else {
        return Err(CodegenIrError::unsupported(format!(
            "object foreach conversion for PHP type {:?}",
            ctx.value_php_type(value)?.codegen_repr()
        )));
    };
    load_value_to_first_int_arg(ctx, value)?;
    abi::emit_call_label(ctx.emitter, "__rt_object_to_foreach_array");
    store_if_result(ctx, inst)
}

/// Lowers an associative-array cast to a fresh hash-backed stdClass instance.
fn lower_cast_to_object(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let source_type = ctx.value_php_type(value)?.codegen_repr();
    match source_type {
        PhpType::AssocArray { .. } => {
            let first_arg = abi::int_arg_reg_name(ctx.emitter.target, 0);
            ctx.load_value_to_reg(value, first_arg)?;
            abi::emit_call_label(ctx.emitter, "__rt_hash_clone_shallow");
        }
        PhpType::Array(_) => lower_array_storage_to_owned_hash(ctx, value)?,
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "object cast for PHP type {:?}",
                other
            )))
        }
    }
    let result_reg = abi::int_result_reg(ctx.emitter);
    let first_arg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    if first_arg != result_reg {
        abi::emit_reg_move(ctx.emitter, first_arg, result_reg);
    }
    abi::emit_call_label(ctx.emitter, "__rt_hash_to_mixed");
    if first_arg != result_reg {
        abi::emit_reg_move(ctx.emitter, first_arg, result_reg);
    }
    abi::emit_call_label(ctx.emitter, "__rt_stdclass_from_hash");
    store_if_result(ctx, inst)
}

/// Produces an owned hash from storage that may already be a runtime-promoted hash.
fn lower_array_storage_to_owned_hash(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let hash = ctx.next_label("object_cast_array_hash");
            let done = ctx.next_label("object_cast_array_done");
            ctx.load_value_to_reg(value, "x0")?;
            abi::emit_push_reg(ctx.emitter, "x0");
            abi::emit_call_label(ctx.emitter, "__rt_heap_kind");
            ctx.emitter.instruction("cmp x0, #3");                              // distinguish a dynamically promoted hash from indexed storage
            ctx.emitter.instruction(&format!("b.eq {}", hash));                 // clone an existing hash so the stdClass remains isolated
            abi::emit_pop_reg(ctx.emitter, "x0");
            abi::emit_call_label(ctx.emitter, "__rt_array_to_hash");           // convert indexed storage into a fresh owned hash
            ctx.emitter.instruction(&format!("b {}", done));
            ctx.emitter.label(&hash);
            abi::emit_pop_reg(ctx.emitter, "x0");
            abi::emit_call_label(ctx.emitter, "__rt_hash_clone_shallow");      // duplicate promoted associative storage for object value semantics
            ctx.emitter.label(&done);
        }
        Arch::X86_64 => {
            let hash = ctx.next_label("object_cast_array_hash");
            let done = ctx.next_label("object_cast_array_done");
            ctx.load_value_to_reg(value, "rax")?;
            abi::emit_push_reg(ctx.emitter, "rax");
            abi::emit_call_label(ctx.emitter, "__rt_heap_kind");
            ctx.emitter.instruction("cmp rax, 3");                              // distinguish a dynamically promoted hash from indexed storage
            ctx.emitter.instruction(&format!("je {}", hash));                   // clone an existing hash so the stdClass remains isolated
            abi::emit_pop_reg(ctx.emitter, "rdi");
            abi::emit_call_label(ctx.emitter, "__rt_array_to_hash");           // convert indexed storage into a fresh owned hash
            ctx.emitter.instruction(&format!("jmp {}", done));
            ctx.emitter.label(&hash);
            abi::emit_pop_reg(ctx.emitter, "rdi");
            abi::emit_call_label(ctx.emitter, "__rt_hash_clone_shallow");      // duplicate promoted associative storage for object value semantics
            ctx.emitter.label(&done);
        }
    }
    Ok(())
}

/// Lowers PHP's `(array)` cast from a boxed runtime value to owned Mixed-element storage.
fn lower_cast_to_array(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    match ctx.value_php_type(value)?.codegen_repr() {
        PhpType::Mixed | PhpType::Union(_) => {
            load_value_to_first_int_arg(ctx, value)?;
            abi::emit_call_label(ctx.emitter, "__rt_array_from_mixed");
            store_if_result(ctx, inst)
        }
        PhpType::Array(ref element) if element.codegen_repr() == PhpType::Mixed => {
            ctx.load_value_to_result(value)?;
            store_if_result(ctx, inst)
        }
        PhpType::Array(_) => super::arrays::lower_array_to_mixed(ctx, inst),
        PhpType::Object(_) => {
            load_value_to_first_int_arg(ctx, value)?;
            abi::emit_call_label(ctx.emitter, "__rt_object_to_array");
            store_if_result(ctx, inst)
        }
        other => Err(CodegenIrError::unsupported(format!(
            "array cast for PHP type {:?}",
            other
        ))),
    }
}

/// Lowers a runtime-typed PHP array cast through the tag-dispatch helper.
fn lower_mixed_array_cast(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_array");
    store_if_result(ctx, inst)
}

/// Lowers an explicit cast to PHP int for concrete scalar operands.
fn lower_cast_to_int(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    if ctx.value_ir_type(value)? == IrType::TaggedScalar {
        ctx.load_value_to_result(value)?;
        crate::codegen::sentinels::emit_tagged_scalar_to_int_null_as_zero(ctx.emitter);
        return store_if_result(ctx, inst);
    }
    let raw_ty = ctx.raw_value_php_type(value)?;
    if matches!(raw_ty, PhpType::Resource(_)) {
        ctx.load_value_to_result(value)?;
        emit_resource_display_id_to_int(ctx);
        return store_if_result(ctx, inst);
    }
    match raw_ty.codegen_repr() {
        PhpType::Int | PhpType::Bool => {
            ctx.load_value_to_result(value)?;
        }
        PhpType::TaggedScalar => {
            ctx.load_value_to_result(value)?;
            crate::codegen::sentinels::emit_tagged_scalar_to_int_null_as_zero(ctx.emitter);
        }
        PhpType::Void | PhpType::Never => {
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
        }
        PhpType::Float => {
            ctx.load_value_to_result(value)?;
            abi::emit_float_result_to_int_result(ctx.emitter);
        }
        PhpType::Str => {
            ctx.load_value_to_result(value)?;
            abi::emit_call_label(ctx.emitter, "__rt_str_to_int");
        }
        PhpType::Mixed | PhpType::Union(_) => {
            load_value_to_first_int_arg(ctx, value)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_int");
        }
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => {
            predicates::emit_array_truthiness(ctx, value)?;
        }
        PhpType::Object(class_name) => {
            emit_object_to_int_warning(ctx, &class_name);
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 1);
        }
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "int cast for PHP type {:?}",
                other
            )))
        }
    }
    store_if_result(ctx, inst)
}

/// Emits PHP's warning for an object-to-integer cast; PHP still produces integer `1` afterward.
fn emit_object_to_int_warning(ctx: &mut FunctionContext<'_>, class_name: &str) {
    let message = format!(
        "Warning: Object of class {} could not be converted to int\n",
        class_name.trim_start_matches('\\')
    );
    let (label, len) = ctx.data.add_string(message.as_bytes());
    crate::codegen::emit_write_literal_stderr(ctx.emitter, &label, len);
}

/// Lowers an explicit cast to PHP float for concrete scalar operands.
fn lower_cast_to_float(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    if ctx.value_ir_type(value)? == IrType::TaggedScalar {
        ctx.load_value_to_result(value)?;
        crate::codegen::sentinels::emit_tagged_scalar_to_int_null_as_zero(ctx.emitter);
        abi::emit_int_result_to_float_result(ctx.emitter);
        return store_if_result(ctx, inst);
    }
    let raw_ty = ctx.raw_value_php_type(value)?;
    if matches!(raw_ty, PhpType::Resource(_)) {
        ctx.load_value_to_result(value)?;
        emit_resource_display_id_to_int(ctx);
        abi::emit_int_result_to_float_result(ctx.emitter);
        return store_if_result(ctx, inst);
    }
    match raw_ty.codegen_repr() {
        PhpType::Float => {
            ctx.load_value_to_result(value)?;
        }
        PhpType::Int | PhpType::Bool => {
            ctx.load_value_to_result(value)?;
            abi::emit_int_result_to_float_result(ctx.emitter);
        }
        PhpType::TaggedScalar => {
            ctx.load_value_to_result(value)?;
            crate::codegen::sentinels::emit_tagged_scalar_to_int_null_as_zero(ctx.emitter);
            abi::emit_int_result_to_float_result(ctx.emitter);
        }
        PhpType::Void | PhpType::Never => {
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
            abi::emit_int_result_to_float_result(ctx.emitter);
        }
        PhpType::Str => {
            ctx.load_value_to_result(value)?;
            abi::emit_call_label(ctx.emitter, "__rt_str_to_number");
        }
        PhpType::Mixed | PhpType::Union(_) => {
            load_value_to_first_int_arg(ctx, value)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_float");
        }
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => {
            predicates::emit_array_truthiness(ctx, value)?;
            abi::emit_int_result_to_float_result(ctx.emitter);
        }
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "float cast for PHP type {:?}",
                other
            )))
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers an explicit cast to PHP string for concrete scalar operands.
pub(super) fn lower_cast_to_string(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let raw_ty = ctx.raw_value_php_type(value)?;
    if matches!(raw_ty, PhpType::Resource(_)) {
        ctx.load_value_to_result(value)?;
        abi::emit_call_label(ctx.emitter, "__rt_resource_to_string");
        return store_if_result(ctx, inst);
    }
    match raw_ty.codegen_repr() {
        PhpType::Str => {
            ctx.load_value_to_result(value)?;
            store_if_result(ctx, inst)
        }
        PhpType::Float => strings::lower_float_to_string(ctx, inst),
        PhpType::Int | PhpType::Bool | PhpType::Void | PhpType::Never | PhpType::TaggedScalar => {
            strings::lower_int_like_to_string(ctx, inst)
        }
        PhpType::Mixed | PhpType::Union(_) => {
            emit_mixed_string_context_result(ctx, value)?;
            store_if_result(ctx, inst)
        }
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable => {
            lower_array_like_to_string(ctx, inst)
        }
        PhpType::Object(class_name) => lower_object_to_string(ctx, inst, &class_name),
        other => Err(CodegenIrError::unsupported(format!(
            "string cast for PHP type {:?}",
            other
        ))),
    }
}

/// Leaves a string result for a boxed Mixed value, dispatching objects through `__toString()`.
pub(super) fn emit_mixed_string_context_result(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
) -> Result<()> {
    emit_mixed_string_context(ctx, value, MixedStringContextMode::Result)
}

/// Writes a boxed Mixed value to stdout, dispatching objects through `__toString()`.
pub(super) fn emit_mixed_string_context_stdout(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
) -> Result<()> {
    emit_mixed_string_context(ctx, value, MixedStringContextMode::Stdout)
}

/// Describes whether a Mixed string context should leave a string result or write it.
pub(in crate::codegen) enum MixedStringContextMode {
    Result,
    Stdout,
}

/// Handles PHP string contexts for boxed Mixed values with an object-aware branch.
///
/// The dispatch below carries one arm per class publishing `__toString`, so a program
/// with several string contexts used to emit the same ladder once per site. When the
/// module shares it, the site keeps only the load and calls the shared helper; the
/// helper's own body takes the inline path, which is what terminates the recursion.
fn emit_mixed_string_context(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    mode: MixedStringContextMode,
) -> Result<()> {
    ctx.load_value_to_result(value)?;
    if let Some(label) = crate::codegen::shared_mixed_string::shared_ladder_label(ctx, &mode) {
        abi::emit_call_label(ctx.emitter, label);
        return Ok(());
    }
    emit_mixed_string_dispatch_from_result(ctx, value, mode)
}

/// Emits the object-aware string dispatch for a boxed Mixed already in the result register.
///
/// Split out so the shared helper can emit the SAME arms rather than a reimplementation of
/// them: what moves is where the ladder lives, not what it does.
pub(in crate::codegen) fn emit_mixed_string_dispatch_from_result(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    mode: MixedStringContextMode,
) -> Result<()> {
    let candidates = mixed_method_candidates(ctx, "__toString", 1)?;
    let receiver_reg = abi::nested_call_reg(ctx.emitter);
    let object_label = ctx.next_label("mixed_string_object");
    let no_match_label = ctx.next_label("mixed_string_no_match");
    let done_label = ctx.next_label("mixed_string_done");
    let match_labels = candidates
        .iter()
        .map(|candidate| {
            ctx.next_label(&format!(
                "mixed_string_{}",
                label_fragment(&candidate.class_name)
            ))
        })
        .collect::<Vec<_>>();

    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    emit_branch_if_unboxed_mixed_object(ctx, &object_label);
    emit_mixed_string_scalar_fallback(ctx, &mode)?;
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&object_label);
    discard_preserved_mixed_pointer(ctx);
    move_unboxed_mixed_object_payload(ctx, receiver_reg);
    emit_mixed_method_class_dispatch(
        ctx,
        receiver_reg,
        &candidates,
        &match_labels,
        &no_match_label,
    );

    for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        let return_ty = emit_mixed_tostring_candidate_call(ctx, value, receiver_reg, candidate)?;
        coerce_tostring_return_to_string_result(ctx, &return_ty)?;
        if matches!(mode, MixedStringContextMode::Stdout) {
            abi::emit_write_stdout(ctx.emitter, &PhpType::Str);
        }
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&no_match_label);
    super::output_values::emit_dynamic_object_to_string(ctx, receiver_reg);
    if matches!(mode, MixedStringContextMode::Stdout) {
        abi::emit_write_stdout(ctx.emitter, &PhpType::Str);
    }
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Branches to the object path when `__rt_mixed_unbox` returned an object tag.
fn emit_branch_if_unboxed_mixed_object(ctx: &mut FunctionContext<'_>, object_label: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");                              // check whether the boxed Mixed value contains an object
            ctx.emitter.instruction(&format!("b.eq {}", object_label));         // dispatch object string contexts through __toString
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");                              // check whether the boxed Mixed value contains an object
            ctx.emitter.instruction(&format!("je {}", object_label));           // dispatch object string contexts through __toString
        }
    }
}

/// Runs the existing scalar Mixed string behavior after restoring the original box.
fn emit_mixed_string_scalar_fallback(
    ctx: &mut FunctionContext<'_>,
    mode: &MixedStringContextMode,
) -> Result<()> {
    abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match mode {
        MixedStringContextMode::Result => {
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_string");
        }
        MixedStringContextMode::Stdout => {
            abi::emit_call_label(ctx.emitter, "__rt_mixed_write_stdout");
        }
    }
    Ok(())
}

/// Discards the saved boxed Mixed pointer once the object branch no longer needs it.
fn discard_preserved_mixed_pointer(ctx: &mut FunctionContext<'_>) {
    abi::emit_pop_reg(ctx.emitter, abi::temp_int_reg(ctx.emitter.target));
}

/// Moves the unboxed object payload into the callee-saved receiver dispatch register.
fn move_unboxed_mixed_object_payload(ctx: &mut FunctionContext<'_>, receiver_reg: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov {}, x1", receiver_reg));      // preserve the unboxed object pointer for __toString dispatch
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov {}, rdi", receiver_reg));     // preserve the unboxed object pointer for __toString dispatch
        }
    }
}

/// Emits one concrete `__toString()` candidate call for a boxed Mixed object.
fn emit_mixed_tostring_candidate_call(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    receiver_reg: &str,
    candidate: &super::MixedMethodCandidate,
) -> Result<PhpType> {
    let receiver_ty = PhpType::Object(candidate.class_name.clone());
    let mut param_types = Vec::with_capacity(candidate.target.params.len() + 1);
    param_types.push(receiver_ty.clone());
    param_types.extend(candidate.target.params.iter().map(|param| param.codegen_repr()));
    let mut ref_params = Vec::with_capacity(candidate.target.ref_params.len() + 1);
    ref_params.push(false);
    ref_params.extend(candidate.target.ref_params.iter().copied());
    let operands = [value];
    let call_args = materialize_method_call_args_with_receiver_reg_and_refs(
        ctx,
        receiver_reg,
        &receiver_ty,
        &operands,
        &param_types,
        &ref_params,
    )?;
    let caller_stack_pad_bytes = direct_call_stack_pad_bytes(ctx, call_args.overflow_bytes);
    abi::emit_reserve_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    if let Some(slot) = candidate.target.dynamic_slot {
        emit_dynamic_instance_method_call(ctx, slot);
    } else {
        abi::emit_call_label(
            ctx.emitter,
            &method_symbol(&candidate.target.impl_class, &candidate.target.method_key),
        );
    }
    abi::emit_release_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    abi::emit_release_temporary_stack(ctx.emitter, call_args.overflow_bytes);
    Ok(candidate.target.return_ty.clone())
}

/// Normalizes a `__toString()` return into a string result pair.
fn coerce_tostring_return_to_string_result(
    ctx: &mut FunctionContext<'_>,
    return_ty: &PhpType,
) -> Result<()> {
    match return_ty.codegen_repr() {
        PhpType::Str => Ok(()),
        PhpType::Mixed | PhpType::Union(_) => {
            super::cast_loaded_mixed_pointer_to_result(ctx, &PhpType::Str)
        }
        other => Err(CodegenIrError::unsupported(format!(
            "__toString return value for PHP type {:?}",
            other
        ))),
    }
}

/// Lowers an object string cast through `__toString()`, statically bound where the class has one.
fn lower_object_to_string(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_name: &str,
) -> Result<()> {
    let normalized = class_name.trim_start_matches('\\');
    if interface_has_tostring(ctx, normalized) {
        // The receiver may be an object the INTERPRETER built, whose class id is `stdClass` and
        // therefore matches no interface table. The runtime resolver reaches its real
        // `__toString()`; the scan's defensive zero would hand back a null string pointer.
        let value = expect_operand(inst, 0)?;
        return super::method_intrinsics::lower_interface_method_call_with_miss(
            ctx,
            inst,
            normalized,
            "__toString",
            super::iterators::InterfaceDispatchMiss::DynamicToString(value),
        );
    }
    if object_class_has_tostring(ctx, normalized) {
        return lower_runtime_object_method_call(ctx, inst, normalized, "__toString");
    }
    // Everything left is a class whose `__toString` cannot be bound HERE, and neither remaining
    // case is a compile-time refusal:
    //   - `normalized` is empty, which is how the DECLARED type `object` is spelled. A value
    //     typed `object` has a concrete class at run time and PHP calls its `__toString`; the
    //     static fatal below used to fire for all of them, printing an empty class name.
    //   - the class is named but publishes no `__toString`. A SUBCLASS of it may still publish
    //     one, and even when none does PHP throws a CATCHABLE `Error` rather than dying.
    // The runtime resolver settles both from the object itself.
    let value = expect_operand(inst, 0)?;
    super::output_values::emit_value_dynamic_object_to_string(ctx, value)?;
    store_if_result(ctx, inst)
}

/// Returns true when interface metadata exposes a string-returning `__toString()` contract.
fn interface_has_tostring(ctx: &FunctionContext<'_>, interface_name: &str) -> bool {
    ctx.module
        .interface_infos
        .get(interface_name)
        .and_then(|interface| interface.methods.get("__tostring"))
        .is_some_and(|signature| signature.return_type.codegen_repr() == PhpType::Str)
}

/// Returns true when class metadata exposes a `__toString()` method.
fn object_class_has_tostring(ctx: &FunctionContext<'_>, class_name: &str) -> bool {
    ctx.module
        .class_infos
        .get(class_name)
        .is_some_and(|class_info| class_info.methods.contains_key("__tostring"))
}

/// Lowers array-like PHP values to the literal string used by PHP casts.
fn lower_array_like_to_string(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    emit_array_like_string_result(ctx);
    store_if_result(ctx, inst)
}

/// Materializes PHP's array-to-string placeholder in the active string result registers.
pub(super) fn emit_array_like_string_result(ctx: &mut FunctionContext<'_>) {
    let (label, len) = ctx.data.add_string(b"Array");
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_symbol_address(ctx.emitter, ptr_reg, &label);
    abi::emit_load_int_immediate(ctx.emitter, len_reg, len as i64);
}

/// Converts the loaded native resource payload into PHP's resource id.
///
/// This used to be `payload + 1`, which only ever worked for descriptor-backed
/// resources; a `HashContext` handle is a malloc'd address, so `(int)` on one
/// produced a raw pointer. `__rt_resource_id_of` answers from the resource-id
/// registry instead, which is small, creation-ordered and stable across runs for
/// every resource kind.
fn emit_resource_display_id_to_int(ctx: &mut FunctionContext<'_>) {
    abi::emit_call_label(ctx.emitter, "__rt_resource_id_of");
}

/// Returns the cast target immediate attached to a `Cast` instruction.
fn expect_cast_target(inst: &Instruction) -> Result<IrType> {
    match inst.immediate {
        Some(Immediate::CastTarget(target)) => Ok(target),
        _ => Err(CodegenIrError::invalid_module(format!(
            "{} missing cast target immediate",
            inst.op.name()
        ))),
    }
}
