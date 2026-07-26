//! Purpose:
//! Emits the `__rt_array_flip_mixed` runtime helper that flips an associative hash whose
//! keys and values are the gradual-typing `Mixed` representation (produced by
//! `__rt_mixed_to_owned_hash`). Lets `array_flip()` accept a boxed `Mixed`/union operand.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Iterates the source hash in insertion order (`__rt_hash_iter_next`); each entry's value
//!   becomes the new key and each entry's key becomes the new (boxed `Mixed`) value.
//! - PHP `array_flip()` only flips `int`/`string` values; any other value is skipped (PHP
//!   emits a warning and drops the entry). Boxed `Mixed` values (tag 7) are unboxed first.
//! - String keys are persisted by `__rt_hash_set`, and old keys boxed as values are persisted
//!   by `__rt_mixed_from_value`, so the returned hash owns every payload independently.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// array_flip_mixed: flip an owned `Mixed`-keyed/`Mixed`-valued hash into a fresh owned hash.
///
/// Input:  x0 (rdi) = source hash pointer (associative, `Mixed` keys/values, refcount owned)
/// Output: x0 (rax) = freshly owned hash `{value: key, ...}` with duplicate values last-wins
///
/// Only `int`/`string` source values become keys (matching PHP `array_flip()`); every other
/// value is skipped. The source hash is neither mutated nor released here (the caller owns it).
pub fn emit_array_flip_mixed(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_flip_mixed_linux_x86_64(emitter);
        return;
    }
    emit_array_flip_mixed_aarch64(emitter);
}

