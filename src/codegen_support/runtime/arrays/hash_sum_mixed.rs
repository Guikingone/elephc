//! Purpose:
//! Emits `__rt_hash_sum_mixed` for associative arrays whose values are boxed Mixed cells.
//! Coerces values through the shared PHP integer-cast helper while preserving insertion order.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()`.
//!
//! Key details:
//! - The helper supports ARM64 and Linux x86_64 ABIs and only borrows hash entries and Mixed boxes.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the target-specific boxed-Mixed associative-array sum helper.
///
/// The hash pointer arrives in `x0`/`rdi`. Entries tagged 7 carry a borrowed Mixed-cell
/// pointer; raw scalar entries (another tag) are boxed temporarily so they are cast the same way.
/// The integer result is returned in `x0`/`rax`.
pub fn emit_hash_sum_mixed(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_hash_sum_mixed_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: hash_sum_mixed ---");
    emitter.label_global("__rt_hash_sum_mixed");

    // -- preserve iteration state across hash and coercion helper calls --
    emitter.instruction("sub sp, sp, #64");                                     // reserve aligned storage for hash pointer, cursor, accumulator, and saved linkage
    emitter.instruction("stp x29, x30, [sp, #48]");                             // preserve the caller frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish the helper frame pointer above local iteration state
    emitter.instruction("str x0, [sp]");                                        // save the source associative-array pointer
    emitter.instruction("str xzr, [sp, #8]");                                   // initialize the insertion-order cursor
    emitter.instruction("str xzr, [sp, #16]");                                  // initialize the integer accumulator

    // -- visit, coerce, and accumulate each boxed value --
    emitter.label("__rt_hash_sum_mixed_loop");
    emitter.instruction("ldr x0, [sp]");                                        // reload the source hash pointer for the iterator
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload the insertion-order cursor
    emitter.instruction("bl __rt_hash_iter_next");                              // fetch the next entry with cursor x0, payload x3/x4, and tag x5
    emitter.instruction("cmn x0, #1");                                          // did the iterator return its terminal negative-one cursor?
    emitter.instruction("b.eq __rt_hash_sum_mixed_done");                       // finish after every associative entry has contributed
    emitter.instruction("str x0, [sp, #8]");                                    // preserve the next insertion-order cursor
    emitter.instruction("cmp x5, #7");                                          // is the entry a boxed Mixed cell?
    emitter.instruction("b.ne __rt_hash_sum_mixed_raw");                        // raw scalar slots are boxed temporarily below
    emitter.instruction("mov x0, x3");                                          // pass the borrowed Mixed-cell pointer to the integer-cast helper
    emitter.instruction("bl __rt_mixed_cast_int");                              // coerce the boxed value with PHP integer conversion rules
    emitter.instruction("b __rt_hash_sum_mixed_add");                           // accumulate it
    // -- a raw slot (e.g. an int stored without boxing) goes through a temporary box --
    emitter.label("__rt_hash_sum_mixed_raw");
    emitter.instruction("mov x0, x5");                                          // runtime tag
    emitter.instruction("mov x1, x3");                                          // payload low word
    emitter.instruction("mov x2, x4");                                          // payload high word
    emitter.instruction("bl __rt_mixed_from_value");                            // x0 = temporary Mixed box
    emitter.instruction("str x0, [sp, #24]");                                   // keep it for the release
    emitter.instruction("bl __rt_mixed_cast_int");                              // coerce with PHP integer conversion rules
    emitter.instruction("str x0, [sp, #32]");                                   // keep the coerced value
    emitter.instruction("ldr x0, [sp, #24]");                                   // temporary box
    emitter.instruction("bl __rt_decref_mixed");                                // release it
    emitter.instruction("ldr x0, [sp, #32]");                                   // coerced value
    emitter.label("__rt_hash_sum_mixed_add");
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload the running integer accumulator
    emitter.instruction("add x9, x9, x0");                                      // add the coerced associative value to the sum
    emitter.instruction("str x9, [sp, #16]");                                   // preserve the updated accumulator
    emitter.instruction("b __rt_hash_sum_mixed_loop");                          // continue with the next insertion-order entry

    // -- return the accumulated integer --
    emitter.label("__rt_hash_sum_mixed_done");
    emitter.instruction("ldr x0, [sp, #16]");                                   // return zero for empty hashes or the accumulated integer sum
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore the caller frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the aligned helper frame
    emitter.instruction("ret");                                                 // return the integer sum to generated code
}

/// Emits the Linux x86_64 System V implementation of boxed-Mixed hash sum.
fn emit_hash_sum_mixed_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hash_sum_mixed ---");
    emitter.label_global("__rt_hash_sum_mixed");

    // -- preserve iteration state across hash and coercion helper calls --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer and align the stack for nested calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for iteration locals
    emitter.instruction("sub rsp, 32");                                         // reserve hash pointer, cursor, and integer accumulator slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the source associative-array pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], 0");                         // initialize the insertion-order cursor
    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                         // initialize the integer accumulator

    // -- visit, coerce, and accumulate each boxed value --
    emitter.label("__rt_hash_sum_mixed_loop_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the source hash pointer for the iterator
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // reload the insertion-order cursor
    emitter.instruction("call __rt_hash_iter_next");                            // fetch the next entry with cursor rax, payload rcx/r8, and tag r9
    emitter.instruction("cmp rax, -1");                                         // did the iterator return its terminal cursor?
    emitter.instruction("je __rt_hash_sum_mixed_done_x86");                     // finish after every associative entry has contributed
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // preserve the next insertion-order cursor
    emitter.instruction("cmp r9, 7");                                           // is the entry a boxed Mixed cell?
    emitter.instruction("jne __rt_hash_sum_mixed_raw_x86");                     // raw scalar slots are boxed temporarily below
    emitter.instruction("mov rax, rcx");                                        // pass the borrowed Mixed-cell pointer in the x86 Mixed-helper input register
    emitter.instruction("call __rt_mixed_cast_int");                            // coerce the boxed value with PHP integer conversion rules
    emitter.instruction("jmp __rt_hash_sum_mixed_add_x86");                     // accumulate it
    // -- a raw slot (e.g. an int stored without boxing) goes through a temporary box --
    emitter.label("__rt_hash_sum_mixed_raw_x86");
    emitter.instruction("mov rax, r9");                                         // runtime tag
    emitter.instruction("mov rdi, rcx");                                        // payload low word
    emitter.instruction("mov rsi, r8");                                         // payload high word
    emitter.instruction("call __rt_mixed_from_value");                          // rax = temporary Mixed box
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // keep it for the release
    emitter.instruction("call __rt_mixed_cast_int");                            // coerce with PHP integer conversion rules
    emitter.instruction("push rax");                                            // keep the coerced value
    emitter.instruction("sub rsp, 8");                                          // keep the stack 16-byte aligned
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // temporary box
    emitter.instruction("call __rt_decref_mixed");                              // release it
    emitter.instruction("add rsp, 8");                                          // undo the alignment pad
    emitter.instruction("pop rax");                                             // coerced value
    emitter.label("__rt_hash_sum_mixed_add_x86");
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // add the coerced associative value to the running sum
    emitter.instruction("jmp __rt_hash_sum_mixed_loop_x86");                    // continue with the next insertion-order entry

    // -- return the accumulated integer --
    emitter.label("__rt_hash_sum_mixed_done_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // return zero for empty hashes or the accumulated integer sum
    emitter.instruction("mov rsp, rbp");                                        // release helper-local iteration storage
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the integer sum to generated code
}
