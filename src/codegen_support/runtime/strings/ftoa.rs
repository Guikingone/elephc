//! Purpose:
//! Emits the `__rt_ftoa` runtime helper assembly for float-to-string conversion.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Converts a double-precision float to a PHP-compatible byte string.
///
/// # Input
/// - ARM64: `d0` holds the float value
/// - x86_64: `xmm0` holds the float value (SysV variadic ABI)
///
/// # Output
/// - ARM64: `x1` = pointer to string, `x2` = length
/// - x86_64: `rax` = pointer to string, `rdx` = length
///
/// # Behavior
/// Formats the float using `snprintf` with `"%.14G"` format into the global
/// `_concat_buf` buffer at the current `_concat_off` cursor, then advances
/// `_concat_off` by the number of characters written. An exponent form whose mantissa has no
/// fraction gets PHP's `.0` (`1E+25` becomes `1.0E+25`).
///
/// # ABI Notes
/// - Apple ARM64: variadic floats are passed on the stack, not in SIMD registers
/// - Linux x86_64: delegates to `emit_ftoa_linux_x86_64`; uses SysV variadic ABI with `eax=1` to indicate one SIMD register argument
pub fn emit_ftoa(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ftoa_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ftoa ---");
    emitter.label_global("__rt_ftoa");

    // -- set up stack frame (64 bytes) --
    emitter.instruction("sub sp, sp, #64");                                     // allocate 64 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish new frame pointer

    // -- get current concat_buf position --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("ldr x10, [x9]");                                       // load current write offset
    emitter.instruction("str x10, [sp, #32]");                                  // save original offset on stack
    emitter.instruction("str x9, [sp, #40]");                                   // save offset variable address on stack

    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_concat_buf");
    emitter.instruction("add x0, x11, x10");                                    // compute output buffer: concat_buf + offset
    emitter.instruction("str x0, [sp, #24]");                                   // save output buffer start on stack
    emitter.instruction(&format!("mov x12, #{}", crate::codegen_support::runtime::data::CONCAT_BUF_SIZE - 32)); // last offset with room for 32 bytes
    emitter.instruction("cmp x10, x12");                                        // would formatting write past the concat scratch buffer?
    emitter.instruction("b.hi __rt_heap_exhausted");                            // refuse instead of corrupting memory
    emit_ftoa_fast_path_aarch64(emitter);

    // -- call snprintf(buf, 32, "%.14G", double) --
    emitter.instruction("mov x1, #32");                                         // buffer size limit = 32 bytes
    crate::codegen_support::abi::emit_symbol_address(emitter, "x2", "_fmt_g");          // load page address of format string "%.14G"
    // -- Apple ARM64 variadic ABI: float arg goes on stack, not in SIMD reg --
    emitter.instruction("str d0, [sp]");                                        // push double onto stack for variadic call
    emitter.bl_c("snprintf");                                        // call snprintf; returns char count in x0

    // -- x0 = number of chars written --
    emitter.instruction("mov x2, x0");                                          // save string length as return value

    // -- PHP prints an integral exponent mantissa with ".0" (1.0E+25): insert it before 'E' --
    emitter.instruction("ldr x3, [sp, #24]");                                   // formatted text
    emitter.instruction("mov x4, #0");                                          // scan index
    emitter.label("__rt_ftoa_exp_scan");
    emitter.instruction("cmp x4, x2");                                          // end of text?
    emitter.instruction("b.ge __rt_ftoa_exp_done");                             // no exponent: nothing to fix
    emitter.instruction("ldrb w5, [x3, x4]");                                   // current byte
    emitter.instruction("cmp w5, #0x2E");                                       // '.'?
    emitter.instruction("b.eq __rt_ftoa_exp_done");                             // mantissa already has a fraction
    emitter.instruction("cmp w5, #0x45");                                       // 'E'?
    emitter.instruction("b.eq __rt_ftoa_exp_insert");                           // insert ".0" here
    emitter.instruction("add x4, x4, #1");                                      // next byte
    emitter.instruction("b __rt_ftoa_exp_scan");                                // continue
    emitter.label("__rt_ftoa_exp_insert");
    emitter.instruction("mov x6, x2");                                          // shift cursor = end
    emitter.label("__rt_ftoa_exp_shift");
    emitter.instruction("cmp x6, x4");                                          // shifted the exponent suffix?
    emitter.instruction("b.le __rt_ftoa_exp_put");                              // yes
    emitter.instruction("sub x6, x6, #1");                                      // previous byte
    emitter.instruction("ldrb w5, [x3, x6]");                                   // load it
    emitter.instruction("add x7, x6, #2");                                      // two bytes to the right
    emitter.instruction("strb w5, [x3, x7]");                                   // move it
    emitter.instruction("b __rt_ftoa_exp_shift");                               // continue
    emitter.label("__rt_ftoa_exp_put");
    emitter.instruction("mov w5, #0x2E");                                       // '.'
    emitter.instruction("strb w5, [x3, x4]");                                   // write it before 'E'
    emitter.instruction("add x7, x4, #1");                                      // next position
    emitter.instruction("mov w5, #0x30");                                       // '0'
    emitter.instruction("strb w5, [x3, x7]");                                   // write it
    emitter.instruction("add x2, x2, #2");                                      // two more bytes
    emitter.label("__rt_ftoa_exp_done");

    // -- PHP does not pad the exponent to two digits (1.0E-7, not 1.0E-07) --
    emitter.instruction("mov x4, #0");                                          // scan index
    emitter.label("__rt_ftoa_exp0_scan");
    emitter.instruction("cmp x4, x2");                                          // end of text?
    emitter.instruction("b.ge __rt_ftoa_exp0_done");                            // no exponent
    emitter.instruction("ldrb w5, [x3, x4]");                                   // current byte
    emitter.instruction("cmp w5, #0x45");                                       // 'E'?
    emitter.instruction("b.eq __rt_ftoa_exp0_found");                           // inspect the exponent digits
    emitter.instruction("add x4, x4, #1");                                      // next byte
    emitter.instruction("b __rt_ftoa_exp0_scan");                               // continue
    emitter.label("__rt_ftoa_exp0_found");
    emitter.instruction("add x4, x4, #2");                                      // first exponent digit (after the sign)
    emitter.instruction("sub x6, x2, x4");                                      // number of exponent digits
    emitter.instruction("cmp x6, #2");                                          // padded two-digit exponent?
    emitter.instruction("b.lt __rt_ftoa_exp0_done");                            // single digit: keep
    emitter.instruction("ldrb w5, [x3, x4]");                                   // first exponent digit
    emitter.instruction("cmp w5, #0x30");                                       // leading zero?
    emitter.instruction("b.ne __rt_ftoa_exp0_done");                            // no
    emitter.label("__rt_ftoa_exp0_shift");
    emitter.instruction("add x7, x4, #1");                                      // next digit
    emitter.instruction("cmp x7, x2");                                          // past the end?
    emitter.instruction("b.ge __rt_ftoa_exp0_shrink");                          // shifted every digit
    emitter.instruction("ldrb w5, [x3, x7]");                                   // load it
    emitter.instruction("strb w5, [x3, x4]");                                   // move it left
    emitter.instruction("mov x4, x7");                                          // advance
    emitter.instruction("b __rt_ftoa_exp0_shift");                              // continue
    emitter.label("__rt_ftoa_exp0_shrink");
    emitter.instruction("sub x2, x2, #1");                                      // one byte shorter
    emitter.label("__rt_ftoa_exp0_done");

    // -- update concat_off by chars written --
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload offset variable address
    emitter.instruction("ldr x10, [sp, #32]");                                  // reload original offset
    emitter.instruction("add x10, x10, x2");                                    // new offset = original + chars written
    emitter.instruction("str x10, [x9]");                                       // store updated offset

    // -- set return pointer --
    emitter.instruction("ldr x1, [sp, #24]");                                   // return pointer to start of formatted string

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the `__rt_ftoa` routine for Linux x86_64.
///
/// # Input
/// - `xmm0` holds the float value (SysV variadic ABI)
///
/// # Output
/// - `rax` = pointer to formatted string, `rdx` = length
///
/// # Behavior
/// Same as `emit_ftoa` but for the Linux x86_64 target. Uses `rbp`-based
/// frame with 32 bytes of scratch space for concat cursor and output pointer.
/// Sets `eax = 1` to signal one SIMD register argument to `snprintf`.
fn emit_ftoa_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ftoa ---");
    emitter.label_global("__rt_ftoa");

    emitter.instruction("push rbp");                                            // save the caller frame pointer before using stack locals
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the formatting helper
    emitter.instruction("sub rsp, 32");                                         // reserve aligned scratch space for concat offsets and the output pointer

    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_concat_off");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // load the current concat cursor so formatted bytes append after prior output
    emitter.instruction("mov QWORD PTR [rbp - 8], r9");                         // save the original concat cursor for the final offset update
    emitter.instruction("mov QWORD PTR [rbp - 16], r8");                        // save the concat cursor symbol address for the final store

    crate::codegen_support::abi::emit_symbol_address(emitter, "r10", "_concat_buf");
    emitter.instruction("lea rdi, [r10 + r9]");                                 // compute the destination buffer inside the concat scratch area
    emitter.instruction("mov QWORD PTR [rbp - 24], rdi");                       // preserve the destination pointer for the return value
    emitter.instruction(&format!("cmp r9, {}", crate::codegen_support::runtime::data::CONCAT_BUF_SIZE - 32)); // room for 32 bytes in the concat scratch buffer?
    emitter.instruction("ja __rt_heap_exhausted");                              // refuse instead of corrupting memory
    emit_ftoa_fast_path_x86_64(emitter);

    emitter.instruction("mov esi, 32");                                         // cap float formatting to the same 32-byte scratch window used on AArch64
    crate::codegen_support::abi::emit_symbol_address(emitter, "rdx", "_fmt_g");
    emitter.instruction("mov eax, 1");                                          // SysV variadic ABI: one SIMD register is live for the double argument
    emitter.instruction("call snprintf");                                       // format xmm0 using "%.14G" into the concat scratch buffer

    emitter.instruction("mov rdx, rax");                                        // return the formatted byte count in the string-length result register

    // -- PHP prints an integral exponent mantissa with ".0" (1.0E+25): insert it before 'E' --
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // formatted text
    emitter.instruction("xor ecx, ecx");                                        // scan index
    emitter.label("__rt_ftoa_exp_scan");
    emitter.instruction("cmp rcx, rdx");                                        // end of text?
    emitter.instruction("jge __rt_ftoa_exp_done");                              // no exponent: nothing to fix
    emitter.instruction("movzx eax, BYTE PTR [rdi + rcx]");                     // current byte
    emitter.instruction("cmp eax, 0x2E");                                       // '.'?
    emitter.instruction("je __rt_ftoa_exp_done");                               // mantissa already has a fraction
    emitter.instruction("cmp eax, 0x45");                                       // 'E'?
    emitter.instruction("je __rt_ftoa_exp_insert");                             // insert ".0" here
    emitter.instruction("add rcx, 1");                                          // next byte
    emitter.instruction("jmp __rt_ftoa_exp_scan");                              // continue
    emitter.label("__rt_ftoa_exp_insert");
    emitter.instruction("mov rsi, rdx");                                        // shift cursor = end
    emitter.label("__rt_ftoa_exp_shift");
    emitter.instruction("cmp rsi, rcx");                                        // shifted the exponent suffix?
    emitter.instruction("jle __rt_ftoa_exp_put");                               // yes
    emitter.instruction("sub rsi, 1");                                          // previous byte
    emitter.instruction("movzx eax, BYTE PTR [rdi + rsi]");                     // load it
    emitter.instruction("mov BYTE PTR [rdi + rsi + 2], al");                    // move it two bytes to the right
    emitter.instruction("jmp __rt_ftoa_exp_shift");                             // continue
    emitter.label("__rt_ftoa_exp_put");
    emitter.instruction("mov BYTE PTR [rdi + rcx], 0x2E");                      // '.'
    emitter.instruction("mov BYTE PTR [rdi + rcx + 1], 0x30");                  // '0'
    emitter.instruction("add rdx, 2");                                          // two more bytes
    emitter.label("__rt_ftoa_exp_done");

    // -- PHP does not pad the exponent to two digits (1.0E-7, not 1.0E-07) --
    emitter.instruction("xor ecx, ecx");                                        // scan index
    emitter.label("__rt_ftoa_exp0_scan");
    emitter.instruction("cmp rcx, rdx");                                        // end of text?
    emitter.instruction("jge __rt_ftoa_exp0_done");                             // no exponent
    emitter.instruction("cmp BYTE PTR [rdi + rcx], 0x45");                      // 'E'?
    emitter.instruction("je __rt_ftoa_exp0_found");                             // inspect the exponent digits
    emitter.instruction("add rcx, 1");                                          // next byte
    emitter.instruction("jmp __rt_ftoa_exp0_scan");                             // continue
    emitter.label("__rt_ftoa_exp0_found");
    emitter.instruction("add rcx, 2");                                          // first exponent digit (after the sign)
    emitter.instruction("mov rsi, rdx");                                        // end
    emitter.instruction("sub rsi, rcx");                                        // number of exponent digits
    emitter.instruction("cmp rsi, 2");                                          // padded two-digit exponent?
    emitter.instruction("jl __rt_ftoa_exp0_done");                              // single digit: keep
    emitter.instruction("cmp BYTE PTR [rdi + rcx], 0x30");                      // leading zero?
    emitter.instruction("jne __rt_ftoa_exp0_done");                             // no
    emitter.label("__rt_ftoa_exp0_shift");
    emitter.instruction("lea rsi, [rcx + 1]");                                  // next digit
    emitter.instruction("cmp rsi, rdx");                                        // past the end?
    emitter.instruction("jge __rt_ftoa_exp0_shrink");                           // shifted every digit
    emitter.instruction("movzx eax, BYTE PTR [rdi + rsi]");                     // load it
    emitter.instruction("mov BYTE PTR [rdi + rcx], al");                        // move it left
    emitter.instruction("mov rcx, rsi");                                        // advance
    emitter.instruction("jmp __rt_ftoa_exp0_shift");                            // continue
    emitter.label("__rt_ftoa_exp0_shrink");
    emitter.instruction("sub rdx, 1");                                          // one byte shorter
    emitter.label("__rt_ftoa_exp0_done");
    emitter.instruction("mov r8, QWORD PTR [rbp - 16]");                        // reload the concat cursor symbol address
    emitter.instruction("mov r9, QWORD PTR [rbp - 8]");                         // reload the original concat cursor
    emitter.instruction("add r9, rdx");                                         // advance the concat cursor by the number of formatted bytes
    emitter.instruction("mov QWORD PTR [r8], r9");                              // publish the updated concat cursor for subsequent string writes
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // return the pointer to the formatted float text

    emitter.instruction("add rsp, 32");                                         // release the local scratch area before returning
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return pointer+length in rax/rdx
}