/// Emits `__rt_array_flip_mixed` for the AArch64 ABI.
fn emit_array_flip_mixed_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_flip_mixed ---");
    emitter.label_global("__rt_array_flip_mixed");

    // Stack layout (80 bytes):
    //   [sp,#0]  = source hash pointer
    //   [sp,#8]  = destination hash pointer (result)
    //   [sp,#16] = iterator cursor
    //   [sp,#24] = current source key low word (int value or string pointer)
    //   [sp,#32] = current source key high word (-1 marks an integer key)
    //   [sp,#40] = new key low word (from the source value)
    //   [sp,#48] = new key high word (-1 marks an integer key)
    //   [sp,#64] = saved x29 / [sp,#72] = saved x30
    emitter.instruction("sub sp, sp, #80");                                     // reserve the flip-mixed conversion frame
    emitter.instruction("stp x29, x30, [sp, #64]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #64");                                    // establish the local frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the source hash pointer

    // -- allocate the destination hash with mixed value slots --
    emitter.instruction("ldr x0, [x0]");                                        // load the source entry count for the result capacity
    emitter.instruction("lsl x0, x0, #1");                                      // double the count to give the destination hash headroom
    emitter.instruction("mov x9, #16");                                         // minimum destination bucket count
    emitter.instruction("cmp x0, x9");                                          // clamp the destination capacity to the minimum
    emitter.instruction("csel x0, x9, x0, lt");                                 // use 16 when doubled count is below the minimum
    emitter.instruction("mov x1, #7");                                          // value_type tag 7 = boxed mixed values
    emitter.instruction("bl __rt_hash_new");                                    // allocate the destination hash table
    emitter.instruction("str x0, [sp, #8]");                                    // save the destination hash pointer
    emitter.instruction("str xzr, [sp, #16]");                                  // initialize the iterator cursor to the head sentinel

    emitter.label("__rt_array_flip_mixed_loop");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the source hash pointer
    emitter.instruction("ldr x1, [sp, #16]");                                   // reload the iterator cursor
    emitter.instruction("bl __rt_hash_iter_next");                              // advance to the next source entry
    emitter.instruction("cmn x0, #1");                                          // did iteration reach the done sentinel (-1)?
    emitter.instruction("b.eq __rt_array_flip_mixed_done");                     // stop once every source entry has been flipped
    emitter.instruction("str x0, [sp, #16]");                                   // save the updated iterator cursor
    emitter.instruction("str x1, [sp, #24]");                                   // save the source key low word for the flipped value
    emitter.instruction("str x2, [sp, #32]");                                   // save the source key high word for the flipped value

    // -- unbox the source value when it is a boxed mixed cell --
    emitter.instruction("cmp x5, #7");                                          // is the source value a boxed mixed cell?
    emitter.instruction("b.ne __rt_array_flip_mixed_value_concrete");           // concrete values already carry their runtime tag
    emitter.instruction("mov x0, x3");                                          // move the boxed mixed pointer into the unbox input register
    emitter.instruction("bl __rt_mixed_unbox");                                 // unbox to the concrete tag/low/high triple
    emitter.instruction("mov x5, x0");                                          // adopt the unboxed runtime tag
    emitter.instruction("mov x3, x1");                                          // adopt the unboxed value low word
    emitter.instruction("mov x4, x2");                                          // adopt the unboxed value high word

    emitter.label("__rt_array_flip_mixed_value_concrete");
    emitter.instruction("cmp x5, #0");                                          // is the source value an integer?
    emitter.instruction("b.eq __rt_array_flip_mixed_newkey_int");               // integers flip into integer keys
    emitter.instruction("cmp x5, #1");                                          // is the source value a string?
    emitter.instruction("b.eq __rt_array_flip_mixed_newkey_str");               // strings flip into string keys
    emitter.instruction("b __rt_array_flip_mixed_advance");                     // PHP skips non int/string values

    emitter.label("__rt_array_flip_mixed_newkey_int");
    emitter.instruction("str x3, [sp, #40]");                                   // new key low word = integer value
    emitter.instruction("mov x9, #-1");                                         // integer key sentinel
    emitter.instruction("str x9, [sp, #48]");                                   // new key high word = -1 (integer key)
    emitter.instruction("b __rt_array_flip_mixed_box_old");                     // continue to box the old key as the value

    emitter.label("__rt_array_flip_mixed_newkey_str");
    emitter.instruction("str x3, [sp, #40]");                                   // new key low word = string pointer
    emitter.instruction("str x4, [sp, #48]");                                   // new key high word = string length

    // -- box the old source key into a mixed value cell --
    emitter.label("__rt_array_flip_mixed_box_old");
    emitter.instruction("ldr x1, [sp, #24]");                                   // reload the source key low word
    emitter.instruction("ldr x2, [sp, #32]");                                   // reload the source key high word
    emitter.instruction("cmn x2, #1");                                          // is the source key an integer key?
    emitter.instruction("b.eq __rt_array_flip_mixed_box_old_int");              // integers box without a high word
    emitter.instruction("mov x0, #1");                                          // string mixed tag
    emitter.instruction("bl __rt_mixed_from_value");                            // box (and persist) the string key as a mixed value
    emitter.instruction("b __rt_array_flip_mixed_insert");                      // proceed to the destination insert
    emitter.label("__rt_array_flip_mixed_box_old_int");
    emitter.instruction("mov x0, #0");                                          // integer mixed tag
    emitter.instruction("mov x2, #0");                                          // integer mixed payloads have no high word
    emitter.instruction("bl __rt_mixed_from_value");                            // box the integer key as a mixed value

    emitter.label("__rt_array_flip_mixed_insert");
    emitter.instruction("mov x3, x0");                                          // value low word = boxed mixed pointer
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the destination hash pointer
    emitter.instruction("ldr x1, [sp, #40]");                                   // reload the new key low word
    emitter.instruction("ldr x2, [sp, #48]");                                   // reload the new key high word
    emitter.instruction("mov x4, #0");                                          // value high word is unused for boxed mixed
    emitter.instruction("mov x5, #7");                                          // value tag 7 = boxed mixed value
    emitter.instruction("bl __rt_hash_set");                                    // insert the flipped key/value pair
    emitter.instruction("str x0, [sp, #8]");                                    // persist the possibly-grown destination hash pointer

    emitter.label("__rt_array_flip_mixed_advance");
    emitter.instruction("b __rt_array_flip_mixed_loop");                        // continue with the next source entry

    emitter.label("__rt_array_flip_mixed_done");
    emitter.instruction("ldr x0, [sp, #8]");                                    // return the destination hash pointer
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // release the conversion frame
    emitter.instruction("ret");                                                 // return the flipped hash in x0
}

