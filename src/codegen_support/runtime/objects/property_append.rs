//! Purpose:
//! Appends raw integer buckets to a prepared SPL backing-object property hash.
//!
//! Called from:
//! - Typed EIR lowering for the internal SPL object append operation.
//!
//! Key details:
//! - Dense class metadata selects the final dynamic property-hash slot.
//! - The helper consumes one Mixed owner and publishes hash growth/COW back to the object.
//! - Missing storage is diagnosed rather than silently discarding the value.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits the object/owned-Mixed C ABI append entry on every supported target.
pub(crate) fn emit_object_property_append(e: &mut Emitter) {
    let arm = e.target.arch == Arch::AArch64;
    let stack = if arm { "sp" } else { "rsp" };
    let result = abi::int_result_reg(e);
    let arg0 = abi::int_arg_reg_name(e.target, 0);
    let arg1 = abi::int_arg_reg_name(e.target, 1);
    let arg2 = abi::int_arg_reg_name(e.target, 2);
    let arg3 = abi::int_arg_reg_name(e.target, 3);
    let class = if arm { "x12" } else { "r10" };
    let temp = if arm { "x11" } else { "r11" };
    e.blank();
    e.label_global("__rt_spl_object_property_append");
    abi::emit_frame_prologue(e, 64);
    abi::emit_store_to_address(e, arg0, stack, 0);
    abi::emit_store_to_address(e, arg1, stack, 8);
    abi::emit_load_from_address(e, class, arg0, 0);
    if arm {
        e.instruction("cmn x12, #2");                                           // incomplete objects keep their opaque property hash at offset 24
        e.instruction("b.eq __rt_spl_object_append_incomplete");                // handle the special incomplete-object payload separately
    } else {
        e.instruction("cmp r10, -2");                                           // incomplete objects keep their opaque property hash at offset 24
        e.instruction("je __rt_spl_object_append_incomplete");                  // handle the special incomplete-object payload separately
    }
    abi::emit_load_symbol_to_reg(e, temp, "_class_gc_desc_count", 0);
    if arm {
        e.instruction("cmp x12, x11");                                          // validate the runtime class before dense metadata lookup
        e.instruction("b.hs __rt_spl_object_append_unsupported");               // reject unknown object layouts before taking table ownership
    } else {
        e.instruction("cmp r10, r11");                                          // validate the runtime class before dense metadata lookup
        e.instruction("jae __rt_spl_object_append_unsupported");                // reject unknown object layouts before taking table ownership
    }
    abi::emit_symbol_address(e, temp, "_class_object_dynamic_prop_flags");
    if arm {
        e.instruction("ldr x11, [x11, x12, lsl #3]");                           // inspect the prepared dynamic property-tail capability
        e.instruction("cbz x11, __rt_spl_object_append_unsupported");           // unavailable storage must not silently lose the write
    } else {
        e.instruction("mov r11, QWORD PTR [r11 + r10 * 8]");                    // inspect the prepared dynamic property-tail capability
        e.instruction("test r11, r11");                                         // check whether this class reserves the owning hash slot
        e.instruction("jz __rt_spl_object_append_unsupported");                 // unavailable storage must not silently lose the write
    }
    abi::emit_symbol_address(e, temp, "_class_object_payload_sizes");
    if arm {
        e.instruction("ldr x11, [x11, x12, lsl #3]");                           // load the class payload size including its dynamic tail
        e.instruction("sub x11, x11, #8");                                      // the final payload word owns the property hash
        e.instruction("add x0, x0, x11");                                       // locate the owning property-hash slot
    } else {
        e.instruction("mov r11, QWORD PTR [r11 + r10 * 8]");                    // load the class payload size including its dynamic tail
        e.instruction("sub r11, 8");                                            // the final payload word owns the property hash
        e.instruction("add rdi, r11");                                          // locate the owning property-hash slot
    }
    abi::emit_jump(e, "__rt_spl_object_append_slot_ready");
    e.label("__rt_spl_object_append_incomplete");
    if arm { e.instruction("add x0, x0, #24"); }                                // locate the incomplete object's opaque property-hash slot
    else { e.instruction("add rdi, 24"); }                                      // locate the incomplete object's opaque property-hash slot
    e.label("__rt_spl_object_append_slot_ready");
    abi::emit_store_to_address(e, arg0, stack, 16);
    abi::emit_load_from_address(e, result, arg0, 0);
    if arm { e.instruction("cbnz x0, __rt_spl_object_append_have_hash"); }      // preserve an existing property table and its next integer index
    else {
        e.instruction("test rax, rax");                                         // inspect the lazy property-hash pointer
        e.instruction("jnz __rt_spl_object_append_have_hash");                  // preserve an existing property table and its next integer index
    }
    abi::emit_load_int_immediate(e, arg0, 8);
    abi::emit_load_int_immediate(e, arg1, 7);
    abi::emit_call_label(e, "__rt_hash_new");
    abi::emit_load_from_address(e, temp, stack, 16);
    abi::emit_store_to_address(e, result, temp, 0);
    e.label("__rt_spl_object_append_have_hash");
    abi::emit_reg_move(e, arg0, result);
    abi::emit_load_from_address(e, arg1, stack, 8);
    abi::emit_load_int_immediate(e, arg2, 0);
    abi::emit_load_int_immediate(e, arg3, 7);
    abi::emit_call_label(e, "__rt_hash_append");
    abi::emit_load_from_address(e, temp, stack, 16);
    abi::emit_store_to_address(e, result, temp, 0);
    abi::emit_load_int_immediate(e, result, 1);
    abi::emit_frame_restore(e, 64);
    abi::emit_return(e);
    e.label("__rt_spl_object_append_unsupported");
    abi::emit_load_from_address(e, result, stack, 8);
    abi::emit_call_label(e, "__rt_decref_mixed");
    abi::emit_load_from_address(e, arg0, stack, 0);
    abi::emit_call_label(e, "__rt_throw_object_not_array");
}
