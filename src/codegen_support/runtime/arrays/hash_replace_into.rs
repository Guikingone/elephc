//! Purpose:
//! Emits the `__rt_hash_replace_into` runtime helper backing PHP `array_replace()` on
//! associative hashes: it overlays every entry of a source hash onto an owned destination
//! hash, overwriting matching keys last-wins while preserving insertion order.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - Invoked by the EIR `array_replace` lowering in
//!   `crate::codegen::lower_inst::builtins::arrays::lower_array_replace()`, after the base
//!   argument has been shallow-cloned into an owned destination.
//!
//! Key details:
//! - The destination is assumed to be uniquely owned (refcount 1, produced by
//!   `__rt_hash_clone_shallow`), so `__rt_hash_set` mutates it in place; the possibly-grown
//!   pointer is threaded back through each insertion.
//! - Value ownership mirrors `__rt_hash_union`: string payloads are re-persisted and
//!   refcounted payloads (tags 4-7) are retained for the destination before insertion;
//!   `__rt_hash_set` releases any value it overwrites, keeping refcounts balanced.
//! - Unlike `__rt_hash_union` (first-wins), this helper always inserts, so an existing key
//!   is overwritten by the source value (PHP array_replace last-wins semantics).

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits `__rt_hash_replace_into` for the host target.
///
/// Input:  x0 / rdi = owned destination hash pointer, x1 / rsi = source hash pointer.
/// Output: x0 / rax = destination hash pointer (may differ if `__rt_hash_set` grew it).
/// The source hash is only read (its values are retained, not moved); the destination is
/// mutated in place and returned.
pub fn emit_hash_replace_into(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_hash_replace_into_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: hash_replace_into ---");
    emitter.label_global("__rt_hash_replace_into");

    // -- set up stack frame (no clone: the destination is already owned) --
    emitter.instruction("sub sp, sp, #96");                                     // reserve spill slots for the replace walk
    emitter.instruction("stp x29, x30, [sp, #80]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #80");                                    // establish a stable frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save the source associative-array pointer
    emitter.instruction("str x0, [sp, #8]");                                    // save the owned destination associative-array pointer
    emitter.instruction("str xzr, [sp, #16]");                                  // initialize the insertion-order iterator cursor

    // -- walk source entries and overwrite every key into the destination --
    emitter.label("__rt_hash_replace_into_loop");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the source associative-array pointer
    emitter.instruction("ldr x1, [sp, #16]");                                   // reload the insertion-order iterator cursor
    emitter.instruction("bl __rt_hash_iter_next");                              // fetch the next source entry in insertion order
    emitter.instruction("cmn x0, #1");                                          // did the iterator report the terminal sentinel?
    emitter.instruction("b.eq __rt_hash_replace_into_done");                    // finish once every source entry has been overlaid
    emitter.instruction("str x0, [sp, #16]");                                   // save the next insertion-order iterator cursor
    emitter.instruction("str x1, [sp, #24]");                                   // save the borrowed source key pointer
    emitter.instruction("str x2, [sp, #32]");                                   // save the borrowed source key length
    emitter.instruction("str x3, [sp, #40]");                                   // save the borrowed source value low word
    emitter.instruction("str x4, [sp, #48]");                                   // save the borrowed source value high word
    emitter.instruction("str x5, [sp, #56]");                                   // save the source value runtime tag

    // -- make the source value owned by the destination before insertion --
    emitter.instruction("ldr x5, [sp, #56]");                                   // reload the source value runtime tag
    emitter.instruction("cmp x5, #1");                                          // is the source value a string payload?
    emitter.instruction("b.eq __rt_hash_replace_into_value_string");            // strings must be persisted for the destination owner
    emitter.instruction("cmp x5, #4");                                          // is the source value in the refcounted payload range?
    emitter.instruction("b.lo __rt_hash_replace_into_value_scalar");            // scalar payloads can be copied directly
    emitter.instruction("cmp x5, #7");                                          // is the source value still a supported refcounted payload?
    emitter.instruction("b.hi __rt_hash_replace_into_value_scalar");            // unknown high tags fall back to scalar copying
    emitter.instruction("ldr x0, [sp, #40]");                                   // load the borrowed refcounted source payload
    emitter.instruction("bl __rt_incref");                                      // retain the source payload for the destination
    emitter.instruction("ldr x3, [sp, #40]");                                   // reload the retained source value low word
    emitter.instruction("ldr x4, [sp, #48]");                                   // reload the source value high word
    emitter.instruction("ldr x5, [sp, #56]");                                   // reload the source value runtime tag
    emitter.instruction("b __rt_hash_replace_into_insert");                     // insert the retained payload

    emitter.label("__rt_hash_replace_into_value_string");
    emitter.instruction("ldr x1, [sp, #40]");                                   // load the borrowed source string pointer
    emitter.instruction("ldr x2, [sp, #48]");                                   // load the borrowed source string length
    emitter.instruction("bl __rt_str_persist");                                 // duplicate the string payload for the destination
    emitter.instruction("mov x3, x1");                                          // move the owned string pointer into the hash-set value low word
    emitter.instruction("mov x4, x2");                                          // move the owned string length into the hash-set value high word
    emitter.instruction("ldr x5, [sp, #56]");                                   // reload the string runtime tag
    emitter.instruction("b __rt_hash_replace_into_insert");                     // insert the owned string payload

    emitter.label("__rt_hash_replace_into_value_scalar");
    emitter.instruction("ldr x3, [sp, #40]");                                   // reload the scalar source value low word
    emitter.instruction("ldr x4, [sp, #48]");                                   // reload the scalar source value high word
    emitter.instruction("ldr x5, [sp, #56]");                                   // reload the scalar runtime tag

    // -- overwrite (or insert) the key/value pair into the destination --
    emitter.label("__rt_hash_replace_into_insert");
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the current destination associative-array pointer
    emitter.instruction("ldr x1, [sp, #24]");                                   // reload the source key pointer
    emitter.instruction("ldr x2, [sp, #32]");                                   // reload the source key length
    emitter.instruction("bl __rt_hash_set");                                    // last-wins insert releases any overwritten destination value
    emitter.instruction("str x0, [sp, #8]");                                    // save the possibly grown destination associative-array pointer
    emitter.instruction("b __rt_hash_replace_into_loop");                       // continue scanning source-side entries

    emitter.label("__rt_hash_replace_into_done");
    emitter.instruction("ldr x0, [sp, #8]");                                    // return the completed destination associative-array pointer
    emitter.instruction("ldp x29, x30, [sp, #80]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #96");                                     // release the replace spill slots
    emitter.instruction("ret");                                                 // return to generated code
}

