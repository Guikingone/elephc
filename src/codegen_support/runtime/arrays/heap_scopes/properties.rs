//! Purpose:
//! Preserves borrowed old property views while a native loader releases and replaces a Mixed slot.
//!
//! Called from:
//! - Typed native property replacement and generated property read paths.
//!
//! Key details:
//! - Records hold no object, array or resource payload owner.
//! - Destructor exceptions are captured and returned after the new value is published.
//! - Scope removal supports nested writes and out-of-order Fiber completion.

use super::*;

const HEAD: &str = "_native_property_scope_head";
const READ_HOOK: &str = "_native_property_read_fn";
const OFFSET: usize = 16;
const SNAPSHOT: usize = 24;
const VALID: usize = 32;
const SIZE: i64 = 48;
const CHILDREN: usize = 40;

mod arrays;
mod hashes;

/// Emits the scope protocol and the protected native replacement operation.
pub(super) fn emit(e: &mut Emitter) {
    arrays::emit(e);
    hashes::emit(e);
    emit_find(e);
    emit_begin(e);
    emit_read(e);
    emit_end(e);
    emit_invalidate(e);
    emit_replace(e);
    emit_shutdown(e);
}

/// Finds the newest borrowed view of exactly one receiver and physical slot.
fn emit_find(e: &mut Emitter) {
    let value = result(e);
    let object = if arm64(e) { "x14" } else { "r8" };
    let offset = if arm64(e) { "x15" } else { "r9" };
    let item = if arm64(e) { "x11" } else { "r11" };
    e.label_global("__rt_native_property_find");
    abi::emit_reg_move(e, object, abi::int_arg_reg_name(e.target, 0));
    abi::emit_reg_move(e, offset, abi::int_arg_reg_name(e.target, 1));
    abi::emit_load_symbol_to_reg(e, value, HEAD, 0);
    e.label("__rt_native_property_find_loop");
    zero(e, value, "__rt_native_property_find_done");
    abi::emit_load_from_address(e, item, value, OBJECT);
    equal(e, item, object, "__rt_native_property_find_offset");
    abi::emit_jump(e, "__rt_native_property_find_next");
    e.label("__rt_native_property_find_offset");
    abi::emit_load_from_address(e, item, value, OFFSET);
    equal(e, item, offset, "__rt_native_property_find_done");
    e.label("__rt_native_property_find_next");
    abi::emit_load_from_address(e, value, value, 0);
    abi::emit_jump(e, "__rt_native_property_find_loop");
    e.label("__rt_native_property_find_done");
    abi::emit_return(e);
}

/// Captures logical bits before the slot owner is retired and publishes a complete node.
fn emit_begin(e: &mut Emitter) {
    let value = result(e);
    let item = if arm64(e) { "x10" } else { "r10" };
    e.label_global("__rt_native_property_begin");
    abi::emit_frame_prologue(e, 80);
    for index in 0..3 {
        abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, index), index * 8);
    }
    arg(e, 0, 16);
    abi::emit_call_label(e, "__rt_heap_scope_capture");
    abi::emit_store_to_sp(e, value, 24);
    abi::emit_load_int_immediate(e, item, 0);
    abi::emit_store_to_sp(e, item, 48);
    arg(e, 0, 24);
    ins(e, "add x1, sp, #48", "lea rsi, [rsp + 48]");
    abi::emit_call_label(e, "__rt_native_array_capture");
    abi::emit_load_temporary_stack_slot(e, value, 48);
    abi::emit_store_to_sp(e, value, 32);
    abi::emit_load_int_immediate(e, value, SIZE);
    abi::emit_call_label(e, "__rt_heap_alloc");
    abi::emit_store_to_sp(e, value, 40);
    abi::emit_load_symbol_to_reg(e, item, HEAD, 0);
    abi::emit_store_to_address(e, item, value, 0);
    for (saved, field) in [(0, OBJECT), (8, OFFSET), (24, SNAPSHOT), (32, CHILDREN)] {
        abi::emit_load_temporary_stack_slot(e, item, saved);
        abi::emit_store_to_address(e, item, value, field);
    }
    abi::emit_load_int_immediate(e, item, 1);
    abi::emit_store_to_address(e, item, value, VALID);
    abi::emit_store_reg_to_symbol(e, value, HEAD, 0);
    abi::emit_symbol_address(e, item, "__rt_native_property_read");
    abi::emit_store_reg_to_symbol(e, item, READ_HOOK, 0);
    abi::emit_load_temporary_stack_slot(e, value, 40);
    abi::emit_frame_restore(e, 80);
    abi::emit_return(e);
}

