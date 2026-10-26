//! Purpose:
//! Publishes a heap root hole before releasing its former Mixed owner.
//!
//! Called from:
//! - Generated destructive root removal through a typed compiler operation.
//!
//! Key details:
//! - Copy-on-write moves the property's array owner; it must not be released a second time.
//! - Both the new backing pointer and the null root are visible before any PHP destructor.

use super::*;
use crate::types::spl_heap_layout::{PRIORITIES_OFFSET, VALUES_OFFSET};

/// Retires one data or priority root through its canonical native property slot.
pub(super) fn emit(e: &mut Emitter) {
    let value = result(e);
    let object = if arm64(e) { "x12" } else { "r8" };
    let slot = if arm64(e) { "x11" } else { "r11" };
    let temporary = if arm64(e) { "x10" } else { "r10" };
    e.label_global("__rt_heap_scope_release_root");
    abi::emit_frame_prologue(e, 48);
    abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, 0), 0);
    abi::emit_reg_move(e, temporary, abi::int_arg_reg_name(e.target, 1));
    abi::emit_load_temporary_stack_slot(e, object, 0);
    zero(e, temporary, "__rt_heap_scope_release_root_data");
    ins(e, &format!("add x11, x12, #{PRIORITIES_OFFSET}"), &format!("lea r11, [r8 + {PRIORITIES_OFFSET}]"));
    abi::emit_jump(e, "__rt_heap_scope_release_root_slot");
    e.label("__rt_heap_scope_release_root_data");
    ins(e, &format!("add x11, x12, #{VALUES_OFFSET}"), &format!("lea r11, [r8 + {VALUES_OFFSET}]"));
    e.label("__rt_heap_scope_release_root_slot");
    abi::emit_store_to_sp(e, slot, 8);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 0), slot, 0);
    abi::emit_call_label(e, "__rt_array_ensure_unique");
    abi::emit_load_temporary_stack_slot(e, slot, 8);
    abi::emit_store_to_address(e, value, slot, 0);
    abi::emit_reg_move(e, slot, value);
    abi::emit_load_from_address(e, temporary, slot, 0);
    zero(e, temporary, "__rt_heap_scope_release_root_done");
    abi::emit_load_from_address(e, value, slot, 24);
    abi::emit_load_int_immediate(e, temporary, 0);
    abi::emit_store_to_address(e, temporary, slot, 24);
    abi::emit_call_label(e, "__rt_decref_mixed");
    e.label("__rt_heap_scope_release_root_done");
    abi::emit_frame_restore(e, 48);
    abi::emit_return(e);
}
