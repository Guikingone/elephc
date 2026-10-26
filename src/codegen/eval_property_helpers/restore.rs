//! Purpose:
//! Emits the native serialized-property writer using shared typed slot replacement.
//!
//! Called from:
//! - `super::emit_restore_property_helper()` during user assembly generation.
//!
//! Key details:
//! - Mangled PHP keys distinguish inherited private physical slots.
//! - Inputs remain borrowed; ordinary typed writers retain before replacing storage.
//! - Reserved native slots cannot be restored through PHP property data.

use super::*;
use crate::ir::{Immediate, RuntimeCallTarget, RuntimeFnId};

/// Emits only when lowered hydration code references this internal builtin.
pub(super) fn emit(module: &Module, emitter: &mut Emitter, data: &mut DataSection) {
    let used = module.functions.iter()
        .chain(module.class_methods.iter()).chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter()).chain(module.callback_wrappers.iter())
        .chain(module.extern_callback_trampolines.iter())
        .chain(module.runtime_callable_invokers.iter())
        .any(|function| function.instructions.iter().any(|inst| matches!(inst.immediate,
            Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(RuntimeFnId::ElephcObjectRestoreProperty))))));
    if !used { return; }
    let slots = collect_eval_property_slots(module).into_iter()
        .filter(|slot| !slot.property.starts_with('\0'))
        .map(|mut slot| {
            slot.property = match slot.visibility {
                Visibility::Private => format!("\0{}\0{}", slot.declaring_class, slot.property),
                Visibility::Protected => format!("\0*\0{}", slot.property),
                _ => slot.property,
            };
            // Mangling already identifies the exact slot; there is no user scope lookup.
            slot.visibility = Visibility::Public;
            slot
        }).collect::<Vec<_>>();
    if !module.required_runtime_features.eval_bridge && !super::hydrate::is_used(module) {
        emit_property_hash_slot_helper(module, emitter);
    }
    emitter.label_global("__rt_object_restore_property");
    let fail = "__rt_object_restore_property_fail";
    let done = "__rt_object_restore_property_done";
    match module.target.arch {
        Arch::AArch64 => {
            emitter.instruction("sub sp, sp, #96");                             // reserve the shared typed-writer frame layout
            emitter.instruction("stp x29, x30, [sp, #80]");                     // preserve the caller frame and link register
            emitter.instruction("add x29, sp, #80");                            // establish a stable frame pointer
            emitter.instruction("str x1, [sp]");                                // preserve the borrowed mangled key pointer
            emitter.instruction("str x2, [sp, #8]");                            // preserve the key byte length
            emitter.instruction("str x0, [sp, #16]");                           // preserve the raw receiver for slot replacement
            emitter.instruction("str x3, [sp, #24]");                           // preserve the borrowed boxed replacement
            emitter.instruction(&format!("cbz x0, {fail}"));                    // reject a missing receiver
            for (class_id, class_slots) in grouped_slots(&slots) {
                let next = format!("__rt_restore_property_next_{class_id}");
                emitter.instruction("ldr x9, [sp, #16]");                       // reload after string comparison scratch use
                emitter.instruction("ldr x9, [x9]");                            // recover the runtime receiver class id
                abi::emit_load_int_immediate(emitter, "x10", class_id as i64);
                emitter.instruction("cmp x9, x10");                             // select one receiver class layout
                emitter.instruction(&format!("b.ne {next}"));                   // try the next class if ids differ
                for slot in class_slots {
                    emit_aarch64_property_name_compare(module, emitter, data, slot, "restore", fail);
                }
                emitter.label(&next);
            }
            emitter.instruction("ldr x9, [sp, #8]");                            // inspect the key length before reading a byte
            emitter.instruction(&format!("cbz x9, {fail}"));                    // empty property names are not restored
            emitter.instruction("ldr x9, [sp]");                                // recover the unmatched key pointer
            emitter.instruction("ldrb w9, [x9]");                               // a leading NUL means a missing declared slot
            emitter.instruction(&format!("cbz x9, {fail}"));                    // never publish reserved or unknown mangled keys dynamically
            emit_dynamic_property_set_fallback(emitter, fail, done);
            for slot in &slots {
                emitter.label(&slot_body_label(module, slot, "restore"));
                emit_restore_slot(module, emitter, data, slot, fail);
                emitter.instruction("mov x0, #1");                              // report a successful declared-property replacement
                emitter.instruction(&format!("b {done}"));                      // join the common writer epilogue
            }
            emitter.label(fail);
            emitter.instruction("mov x0, #0");                                  // report an unsupported property restoration
            emitter.label(done);
            emitter.instruction("ldp x29, x30, [sp, #80]");                     // restore the caller frame and link register
            emitter.instruction("add sp, sp, #96");                             // discard borrowed inputs and replacement temporaries
            emitter.instruction("ret");                                         // return the boolean restoration status
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");                                    // preserve the caller frame pointer
            emitter.instruction("mov rbp, rsp");                                // establish the shared typed-writer frame base
            emitter.instruction("sub rsp, 64");                                 // reserve aligned input and replacement temporaries
            emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                // preserve the borrowed mangled key pointer
            emitter.instruction("mov QWORD PTR [rbp - 16], rdx");               // preserve the key byte length
            emitter.instruction("mov QWORD PTR [rbp - 24], rdi");               // preserve the raw receiver
            emitter.instruction("mov QWORD PTR [rbp - 32], rcx");               // preserve the borrowed boxed replacement
            emitter.instruction("test rdi, rdi");                               // inspect the receiver before reading its header
            emitter.instruction(&format!("jz {fail}"));                         // reject a missing receiver
            for (class_id, class_slots) in grouped_slots(&slots) {
                let next = format!("__rt_restore_property_next_{class_id}");
                emitter.instruction("mov r11, QWORD PTR [rbp - 24]");           // reload after string comparison scratch use
                emitter.instruction("mov r11, QWORD PTR [r11]");                // recover the runtime receiver class id
                abi::emit_load_int_immediate(emitter, "r10", class_id as i64);
                emitter.instruction("cmp r11, r10");                            // select one receiver class layout
                emitter.instruction(&format!("jne {next}"));                    // try the next class if ids differ
                for slot in class_slots {
                    emit_x86_64_property_name_compare(module, emitter, data, slot, "restore", fail);
                }
                emitter.label(&next);
            }
            emitter.instruction("cmp QWORD PTR [rbp - 16], 0");                 // check the unmatched property-name length
            emitter.instruction(&format!("je {fail}"));                         // empty property names are not restored
            emitter.instruction("mov r11, QWORD PTR [rbp - 8]");                // recover the unmatched key pointer
            emitter.instruction("cmp BYTE PTR [r11], 0");                       // a leading NUL means a missing declared slot
            emitter.instruction(&format!("je {fail}"));                         // never publish unknown mangled keys dynamically
            emit_dynamic_property_set_fallback(emitter, fail, done);
            for slot in &slots {
                emitter.label(&slot_body_label(module, slot, "restore"));
                emit_restore_slot(module, emitter, data, slot, fail);
                emitter.instruction("mov rax, 1");                              // report a successful declared-property replacement
                emitter.instruction(&format!("jmp {done}"));                    // join the common writer epilogue
            }
            emitter.label(fail);
            emitter.instruction("xor eax, eax");                                // report an unsupported property restoration
            emitter.label(done);
            emitter.instruction("mov rsp, rbp");                                // discard borrowed inputs and replacement temporaries
            emitter.instruction("pop rbp");                                     // restore the caller frame pointer
            emitter.instruction("ret");                                         // return the boolean restoration status
        }
    }
}

