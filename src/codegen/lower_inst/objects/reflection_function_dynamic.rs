//! Purpose:
//! Lowers `new ReflectionFunction($dynamicValue)` for a `Closure`/`callable`-typed operand whose
//! identity is not statically resolvable at the call site, or a `Mixed`/`Union` operand the
//! checker accepted for the same reason. The runtime callable descriptor
//! (`crate::codegen_support::callable_descriptor`) already carries a full signature record and
//! the target's real name, so no additional metadata table is emitted: the only per-call-site
//! cost is instructions dereferencing the descriptor.
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects::reflection::lower_reflection_owner_new()` routes here
//!   whenever `reflection::is_reflection_function_static_operand()` returns `false` for the
//!   constructor's single operand.
//!
//! Key details:
//! - The ctor performs a runtime tag check before touching the descriptor: a `Mixed`/`Union`
//!   operand is unboxed and tag 10 (callable descriptor) proceeds; array/object/resource tags
//!   throw a catchable, php-verified `\TypeError` ("must be of type Closure|string, X given");
//!   a runtime string or weak-coercible scalar is a PHP-valid input this compiler cannot yet
//!   resolve (dynamic function-name lookup) and hits a loud runtime fatal instead of a silently
//!   wrong guess.
//! - Populated slots: `__parameter_count`/`__required_parameter_count` (read off the
//!   descriptor's signature record), `__name`/`__short_name`/`name` (the `"{closure}"` marker
//!   for an anonymous closure descriptor, or the descriptor's own real name for a wrapped named
//!   target), and `__is_anonymous`.
//! - `__unbacked_name` stays `false` (name methods stay backed); `__unbacked_file`,
//!   `__unbacked_params`, and `__unbacked_return_type` are set `true`: no per-value source
//!   file, parameter array, or declared return type exists on a runtime descriptor, so those
//!   methods throw a catchable `\ReflectionException` (see the shell gating in
//!   `crate::types::checker::builtin_types::reflection`).
//! - `getShortName()` intentionally returns the same value as `getName()` for this path — no
//!   runtime namespace-splitting routine exists (an anonymous closure's short name is
//!   `"{closure}"` too).

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen::{CodegenIrError, Result};
use crate::codegen_support::callable_descriptor::{
    CALLABLE_DESC_KIND_CLOSURE, CALLABLE_DESC_SIGNATURE_OFFSET,
};
use crate::ir::{Instruction, ValueId};
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::reflection::emit_reflection_string_property;

/// Byte offset of a callable descriptor's `php_name` pointer word (see
/// `crate::codegen_support::callable_descriptor::static_descriptor_from_spec`'s word order:
/// kind(0), entry(8), name_ptr(16), name_len(24), signature(32), …).
const CALLABLE_DESC_NAME_PTR_OFFSET: usize = 16;
/// Byte offset of a callable descriptor's `php_name` length word.
const CALLABLE_DESC_NAME_LEN_OFFSET: usize = 24;
/// Byte offset of the signature record's `visible_param_count` word (record-relative).
const SIGNATURE_NUM_PARAMS_OFFSET: usize = 0;
/// Byte offset of the signature record's `required_count` word (record-relative).
const SIGNATURE_NUM_REQUIRED_OFFSET: usize = 8;

const MIXED_TAG_INT: i64 = 0;
const MIXED_TAG_STRING: i64 = 1;
const MIXED_TAG_FLOAT: i64 = 2;
const MIXED_TAG_BOOL: i64 = 3;
const MIXED_TAG_ARRAY: i64 = 4;
const MIXED_TAG_ASSOC_ARRAY: i64 = 5;
const MIXED_TAG_NULL: i64 = 8;
const MIXED_TAG_RESOURCE: i64 = 9;
const MIXED_TAG_CALLABLE: i64 = 10;

/// Fixed-size reserved temporary-stack layout used throughout this lowering. Every slot is
/// addressed with `abi::emit_store_to_sp`/`emit_load_temporary_stack_slot` (SP-relative, fixed
/// offset) — never `emit_push_reg*`/`emit_pop_reg*` (which move SP) while any of these fixed
/// offsets are still needed afterward.
const DESCRIPTOR_SLOT: usize = 0;
const OBJECT_SLOT: usize = 16;
/// Holds the descriptor's persisted real name (pointer, then length at `+8`) across the
/// object-pointer reload in `emit_runtime_named_target_name`.
const NAME_SLOT: usize = 32;
const TEMP_STACK_BYTES: usize = 48;

/// Lowers `new ReflectionFunction($dynamicValue)` — see the module doc comment. The operand's
/// static PHP type (`Callable` for an unboxed descriptor pointer, `Mixed`/`Union` for a boxed
/// cell that may or may not be callable at runtime) selects how the descriptor pointer is
/// obtained before the shared property-population sequence runs.
pub(super) fn lower_reflection_function_new_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    function_operand: ValueId,
) -> Result<()> {
    let operand_ty = ctx.value_php_type(function_operand)?.codegen_repr();
    abi::emit_reserve_temporary_stack(ctx.emitter, TEMP_STACK_BYTES);
    match operand_ty {
        PhpType::Callable => {
            // Already an unboxed descriptor pointer by ABI invariant — no tag to check.
            ctx.load_value_to_result(function_operand)?;
            abi::emit_store_to_sp(ctx.emitter, abi::int_result_reg(ctx.emitter), DESCRIPTOR_SLOT);
        }
        PhpType::Mixed | PhpType::Union(_) => {
            ctx.load_value_to_result(function_operand)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            emit_unbox_tag_dispatch(ctx)?;
        }
        other => {
            abi::emit_release_temporary_stack(ctx.emitter, TEMP_STACK_BYTES);
            return Err(CodegenIrError::unsupported(format!(
                "ReflectionFunction dynamic construction for PHP type {:?}",
                other
            )));
        }
    }

    let (
        class_id,
        property_count,
        uninitialized_marker_offsets,
        name_off,
        public_name_off,
        short_off,
        np_off,
        nr_off,
        unbacked_file_off,
        unbacked_params_off,
        is_anonymous_off,
        unbacked_return_type_off,
    ) = {
        let class_info = ctx
            .module
            .class_infos
            .get("ReflectionFunction")
            .ok_or_else(|| CodegenIrError::unsupported("unknown class ReflectionFunction"))?;
        let slot = |name: &str| -> Result<usize> {
            class_info
                .property_offsets
                .get(name)
                .copied()
                .ok_or_else(|| CodegenIrError::missing_entry("property offset", 0))
        };
        (
            class_info.class_id,
            class_info.properties.len(),
            super::uninitialized_property_marker_offsets(class_info),
            slot("__name")?,
            slot("name")?,
            slot("__short_name")?,
            slot("__parameter_count")?,
            slot("__required_parameter_count")?,
            slot("__unbacked_file")?,
            slot("__unbacked_params")?,
            slot("__is_anonymous")?,
            slot("__unbacked_return_type")?,
        )
    };
    super::emit_object_allocation(
        ctx,
        class_id,
        property_count,
        false,
        &uninitialized_marker_offsets,
        &[],
    )?;
    abi::emit_store_to_sp(ctx.emitter, abi::int_result_reg(ctx.emitter), OBJECT_SLOT);

    // `__unbacked_name` stays false (name methods are backed below); file/params/return-type
    // are unconditionally unbacked for every dynamic instance — no per-value source file,
    // parameter array, or declared return type exists on a runtime callable descriptor.
    emit_store_object_property_immediate(ctx, 1, unbacked_file_off);
    emit_store_object_property_immediate(ctx, 1, unbacked_params_off);
    emit_store_object_property_immediate(ctx, 1, unbacked_return_type_off);

    // -- branch on the descriptor's own `kind` field to select the name/anonymity story --
    let closure_label = ctx.next_label("reflect_fn_dyn_closure");
    let named_label = ctx.next_label("reflect_fn_dyn_named");
    let done_label = ctx.next_label("reflect_fn_dyn_name_done");
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        DESCRIPTOR_SLOT,
    );
    let kind_reg = abi::secondary_scratch_reg(ctx.emitter);
    abi::emit_load_from_address(ctx.emitter, kind_reg, abi::int_result_reg(ctx.emitter), 0);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("cmp {}, #{}", kind_reg, CALLABLE_DESC_KIND_CLOSURE)); // is the reflected target an anonymous closure literal?
            ctx.emitter.instruction(&format!("b.eq {}", closure_label));        // anonymous closures get the "{closure}" marker name
            ctx.emitter.instruction(&format!("b {}", named_label));             // every other kind keeps its own real target name
        }
        Arch::X86_64 => {
            ctx.emitter
                .instruction(&format!("cmp {}, {}", kind_reg, CALLABLE_DESC_KIND_CLOSURE)); // is the reflected target an anonymous closure literal?
            ctx.emitter.instruction(&format!("je {}", closure_label));          // anonymous closures get the "{closure}" marker name
            ctx.emitter.instruction(&format!("jmp {}", named_label));           // every other kind keeps its own real target name
        }
    }

    ctx.emitter.label(&closure_label);
    // `emit_reflection_string_property` requires the object pointer in the ABI integer result
    // register on entry — reload it before the first chained call.
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        OBJECT_SLOT,
    );
    emit_reflection_string_property(ctx, "{closure}", name_off, name_off + 8);
    emit_reflection_string_property(ctx, "{closure}", public_name_off, public_name_off + 8);
    emit_reflection_string_property(ctx, "{closure}", short_off, short_off + 8);
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        OBJECT_SLOT,
    );
    emit_store_object_property_immediate(ctx, 1, is_anonymous_off);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&named_label);
    // A wrapped/first-class-callable-resolved target carries its own real name in the
    // descriptor — copy it at runtime (persisted, matching every other Reflection string
    // slot's ownership contract). `getShortName()` reuses the same value.
    emit_runtime_named_target_name(ctx, name_off, public_name_off, short_off);
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        OBJECT_SLOT,
    );
    emit_store_object_property_immediate(ctx, 0, is_anonymous_off);

    ctx.emitter.label(&done_label);

    // -- read the descriptor's signature record for the two parameter counts --
    emit_param_count_properties(ctx, np_off, nr_off);

    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        OBJECT_SLOT,
    );
    abi::emit_release_temporary_stack(ctx.emitter, TEMP_STACK_BYTES);
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("reflection object_new missing result"))?;
    ctx.store_result_value(result)
}

/// Stores a compile-time-known small integer (`0` or `1`) into an object property slot.
/// Requires the object pointer to already be in the ABI integer result register.
fn emit_store_object_property_immediate(
    ctx: &mut FunctionContext<'_>,
    value: i64,
    low_offset: usize,
) {
    let object_reg = abi::int_result_reg(ctx.emitter);
    let scratch = abi::secondary_scratch_reg(ctx.emitter);
    abi::emit_load_int_immediate(ctx.emitter, scratch, value);
    abi::emit_store_to_address(ctx.emitter, scratch, object_reg, low_offset);
    abi::emit_store_zero_to_address(ctx.emitter, object_reg, low_offset + 8);
}

/// Copies the descriptor's own `php_name` (pointer/length) into the object's `__name`, public
/// `name`, and `__short_name` slots, persisting one fresh heap-owned copy.
fn emit_runtime_named_target_name(
    ctx: &mut FunctionContext<'_>,
    name_off: usize,
    public_name_off: usize,
    short_off: usize,
) {
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        DESCRIPTOR_SLOT,
    );
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_from_address(ctx.emitter, "x1", "x0", CALLABLE_DESC_NAME_PTR_OFFSET);
            abi::emit_load_from_address(ctx.emitter, "x2", "x0", CALLABLE_DESC_NAME_LEN_OFFSET);
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");
        }
        Arch::X86_64 => {
            // int_result_reg (rax) holds the descriptor pointer; preserve it in a scratch
            // register first since the name pointer must land in rax (persist's input reg).
            let desc_reg = abi::secondary_scratch_reg(ctx.emitter);
            ctx.emitter.instruction(&format!("mov {}, rax", desc_reg));         // preserve the descriptor pointer before reusing rax
            abi::emit_load_from_address(ctx.emitter, "rax", desc_reg, CALLABLE_DESC_NAME_PTR_OFFSET);
            abi::emit_load_from_address(ctx.emitter, "rdx", desc_reg, CALLABLE_DESC_NAME_LEN_OFFSET);
            abi::emit_call_label(ctx.emitter, "__rt_str_persist");
        }
    }
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    // Park the owned name in its own fixed stack slot (not a push/pop pair) so OBJECT_SLOT's
    // fixed offset stays valid for the reload right below.
    abi::emit_store_to_sp(ctx.emitter, ptr_reg, NAME_SLOT);
    abi::emit_store_to_sp(ctx.emitter, len_reg, NAME_SLOT + 8);
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        OBJECT_SLOT,
    );
    let object_reg = abi::symbol_scratch_reg(ctx.emitter);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!("mov {}, x0", object_reg)), // preserve the object pointer while reloading the owned name
        Arch::X86_64 => ctx.emitter.instruction(&format!("mov {}, rax", object_reg)), // preserve the object pointer while reloading the owned name
    }
    let reload_ptr_reg = abi::int_result_reg(ctx.emitter);
    let reload_len_reg = abi::secondary_scratch_reg(ctx.emitter);
    abi::emit_load_temporary_stack_slot(ctx.emitter, reload_ptr_reg, NAME_SLOT);
    abi::emit_load_temporary_stack_slot(ctx.emitter, reload_len_reg, NAME_SLOT + 8);
    abi::emit_store_to_address(ctx.emitter, reload_ptr_reg, object_reg, name_off);
    abi::emit_store_to_address(ctx.emitter, reload_len_reg, object_reg, name_off + 8);
    abi::emit_store_to_address(ctx.emitter, reload_ptr_reg, object_reg, public_name_off);
    abi::emit_store_to_address(ctx.emitter, reload_len_reg, object_reg, public_name_off + 8);
    abi::emit_store_to_address(ctx.emitter, reload_ptr_reg, object_reg, short_off);
    abi::emit_store_to_address(ctx.emitter, reload_len_reg, object_reg, short_off + 8);
}

/// Reads `descriptor->signature->{visible_param_count,required_count}` and stores both into the
/// object's `__parameter_count`/`__required_parameter_count` slots. A null signature record
/// defensively yields `0`/`0` rather than dereferencing a null pointer.
fn emit_param_count_properties(ctx: &mut FunctionContext<'_>, np_off: usize, nr_off: usize) {
    let zero_label = ctx.next_label("reflect_fn_dyn_no_sig");
    let done_label = ctx.next_label("reflect_fn_dyn_sig_done");

    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        DESCRIPTOR_SLOT,
    );
    let sig_reg = abi::secondary_scratch_reg(ctx.emitter);
    abi::emit_load_from_address(
        ctx.emitter,
        sig_reg,
        abi::int_result_reg(ctx.emitter),
        CALLABLE_DESC_SIGNATURE_OFFSET,
    );
    let np_reg = abi::tertiary_scratch_reg(ctx.emitter);
    let nr_reg = abi::symbol_scratch_reg(ctx.emitter);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz {}, {}", sig_reg, zero_label)); // no signature record: fall back to zero counts
            abi::emit_load_from_address(ctx.emitter, np_reg, sig_reg, SIGNATURE_NUM_PARAMS_OFFSET);
            abi::emit_load_from_address(ctx.emitter, nr_reg, sig_reg, SIGNATURE_NUM_REQUIRED_OFFSET);
            abi::emit_jump(ctx.emitter, &done_label);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("test {}, {}", sig_reg, sig_reg)); // no signature record: fall back to zero counts
            ctx.emitter.instruction(&format!("je {}", zero_label));             // branch to the zero-count fallback
            abi::emit_load_from_address(ctx.emitter, np_reg, sig_reg, SIGNATURE_NUM_PARAMS_OFFSET);
            abi::emit_load_from_address(ctx.emitter, nr_reg, sig_reg, SIGNATURE_NUM_REQUIRED_OFFSET);
            abi::emit_jump(ctx.emitter, &done_label);
        }
    }
    ctx.emitter.label(&zero_label);
    abi::emit_load_int_immediate(ctx.emitter, np_reg, 0);
    abi::emit_load_int_immediate(ctx.emitter, nr_reg, 0);

    ctx.emitter.label(&done_label);
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        OBJECT_SLOT,
    );
    let object_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_store_to_address(ctx.emitter, np_reg, object_reg, np_off);
    abi::emit_store_zero_to_address(ctx.emitter, object_reg, np_off + 8);
    abi::emit_store_to_address(ctx.emitter, nr_reg, object_reg, nr_off);
    abi::emit_store_zero_to_address(ctx.emitter, object_reg, nr_off + 8);
}

/// After `__rt_mixed_unbox` (tag in the int-result register, payload in the unbox payload
/// registers), branches on the runtime tag and either stores the unboxed descriptor pointer
/// into `DESCRIPTOR_SLOT` (tag 10) or diverts to a throw/fatal.
fn emit_unbox_tag_dispatch(ctx: &mut FunctionContext<'_>) -> Result<()> {
    let callable_label = ctx.next_label("reflect_fn_dyn_callable");
    let string_label = ctx.next_label("reflect_fn_dyn_string");
    let scalar_label = ctx.next_label("reflect_fn_dyn_scalar");
    let type_error_label = ctx.next_label("reflect_fn_dyn_type_error");
    let done_label = ctx.next_label("reflect_fn_dyn_unbox_done");
    // `__rt_mixed_unbox`'s own convention: AArch64 tag=x0, payload_lo=x1; x86_64 tag=rax,
    // payload_lo=rdi.
    let payload_lo = match ctx.emitter.target.arch {
        Arch::AArch64 => "x1",
        Arch::X86_64 => "rdi",
    };
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cmp x0, #{}", MIXED_TAG_CALLABLE)); // does the boxed union hold a callable descriptor?
            ctx.emitter.instruction(&format!("b.eq {}", callable_label));       // callable descriptors proceed to the property bake
            ctx.emitter.instruction(&format!("cmp x0, #{}", MIXED_TAG_STRING)); // a runtime string function name is PHP-valid but unresolved here
            ctx.emitter.instruction(&format!("b.eq {}", string_label));         // strings hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("cmp x0, #{}", MIXED_TAG_INT));    // int/float/bool/null weak-coerce to a string in real PHP too
            ctx.emitter.instruction(&format!("b.eq {}", scalar_label));         // weak-coercible scalars hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("cmp x0, #{}", MIXED_TAG_FLOAT));  // float payloads take the scalar arm too
            ctx.emitter.instruction(&format!("b.eq {}", scalar_label));         // weak-coercible scalars hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("cmp x0, #{}", MIXED_TAG_BOOL));   // bool payloads take the scalar arm too
            ctx.emitter.instruction(&format!("b.eq {}", scalar_label));         // weak-coercible scalars hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("cmp x0, #{}", MIXED_TAG_NULL));   // null payloads take the scalar arm too
            ctx.emitter.instruction(&format!("b.eq {}", scalar_label));         // weak-coercible scalars hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("b {}", type_error_label));        // array/object/resource/other: not coercible to Closure|string
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp rax, {}", MIXED_TAG_CALLABLE)); // does the boxed union hold a callable descriptor?
            ctx.emitter.instruction(&format!("je {}", callable_label));         // callable descriptors proceed to the property bake
            ctx.emitter.instruction(&format!("cmp rax, {}", MIXED_TAG_STRING)); // a runtime string function name is PHP-valid but unresolved here
            ctx.emitter.instruction(&format!("je {}", string_label));           // strings hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("cmp rax, {}", MIXED_TAG_INT));    // int/float/bool/null weak-coerce to a string in real PHP too
            ctx.emitter.instruction(&format!("je {}", scalar_label));           // weak-coercible scalars hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("cmp rax, {}", MIXED_TAG_FLOAT));  // float payloads take the scalar arm too
            ctx.emitter.instruction(&format!("je {}", scalar_label));           // weak-coercible scalars hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("cmp rax, {}", MIXED_TAG_BOOL));   // bool payloads take the scalar arm too
            ctx.emitter.instruction(&format!("je {}", scalar_label));           // weak-coercible scalars hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("cmp rax, {}", MIXED_TAG_NULL));   // null payloads take the scalar arm too
            ctx.emitter.instruction(&format!("je {}", scalar_label));           // weak-coercible scalars hit the loud not-yet-supported fatal
            ctx.emitter.instruction(&format!("jmp {}", type_error_label));      // array/object/resource/other: not coercible to Closure|string
        }
    }

    ctx.emitter.label(&callable_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!("mov x0, {}", payload_lo)), // move the unboxed descriptor pointer out of the unbox payload register
        Arch::X86_64 => ctx.emitter.instruction(&format!("mov rax, {}", payload_lo)), // move the unboxed descriptor pointer out of the unbox payload register
    }
    abi::emit_store_to_sp(ctx.emitter, abi::int_result_reg(ctx.emitter), DESCRIPTOR_SLOT);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&string_label);
    emit_not_yet_supported_fatal(
        ctx,
        "ReflectionFunction::__construct(): a runtime string function name is not yet supported (pass a closure literal, a first-class callable, or a compile-time-constant function name string)\n",
    );

    ctx.emitter.label(&scalar_label);
    emit_not_yet_supported_fatal(
        ctx,
        "ReflectionFunction::__construct(): a weak-coerced int/float/bool/null function-name lookup is not yet supported\n",
    );

    ctx.emitter.label(&type_error_label);
    emit_wrong_tag_type_error_dispatch(ctx);

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Dispatches on the `__rt_mixed_unbox` tag (still in the int-result register) to throw a
/// catchable `\TypeError` with PHP's exact wording for the tags that reach this branch. A
/// non-Stringable object always gets the generic `"object given"` wording, not its real class
/// name — a small, documented simplification.
fn emit_wrong_tag_type_error_dispatch(ctx: &mut FunctionContext<'_>) {
    let array_label = ctx.next_label("reflect_fn_dyn_te_array");
    let resource_label = ctx.next_label("reflect_fn_dyn_te_resource");
    let generic_label = ctx.next_label("reflect_fn_dyn_te_generic");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cmp x0, #{}", MIXED_TAG_ARRAY));  // indexed arrays get PHP's "array given" wording
            ctx.emitter.instruction(&format!("b.eq {}", array_label));          // branch to the array-specific TypeError message
            ctx.emitter
                .instruction(&format!("cmp x0, #{}", MIXED_TAG_ASSOC_ARRAY));   // associative arrays get the same "array given" wording
            ctx.emitter.instruction(&format!("b.eq {}", array_label));          // branch to the array-specific TypeError message
            ctx.emitter
                .instruction(&format!("cmp x0, #{}", MIXED_TAG_RESOURCE));      // resources get PHP's "resource given" wording
            ctx.emitter.instruction(&format!("b.eq {}", resource_label));       // branch to the resource-specific TypeError message
            ctx.emitter.instruction(&format!("b {}", generic_label));           // object/other: generic "object given" fallback
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp rax, {}", MIXED_TAG_ARRAY));  // indexed arrays get PHP's "array given" wording
            ctx.emitter.instruction(&format!("je {}", array_label));            // branch to the array-specific TypeError message
            ctx.emitter
                .instruction(&format!("cmp rax, {}", MIXED_TAG_ASSOC_ARRAY));   // associative arrays get the same "array given" wording
            ctx.emitter.instruction(&format!("je {}", array_label));            // branch to the array-specific TypeError message
            ctx.emitter
                .instruction(&format!("cmp rax, {}", MIXED_TAG_RESOURCE));      // resources get PHP's "resource given" wording
            ctx.emitter.instruction(&format!("je {}", resource_label));         // branch to the resource-specific TypeError message
            ctx.emitter.instruction(&format!("jmp {}", generic_label));         // object/other: generic "object given" fallback
        }
    }
    ctx.emitter.label(&array_label);
    emit_construct_type_error(ctx, "array");
    ctx.emitter.label(&resource_label);
    emit_construct_type_error(ctx, "resource");
    ctx.emitter.label(&generic_label);
    emit_construct_type_error(ctx, "object");
}

/// Constructs and throws a catchable `\TypeError` for `new ReflectionFunction($x)` with PHP's
/// exact message for `given_type`. Never returns.
fn emit_construct_type_error(ctx: &mut FunctionContext<'_>, given_type: &str) {
    let message = format!(
        "ReflectionFunction::__construct(): Argument #1 ($function) must be of type Closure|string, {} given",
        given_type
    );
    super::reflection_dynamic::emit_reflection_dynamic_type_error_throw(ctx, message.as_bytes());
}

/// Writes `message` to stderr and exits with status 1 — a loud runtime fatal for a PHP-valid
/// input this compiler cannot yet resolve (never a silent-wrong guess).
fn emit_not_yet_supported_fatal(ctx: &mut FunctionContext<'_>, message: &str) {
    let (message_label, message_len) = ctx.data.add_string(message.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, #2");                              // write the "not yet supported" diagnostic to stderr
            ctx.emitter.adrp("x1", &message_label);
            ctx.emitter.add_lo12("x1", "x1", &message_label);
            ctx.emitter.instruction(&format!("mov x2, #{}", message_len));      // pass the diagnostic byte length to write
            ctx.emitter.syscall(4);
            abi::emit_exit(ctx.emitter, 1);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov edi, 2");                              // write the "not yet supported" diagnostic to stderr
            abi::emit_symbol_address(ctx.emitter, "rsi", &message_label);
            ctx.emitter.instruction(&format!("mov edx, {}", message_len));      // pass the diagnostic byte length to write
            ctx.emitter.instruction("mov eax, 1");                              // Linux x86_64 syscall 1 = write
            ctx.emitter.instruction("syscall");                                 // emit the diagnostic before terminating
            abi::emit_exit(ctx.emitter, 1);
        }
    }
}
