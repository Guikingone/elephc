//! Purpose:
//! Emits the `__rt_array_search_str` runtime helper for linear search of string-element arrays.
//! Returns the first matching index (or -1) using byte-exact string comparison.
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::arrays`.
//!
//! Key details:
//! - String elements use 16-byte slots (pointer + length). Each element is compared against the
//!   needle through `__rt_str_eq`, so loop state is kept on the stack across that call. Mirrors
//!   `__rt_array_search` (integer) but for string-typed indexed arrays.

use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_array_search_str` runtime helper.
///
/// Input:  x0 = string-element array pointer, x1 = needle pointer, x2 = needle length
/// Output: x0 = first matching index, or -1 when the needle is absent
pub fn emit_array_search_str(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_search_str_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: array_search_str ---");
    emitter.label_global("__rt_array_search_str");

    // Stack layout:
    //   [sp, #0]  = needle pointer
    //   [sp, #8]  = needle length
    //   [sp, #16] = data region base (array + 24)
    //   [sp, #24] = array length
    //   [sp, #32] = loop index i
    //   [sp, #48] = saved x29
    //   [sp, #56] = saved x30
    emitter.instruction("sub sp, sp, #64");                                     // allocate the string-search stack frame
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // set up new frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save the needle pointer across the str_eq calls
    emitter.instruction("str x2, [sp, #8]");                                    // save the needle length across the str_eq calls
    emitter.instruction("ldr x9, [x0]");                                        // x9 = array length from the header
    emitter.instruction("str x9, [sp, #24]");                                   // save the array length
    emitter.instruction("add x10, x0, #24");                                    // x10 = base of the 16-byte string data region
    emitter.instruction("str x10, [sp, #16]");                                  // save the data region base
    emitter.instruction("str xzr, [sp, #32]");                                  // initialize the loop index to zero

    emitter.label("__rt_array_search_str_loop");
    emitter.instruction("ldr x11, [sp, #32]");                                  // reload the loop index
    emitter.instruction("ldr x9, [sp, #24]");                                   // reload the array length
    emitter.instruction("cmp x11, x9");                                         // have all elements been scanned?
    emitter.instruction("b.ge __rt_array_search_str_notfound");                 // stop once the scan reaches the array length

    // -- load string element at index i (16 bytes per element) --
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the data region base
    emitter.instruction("lsl x12, x11, #4");                                    // x12 = i * 16 (string slot stride)
    emitter.instruction("add x12, x10, x12");                                   // x12 = &element[i]
    emitter.instruction("ldr x1, [x12]");                                       // x1 = element string pointer
    emitter.instruction("ldr x2, [x12, #8]");                                   // x2 = element string length
    emitter.instruction("ldr x3, [sp, #0]");                                    // x3 = needle pointer
    emitter.instruction("ldr x4, [sp, #8]");                                    // x4 = needle length
    emitter.instruction("bl __rt_str_eq");                                      // x0 = 1 when the element equals the needle
    emitter.instruction("cbnz x0, __rt_array_search_str_found");                // return the current index on a match

    emitter.instruction("ldr x11, [sp, #32]");                                  // reload the loop index after str_eq clobbered it
    emitter.instruction("add x11, x11, #1");                                    // advance to the next element
    emitter.instruction("str x11, [sp, #32]");                                  // save the updated loop index
    emitter.instruction("b __rt_array_search_str_loop");                        // continue the linear scan

    emitter.label("__rt_array_search_str_found");
    emitter.instruction("ldr x0, [sp, #32]");                                   // return the matching index
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate the stack frame
    emitter.instruction("ret");                                                 // return the matching index

    emitter.label("__rt_array_search_str_notfound");
    emitter.instruction("mov x0, #-1");                                         // return the not-found sentinel
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate the stack frame
    emitter.instruction("ret");                                                 // return -1 to the caller
}

/// Emits the x86_64-linux implementation of `__rt_array_search_str`.
///
/// Input:  rdi = string-element array pointer, rsi = needle pointer, rdx = needle length
/// Output: rax = first matching index, or -1 when the needle is absent
fn emit_array_search_str_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_search_str ---");
    emitter.label_global("__rt_array_search_str");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the string-search bookkeeping
    emitter.instruction("sub rsp, 48");                                         // reserve aligned spill slots while keeping nested str_eq calls 16-byte aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                        // save the needle pointer across the str_eq calls
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the needle length across the str_eq calls
    emitter.instruction("mov rax, QWORD PTR [rdi]");                            // load the array length from the header
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the array length
    emitter.instruction("lea rax, [rdi + 24]");                                 // compute the base of the 16-byte string data region
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the data region base
    emitter.instruction("mov QWORD PTR [rbp - 40], 0");                         // initialize the loop index to zero

    emitter.label("__rt_array_search_str_loop");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 40]");                       // reload the loop index
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 24]");                       // have all elements been scanned?
    emitter.instruction("jge __rt_array_search_str_notfound");                  // stop once the scan reaches the array length
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload the data region base
    emitter.instruction("mov r11, rcx");                                        // copy the loop index before scaling it
    emitter.instruction("shl r11, 4");                                          // r11 = i * 16 (string slot stride)
    emitter.instruction("add r10, r11");                                        // r10 = &element[i]
    emitter.instruction("mov rdi, QWORD PTR [r10]");                            // rdi = element string pointer
    emitter.instruction("mov rsi, QWORD PTR [r10 + 8]");                        // rsi = element string length
    emitter.instruction("mov rdx, QWORD PTR [rbp - 8]");                        // rdx = needle pointer
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // rcx = needle length
    emitter.instruction("call __rt_str_eq");                                    // rax = 1 when the element equals the needle
    emitter.instruction("test rax, rax");                                       // did the current element match the needle?
    emitter.instruction("jne __rt_array_search_str_found");                     // return the current index on a match
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // reload the loop index after str_eq clobbered it
    emitter.instruction("add r10, 1");                                          // advance to the next element
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // save the updated loop index
    emitter.instruction("jmp __rt_array_search_str_loop");                      // continue the linear scan

    emitter.label("__rt_array_search_str_found");
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // return the matching index
    emitter.instruction("add rsp, 48");                                         // deallocate the spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the matching index

    emitter.label("__rt_array_search_str_notfound");
    emitter.instruction("mov rax, -1");                                         // return the not-found sentinel
    emitter.instruction("add rsp, 48");                                         // deallocate the spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return -1 to the caller
}
