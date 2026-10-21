//! Purpose:
//! Emits the ownership finisher used after a parsed value is stored into a typed property slot.
//!
//! Called from:
//! - `__rt_obj_store_prop` in the target-specific object-storage helpers.
//!
//! Key details:
//! - Scalar and string slots copy the box payload; the box itself must be freed, and a string
//!   pointer transfers its persisted owner to the property slot.
//! - Array, hash, and object slots receive a payload pointer. The property acquires its own
//!   payload retain while the parsed box stays alive until the decoder context completes, so a
//!   later `R:`/`r:` slot keeps addressing live storage.
//! - The helper takes a borrowed box and consumes it in every path.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits the target-specific post-store ownership finisher.
pub(super) fn emit(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// Emits the AArch64 post-store ownership finisher.
///
/// Input: x0 = parsed value box. The property slot already holds its copy or payload pointer.
fn emit_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unserialize stored-property ownership ---");
    emitter.label_global("__rt_unser_finish_stored_prop");
    emitter.instruction("cbz x0, __rt_unser_finish_stored_prop_done");          // a missing box needs no cleanup
    emitter.instruction("ldr x9, [x0]");                                        // boxed value tag
    emitter.instruction("cmp x9, #1");                                          // is the stored payload an owned string?
    emitter.instruction("b.eq __rt_unser_finish_stored_prop_string");          // the slot now owns the persisted string
    emitter.instruction("cmp x9, #4");                                          // is the value below the refcounted container range?
    emitter.instruction("b.lo __rt_unser_finish_stored_prop_scalar");          // scalars copy inline and own no heap payload
    emitter.instruction("cmp x9, #6");                                          // does the value stay inside the container range?
    emitter.instruction("b.hi __rt_unser_finish_stored_prop_scalar");          // other wire values transfer no container owner
    emitter.instruction("ldr x9, [x0, #8]");                                    // container payload stored in the property slot
    emitter.instruction("stp x0, x30, [sp, #-16]!");                            // preserve the parsed box across the payload retain
    emitter.instruction("mov x0, x9");                                          // pass the property's payload to the retain helper
    emitter.instruction("bl __rt_incref");                                      // give the property slot its own container owner
    emitter.instruction("ldp x0, x30, [sp], #16");                              // restore the parsed box after the retain
    abi::emit_jump(emitter, "__rt_unserialize_defer_data");                     // keep the registry box alive until the context completes
    emitter.label("__rt_unser_finish_stored_prop_string");
    abi::emit_jump(emitter, "__rt_heap_free");                                  // free the shell; the property slot owns the string payload
    emitter.label("__rt_unser_finish_stored_prop_scalar");
    abi::emit_jump(emitter, "__rt_decref_mixed");                               // release the scalar box through the counted path
    emitter.label("__rt_unser_finish_stored_prop_done");
    abi::emit_return(emitter);
}

/// Emits the x86_64 post-store ownership finisher.
///
/// Input: rax = parsed value box. The property slot already holds its copy or payload pointer.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: unserialize stored-property ownership ---");
    emitter.label_global("__rt_unser_finish_stored_prop");
    emitter.instruction("test rax, rax");                                       // a missing box needs no cleanup
    emitter.instruction("jz __rt_unser_finish_stored_prop_done_x");             // nothing to release
    emitter.instruction("mov r10, QWORD PTR [rax]");                           // boxed value tag
    emitter.instruction("cmp r10, 1");                                         // is the stored payload an owned string?
    emitter.instruction("je __rt_unser_finish_stored_prop_string_x");          // the slot now owns the persisted string
    emitter.instruction("cmp r10, 4");                                         // is the value below the refcounted container range?
    emitter.instruction("jb __rt_unser_finish_stored_prop_scalar_x");          // scalars copy inline and own no heap payload
    emitter.instruction("cmp r10, 6");                                         // does the value stay inside the container range?
    emitter.instruction("ja __rt_unser_finish_stored_prop_scalar_x");          // other wire values transfer no container owner
    emitter.instruction("mov rdi, QWORD PTR [rax + 8]");                       // container payload stored in the property slot
    emitter.instruction("push rax");                                           // preserve the parsed box across the payload retain
    emitter.instruction("call __rt_incref");                                   // give the property slot its own container owner
    emitter.instruction("pop rdi");                                            // restore the parsed box for the context deferral
    abi::emit_jump(emitter, "__rt_unserialize_defer_data");                     // keep the registry box alive until the context completes
    emitter.label("__rt_unser_finish_stored_prop_string_x");
    abi::emit_jump(emitter, "__rt_heap_free");                                  // free the shell; the property slot owns the string payload
    emitter.label("__rt_unser_finish_stored_prop_scalar_x");
    abi::emit_jump(emitter, "__rt_decref_mixed");                               // release the scalar box through the counted path
    emitter.label("__rt_unser_finish_stored_prop_done_x");
    abi::emit_return(emitter);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::Target;

    /// Every target keeps container payloads alive through the context and frees copied
    /// scalar/string box shells after the property slot has taken what it needs.
    #[test]
    fn stored_property_finisher_contract_holds_on_all_targets() {
        for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
            let mut emitter = Emitter::new(Target::parse(name).unwrap());
            emit(&mut emitter);
            let asm = emitter.output();
            assert!(asm.contains("__rt_unser_finish_stored_prop"), "{name}");
            assert!(asm.contains("__rt_incref"), "{name}");
            assert!(asm.contains("__rt_unserialize_defer_data"), "{name}");
            assert!(asm.contains("__rt_heap_free"), "{name}");
        }
    }
}
