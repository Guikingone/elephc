//! Purpose:
//! Extends the borrowed object-store index when shutdown forbids handle reuse.
//!
//! Called from:
//! - `super::handles::emit_object_handles()` emits the rare acquisition slow path.
//!
//! Key details:
//! - Storage lives outside the managed PHP heap and owns no indexed payload.
//! - Every codegen-visible volatile integer and SIMD register survives libc allocation.
//! - A full arena reset frees grown metadata and restores the initial fixed capacity.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

const FRAME: usize = 720;
const OLD_PTR: usize = 160;
const OLD_CAPACITY: usize = 168;
const CAPACITY: usize = 176;
const NEW_PTR: usize = 184;

/// Emits the all-register-preserving inverse-table growth boundary.
pub(super) fn emit_object_store_growth(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    let word = abi::secondary_scratch_reg(emitter);
    let a0 = abi::int_arg_reg_name(emitter.target, 0);
    let a1 = abi::int_arg_reg_name(emitter.target, 1);
    let a2 = abi::int_arg_reg_name(emitter.target, 2);
    emitter.blank();
    emitter.label_global("__rt_object_store_grow");
    save_registers(emitter);
    abi::emit_load_symbol_to_reg(emitter, result, "_obj_handle_next", 0);
    abi::emit_load_int_immediate(emitter, word, 0x1_0000_0000);
    ins(emitter, "cmp x0, x10", "cmp rax, r10");
    ins(emitter, "b.ls __rt_object_store_grow_valid", "jbe __rt_object_store_grow_valid");
    abi::emit_jump(emitter, "__rt_heap_allocation_failed");
    emitter.label("__rt_object_store_grow_valid");
    abi::emit_load_symbol_to_reg(emitter, result, "_obj_handle_capacity", 0);
    abi::store_at_offset(emitter, result, OLD_CAPACITY);
    abi::store_at_offset(emitter, result, CAPACITY);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_heap_allocation_failed");
    emitter.label("__rt_object_store_grow_capacity");
    abi::load_at_offset(emitter, result, CAPACITY);
    abi::emit_cmp_reg_to_symbol(emitter, result, "_obj_handle_next");
    ins(emitter, "b.hs __rt_object_store_grow_allocate", "jae __rt_object_store_grow_allocate");
    ins(emitter, "lsl x0, x0, #1", "shl rax, 1");
    abi::emit_load_int_immediate(emitter, word, 0x1_0000_0000);
    ins(emitter, "cmp x0, x10", "cmp rax, r10");
    ins(emitter, "b.ls __rt_object_store_grow_store_capacity", "jbe __rt_object_store_grow_store_capacity");
    abi::emit_reg_move(emitter, result, word);
    emitter.label("__rt_object_store_grow_store_capacity");
    abi::store_at_offset(emitter, result, CAPACITY);
    abi::emit_jump(emitter, "__rt_object_store_grow_capacity");
    emitter.label("__rt_object_store_grow_allocate");
    abi::emit_load_symbol_to_reg(emitter, result, "_obj_handle_live_ptr", 0);
    abi::store_at_offset(emitter, result, OLD_PTR);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_object_store_grow_initial");
    abi::emit_reg_move(emitter, a0, result);
    abi::load_at_offset(emitter, a1, CAPACITY);
    ins(emitter, "lsl x1, x1, #3", "shl rsi, 3");
    emitter.bl_c("realloc");                                                    // retain every previously acquired inverse entry while extending capacity
    abi::emit_jump(emitter, "__rt_object_store_grow_allocated");
    emitter.label("__rt_object_store_grow_initial");
    abi::load_at_offset(emitter, a0, CAPACITY);
    ins(emitter, "lsl x0, x0, #3", "shl rdi, 3");
    emitter.bl_c("malloc");                                                     // fixed BSS storage itself must never be passed to realloc
    emitter.label("__rt_object_store_grow_allocated");
    abi::emit_branch_if_int_result_zero(emitter, "__rt_heap_allocation_failed");
    abi::store_at_offset(emitter, result, NEW_PTR);
    abi::load_at_offset(emitter, result, OLD_PTR);
    abi::emit_branch_if_int_result_nonzero(emitter, "__rt_object_store_grow_zero_tail");
    abi::load_at_offset(emitter, a0, NEW_PTR);
    abi::emit_symbol_address(emitter, a1, "_obj_handle_live");
    abi::load_at_offset(emitter, a2, OLD_CAPACITY);
    ins(emitter, "lsl x2, x2, #3", "shl rdx, 3");
    emitter.bl_c("memcpy");                                                     // transfer the borrowed identities from initial storage once
    emitter.label("__rt_object_store_grow_zero_tail");
    abi::load_at_offset(emitter, a0, NEW_PTR);
    abi::load_at_offset(emitter, word, OLD_CAPACITY);
    ins(emitter, "add x0, x0, x10, lsl #3", "lea rdi, [rdi + r10 * 8]");
    abi::emit_load_int_immediate(emitter, a1, 0);
    abi::load_at_offset(emitter, a2, CAPACITY);
    ins(emitter, "sub x2, x2, x10", "sub rdx, r10");
    ins(emitter, "lsl x2, x2, #3", "shl rdx, 3");
    emitter.bl_c("memset");                                                     // keep never-acquired entries null in every grown allocation
    abi::load_at_offset(emitter, result, NEW_PTR);
    abi::emit_store_reg_to_symbol(emitter, result, "_obj_handle_live_ptr", 0);
    abi::load_at_offset(emitter, result, CAPACITY);
    abi::emit_store_reg_to_symbol(emitter, result, "_obj_handle_capacity", 0);
    restore_registers(emitter);
}

