//! Purpose:
//! Emits `__rt_sprintf_pack_mixed`, which turns one boxed Mixed cell into the 16-byte tagged
//! record `__rt_sprintf` consumes. Shared by the `sprintf()`/`printf()` argument packer and by
//! `__rt_vsprintf`'s per-element loop.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via
//!   `crate::codegen_support::runtime::strings`.
//! - `crate::codegen::lower_inst::builtins::strings::printf` for a Mixed operand whose
//!   conversion category is not known at compile time.
//!
//! Key details:
//! - Record tags match `__rt_sprintf`: int = 0, string = 1 | (len << 8), float = 2,
//!   bool = 3, and deferred boxed `Mixed` = 7.
//!   `__rt_sprintf` COERCES a record whose tag disagrees with the conversion character, so
//!   packing the cell's real runtime type is enough — the caller does not need to know whether
//!   the format asks for `%d`, `%s` or `%f`.
//! - Boxed PHP null (cell tag 8) becomes a ZERO-LENGTH STRING record, not an integer. That is
//!   what makes `%s` render `""` and `%d` render `0`, matching PHP on both: `__rt_sprintf`
//!   already guards a null string pointer on all three conversion paths ("treat a null string
//!   pointer as empty" for `%s`, "a null pointer parses as zero" for the int and float paths).
//!   Packing null as an integer instead would print the cell's raw low word — which is the
//!   null SENTINEL, and is exactly how `vsprintf("%s", [null])` used to answer
//!   `9223372036854775806`.
//! - Array (4/5), object (6), resource (9), and callable (10) cells preserve the BOX pointer
//!   in a tag-7 record. `__rt_sprintf` defers their conversion until it knows whether the
//!   runtime format asks for a string, integer, or float; native handles and heap pointers
//!   are therefore never formatted as numbers.
//! - `__rt_mixed_unbox` is used here so nested tag-7 wrappers still collapse to scalar records
//!   when possible. The helper owns a small ABI-aligned frame for that call on both targets.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits `__rt_sprintf_pack_mixed(boxed_cell) -> (payload, tag)`.
///
/// Inputs: `x0` = boxed Mixed pointer, possibly null (AArch64); `rdi` (x86_64).
/// Outputs: `x0`/`rax` = record payload word, `x1`/`rdx` = record tag/metadata word.
pub fn emit_sprintf_pack_mixed(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_sprintf_pack_mixed_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: sprintf_pack_mixed ---");
    emitter.label_global("__rt_sprintf_pack_mixed");

    emitter.instruction("sub sp, sp, #32");                                     // reserve an aligned frame for nested Mixed unboxing
    emitter.instruction("stp x29, x30, [sp, #16]");                             // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #16");                                    // establish this helper's frame pointer
    emitter.instruction("str x0, [sp, #8]");                                    // preserve the original box for deferred non-scalar records
    emitter.instruction("cbz x0, __rt_spm_null");                               // a null cell pointer packs as PHP null
    emitter.instruction("bl __rt_mixed_unbox");                                 // peel nested tag-7 wrappers to a concrete runtime value
    emitter.instruction("mov x9, x0");                                          // concrete cell runtime tag
    emitter.instruction("mov x10, x1");                                         // concrete cell low payload word
    emitter.instruction("mov x11, x2");                                         // concrete cell high payload word
    emitter.instruction("cmp x9, #0");                                          // integer cell?
    emitter.instruction("b.eq __rt_spm_int");                                   // build an integer record
    emitter.instruction("cmp x9, #1");                                          // string cell?
    emitter.instruction("b.eq __rt_spm_str");                                   // build a string record
    emitter.instruction("cmp x9, #2");                                          // float cell?
    emitter.instruction("b.eq __rt_spm_float");                                 // build a float record
    emitter.instruction("cmp x9, #3");                                          // bool cell?
    emitter.instruction("b.eq __rt_spm_bool");                                  // build a bool record
    emitter.instruction("cmp x9, #8");                                          // canonical boxed PHP null?
    emitter.instruction("b.eq __rt_spm_null");                                  // → empty-string record, never the raw sentinel
    emitter.instruction("ldr x0, [sp, #8]");                                    // preserve the boxed value for conversion-specific coercion
    emitter.instruction("mov x1, #7");                                          // tag 7 = deferred boxed Mixed operand
    emitter.instruction("b __rt_spm_done");                                     // return the box pointer, never its raw payload

    emitter.label("__rt_spm_int");
    emitter.instruction("mov x0, x10");                                         // payload = the concrete integer value
    emitter.instruction("mov x1, #0");                                          // tag 0 = integer operand
    emitter.instruction("b __rt_spm_done");                                     // return payload/tag

    emitter.label("__rt_spm_str");
    emitter.instruction("mov x1, x11");                                         // concrete high word = string byte length
    emitter.instruction("mov x0, x10");                                         // payload = the string pointer
    emitter.instruction("lsl x1, x1, #8");                                      // pack the length above the tag byte
    emitter.instruction("orr x1, x1, #1");                                      // tag 1 = string operand
    emitter.instruction("b __rt_spm_done");                                     // return payload/tag

    emitter.label("__rt_spm_float");
    emitter.instruction("mov x0, x10");                                         // payload = the double's bit pattern
    emitter.instruction("mov x1, #2");                                          // tag 2 = float operand
    emitter.instruction("b __rt_spm_done");                                     // return payload/tag

    emitter.label("__rt_spm_bool");
    emitter.instruction("mov x0, x10");                                         // payload = the boolean value
    emitter.instruction("mov x1, #3");                                          // tag 3 = bool operand
    emitter.instruction("b __rt_spm_done");                                     // return payload/tag

    emitter.label("__rt_spm_null");
    emitter.instruction("mov x0, #0");                                          // null string pointer, which every conversion guards
    emitter.instruction("mov x1, #1");                                          // (0 << 8) | 1 = a zero-length string record
    emitter.label("__rt_spm_done");
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore the caller frame and return address
    emitter.instruction("add sp, sp, #32");                                     // release this helper's aligned frame
    emitter.instruction("ret");                                                 // return payload/tag
}

