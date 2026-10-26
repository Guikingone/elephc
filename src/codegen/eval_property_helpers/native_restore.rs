//! Purpose:
//! Resolves serialized native SPL property names without applying ordinary write type checks.
//!
//! Called from:
//! - User assembly finalization when a trusted NativePropertyRestore operation is present.
//!
//! Key details:
//! - Runtime class dispatch selects physical slots, including hidden private ancestor fields.
//! - Initialized readonly slots throw before replacement; destructor failures remain pending results.
//! - The ordinary checked hydration and eval writers keep their existing contracts.

use super::*;
use crate::ir::{Immediate, RuntimeCallTarget};

/// Emits the module-specific name dispatcher only for trusted native loading.
pub(super) fn emit(module: &Module, e: &mut Emitter, data: &mut DataSection) {
    let used = module.functions.iter().chain(module.class_methods.iter()).chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter()).chain(module.callback_wrappers.iter())
        .chain(module.extern_callback_trampolines.iter()).chain(module.runtime_callable_invokers.iter())
        .any(|f| f.instructions.iter().any(|i| matches!(i.immediate,
            Some(Immediate::RuntimeCall(RuntimeCallTarget::NativePropertyRestore)))));
    if !used { return; }
    let slots = collect_eval_property_slots(module).into_iter()
        .filter(|s| !s.property.starts_with('\0')
            && crate::types::spl_heap_layout::has_native_prefix(&module.class_infos[&s.class_name]))
        .collect::<Vec<_>>();
    let mut classes = module.class_infos.iter()
        .filter(|(_, info)| crate::types::spl_heap_layout::has_native_prefix(info)).collect::<Vec<_>>();
    classes.sort_by_key(|(_, info)| info.class_id);
    e.label_global("__rt_native_property_restore");
    abi::emit_frame_prologue(e, 128);
    for i in 0..4 { abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, i), i * 8); }
    normalize_name(e);
    for (name, info) in &classes {
        let next = format!("__rt_native_restore_next_{}", info.class_id);
        let a = scratch(e);
        let b = scratch2(e);
        load(e, a, 0);
        abi::emit_load_from_address(e, a, a, 0);
        abi::emit_load_int_immediate(e, b, info.class_id as i64);
        compare(e, a, b, &next, false);
        for slot in slots.iter().filter(|s| s.class_id == info.class_id) {
            match_slot(e, data, slot);
        }
        dynamic(e, data, name, info);
        e.label(&next);
    }
    throw_error(e, data, "Invalid native property receiver");
    for slot in &slots {
        e.label(&body_label(slot));
        if slot.is_reference {
            // Typed reference storage requires its own detachment representation before hydration.
            throw_error(e, data, "Native reference property detachment is not implemented");
            continue;
        }
        let info = &module.class_infos[&slot.class_name];
        let index = (slot.offset - 8) / 16;
        assert!(info.native_property_contracts.contains_key(&index));
        assert_eq!(slot.ty.codegen_repr(), PhpType::Mixed);
        let readonly = module.class_infos.get(&slot.declaring_class)
            .is_some_and(|i| i.readonly_properties.contains(&slot.property));
        if readonly {
            let ready = format!("{}_uninitialized", body_label(slot));
            let a = scratch(e);
            let b = scratch2(e);
            load(e, a, 0);
            abi::emit_load_from_address(e, a, a, slot.offset + 8);
            abi::emit_load_int_immediate(e, b, UNINITIALIZED_TYPED_PROPERTY_SENTINEL as i64);
            compare(e, a, b, &ready, true);
            throw_error(e, data, &format!("Cannot modify readonly property {}::${}", slot.declaring_class, slot.property));
            e.label(&ready);
        }
        argument(e, 0, 0);
        abi::emit_load_int_immediate(e, abi::int_arg_reg_name(e.target, 1), slot.offset as i64);
        argument(e, 2, 24);
        abi::emit_call_label(e, "__rt_native_property_replace");
        abi::emit_jump(e, "__rt_native_restore_done");
    }
    e.label("__rt_native_restore_null");
    abi::emit_load_int_immediate(e, abi::int_result_reg(e), 0);
    abi::emit_call_label(e, "__rt_mixed_clone");
    e.label("__rt_native_restore_done");
    abi::emit_frame_restore(e, 128);
    abi::emit_return(e);
    e.label("__rt_native_restore_dynamic_set");
    let context = scratch(e);
    abi::emit_reg_move(e, context, abi::int_arg_reg_name(e.target, 0));
    for (i, offset) in [40, 8, 16, 24].into_iter().enumerate() {
        abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, i), context, offset);
    }
    abi::emit_jump(e, "__rt_property_hash_set");
}

