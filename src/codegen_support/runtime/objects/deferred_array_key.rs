//! Purpose:
//! Boxes deferred original offsets and raises type-named errors for actual PHP arrays.
//!
//! Called from:
//! - Mixed array getters/setters after receiver dispatch, emitted by the managed runtime.
//!
//! Key details:
//! - The key pair borrows its payload; only the recovered boxed offset acquires an owner.
//! - The SPL converter consumes that box before returning a persistent diagnostic name row.
//! - Existing Throwable construction owns the message and performs the normal unwind.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};
use crate::codegen_support::deferred_array_keys::{ARRAY_OFFSET_TYPE_SUFFIX, DEFERRED_KEY_TAG_BIAS};

/// Emits original-offset boxing and the array-only illegal-offset thrower.
pub(crate) fn emit_deferred_array_key_runtime(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// Uses the runtime string/value registers, preserving the borrowed key until it is boxed.
fn emit_aarch64(emitter: &mut Emitter) {
    emitter.label_global("__rt_box_deferred_array_key");
    emitter.instruction("cmn x2, #1");                                          // minus one is the ordinary integer-key sentinel
    emitter.instruction("b.eq __rt_deferred_key_box_int");                      // box integer offsets without changing their payload
    emitter.instruction("b.ge __rt_deferred_key_box_string");                   // nonnegative lengths describe string offsets
    emitter.instruction("neg x0, x2");                                          // recover the original runtime tag from the deferred domain
    abi::emit_load_int_immediate(emitter, "x9", DEFERRED_KEY_TAG_BIAS);
    emitter.instruction("sub x0, x0, x9");                                      // remove the encoding bias
    emitter.instruction("mov x2, #0");                                          // object, array and Closure payloads carry no high word here
    emitter.instruction("b __rt_mixed_from_value");                             // return an independently owned original-offset box
    emitter.label("__rt_deferred_key_box_int");
    emitter.instruction("mov x0, #0");                                          // runtime tag zero denotes an integer
    emitter.instruction("mov x2, #0");                                          // integer values have no high payload
    emitter.instruction("b __rt_mixed_from_value");                             // return an independently owned integer-offset box
    emitter.label("__rt_deferred_key_box_string");
    emitter.instruction("mov x0, #1");                                          // runtime tag one denotes a string pointer and byte length
    emitter.instruction("b __rt_mixed_from_value");                             // preserve the original string bytes in an owned box

    emitter.label_global("__rt_throw_deferred_array_key");
    emitter.instruction("stp x29, x30, [sp, #-16]!");                           // preserve linkage while resolving the diagnostic name
    emitter.instruction("mov x29, sp");                                         // establish the short diagnostic frame
    abi::emit_call_label(emitter, "__rt_box_deferred_array_key");
    emitter.instruction("mov x1, #0");                                          // fixed-array conversion rejects each deferred non-scalar type
    abi::emit_call_label(emitter, "__rt_spl_offset_convert");
    emitter.instruction("mov x2, x1");                                          // converter consumed the box and returned its persistent name row
    abi::emit_symbol_address(emitter, "x0", "_spl_fixed_offset_type_prefix");
    abi::emit_load_int_immediate(emitter, "x1", "Cannot access offset of type ".len() as i64);
    abi::emit_symbol_address(emitter, "x3", "_php_array_offset_type_suffix");
    abi::emit_load_int_immediate(emitter, "x4", ARRAY_OFFSET_TYPE_SUFFIX.len() as i64);
    emitter.instruction("ldp x29, x30, [sp], #16");                             // restore linkage before the shared non-returning throw
    emitter.instruction("b __rt_spl_throw_offset_type");                        // raise TypeError with the original key type and array context
}

/// Uses SysV key inputs and the internal boxed-value result convention explicitly.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.label_global("__rt_box_deferred_array_key");
    emitter.instruction("mov rdi, rsi");                                        // preserve the borrowed key payload in the boxing ABI register
    emitter.instruction("cmp rdx, -1");                                         // minus one is the ordinary integer-key sentinel
    emitter.instruction("je __rt_deferred_key_box_int");                        // box integer offsets without changing their payload
    emitter.instruction("jge __rt_deferred_key_box_string");                    // nonnegative lengths describe string offsets
    emitter.instruction("mov rax, rdx");                                        // recover the original runtime tag from the deferred domain
    emitter.instruction("neg rax");                                             // begin undoing the negative high-word encoding
    abi::emit_load_int_immediate(emitter, "r10", DEFERRED_KEY_TAG_BIAS);
    emitter.instruction("sub rax, r10");                                        // remove the encoding bias
    emitter.instruction("xor esi, esi");                                        // object, array and Closure payloads carry no high word here
    emitter.instruction("jmp __rt_mixed_from_value");                           // return an independently owned original-offset box
    emitter.label("__rt_deferred_key_box_int");
    emitter.instruction("xor eax, eax");                                        // runtime tag zero denotes an integer
    emitter.instruction("xor esi, esi");                                        // integer values have no high payload
    emitter.instruction("jmp __rt_mixed_from_value");                           // return an independently owned integer-offset box
    emitter.label("__rt_deferred_key_box_string");
    emitter.instruction("mov eax, 1");                                          // runtime tag one denotes a string pointer and byte length
    emitter.instruction("mov rsi, rdx");                                        // pass the original string byte length in the boxing ABI
    emitter.instruction("jmp __rt_mixed_from_value");                           // preserve the original string bytes in an owned box

    emitter.label_global("__rt_throw_deferred_array_key");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer and align nested calls
    emitter.instruction("mov rbp, rsp");                                        // establish the short diagnostic frame
    abi::emit_call_label(emitter, "__rt_box_deferred_array_key");
    emitter.instruction("mov rdi, rax");                                        // pass the owned offset box in the converter ABI
    emitter.instruction("xor esi, esi");                                        // fixed-array conversion rejects every deferred non-scalar type
    abi::emit_call_label(emitter, "__rt_spl_offset_convert");
    emitter.instruction("mov rdx, rdi");                                        // converter consumed the box and returned its persistent name row
    abi::emit_symbol_address(emitter, "rdi", "_spl_fixed_offset_type_prefix");
    abi::emit_load_int_immediate(emitter, "rsi", "Cannot access offset of type ".len() as i64);
    abi::emit_symbol_address(emitter, "rcx", "_php_array_offset_type_suffix");
    abi::emit_load_int_immediate(emitter, "r8", ARRAY_OFFSET_TYPE_SUFFIX.len() as i64);
    emitter.instruction("pop rbp");                                             // restore linkage before the shared non-returning throw
    emitter.instruction("jmp __rt_spl_throw_offset_type");                      // raise TypeError with the original key type and array context
}
