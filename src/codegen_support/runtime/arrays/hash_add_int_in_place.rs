//! Purpose:
//! Emits `__rt_hash_add_int_in_place`, the guarded in-place fast path for
//! `$hash[$key] = $hash[$key] + <int literal>` on a local `array<..., mixed>`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters` during managed runtime emission.
//!
//! Key details:
//! - Input (x86_64): rdi = hash, rsi/rdx = normalized key (`__rt_hash_get` ABI), rcx = addend.
//!   Output: rax = 1 when the element was updated, 0 when nothing was touched and the caller must
//!   run the ordinary read-add-write statement instead.
//! - The update is applied only when it cannot be told apart from the ordinary statement: no hash
//!   write guard is active, the hash has one logical owner (the copy-on-write rule
//!   `__rt_hash_ensure_unique` applies), the key exists (so no undefined-key warning is due), the
//!   entry holds an integer either inline or in a boxed `Mixed` cell owned only by that entry,
//!   and the sum does not overflow (PHP would produce a float). References, strings, floats and
//!   every other value fall back.
//! - `__rt_hash_add_int_in_place_boxed` takes a frame slot's Mixed cell instead, requires the
//!   cell itself to be owned only by that slot and to hold a hash, then applies the same guards.
//! - AArch64 always answers 0, keeping the ordinary statement there.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::runtime::arrays::hash_layout::PINS_OFFSET;