/// Emits the x86_64 System V implementation of `__rt_hash_replace_into`.
///
/// Mirrors the ARM64 logic: rdi = owned destination hash, rsi = source hash; returns the
/// destination hash pointer in rax. Uses an rbp frame with spill slots (rbp is preserved),
/// matching the other hash walk helpers such as `__rt_hash_union`.
fn emit_hash_replace_into_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hash_replace_into ---");
    emitter.label_global("__rt_hash_replace_into");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving replace spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the replace walk
    emitter.instruction("sub rsp, 80");                                         // reserve local storage while keeping nested calls aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                        // save the source associative-array pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // save the owned destination associative-array pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                         // initialize the insertion-order iterator cursor

    emitter.label("__rt_hash_replace_into_x86_loop");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the source associative-array pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // reload the insertion-order iterator cursor
    emitter.instruction("call __rt_hash_iter_next");                            // fetch the next source entry in insertion order
    emitter.instruction("cmp rax, -1");                                         // did the iterator report the terminal sentinel?
    emitter.instruction("je __rt_hash_replace_into_x86_done");                  // finish once every source entry has been overlaid
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the next insertion-order iterator cursor
    emitter.instruction("mov QWORD PTR [rbp - 32], rdi");                       // save the borrowed source key pointer
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // save the borrowed source key length
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // save the borrowed source value low word
    emitter.instruction("mov QWORD PTR [rbp - 56], r8");                        // save the borrowed source value high word
    emitter.instruction("mov QWORD PTR [rbp - 64], r9");                        // save the source value runtime tag

    emitter.instruction("mov r10, QWORD PTR [rbp - 64]");                       // reload the source value runtime tag
    emitter.instruction("cmp r10, 1");                                          // is the source value a string payload?
    emitter.instruction("je __rt_hash_replace_into_x86_value_string");          // strings must be persisted for the destination owner
    emitter.instruction("cmp r10, 4");                                          // is the source value in the refcounted payload range?
    emitter.instruction("jb __rt_hash_replace_into_x86_value_scalar");          // scalar payloads can be copied directly
    emitter.instruction("cmp r10, 7");                                          // is the source value still a supported refcounted payload?
    emitter.instruction("ja __rt_hash_replace_into_x86_value_scalar");          // unknown high tags fall back to scalar copying
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // load the borrowed refcounted source payload
    emitter.instruction("call __rt_incref");                                    // retain the source payload for the destination
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the retained source value low word
    emitter.instruction("mov r8, QWORD PTR [rbp - 56]");                        // reload the source value high word
    emitter.instruction("mov r9, QWORD PTR [rbp - 64]");                        // reload the source value runtime tag
    emitter.instruction("jmp __rt_hash_replace_into_x86_insert");               // insert the retained payload

    emitter.label("__rt_hash_replace_into_x86_value_string");
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // load the borrowed source string pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 56]");                       // load the borrowed source string length
    emitter.instruction("call __rt_str_persist");                               // duplicate the string payload for the destination
    emitter.instruction("mov rcx, rax");                                        // move the owned string pointer into the hash-set value low word
    emitter.instruction("mov r8, rdx");                                         // move the owned string length into the hash-set value high word
    emitter.instruction("mov r9, QWORD PTR [rbp - 64]");                        // reload the string runtime tag
    emitter.instruction("jmp __rt_hash_replace_into_x86_insert");               // insert the owned string payload

    emitter.label("__rt_hash_replace_into_x86_value_scalar");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the scalar source value low word
    emitter.instruction("mov r8, QWORD PTR [rbp - 56]");                        // reload the scalar source value high word
    emitter.instruction("mov r9, QWORD PTR [rbp - 64]");                        // reload the scalar runtime tag

    emitter.label("__rt_hash_replace_into_x86_insert");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the current destination associative-array pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // reload the source key pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // reload the source key length
    emitter.instruction("call __rt_hash_set");                                  // last-wins insert releases any overwritten destination value
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the possibly grown destination associative-array pointer
    emitter.instruction("jmp __rt_hash_replace_into_x86_loop");                 // continue scanning source-side entries

    emitter.label("__rt_hash_replace_into_x86_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return the completed destination associative-array pointer
    emitter.instruction("add rsp, 80");                                         // release the replace spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to generated code
}
