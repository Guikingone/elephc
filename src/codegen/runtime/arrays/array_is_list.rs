//! Purpose:
//! Emits the `__rt_hash_is_list` and `__rt_mixed_array_is_list` runtime helpers backing
//! PHP `array_is_list()` for associative-hash and boxed-Mixed operands.
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::arrays`.
//! - Invoked by the EIR `array_is_list` lowering in
//!   `crate::codegen_ir::lower_inst::builtins::arrays::lower_array_is_list()`.
//!
//! Key details:
//! - A hash is a list iff its insertion-order keys are exactly the integers `0..count-1`.
//!   The walk follows the insertion-order `next` links, so key order (not bucket order) is
//!   what PHP observes; a string key or an out-of-sequence integer key fails immediately.
//! - Both helpers use the single-arg int-result ABI (arg + result in `x0` / `rax`), matching
//!   the boxed-Mixed container helpers such as `__rt_mixed_count`.

use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits `__rt_hash_is_list` and `__rt_mixed_array_is_list` for the host target.
///
/// `__rt_hash_is_list` takes an associative-hash pointer and returns `1` when the hash's
/// insertion-order keys are `0, 1, .., count-1` and `0` otherwise (an empty hash is a
/// list). `__rt_mixed_array_is_list` unboxes a Mixed cell: tag 4 (indexed array) is always
/// a list, tag 5 (associative hash) delegates to `__rt_hash_is_list`, and any other tag
/// (including null) is not a list.
pub fn emit_array_is_list(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_is_list_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: hash_is_list ---");
    emitter.label_global("__rt_hash_is_list");

    // x0 = associative-hash pointer. Output: x0 = 1 (list) or 0 (not a list).
    emitter.instruction("cbz x0, __rt_hash_is_list_true");                      // a null/absent hash counts as an empty list
    emitter.instruction("ldr x1, [x0, #24]");                                   // x1 = insertion-order head slot from the hash header
    emitter.instruction("mov x2, #0");                                          // x2 = expected next list index (0, 1, 2, ...)

    emitter.label("__rt_hash_is_list_loop");
    emitter.instruction("cmn x1, #1");                                          // has the insertion-order walk reached the -1 terminator?
    emitter.instruction("b.eq __rt_hash_is_list_true");                         // every visited key matched its index -> the hash is a list
    emitter.instruction("lsl x3, x1, #6");                                      // x3 = slot index * 64 (hash entry stride)
    emitter.instruction("add x3, x0, x3");                                      // advance from the hash base to the selected slot block
    emitter.instruction("add x3, x3, #40");                                     // skip the 40-byte hash header to the entry itself
    emitter.instruction("ldr x4, [x3, #16]");                                   // x4 = stored key length (-1 marks an integer key)
    emitter.instruction("cmn x4, #1");                                          // is the key an integer key (key_len == -1)?
    emitter.instruction("b.ne __rt_hash_is_list_false");                        // a string key can never form a 0..n-1 list
    emitter.instruction("ldr x5, [x3, #8]");                                    // x5 = integer key value (stored inline in key_ptr)
    emitter.instruction("cmp x5, x2");                                          // does the integer key equal the expected index?
    emitter.instruction("b.ne __rt_hash_is_list_false");                        // a gap or reordering breaks the list invariant
    emitter.instruction("add x2, x2, #1");                                      // advance the expected index for the next entry
    emitter.instruction("ldr x1, [x3, #56]");                                   // x1 = next insertion-order slot from the entry chain
    emitter.instruction("b __rt_hash_is_list_loop");                            // continue walking the insertion-order chain

    emitter.label("__rt_hash_is_list_true");
    emitter.instruction("mov x0, #1");                                          // return PHP true (keys are a dense 0-based sequence)
    emitter.instruction("ret");                                                 // return to the caller

    emitter.label("__rt_hash_is_list_false");
    emitter.instruction("mov x0, #0");                                          // return PHP false (not a list)
    emitter.instruction("ret");                                                 // return to the caller

    emitter.blank();
    emitter.comment("--- runtime: mixed_array_is_list ---");
    emitter.label_global("__rt_mixed_array_is_list");

    // x0 = boxed Mixed pointer. Output: x0 = 1 (list) or 0 (not a list).
    emitter.instruction("cbz x0, __rt_mixed_array_is_list_false");              // a null Mixed is not a list
    emitter.instruction("ldr x1, [x0]");                                        // x1 = runtime value tag from the Mixed cell
    emitter.instruction("cmp x1, #4");                                          // tag 4 = indexed array (always a dense list)?
    emitter.instruction("b.eq __rt_mixed_array_is_list_true");                  // packed arrays are lists by construction
    emitter.instruction("cmp x1, #5");                                          // tag 5 = associative hash?
    emitter.instruction("b.ne __rt_mixed_array_is_list_false");                 // any non-array payload is not a list
    emitter.instruction("ldr x0, [x0, #8]");                                    // x0 = boxed hash payload pointer
    emitter.instruction("b __rt_hash_is_list");                                 // tail-call the hash key-sequence scan

    emitter.label("__rt_mixed_array_is_list_true");
    emitter.instruction("mov x0, #1");                                          // return PHP true for a packed indexed array
    emitter.instruction("ret");                                                 // return to the caller

    emitter.label("__rt_mixed_array_is_list_false");
    emitter.instruction("mov x0, #0");                                          // return PHP false (null or non-array Mixed)
    emitter.instruction("ret");                                                 // return to the caller
}

/// Emits the x86_64 System V implementation of `__rt_hash_is_list` and
/// `__rt_mixed_array_is_list`.
///
/// Input/result use the single-arg int-result ABI: the operand and result both live in
/// `rax`. Only caller-saved registers (`rax`, `rcx`, `rdx`, `r8`, `r9`) are touched, so no
/// frame or callee-saved spill is needed.
fn emit_array_is_list_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hash_is_list ---");
    emitter.label_global("__rt_hash_is_list");

    // rax = associative-hash pointer. Output: rax = 1 (list) or 0 (not a list).
    emitter.instruction("test rax, rax");                                       // is the hash pointer null/absent?
    emitter.instruction("je __rt_hash_is_list_true_x");                         // a null/absent hash counts as an empty list
    emitter.instruction("mov rcx, QWORD PTR [rax + 24]");                       // rcx = insertion-order head slot from the hash header
    emitter.instruction("xor edx, edx");                                        // rdx = expected next list index (0, 1, 2, ...)

    emitter.label("__rt_hash_is_list_loop_x");
    emitter.instruction("cmp rcx, -1");                                         // has the insertion-order walk reached the -1 terminator?
    emitter.instruction("je __rt_hash_is_list_true_x");                         // every visited key matched its index -> the hash is a list
    emitter.instruction("mov r8, rcx");                                         // copy the slot index before scaling it into a byte offset
    emitter.instruction("shl r8, 6");                                           // r8 = slot index * 64 (hash entry stride)
    emitter.instruction("add r8, rax");                                         // advance from the hash base to the selected slot block
    emitter.instruction("add r8, 40");                                          // skip the 40-byte hash header to the entry itself
    emitter.instruction("mov r9, QWORD PTR [r8 + 16]");                         // r9 = stored key length (-1 marks an integer key)
    emitter.instruction("cmp r9, -1");                                          // is the key an integer key (key_len == -1)?
    emitter.instruction("jne __rt_hash_is_list_false_x");                       // a string key can never form a 0..n-1 list
    emitter.instruction("mov r9, QWORD PTR [r8 + 8]");                          // r9 = integer key value (stored inline in key_ptr)
    emitter.instruction("cmp r9, rdx");                                         // does the integer key equal the expected index?
    emitter.instruction("jne __rt_hash_is_list_false_x");                       // a gap or reordering breaks the list invariant
    emitter.instruction("add rdx, 1");                                          // advance the expected index for the next entry
    emitter.instruction("mov rcx, QWORD PTR [r8 + 56]");                        // rcx = next insertion-order slot from the entry chain
    emitter.instruction("jmp __rt_hash_is_list_loop_x");                        // continue walking the insertion-order chain

    emitter.label("__rt_hash_is_list_true_x");
    emitter.instruction("mov eax, 1");                                          // return PHP true (keys are a dense 0-based sequence)
    emitter.instruction("ret");                                                 // return to the caller

    emitter.label("__rt_hash_is_list_false_x");
    emitter.instruction("xor eax, eax");                                        // return PHP false (not a list)
    emitter.instruction("ret");                                                 // return to the caller

    emitter.blank();
    emitter.comment("--- runtime: mixed_array_is_list ---");
    emitter.label_global("__rt_mixed_array_is_list");

    // rax = boxed Mixed pointer. Output: rax = 1 (list) or 0 (not a list).
    emitter.instruction("test rax, rax");                                       // is the Mixed pointer null?
    emitter.instruction("je __rt_mixed_array_is_list_false_x");                 // a null Mixed is not a list
    emitter.instruction("mov rcx, QWORD PTR [rax]");                            // rcx = runtime value tag from the Mixed cell
    emitter.instruction("cmp rcx, 4");                                          // tag 4 = indexed array (always a dense list)?
    emitter.instruction("je __rt_mixed_array_is_list_true_x");                  // packed arrays are lists by construction
    emitter.instruction("cmp rcx, 5");                                          // tag 5 = associative hash?
    emitter.instruction("jne __rt_mixed_array_is_list_false_x");                // any non-array payload is not a list
    emitter.instruction("mov rax, QWORD PTR [rax + 8]");                        // rax = boxed hash payload pointer
    emitter.instruction("jmp __rt_hash_is_list");                               // tail-call the hash key-sequence scan

    emitter.label("__rt_mixed_array_is_list_true_x");
    emitter.instruction("mov eax, 1");                                          // return PHP true for a packed indexed array
    emitter.instruction("ret");                                                 // return to the caller

    emitter.label("__rt_mixed_array_is_list_false_x");
    emitter.instruction("xor eax, eax");                                        // return PHP false (null or non-array Mixed)
    emitter.instruction("ret");                                                 // return to the caller
}
