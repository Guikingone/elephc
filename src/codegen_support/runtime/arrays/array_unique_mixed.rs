//! Purpose:
//! Emits `__rt_array_unique_mixed`, which de-duplicates indexed arrays whose
//! 8-byte slots contain boxed Mixed cells.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()`.
//!
//! Key details:
//! - Default `array_unique()` semantics compare each value after PHP string
//!   conversion, rather than comparing Mixed-cell addresses.
//! - Accepted values are retained into a fresh result array so the caller can
//!   release a gradual-boundary source rebuild without invalidating the result.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the target-specific Mixed-slot `array_unique()` runtime helper.
///
/// The input is an indexed-array pointer in `x0`/`rdi`; the result is a fresh
/// indexed array in `x0`/`rax`. Each candidate is stringified through
/// `__rt_mixed_cast_string`, compared byte-for-byte, and retained only on its
/// first occurrence.
pub fn emit_array_unique_mixed(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emitter.blank();
        emitter.comment("--- runtime: array_unique_mixed ---");
        emitter.label_global("__rt_array_unique_mixed");

        // -- preserve the source and allocate the worst-case result capacity --
        emitter.instruction("push rbp");                                            // preserve the caller frame pointer across nested runtime calls
        emitter.instruction("mov rbp, rsp");                                        // establish a stable frame for source, result, cursors, and string scratch
        emitter.instruction("sub rsp, 64");                                         // reserve eight aligned spill slots while preserving the SysV call alignment
        emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the source boxed-Mixed indexed-array pointer
        emitter.instruction("mov r10, QWORD PTR [rdi]");                            // load the source logical length
        emitter.instruction("mov QWORD PTR [rbp - 16], r10");                       // preserve the source length across allocation and comparison calls
        emitter.instruction("mov rdi, r10");                                        // use the source length as the worst-case destination capacity
        emitter.instruction("mov rsi, 8");                                          // allocate pointer-sized slots for retained Mixed cells
        emitter.instruction("call __rt_array_new");                                 // allocate the fresh de-duplicated indexed array
        emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the destination pointer across comparison and append calls
        emitter.instruction("mov QWORD PTR [rbp - 32], 0");                         // initialize the source cursor

        emitter.label("__rt_array_unique_mixed_outer_x86");
        emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // reload the source cursor
        emitter.instruction("cmp rcx, QWORD PTR [rbp - 16]");                       // compare the source cursor with the logical source length
        emitter.instruction("jge __rt_array_unique_mixed_done_x86");                 // finish after every source Mixed cell has been considered
        emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the source indexed-array pointer
        emitter.instruction("mov rax, QWORD PTR [r10 + 24 + rcx * 8]");             // load the current candidate Mixed-cell pointer
        emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // preserve the candidate across string conversions
        emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // initialize the destination scan cursor

        emitter.label("__rt_array_unique_mixed_inner_x86");
        emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // reload the destination scan cursor
        emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the current destination pointer
        emitter.instruction("cmp rdx, QWORD PTR [r10]");                            // compare the scan cursor with the accepted-value count
        emitter.instruction("jge __rt_array_unique_mixed_add_x86");                  // append when no accepted value has the same string form
        emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // pass the candidate Mixed cell to PHP string conversion
        emitter.instruction("call __rt_mixed_cast_string");                         // materialize the candidate's PHP string pointer and length
        emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // save the candidate string pointer across the existing-value conversion
        emitter.instruction("mov QWORD PTR [rbp - 64], rdx");                       // save the candidate string length across the existing-value conversion
        emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the destination after the conversion call
        emitter.instruction("mov r11, QWORD PTR [rbp - 48]");                       // reload the current destination scan cursor
        emitter.instruction("mov rax, QWORD PTR [r10 + 24 + r11 * 8]");             // pass the accepted Mixed cell to PHP string conversion
        emitter.instruction("call __rt_mixed_cast_string");                         // materialize the accepted value's PHP string pointer and length
        emitter.instruction("mov rcx, rdx");                                        // pass the accepted string length as the fourth strcmp argument
        emitter.instruction("mov rdx, rax");                                        // pass the accepted string pointer as the third strcmp argument
        emitter.instruction("mov rdi, QWORD PTR [rbp - 56]");                       // reload the candidate string pointer as the first strcmp argument
        emitter.instruction("mov rsi, QWORD PTR [rbp - 64]");                       // reload the candidate string length as the second strcmp argument
        emitter.instruction("call __rt_strcmp");                                    // compare default SORT_STRING representations byte-for-byte
        emitter.instruction("test rax, rax");                                       // did the candidate match an already accepted value?
        emitter.instruction("je __rt_array_unique_mixed_skip_x86");                  // discard a duplicate candidate while preserving the first occurrence
        emitter.instruction("add QWORD PTR [rbp - 48], 1");                         // advance to the next accepted destination value
        emitter.instruction("jmp __rt_array_unique_mixed_inner_x86");                // continue scanning for a duplicate string representation

        emitter.label("__rt_array_unique_mixed_add_x86");
        emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // pass the current destination pointer to the retaining append helper
        emitter.instruction("mov rsi, QWORD PTR [rbp - 40]");                       // pass the borrowed candidate Mixed cell to be retained
        emitter.instruction("call __rt_array_push_refcounted");                     // append and retain the first occurrence, growing the result if needed
        emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // persist a possibly relocated destination pointer

        emitter.label("__rt_array_unique_mixed_skip_x86");
        emitter.instruction("add QWORD PTR [rbp - 32], 1");                         // advance to the next source candidate
        emitter.instruction("jmp __rt_array_unique_mixed_outer_x86");                // continue de-duplicating the source array

        emitter.label("__rt_array_unique_mixed_done_x86");
        emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // return the fresh retained Mixed-slot result array
        emitter.instruction("add rsp, 64");                                         // release all helper spill slots
        emitter.instruction("pop rbp");                                             // restore the caller frame pointer
        emitter.instruction("ret");                                                 // return the de-duplicated indexed array
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: array_unique_mixed ---");
    emitter.label_global("__rt_array_unique_mixed");

    // -- preserve the source and allocate the worst-case result capacity --
    emitter.instruction("sub sp, sp, #80");                                     // reserve source, result, cursor, string scratch, and saved frame slots
    emitter.instruction("stp x29, x30, [sp, #64]");                             // preserve the caller frame pointer and return address
    emitter.instruction("add x29, sp, #64");                                    // establish a stable helper frame
    emitter.instruction("str x0, [sp, #0]");                                    // save the source boxed-Mixed indexed-array pointer
    emitter.instruction("ldr x9, [x0]");                                        // load the source logical length
    emitter.instruction("str x9, [sp, #8]");                                    // preserve the source length across allocation and comparison calls
    emitter.instruction("mov x0, x9");                                          // use the source length as the worst-case destination capacity
    emitter.instruction("mov x1, #8");                                          // allocate pointer-sized slots for retained Mixed cells
    emitter.instruction("bl __rt_array_new");                                   // allocate the fresh de-duplicated indexed array
    emitter.instruction("str x0, [sp, #16]");                                   // save the destination pointer across comparison and append calls
    emitter.instruction("str xzr, [sp, #24]");                                  // initialize the source cursor

    emitter.label("__rt_array_unique_mixed_outer");
    emitter.instruction("ldr x4, [sp, #24]");                                   // reload the source cursor
    emitter.instruction("ldr x9, [sp, #8]");                                    // reload the logical source length
    emitter.instruction("cmp x4, x9");                                          // compare the source cursor with the logical source length
    emitter.instruction("b.ge __rt_array_unique_mixed_done");                   // finish after every source Mixed cell has been considered
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload the source indexed-array pointer
    emitter.instruction("add x10, x10, #24");                                   // address the first source payload slot
    emitter.instruction("ldr x6, [x10, x4, lsl #3]");                           // load the current candidate Mixed-cell pointer
    emitter.instruction("str x6, [sp, #32]");                                   // preserve the candidate across string conversions
    emitter.instruction("str xzr, [sp, #40]");                                  // initialize the destination scan cursor

    emitter.label("__rt_array_unique_mixed_inner");
    emitter.instruction("ldr x7, [sp, #40]");                                   // reload the destination scan cursor
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the current destination pointer
    emitter.instruction("ldr x5, [x10]");                                       // load the number of values already accepted
    emitter.instruction("cmp x7, x5");                                          // compare the scan cursor with the accepted-value count
    emitter.instruction("b.ge __rt_array_unique_mixed_add");                    // append when no accepted value has the same string form
    emitter.instruction("ldr x0, [sp, #32]");                                   // pass the candidate Mixed cell to PHP string conversion
    emitter.instruction("bl __rt_mixed_cast_string");                           // materialize the candidate's PHP string pointer and length
    emitter.instruction("stp x1, x2, [sp, #48]");                              // save the candidate string across the existing-value conversion
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the destination after the conversion call
    emitter.instruction("ldr x7, [sp, #40]");                                   // reload the current destination scan cursor
    emitter.instruction("add x10, x10, #24");                                   // address the first accepted destination payload slot
    emitter.instruction("ldr x0, [x10, x7, lsl #3]");                           // pass the accepted Mixed cell to PHP string conversion
    emitter.instruction("bl __rt_mixed_cast_string");                           // materialize the accepted value's PHP string pointer and length
    emitter.instruction("mov x3, x1");                                          // pass the accepted string pointer as the third strcmp argument
    emitter.instruction("mov x4, x2");                                          // pass the accepted string length as the fourth strcmp argument
    emitter.instruction("ldp x1, x2, [sp, #48]");                              // reload the candidate string pointer and length
    emitter.instruction("bl __rt_strcmp");                                      // compare default SORT_STRING representations byte-for-byte
    emitter.instruction("cbz x0, __rt_array_unique_mixed_skip");                // discard a duplicate candidate while preserving the first occurrence
    emitter.instruction("ldr x7, [sp, #40]");                                   // reload the destination scan cursor after string comparison
    emitter.instruction("add x7, x7, #1");                                      // advance to the next accepted destination value
    emitter.instruction("str x7, [sp, #40]");                                   // persist the advanced destination scan cursor
    emitter.instruction("b __rt_array_unique_mixed_inner");                     // continue scanning for a duplicate string representation

    emitter.label("__rt_array_unique_mixed_add");
    emitter.instruction("ldr x0, [sp, #16]");                                   // pass the current destination pointer to the retaining append helper
    emitter.instruction("ldr x1, [sp, #32]");                                   // pass the borrowed candidate Mixed cell to be retained
    emitter.instruction("bl __rt_array_push_refcounted");                       // append and retain the first occurrence, growing the result if needed
    emitter.instruction("str x0, [sp, #16]");                                   // persist a possibly relocated destination pointer

    emitter.label("__rt_array_unique_mixed_skip");
    emitter.instruction("ldr x4, [sp, #24]");                                   // reload the source cursor after nested runtime calls
    emitter.instruction("add x4, x4, #1");                                      // advance to the next source candidate
    emitter.instruction("str x4, [sp, #24]");                                   // persist the advanced source cursor
    emitter.instruction("b __rt_array_unique_mixed_outer");                     // continue de-duplicating the source array

    emitter.label("__rt_array_unique_mixed_done");
    emitter.instruction("ldr x0, [sp, #16]");                                   // return the fresh retained Mixed-slot result array
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore the caller frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // release all helper frame slots
    emitter.instruction("ret");                                                 // return the de-duplicated indexed array
}