/// Emits the in-place integer add helper for the selected target.
pub fn emit_hash_add_int_in_place(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hash_add_int_in_place ---");
    if emitter.target.arch == Arch::AArch64 {
        emitter.label_global("__rt_hash_add_int_in_place_boxed");
        emitter.label_global("__rt_hash_add_int_in_place");
        emitter.instruction("mov x0, #0");                                      // no fast path on this target: run the ordinary statement
        emitter.instruction("ret");                                             // return the fallback answer
        return;
    }
    // -- boxed slot: the frame's Mixed cell must own the only reference to a hash --
    emitter.label_global("__rt_hash_add_int_in_place_boxed");
    emitter.instruction("test rdi, rdi");                                       // an empty slot holds no hash
    emitter.instruction("jz __rt_hash_add_int_unboxed_fallback_x");             // the ordinary statement handles it
    abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("lea r11, [r10 + 16]");                                 // first payload address inside the heap arena
    emitter.instruction("cmp rdi, r11");                                        // does the cell lie below the heap arena?
    emitter.instruction("jb __rt_hash_add_int_unboxed_fallback_x");             // static cells are never mutated in place
    abi::emit_symbol_address(emitter, "r11", "_heap_off");
    emitter.instruction("add r10, QWORD PTR [r11]");                            // live end of the heap arena
    emitter.instruction("cmp rdi, r10");                                        // does the cell lie past the live heap?
    emitter.instruction("jae __rt_hash_add_int_unboxed_fallback_x");            // static cells are never mutated in place
    emitter.instruction("cmp DWORD PTR [rdi - 12], 1");                         // is the cell owned only by this slot?
    emitter.instruction("jne __rt_hash_add_int_unboxed_fallback_x");            // a shared cell shares its hash too
    emitter.instruction("cmp QWORD PTR [rdi], 5");                              // does the cell hold an associative array?
    emitter.instruction("jne __rt_hash_add_int_unboxed_fallback_x");            // any other payload takes the ordinary statement
    emitter.instruction("mov rdi, QWORD PTR [rdi + 8]");                        // continue with the boxed hash itself
    emitter.instruction("jmp __rt_hash_add_int_in_place");                      // apply the shared hash guards
    emitter.label("__rt_hash_add_int_unboxed_fallback_x");
    emitter.instruction("xor eax, eax");                                        // nothing was touched: run the ordinary statement
    emitter.instruction("ret");                                                 // return the fallback answer
    emitter.label_global("__rt_hash_add_int_in_place");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame
    emitter.instruction("sub rsp, 16");                                         // reserve the addend spill slot, keeping calls aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rcx");                        // preserve the addend across the lookup
    abi::emit_load_symbol_to_reg(emitter, "r10", "_hash_write_guard_top", 0);
    emitter.instruction("test r10, r10");                                       // is an entry owner release in progress?
    emitter.instruction("jnz __rt_hash_add_int_fallback_x");                    // guarded writes keep the ordinary set path
    emitter.instruction("test rdi, rdi");                                       // null containers have no element to update
    emitter.instruction("jz __rt_hash_add_int_fallback_x");                     // the ordinary statement diagnoses and autovivifies
    abi::emit_load_int_immediate(emitter, "r10", crate::codegen_support::sentinels::NULL_SENTINEL);
    emitter.instruction("cmp rdi, r10");                                        // does the local carry the in-band null-container sentinel?
    emitter.instruction("je __rt_hash_add_int_fallback_x");                     // sentinel containers take the ordinary statement
    emitter.instruction("mov r10d, DWORD PTR [rdi - 12]");                      // load the hash refcount from the uniform heap header
    emitter.instruction(&format!("sub r10, QWORD PTR [rdi + {PINS_OFFSET}]"));  // exclude lifetime pins, as copy-on-write does
    emitter.instruction("cmp r10, 1");                                          // does the hash have more than one logical owner?
    emitter.instruction("ja __rt_hash_add_int_fallback_x");                     // shared hashes must be separated by the ordinary write
    emitter.instruction("call __rt_hash_get");                                  // rax = found, rdi/rsi = payload, rcx = tag, r8 = entry address
    emitter.instruction("test rax, rax");                                       // does the key exist?
    emitter.instruction("jz __rt_hash_add_int_fallback_x");                     // a missing key warns and autovivifies on the ordinary path
    emitter.instruction("test rcx, rcx");                                       // is the entry an inline integer?
    emitter.instruction("jz __rt_hash_add_int_inline_x");                       // add straight into the entry payload
    emitter.instruction("cmp rcx, 7");                                          // is the entry a boxed Mixed cell?
    emitter.instruction("jne __rt_hash_add_int_fallback_x");                    // references and typed non-integers take the ordinary path
    abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("lea r11, [r10 + 16]");                                 // first payload address inside the heap arena
    emitter.instruction("cmp rdi, r11");                                        // does the cell lie below the heap arena?
    emitter.instruction("jb __rt_hash_add_int_fallback_x");                     // static cells are never mutated in place
    abi::emit_symbol_address(emitter, "r11", "_heap_off");
    emitter.instruction("add r10, QWORD PTR [r11]");                            // live end of the heap arena
    emitter.instruction("cmp rdi, r10");                                        // does the cell lie past the live heap?
    emitter.instruction("jae __rt_hash_add_int_fallback_x");                    // static cells are never mutated in place
    emitter.instruction("cmp DWORD PTR [rdi - 12], 1");                        // is the cell owned only by this entry?
    emitter.instruction("jne __rt_hash_add_int_fallback_x");                    // a shared cell must not observe the update
    emitter.instruction("cmp QWORD PTR [rdi], 0");                              // does the cell hold an integer?
    emitter.instruction("jne __rt_hash_add_int_fallback_x");                    // floats, strings and containers keep PHP's generic add
    emitter.instruction("mov rax, QWORD PTR [rdi + 8]");                        // load the boxed integer
    emitter.instruction("add rax, QWORD PTR [rbp - 8]");                        // add the literal addend
    emitter.instruction("jo __rt_hash_add_int_fallback_x");                     // an overflowing sum becomes a float on the ordinary path
    emitter.instruction("mov QWORD PTR [rdi + 8], rax");                        // store the sum into the uniquely owned cell
    emitter.instruction("jmp __rt_hash_add_int_done_x");                        // report the completed update
    emitter.label("__rt_hash_add_int_inline_x");
    emitter.instruction("mov rax, QWORD PTR [r8 + 24]");                        // load the inline integer payload
    emitter.instruction("add rax, QWORD PTR [rbp - 8]");                        // add the literal addend
    emitter.instruction("jo __rt_hash_add_int_fallback_x");                     // an overflowing sum becomes a float on the ordinary path
    emitter.instruction("mov QWORD PTR [r8 + 24], rax");                        // store the sum into the entry payload
    emitter.label("__rt_hash_add_int_done_x");
    emitter.instruction("mov eax, 1");                                          // the element was updated in place
    emitter.instruction("mov rsp, rbp");                                        // release the helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the completed answer
    emitter.label("__rt_hash_add_int_fallback_x");
    emitter.instruction("xor eax, eax");                                        // nothing was touched: run the ordinary statement
    emitter.instruction("mov rsp, rbp");                                        // release the helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the fallback answer
}
