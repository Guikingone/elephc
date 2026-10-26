//! Purpose:
//! Invalidates borrowed root payloads before an allocation can be reused.
//!
//! Called from:
//! - The validated allocator free path on both architectures.
//!
//! Key details:
//! - Uses no allocation, callback or payload release.
//! - Preserves the allocator's live registers and original stack parity.
//! - Resource identity follows its canonical cell; other heap values follow the child payload.

use super::*;

/// Emits the allocation observer, invalidating every matching active borrowed root.
pub(super) fn emit(e: &mut Emitter) {
    let node = if arm64(e) { "x9" } else { "r10" };
    let snapshot = if arm64(e) { "x10" } else { "r11" };
    let tag = if arm64(e) { "x11" } else { "rcx" };
    let address = if arm64(e) { "x12" } else { "rdx" };
    let flags = if arm64(e) { "x13" } else { "r8" };
    let freed = abi::int_arg_reg_name(e.target, 0);
    e.label_global("__rt_heap_scope_invalidate");
    abi::emit_load_symbol_to_reg(e, node, HEAD, 0);
    e.label("__rt_heap_scope_invalidate_loop");
    zero(e, node, "__rt_heap_scope_invalidate_done");
    for (offset, valid, suffix) in [(DATA, DATA_VALID, "data"), (PRIORITY, PRIORITY_VALID, "priority")] {
        let stem = format!("__rt_heap_scope_invalidate_{suffix}");
        abi::emit_load_from_address(e, snapshot, node, offset);
        zero(e, snapshot, &format!("{stem}_done"));
        abi::emit_load_from_address(e, flags, node, valid);
        zero(e, flags, &format!("{stem}_done"));
        abi::emit_load_from_address(e, tag, snapshot, 8);
        tag_is(e, tag, 9, &format!("{stem}_resource"));
        for category in [4, 5, 6, 10] { tag_is(e, tag, category, &format!("{stem}_payload")); }
        abi::emit_jump(e, &format!("{stem}_done"));
        e.label(&format!("{stem}_resource"));
        abi::emit_load_from_address(e, address, snapshot, 0);
        abi::emit_jump(e, &format!("{stem}_compare"));
        e.label(&format!("{stem}_payload"));
        abi::emit_load_from_address(e, address, snapshot, 16);
        e.label(&format!("{stem}_compare"));
        equal(e, address, freed, &format!("{stem}_dead"));
        abi::emit_jump(e, &format!("{stem}_done"));
        e.label(&format!("{stem}_dead"));
        abi::emit_load_int_immediate(e, flags, 0);
        abi::emit_store_to_address(e, flags, node, valid);
        e.label(&format!("{stem}_done"));
    }
    abi::emit_load_from_address(e, node, node, 0);
    abi::emit_jump(e, "__rt_heap_scope_invalidate_loop");
    e.label("__rt_heap_scope_invalidate_done");
    abi::emit_jump(e, "__rt_native_property_invalidate");
}

/// Calls the leaf observer without changing any allocator working register or linkage.
pub(super) fn emit_free_observer(e: &mut Emitter) {
    if arm64(e) {
        e.instruction("sub sp, sp, #160");                                      // preserve the volatile register set used by borrowed-view invalidation
        let registers = (0..=18).map(|index| format!("x{index}")).chain(["x30".to_string()]).collect::<Vec<_>>();
        for (index, register) in registers.iter().enumerate() { abi::emit_store_to_sp(e, register, index * 8); }
        abi::emit_call_label(e, "__rt_heap_scope_invalidate");
        for (index, register) in registers.iter().enumerate() { abi::emit_load_temporary_stack_slot(e, register, index * 8); }
        e.instruction("add sp, sp, #160");                                      // restore the allocator's original registers and stack position
    } else {
        e.instruction("sub rsp, 72");                                           // align the observer call and preserve every volatile integer register
        let registers = ["rax", "rdi", "rsi", "rcx", "rdx", "r8", "r9", "r10", "r11"];
        for (index, register) in registers.iter().enumerate() { abi::emit_store_to_sp(e, register, index * 8); }
        abi::emit_reg_move(e, "rdi", "rax");
        abi::emit_call_label(e, "__rt_heap_scope_invalidate");
        for (index, register) in registers.iter().enumerate() { abi::emit_load_temporary_stack_slot(e, register, index * 8); }
        e.instruction("add rsp, 72");                                           // restore the allocator's entry parity and volatile integer registers
    }
}
