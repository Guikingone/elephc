//! Purpose:
//! Boxes raw objects and installs dynamic-object destructor hooks.
//!
//! Called from:
//! - The eval bridge runtime facade and sibling bridge emitters.
//!
//! Key details:
//! - Both supported architectures expose the same bridge callbacks.

use super::*;

/// Bucket mask of the eval owner filter: `(identity >> 4) & EVAL_OWNER_FILTER_MASK`. Must match
/// `OWNER_FILTER_BUCKETS - 1` in elephc-magician's `ffi::dynamic_destructors`, which keeps the
/// counts this reads.
pub(crate) const EVAL_OWNER_FILTER_MASK: u64 = 4095;

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

/// Emits the ARM64 wrapper that installs the dynamic object destructor callback and the
/// eval owner filter (x1) that `__rt_eval_object_has_owner` and the destructor path read.
pub(super) fn emit_aarch64_install_dynamic_object_destructor_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_dynamic_object_destructor_hook");
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_dynamic_object_destruct_fn");
    emitter.instruction("str x0, [x9]");                                        // store the Rust callback pointer for object destruction
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_owner_filter");
    emitter.instruction("str x1, [x9]");                                        // publish the per-bucket count of eval-owned identities
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the ARM64 `__rt_eval_object_has_owner`: x0 = object identity, returns x0 = 1 when an
/// eval context owns it. A zero filter bucket answers without leaving generated code; only a
/// bucket that holds a registered identity asks the registry.
pub(super) fn emit_aarch64_eval_object_has_owner(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: eval_object_has_owner ---");
    emitter.label_global("__rt_eval_object_has_owner");
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_owner_filter");
    emitter.instruction("ldr x9, [x9]");                                        // x9 = owner filter, published when the first eval context installed its hooks
    emitter.instruction("cbz x9, __rt_eval_object_has_owner_none");             // no context yet means no eval-owned object either
    emitter.instruction("lsr x10, x0, #4");                                     // drop the identity's alignment bits, as the registry does
    emitter.instruction(&format!("and x10, x10, #{}", EVAL_OWNER_FILTER_MASK)); // x10 = filter bucket of this identity
    emitter.instruction("ldr w10, [x9, x10, lsl #2]");                          // w10 = registered identities in that bucket
    emitter.instruction("cbz w10, __rt_eval_object_has_owner_none");            // an empty bucket proves the identity is unregistered
    let owner_probe = emitter.target.extern_symbol("__elephc_eval_object_has_dynamic_owner");
    emitter.instruction(&format!("b {}", owner_probe));                         // a shared bucket still needs the exact registry answer
    emitter.label("__rt_eval_object_has_owner_none");
    emitter.instruction("mov x0, #0");                                          // not eval-owned
    emitter.instruction("ret");                                                 // return to the native dispatch path
}

/// Emits the ARM64 wrapper that installs the eval `Generator` protocol callback.
pub(super) fn emit_aarch64_install_generator_protocol_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_generator_protocol_hook");
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_generator_protocol_fn");
    emitter.instruction("str x0, [x9]");                                        // store the Rust callback pointer for interpreted generators
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the ARM64 C-ABI wrappers that let the INTERPRETER drive a compiled generator.
///
/// `__elephc_eval_install_generator_protocol_hook` is the other direction: it lets compiled code
/// drive an interpreted generator. Both are needed the moment a compiled class and an
/// eval-declared subclass each own one end of a `yield from` -- `twig/twig`'s `Template::yield()`
/// is compiled while the template's `doDisplay()` is interpreted, and each delegates to the other.
///
/// The `__rt_gen_*` helpers are emitted with a plain assembler label rather than a C symbol, so
/// they cannot be named from Rust portably (Mach-O would ask for one underscore too many). These
/// wrappers give them a C name; each is a tail call, so the ABI is the helper's own.
pub(super) fn emit_aarch64_eval_generator_drive(emitter: &mut Emitter) {
    for (wrapper, helper) in EVAL_GENERATOR_DRIVE_WRAPPERS {
        label_c_global(emitter, wrapper);
        emitter.instruction(&format!("b {}", helper));                          // tail-call the runtime helper, keeping its ABI
    }
}

/// Emits the x86_64 C-ABI wrappers that let the INTERPRETER drive a compiled generator.
pub(super) fn emit_x86_64_eval_generator_drive(emitter: &mut Emitter) {
    for (wrapper, helper) in EVAL_GENERATOR_DRIVE_WRAPPERS {
        label_c_global(emitter, wrapper);
        emitter.instruction(&format!("jmp {}", helper));                        // tail-call the runtime helper, keeping its ABI
    }
}

/// The C name each generator helper is exposed under, paired with the helper itself.
const EVAL_GENERATOR_DRIVE_WRAPPERS: &[(&str, &str)] = &[
    ("__elephc_eval_gen_valid", "__rt_gen_valid"),
    ("__elephc_eval_gen_current", "__rt_gen_current"),
    ("__elephc_eval_gen_key", "__rt_gen_key"),
    ("__elephc_eval_gen_next", "__rt_gen_next"),
    ("__elephc_eval_gen_get_return", "__rt_gen_get_return"),
];

