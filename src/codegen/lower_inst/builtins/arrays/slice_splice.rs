//! Purpose:
//! Slice, splice, chunk, pad, and result normalization.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::arrays`.
//!
//! Key details:
//! - Preserves callback ABI, target parity, array storage, and ownership contracts.

use super::*;

/// Calls the appropriate legacy runtime helper after materializing slice arguments.
pub(super) fn lower_array_slice_call(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    offset: ValueId,
    length: Option<ValueId>,
    source_elem_ty: &PhpType,
) -> Result<()> {
    lower_slice_like_args(ctx, array, offset, length, "array_slice")?;
    abi::emit_call_label(ctx.emitter, array_slice_runtime_helper(source_elem_ty));
    Ok(())
}

/// Calls the appropriate legacy runtime helper after materializing splice arguments.
pub(super) fn lower_array_splice_call(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    offset: ValueId,
    length: Option<ValueId>,
    elem_ty: &PhpType,
) -> Result<()> {
    lower_slice_like_args(ctx, array, offset, length, "array_splice")?;
    abi::emit_call_label(ctx.emitter, array_splice_runtime_helper(elem_ty));
    Ok(())
}

/// Materializes the shared `(array, offset, length)` argument triple for `array_slice` and
/// `array_splice` into the runtime argument registers.
///
/// The offset and length are resolved to plain integers first — unboxing a `Mixed` cell read from a
/// heterogeneous array via `__rt_mixed_cast_int` — and spilled to the stack, because that unbox call
/// clobbers caller-saved registers. The array pointer (a plain stack load that clobbers nothing) is
/// then placed, and the staged integers are restored into the offset/length argument registers, so
/// the runtime helper sees the array pointer plus two genuine integers rather than a boxed pointer.
pub(super) fn lower_slice_like_args(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    offset: ValueId,
    length: Option<ValueId>,
    name: &str,
) -> Result<()> {
    resolve_int_operand_to_result(ctx, offset, &format!("{} offset", name))?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    resolve_slice_length_to_result(ctx, length, name)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(array, "x0")?;
            abi::emit_pop_reg(ctx.emitter, "x2"); // restore the resolved length into the third runtime argument
            abi::emit_pop_reg(ctx.emitter, "x1"); // restore the resolved offset into the second runtime argument
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(array, "rdi")?;
            abi::emit_pop_reg(ctx.emitter, "rdx"); // restore the resolved length into the third runtime argument
            abi::emit_pop_reg(ctx.emitter, "rsi"); // restore the resolved offset into the second runtime argument
        }
    }
    Ok(())
}

/// Resolves an optional `array_slice`/`array_splice` length into the integer result register.
///
/// An absent or `Void` length becomes the runtime "until the end" sentinel; otherwise the length is
/// resolved through the shared integer resolver, unboxing a `Mixed` value to a plain integer.
pub(super) fn resolve_slice_length_to_result(
    ctx: &mut FunctionContext<'_>,
    length: Option<ValueId>,
    name: &str,
) -> Result<()> {
    let until_end = match length {
        None => true,
        Some(length) => matches!(ctx.value_php_type(length)?.codegen_repr(), PhpType::Void),
    };
    if until_end {
        let reg = abi::int_result_reg(ctx.emitter);
        emit_array_slice_until_end_sentinel(ctx, reg);
        return Ok(());
    }
    resolve_int_operand_to_result(
        ctx,
        length.expect("length present"),
        &format!("{} length", name),
    )
}

/// Resolves the offset/length arguments for a boxed-Mixed `array_slice`/`array_splice` into the
/// refcounted runtime helper's argument registers, restoring a previously-staged array pointer.
///
/// On entry the converted (now-owned) indexed-array pointer must be the topmost value on the
/// temporary stack. The offset and length are resolved to plain integers first — `__rt_mixed_cast_int`
/// unboxes a `Mixed` cell read from a heterogeneous array, and an absent/`Void` length becomes the
/// until-the-end sentinel — and spilled to the stack, because each unbox call clobbers caller-saved
/// registers. The three staged values are then popped into the array/offset/length argument registers
/// so the helper sees a pointer plus two genuine integers rather than a boxed pointer.
pub(super) fn materialize_mixed_slice_args(
    ctx: &mut FunctionContext<'_>,
    offset: ValueId,
    length: Option<ValueId>,
    name: &str,
) -> Result<()> {
    resolve_int_operand_to_result(ctx, offset, &format!("{} offset", name))?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    resolve_slice_length_to_result(ctx, length, name)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_pop_reg(ctx.emitter, "x2"); // restore the resolved length into the third runtime argument
            abi::emit_pop_reg(ctx.emitter, "x1"); // restore the resolved offset into the second runtime argument
            abi::emit_pop_reg(ctx.emitter, "x0"); // restore the converted array pointer into the first runtime argument
        }
        Arch::X86_64 => {
            abi::emit_pop_reg(ctx.emitter, "rdx"); // restore the resolved length into the third runtime argument
            abi::emit_pop_reg(ctx.emitter, "rsi"); // restore the resolved offset into the second runtime argument
            abi::emit_pop_reg(ctx.emitter, "rdi"); // restore the converted array pointer into the first runtime argument
        }
    }
    Ok(())
}

