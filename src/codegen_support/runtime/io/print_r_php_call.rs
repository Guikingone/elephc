//! Purpose:
//! Detaches native print_r captures while arbitrary PHP code executes.
//! Restores each suspended call's capture from its own native stack frame.
//!
//! Called from:
//! - Print_r output flushing, argument/view cleanup and magic debug callbacks.
//!
//! Key details:
//! - The unary callback ABI matches the existing guarded cleanup invoker.
//! - No active capture or scope pointer escapes into nested PHP or another Fiber.
//! - Saved buffers remain owned by their enclosing render activation.

use crate::codegen_support::{abi, emit::Emitter};

const FRAME: usize = 96;
const CAUGHT: usize = 72;
const GLOBALS: [(&str, usize); 5] = [
    ("_print_r_mode", 32), ("_print_r_buf", 40), ("_print_r_cap", 48),
    ("_print_r_off", 56), ("_print_r_scope", 64),
];

/// Emits a nonescaping callback boundary with stack-owned capture restoration.
pub fn emit_print_r_php_call(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_print_r_php_call");
    abi::emit_frame_prologue(emitter, FRAME);
    for index in 0..3 {
        abi::store_at_offset(emitter, abi::int_arg_reg_name(emitter.target, index), (index + 1) * 8);
    }
    for (symbol, slot) in GLOBALS {
        abi::emit_load_symbol_to_reg(emitter, result, symbol, 0);
        abi::store_at_offset(emitter, result, slot);
        abi::emit_store_zero_to_symbol(emitter, symbol, 0);
    }
    for index in 0..3 {
        abi::load_at_offset(emitter, abi::int_arg_reg_name(emitter.target, index), (index + 1) * 8);
    }
    abi::emit_call_label(emitter, "__rt_cleanup_invoke");
    abi::store_at_offset(emitter, result, CAUGHT);
    for (symbol, slot) in GLOBALS {
        abi::load_at_offset(emitter, result, slot);
        abi::emit_store_reg_to_symbol(emitter, result, symbol, 0);
    }
    abi::load_at_offset(emitter, result, CAUGHT);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
}

/// Guards a unary PHP callback while leaving native formatting state private.
pub(super) fn emit_guarded_php_call(
    emitter: &mut Emitter, entry: &str, payload: &str, pending_slot: usize,
) {
    let argument = abi::int_arg_reg_name(emitter.target, 1);
    abi::emit_reg_move(emitter, argument, payload);
    abi::emit_symbol_address(emitter, abi::int_arg_reg_name(emitter.target, 0), entry);
    abi::emit_frame_slot_address(emitter, abi::int_arg_reg_name(emitter.target, 2), pending_slot);
    abi::emit_call_label(emitter, "__rt_print_r_php_call");
}