/// Lists the volatile integer registers whose values acquisition promises to preserve.
fn integer_registers(arch: Arch) -> Vec<String> {
    match arch {
        Arch::AArch64 => (0..19).map(|index| format!("x{index}")).collect(),
        Arch::X86_64 => ["rax", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11"]
            .iter().map(|register| register.to_string()).collect(),
    }
}

/// Parks all visible register values before any C allocator can clobber them.
fn save_registers(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::AArch64 {
        emitter.instruction("sub sp, sp, #720");                                // reserve volatile registers, vector state, locals and linkage
        emitter.instruction("str x29, [sp, #704]");                             // use unsigned stores beyond the pair-instruction displacement range
        emitter.instruction("str x30, [sp, #712]");                             // preserve the acquire helper's return address independently
        emitter.instruction("add x29, sp, #704");                               // keep canonical frame-relative local addressing
    } else {
        abi::emit_frame_prologue(emitter, FRAME);
    }
    for (index, register) in integer_registers(emitter.target.arch).iter().enumerate() {
        abi::store_at_offset(emitter, register, 8 + index * 8);
    }
    vector_registers(emitter, false);
}

/// Restores vectors before integer scratch registers, then returns without clobbering x9.
fn restore_registers(emitter: &mut Emitter) {
    vector_registers(emitter, true);
    for (index, register) in integer_registers(emitter.target.arch).iter().enumerate() {
        abi::load_at_offset(emitter, register, 8 + index * 8);
    }
    if emitter.target.arch == Arch::AArch64 {
        emitter.instruction("ldr x30, [sp, #712]");                             // restore linkage without using a previously restored volatile register
        emitter.instruction("ldr x29, [sp, #704]");                             // return the caller's frame pointer intact
        emitter.instruction("add sp, sp, #720");                                // release the complete slow-path frame
    } else {
        emitter.instruction("leave");                                           // restore the actual ABI frame extent without clobbering live registers
    }
    abi::emit_return(emitter);
}

/// Saves or restores full vector lanes used by either supported instruction set.
fn vector_registers(emitter: &mut Emitter, restore: bool) {
    let count = if emitter.target.arch == Arch::AArch64 { 32 } else { 16 };
    let address = abi::symbol_scratch_reg(emitter);
    for index in 0..count {
        abi::emit_frame_slot_address(emitter, address, 208 + index * 16);
        let instruction = match (emitter.target.arch, restore) {
            (Arch::AArch64, false) => format!("str q{index}, [{address}]"),
            (Arch::AArch64, true) => format!("ldr q{index}, [{address}]"),
            (Arch::X86_64, false) => format!("movdqu XMMWORD PTR [{address}], xmm{index}"),
            (Arch::X86_64, true) => format!("movdqu xmm{index}, XMMWORD PTR [{address}]"),
        };
        emitter.instruction(&instruction);                                      // libc allocation must not alter a live constructor floating-point argument
    }
}

/// Selects equivalent target instructions without changing ownership or capacity order.
fn ins(emitter: &mut Emitter, arm: &str, x86: &str) {
    let instruction = if emitter.target.arch == Arch::AArch64 { arm } else { x86 };
    emitter.instruction(instruction);                                           // perform the target's form of this object-store capacity step
}
