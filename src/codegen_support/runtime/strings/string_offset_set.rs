//! Purpose:
//! Emits the `__rt_string_offset_set` runtime helper assembly for PHP string offset
//! assignment (`$s[$i] = $c`). Keeps PHP byte-string pointer/length behavior and
//! target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - The helper NEVER mutates the source string in place: it copies the source bytes
//!   into the shared `_concat_buf` scratch, so aliases (`$b = $a; $a[0] = 'X';`) stay
//!   copy-on-write safe. The surrounding lowering persists the fresh scratch result
//!   into the destination local and releases the previous owner, matching `$s = <new>`.
//! - PHP semantics: the replacement's FIRST byte is written at `$i`; offsets past the
//!   end right-pad with spaces (0x20); negative offsets index from the end; an
//!   out-of-range negative offset and an empty replacement are no-ops that copy the
//!   source through unchanged.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits the `__rt_string_offset_set` runtime helper.
///
/// Input:  x1=str_ptr, x2=str_len, x3=offset (signed), x4=char_ptr, x5=char_len
/// Output: x1=result_ptr (into `_concat_buf`), x2=result_len
///
/// Dispatches to `emit_string_offset_set_linux_x86_64` on x86_64; uses the ARM64 path otherwise.
pub fn emit_string_offset_set(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_string_offset_set_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: string_offset_set ---");
    emitter.label_global("__rt_string_offset_set");

    // -- an empty replacement is a no-op: copy the source through unchanged --
    emitter.instruction("cbz x5, __rt_sos_copy_unchanged");                     // empty replacement string leaves the subject unchanged (PHP no-op)

    // -- normalize a negative offset to length-relative, rejecting out-of-range ones --
    emitter.instruction("cmp x3, #0");                                          // is the requested byte offset negative?
    emitter.instruction("b.ge __rt_sos_offset_nonneg");                         // keep non-negative offsets unchanged
    emitter.instruction("add x3, x3, x2");                                      // negative offsets count back from the string end
    emitter.instruction("cmp x3, #0");                                          // did the adjusted offset land before the start?
    emitter.instruction("b.lt __rt_sos_copy_unchanged");                        // out-of-range negative offsets are a no-op (PHP warning)
    emitter.label("__rt_sos_offset_nonneg");

    // -- new_len = max(str_len, offset + 1) --
    emitter.instruction("add x6, x3, #1");                                      // one past the written offset
    emitter.instruction("cmp x6, x2");                                          // does the write extend beyond the current length?
    emitter.instruction("csel x7, x6, x2, gt");                                 // x7 = new_len = max(offset + 1, str_len)

    // -- dest = _concat_buf + _concat_off --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x8", "_concat_off");
    emitter.instruction("ldr x8, [x8]");                                        // load the current concat scratch write offset
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_concat_buf");
    emitter.instruction("add x10, x9, x8");                                     // x10 = destination start = buf + offset
    emitter.instruction("mov x11, x10");                                        // x11 = destination write cursor

    // -- copy the source bytes into the scratch destination --
    emitter.instruction("mov x12, x1");                                         // x12 = source cursor
    emitter.instruction("mov x13, x2");                                         // x13 = remaining source bytes
    emitter.label("__rt_sos_copy_src");
    emitter.instruction("cbz x13, __rt_sos_pad_check");                         // all source bytes copied
    emitter.instruction("ldrb w14, [x12], #1");                                 // load one source byte and advance
    emitter.instruction("strb w14, [x11], #1");                                 // store it into the scratch and advance
    emitter.instruction("sub x13, x13, #1");                                    // decrement the remaining source bytes
    emitter.instruction("b __rt_sos_copy_src");                                 // continue copying the source

    emitter.label("__rt_sos_pad_check");
    emitter.instruction("cmp x3, x2");                                          // does the write fall within the existing bytes?
    emitter.instruction("b.lt __rt_sos_set_inplace");                           // an in-range offset overwrites an existing byte

    // -- offset >= str_len: right-pad with spaces up to the offset, then write the byte --
    emitter.instruction("sub x13, x3, x2");                                     // x13 = number of padding spaces to insert
    emitter.instruction("mov w14, #0x20");                                      // ASCII space fills the gap before the offset
    emitter.label("__rt_sos_pad_loop");
    emitter.instruction("cbz x13, __rt_sos_set_after_pad");                     // padding complete
    emitter.instruction("strb w14, [x11], #1");                                 // write one padding space and advance the cursor
    emitter.instruction("sub x13, x13, #1");                                    // decrement the remaining padding count
    emitter.instruction("b __rt_sos_pad_loop");                                 // continue padding with spaces
    emitter.label("__rt_sos_set_after_pad");
    emitter.instruction("ldrb w14, [x4]");                                      // load the replacement string's first byte
    emitter.instruction("strb w14, [x11]");                                     // write it at the destination offset
    emitter.instruction("b __rt_sos_finish");                                   // publish the scratch result

    emitter.label("__rt_sos_set_inplace");
    emitter.instruction("add x12, x10, x3");                                    // x12 = destination start + offset
    emitter.instruction("ldrb w14, [x4]");                                      // load the replacement string's first byte
    emitter.instruction("strb w14, [x12]");                                     // overwrite the existing byte at the offset

    emitter.label("__rt_sos_finish");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x6", "_concat_off");
    emitter.instruction("ldr x8, [x6]");                                        // reload the concat scratch write offset
    emitter.instruction("add x8, x8, x7");                                      // advance it by the new string length
    emitter.instruction("str x8, [x6]");                                        // store the updated concat scratch offset
    emitter.instruction("mov x1, x10");                                         // return the scratch result pointer
    emitter.instruction("mov x2, x7");                                          // return the new string length
    emitter.instruction("ret");                                                 // return the mutated copy to the caller

    // -- no-op path: copy the source through unchanged into fresh scratch --
    emitter.label("__rt_sos_copy_unchanged");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x8", "_concat_off");
    emitter.instruction("ldr x8, [x8]");                                        // load the current concat scratch write offset
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_concat_buf");
    emitter.instruction("add x10, x9, x8");                                     // x10 = destination start = buf + offset
    emitter.instruction("mov x11, x10");                                        // x11 = destination write cursor
    emitter.instruction("mov x12, x1");                                         // x12 = source cursor
    emitter.instruction("mov x13, x2");                                         // x13 = remaining source bytes
    emitter.label("__rt_sos_cu_loop");
    emitter.instruction("cbz x13, __rt_sos_cu_done");                           // all source bytes copied unchanged
    emitter.instruction("ldrb w14, [x12], #1");                                 // load one source byte and advance
    emitter.instruction("strb w14, [x11], #1");                                 // store it into the scratch and advance
    emitter.instruction("sub x13, x13, #1");                                    // decrement the remaining source bytes
    emitter.instruction("b __rt_sos_cu_loop");                                  // continue the unchanged copy
    emitter.label("__rt_sos_cu_done");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x6", "_concat_off");
    emitter.instruction("ldr x8, [x6]");                                        // reload the concat scratch write offset
    emitter.instruction("add x8, x8, x2");                                      // advance it by the unchanged source length
    emitter.instruction("str x8, [x6]");                                        // store the updated concat scratch offset
    emitter.instruction("mov x1, x10");                                         // return the scratch result pointer
    emitter.instruction("ret");                                                 // x2 already holds the unchanged length
}