/// Bit patterns of the fast-path bounds: `%.14G` prints `[1e-4, 1e14)` in fixed notation.
const FAST_LOW_BITS: u64 = 0x3F1A_36E2_EB1C_432D; // 1e-4
const FAST_HIGH_BITS: u64 = 0x42D6_BCC4_1E90_0000; // 1e14
const TEN_BITS: u64 = 0x4024_0000_0000_0000; // 10.0
const ONE_BITS: u64 = 0x3FF0_0000_0000_0000; // 1.0
/// Largest number of fraction digits the fast path tries.
const FAST_MAX_FRACTION_DIGITS: i64 = 8;

/// Emits the AArch64 exact fast path of `__rt_ftoa`. When `|x|` lies in `[1e-4, 1e14)` and is
/// the double nearest to `m / 10^k` for some `k <= 8` with `m < 1e14`, `%.14G` prints exactly
/// that decimal (fixed notation, no trailing zeros), so it is written directly and control
/// continues at `__rt_ftoa_exp0_done` with the length in x2. Anything else falls through to
/// `snprintf`. Input: d0 = value, x0 = destination. Clobbers x3-x8, x12-x15, d1-d5.
fn emit_ftoa_fast_path_aarch64(emitter: &mut Emitter) {
    let mov_bits = |emitter: &mut Emitter, reg: &str, bits: u64| {
        crate::codegen_support::abi::emit_load_int_immediate(emitter, reg, bits as i64);
    };
    emitter.instruction("fabs d1, d0");                                         // a = |x|
    mov_bits(emitter, "x12", FAST_LOW_BITS);
    emitter.instruction("fmov d2, x12");
    emitter.instruction("fcmp d1, d2");                                         // a >= 1e-4 (false for NaN)?
    emitter.instruction("b.lt __rt_ftoa_slow");                                 // lt also covers unordered
    mov_bits(emitter, "x12", FAST_HIGH_BITS);
    emitter.instruction("fmov d5, x12");                                        // 1e14
    emitter.instruction("fcmp d1, d5");
    emitter.instruction("b.ge __rt_ftoa_slow");                                 // exponent notation range
    mov_bits(emitter, "x12", ONE_BITS);
    emitter.instruction("fmov d3, x12");                                        // scale = 10^k
    mov_bits(emitter, "x12", TEN_BITS);
    emitter.instruction("fmov d4, x12");                                        // 10.0
    emitter.instruction("mov x4, #0");                                          // k
    emitter.instruction("mov x5, #1");                                          // 10^k as an integer
    emitter.label("__rt_ftoa_fast_try");
    emitter.instruction("fmul d2, d1, d3");                                     // a * 10^k
    emitter.instruction("fcmp d2, d5");                                         // more than 14 digits?
    emitter.instruction("b.ge __rt_ftoa_slow");
    emitter.instruction("fcvtns x6, d2");                                       // m = round(a * 10^k)
    emitter.instruction("scvtf d2, x6");
    emitter.instruction("fdiv d2, d2, d3");                                     // m / 10^k
    emitter.instruction("fcmp d2, d1");                                         // does it round-trip to a?
    emitter.instruction("b.eq __rt_ftoa_fast_found");
    emitter.instruction("add x4, x4, #1");                                      // one more fraction digit
    emitter.instruction(&format!("cmp x4, #{}", FAST_MAX_FRACTION_DIGITS));
    emitter.instruction("b.gt __rt_ftoa_slow");
    emitter.instruction("fmul d3, d3, d4");                                     // 10^(k+1), exact
    emitter.instruction("mov x7, #10");
    emitter.instruction("mul x5, x5, x7");
    emitter.instruction("b __rt_ftoa_fast_try");
    emitter.label("__rt_ftoa_fast_found");
    emitter.instruction("mov x3, x0");                                          // write cursor
    emitter.instruction("fmov x7, d0");                                         // sign bit of x
    emitter.instruction("tbz x7, #63, 1f");
    emitter.instruction("mov w7, #0x2D");                                       // '-'
    emitter.instruction("strb w7, [x3], #1");
    emitter.label("1");
    emitter.instruction("udiv x7, x6, x5");                                     // integer part
    emitter.instruction("msub x8, x7, x5, x6");                                 // fraction digits as an integer
    // -- integer part: count its digits, then write them backwards --
    emitter.instruction("mov x12, x7");
    emitter.instruction("mov x13, #0");                                         // digit count
    emitter.instruction("mov x14, #10");
    emitter.label("2");
    emitter.instruction("udiv x12, x12, x14");
    emitter.instruction("add x13, x13, #1");
    emitter.instruction("cbnz x12, 2b");
    emitter.instruction("add x3, x3, x13");                                     // past the integer digits
    emitter.instruction("mov x15, x3");
    emitter.label("3");
    emitter.instruction("udiv x12, x7, x14");
    emitter.instruction("msub x13, x12, x14, x7");                              // lowest digit
    emitter.instruction("add w13, w13, #0x30");
    emitter.instruction("strb w13, [x15, #-1]!");
    emitter.instruction("mov x7, x12");
    emitter.instruction("cbnz x7, 3b");
    // -- fraction: '.', k zero-padded digits, trailing zeros dropped --
    emitter.instruction("cbz x8, 6f");                                          // integral value: no fraction
    emitter.instruction("mov w13, #0x2E");                                      // '.'
    emitter.instruction("strb w13, [x3], #1");
    emitter.instruction("add x3, x3, x4");                                      // past the k fraction digits
    emitter.instruction("mov x15, x3");
    emitter.instruction("mov x7, x4");
    emitter.label("4");
    emitter.instruction("udiv x12, x8, x14");
    emitter.instruction("msub x13, x12, x14, x8");
    emitter.instruction("add w13, w13, #0x30");
    emitter.instruction("strb w13, [x15, #-1]!");
    emitter.instruction("mov x8, x12");
    emitter.instruction("subs x7, x7, #1");
    emitter.instruction("b.ne 4b");
    emitter.label("5");
    emitter.instruction("ldrb w13, [x3, #-1]");                                 // drop trailing zeros
    emitter.instruction("cmp w13, #0x30");
    emitter.instruction("b.ne 6f");
    emitter.instruction("sub x3, x3, #1");
    emitter.instruction("b 5b");
    emitter.label("6");
    emitter.instruction("sub x2, x3, x0");                                      // formatted length
    emitter.instruction("b __rt_ftoa_exp0_done");
    emitter.label("__rt_ftoa_slow");
}

