//! Purpose:
//! Emits `__rt_array_from_name_list`, the runtime loop that materializes a PHP indexed
//! `array<string>` from a resolved `{name_ptr, name_len}` list — the runtime counterpart of
//! `get_class_methods()`'s compile-time unrolled array construction, driven by a count only
//! known at runtime.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::get_class_methods`, after
//!   `__rt_sorted_name_search` resolves a `_class_methods_table` row.
//!
//! Key details:
//! - Input:  x0/rdi=list_ptr, x1/rsi=list_count.
//! - Output: x0/rax = the built array pointer (an empty array when `list_count == 0`).
//! - Per-entry sequence mirrors
//!   `crate::codegen::lower_inst::builtins::get_class_methods::emit_string_array_fill_*`
//!   exactly (`__rt_array_new` with a 16-byte element stride, then `__rt_array_push_str` per
//!   name), just driven by a runtime loop instead of unrolled per compile-time-known name.
//!   This is what makes a dynamic call and a literal call return the same array layout.
//! - Sibling of `__rt_hash_from_name_list`, which builds the `name => name` ASSOC hash the
//!   `class_implements()` family needs; `get_class_methods()` returns a plain `0..n-1` list, so
//!   it needs this one instead. The two are deliberately separate rather than one parameterized
//!   helper: they share no per-entry work (no key normalization, no value persist here).
//! - None of the called runtime helpers preserve caller registers beyond the frame
//!   pointer/return address, so every value that must survive a nested call is spilled to a
//!   fixed stack slot before the call and reloaded after.

use crate::codegen::{emit::Emitter, platform::Arch};

/// Element stride of an indexed PHP array slot, matching every `__rt_array_new` call site.
const ELEMENT_STRIDE: i64 = 16;

/// Emits the target-specific `__rt_array_from_name_list` helper.
pub fn emit_rt_array_from_name_list(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// Emits the AArch64 implementation of `__rt_array_from_name_list`.
///
/// Spill layout (sp-relative, 48-byte frame): #16 cursor_ptr, #24 remaining_count, #32 array_ptr.
fn emit_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_from_name_list ---");
    emitter.label_global("__rt_array_from_name_list");

    emitter.instruction("stp x29, x30, [sp, #-48]!");                           // save frame pointer/return address, reserve 3 spill slots
    emitter.instruction("mov x29, sp");                                         // establish the new frame pointer

    emitter.instruction("str x0, [sp, #16]");                                   // cursor_ptr = list_ptr
    emitter.instruction("str x1, [sp, #24]");                                   // remaining_count = list_count

    // -- allocate the result array: capacity = max(list_count, 1), matching the literal path --
    emitter.instruction("mov x9, #1");                                          // x9 = the minimum capacity
    emitter.instruction("cmp x1, x9");                                          // is list_count below the minimum?
    emitter.instruction("csel x0, x9, x1, lo");                                 // x0 = max(list_count, 1)
    emitter.instruction(&format!("mov x1, #{}", ELEMENT_STRIDE));               // element stride argument for __rt_array_new
    emitter.instruction("bl __rt_array_new");                                   // x0 = new empty indexed array
    emitter.instruction("str x0, [sp, #32]");                                   // array_ptr = the new empty array

    emitter.instruction("b __rt_array_from_name_list_check");                   // enter the loop at the remaining-count test

    emitter.label("__rt_array_from_name_list_top");
    emitter.instruction("ldr x3, [sp, #16]");                                   // reload cursor_ptr
    emitter.instruction("ldr x1, [x3]");                                        // name_ptr for this entry
    emitter.instruction("ldr x2, [x3, #8]");                                    // name_len for this entry
    emitter.instruction("add x3, x3, #16");                                     // advance the cursor to the next entry
    emitter.instruction("str x3, [sp, #16]");                                   // save the advanced cursor
    emitter.instruction("ldr x4, [sp, #24]");                                   // reload remaining_count
    emitter.instruction("sub x4, x4, #1");                                      // one fewer entry left to append
    emitter.instruction("str x4, [sp, #24]");                                   // save the decremented remaining count
    emitter.instruction("ldr x0, [sp, #32]");                                   // reload the current array pointer
    emitter.instruction("bl __rt_array_push_str");                              // x0 = the (possibly grown) updated array
    emitter.instruction("str x0, [sp, #32]");                                   // save the updated array pointer

    emitter.label("__rt_array_from_name_list_check");
    emitter.instruction("ldr x4, [sp, #24]");                                   // reload remaining_count
    emitter.instruction("cbnz x4, __rt_array_from_name_list_top");              // append the next entry while entries remain

    emitter.instruction("ldr x0, [sp, #32]");                                   // the finished array pointer is the return value
    emitter.instruction("ldp x29, x30, [sp], #48");                             // restore frame pointer and return address
    emitter.instruction("ret");                                                 // return the built array in x0
}

/// Emits the x86_64 implementation of `__rt_array_from_name_list`.
///
/// Spill layout (rbp-relative, 48-byte frame): -8 cursor_ptr, -16 remaining_count, -24 array_ptr.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_from_name_list ---");
    emitter.label_global("__rt_array_from_name_list");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the new frame pointer
    emitter.instruction("sub rsp, 48");                                         // reserve 3 spill slots (16-byte aligned)

    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // cursor_ptr = list_ptr
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // remaining_count = list_count

    // -- allocate the result array: capacity = max(list_count, 1), matching the literal path --
    emitter.instruction("mov rax, rsi");                                        // rax = list_count
    emitter.instruction("cmp rax, 1");                                          // is list_count below the minimum capacity?
    emitter.instruction("jae __rt_array_from_name_list_cap_ok");                // keep list_count when it already meets the minimum
    emitter.instruction("mov rax, 1");                                          // otherwise clamp the capacity to the minimum

    emitter.label("__rt_array_from_name_list_cap_ok");
    emitter.instruction("mov rdi, rax");                                        // capacity argument for __rt_array_new
    emitter.instruction(&format!("mov rsi, {}", ELEMENT_STRIDE));               // element stride argument for __rt_array_new
    emitter.instruction("call __rt_array_new");                                 // rax = new empty indexed array
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // array_ptr = the new empty array

    emitter.instruction("jmp __rt_array_from_name_list_check");                 // enter the loop at the remaining-count test

    emitter.label("__rt_array_from_name_list_top");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 8]");                        // reload cursor_ptr
    emitter.instruction("mov rsi, QWORD PTR [rcx]");                            // name_ptr for this entry
    emitter.instruction("mov rdx, QWORD PTR [rcx + 8]");                        // name_len for this entry
    emitter.instruction("add rcx, 16");                                         // advance the cursor to the next entry
    emitter.instruction("mov QWORD PTR [rbp - 8], rcx");                        // save the advanced cursor
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // reload remaining_count
    emitter.instruction("dec rcx");                                             // one fewer entry left to append
    emitter.instruction("mov QWORD PTR [rbp - 16], rcx");                       // save the decremented remaining count
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // reload the current array pointer
    emitter.instruction("call __rt_array_push_str");                            // rax = the (possibly grown) updated array
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the updated array pointer

    emitter.label("__rt_array_from_name_list_check");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // reload remaining_count
    emitter.instruction("test rcx, rcx");                                       // any entries left to append?
    emitter.instruction("jnz __rt_array_from_name_list_top");                   // append the next entry while entries remain

    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // the finished array pointer is the return value
    emitter.instruction("add rsp, 48");                                         // drop the spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the built array in rax
}