/// Acquires an owned view without retaining the old Mixed box whose final release is active.
fn emit_read(e: &mut Emitter) {
    let value = result(e);
    let item = if arm64(e) { "x10" } else { "r10" };
    e.label_global("__rt_native_property_read");
    abi::emit_frame_prologue(e, 48);
    for index in 0..2 {
        abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, index), index * 8);
    }
    abi::emit_call_label(e, "__rt_native_property_find");
    zero(e, value, "__rt_native_property_read_physical");
    abi::emit_load_from_address(e, item, value, VALID);
    zero(e, item, "__rt_native_property_read_null");
    abi::emit_reg_move(e, item, value);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 1), item, CHILDREN);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 0), item, SNAPSHOT);
    abi::emit_call_label(e, "__rt_native_value_read");
    abi::emit_jump(e, "__rt_native_property_read_done");
    e.label("__rt_native_property_read_physical");
    abi::emit_load_temporary_stack_slot(e, value, 0);
    abi::emit_load_temporary_stack_slot(e, item, 8);
    ins(e, "ldr x0, [x0, x10]", "mov rax, QWORD PTR [rax + r10]");
    abi::emit_call_label(e, "__rt_incref");
    abi::emit_jump(e, "__rt_native_property_read_done");
    e.label("__rt_native_property_read_null");
    abi::emit_load_int_immediate(e, value, 0);
    abi::emit_call_label(e, "__rt_mixed_clone");
    e.label("__rt_native_property_read_done");
    abi::emit_frame_restore(e, 48);
    abi::emit_return(e);
}

/// Unlinks before releasing inert scratch so no stale node is visible to the allocator observer.
fn emit_end(e: &mut Emitter) {
    let value = result(e);
    let token = if arm64(e) { "x12" } else { "r8" };
    let link = if arm64(e) { "x9" } else { "r10" };
    let cursor = if arm64(e) { "x10" } else { "r11" };
    let next = if arm64(e) { "x11" } else { "rdx" };
    e.label_global("__rt_native_property_end");
    abi::emit_frame_prologue(e, 32);
    abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, 0), 0);
    abi::emit_reg_move(e, token, abi::int_arg_reg_name(e.target, 0));
    abi::emit_symbol_address(e, link, HEAD);
    e.label("__rt_native_property_end_search");
    abi::emit_load_from_address(e, cursor, link, 0);
    zero(e, cursor, "__rt_native_property_end_done");
    equal(e, cursor, token, "__rt_native_property_end_found");
    abi::emit_reg_move(e, link, cursor);
    abi::emit_jump(e, "__rt_native_property_end_search");
    e.label("__rt_native_property_end_found");
    abi::emit_load_from_address(e, next, cursor, 0);
    abi::emit_store_to_address(e, next, link, 0);
    abi::emit_load_symbol_to_reg(e, value, HEAD, 0);
    zero(e, value, "__rt_native_property_end_clear");
    abi::emit_jump(e, "__rt_native_property_end_drop");
    e.label("__rt_native_property_end_clear");
    abi::emit_store_zero_to_symbol(e, READ_HOOK, 0);
    e.label("__rt_native_property_end_drop");
    abi::emit_load_temporary_stack_slot(e, value, 0);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 0), value, CHILDREN);
    abi::emit_call_label(e, "__rt_native_array_drop");
    abi::emit_load_temporary_stack_slot(e, value, 0);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 0), value, SNAPSHOT);
    abi::emit_call_label(e, "__rt_heap_scope_drop_capture");
    abi::emit_load_temporary_stack_slot(e, value, 0);
    abi::emit_call_label(e, "__rt_heap_free");
    e.label("__rt_native_property_end_done");
    abi::emit_frame_restore(e, 32);
    abi::emit_return(e);
}