/// Emits the ARM64 wrapper that installs the eval class-autoload callback.
pub(super) fn emit_aarch64_install_class_autoload_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_class_autoload_hook");
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_class_autoload_fn");
    emitter.instruction("str x0, [x9]");                                        // store the Rust callback pointer for class autoloading
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the ARM64 wrapper that installs the eval unserialize-object callback.
pub(super) fn emit_aarch64_install_unserialize_object_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_unserialize_object_hook");
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_unserialize_object_fn");
    emitter.instruction("str x0, [x9]");                                        // store the Rust callback pointer for eval-declared hydration
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the ARM64 wrapper that installs the eval object class-relation callback.
pub(super) fn emit_aarch64_install_object_relation_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_object_relation_hook");
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_object_relation_fn");
    emitter.instruction("str x0, [x9]");                                        // store the Rust callback pointer for eval class relations
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the ARM64 wrapper that installs the eval serialize-object callback.
pub(super) fn emit_aarch64_install_serialize_object_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_serialize_object_hook");
    abi::emit_symbol_address(emitter, "x9", "_elephc_eval_serialize_object_fn");
    emitter.instruction("str x0, [x9]");                                        // store the Rust callback pointer for eval object rendering
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

/// Emits the x86_64 wrapper that installs the dynamic object destructor callback and the
/// eval owner filter (rsi) that `__rt_eval_object_has_owner` and the destructor path read.
pub(super) fn emit_x86_64_install_dynamic_object_destructor_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_dynamic_object_destructor_hook");
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_dynamic_object_destruct_fn");
    emitter.instruction("mov QWORD PTR [r10], rdi");                            // store the Rust callback pointer for object destruction
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_owner_filter");
    emitter.instruction("mov QWORD PTR [r10], rsi");                            // publish the per-bucket count of eval-owned identities
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the x86_64 `__rt_eval_object_has_owner`: rdi = object identity, returns rax = 1 when
/// an eval context owns it. A zero filter bucket answers without leaving generated code; only a
/// bucket that holds a registered identity asks the registry.
pub(super) fn emit_x86_64_eval_object_has_owner(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: eval_object_has_owner ---");
    emitter.label_global("__rt_eval_object_has_owner");
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_owner_filter");
    emitter.instruction("mov r10, QWORD PTR [r10]");                            // r10 = owner filter, published when the first eval context installed its hooks
    emitter.instruction("test r10, r10");                                       // has any eval context installed its hooks?
    emitter.instruction("jz __rt_eval_object_has_owner_none");                  // no context yet means no eval-owned object either
    emitter.instruction("mov r11, rdi");                                        // copy the identity to derive its bucket
    emitter.instruction("shr r11, 4");                                          // drop the identity's alignment bits, as the registry does
    emitter.instruction(&format!("and r11, {}", EVAL_OWNER_FILTER_MASK));       // r11 = filter bucket of this identity
    emitter.instruction("mov eax, DWORD PTR [r10 + r11 * 4]");                  // eax = registered identities in that bucket
    emitter.instruction("test eax, eax");                                       // is the bucket empty?
    emitter.instruction("jz __rt_eval_object_has_owner_none");                  // an empty bucket proves the identity is unregistered
    let owner_probe = emitter.target.extern_symbol("__elephc_eval_object_has_dynamic_owner");
    emitter.instruction(&format!("jmp {}", owner_probe));                       // a shared bucket still needs the exact registry answer
    emitter.label("__rt_eval_object_has_owner_none");
    emitter.instruction("xor eax, eax");                                        // not eval-owned
    emitter.instruction("ret");                                                 // return to the native dispatch path
}

/// Emits the x86_64 wrapper that installs the eval `Generator` protocol callback.
pub(super) fn emit_x86_64_install_generator_protocol_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_generator_protocol_hook");
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_generator_protocol_fn");
    emitter.instruction("mov QWORD PTR [r10], rdi");                            // store the Rust callback pointer for interpreted generators
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the x86_64 wrapper that installs the eval class-autoload callback.
pub(super) fn emit_x86_64_install_class_autoload_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_class_autoload_hook");
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_class_autoload_fn");
    emitter.instruction("mov QWORD PTR [r10], rdi");                            // store the Rust callback pointer for class autoloading
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the x86_64 wrapper that installs the eval unserialize-object callback.
pub(super) fn emit_x86_64_install_unserialize_object_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_unserialize_object_hook");
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_unserialize_object_fn");
    emitter.instruction("mov QWORD PTR [r10], rdi");                            // store the Rust callback pointer for eval-declared hydration
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the x86_64 wrapper that installs the eval object class-relation callback.
pub(super) fn emit_x86_64_install_object_relation_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_object_relation_hook");
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_object_relation_fn");
    emitter.instruction("mov QWORD PTR [r10], rdi");                            // store the Rust callback pointer for eval class relations
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}

/// Emits the x86_64 wrapper that installs the eval serialize-object callback.
pub(super) fn emit_x86_64_install_serialize_object_hook(emitter: &mut Emitter) {
    label_c_global(emitter, "__elephc_eval_install_serialize_object_hook");
    abi::emit_symbol_address(emitter, "r10", "_elephc_eval_serialize_object_fn");
    emitter.instruction("mov QWORD PTR [r10], rdi");                            // store the Rust callback pointer for eval object rendering
    emitter.instruction("ret");                                                 // return after installing the optional eval hook
}
