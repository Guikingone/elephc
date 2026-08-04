//! Purpose:
//! Lowers PHP's single-array `min()` / `max()` form for the EIR backend.
//! Walks an indexed array's payload slots in place and reduces them to one element.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::math::lower_min_max()`.
//!
//! Key details:
//! - Indexed arrays store their logical length in the first header word and their
//!   payload slots 24 bytes after the header, one 8-byte slot per element.
//! - An empty array is PHP's `ValueError`, thrown through the shared math
//!   `emit_throw_value_error()` path so it stays catchable like `clamp()`'s.
//! - Scratch is limited to the registers the backend already treats as clobbered by
//!   a builtin call (`x9`–`x13`, `d0`/`d1`; `rax`/`rcx`/`rdx`/`r10`/`r11`, `xmm0`/`xmm1`),
//!   so no register-allocated value can be destroyed by the loop.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen::{CodegenIrError, Result};
use crate::ir::Instruction;
use crate::types::PhpType;

use super::super::super::super::context::FunctionContext;
use super::super::expect_operand;

/// Byte offset of the first payload slot inside an indexed-array allocation.
const ARRAY_DATA_OFFSET: i64 = 24;

/// Lowers PHP's single-argument `min()` / `max()` form and reports whether it applied.
///
/// Returns `Ok(false)` for the variadic form so the caller keeps its own lowering.
/// Any single-argument call whose operand is an array is handled here — including the
/// element types the reduction cannot compare, which are rejected with an explicit
/// diagnostic rather than falling through to the numeric paths.
pub(super) fn try_lower_single_array(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    want_max: bool,
) -> Result<bool> {
    if inst.operands.len() != 1 {
        return Ok(false);
    }
    let array = expect_operand(inst, 0)?;
    match ctx.value_php_type(array)?.codegen_repr() {
        PhpType::Array(element) => {
            lower_array_min_max(ctx, inst, want_max, &element.codegen_repr())?;
            Ok(true)
        }
        PhpType::AssocArray { key, value } => Err(unsupported_element_error(
            super::min_max_name(want_max),
            &format!("array<{}, {}>", key, value),
        )),
        _ => Ok(false),
    }
}

/// Formats the diagnostic for a single-array `min()` / `max()` the reduction cannot compare.
///
/// The reduction walks 8-byte scalar payload slots, so it covers indexed arrays of
/// `int`, `float`, and `bool`. String, boxed-`Mixed`, and hash-backed associative
/// arrays would need PHP's full comparison table on heap values and are rejected here.
fn unsupported_element_error(name: &str, shape: &str) -> CodegenIrError {
    CodegenIrError::unsupported(format!(
        "{}() with a single array argument requires an indexed array of int, float, or bool values, got {}",
        name, shape
    ))
}

/// Lowers `min($array)` / `max($array)` by reducing the array's payload slots.
///
/// `element` is the array's codegen element representation. Integer-like elements
/// are compared as signed 64-bit words and floating elements through the same
/// `fmin`/`fmax` (AArch64) and `minsd`/`maxsd` (x86_64) selection the variadic form
/// uses, so both call forms agree on every target. An empty array throws PHP's
/// `ValueError` and never falls through to the reduction.
fn lower_array_min_max(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    want_max: bool,
    element: &PhpType,
) -> Result<()> {
    let name = super::min_max_name(want_max);
    let array = expect_operand(inst, 0)?;
    let float_elements = match element {
        PhpType::Float => true,
        // `Void`/`Never` is the element type of an empty array literal and of an
        // all-null array: both store zeroed scalar slots, so the integer reduction
        // handles them and an empty one reaches the ValueError path.
        PhpType::Int | PhpType::Bool | PhpType::Void | PhpType::Never => false,
        other => {
            return Err(unsupported_element_error(
                name,
                &format!("array<{}>", other),
            ))
        }
    };
    let result_ty = inst
        .result
        .map(|value| ctx.value_php_type(value))
        .transpose()?
        .unwrap_or_else(|| element.clone())
        .codegen_repr();
    let empty_label = ctx.next_label("min_max_array_empty");
    let loop_label = ctx.next_label("min_max_array_loop");
    let reduced_label = ctx.next_label("min_max_array_reduced");
    let done_label = ctx.next_label("min_max_array_done");
    let message = format!(
        "{}(): Argument #1 ($value) must contain at least one element",
        name
    );
    let (message_label, message_len) = ctx.data.add_string(message.as_bytes());

    ctx.load_value_to_result(array)?;
    match (ctx.emitter.target.arch, float_elements) {
        (Arch::AArch64, false) => {
            emit_int_reduce_aarch64(ctx, want_max, &empty_label, &loop_label, &reduced_label)
        }
        (Arch::AArch64, true) => {
            emit_float_reduce_aarch64(ctx, want_max, &empty_label, &loop_label, &reduced_label)
        }
        (Arch::X86_64, false) => {
            emit_int_reduce_x86_64(ctx, want_max, &empty_label, &loop_label, &reduced_label)
        }
        (Arch::X86_64, true) => {
            emit_float_reduce_x86_64(ctx, want_max, &empty_label, &loop_label, &reduced_label)
        }
    }
    ctx.emitter.label(&reduced_label);
    abi::emit_jump(ctx.emitter, &done_label);
    ctx.emitter.label(&empty_label);
    super::emit_throw_value_error(ctx, &message_label, message_len);
    ctx.emitter.label(&done_label);
    materialize_result(ctx, element, float_elements, &result_ty)
}