/// Emits `__rt_array_flip_mixed` for the x86_64 Linux ABI.
fn emit_array_flip_mixed_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_flip_mixed ---");
    emitter.label_global("__rt_array_flip_mixed");

    // Frame (rbp-relative): [rbp-8]=src, [rbp-16]=dst, [rbp-24]=cursor,
    //   [rbp-32]=key_lo, [rbp-40]=key_hi, [rbp-48]=new_key_lo, [rbp-56]=new_key_hi.
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 80");                                         // reserve aligned flip-mixed spill slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the source hash pointer

    emitter.instruction("mov rdi, QWORD PTR [rdi]");                            // load the source entry count for the result capacity
    emitter.instruction("shl rdi, 1");                                          // double the count to give the destination hash headroom
    emitter.instruction("cmp rdi, 16");                                         // clamp the destination capacity to the minimum
    emitter.instruction("jge __rt_array_flip_mixed_cap_x86");                   // keep the doubled count when it meets the minimum
    emitter.instruction("mov rdi, 16");                                         // fall back to the minimum destination capacity
    emitter.label("__rt_array_flip_mixed_cap_x86");
    emitter.instruction("mov rsi, 7");                                          // value_type tag 7 = boxed mixed values
    emitter.instruction("call __rt_hash_new");                                  // allocate the destination hash table
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the destination hash pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                         // initialize the iterator cursor to the head sentinel

    emitter.label("__rt_array_flip_mixed_loop_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the source hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // reload the iterator cursor
    emitter.instruction("call __rt_hash_iter_next");                            // advance to the next source entry
    emitter.instruction("cmp rax, -1");                                         // did iteration reach the done sentinel?
    emitter.instruction("je __rt_array_flip_mixed_done_x86");                   // stop once every source entry has been flipped
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the updated iterator cursor
    emitter.instruction("mov QWORD PTR [rbp - 32], rdi");                       // save the source key low word for the flipped value
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // save the source key high word for the flipped value

    emitter.instruction("cmp r9, 7");                                           // is the source value a boxed mixed cell?
    emitter.instruction("jne __rt_array_flip_mixed_value_concrete_x86");        // concrete values already carry their runtime tag
    emitter.instruction("mov rax, rcx");                                        // move the boxed mixed pointer into the unbox input register
    emitter.instruction("call __rt_mixed_unbox");                              // unbox to the concrete tag/low/high triple
    emitter.instruction("mov r9, rax");                                         // adopt the unboxed runtime tag
    emitter.instruction("mov rcx, rdi");                                        // adopt the unboxed value low word
    emitter.instruction("mov r8, rdx");                                         // adopt the unboxed value high word

    emitter.label("__rt_array_flip_mixed_value_concrete_x86");
    emitter.instruction("cmp r9, 0");                                           // is the source value an integer?
    emitter.instruction("je __rt_array_flip_mixed_newkey_int_x86");             // integers flip into integer keys
    emitter.instruction("cmp r9, 1");                                           // is the source value a string?
    emitter.instruction("je __rt_array_flip_mixed_newkey_str_x86");             // strings flip into string keys
    emitter.instruction("jmp __rt_array_flip_mixed_advance_x86");               // PHP skips non int/string values

    emitter.label("__rt_array_flip_mixed_newkey_int_x86");
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // new key low word = integer value
    emitter.instruction("mov QWORD PTR [rbp - 56], -1");                        // new key high word = -1 (integer key)
    emitter.instruction("jmp __rt_array_flip_mixed_box_old_x86");               // continue to box the old key as the value

    emitter.label("__rt_array_flip_mixed_newkey_str_x86");
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // new key low word = string pointer
    emitter.instruction("mov QWORD PTR [rbp - 56], r8");                        // new key high word = string length

    emitter.label("__rt_array_flip_mixed_box_old_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // reload the source key high word
    emitter.instruction("cmp rax, -1");                                         // is the source key an integer key?
    emitter.instruction("je __rt_array_flip_mixed_box_old_int_x86");            // integers box without a high word
    emitter.instruction("mov rax, 1");                                          // string mixed tag
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // string key pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 40]");                       // string key length
    emitter.instruction("call __rt_mixed_from_value");                          // box (and persist) the string key as a mixed value
    emitter.instruction("jmp __rt_array_flip_mixed_insert_x86");                // proceed to the destination insert
    emitter.label("__rt_array_flip_mixed_box_old_int_x86");
    emitter.instruction("mov rax, 0");                                          // integer mixed tag
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // integer key value
    emitter.instruction("mov rsi, 0");                                          // integer mixed payloads have no high word
    emitter.instruction("call __rt_mixed_from_value");                          // box the integer key as a mixed value

    emitter.label("__rt_array_flip_mixed_insert_x86");
    emitter.instruction("mov rcx, rax");                                        // value low word = boxed mixed pointer
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the destination hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 48]");                       // reload the new key low word
    emitter.instruction("mov rdx, QWORD PTR [rbp - 56]");                       // reload the new key high word
    emitter.instruction("xor r8, r8");                                          // value high word is unused for boxed mixed
    emitter.instruction("mov r9, 7");                                           // value tag 7 = boxed mixed value
    emitter.instruction("call __rt_hash_set");                                  // insert the flipped key/value pair
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // persist the possibly-grown destination hash pointer

    emitter.label("__rt_array_flip_mixed_advance_x86");
    emitter.instruction("jmp __rt_array_flip_mixed_loop_x86");                  // continue with the next source entry

    emitter.label("__rt_array_flip_mixed_done_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return the destination hash pointer
    emitter.instruction("add rsp, 80");                                         // release the conversion frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the flipped hash in rax
}
