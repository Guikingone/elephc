//! Purpose:
//! Emits `__rt_crc32`, the runtime helper for PHP's `crc32()` — a table-driven
//! (one byte per step) CRC-32/ISO-HDLC computation over the input string,
//! returning the 32-bit checksum as a zero-extended, non-negative integer.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` and the minimal x86
//!   runtime via `crate::codegen_support::runtime::strings`.
//! - The EIR `crc32()` builtin lowerer in `crate::codegen::lower_inst::builtins::strings`.
//!
//! Key details:
//! - Input string ptr/len arrive in x1/x2 (AArch64) or rax/edx (x86_64); the
//!   result is returned in x0 / rax. The 1 KiB lookup table is emitted right after the
//!   helper body and addressed PC-relatively. Uses the reflected polynomial 0xEDB88320
//!   with init and final XOR of 0xFFFFFFFF — the standard zlib/PHP CRC-32.
//! - Writing the 32-bit result register zero-extends, so the value is always a
//!   non-negative PHP int in `0 ..= 0xFFFF_FFFF` (matching 64-bit PHP).

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Reflected CRC-32/ISO-HDLC polynomial (zlib / PHP `crc32()`).
const CRC32_POLY: u32 = 0xEDB8_8320;

/// Builds the 256-entry byte table of the reflected CRC-32 at compile time.
const fn crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut index = 0;
    while index < 256 {
        let mut crc = index as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ CRC32_POLY } else { crc >> 1 };
            bit += 1;
        }
        table[index] = crc;
        index += 1;
    }
    table
}

/// Emits the lookup table right after the helper body (same section, so the helper stays one
/// self-contained, dead-strippable unit reachable with a PC-relative address).
fn emit_crc32_table(emitter: &mut Emitter, label: &str) {
    emitter.raw(".p2align 2");
    emitter.label(label);
    for row in crc32_table().chunks(8) {
        let words: Vec<String> = row.iter().map(|word| format!("0x{:08x}", word)).collect();
        emitter.raw(&format!("    .long {}", words.join(", ")));
    }
}

/// Emits `__rt_crc32(str_ptr, str_len) -> crc`, the CRC-32 checksum helper.
///
/// Table-driven, one byte per step: `crc = table[(crc ^ byte) & 0xFF] ^ (crc >> 8)`.
pub fn emit_crc32(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_crc32_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: crc32 ---");
    emitter.label_global("__rt_crc32");
    emitter.instruction("mov w3, #-1");                                         // crc = 0xFFFFFFFF (CRC-32 initial value)
    emitter.instruction("mov x4, #0");                                          // byte index = 0
    emitter.instruction("adr x9, __rt_crc32_table");                            // x9 = base of the 256-entry byte table
    emitter.label("__rt_crc32_loop");
    emitter.instruction("cmp x4, x2");                                          // processed every input byte?
    emitter.instruction("b.hs __rt_crc32_done");                                // yes → apply the final XOR
    emitter.instruction("ldrb w5, [x1, x4]");                                   // load the current input byte
    emitter.instruction("eor w5, w5, w3");                                      // combine the byte with the low crc bits
    emitter.instruction("and w5, w5, #0xff");                                   // table index = (crc ^ byte) & 0xFF
    emitter.instruction("ldr w6, [x9, x5, lsl #2]");                            // load table[index]
    emitter.instruction("eor w3, w6, w3, lsr #8");                              // crc = table[index] ^ (crc >> 8)
    emitter.instruction("add x4, x4, #1");                                      // advance to the next byte
    emitter.instruction("b __rt_crc32_loop");                                   // continue the byte loop
    emitter.label("__rt_crc32_done");
    emitter.instruction("mvn w0, w3");                                          // result = crc ^ 0xFFFFFFFF; w-write zero-extends x0
    emitter.instruction("ret");                                                 // return the non-negative 32-bit checksum
    emit_crc32_table(emitter, "__rt_crc32_table");
}

/// Emits the Linux x86_64 variant of `__rt_crc32`.
fn emit_crc32_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: crc32 ---");
    emitter.label_global("__rt_crc32");
    emitter.instruction("mov rsi, rax");                                        // rsi = input pointer (rax/rdx hold ptr/len on entry)
    emitter.instruction("mov ecx, -1");                                         // crc = 0xFFFFFFFF (CRC-32 initial value)
    emitter.instruction("xor r8, r8");                                          // byte index = 0
    emitter.instruction("lea r10, [rip + __rt_crc32_table_x86]");               // r10 = base of the 256-entry byte table
    emitter.label("__rt_crc32_loop_x86");
    emitter.instruction("cmp r8, rdx");                                         // processed every input byte?
    emitter.instruction("jae __rt_crc32_done_x86");                             // yes → apply the final XOR
    emitter.instruction("movzx r9d, BYTE PTR [rsi + r8]");                      // load the current input byte
    emitter.instruction("xor r9d, ecx");                                        // combine the byte with the low crc bits
    emitter.instruction("movzx r9d, r9b");                                      // table index = (crc ^ byte) & 0xFF
    emitter.instruction("shr ecx, 8");                                          // crc >> 8
    emitter.instruction("xor ecx, DWORD PTR [r10 + r9*4]");                     // crc = table[index] ^ (crc >> 8)
    emitter.instruction("inc r8");                                              // advance to the next byte
    emitter.instruction("jmp __rt_crc32_loop_x86");                             // continue the byte loop
    emitter.label("__rt_crc32_done_x86");
    emitter.instruction("mov eax, ecx");                                        // move crc into the result register (zero-extends rax)
    emitter.instruction("not eax");                                             // result = crc ^ 0xFFFFFFFF
    emitter.instruction("ret");                                                 // return the non-negative 32-bit checksum
    emit_crc32_table(emitter, "__rt_crc32_table_x86");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the generated table against well-known CRC-32 entries.
    #[test]
    fn table_matches_reference_values() {
        let table = crc32_table();
        assert_eq!(table[0], 0x0000_0000);
        assert_eq!(table[1], 0x7707_3096);
        assert_eq!(table[128], 0xEDB8_8320);
        assert_eq!(table[255], 0x2D02_EF8D);
    }
}
