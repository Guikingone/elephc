//! Purpose:
//! Keeps a non-owning logical value snapshot across SPL root retirement callbacks.
//!
//! Called from:
//! - Compiler-private SPL heap methods through typed EIR runtime calls.
//!
//! Key details:
//! - Scratch owns its allocation only; it never retains the captured PHP payload.
//! - Reads acquire a real owner while the captured value is still live.
//! - Resources reuse their original cell instead of treating scratch as a refcounted cell.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits a scoped snapshot, an owned read, and explicit scratch retirement.
pub fn emit_borrowed_value(emitter: &mut Emitter) {
    emit_snapshot(emitter);
    emit_read(emitter);
    emitter.label_global("__rt_borrowed_value_free");
    let result = abi::int_result_reg(emitter);
    abi::emit_reg_move(emitter, result, abi::int_arg_reg_name(emitter.target, 0));
    abi::emit_jump(emitter, "__rt_heap_free");
}

/// Copies the dereferenced tag and payload without acquiring a payload owner.
fn emit_snapshot(e: &mut Emitter) {
    let result = abi::int_result_reg(e);
    let arm = e.target.arch == Arch::AArch64;
    let lo = if arm { "x1" } else { "rdi" };
    let hi = if arm { "x2" } else { "rdx" };
    let scratch = if arm { "x10" } else { "r10" };
    e.label_global("__rt_borrowed_value_snapshot");
    abi::emit_frame_prologue(e, 48);
    abi::emit_reg_move(e, result, abi::int_arg_reg_name(e.target, 0));
    abi::emit_call_label(e, "__rt_mixed_deref");
    abi::emit_store_to_sp(e, result, 0);
    abi::emit_call_label(e, "__rt_mixed_unbox");
    abi::emit_store_to_sp(e, result, 8);
    abi::emit_store_to_sp(e, lo, 16);
    abi::emit_store_to_sp(e, hi, 24);
    abi::emit_load_int_immediate(e, result, 32);
    abi::emit_call_label(e, "__rt_heap_alloc");
    for offset in [0, 8, 16, 24] {
        abi::emit_load_temporary_stack_slot(e, scratch, offset);
        abi::emit_store_to_address(e, scratch, result, offset);
    }
    abi::emit_frame_restore(e, 48);
    abi::emit_return(e);
}

/// Materializes an owned value without letting a pointer into scratch escape.
fn emit_read(e: &mut Emitter) {
    let result = abi::int_result_reg(e);
    let input = abi::int_arg_reg_name(e.target, 0);
    let arm = e.target.arch == Arch::AArch64;
    let lo = if arm { "x1" } else { "rdi" };
    let hi = if arm { "x2" } else { "rsi" };
    let scratch = if arm { "x10" } else { "r10" };
    e.label_global("__rt_borrowed_value_read");
    abi::emit_frame_prologue(e, 32);
    abi::emit_store_to_sp(e, input, 0);
    abi::emit_reg_move(e, scratch, input);
    abi::emit_load_from_address(e, result, scratch, 8);
    if arm {
        e.instruction("cmp x0, #9");                                            // resources require the original cell's ownership and identity
        e.instruction("b.eq __rt_borrowed_value_read_resource");                // never retain or return a pointer into the scratch allocation
    } else {
        e.instruction("cmp rax, 9");                                            // resources require the original cell's ownership and identity
        e.instruction("je __rt_borrowed_value_read_resource");                  // never retain or return a pointer into the scratch allocation
    }
    abi::emit_load_from_address(e, lo, scratch, 16);
    abi::emit_load_from_address(e, hi, scratch, 24);
    abi::emit_call_label(e, "__rt_mixed_from_value");
    abi::emit_jump(e, "__rt_borrowed_value_read_done");
    e.label("__rt_borrowed_value_read_resource");
    abi::emit_load_from_address(e, result, scratch, 0);
    abi::emit_store_to_sp(e, result, 8);
    abi::emit_call_label(e, "__rt_incref");
    abi::emit_load_temporary_stack_slot(e, result, 8);
    e.label("__rt_borrowed_value_read_done");
    abi::emit_frame_restore(e, 32);
    abi::emit_return(e);
}