/// Marks borrowed payloads dead before allocator reuse, without allocation or PHP callbacks.
fn emit_invalidate(e: &mut Emitter) {
    let node = if arm64(e) { "x9" } else { "r10" };
    let snapshot = if arm64(e) { "x10" } else { "r11" };
    let tag = if arm64(e) { "x11" } else { "rsi" };
    let address = if arm64(e) { "x12" } else { "rdx" };
    let memo = if arm64(e) { "x14" } else { "r9" };
    let count = if arm64(e) { "x15" } else { "rcx" };
    let index = if arm64(e) { "x16" } else { "r8" };
    e.label_global("__rt_native_property_invalidate");
    abi::emit_load_symbol_to_reg(e, node, HEAD, 0);
    e.label("__rt_native_property_invalidate_loop");
    zero(e, node, "__rt_native_property_invalidate_done");
    abi::emit_load_from_address(e, snapshot, node, SNAPSHOT);
    invalidate_snapshot(e, snapshot, tag, address, "root");
    abi::emit_load_from_address(e, memo, node, CHILDREN);
    e.label("__rt_native_property_invalidate_memo");
    zero(e, memo, "__rt_native_property_invalidate_next");
    abi::emit_load_from_address(e, count, memo, 16);
    abi::emit_load_int_immediate(e, index, 0);
    e.label("__rt_native_property_invalidate_entry");
    ins(e, &format!("cmp {index}, {count}"), &format!("cmp {index}, {count}"));
    ins(e, "b.hs __rt_native_property_invalidate_next_memo", "jae __rt_native_property_invalidate_next_memo");
    ins(e, "add x10, x14, x16, lsl #4", "mov r11, r8");
    if !arm64(e) { ins(e, "", "shl r11, 4"); ins(e, "", "add r11, r9"); }
    abi::emit_load_from_address(e, snapshot, snapshot, 40);
    invalidate_snapshot(e, snapshot, tag, address, "child");
    ins(e, &format!("add {index}, {index}, #1"), &format!("add {index}, 1"));
    abi::emit_jump(e, "__rt_native_property_invalidate_entry");
    e.label("__rt_native_property_invalidate_next_memo");
    abi::emit_load_from_address(e, memo, memo, 0);
    abi::emit_jump(e, "__rt_native_property_invalidate_memo");
    e.label("__rt_native_property_invalidate_next");
    abi::emit_load_from_address(e, node, node, 0);
    abi::emit_jump(e, "__rt_native_property_invalidate_loop");
    e.label("__rt_native_property_invalidate_done");
    abi::emit_return(e);
}

/// Invalidates one raw snapshot before a heap identity can be allocated for a different value.
fn invalidate_snapshot(e: &mut Emitter, snapshot: &str, tag: &str, address: &str, suffix: &str) {
    let stem = format!("__rt_native_property_invalidate_{suffix}");
    let freed = abi::int_arg_reg_name(e.target, 0);
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
    abi::emit_load_int_immediate(e, address, 8);
    abi::emit_store_to_address(e, address, snapshot, 8);
    abi::emit_load_int_immediate(e, address, 0);
    abi::emit_store_to_address(e, address, snapshot, 16);
    abi::emit_store_to_address(e, address, snapshot, 24);
    e.label(&format!("{stem}_done"));
}