/// Emits the x86_64 exact fast path of `__rt_ftoa` (see `emit_ftoa_fast_path_aarch64`).
/// Input: xmm0 = value, rdi = destination. On success continues at `__rt_ftoa_exp0_done` with
/// the length in rdx. Clobbers rax, rcx, rdx, rsi, r8-r11, xmm1-xmm5.
fn emit_ftoa_fast_path_x86_64(emitter: &mut Emitter) {
    emitter.instruction("movq rax, xmm0");
    emitter.instruction("btr rax, 63");                                         // a = |x|
    emitter.instruction("movq xmm1, rax");
    emitter.instruction(&format!("mov rax, 0x{:x}", FAST_LOW_BITS));
    emitter.instruction("movq xmm2, rax");
    emitter.instruction("ucomisd xmm1, xmm2");                                  // a >= 1e-4? (CF set below or unordered)
    emitter.instruction("jb __rt_ftoa_slow");
    emitter.instruction(&format!("mov rax, 0x{:x}", FAST_HIGH_BITS));
    emitter.instruction("movq xmm5, rax");                                      // 1e14
    emitter.instruction("ucomisd xmm1, xmm5");
    emitter.instruction("jae __rt_ftoa_slow");                                  // exponent notation range
    emitter.instruction(&format!("mov rax, 0x{:x}", ONE_BITS));
    emitter.instruction("movq xmm3, rax");                                      // scale = 10^k
    emitter.instruction(&format!("mov rax, 0x{:x}", TEN_BITS));
    emitter.instruction("movq xmm4, rax");                                      // 10.0
    emitter.instruction("xor ecx, ecx");                                        // k
    emitter.instruction("mov r9d, 1");                                          // 10^k as an integer
    emitter.label("__rt_ftoa_fast_try");
    emitter.instruction("movapd xmm2, xmm1");
    emitter.instruction("mulsd xmm2, xmm3");                                    // a * 10^k
    emitter.instruction("ucomisd xmm2, xmm5");                                  // more than 14 digits?
    emitter.instruction("jae __rt_ftoa_slow");
    emitter.instruction("cvtsd2si r8, xmm2");                                   // m = round(a * 10^k)
    emitter.instruction("cvtsi2sd xmm2, r8");
    emitter.instruction("divsd xmm2, xmm3");                                    // m / 10^k
    emitter.instruction("ucomisd xmm2, xmm1");                                  // does it round-trip to a?
    emitter.instruction("jp 1f");
    emitter.instruction("je __rt_ftoa_fast_found");
    emitter.label("1");
    emitter.instruction("add ecx, 1");                                          // one more fraction digit
    emitter.instruction(&format!("cmp ecx, {}", FAST_MAX_FRACTION_DIGITS));
    emitter.instruction("jg __rt_ftoa_slow");
    emitter.instruction("mulsd xmm3, xmm4");                                    // 10^(k+1), exact
    emitter.instruction("imul r9, r9, 10");
    emitter.instruction("jmp __rt_ftoa_fast_try");
    emitter.label("__rt_ftoa_fast_found");
    emitter.instruction("mov rsi, rdi");                                        // write cursor
    emitter.instruction("movq rax, xmm0");
    emitter.instruction("test rax, rax");                                       // negative?
    emitter.instruction("jns 2f");
    emitter.instruction("mov BYTE PTR [rsi], 0x2D");                            // '-'
    emitter.instruction("add rsi, 1");
    emitter.label("2");
    emitter.instruction("mov rax, r8");
    emitter.instruction("xor edx, edx");
    emitter.instruction("div r9");                                              // rax = integer part, rdx = fraction digits
    emitter.instruction("mov r11, rdx");
    emitter.instruction("mov r10, 10");
    // -- integer part: count its digits, then write them backwards --
    emitter.instruction("mov r8, rax");
    emitter.instruction("mov rax, r8");
    emitter.instruction("xor r9d, r9d");                                        // digit count
    emitter.label("3");
    emitter.instruction("xor edx, edx");
    emitter.instruction("div r10");
    emitter.instruction("add r9, 1");
    emitter.instruction("test rax, rax");
    emitter.instruction("jnz 3b");
    emitter.instruction("add rsi, r9");                                         // past the integer digits
    emitter.instruction("lea r9, [rsi - 1]");
    emitter.instruction("mov rax, r8");
    emitter.label("4");
    emitter.instruction("xor edx, edx");
    emitter.instruction("div r10");
    emitter.instruction("add dl, 0x30");
    emitter.instruction("mov BYTE PTR [r9], dl");
    emitter.instruction("sub r9, 1");
    emitter.instruction("test rax, rax");
    emitter.instruction("jnz 4b");
    // -- fraction: '.', k zero-padded digits, trailing zeros dropped --
    emitter.instruction("test r11, r11");
    emitter.instruction("jz 7f");                                               // integral value: no fraction
    emitter.instruction("mov BYTE PTR [rsi], 0x2E");                            // '.'
    emitter.instruction("add rsi, 1");
    emitter.instruction("add rsi, rcx");                                        // past the k fraction digits
    emitter.instruction("lea r9, [rsi - 1]");
    emitter.instruction("mov rax, r11");
    emitter.label("5");
    emitter.instruction("xor edx, edx");
    emitter.instruction("div r10");
    emitter.instruction("add dl, 0x30");
    emitter.instruction("mov BYTE PTR [r9], dl");
    emitter.instruction("sub r9, 1");
    emitter.instruction("sub ecx, 1");
    emitter.instruction("jnz 5b");
    emitter.label("6");
    emitter.instruction("cmp BYTE PTR [rsi - 1], 0x30");                        // drop trailing zeros
    emitter.instruction("jne 7f");
    emitter.instruction("sub rsi, 1");
    emitter.instruction("jmp 6b");
    emitter.label("7");
    emitter.instruction("mov rdx, rsi");
    emitter.instruction("sub rdx, rdi");                                        // formatted length
    emitter.instruction("jmp __rt_ftoa_exp0_done");
    emitter.label("__rt_ftoa_slow");
}

#[cfg(test)]
mod tests {
    use crate::codegen_support::platform::{Arch, Platform, Target};

    use super::*;

    /// Verifies that `emit_ftoa` on Linux x86_64 uses the SysV variadic calling convention
    /// by checking that `eax` is set to 1 (one SIMD register argument) before calling `snprintf`.
    #[test]
    fn test_emit_ftoa_linux_x86_64_uses_sysv_variadic_call() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_ftoa(&mut emitter);
        let asm = emitter.output();

        assert!(asm.contains("__rt_ftoa:\n"));
        assert!(asm.contains("mov eax, 1\n"));
        assert!(asm.contains("call snprintf\n"));
        assert!(asm.contains("mov rdx, rax\n"));
    }
}
