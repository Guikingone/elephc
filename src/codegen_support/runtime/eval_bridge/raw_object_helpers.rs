//! Purpose:
//! Boxes raw objects and installs dynamic-object destructor hooks.
//!
//! Called from:
//! - The eval bridge runtime facade and sibling bridge emitters.
//!
//! Key details:
//! - Both supported architectures expose the same bridge callbacks.

use super::*;

/// Emits the ARM64 wrapper that boxes a borrowed raw object pointer for Rust eval.
pub(super) fn emit_aarch64_object_from_raw_wrapper(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_value_object_from_raw");
    emitter.instruction("cbz x0, __elephc_eval_value_object_from_raw_null");    // null raw object pointers become PHP null cells
    emitter.instruction("mov x1, x0");                                          // move the raw object pointer into the Mixed payload
    emitter.instruction("mov x0, #6");                                          // runtime tag 6 = object
    emitter.instruction("mov x2, xzr");                                         // object payloads do not use a high word
    emitter.instruction("b __rt_mixed_from_value");                             // box and retain the borrowed object for eval
    emitter.label("__elephc_eval_value_object_from_raw_null");
    emitter.instruction("mov x0, #8");                                          // runtime tag 8 = null
    emitter.instruction("mov x1, xzr");                                         // null has no low payload word
    emitter.instruction("mov x2, xzr");                                         // null has no high payload word
    emitter.instruction("b __rt_mixed_from_value");                             // box the null payload and return to Rust
}

/// Emits the ARM64 wrapper that installs the dynamic object destructor callback.
pub(super) fn emit_aarch64_install_dynamic_object_destructor_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_dynamic_object_destructor_hook");
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_dynamic_object_destruct_fn");
    emitter.instruction("str x0, [x9]");                                        // store the Rust callback pointer for object destruction
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the x86_64 wrapper that boxes a borrowed raw object pointer for Rust eval.
pub(super) fn emit_x86_64_object_from_raw_wrapper(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_value_object_from_raw");
    emitter.instruction("test rdi, rdi");                                       // null raw object pointers become PHP null cells
    emitter.instruction("jz __elephc_eval_value_object_from_raw_null_x86");     // branch to null boxing for missing object payloads
    emitter.instruction("mov eax, 6");                                          // runtime tag 6 = object
    emitter.instruction("xor esi, esi");                                        // object payloads do not use a high word
    emitter.instruction("jmp __rt_mixed_from_value");                           // box and retain the borrowed object for eval
    emitter.label("__elephc_eval_value_object_from_raw_null_x86");
    emitter.instruction("mov eax, 8");                                          // runtime tag 8 = null
    emitter.instruction("xor edi, edi");                                        // null has no low payload word
    emitter.instruction("xor esi, esi");                                        // null has no high payload word
    emitter.instruction("jmp __rt_mixed_from_value");                           // box the null payload and return to Rust
}

/// Emits the x86_64 wrapper that installs the dynamic object destructor callback.
pub(super) fn emit_x86_64_install_dynamic_object_destructor_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_dynamic_object_destructor_hook");
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_dynamic_object_destruct_fn");
    emitter.instruction("mov QWORD PTR [r10], rdi");                            // store the Rust callback pointer for object destruction
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}
