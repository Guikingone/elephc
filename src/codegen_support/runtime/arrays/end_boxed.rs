//! Purpose:
//! Emits the `__rt_end_boxed` runtime helper for PHP `end()` on a boxed Mixed array.
//! Returns the last element of an indexed or associative array as an owned Mixed cell.
//!
//! Called from:
//! - `crate::codegen_support::runtime::arrays::emit_end_boxed()`.
//!
//! Key details:
//! - Tag 4 (indexed) reuses `__rt_mixed_array_get` to read and box the last element.
//! - Tag 5 (associative) reads the insertion-order tail entry and boxes its value.
//! - Empty arrays and non-array payloads return a boxed `false`, matching PHP `end()`.

use crate::codegen::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_end_boxed` runtime helper for `end()` on a boxed Mixed receiver.
/// Dispatches to the target-specific implementation.
pub fn emit_end_boxed(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_end_boxed_x86_64(emitter);
        return;
    }
    emit_end_boxed_aarch64(emitter);
}

/// Emits `__rt_end_boxed` for ARM64.
///
/// Input: `x0` = pointer to a boxed Mixed array.
/// Output: `x0` = owned Mixed cell holding the last element, or boxed `false` when the
/// receiver is null, empty, or not an array.
fn emit_end_boxed_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: end_boxed ---");
    emitter.label_global("__rt_end_boxed");

    // -- set up stack frame and save the receiver --
    emitter.instruction("sub sp, sp, #32");                                     // reserve frame: saved receiver + saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish the local frame
    emitter.instruction("str x0, [sp, #0]");                                    // save the boxed Mixed receiver pointer

    emitter.instruction("cbz x0, __rt_end_boxed_false");                        // null receiver → boxed false
    emitter.instruction("ldr x9, [x0]");                                        // load the Mixed runtime tag
    emitter.instruction("cmp x9, #4");                                          // tag 4 = indexed array?
    emitter.instruction("b.eq __rt_end_boxed_indexed");                         // read the last indexed element
    emitter.instruction("cmp x9, #5");                                          // tag 5 = associative array?
    emitter.instruction("b.eq __rt_end_boxed_assoc");                           // read the insertion-order tail value
    emitter.instruction("b __rt_end_boxed_false");                              // any other payload → boxed false

    // -- indexed array: the last element is at index length-1 --
    emitter.label("__rt_end_boxed_indexed");
    emitter.instruction("ldr x10, [x0, #8]");                                   // load the indexed-array payload pointer
    emitter.instruction("cbz x10, __rt_end_boxed_false");                       // defensive null guard for the payload
    emitter.instruction("ldr x11, [x10]");                                      // load the array length from the header
    emitter.instruction("subs x12, x11, #1");                                   // compute the last index (length - 1)
    emitter.instruction("b.lt __rt_end_boxed_false");                           // an empty array yields boxed false
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the boxed Mixed receiver
    emitter.instruction("mov x1, x12");                                         // pass the last index as the lookup key
    emitter.instruction("mov x2, #-1");                                         // key_hi = -1 marks an integer key
    emitter.instruction("bl __rt_mixed_array_get");                             // read and box the last element as an owned Mixed
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the owned Mixed result

    // -- associative array: read the insertion-order tail entry --
    emitter.label("__rt_end_boxed_assoc");
    emitter.instruction("ldr x10, [x0, #8]");                                   // load the hash payload pointer
    emitter.instruction("cbz x10, __rt_end_boxed_false");                       // defensive null guard for the hash
    emitter.instruction("ldr x11, [x10, #32]");                                 // load the insertion-order tail slot index
    emitter.instruction("cmn x11, #1");                                         // is the hash empty (tail == -1)?
    emitter.instruction("b.eq __rt_end_boxed_false");                           // empty hash yields boxed false
    emitter.instruction("mov x12, #64");                                        // hash entry stride in bytes
    emitter.instruction("mul x13, x11, x12");                                   // byte offset of the tail slot
    emitter.instruction("add x13, x10, x13");                                   // advance from the hash base to the slot
    emitter.instruction("add x13, x13, #40");                                   // skip the 40-byte hash header
    emitter.instruction("ldr x3, [x13, #24]");                                  // load the tail value low word
    emitter.instruction("ldr x4, [x13, #32]");                                  // load the tail value high word
    emitter.instruction("ldr x5, [x13, #40]");                                  // load the tail value runtime tag
    emitter.instruction("cmp x5, #7");                                          // is the entry already a boxed Mixed cell?
    emitter.instruction("b.eq __rt_end_boxed_assoc_boxed");                     // share the stored Mixed cell after a retain
    emitter.instruction("mov x0, x5");                                          // mixed_from_value tag argument
    emitter.instruction("mov x1, x3");                                          // mixed_from_value low payload argument
    emitter.instruction("mov x2, x4");                                          // mixed_from_value high payload argument
    emitter.instruction("bl __rt_mixed_from_value");                            // box the typed tail value into a fresh Mixed cell
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the owned Mixed result
    emitter.label("__rt_end_boxed_assoc_boxed");
    emitter.instruction("mov x0, x3");                                          // move the stored Mixed cell into the return register
    emitter.instruction("bl __rt_incref");                                      // retain it so the caller owns the returned result
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the owned Mixed result

    // -- empty or non-array receiver: PHP end() returns false --
    emitter.label("__rt_end_boxed_false");
    emitter.instruction("mov x0, #3");                                          // runtime tag 3 = bool
    emitter.instruction("mov x1, #0");                                          // bool false payload
    emitter.instruction("mov x2, #0");                                          // bool payloads do not use a high word
    emitter.instruction("bl __rt_mixed_from_value");                            // box false into a fresh Mixed cell
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the owned Mixed result
}

/// Emits `__rt_end_boxed` for x86_64.
///
/// Input: `rdi` = pointer to a boxed Mixed array.
/// Output: `rax` = owned Mixed cell holding the last element, or boxed `false` when the
/// receiver is null, empty, or not an array.
fn emit_end_boxed_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: end_boxed ---");
    emitter.label_global("__rt_end_boxed");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 16");                                         // reserve an aligned slot for the saved receiver
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the boxed Mixed receiver pointer

    emitter.instruction("test rdi, rdi");                                       // null receiver → boxed false
    emitter.instruction("je __rt_end_boxed_false");                             // branch to the false path on a null receiver
    emitter.instruction("mov r10, QWORD PTR [rdi]");                            // load the Mixed runtime tag
    emitter.instruction("cmp r10, 4");                                          // tag 4 = indexed array?
    emitter.instruction("je __rt_end_boxed_indexed");                           // read the last indexed element
    emitter.instruction("cmp r10, 5");                                          // tag 5 = associative array?
    emitter.instruction("je __rt_end_boxed_assoc");                             // read the insertion-order tail value
    emitter.instruction("jmp __rt_end_boxed_false");                            // any other payload → boxed false

    emitter.label("__rt_end_boxed_indexed");
    emitter.instruction("mov r10, QWORD PTR [rdi + 8]");                        // load the indexed-array payload pointer
    emitter.instruction("test r10, r10");                                       // defensive null guard for the payload
    emitter.instruction("je __rt_end_boxed_false");                             // branch to the false path on a null payload
    emitter.instruction("mov r11, QWORD PTR [r10]");                            // load the array length from the header
    emitter.instruction("sub r11, 1");                                          // compute the last index (length - 1)
    emitter.instruction("jl __rt_end_boxed_false");                             // an empty array yields boxed false
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the boxed Mixed receiver
    emitter.instruction("mov rsi, r11");                                        // pass the last index as the lookup key
    emitter.instruction("mov rdx, -1");                                         // key_hi = -1 marks an integer key
    emitter.instruction("call __rt_mixed_array_get");                           // read and box the last element as an owned Mixed
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the owned Mixed result

    emitter.label("__rt_end_boxed_assoc");
    emitter.instruction("mov r10, QWORD PTR [rdi + 8]");                        // load the hash payload pointer
    emitter.instruction("test r10, r10");                                       // defensive null guard for the hash
    emitter.instruction("je __rt_end_boxed_false");                             // branch to the false path on a null hash
    emitter.instruction("mov r11, QWORD PTR [r10 + 32]");                       // load the insertion-order tail slot index
    emitter.instruction("cmp r11, -1");                                         // is the hash empty (tail == -1)?
    emitter.instruction("je __rt_end_boxed_false");                             // empty hash yields boxed false
    emitter.instruction("shl r11, 6");                                          // scale the tail slot index by the 64-byte stride
    emitter.instruction("add r11, r10");                                        // advance from the hash base to the slot
    emitter.instruction("add r11, 40");                                         // skip the 40-byte hash header
    emitter.instruction("mov rcx, QWORD PTR [r11 + 24]");                       // load the tail value low word
    emitter.instruction("mov r8, QWORD PTR [r11 + 32]");                        // load the tail value high word
    emitter.instruction("mov r9, QWORD PTR [r11 + 40]");                        // load the tail value runtime tag
    emitter.instruction("cmp r9, 7");                                           // is the entry already a boxed Mixed cell?
    emitter.instruction("je __rt_end_boxed_assoc_boxed");                       // share the stored Mixed cell after a retain
    emitter.instruction("mov rax, r9");                                         // mixed_from_value tag argument
    emitter.instruction("mov rdi, rcx");                                        // mixed_from_value low payload argument
    emitter.instruction("mov rsi, r8");                                         // mixed_from_value high payload argument
    emitter.instruction("call __rt_mixed_from_value");                          // box the typed tail value into a fresh Mixed cell
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the owned Mixed result
    emitter.label("__rt_end_boxed_assoc_boxed");
    emitter.instruction("mov rax, rcx");                                        // move the stored Mixed cell into the return register
    abi::emit_push_reg(emitter, "rax");
    emitter.instruction("call __rt_incref");                                    // retain it so the caller owns the returned result
    abi::emit_pop_reg(emitter, "rax");
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the owned Mixed result

    emitter.label("__rt_end_boxed_false");
    emitter.instruction("mov rax, 3");                                          // runtime tag 3 = bool
    emitter.instruction("xor edi, edi");                                        // bool false payload
    emitter.instruction("xor esi, esi");                                        // bool payloads do not use a high word
    emitter.instruction("call __rt_mixed_from_value");                          // box false into a fresh Mixed cell
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the owned Mixed result
}