/// Emits the Linux x86_64 string runtime helper for sprintf_pack_mixed.
fn emit_sprintf_pack_mixed_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: sprintf_pack_mixed ---");
    emitter.label_global("__rt_sprintf_pack_mixed");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame
    emitter.instruction("sub rsp, 16");                                         // keep the nested unbox call 16-byte aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the original box for deferred non-scalar records
    emitter.instruction("test rdi, rdi");                                       // a null cell pointer packs as PHP null
    emitter.instruction("jz __rt_spm_null");                                    // → empty-string record
    emitter.instruction("mov rax, rdi");                                        // __rt_mixed_unbox reads its boxed pointer from rax
    emitter.instruction("call __rt_mixed_unbox");                               // peel nested tag-7 wrappers to a concrete runtime value
    emitter.instruction("mov r9, rax");                                         // concrete cell runtime tag
    emitter.instruction("mov r10, rdi");                                        // concrete cell low payload word
    emitter.instruction("mov r11, rdx");                                        // concrete cell high payload word
    emitter.instruction("cmp r9, 0");                                           // integer cell?
    emitter.instruction("je __rt_spm_int");                                     // build an integer record
    emitter.instruction("cmp r9, 1");                                           // string cell?
    emitter.instruction("je __rt_spm_str");                                     // build a string record
    emitter.instruction("cmp r9, 2");                                           // float cell?
    emitter.instruction("je __rt_spm_float");                                   // build a float record
    emitter.instruction("cmp r9, 3");                                           // bool cell?
    emitter.instruction("je __rt_spm_bool");                                    // build a bool record
    emitter.instruction("cmp r9, 8");                                           // canonical boxed PHP null?
    emitter.instruction("je __rt_spm_null");                                    // → empty-string record, never the raw sentinel
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // preserve the boxed value for conversion-specific coercion
    emitter.instruction("mov rdx, 7");                                          // tag 7 = deferred boxed Mixed operand
    emitter.instruction("jmp __rt_spm_done");                                   // return the box pointer, never its raw payload

    emitter.label("__rt_spm_int");
    emitter.instruction("mov rax, r10");                                        // payload = the concrete integer value
    emitter.instruction("mov rdx, 0");                                          // tag 0 = integer operand
    emitter.instruction("jmp __rt_spm_done");                                   // return payload/tag

    emitter.label("__rt_spm_str");
    emitter.instruction("mov rdx, r11");                                        // concrete high word = string byte length
    emitter.instruction("mov rax, r10");                                        // payload = the string pointer
    emitter.instruction("shl rdx, 8");                                          // pack the length above the tag byte
    emitter.instruction("or rdx, 1");                                           // tag 1 = string operand
    emitter.instruction("jmp __rt_spm_done");                                   // return payload/tag

    emitter.label("__rt_spm_float");
    emitter.instruction("mov rax, r10");                                        // payload = the double's bit pattern
    emitter.instruction("mov rdx, 2");                                          // tag 2 = float operand
    emitter.instruction("jmp __rt_spm_done");                                   // return payload/tag

    emitter.label("__rt_spm_bool");
    emitter.instruction("mov rax, r10");                                        // payload = the boolean value
    emitter.instruction("mov rdx, 3");                                          // tag 3 = bool operand
    emitter.instruction("jmp __rt_spm_done");                                   // return payload/tag

    emitter.label("__rt_spm_null");
    emitter.instruction("mov rax, 0");                                          // null string pointer, which every conversion guards
    emitter.instruction("mov rdx, 1");                                          // (0 << 8) | 1 = a zero-length string record
    emitter.label("__rt_spm_done");
    emitter.instruction("leave");                                               // release the helper frame and restore rbp
    emitter.instruction("ret");                                                 // return payload/tag
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::{Platform, Target};

    /// Emits the Mixed packer for one target and returns its assembly text.
    fn emit_for(target: Target) -> String {
        let mut emitter = Emitter::new(target);
        emit_sprintf_pack_mixed(&mut emitter);
        emitter.output()
    }

    /// Pins both halves of the record protocol: scalar ints keep tag zero, while
    /// non-scalars preserve the original box pointer under deferred tag seven.
    #[test]
    fn test_sprintf_pack_mixed_distinguishes_ints_from_deferred_non_scalars_on_both_arches() {
        let arm = emit_for(Target::new(Platform::MacOS, Arch::AArch64));
        assert!(arm.contains("cmp x9, #0"), "{arm}");
        assert!(arm.contains("__rt_spm_int:"), "{arm}");
        assert!(arm.contains("mov x1, #7"), "{arm}");
        assert!(arm.contains("ldr x0, [sp, #8]"), "{arm}");

        let x64 = emit_for(Target::new(Platform::Linux, Arch::X86_64));
        assert!(x64.contains("cmp r9, 0"), "{x64}");
        assert!(x64.contains("__rt_spm_int:"), "{x64}");
        assert!(x64.contains("mov rdx, 7"), "{x64}");
        assert!(x64.contains("mov rax, QWORD PTR [rbp - 8]"), "{x64}");
    }
}