/// Captures destructor errors, publishes the replacement, and returns an owned Throwable or null.
fn emit_replace(e: &mut Emitter) {
    let value = result(e);
    let base = if arm64(e) { "x10" } else { "r10" };
    let offset = if arm64(e) { "x11" } else { "r11" };
    e.label_global("__rt_native_property_replace");
    abi::emit_frame_prologue(e, 128);
    for index in 0..3 {
        abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, index), index * 8);
    }
    abi::emit_load_symbol_to_reg(e, base, "_gc_release_suppressed", 0);
    abi::emit_store_to_sp(e, base, 72);
    abi::emit_load_int_immediate(e, base, 0);
    abi::emit_store_to_sp(e, base, 64);
    abi::emit_load_int_immediate(e, base, 1);
    abi::emit_store_reg_to_symbol(e, base, "_gc_release_suppressed", 0);
    abi::emit_load_temporary_stack_slot(e, base, 0);
    abi::emit_load_temporary_stack_slot(e, offset, 8);
    ins(e, "ldr x2, [x10, x11]", "mov rdx, QWORD PTR [r10 + r11]");
    abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, 2), 24);
    arg(e, 0, 0);
    arg(e, 1, 8);
    abi::emit_call_label(e, "__rt_native_property_begin");
    abi::emit_store_to_sp(e, value, 32);
    abi::emit_load_temporary_stack_slot(e, value, 16);
    abi::emit_call_label(e, "__rt_incref");
    abi::emit_load_temporary_stack_slot(e, value, 24);
    crate::codegen_support::runtime::exceptions::emit_guarded_cleanup_call(
        e, "__rt_decref_mixed", value, 48,
    );
    abi::emit_load_temporary_stack_slot(e, base, 0);
    abi::emit_load_temporary_stack_slot(e, offset, 8);
    abi::emit_load_temporary_stack_slot(e, value, 16);
    ins(e, "str x0, [x10, x11]", "mov QWORD PTR [r10 + r11], rax");
    abi::emit_load_int_immediate(e, value, 0);
    ins(e, "add x11, x11, #8", "add r11, 8");
    ins(e, "str x0, [x10, x11]", "mov QWORD PTR [r10 + r11], rax");
    arg(e, 0, 32);
    abi::emit_call_label(e, "__rt_native_property_end");
    abi::emit_load_temporary_stack_slot(e, base, 64);
    zero(e, base, "__rt_native_property_replace_null");
    abi::emit_reg_move(e, if arm64(e) { "x1" } else { "rdi" }, base);
    abi::emit_load_int_immediate(e, value, 6);
    abi::emit_load_int_immediate(e, if arm64(e) { "x2" } else { "rsi" }, 0);
    abi::emit_call_label(e, "__rt_mixed_from_value");
    abi::emit_store_to_sp(e, value, 80);
    abi::emit_load_temporary_stack_slot(e, value, 64);
    abi::emit_call_label(e, "__rt_decref_any");
    abi::emit_jump(e, "__rt_native_property_replace_return");
    e.label("__rt_native_property_replace_null");
    abi::emit_load_int_immediate(e, value, 0);
    abi::emit_call_label(e, "__rt_mixed_clone");
    abi::emit_store_to_sp(e, value, 80);
    e.label("__rt_native_property_replace_return");
    abi::emit_load_temporary_stack_slot(e, base, 72);
    abi::emit_store_reg_to_symbol(e, base, "_gc_release_suppressed", 0);
    abi::emit_load_temporary_stack_slot(e, value, 80);
    abi::emit_frame_restore(e, 128);
    abi::emit_return(e);
}

/// Drains raw scopes abandoned by non-Throwable process exits after ordinary PHP cleanup.
fn emit_shutdown(e: &mut Emitter) {
    e.label_global("__rt_native_property_shutdown");
    abi::emit_frame_prologue(e, 16);
    e.label("__rt_native_property_shutdown_loop");
    abi::emit_load_symbol_to_reg(e, abi::int_arg_reg_name(e.target, 0), HEAD, 0);
    zero(e, abi::int_arg_reg_name(e.target, 0), "__rt_native_property_shutdown_done");
    abi::emit_call_label(e, "__rt_native_property_end");
    abi::emit_jump(e, "__rt_native_property_shutdown_loop");
    e.label("__rt_native_property_shutdown_done");
    abi::emit_frame_restore(e, 16);
    abi::emit_return(e);
}