/// Materializes a boxed-Mixed indexed array for `array_slice()` on AArch64.
pub(super) fn lower_mixed_array_slice_aarch64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    offset: ValueId,
    length: Option<ValueId>,
) -> Result<()> {
    let empty_label = ctx.next_label("mixed_array_slice_empty");
    let done_label = ctx.next_label("mixed_array_slice_done");
    ctx.load_value_to_reg(array, "x0")?;
    abi::emit_push_reg(ctx.emitter, "x0");
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    ctx.emitter.instruction("cmp x0, #4");                                      // require an indexed-array payload before slicing the Mixed cell
    ctx.emitter.instruction(&format!("b.ne {}", empty_label));                  // return an empty slice for non-array Mixed payloads
    ctx.emitter.instruction(&format!("cbz x1, {}", empty_label));               // return an empty slice for null array payloads
    ctx.emitter.instruction("mov x0, x1");                                      // pass the unboxed indexed-array payload to the Mixed conversion helper
    ctx.emitter.instruction("ldr x1, [x0, #-8]");                               // load indexed-array metadata before Mixed-slot conversion
    ctx.emitter.instruction("lsr x1, x1, #8");                                  // move the runtime value_type tag into the low bits
    ctx.emitter.instruction("and x1, x1, #0x7f");                               // isolate the indexed-array value_type tag
    abi::emit_call_label(ctx.emitter, "__rt_array_to_mixed");
    abi::emit_pop_reg(ctx.emitter, "x10");
    ctx.emitter.instruction("str x0, [x10, #8]");                               // publish the converted unique array back into the Mixed cell
    abi::emit_push_reg(ctx.emitter, "x0");
    materialize_mixed_slice_args(ctx, offset, length, "array_slice")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_slice_refcounted");
    ctx.emitter.instruction(&format!("b {}", done_label));                      // skip the empty-array fallback after slicing the boxed payload
    ctx.emitter.label(&empty_label);
    abi::emit_pop_reg(ctx.emitter, "x9");
    allocate_empty_mixed_array_result(ctx);
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Materializes a boxed-Mixed indexed array for `array_slice()` on x86_64.
pub(super) fn lower_mixed_array_slice_x86_64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    offset: ValueId,
    length: Option<ValueId>,
) -> Result<()> {
    let empty_label = ctx.next_label("mixed_array_slice_empty");
    let done_label = ctx.next_label("mixed_array_slice_done");
    ctx.load_value_to_reg(array, "rax")?;
    abi::emit_push_reg(ctx.emitter, "rax");
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    ctx.emitter.instruction("cmp rax, 4");                                      // require an indexed-array payload before slicing the Mixed cell
    ctx.emitter.instruction(&format!("jne {}", empty_label));                   // return an empty slice for non-array Mixed payloads
    ctx.emitter.instruction("test rdi, rdi");                                   // verify the unboxed indexed-array payload is present
    ctx.emitter.instruction(&format!("je {}", empty_label));                    // return an empty slice for null array payloads
    ctx.emitter.instruction("mov rsi, QWORD PTR [rdi - 8]");                    // load indexed-array metadata before Mixed-slot conversion
    ctx.emitter.instruction("shr rsi, 8");                                      // move the runtime value_type tag into the low bits
    ctx.emitter.instruction("and rsi, 0x7f");                                   // isolate the indexed-array value_type tag
    abi::emit_call_label(ctx.emitter, "__rt_array_to_mixed");
    abi::emit_pop_reg(ctx.emitter, "r10");
    ctx.emitter.instruction("mov QWORD PTR [r10 + 8], rax");                    // publish the converted unique array back into the Mixed cell
    abi::emit_push_reg(ctx.emitter, "rax");
    materialize_mixed_slice_args(ctx, offset, length, "array_slice")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_slice_refcounted");
    ctx.emitter.instruction(&format!("jmp {}", done_label));                    // skip the empty-array fallback after slicing the boxed payload
    ctx.emitter.label(&empty_label);
    abi::emit_pop_reg(ctx.emitter, "r11");
    allocate_empty_mixed_array_result(ctx);
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Materializes and mutates a boxed-Mixed indexed array for `array_splice()` on AArch64.
pub(super) fn lower_mixed_array_splice_aarch64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    offset: ValueId,
    length: Option<ValueId>,
) -> Result<()> {
    let drop_label = ctx.next_label("mixed_array_splice_empty");
    let done_label = ctx.next_label("mixed_array_splice_done");
    ctx.load_value_to_reg(array, "x0")?;
    abi::emit_push_reg(ctx.emitter, "x0");
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    ctx.emitter.instruction("cmp x0, #4");                                      // require an indexed-array payload before splicing the Mixed cell
    ctx.emitter.instruction(&format!("b.ne {}", drop_label));                   // return an empty removed-elements array for non-array Mixed payloads
    ctx.emitter.instruction(&format!("cbz x1, {}", drop_label));                // return an empty removed-elements array for null array payloads
    ctx.emitter.instruction("mov x0, x1");                                      // pass the unboxed indexed-array payload to the Mixed conversion helper
    ctx.emitter.instruction("ldr x1, [x0, #-8]");                               // load indexed-array metadata before Mixed-slot conversion
    ctx.emitter.instruction("lsr x1, x1, #8");                                  // move the runtime value_type tag into the low bits
    ctx.emitter.instruction("and x1, x1, #0x7f");                               // isolate the indexed-array value_type tag
    abi::emit_call_label(ctx.emitter, "__rt_array_to_mixed");
    abi::emit_pop_reg(ctx.emitter, "x10");
    ctx.emitter.instruction("str x0, [x10, #8]");                               // publish the converted unique array back into the Mixed cell
    abi::emit_push_reg(ctx.emitter, "x0");
    materialize_mixed_slice_args(ctx, offset, length, "array_splice")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_splice_refcounted");
    ctx.emitter.instruction(&format!("b {}", done_label));                      // skip the empty-array fallback after splicing the boxed payload
    ctx.emitter.label(&drop_label);
    abi::emit_pop_reg(ctx.emitter, "x9");
    allocate_empty_mixed_array_result(ctx);
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Materializes and mutates a boxed-Mixed indexed array for `array_splice()` on x86_64.
pub(super) fn lower_mixed_array_splice_x86_64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    offset: ValueId,
    length: Option<ValueId>,
) -> Result<()> {
    let drop_label = ctx.next_label("mixed_array_splice_empty");
    let done_label = ctx.next_label("mixed_array_splice_done");
    ctx.load_value_to_reg(array, "rax")?;
    abi::emit_push_reg(ctx.emitter, "rax");
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    ctx.emitter.instruction("cmp rax, 4");                                      // require an indexed-array payload before splicing the Mixed cell
    ctx.emitter.instruction(&format!("jne {}", drop_label));                    // return an empty removed-elements array for non-array Mixed payloads
    ctx.emitter.instruction("test rdi, rdi");                                   // verify the unboxed indexed-array payload is present
    ctx.emitter.instruction(&format!("je {}", drop_label));                     // return an empty removed-elements array for null array payloads
    ctx.emitter.instruction("mov rsi, QWORD PTR [rdi - 8]");                    // load indexed-array metadata before Mixed-slot conversion
    ctx.emitter.instruction("shr rsi, 8");                                      // move the runtime value_type tag into the low bits
    ctx.emitter.instruction("and rsi, 0x7f");                                   // isolate the indexed-array value_type tag
    abi::emit_call_label(ctx.emitter, "__rt_array_to_mixed");
    abi::emit_pop_reg(ctx.emitter, "r10");
    ctx.emitter.instruction("mov QWORD PTR [r10 + 8], rax");                    // publish the converted unique array back into the Mixed cell
    abi::emit_push_reg(ctx.emitter, "rax");
    materialize_mixed_slice_args(ctx, offset, length, "array_splice")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_splice_refcounted");
    ctx.emitter.instruction(&format!("jmp {}", done_label));                    // skip the empty-array fallback after splicing the boxed payload
    ctx.emitter.label(&drop_label);
    abi::emit_pop_reg(ctx.emitter, "r11");
    allocate_empty_mixed_array_result(ctx);
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Adapts the removed-elements array returned by `array_splice` to the EIR result type.
pub(super) fn normalize_array_splice_result(
    ctx: &mut FunctionContext<'_>,
    elem_ty: &PhpType,
    result_ty: &PhpType,
) -> Result<()> {
    let removed_ty = PhpType::Array(Box::new(elem_ty.codegen_repr()));
    match result_ty {
        PhpType::Mixed => {
            emit_box_current_owned_value_as_mixed(ctx.emitter, &removed_ty);
            Ok(())
        }
        PhpType::Array(result_elem) if result_elem.codegen_repr() == elem_ty.codegen_repr() => {
            Ok(())
        }
        other => Err(CodegenIrError::unsupported(format!(
            "array_splice result PHP type {:?}",
            other
        ))),
    }
}

/// Allocates an empty boxed-Mixed indexed array for dynamic splice fallback paths.
pub(super) fn allocate_empty_mixed_array_result(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_int_immediate(ctx.emitter, "x0", 0);
            abi::emit_load_int_immediate(ctx.emitter, "x1", 8);
        }
        Arch::X86_64 => {
            abi::emit_load_int_immediate(ctx.emitter, "rdi", 0);
            abi::emit_load_int_immediate(ctx.emitter, "rsi", 8);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_array_new");
    crate::codegen::emit_array_value_type_stamp(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        &PhpType::Mixed,
    );
}

/// Calls the appropriate legacy runtime helper after materializing chunk arguments.
pub(super) fn lower_array_chunk_call(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    length: ValueId,
    source_elem_ty: &PhpType,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(array, "x0")?;
            ctx.load_value_to_reg(length, "x1")?;
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(array, "rdi")?;
            ctx.load_value_to_reg(length, "rsi")?;
        }
    }
    abi::emit_call_label(ctx.emitter, array_chunk_runtime_helper(source_elem_ty));
    Ok(())
}