/// Demangles valid names for declared-slot lookup, preserving original bytes for dynamic keys.
fn normalize_name(e: &mut Emitter) {
    let a = scratch(e);
    let b = scratch2(e);
    let c = if arm(e) { "x12" } else { "r8" };
    let d = if arm(e) { "x13" } else { "r9" };
    let done = "__rt_native_restore_name_ready";
    load(e, a, 8);
    load(e, b, 16);
    abi::emit_store_to_sp(e, a, 88);
    abi::emit_store_to_sp(e, b, 96);
    abi::emit_load_int_immediate(e, c, 0);
    abi::emit_store_to_sp(e, c, 32);
    abi::emit_store_to_sp(e, c, 56);
    branch_zero(e, b, done);
    ins(e, "ldrb w13, [x10]", "movzx r9d, BYTE PTR [r10]");
    branch_nonzero(e, d, done);
    abi::emit_load_int_immediate(e, c, 1);
    e.label("__rt_native_restore_name_scan");
    ins(e, "cmp x12, x11", "cmp r8, r11");
    ins(e, &format!("b.hs {done}"), &format!("jae {done}"));
    ins(e, "ldrb w13, [x10, x12]", "movzx r9d, BYTE PTR [r10 + r8]");
    branch_zero(e, d, "__rt_native_restore_name_found");
    ins(e, "add x12, x12, #1", "add r8, 1");
    abi::emit_jump(e, "__rt_native_restore_name_scan");
    e.label("__rt_native_restore_name_found");
    abi::emit_load_int_immediate(e, d, 1);
    compare(e, c, d, done, true);
    ins(e, "add x13, x10, #1", "lea r9, [r10 + 1]");
    abi::emit_store_to_sp(e, d, 32);
    ins(e, "sub x13, x12, #1", "lea r9, [r8 - 1]");
    abi::emit_store_to_sp(e, d, 56);
    ins(e, "add x10, x10, x12", "add r10, r8");
    ins(e, "add x10, x10, #1", "add r10, 1");
    abi::emit_store_to_sp(e, a, 88);
    ins(e, "sub x11, x11, x12", "sub r11, r8");
    ins(e, "sub x11, x11, #1", "sub r11, 1");
    abi::emit_store_to_sp(e, b, 96);
    e.label(done);
}

/// Matches case-sensitive property bytes and, for private slots, the case-insensitive class scope.
fn match_slot(e: &mut Emitter, data: &mut DataSection, slot: &EvalPropertySlot) {
    let (label, len) = data.add_string(slot.property.as_bytes());
    let skip = format!("{}_name_miss", body_label(slot));
    let regs = if arm(e) { ["x1", "x2", "x3", "x4"] } else { ["rdi", "rsi", "rdx", "rcx"] };
    load(e, regs[0], 88);
    load(e, regs[1], 96);
    abi::emit_symbol_address(e, regs[2], &label);
    abi::emit_load_int_immediate(e, regs[3], len as i64);
    abi::emit_call_label(e, "__rt_str_eq");
    branch_zero(e, abi::int_result_reg(e), &skip);
    if slot.visibility == Visibility::Private {
        let class = slot.declaring_class.as_bytes();
        let a = scratch(e);
        let b = scratch2(e);
        load(e, a, 56);
        abi::emit_load_int_immediate(e, b, class.len() as i64);
        compare(e, a, b, &skip, false);
        load(e, b, 32);
        for (i, byte) in class.iter().copied().enumerate() {
            ins(e, &format!("ldrb w10, [x11, #{i}]"), &format!("movzx r10d, BYTE PTR [r11 + {i}]"));
            if byte.is_ascii_alphabetic() { ins(e, "orr w10, w10, #32", "or r10d, 32"); }
            ins(e, &format!("cmp w10, #{}", byte.to_ascii_lowercase()), &format!("cmp r10d, {}", byte.to_ascii_lowercase()));
            ins(e, &format!("b.ne {skip}"), &format!("jne {skip}"));
        }
    }
    abi::emit_jump(e, &body_label(slot));
    e.label(&skip);
}

/// Restores unmatched names through the receiver's GC-visible dynamic hash.
fn dynamic(e: &mut Emitter, data: &mut DataSection, name: &str, info: &ClassInfo) {
    if !info.has_property_hash_storage() {
        throw_error(e, data, "Native property loading requires dynamic property storage");
        return;
    }
    let a = scratch(e);
    let b = scratch2(e);
    load(e, a, 0);
    abi::emit_load_int_immediate(e, b, (8 + info.properties.len() * 16) as i64);
    ins(e, &format!("add {a}, {a}, {b}"), &format!("add {a}, {b}"));
    abi::emit_store_to_sp(e, a, 40);
    // Capture a dynamic writer error without unwinding past the native property loop.
    // Exact publication and borrowed old-entry views still need the dedicated dynamic protocol.
    abi::emit_load_int_immediate(e, a, 0);
    abi::emit_store_to_sp(e, a, 48);
    load(e, abi::int_result_reg(e), 24);
    abi::emit_call_label(e, "__rt_incref");
    let context = abi::int_arg_reg_name(e.target, 0);
    ins(e, &format!("mov {context}, sp"), &format!("mov {context}, rsp"));
    crate::codegen_support::runtime::emit_guarded_cleanup_call(
        e, "__rt_native_restore_dynamic_set", abi::int_arg_reg_name(e.target, 0), 64,
    );
    load(e, a, 48);
    let clean = format!("__rt_native_restore_dynamic_clean_{}", info.class_id);
    branch_zero(e, a, &clean);
    let payload = if arm(e) { "x1" } else { "rdi" };
    abi::emit_reg_move(e, payload, a);
    abi::emit_load_int_immediate(e, if arm(e) { "x2" } else { "rsi" }, 0);
    abi::emit_load_int_immediate(e, abi::int_result_reg(e), 6);
    abi::emit_call_label(e, "__rt_mixed_from_value");
    abi::emit_store_to_sp(e, abi::int_result_reg(e), 64);
    load(e, abi::int_result_reg(e), 48);
    abi::emit_call_label(e, "__rt_decref_any");
    load(e, abi::int_result_reg(e), 64);
    abi::emit_jump(e, "__rt_native_restore_done");
    e.label(&clean);
    let _ = (data, name); // Diagnostics for dynamic creation are connected with the public loader.
    abi::emit_jump(e, "__rt_native_restore_null");
}

