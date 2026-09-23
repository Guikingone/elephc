//! Purpose:
//! Emits `__rt_class_name_is_a`, the runtime answer to `is_a($class, $target, true)` and
//! `is_subclass_of($class, $target)` when either name is only known at run time.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via
//!   `crate::codegen_support::runtime::exceptions`.
//!
//! Key details:
//! - Both names resolve through `__rt_instanceof_lookup`, and the relation is answered by
//!   `__rt_exception_matches` against a one-word fake object header holding the source class id,
//!   the same pair the eval bridge's string branch uses.
//! - A source naming an INTERFACE answers false: the matcher walks class parents and implemented
//!   interfaces of a class, and has no interface-to-interface walk.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_class_name_is_a`.
///
/// Inputs (ARM64): x0/x1 = source class name ptr/len, x2/x3 = target name ptr/len,
/// x4 = 1 to exclude the exact class (`is_subclass_of`).
/// Inputs (x86_64): rdi/rsi = source ptr/len, rdx/rcx = target ptr/len, r8 = exclude flag.
/// Output: x0 / rax = 1 when the relation holds, 0 otherwise (including an unknown name).
pub fn emit_class_name_is_a(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

fn emit_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: class_name_is_a ---");
    emitter.label_global("__rt_class_name_is_a");
    emitter.instruction("sub sp, sp, #64");                                     // reserve the saved names, target metadata and fake header
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish the relation helper frame
    emitter.instruction("str x0, [sp, #0]");                                    // keep the source class-name pointer across the target lookup
    emitter.instruction("str x1, [sp, #8]");                                    // keep the source class-name length across the target lookup
    emitter.instruction("str x4, [sp, #16]");                                   // keep the exact-class exclusion flag
    emitter.instruction("mov x1, x2");                                          // target name pointer -> lookup string pointer
    emitter.instruction("mov x2, x3");                                          // target name length -> lookup string length
    emitter.instruction("bl __rt_instanceof_lookup");                           // resolve the target to its class/interface id and kind
    emitter.instruction("cbz x0, __rt_class_name_is_a_false");                  // an unknown target name relates to nothing
    emitter.instruction("str x1, [sp, #24]");                                   // keep the target id
    emitter.instruction("str x2, [sp, #32]");                                   // keep the target kind: 0 class, 1 interface
    emitter.instruction("ldr x1, [sp, #0]");                                    // source name pointer -> lookup string pointer
    emitter.instruction("ldr x2, [sp, #8]");                                    // source name length -> lookup string length
    emitter.instruction("bl __rt_instanceof_lookup");                           // resolve the source class name to its class id
    emitter.instruction("cbz x0, __rt_class_name_is_a_false");                  // an unknown source class relates to nothing
    emitter.instruction("cbnz x2, __rt_class_name_is_a_false");                 // an interface source has no class-parent walk here
    emitter.instruction("str x1, [sp, #40]");                                   // the fake object header: its first word is the class id
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the exact-class exclusion flag
    emitter.instruction("cbz x10, __rt_class_name_is_a_match");                 // is_a() accepts the exact class
    emitter.instruction("ldr x11, [sp, #32]");                                  // reload the target kind
    emitter.instruction("cbnz x11, __rt_class_name_is_a_match");                // an interface target is never the exact class
    emitter.instruction("ldr x12, [sp, #24]");                                  // reload the target class id
    emitter.instruction("cmp x1, x12");                                         // is the source exactly the target class?
    emitter.instruction("b.eq __rt_class_name_is_a_false");                     // is_subclass_of() excludes the class itself
    emitter.label("__rt_class_name_is_a_match");
    emitter.instruction("add x0, sp, #40");                                     // pass the fake header as the tested object
    emitter.instruction("ldr x1, [sp, #24]");                                   // pass the target id
    emitter.instruction("ldr x2, [sp, #32]");                                   // pass the target kind
    emitter.instruction("bl __rt_exception_matches");                           // walk parents and implemented interfaces
    emitter.instruction("b __rt_class_name_is_a_done");                         // keep the matcher's answer
    emitter.label("__rt_class_name_is_a_false");
    emitter.instruction("mov x0, #0");                                          // the relation does not hold
    emitter.label("__rt_class_name_is_a_done");
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return the relation answer
}

fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: class_name_is_a ---");
    emitter.label_global("__rt_class_name_is_a");
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the relation helper frame
    emitter.instruction("sub rsp, 48");                                         // reserve saved names, target metadata and fake header
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // keep the source class-name pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // keep the source class-name length
    emitter.instruction("mov QWORD PTR [rbp - 24], r8");                        // keep the exact-class exclusion flag
    emitter.instruction("mov rax, rdx");                                        // target name pointer -> lookup string pointer
    emitter.instruction("mov rdx, rcx");                                        // target name length -> lookup string length
    emitter.instruction("call __rt_instanceof_lookup");                         // resolve the target to its class/interface id and kind
    emitter.instruction("test rax, rax");                                       // did the target name resolve?
    emitter.instruction("je __rt_class_name_is_a_false_x");                     // an unknown target name relates to nothing
    emitter.instruction("mov QWORD PTR [rbp - 32], rdi");                       // keep the target id
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // keep the target kind: 0 class, 1 interface
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // source name pointer -> lookup string pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // source name length -> lookup string length
    emitter.instruction("call __rt_instanceof_lookup");                         // resolve the source class name to its class id
    emitter.instruction("test rax, rax");                                       // did the source name resolve?
    emitter.instruction("je __rt_class_name_is_a_false_x");                     // an unknown source class relates to nothing
    emitter.instruction("test rdx, rdx");                                       // did the source name an interface?
    emitter.instruction("jne __rt_class_name_is_a_false_x");                    // an interface source has no class-parent walk here
    emitter.instruction("mov QWORD PTR [rbp - 48], rdi");                       // the fake object header: its first word is the class id
    emitter.instruction("cmp QWORD PTR [rbp - 24], 0");                         // is the exact class excluded?
    emitter.instruction("je __rt_class_name_is_a_match_x");                     // is_a() accepts the exact class
    emitter.instruction("cmp QWORD PTR [rbp - 40], 0");                         // is the target an interface?
    emitter.instruction("jne __rt_class_name_is_a_match_x");                    // an interface target is never the exact class
    emitter.instruction("cmp rdi, QWORD PTR [rbp - 32]");                       // is the source exactly the target class?
    emitter.instruction("je __rt_class_name_is_a_false_x");                     // is_subclass_of() excludes the class itself
    emitter.label("__rt_class_name_is_a_match_x");
    emitter.instruction("lea rdi, [rbp - 48]");                                 // pass the fake header as the tested object
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // pass the target id
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // pass the target kind
    emitter.instruction("call __rt_exception_matches");                         // walk parents and implemented interfaces
    emitter.instruction("movzx eax, al");                                       // the matcher answers in al; widen it to the result
    emitter.instruction("jmp __rt_class_name_is_a_done_x");                     // keep the matcher's answer
    emitter.label("__rt_class_name_is_a_false_x");
    emitter.instruction("xor eax, eax");                                        // the relation does not hold
    emitter.label("__rt_class_name_is_a_done_x");
    emitter.instruction("leave");                                               // release the helper frame and restore rbp
    emitter.instruction("ret");                                                 // return the relation answer
}