/// Calls the appropriate legacy runtime helper after materializing pad arguments.
pub(super) fn lower_array_pad_call(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    target_size: ValueId,
    pad_value: ValueId,
    source_elem_ty: &PhpType,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(array, "x0")?;
            ctx.load_value_to_reg(target_size, "x1")?;
            ctx.load_value_to_reg(pad_value, "x2")?;
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(array, "rdi")?;
            ctx.load_value_to_reg(target_size, "rsi")?;
            ctx.load_value_to_reg(pad_value, "rdx")?;
        }
    }
    abi::emit_call_label(ctx.emitter, array_pad_runtime_helper(source_elem_ty));
    Ok(())
}

/// Emits the `-1` runtime sentinel used when slicing to the end of the source array.
pub(super) fn emit_array_slice_until_end_sentinel(ctx: &mut FunctionContext<'_>, reg: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov {}, #-1", reg));              // use -1 as the array_slice() runtime sentinel for length until the end
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov {}, -1", reg));               // use -1 as the x86_64 array_slice() runtime sentinel for length until the end
        }
    }
}

/// Returns the helper that matches the chunk source element ownership representation.
pub(super) fn array_chunk_runtime_helper(source_elem_ty: &PhpType) -> &'static str {
    if source_elem_ty.is_refcounted() {
        "__rt_array_chunk_refcounted"
    } else {
        "__rt_array_chunk"
    }
}

