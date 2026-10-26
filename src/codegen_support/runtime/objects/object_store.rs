//! Purpose:
//! Looks up current object-store entries by PHP handle and retires arena-local identity state.
//!
//! Called from:
//! - `super::handles::emit_object_handles()` and the shutdown destructor visitor.
//! - The full web arena reset after all old request owners have been retired.
//!
//! Key details:
//! - Inverse entries borrow payload pointers; they neither retain objects nor become GC roots.
//! - Acquisition and final heap release update both directions before handle reuse.
//! - Lookup bounds-checks both the logical cursor and the reserved table capacity.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits the non-owning lookup and full-arena identity reset for every supported target.
pub(super) fn emit_object_store(emitter: &mut Emitter) {
    emit_lookup(emitter);
    emit_reset(emitter);
}

/// Looks up a live payload from an integer handle in the internal integer result register.
fn emit_lookup(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    let base = abi::secondary_scratch_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_object_from_handle");
    abi::emit_branch_if_int_result_zero(emitter, "__rt_object_from_handle_missing");
    for count in ["_obj_handle_next", "_obj_handle_capacity"] {
        abi::emit_cmp_reg_to_symbol(emitter, result, count);
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction("b.hs __rt_object_from_handle_missing");    // reject unknown handles before reading inverse storage
            }
            Arch::X86_64 => {
                emitter.instruction("jae __rt_object_from_handle_missing");     // reject unknown handles before reading inverse storage
            }
        }
    }
    abi::emit_load_symbol_to_reg(emitter, base, "_obj_handle_live_ptr", 0);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("cbnz x10, __rt_object_from_handle_ready");     // grown storage replaces the initial borrowed-pointer table
        }
        Arch::X86_64 => {
            emitter.instruction("test r10, r10");                               // inspect the optional grown inverse storage
            emitter.instruction("jnz __rt_object_from_handle_ready");           // prefer the current dynamic table when present
        }
    }
    abi::emit_symbol_address(emitter, base, "_obj_handle_live");
    emitter.label("__rt_object_from_handle_ready");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("ldr x0, [x10, x0, lsl #3]");                   // borrow the current payload under this PHP handle
        }
        Arch::X86_64 => {
            emitter.instruction("mov rax, QWORD PTR [r10 + rax * 8]");          // borrow the current payload under this PHP handle
        }
    }
    abi::emit_return(emitter);
    emitter.label("__rt_object_from_handle_missing");
    abi::emit_load_int_immediate(emitter, result, 0);
    abi::emit_return(emitter);
}

/// Clears borrowed identities before the whole old arena is reused by the next request.
fn emit_reset(emitter: &mut Emitter) {
    emitter.blank();
    emitter.label_global("__rt_object_handles_reset");
    abi::emit_frame_prologue(emitter, 16);
    abi::emit_load_symbol_to_reg(emitter, abi::int_arg_reg_name(emitter.target, 0), "_obj_handle_live_ptr", 0);
    abi::emit_store_zero_to_symbol(emitter, "_obj_handle_live_ptr", 0);
    emitter.bl_c("free");                                                       // discard only unmanaged metadata after retiring old request owners
    abi::emit_load_symbol_to_reg(emitter, abi::int_result_reg(emitter), "_obj_handle_initial_capacity", 0);
    abi::emit_store_reg_to_symbol(emitter, abi::int_result_reg(emitter), "_obj_handle_capacity", 0);
    for (symbol, count, shift) in [
        ("_obj_handle_index", "_obj_handle_index_count", 2),
        ("_obj_weak_reference_index", "_obj_handle_index_count", 3),
        ("_obj_handle_live", "_obj_handle_capacity", 3),
    ] {
        abi::emit_symbol_address(emitter, abi::int_arg_reg_name(emitter.target, 0), symbol);
        abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(emitter.target, 1), 0);
        abi::emit_load_symbol_to_reg(emitter, abi::int_arg_reg_name(emitter.target, 2), count, 0);
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction(&format!("lsl x2, x2, #{shift}"));          // convert table entries to the exact byte length
            }
            Arch::X86_64 => {
                emitter.instruction(&format!("shl rdx, {shift}"));              // convert table entries to the exact byte length
            }
        }
        emitter.bl_c("memset");                                                 // retire only metadata after all old heap owners are gone
    }
    abi::emit_store_zero_to_symbol(emitter, "_obj_handle_free_top", 0);
    abi::emit_store_zero_to_symbol(emitter, "_obj_handle_shutdown", 0);
    let result = abi::int_result_reg(emitter);
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::emit_store_reg_to_symbol(emitter, result, "_obj_handle_next", 0);
    abi::emit_frame_restore(emitter, 16);
    abi::emit_return(emitter);
}
