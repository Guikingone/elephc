//! Purpose:
//! Emits `__rt_constant`, the runtime helper backing a non-literal
//! `constant($name)` call: it resolves a registered scalar constant into an
//! owned boxed Mixed, or throws a catchable `\Error` on a miss.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` (gated by the
//!   `const_introspection` feature) and the EIR `constant` lowering.
//!
//! Key details:
//! - Input:  x0/rdi=name_ptr, x1/rsi=name_len.
//! - Output: x0/rax = boxed Mixed pointer on a hit (does not return on a miss).
//! - On a hit, the entry's pre-boxed `(tag, lo, hi)` triple is handed to
//!   `__rt_mixed_from_value`, which owns the payload (string persist / refcount).
//! - On a miss, the helper builds the PHP 8 message `Undefined constant "NAME"`
//!   with `__rt_concat`/`__rt_str_persist`, stamps a `\Error` object
//!   (`_constant_error_class_id`), publishes it to `_exc_value`, and branches to
//!   `__rt_throw_current`. Class-constant / enum-case names are not in the table
//!   (Stage 0 deferral), so they correctly take this throwing path.

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Byte length of the `Undefined constant "` message prefix.
const MSG_PREFIX_LEN: i64 = 20;

/// Emits the target-specific `__rt_constant` constant-resolution helper.
pub fn emit_rt_constant(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// Emits the AArch64 implementation of `__rt_constant`.
fn emit_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: constant ---");
    emitter.label_global("__rt_constant");

    // -- frame and query spill (slots reused for the owned message on a miss) --
    emitter.instruction("sub sp, sp, #32");                                     // reserve the helper frame
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish the new frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // spill the constant name pointer
    emitter.instruction("str x1, [sp, #8]");                                    // spill the constant name length

    // -- search the constant registry --
    abi::emit_symbol_address(emitter, "x2", "_const_table");
    abi::emit_load_symbol_to_reg(emitter, "x3", "_const_table_count", 0);
    emitter.instruction("mov x4, #40");                                         // constant entries are 40 bytes wide
    emitter.instruction("bl __rt_sorted_name_search");                          // locate the constant entry
    emitter.instruction("cbz x0, __rt_constant_miss");                          // a null entry means the constant is undefined

    // -- hit: reconstruct an owned boxed Mixed from the stored payload --
    emitter.instruction("mov x9, x0");                                          // keep the matched entry pointer
    emitter.instruction("ldr x0, [x9, #16]");                                   // load the boxed Mixed tag
    emitter.instruction("ldr x1, [x9, #24]");                                   // load the payload low word
    emitter.instruction("ldr x2, [x9, #32]");                                   // load the payload high word
    emitter.instruction("bl __rt_mixed_from_value");                            // box the stored scalar into an owned Mixed
    emitter.instruction("b __rt_constant_done");                                // return the boxed value

    // -- miss: build `Undefined constant "NAME"` and throw a catchable \Error --
    emitter.label("__rt_constant_miss");
    abi::emit_symbol_address(emitter, "x1", "_constant_undefined_msg_prefix");
    emitter.instruction(&format!("mov x2, #{}", MSG_PREFIX_LEN));               // message prefix byte length
    emitter.instruction("ldr x3, [sp, #0]");                                    // reload the constant name pointer
    emitter.instruction("ldr x4, [sp, #8]");                                    // reload the constant name length
    emitter.instruction("bl __rt_concat");                                      // prefix concatenated with the name
    abi::emit_symbol_address(emitter, "x3", "_constant_undefined_msg_suffix");
    emitter.instruction("mov x4, #1");                                          // closing-quote suffix byte length
    emitter.instruction("bl __rt_concat");                                      // append the closing quote to the message
    emitter.instruction("bl __rt_str_persist");                                 // own a heap copy of the message bytes
    emitter.instruction("str x1, [sp, #0]");                                    // park the owned message pointer
    emitter.instruction("str x2, [sp, #8]");                                    // park the owned message length
    emitter.instruction("mov x0, #32");                                         // request Throwable payload storage
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the \Error object payload
    emitter.instruction("mov x9, #6");                                          // heap kind 6 = object instance
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp the allocation as a runtime object
    abi::emit_load_symbol_to_reg(emitter, "x9", "_constant_error_class_id", 0);
    emitter.instruction("str x9, [x0]");                                        // store the \Error class id at the object header
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the owned message pointer
    emitter.instruction("str x9, [x0, #8]");                                    // store the exception message pointer
    emitter.instruction("ldr x9, [sp, #8]");                                    // reload the owned message length
    emitter.instruction("str x9, [x0, #16]");                                   // store the exception message length
    emitter.instruction("str xzr, [x0, #24]");                                  // exception code defaults to zero
    abi::emit_symbol_address(emitter, "x9", "_exc_value");
    emitter.instruction("str x0, [x9]");                                        // publish the active exception object
    emitter.instruction("b __rt_throw_current");                                // enter the standard exception unwinder

    emitter.label("__rt_constant_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return the boxed Mixed pointer in x0
}

/// Emits the x86_64 implementation of `__rt_constant`.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: constant ---");
    emitter.label_global("__rt_constant");

    // -- frame and query spill (slots reused for the owned message on a miss) --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 32");                                         // reserve local slots for the query and message
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // spill the constant name pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // spill the constant name length

    // -- search the constant registry --
    abi::emit_symbol_address(emitter, "rdx", "_const_table");
    abi::emit_load_symbol_to_reg(emitter, "rcx", "_const_table_count", 0);
    emitter.instruction("mov r8, 40");                                          // constant entries are 40 bytes wide
    emitter.instruction("call __rt_sorted_name_search");                        // locate the constant entry
    emitter.instruction("test rax, rax");                                       // did the search find a matching entry?
    emitter.instruction("je __rt_constant_miss");                               // a null entry means the constant is undefined

    // -- hit: reconstruct an owned boxed Mixed from the stored payload --
    emitter.instruction("mov r9, rax");                                         // keep the matched entry pointer
    emitter.instruction("mov rax, QWORD PTR [r9 + 16]");                        // load the boxed Mixed tag
    emitter.instruction("mov rdi, QWORD PTR [r9 + 24]");                        // load the payload low word
    emitter.instruction("mov rsi, QWORD PTR [r9 + 32]");                        // load the payload high word
    emitter.instruction("call __rt_mixed_from_value");                          // box the stored scalar into an owned Mixed
    emitter.instruction("jmp __rt_constant_done");                              // return the boxed value

    // -- miss: build `Undefined constant "NAME"` and throw a catchable \Error --
    emitter.label("__rt_constant_miss");
    abi::emit_symbol_address(emitter, "rax", "_constant_undefined_msg_prefix");
    emitter.instruction(&format!("mov rdx, {}", MSG_PREFIX_LEN));               // message prefix byte length
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the constant name pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // reload the constant name length
    emitter.instruction("call __rt_concat");                                    // prefix concatenated with the name
    abi::emit_symbol_address(emitter, "rdi", "_constant_undefined_msg_suffix");
    emitter.instruction("mov rsi, 1");                                          // closing-quote suffix byte length
    emitter.instruction("call __rt_concat");                                    // append the closing quote to the message
    emitter.instruction("mov rdi, rax");                                        // move the message pointer into the persist argument
    emitter.instruction("call __rt_str_persist");                               // own a heap copy of the message bytes
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // park the owned message pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // park the owned message length
    emitter.instruction("mov rax, 32");                                         // request Throwable payload storage
    emitter.instruction("call __rt_heap_alloc");                                // allocate the \Error object payload
    emitter.instruction("mov r10, 0x4548504c00000006");                         // x86_64 heap-kind word: object magic + kind 6
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp the allocation as a runtime object
    abi::emit_load_symbol_to_reg(emitter, "r10", "_constant_error_class_id", 0);
    emitter.instruction("mov QWORD PTR [rax], r10");                            // store the \Error class id at the object header
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the owned message pointer
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // store the exception message pointer
    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                       // reload the owned message length
    emitter.instruction("mov QWORD PTR [rax + 16], r10");                       // store the exception message length
    emitter.instruction("mov QWORD PTR [rax + 24], 0");                         // exception code defaults to zero
    abi::emit_store_reg_to_symbol(emitter, "rax", "_exc_value", 0);
    emitter.instruction("mov rsp, rbp");                                        // release the helper frame before throwing
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before throwing
    emitter.instruction("jmp __rt_throw_current");                              // enter the standard exception unwinder

    emitter.label("__rt_constant_done");
    emitter.instruction("mov rsp, rbp");                                        // discard the helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the boxed Mixed pointer in rax
}
