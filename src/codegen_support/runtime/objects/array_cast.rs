//! Purpose:
//! Projects objects through PHP's array-cast view while keeping ordinary own-property views separate.
//!
//! Called from:
//! - The object and boxed-Mixed array-cast lowerers through the managed runtime emitter.
//!
//! Key details:
//! - Accepts the legacy object-projection input in x0/rax and returns an owned hash.
//! - Metadata binds a nonvirtual SPL copy provider and its authoritative scalar flags offset.
//! - Callback arrays and hashes are copied into normalized, independently owned hash storage.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits the shared-target array-cast handler and its generic-property fallback.
pub(crate) fn emit_object_array_cast(e: &mut Emitter) {
    let arm = e.target.arch == Arch::AArch64;
    let stack = if arm { "sp" } else { "rsp" };
    let result = abi::int_result_reg(e);
    let arg0 = abi::int_arg_reg_name(e.target, 0);
    let arg1 = abi::int_arg_reg_name(e.target, 1);
    let class = if arm { "x12" } else { "r10" };
    let table = if arm { "x10" } else { "r11" };
    let scratch = if arm { "x11" } else { "rcx" };
    e.blank();
    e.label_global("__rt_object_array_cast");
    abi::emit_frame_prologue(e, 64);
    abi::emit_store_to_address(e, result, stack, 0);
    if arm {
        e.instruction("cbz x0, __rt_object_array_cast_properties");             // preserve the generic empty-object projection for null pointers
    } else {
        e.instruction("test rax, rax");                                         // inspect the object pointer before reading its class id
        e.instruction("jz __rt_object_array_cast_properties");                  // preserve the generic empty-object projection for null pointers
    }
    abi::emit_load_from_address(e, class, result, 0);
    abi::emit_load_symbol_to_reg(e, scratch, "_class_gc_desc_count", 0);
    if arm {
        e.instruction("cmp x12, x11");                                          // validate the class id before reading the dense handler table
        e.instruction("b.hs __rt_object_array_cast_properties");                // generic projection handles incomplete and unknown class ids
    } else {
        e.instruction("cmp r10, rcx");                                          // validate the class id before reading the dense handler table
        e.instruction("jae __rt_object_array_cast_properties");                 // generic projection handles incomplete and unknown class ids
    }
    abi::emit_symbol_address(e, table, "_class_spl_array_cast_ptrs");
    if arm {
        e.instruction("ldr x10, [x10, x12, lsl #3]");                           // select the optional SPL handler descriptor
        e.instruction("cbz x10, __rt_object_array_cast_properties");            // ordinary classes expose their complete own property table
    } else {
        e.instruction("mov r11, QWORD PTR [r11 + r10 * 8]");                    // select the optional SPL handler descriptor
        e.instruction("test r11, r11");                                         // inspect whether this class has a custom array-cast provider
        e.instruction("jz __rt_object_array_cast_properties");                  // ordinary classes expose their complete own property table
    }
    abi::emit_load_from_address(e, scratch, table, 0);
    if arm {
        e.instruction("cmp x11, #1");                                           // require the known nonvirtual copy-provider descriptor kind
        e.instruction("b.ne __rt_object_array_cast_bad_hook");                  // unsupported handler layouts must be diagnosed
    } else {
        e.instruction("cmp rcx, 1");                                            // require the known nonvirtual copy-provider descriptor kind
        e.instruction("jne __rt_object_array_cast_bad_hook");                   // unsupported handler layouts must be diagnosed
    }
    abi::emit_load_from_address(e, scratch, table, 8);
    if arm {
        e.instruction("ldr x11, [x0, x11]");                                    // read the builtin provider's scalar flags slot
        e.instruction("tbnz x11, #0, __rt_object_array_cast_properties");       // STD_PROP_LIST selects the wrapper's own PHP declarations
    } else {
        e.instruction("test QWORD PTR [rax + rcx], 1");                         // inspect STD_PROP_LIST in the builtin provider's flags slot
        e.instruction("jnz __rt_object_array_cast_properties");                 // STD_PROP_LIST selects the wrapper's own PHP declarations
    }
    abi::emit_load_from_address(e, table, table, 16);
    abi::emit_load_from_address(e, arg0, stack, 0);
    abi::emit_call_reg(e, table);
    abi::emit_store_to_address(e, result, stack, 8);
    abi::emit_call_label(e, "__rt_heap_kind");
    if arm {
        e.instruction("cmp x0, #3");                                            // distinguish an associative copy from an indexed PHP array
        e.instruction("b.eq __rt_object_array_cast_hash");                      // normalize string property keys in hash copies
        e.instruction("cmp x0, #2");                                            // the remaining supported provider result is an indexed array
        e.instruction("b.ne __rt_object_array_cast_bad_result");                // reject invalid provider return representations
    } else {
        e.instruction("cmp rax, 3");                                            // distinguish an associative copy from an indexed PHP array
        e.instruction("je __rt_object_array_cast_hash");                        // normalize string property keys in hash copies
        e.instruction("cmp rax, 2");                                            // the remaining supported provider result is an indexed array
        e.instruction("jne __rt_object_array_cast_bad_result");                 // reject invalid provider return representations
    }
    abi::emit_load_from_address(e, arg0, stack, 8);
    abi::emit_call_label(e, "__rt_array_to_hash");
    abi::emit_store_to_address(e, result, stack, 16);
    abi::emit_load_from_address(e, result, stack, 8);
    abi::emit_call_label(e, "__rt_decref_array");
    abi::emit_jump(e, "__rt_object_array_cast_done");

    e.label("__rt_object_array_cast_hash");
    abi::emit_load_int_immediate(e, arg0, 8);
    abi::emit_load_int_immediate(e, arg1, 7);
    abi::emit_call_label(e, "__rt_hash_new");
    abi::emit_reg_move(e, arg0, result);
    abi::emit_load_from_address(e, arg1, stack, 8);
    abi::emit_call_label(e, "__rt_hash_project_spread");
    abi::emit_store_to_address(e, result, stack, 16);
    abi::emit_load_from_address(e, result, stack, 8);
    abi::emit_call_label(e, "__rt_decref_hash");
    e.label("__rt_object_array_cast_done");
    abi::emit_load_from_address(e, result, stack, 16);
    abi::emit_frame_restore(e, 64);
    abi::emit_return(e);

    e.label("__rt_object_array_cast_properties");
    abi::emit_load_from_address(e, result, stack, 0);
    abi::emit_load_int_immediate(e, if arm { "x1" } else { "rdx" }, 1);
    abi::emit_load_int_immediate(e, if arm { "x2" } else { "rcx" }, -1);
    abi::emit_call_label(e, "__rt_object_to_hash");
    abi::emit_frame_restore(e, 64);
    abi::emit_return(e);

    e.label("__rt_object_array_cast_bad_result");
    abi::emit_load_from_address(e, result, stack, 8);
    abi::emit_call_label(e, "__rt_decref_any");
    e.label("__rt_object_array_cast_bad_hook");
    abi::emit_load_from_address(e, arg0, stack, 0);
    abi::emit_call_label(e, "__rt_throw_object_not_array");
}