/// Returns the helper that matches the pad source element ownership representation.
pub(super) fn array_pad_runtime_helper(source_elem_ty: &PhpType) -> &'static str {
    if source_elem_ty.is_refcounted() {
        "__rt_array_pad_refcounted"
    } else {
        "__rt_array_pad"
    }
}

/// Returns the helper that matches the source element ownership representation.
pub(super) fn array_slice_runtime_helper(source_elem_ty: &PhpType) -> &'static str {
    if source_elem_ty.is_refcounted() {
        "__rt_array_slice_refcounted"
    } else {
        "__rt_array_slice"
    }
}

/// Returns the helper that matches the spliced element ownership representation.
pub(super) fn array_splice_runtime_helper(elem_ty: &PhpType) -> &'static str {
    if elem_ty.is_refcounted() {
        "__rt_array_splice_refcounted"
    } else {
        "__rt_array_splice"
    }
}

/// Stamps the result array and widens typed slots when the EIR result expects Mixed.
pub(super) fn normalize_indexed_array_result(
    ctx: &mut FunctionContext<'_>,
    name: &str,
    source_elem_ty: &PhpType,
    result_elem_ty: &PhpType,
) -> Result<()> {
    if result_elem_ty == &PhpType::Mixed && source_elem_ty != &PhpType::Mixed {
        let source_tag = runtime_value_tag(name, source_elem_ty)?;
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction(&format!("mov x1, #{}", source_tag));   // pass the source slot value_type tag to widen the indexed-array result to Mixed
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("mov rdi, rax");                        // pass the produced indexed-array pointer to the Mixed-widening helper
                ctx.emitter.instruction(&format!("mov rsi, {}", source_tag));   // pass the source slot value_type tag to widen the indexed-array result to Mixed
            }
        }
        abi::emit_call_label(ctx.emitter, "__rt_array_to_mixed");
        return Ok(());
    }
    crate::codegen::emit_array_value_type_stamp(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        result_elem_ty,
    );
    Ok(())
}