/// Emits the x86_64 Linux variant of `__rt_string_offset_set`.
///
/// Input:  rax=str_ptr, rdx=str_len, rdi=offset (signed), rsi=char_ptr, rcx=char_len
/// Output: rax=result_ptr (into `_concat_buf`), rdx=result_len
fn emit_string_offset_set_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: string_offset_set ---");
    emitter.label_global("__rt_string_offset_set");

    // -- an empty replacement is a no-op: copy the source through unchanged --
    emitter.instruction("test rcx, rcx");                                       // is the replacement string empty?
    emitter.instruction("jz __rt_sos_copy_unchanged");                          // empty replacement leaves the subject unchanged (PHP no-op)

    // -- normalize a negative offset to length-relative, rejecting out-of-range ones --
    emitter.instruction("test rdi, rdi");                                       // is the requested byte offset negative?
    emitter.instruction("jns __rt_sos_offset_nonneg");                          // keep non-negative offsets unchanged
    emitter.instruction("add rdi, rdx");                                        // negative offsets count back from the string end
    emitter.instruction("test rdi, rdi");                                       // did the adjusted offset land before the start?
    emitter.instruction("js __rt_sos_copy_unchanged");                          // out-of-range negative offsets are a no-op (PHP warning)
    emitter.label("__rt_sos_offset_nonneg");

    // -- new_len = max(str_len, offset + 1) --
    emitter.instruction("lea r8, [rdi + 1]");                                   // one past the written offset
    emitter.instruction("cmp r8, rdx");                                         // does the write extend beyond the current length?
    emitter.instruction("jg __rt_sos_have_newlen");                             // keep offset + 1 when it exceeds the current length
    emitter.instruction("mov r8, rdx");                                         // otherwise the new length equals the current length
    emitter.label("__rt_sos_have_newlen");

    // -- dest = _concat_buf + _concat_off --
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_concat_off");
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // load the current concat scratch write offset
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_concat_buf");
    emitter.instruction("add r10, r11");                                        // r10 = destination start = buf + offset
    emitter.instruction("mov r11, r10");                                        // r11 = destination write cursor

    // -- copy the source bytes into the scratch destination (rcx = remaining) --
    emitter.instruction("mov rcx, rdx");                                        // rcx = remaining source bytes
    emitter.label("__rt_sos_copy_src");
    emitter.instruction("test rcx, rcx");                                       // all source bytes copied?
    emitter.instruction("jz __rt_sos_pad_check");                              // move on to padding / byte write
    emitter.instruction("mov r9b, BYTE PTR [rax]");                             // load one source byte
    emitter.instruction("mov BYTE PTR [r11], r9b");                             // store it into the scratch destination
    emitter.instruction("add rax, 1");                                          // advance the source cursor
    emitter.instruction("add r11, 1");                                          // advance the destination cursor
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining source bytes
    emitter.instruction("jmp __rt_sos_copy_src");                               // continue copying the source

    emitter.label("__rt_sos_pad_check");
    emitter.instruction("cmp rdi, rdx");                                        // does the write fall within the existing bytes?
    emitter.instruction("jl __rt_sos_set_inplace");                             // an in-range offset overwrites an existing byte

    // -- offset >= str_len: right-pad with spaces up to the offset, then write the byte --
    emitter.instruction("mov rcx, rdi");                                        // rcx = offset
    emitter.instruction("sub rcx, rdx");                                        // rcx = number of padding spaces to insert
    emitter.label("__rt_sos_pad_loop");
    emitter.instruction("test rcx, rcx");                                       // padding complete?
    emitter.instruction("jz __rt_sos_set_after_pad");                           // stop once every gap byte is a space
    emitter.instruction("mov BYTE PTR [r11], 0x20");                            // write one ASCII space
    emitter.instruction("add r11, 1");                                          // advance the destination cursor
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining padding count
    emitter.instruction("jmp __rt_sos_pad_loop");                              // continue padding with spaces
    emitter.label("__rt_sos_set_after_pad");
    emitter.instruction("mov r9b, BYTE PTR [rsi]");                             // load the replacement string's first byte
    emitter.instruction("mov BYTE PTR [r11], r9b");                             // write it at the destination offset
    emitter.instruction("jmp __rt_sos_finish");                                 // publish the scratch result

    emitter.label("__rt_sos_set_inplace");
    emitter.instruction("lea rcx, [r10 + rdi]");                                // rcx = destination start + offset
    emitter.instruction("mov r9b, BYTE PTR [rsi]");                             // load the replacement string's first byte
    emitter.instruction("mov BYTE PTR [rcx], r9b");                             // overwrite the existing byte at the offset

    emitter.label("__rt_sos_finish");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_concat_off");
    emitter.instruction("mov rcx, QWORD PTR [r9]");                             // reload the concat scratch write offset
    emitter.instruction("add rcx, r8");                                         // advance it by the new string length
    emitter.instruction("mov QWORD PTR [r9], rcx");                             // store the updated concat scratch offset
    emitter.instruction("mov rax, r10");                                        // return the scratch result pointer
    emitter.instruction("mov rdx, r8");                                         // return the new string length
    emitter.instruction("ret");                                                 // return the mutated copy to the caller

    // -- no-op path: copy the source through unchanged into fresh scratch --
    emitter.label("__rt_sos_copy_unchanged");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_concat_off");
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // load the current concat scratch write offset
    crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_concat_buf");
    emitter.instruction("add r10, r11");                                        // r10 = destination start = buf + offset
    emitter.instruction("mov r11, r10");                                        // r11 = destination write cursor
    emitter.instruction("mov rcx, rdx");                                        // rcx = remaining source bytes
    emitter.label("__rt_sos_cu_loop");
    emitter.instruction("test rcx, rcx");                                       // all source bytes copied unchanged?
    emitter.instruction("jz __rt_sos_cu_done");                                 // finish once the source is fully copied
    emitter.instruction("mov r9b, BYTE PTR [rax]");                             // load one source byte
    emitter.instruction("mov BYTE PTR [r11], r9b");                             // store it into the scratch destination
    emitter.instruction("add rax, 1");                                          // advance the source cursor
    emitter.instruction("add r11, 1");                                          // advance the destination cursor
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining source bytes
    emitter.instruction("jmp __rt_sos_cu_loop");                                // continue the unchanged copy
    emitter.label("__rt_sos_cu_done");
    crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_concat_off");
    emitter.instruction("mov rcx, QWORD PTR [r9]");                             // reload the concat scratch write offset
    emitter.instruction("add rcx, rdx");                                        // advance it by the unchanged source length
    emitter.instruction("mov QWORD PTR [r9], rcx");                             // store the updated concat scratch offset
    emitter.instruction("mov rax, r10");                                        // return the scratch result pointer
    emitter.instruction("ret");                                                 // rdx already holds the unchanged length
}

#[cfg(test)]
mod tests {
    use crate::codegen_support::platform::{Arch, Platform, Target};

    use super::*;

    /// Verifies the ARM64 helper reads the source, pads, and returns a scratch pointer/length pair.
    #[test]
    fn test_emit_string_offset_set_arm64_copies_and_pads() {
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emit_string_offset_set(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_string_offset_set:\n"));
        assert!(asm.contains("cbz x5, __rt_sos_copy_unchanged\n"));
        assert!(asm.contains("mov w14, #0x20\n"));
        assert!(asm.contains("ldrb w14, [x4]\n"));
        assert!(asm.contains("mov x1, x10\n"));
    }

    /// Verifies the x86_64 helper reads the source, pads, and returns rax/rdx per the SYSV ABI.
    #[test]
    fn test_emit_string_offset_set_linux_x86_64_copies_and_pads() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_string_offset_set(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_string_offset_set:\n"));
        assert!(asm.contains("jz __rt_sos_copy_unchanged\n"));
        assert!(asm.contains("mov BYTE PTR [r11], 0x20\n"));
        assert!(asm.contains("mov r9b, BYTE PTR [rsi]\n"));
        assert!(asm.contains("mov rax, r10\n"));
    }
}
