//! Purpose:
//! Emits the `__rt_str_bitwise` runtime helper assembly for PHP's bytewise string
//! operators (`&`/`|`/`^` when both operands are strings). Keeps binary-safe,
//! length-driven byte loops and the target-specific ABI variants in one emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - Result length is `min(la, lb)` for AND/XOR and `max(la, lb)` for OR (which then
//!   copies the longer operand's tail verbatim). The result is written into the shared
//!   `_concat_buf` scratch storage, exactly like `__rt_concat`, and returned as a
//!   PHP pointer/length pair. Operand buffers are only read, never written.

use crate::codegen::{emit::Emitter, platform::Arch};

/// Emits the `__rt_str_bitwise` runtime helper for PHP bytewise string operators.
///
/// The operator is selected by the `mode` argument (0 = AND, 1 = OR, 2 = XOR),
/// matching `crate::ir::StrBitKind::as_mode`. The helper resolves the result length
/// and per-byte operation once at entry, then writes the result into `_concat_buf`
/// and advances `_concat_off`, mirroring `__rt_concat`. Dispatches to the x86_64
/// variant on x86_64; uses the AArch64 path otherwise.
///
/// Input:  left_ptr, left_len, right_ptr, right_len, mode
/// Output: result_ptr, result_len (in the same registers `__rt_concat` returns)
///  - AArch64: x1=left_ptr, x2=left_len, x3=right_ptr, x4=right_len, x5=mode -> x1/x2
///  - x86_64:  rax=left_ptr, rdx=left_len, rdi=right_ptr, rsi=right_len, rcx=mode -> rax/rdx
pub fn emit_str_bitwise(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_str_bitwise_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: str_bitwise ---");
    emitter.label_global("__rt_str_bitwise");

    // -- resolve result length, per-byte op, and destination once at entry --
    emitter.instruction("cmp x2, x4");                                          // compare left length against right length
    emitter.instruction("csel x6, x2, x4, ls");                                 // x6 = min(left_len, right_len) = overlap length
    emitter.instruction("csel x7, x4, x2, ls");                                 // x7 = max(left_len, right_len) = OR result length
    emitter.instruction("csel x12, x1, x3, hs");                                // x12 = longer operand pointer (left when left_len >= right_len)
    emitter.instruction("cmp x5, #1");                                          // mode 1 selects OR, which keeps the longer operand's tail
    emitter.instruction("csel x10, x7, x6, eq");                                // x10 = result length (OR uses max, AND/XOR use min)
    crate::codegen::abi::emit_symbol_address(emitter, "x15", "_concat_off");
    emitter.instruction("ldr x8, [x15]");                                       // load the current concat-buffer write offset
    crate::codegen::abi::emit_symbol_address(emitter, "x9", "_concat_buf");
    emitter.instruction("add x9, x9, x8");                                      // x9 = result pointer = concat buffer base + offset
    emitter.instruction("mov x8, x9");                                          // x8 = destination write cursor
    emitter.instruction("mov x11, x6");                                         // x11 = overlap byte counter (min length)
    emitter.instruction("cmp x5, #1");                                          // re-test the mode: 1 = OR
    emitter.instruction("b.eq __rt_str_bitwise_or");                            // branch to the OR overlap loop
    emitter.instruction("cmp x5, #2");                                          // test the mode: 2 = XOR
    emitter.instruction("b.eq __rt_str_bitwise_xor");                           // branch to the XOR overlap loop

    // -- AND (mode 0): out[i] = left[i] & right[i] over the overlap --
    emitter.label("__rt_str_bitwise_and");
    emitter.instruction("cbz x11, __rt_str_bitwise_finish");                    // no overlap bytes left -> finish
    emitter.instruction("ldrb w13, [x1], #1");                                  // load next byte from the left operand, advance
    emitter.instruction("ldrb w14, [x3], #1");                                  // load next byte from the right operand, advance
    emitter.instruction("and w13, w13, w14");                                   // bytewise AND of the two operand bytes
    emitter.instruction("strb w13, [x8], #1");                                  // store the result byte, advance the destination
    emitter.instruction("sub x11, x11, #1");                                    // one fewer overlap byte remaining
    emitter.instruction("b __rt_str_bitwise_and");                              // continue the AND overlap loop

    // -- OR (mode 1): out[i] = left[i] | right[i], then copy the longer tail --
    emitter.label("__rt_str_bitwise_or");
    emitter.instruction("cbz x11, __rt_str_bitwise_or_tail");                   // overlap done -> copy the longer tail
    emitter.instruction("ldrb w13, [x1], #1");                                  // load next byte from the left operand, advance
    emitter.instruction("ldrb w14, [x3], #1");                                  // load next byte from the right operand, advance
    emitter.instruction("orr w13, w13, w14");                                   // bytewise OR of the two operand bytes
    emitter.instruction("strb w13, [x8], #1");                                  // store the result byte, advance the destination
    emitter.instruction("sub x11, x11, #1");                                    // one fewer overlap byte remaining
    emitter.instruction("b __rt_str_bitwise_or");                               // continue the OR overlap loop
    emitter.label("__rt_str_bitwise_or_tail");
    emitter.instruction("add x12, x12, x6");                                    // skip the longer operand past the overlapped prefix
    emitter.instruction("sub x11, x7, x6");                                     // tail length = max - min bytes
    emitter.label("__rt_str_bitwise_or_tail_loop");
    emitter.instruction("cbz x11, __rt_str_bitwise_finish");                    // tail fully copied -> finish
    emitter.instruction("ldrb w13, [x12], #1");                                 // load next tail byte from the longer operand
    emitter.instruction("strb w13, [x8], #1");                                  // copy the tail byte verbatim into the result
    emitter.instruction("sub x11, x11, #1");                                    // one fewer tail byte remaining
    emitter.instruction("b __rt_str_bitwise_or_tail_loop");                     // continue copying the tail

    // -- XOR (mode 2): out[i] = left[i] ^ right[i] over the overlap --
    emitter.label("__rt_str_bitwise_xor");
    emitter.instruction("cbz x11, __rt_str_bitwise_finish");                    // no overlap bytes left -> finish
    emitter.instruction("ldrb w13, [x1], #1");                                  // load next byte from the left operand, advance
    emitter.instruction("ldrb w14, [x3], #1");                                  // load next byte from the right operand, advance
    emitter.instruction("eor w13, w13, w14");                                   // bytewise XOR of the two operand bytes
    emitter.instruction("strb w13, [x8], #1");                                  // store the result byte, advance the destination
    emitter.instruction("sub x11, x11, #1");                                    // one fewer overlap byte remaining
    emitter.instruction("b __rt_str_bitwise_xor");                              // continue the XOR overlap loop

    // -- publish the result length into the concat buffer and return --
    emitter.label("__rt_str_bitwise_finish");
    crate::codegen::abi::emit_symbol_address(emitter, "x15", "_concat_off");
    emitter.instruction("ldr x13, [x15]");                                      // reload the concat-buffer write offset
    emitter.instruction("add x13, x13, x10");                                   // advance the offset by the result length
    emitter.instruction("str x13, [x15]");                                      // publish the updated concat-buffer offset
    emitter.instruction("mov x1, x9");                                          // return the result pointer
    emitter.instruction("mov x2, x10");                                         // return the result length
    emitter.instruction("ret");                                                 // return the bytewise string result in x1/x2
}