/// Constructs a normal catchable Error without an eval context or parser-owned cleanup.
fn throw_error(e: &mut Emitter, data: &mut DataSection, message: &str) {
    let (label, len) = data.add_string(message.as_bytes());
    let (pointer, length) = abi::string_result_regs(e);
    abi::emit_symbol_address(e, pointer, &label);
    abi::emit_load_int_immediate(e, length, len as i64);
    abi::emit_call_label(e, "__rt_str_persist");
    abi::emit_store_to_sp(e, pointer, 72);
    abi::emit_store_to_sp(e, length, 80);
    abi::emit_load_int_immediate(e, abi::int_result_reg(e), 56);
    abi::emit_call_label(e, "__rt_heap_alloc");
    let object = abi::int_result_reg(e);
    let a = scratch(e);
    abi::emit_load_int_immediate(e, a, if arm(e) { 6 } else {
        crate::codegen_support::sentinels::x86_64_heap_kind_word(6) as i64
    });
    ins(e, &format!("stur {a}, [{object}, #-8]"), &format!("mov QWORD PTR [{object} - 8], {a}"));
    abi::emit_call_label(e, "__rt_object_handle_acquire");
    abi::emit_load_symbol_to_reg(e, a, "_spl_error_class_id", 0);
    abi::emit_store_to_address(e, a, object, 0);
    for (saved, offset) in [(72, 8), (80, 16)] {
        load(e, a, saved);
        abi::emit_store_to_address(e, a, object, offset);
    }
    abi::emit_load_int_immediate(e, a, 0);
    for offset in [24, 40] { abi::emit_store_to_address(e, a, object, offset); }
    crate::codegen_support::sentinels::emit_throwable_creation_line_unknown(e, object);
    abi::emit_store_reg_to_symbol(e, object, "_exc_value", 0);
    abi::emit_jump(e, "__rt_throw_current");
}

/// Selects one target-specific bookkeeping instruction.
fn ins(e: &mut Emitter, a: &str, x: &str) {
    e.instruction(if arm(e) { a } else { x });                                  // perform native property name and ownership bookkeeping
}
/// Returns whether this target uses AArch64 registers.
fn arm(e: &Emitter) -> bool { e.target.arch == Arch::AArch64 }
/// Returns a volatile word register independent from the result.
fn scratch(e: &Emitter) -> &'static str { if arm(e) { "x10" } else { "r10" } }
/// Returns a second volatile word register independent from the result.
fn scratch2(e: &Emitter) -> &'static str { if arm(e) { "x11" } else { "r11" } }
/// Loads an enclosing saved input without granting ownership.
fn load(e: &mut Emitter, reg: &str, offset: usize) { abi::emit_load_temporary_stack_slot(e, reg, offset); }
/// Loads one standard C ABI argument from the enclosing frame.
fn argument(e: &mut Emitter, i: usize, offset: usize) { load(e, abi::int_arg_reg_name(e.target, i), offset); }
/// Compares two words and branches on the selected equality relation.
fn compare(e: &mut Emitter, a: &str, b: &str, label: &str, eq: bool) {
    ins(e, &format!("cmp {a}, {b}"), &format!("cmp {a}, {b}"));
    ins(e, &format!("b.{} {label}", if eq { "eq" } else { "ne" }), &format!("j{} {label}", if eq { "e" } else { "ne" }));
}
/// Branches when a nullable pointer or predicate is absent.
fn branch_zero(e: &mut Emitter, reg: &str, label: &str) {
    if !arm(e) { ins(e, "", &format!("test {reg}, {reg}")); }
    ins(e, &format!("cbz {reg}, {label}"), &format!("jz {label}"));
}
/// Branches when a string equality predicate succeeds.
fn branch_nonzero(e: &mut Emitter, reg: &str, label: &str) {
    if !arm(e) { ins(e, "", &format!("test {reg}, {reg}")); }
    ins(e, &format!("cbnz {reg}, {label}"), &format!("jnz {label}"));
}
/// Returns a unique body label for one runtime class and physical property.
fn body_label(slot: &EvalPropertySlot) -> String { format!("__rt_native_restore_slot_{}_{}", slot.class_id, slot.offset) }