/// Checks object type hints through native class/interface tables without linking eval.
fn emit_restore_slot(
    module: &Module, emitter: &mut Emitter, data: &mut DataSection,
    slot: &EvalPropertySlot, fail: &str,
) {
    if let PhpType::Object(class_name) = slot.ty.codegen_repr() {
        if !class_name.is_empty() {
            let target = module.class_infos.get(&class_name).map(|info| (info.class_id, 0))
                .or_else(|| module.interface_infos.get(&class_name).map(|info| (info.interface_id, 1)));
            let Some((id, kind)) = target else {
                abi::emit_jump(emitter, fail);
                return;
            };
            match module.target.arch {
                Arch::AArch64 => {
                    emitter.instruction("ldr x0, [sp, #24]");                   // borrow the assigned cell for native object type validation
                }
                Arch::X86_64 => {
                    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");       // borrow the assigned cell for native object type validation
                }
            }
            abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(module.target, 1), id as i64);
            abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(module.target, 2), kind);
            abi::emit_call_label(emitter, "__rt_mixed_instanceof");
            abi::emit_branch_if_int_result_zero(emitter, fail);
        }
        match module.target.arch {
            Arch::AArch64 => emit_aarch64_store_heap_property_slot(emitter, slot, 6, fail),
            Arch::X86_64 => emit_x86_64_store_heap_property_slot(emitter, slot, 6, fail),
        }
    } else {
        match module.target.arch {
            Arch::AArch64 => emit_aarch64_store_property_slot(module, emitter, data, slot, fail),
            Arch::X86_64 => emit_x86_64_store_property_slot(module, emitter, data, slot, fail),
        }
    }
}