/// Converts the reduced element into the representation the EIR result value expects.
///
/// EIR array element types and checker call-site types are inferred separately, so a
/// reduction over an `array<bool>` can still feed a boxed `Mixed` result. The reduced
/// element is boxed with its own element type (so `min([true, false])` stays `bool`)
/// and int-typed elements promote when the result value is a float.
fn materialize_result(
    ctx: &mut FunctionContext<'_>,
    element: &PhpType,
    float_elements: bool,
    result_ty: &PhpType,
) -> Result<()> {
    match result_ty {
        PhpType::Mixed | PhpType::Union(_) => {
            crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, element);
            Ok(())
        }
        PhpType::Float if !float_elements => {
            abi::emit_int_result_to_float_result(ctx.emitter);
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Emits the AArch64 integer reduction over an indexed array's payload slots.
fn emit_int_reduce_aarch64(
    ctx: &mut FunctionContext<'_>,
    want_max: bool,
    empty_label: &str,
    loop_label: &str,
    reduced_label: &str,
) {
    let exit_label = ctx.next_label("min_max_array_exit");
    let condition = if want_max { "gt" } else { "lt" };
    // -- seed the reduction with the first payload slot --
    ctx.emitter.instruction("ldr x9, [x0]");                                    // x9 = the array's logical element count from its header
    ctx.emitter.instruction(&format!("cbz x9, {}", empty_label));               // an empty array is PHP's ValueError, not a reduction
    ctx.emitter.instruction(&format!("add x10, x0, #{}", ARRAY_DATA_OFFSET));   // x10 = address of the first payload slot
    ctx.emitter.instruction("ldr x11, [x10]");                                  // x11 = running result seeded with element 0
    ctx.emitter.instruction("mov x12, #1");                                     // x12 = cursor starting at the second element
    // -- fold every remaining element into the running result --
    ctx.emitter.label(loop_label);
    ctx.emitter.instruction("cmp x12, x9");                                     // compare the cursor against the element count
    ctx.emitter.instruction(&format!("b.ge {}", exit_label));                   // stop once every element has been folded in
    ctx.emitter.instruction("ldr x13, [x10, x12, lsl #3]");                     // x13 = the payload slot the cursor points at
    ctx.emitter.instruction("cmp x13, x11");                                    // compare the candidate against the running result
    ctx.emitter.instruction(&format!("csel x11, x13, x11, {}", condition));     // keep whichever element wins the min/max comparison
    ctx.emitter.instruction("add x12, x12, #1");                                // advance the cursor to the next payload slot
    abi::emit_jump(ctx.emitter, loop_label);
    ctx.emitter.label(&exit_label);
    ctx.emitter.instruction("mov x0, x11");                                     // publish the reduced element in the integer result register
    abi::emit_jump(ctx.emitter, reduced_label);
}

/// Emits the AArch64 floating reduction over an indexed array's payload slots.
fn emit_float_reduce_aarch64(
    ctx: &mut FunctionContext<'_>,
    want_max: bool,
    empty_label: &str,
    loop_label: &str,
    reduced_label: &str,
) {
    let exit_label = ctx.next_label("min_max_array_exit");
    let select = if want_max { "fmax" } else { "fmin" };
    // -- seed the reduction with the first payload slot --
    ctx.emitter.instruction("ldr x9, [x0]");                                    // x9 = the array's logical element count from its header
    ctx.emitter.instruction(&format!("cbz x9, {}", empty_label));               // an empty array is PHP's ValueError, not a reduction
    ctx.emitter.instruction(&format!("add x10, x0, #{}", ARRAY_DATA_OFFSET));   // x10 = address of the first payload slot
    ctx.emitter.instruction("ldr d0, [x10]");                                   // d0 = running result seeded with element 0
    ctx.emitter.instruction("mov x12, #1");                                     // x12 = cursor starting at the second element
    // -- fold every remaining element into the running result --
    ctx.emitter.label(loop_label);
    ctx.emitter.instruction("cmp x12, x9");                                     // compare the cursor against the element count
    ctx.emitter.instruction(&format!("b.ge {}", exit_label));                   // stop once every element has been folded in
    ctx.emitter.instruction("ldr d1, [x10, x12, lsl #3]");                      // d1 = the payload slot the cursor points at
    ctx.emitter.instruction(&format!("{} d0, d1, d0", select));                 // keep whichever element wins the min/max comparison
    ctx.emitter.instruction("add x12, x12, #1");                                // advance the cursor to the next payload slot
    abi::emit_jump(ctx.emitter, loop_label);
    ctx.emitter.label(&exit_label);
    abi::emit_jump(ctx.emitter, reduced_label);
}

/// Emits the x86_64 integer reduction over an indexed array's payload slots.
fn emit_int_reduce_x86_64(
    ctx: &mut FunctionContext<'_>,
    want_max: bool,
    empty_label: &str,
    loop_label: &str,
    reduced_label: &str,
) {
    let exit_label = ctx.next_label("min_max_array_exit");
    let select = if want_max { "cmovg" } else { "cmovl" };
    // -- seed the reduction with the first payload slot --
    ctx.emitter.instruction("mov r10, QWORD PTR [rax]");                        // r10 = the array's logical element count from its header
    ctx.emitter.instruction("test r10, r10");                                   // check whether the array holds any element at all
    ctx.emitter.instruction(&format!("jz {}", empty_label));                    // an empty array is PHP's ValueError, not a reduction
    ctx.emitter.instruction(&format!("lea r11, [rax + {}]", ARRAY_DATA_OFFSET)); // r11 = address of the first payload slot
    ctx.emitter.instruction("mov rax, QWORD PTR [r11]");                        // rax = running result seeded with element 0
    ctx.emitter.instruction("mov rcx, 1");                                      // rcx = cursor starting at the second element
    // -- fold every remaining element into the running result --
    ctx.emitter.label(loop_label);
    ctx.emitter.instruction("cmp rcx, r10");                                    // compare the cursor against the element count
    ctx.emitter.instruction(&format!("jge {}", exit_label));                    // stop once every element has been folded in
    ctx.emitter.instruction("mov rdx, QWORD PTR [r11 + rcx * 8]");              // rdx = the payload slot the cursor points at
    ctx.emitter.instruction("cmp rdx, rax");                                    // compare the candidate against the running result
    ctx.emitter.instruction(&format!("{} rax, rdx", select));                   // keep whichever element wins the min/max comparison
    ctx.emitter.instruction("add rcx, 1");                                      // advance the cursor to the next payload slot
    abi::emit_jump(ctx.emitter, loop_label);
    ctx.emitter.label(&exit_label);
    abi::emit_jump(ctx.emitter, reduced_label);
}

/// Emits the x86_64 floating reduction over an indexed array's payload slots.
fn emit_float_reduce_x86_64(
    ctx: &mut FunctionContext<'_>,
    want_max: bool,
    empty_label: &str,
    loop_label: &str,
    reduced_label: &str,
) {
    let exit_label = ctx.next_label("min_max_array_exit");
    let select = if want_max { "maxsd" } else { "minsd" };
    // -- seed the reduction with the first payload slot --
    ctx.emitter.instruction("mov r10, QWORD PTR [rax]");                        // r10 = the array's logical element count from its header
    ctx.emitter.instruction("test r10, r10");                                   // check whether the array holds any element at all
    ctx.emitter.instruction(&format!("jz {}", empty_label));                    // an empty array is PHP's ValueError, not a reduction
    ctx.emitter.instruction(&format!("lea r11, [rax + {}]", ARRAY_DATA_OFFSET)); // r11 = address of the first payload slot
    ctx.emitter.instruction("movsd xmm0, QWORD PTR [r11]");                     // xmm0 = running result seeded with element 0
    ctx.emitter.instruction("mov rcx, 1");                                      // rcx = cursor starting at the second element
    // -- fold every remaining element into the running result --
    ctx.emitter.label(loop_label);
    ctx.emitter.instruction("cmp rcx, r10");                                    // compare the cursor against the element count
    ctx.emitter.instruction(&format!("jge {}", exit_label));                    // stop once every element has been folded in
    ctx.emitter.instruction("movsd xmm1, QWORD PTR [r11 + rcx * 8]");           // xmm1 = the payload slot the cursor points at
    ctx.emitter.instruction(&format!("{} xmm0, xmm1", select));                 // keep whichever element wins the min/max comparison
    ctx.emitter.instruction("add rcx, 1");                                      // advance the cursor to the next payload slot
    abi::emit_jump(ctx.emitter, loop_label);
    ctx.emitter.label(&exit_label);
    abi::emit_jump(ctx.emitter, reduced_label);
}