/// Emits the x86_64 Linux variant of `__rt_str_bitwise`.
///
/// Uses the System V AMD64 ABI: left string in rax/rdx, right string in rdi/rsi,
/// mode in rcx. Result returned in rax (pointer) and rdx (length). Shares the
/// `_concat_buf` / `_concat_off` scratch machinery with the AArch64 path.
fn emit_str_bitwise_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: str_bitwise ---");
    emitter.label_global("__rt_str_bitwise");

    // -- spill inputs and resolve min/max/longer/result-length once --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for scratch locals
    emitter.instruction("sub rsp, 64");                                         // reserve local slots for lengths, pointers, and metadata
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the left operand pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // save the right operand pointer
    emitter.instruction("mov QWORD PTR [rbp - 64], rcx");                       // save the operation mode
    emitter.instruction("mov r8, rdx");                                         // r8 = left length
    emitter.instruction("mov r9, rsi");                                         // r9 = right length
    emitter.instruction("cmp r8, r9");                                          // compare left length against right length
    emitter.instruction("mov r10, r8");                                         // tentative min = left length
    emitter.instruction("cmova r10, r9");                                       // min = right length when left length is larger
    emitter.instruction("mov QWORD PTR [rbp - 24], r10");                       // save the overlap (min) length
    emitter.instruction("mov r10, r9");                                         // tentative max = right length
    emitter.instruction("cmova r10, r8");                                       // max = left length when left length is larger
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");                       // save the OR result (max) length
    emitter.instruction("mov r10, rax");                                        // tentative longer pointer = left operand
    emitter.instruction("cmovb r10, rdi");                                      // longer pointer = right operand when left is shorter
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // save the longer-operand pointer
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the min length
    emitter.instruction("mov r11, QWORD PTR [rbp - 32]");                       // reload the max length
    emitter.instruction("cmp rcx, 1");                                          // mode 1 selects OR (max-length result)
    emitter.instruction("cmove r10, r11");                                      // result length = max for OR, else min
    emitter.instruction("mov QWORD PTR [rbp - 48], r10");                       // save the result length
    crate::codegen::abi::emit_symbol_address(emitter, "r8", "_concat_off");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // load the current concat-buffer write offset
    crate::codegen::abi::emit_symbol_address(emitter, "r10", "_concat_buf");
    emitter.instruction("lea r11, [r10 + r9]");                                 // r11 = result pointer = concat buffer base + offset
    emitter.instruction("mov QWORD PTR [rbp - 56], r11");                       // save the result pointer
    emitter.instruction("mov rcx, QWORD PTR [rbp - 64]");                       // reload the operation mode
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // r8 = left read cursor
    emitter.instruction("mov r9, QWORD PTR [rbp - 16]");                        // r9 = right read cursor
    emitter.instruction("mov r10, QWORD PTR [rbp - 56]");                       // r10 = destination write cursor
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                       // r11 = overlap byte counter (min length)
    emitter.instruction("cmp rcx, 1");                                          // mode 1 = OR
    emitter.instruction("je __rt_str_bitwise_or");                              // branch to the OR overlap loop
    emitter.instruction("cmp rcx, 2");                                          // mode 2 = XOR
    emitter.instruction("je __rt_str_bitwise_xor");                             // branch to the XOR overlap loop

    // -- AND (mode 0): out[i] = left[i] & right[i] over the overlap --
    emitter.label("__rt_str_bitwise_and");
    emitter.instruction("test r11, r11");                                       // check whether overlap bytes remain
    emitter.instruction("je __rt_str_bitwise_finish");                          // no overlap bytes left -> finish
    emitter.instruction("mov al, BYTE PTR [r8]");                               // load next byte from the left operand
    emitter.instruction("and al, BYTE PTR [r9]");                               // bytewise AND with the right operand byte
    emitter.instruction("mov BYTE PTR [r10], al");                              // store the result byte
    emitter.instruction("inc r8");                                              // advance the left cursor
    emitter.instruction("inc r9");                                              // advance the right cursor
    emitter.instruction("inc r10");                                             // advance the destination cursor
    emitter.instruction("dec r11");                                             // one fewer overlap byte remaining
    emitter.instruction("jmp __rt_str_bitwise_and");                            // continue the AND overlap loop

    // -- OR (mode 1): out[i] = left[i] | right[i], then copy the longer tail --
    emitter.label("__rt_str_bitwise_or");
    emitter.instruction("test r11, r11");                                       // check whether overlap bytes remain
    emitter.instruction("je __rt_str_bitwise_or_tail");                         // overlap done -> copy the longer tail
    emitter.instruction("mov al, BYTE PTR [r8]");                               // load next byte from the left operand
    emitter.instruction("or al, BYTE PTR [r9]");                                // bytewise OR with the right operand byte
    emitter.instruction("mov BYTE PTR [r10], al");                              // store the result byte
    emitter.instruction("inc r8");                                              // advance the left cursor
    emitter.instruction("inc r9");                                              // advance the right cursor
    emitter.instruction("inc r10");                                             // advance the destination cursor
    emitter.instruction("dec r11");                                             // one fewer overlap byte remaining
    emitter.instruction("jmp __rt_str_bitwise_or");                             // continue the OR overlap loop
    emitter.label("__rt_str_bitwise_or_tail");
    emitter.instruction("mov r8, QWORD PTR [rbp - 40]");                        // r8 = longer-operand pointer
    emitter.instruction("add r8, QWORD PTR [rbp - 24]");                        // skip past the overlapped prefix -> tail source
    emitter.instruction("mov r11, QWORD PTR [rbp - 32]");                       // r11 = max length
    emitter.instruction("sub r11, QWORD PTR [rbp - 24]");                       // tail length = max - min bytes
    emitter.label("__rt_str_bitwise_or_tail_loop");
    emitter.instruction("test r11, r11");                                       // check whether tail bytes remain
    emitter.instruction("je __rt_str_bitwise_finish");                          // tail fully copied -> finish
    emitter.instruction("mov al, BYTE PTR [r8]");                               // load next tail byte from the longer operand
    emitter.instruction("mov BYTE PTR [r10], al");                              // copy the tail byte verbatim into the result
    emitter.instruction("inc r8");                                              // advance the tail source cursor
    emitter.instruction("inc r10");                                             // advance the destination cursor
    emitter.instruction("dec r11");                                             // one fewer tail byte remaining
    emitter.instruction("jmp __rt_str_bitwise_or_tail_loop");                   // continue copying the tail

    // -- XOR (mode 2): out[i] = left[i] ^ right[i] over the overlap --
    emitter.label("__rt_str_bitwise_xor");
    emitter.instruction("test r11, r11");                                       // check whether overlap bytes remain
    emitter.instruction("je __rt_str_bitwise_finish");                          // no overlap bytes left -> finish
    emitter.instruction("mov al, BYTE PTR [r8]");                               // load next byte from the left operand
    emitter.instruction("xor al, BYTE PTR [r9]");                               // bytewise XOR with the right operand byte
    emitter.instruction("mov BYTE PTR [r10], al");                              // store the result byte
    emitter.instruction("inc r8");                                              // advance the left cursor
    emitter.instruction("inc r9");                                              // advance the right cursor
    emitter.instruction("inc r10");                                             // advance the destination cursor
    emitter.instruction("dec r11");                                             // one fewer overlap byte remaining
    emitter.instruction("jmp __rt_str_bitwise_xor");                            // continue the XOR overlap loop

    // -- publish the result length into the concat buffer and return --
    emitter.label("__rt_str_bitwise_finish");
    crate::codegen::abi::emit_symbol_address(emitter, "r8", "_concat_off");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // reload the concat-buffer write offset
    emitter.instruction("add r9, QWORD PTR [rbp - 48]");                        // advance the offset by the result length
    emitter.instruction("mov QWORD PTR [r8], r9");                              // publish the updated concat-buffer offset
    emitter.instruction("mov rax, QWORD PTR [rbp - 56]");                       // return the result pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // return the result length
    emitter.instruction("add rsp, 64");                                         // release the scratch stack frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the bytewise string result in rax/rdx
}

