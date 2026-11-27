//! Purpose:
//! Emits `__rt_array_nested_mixed_fast_get`, the guarded fast path of a two-level read
//! `$a[$i][$k]` whose root is a local `array<mixed>` and whose keys are integers.
//!
//! Called from:
//! - EIR `RuntimeCall(NestedMixedArrayFastGet)` lowering, ahead of the ordinary read chain.
//!
//! Key details:
//! - Inputs: array pointer, outer integer key, inner integer key. Output: an owned Mixed leaf,
//!   or zero when any guard fails; the caller then runs the ordinary (warning-aware) chain.
//! - Only present, in-range elements of packed 8-byte-slot arrays are served, with an outer
//!   cell that holds an indexed array directly (no reference wrapper or marker). Nothing here
//!   warns, throws or runs PHP code, so the borrowed intermediate cell cannot be released
//!   while it is read: the ordinary chain had to clone that cell (a heap cell, a retain and a
//!   release with destructor dispatch) only to read one element out of it.
//! - The leaf is produced exactly like `__rt_mixed_array_get` produces it: a boxed slot's
//!   stored cell is retained and returned, a typed slot is boxed through `__rt_mixed_from_value`.
//! - AArch64 always reports a guard failure (zero), keeping the ordinary chain there.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the nested fast read for the active target.
pub fn emit_nested_mixed_fast_get(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_nested_mixed_fast_get ---");
    emitter.label_global("__rt_array_nested_mixed_fast_get");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("mov x0, #0");                                  // no AArch64 fast path: the caller runs the ordinary chain
            emitter.instruction("ret");                                         // report the guard failure
        }
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// Emits one packed-array guard: storage kind 2, the requested slot tag class and 8-byte slots.
fn emit_packed_guard_x86_64(emitter: &mut Emitter, array: &str) {
    crate::codegen_support::sentinels::emit_branch_if_null_container(
        emitter, array, "r11", "__rt_array_nested_mixed_fast_get_fail",
    );
    emitter.instruction(&format!("mov r9, QWORD PTR [{array} - 8]"));            // load the array kind metadata
    emitter.instruction("cmp r9b, 2");                                          // kind 2 = indexed packed array
    emitter.instruction("jne __rt_array_nested_mixed_fast_get_fail");           // hash storage keeps the ordinary chain
    emitter.instruction(&format!("cmp QWORD PTR [{array} + 16], 8"));            // one-word slots only (strings use two)
    emitter.instruction("jne __rt_array_nested_mixed_fast_get_fail");           // wide slots keep the ordinary chain
    emitter.instruction("shr r9, 8");                                           // move the element value tag into the low bits
    emitter.instruction("and r9d, 0x7f");                                       // ignore the persistent COW flag
}

/// Emits the x86_64 implementation (rdi = array, rsi = outer key, rdx = inner key).
fn emit_x86_64(emitter: &mut Emitter) {
    emit_packed_guard_x86_64(emitter, "rdi");
    emitter.instruction("cmp r9d, 7");                                          // the root must hold boxed Mixed slots
    emitter.instruction("jne __rt_array_nested_mixed_fast_get_fail");           // typed roots keep the ordinary chain
    emitter.instruction("cmp rsi, QWORD PTR [rdi]");                            // unsigned compare: negative or past-the-end keys fail
    emitter.instruction("jae __rt_array_nested_mixed_fast_get_fail");           // missing outer elements warn on the ordinary chain
    emitter.instruction("mov rax, QWORD PTR [rdi + 24 + rsi * 8]");             // borrow the stored outer cell
    emitter.instruction("test rax, rax");                                       // null gap slots read as PHP null
    emitter.instruction("jz __rt_array_nested_mixed_fast_get_fail");            // let the ordinary chain warn about them
    emitter.instruction("cmp QWORD PTR [rax], 4");                              // does the cell hold an indexed array directly?
    emitter.instruction("jne __rt_array_nested_mixed_fast_get_fail");           // references, markers, hashes and scalars keep the chain
    emitter.instruction("mov r10, QWORD PTR [rax + 8]");                        // borrow the inner array
    emit_packed_guard_x86_64(emitter, "r10");
    emitter.instruction("cmp rdx, QWORD PTR [r10]");                            // unsigned compare: negative or past-the-end keys fail
    emitter.instruction("jae __rt_array_nested_mixed_fast_get_fail");           // missing inner elements warn on the ordinary chain
    emitter.instruction("cmp r9d, 7");                                          // are the inner slots boxed Mixed cells?
    emitter.instruction("je __rt_array_nested_mixed_fast_get_boxed");           // clone the stored leaf cell
    emitter.instruction("cmp r9d, 1");                                          // string slots are two words wide
    emitter.instruction("je __rt_array_nested_mixed_fast_get_fail");            // (already excluded by the slot size; defensive)
    emitter.instruction("cmp r9d, 8");                                          // null slots carry no payload
    emitter.instruction("je __rt_array_nested_mixed_fast_get_fail");            // the ordinary chain boxes PHP null
    emitter.instruction("mov rax, r9");                                         // value tag of the typed inner slot
    emitter.instruction("mov rdi, QWORD PTR [r10 + 24 + rdx * 8]");             // scalar or pointer payload of the typed slot
    emitter.instruction("xor esi, esi");                                        // one-word payloads have no high word
    emitter.instruction("jmp __rt_mixed_from_value");                           // tail-call: box the element exactly like the ordinary read
    emitter.label("__rt_array_nested_mixed_fast_get_boxed");
    emitter.instruction("mov rax, QWORD PTR [r10 + 24 + rdx * 8]");             // borrow the stored leaf cell
    emitter.instruction("test rax, rax");                                       // null gap slots read as PHP null
    emitter.instruction("jz __rt_array_nested_mixed_fast_get_fail");            // let the ordinary chain handle them
    emitter.instruction("push rax");                                            // keep the stored leaf across the retain (aligns the call)
    emitter.instruction("call __rt_incref");                                    // retain the stored cell, exactly like the ordinary read
    emitter.instruction("pop rax");                                             // return the retained leaf cell
    emitter.instruction("ret");                                                 // the caller owns one reference
    emitter.label("__rt_array_nested_mixed_fast_get_fail");
    emitter.instruction("xor eax, eax");                                        // zero = guard failure
    emitter.instruction("ret");                                                 // the caller runs the ordinary chain
}