#[cfg(test)]
mod tests {
    use crate::codegen::platform::{Arch, Platform, Target};

    use super::*;

    /// Verifies the AArch64 helper emits the global symbol, the three per-op overlap
    /// loops (AND/OR/XOR), and the OR tail-copy loop that appends the longer operand.
    #[test]
    fn test_emit_str_bitwise_arm64_emits_all_modes() {
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emit_str_bitwise(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_str_bitwise:\n"));
        assert!(asm.contains("and w13, w13, w14"));
        assert!(asm.contains("orr w13, w13, w14"));
        assert!(asm.contains("eor w13, w13, w14"));
        assert!(asm.contains("__rt_str_bitwise_or_tail_loop:\n"));
    }

    /// Verifies the x86_64 helper uses native byte-op loops for all three operators
    /// and returns the result pointer/length in rax/rdx per the AMD64 ABI.
    #[test]
    fn test_emit_str_bitwise_linux_x86_64_uses_native_byte_ops() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_str_bitwise(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_str_bitwise:\n"));
        assert!(asm.contains("and al, BYTE PTR [r9]\n"));
        assert!(asm.contains("or al, BYTE PTR [r9]\n"));
        assert!(asm.contains("xor al, BYTE PTR [r9]\n"));
        assert!(asm.contains("mov rax, QWORD PTR [rbp - 56]\n"));
    }
}
