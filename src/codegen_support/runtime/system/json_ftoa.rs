//! Purpose:
//! Emits the `__rt_json_ftoa` runtime helper: formats a finite double at
//! `serialize_precision = -1` (the shortest decimal string that round-trips
//! back to the same `double`), with a `d.d` mantissa in exponential form, an
//! exponent with no leading zeros, and NO trailing `.0` for integer-valued
//! floats (`100`, not `100.0`). The exponent marker is a caller-supplied
//! parameter: `'e'` (lowercase) for PHP's `json_encode` layout, `'E'`
//! (uppercase) for PHP's `serialize`/`var_export` layout — the only byte that
//! differs between the two formats (thresholds and digits are identical).
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via
//!   `crate::codegen_support::runtime::system`.
//! - `__rt_json_encode_float` (same module group) passes `'e'` for the finite
//!   and substituted-zero paths; `__rt_serialize` passes `'E'`.
//!
//! Key details:
//! - The shortest precision is found by probing `snprintf("%.*e", p, x)` for
//!   `p` in `[0, 16]` and stopping at the first `p` whose `strtod` re-parse
//!   equals `x` (17 significant digits always round-trip, so `p = 16` is the
//!   ceiling). This mirrors the tested `var_export` prelude formatter, minus
//!   the `.0` suffix and using the caller-supplied exponent marker.
//! - The decimal exponent `E` is parsed from the `%e` scratch via `strtol`;
//!   `decpt = E + 1` selects exponential layout when `decpt < -3 || decpt > 17`
//!   (the same thresholds PHP/`zend_gcvt` use), otherwise a decimal layout is
//!   produced with `snprintf("%.*f", max(0, p - E), x)` straight into
//!   `_concat_buf`.
//! - Output ABI matches `__rt_ftoa`: result bytes land in `_concat_buf` at the
//!   current `_concat_off`, the cursor is advanced by the byte count, and the
//!   pointer/length are returned in `x1`/`x2` (AArch64) or `rax`/`rdx`
//!   (x86_64). The caller's `JSON_PRESERVE_ZERO_FRACTION` post-pass still runs.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::abi;

/// Emits `__rt_json_ftoa`, the shortest-round-trip float formatter shared by
/// `json_encode` and `serialize`.
///
/// Input: AArch64 `d0` / x86_64 `xmm0` = a finite double (Inf/NaN are excluded
/// by the caller `__rt_json_encode_float`); AArch64 `w0` / x86_64 `dil` = the
/// ASCII exponent marker to emit in exponential form (`'e'` for json, `'E'`
/// for serialize). The char is stashed on the stack and only consumed on the
/// exponential path, so callers always pass it even for decimal-valued floats.
/// Output: AArch64 `x1`/`x2`, x86_64 `rax`/`rdx` = pointer/length of the
/// formatted slice inside `_concat_buf`, with `_concat_off` advanced past it.
pub(crate) fn emit_json_ftoa(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: json_ftoa (serialize_precision=-1 shortest round-trip) ---");
    emitter.label_global("__rt_json_ftoa");
    emit_short_decimal_fast_path_aarch64(emitter);

    // -- set up stack frame (128 bytes) --
    emitter.instruction("sub sp, sp, #144");                                    // variadic area, scratch, saved double, saved regs, exp char
    emitter.instruction("stp x29, x30, [sp, #112]");                            // save frame pointer and return address
    emitter.instruction("add x29, sp, #112");                                   // establish a new frame pointer
    emitter.instruction("stp x19, x20, [sp, #96]");                             // save callee-saved registers x19/x20
    emitter.instruction("str x21, [sp, #88]");                                  // save callee-saved register x21
    emitter.instruction("str d0, [sp, #80]");                                   // save the input double for re-formatting and compare
    emitter.instruction("strb w0, [sp, #136]");                                 // stash exponent char param ('e' json / 'E' serialize)

    // -- probe for the shortest precision p in [0,16] that round-trips --
    emitter.instruction("mov x19, #0");                                         // p = 0 (precision passed to "%.*e")
    emitter.label("__rt_json_ftoa_probe");
    emitter.instruction("add x0, sp, #16");                                     // snprintf buffer = scratch area
    emitter.instruction("mov x1, #64");                                         // scratch buffer size
    abi::emit_symbol_address(emitter, "x2", "_fmt_star_e");
    emitter.instruction("str x19, [sp, #0]");                                   // Apple variadic arg 0: int precision p (on stack)
    emitter.instruction("ldr d0, [sp, #80]");                                   // reload the input double
    emitter.instruction("str d0, [sp, #8]");                                    // Apple variadic arg 1: the double value (on stack)
    emitter.instruction("ldr x3, [sp, #0]");                                    // AAPCS64 variadic arg 0: int precision p (in x3)
    emitter.bl_c("snprintf");                                                   // format x at precision p (double stays in d0 for AAPCS64)
    emitter.instruction("add x0, sp, #16");                                     // strtod source = formatted scratch string
    emitter.instruction("mov x1, #0");                                          // strtod endptr = NULL
    emitter.bl_c("strtod");                                                     // parse the formatted string back to a double (d0)
    emitter.instruction("ldr d1, [sp, #80]");                                   // reload the original input double
    emitter.instruction("fcmp d0, d1");                                         // did the formatted string round-trip exactly?
    emitter.instruction("b.eq __rt_json_ftoa_probe_done");                      // shortest precision found
    emitter.instruction("cmp x19, #16");                                        // reached the 17-significant-digit ceiling?
    emitter.instruction("b.ge __rt_json_ftoa_probe_done");                      // accept p=16 (always round-trips)
    emitter.instruction("add x19, x19, #1");                                    // try one more digit of precision
    emitter.instruction("b __rt_json_ftoa_probe");                              // re-run the probe
    emitter.label("__rt_json_ftoa_probe_done");

    // -- parse the decimal exponent E from the "%.*e" scratch string --
    emitter.instruction("ldrb w9, [sp, #16]");                                  // scratch[0]
    emitter.instruction("cmp w9, #45");                                         // is the first byte a '-' sign?
    emitter.instruction("cset w20, eq");                                        // neg = 1 when the value is negative
    emitter.instruction("add x10, x20, #1");                                    // index past optional sign and the leading digit
    emitter.instruction("cbz x19, __rt_json_ftoa_have_eidx");                   // p==0 has no '.' or fractional digits
    emitter.instruction("add x10, x10, #1");                                    // skip the '.' separator
    emitter.instruction("add x10, x10, x19");                                   // skip the p fractional digits
    emitter.label("__rt_json_ftoa_have_eidx");
    emitter.instruction("add x0, sp, #16");                                     // base of scratch string
    emitter.instruction("add x0, x0, x10");                                     // address of the 'e' marker
    emitter.instruction("add x0, x0, #1");                                      // address of the exponent text after 'e'
    emitter.instruction("mov x1, #0");                                          // strtol endptr = NULL
    emitter.instruction("mov x2, #10");                                         // base 10
    emitter.bl_c("strtol");                                                     // E = parsed decimal exponent
    emitter.instruction("mov x21, x0");                                         // keep E in a callee-saved register

    // -- choose decimal vs exponential layout by decimal-point position --
    emitter.instruction("add x9, x21, #1");                                     // decpt = E + 1
    emitter.instruction("cmn x9, #3");                                          // compare decpt against -3
    emitter.instruction("b.lt __rt_json_ftoa_exp");                             // decpt < -3 -> exponential form
    emitter.instruction("cmp x9, #17");                                         // compare decpt against 17
    emitter.instruction("b.gt __rt_json_ftoa_exp");                             // decpt > 17 -> exponential form

    // -- decimal form: snprintf("%.*f", max(0, p - E), x) into concat_buf --
    emitter.instruction("sub x9, x19, x21");                                    // fracdigits = p - E
    emitter.instruction("cmp x9, #0");                                          // is the fractional digit count negative?
    emitter.instruction("csel x9, x9, xzr, ge");                                // clamp negatives to zero (integer-valued)
    emitter.instruction("str x9, [sp, #0]");                                    // Apple variadic arg 0: fractional digit count (stack)
    abi::emit_symbol_address(emitter, "x10", "_concat_off");
    emitter.instruction("ldr x11, [x10]");                                      // current concat offset
    abi::emit_symbol_address(emitter, "x12", "_concat_buf");
    emitter.instruction("add x0, x12, x11");                                    // destination = concat_buf + offset
    emitter.instruction("mov x1, #48");                                         // destination size cap
    abi::emit_symbol_address(emitter, "x2", "_fmt_star_f");
    emitter.instruction("ldr d0, [sp, #80]");                                   // reload the input double
    emitter.instruction("str d0, [sp, #8]");                                    // Apple variadic arg 1: the double value (stack)
    emitter.instruction("ldr x3, [sp, #0]");                                    // AAPCS64 variadic arg 0: fractional digit count (x3)
    emitter.bl_c("snprintf");                                                   // format the decimal digits (double stays in d0 for AAPCS64)
    emitter.instruction("mov x2, x0");                                          // result length = bytes written
    abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("ldr x10, [x9]");                                       // original offset (unchanged by snprintf)
    abi::emit_symbol_address(emitter, "x11", "_concat_buf");
    emitter.instruction("add x1, x11, x10");                                    // result pointer = concat_buf + offset
    emitter.instruction("add x10, x10, x2");                                    // advance the cursor past the digits
    emitter.instruction("str x10, [x9]");                                       // publish the new concat offset
    emitter.instruction("b __rt_json_ftoa_done");                               // finished decimal layout

    // -- exponential form: d.dddde[+-]E with json conventions, byte by byte --
    emitter.label("__rt_json_ftoa_exp");
    abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("ldr x10, [x9]");                                       // current concat offset
    abi::emit_symbol_address(emitter, "x11", "_concat_buf");
    emitter.instruction("add x12, x11, x10");                                   // cursor = concat_buf + offset
    emitter.instruction("mov x1, x12");                                         // remember the result start pointer
    emitter.instruction("cbz x20, __rt_json_ftoa_exp_first");                   // skip sign when the value is non-negative
    emitter.instruction("mov w13, #45");                                        // ASCII '-'
    emitter.instruction("strb w13, [x12], #1");                                 // emit the sign and advance the cursor
    emitter.label("__rt_json_ftoa_exp_first");
    emitter.instruction("add x13, sp, #16");                                    // base of scratch string
    emitter.instruction("ldrb w14, [x13, x20]");                                // first significant digit (after optional sign)
    emitter.instruction("strb w14, [x12], #1");                                 // emit the leading digit
    emitter.instruction("mov w13, #46");                                        // ASCII '.'
    emitter.instruction("strb w13, [x12], #1");                                 // emit the decimal point
    emitter.instruction("cbnz x19, __rt_json_ftoa_exp_frac");                   // p>0 copies the fractional digits
    emitter.instruction("mov w13, #48");                                        // ASCII '0' (mantissa needs one fraction digit)
    emitter.instruction("strb w13, [x12], #1");                                 // emit "0" so the mantissa reads "d.0"
    emitter.instruction("b __rt_json_ftoa_exp_e");                              // continue with the exponent marker
    emitter.label("__rt_json_ftoa_exp_frac");
    emitter.instruction("add x13, sp, #16");                                    // base of scratch string
    emitter.instruction("add x13, x13, x20");                                   // skip optional sign
    emitter.instruction("add x13, x13, #2");                                    // skip the leading digit and '.'
    emitter.instruction("mov x14, #0");                                         // fractional copy index
    emitter.label("__rt_json_ftoa_exp_frac_loop");
    emitter.instruction("cmp x14, x19");                                        // copied all p fractional digits?
    emitter.instruction("b.ge __rt_json_ftoa_exp_e");                           // mantissa fraction complete
    emitter.instruction("ldrb w15, [x13, x14]");                                // load fractional digit
    emitter.instruction("strb w15, [x12], #1");                                 // emit fractional digit
    emitter.instruction("add x14, x14, #1");                                    // advance the fractional index
    emitter.instruction("b __rt_json_ftoa_exp_frac_loop");                      // copy the next fractional digit
    emitter.label("__rt_json_ftoa_exp_e");
    emitter.instruction("ldrb w13, [sp, #136]");                                // exponent char param ('e' json / 'E' serialize)
    emitter.instruction("strb w13, [x12], #1");                                 // emit the exponent marker
    emitter.instruction("cmp x21, #0");                                         // is the exponent negative?
    emitter.instruction("b.ge __rt_json_ftoa_exp_pos");                         // positive exponent uses '+'
    emitter.instruction("mov w13, #45");                                        // ASCII '-'
    emitter.instruction("strb w13, [x12], #1");                                 // emit the exponent sign
    emitter.instruction("neg x21, x21");                                        // make the exponent magnitude positive
    emitter.instruction("b __rt_json_ftoa_exp_mag");                            // emit the magnitude digits
    emitter.label("__rt_json_ftoa_exp_pos");
    emitter.instruction("mov w13, #43");                                        // ASCII '+'
    emitter.instruction("strb w13, [x12], #1");                                 // emit the exponent sign
    emitter.label("__rt_json_ftoa_exp_mag");
    emitter.instruction("mov x13, #100");                                       // divisor for the hundreds digit
    emitter.instruction("udiv x14, x21, x13");                                  // hundreds = E / 100
    emitter.instruction("msub x15, x14, x13, x21");                             // remainder = E - hundreds*100
    emitter.instruction("mov x13, #10");                                        // divisor for the tens digit
    emitter.instruction("udiv x16, x15, x13");                                  // tens = remainder / 10
    emitter.instruction("msub x17, x16, x13, x15");                             // ones = remainder - tens*10
    emitter.instruction("cbz x14, __rt_json_ftoa_exp_tens");                    // skip hundreds when it is zero
    emitter.instruction("add w13, w14, #48");                                   // hundreds digit to ASCII
    emitter.instruction("strb w13, [x12], #1");                                 // emit hundreds digit
    emitter.instruction("add w13, w16, #48");                                   // tens digit to ASCII
    emitter.instruction("strb w13, [x12], #1");                                 // emit tens digit
    emitter.instruction("b __rt_json_ftoa_exp_ones");                           // emit the ones digit
    emitter.label("__rt_json_ftoa_exp_tens");
    emitter.instruction("cbz x16, __rt_json_ftoa_exp_ones");                    // skip tens when it (and hundreds) are zero
    emitter.instruction("add w13, w16, #48");                                   // tens digit to ASCII
    emitter.instruction("strb w13, [x12], #1");                                 // emit tens digit
    emitter.label("__rt_json_ftoa_exp_ones");
    emitter.instruction("add w13, w17, #48");                                   // ones digit to ASCII
    emitter.instruction("strb w13, [x12], #1");                                 // emit ones digit
    emitter.instruction("sub x2, x12, x1");                                     // result length = cursor - start
    abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("ldr x10, [x9]");                                       // original concat offset
    emitter.instruction("add x10, x10, x2");                                    // advance past the emitted bytes
    emitter.instruction("str x10, [x9]");                                       // publish the new concat offset

    emitter.label("__rt_json_ftoa_done");
    emitter.instruction("ldr x21, [sp, #88]");                                  // restore callee-saved register x21
    emitter.instruction("ldp x19, x20, [sp, #96]");                             // restore callee-saved registers x19/x20
    emitter.instruction("ldp x29, x30, [sp, #112]");                            // restore frame pointer and return address
    emitter.instruction("add sp, sp, #144");                                    // release the stack frame
    emitter.instruction("ret");                                                 // return result pointer (x1) and length (x2)
}

/// Emits the x86_64 variant of `__rt_json_ftoa`.
///
/// Mirrors the AArch64 path exactly: probe the shortest `%.*e` precision with
/// `snprintf`/`strtod`, parse the exponent with `strtol`, then emit either a
/// `%.*f` decimal slice or a hand-built `d.dddde[+-]E` exponential slice into
/// `_concat_buf`. Variadic calls use the SysV register convention (`rcx` =
/// `*` precision, `xmm0` = double, `al` = 1).
///
/// Input: `xmm0` = a finite double, `dil` = the ASCII exponent marker.
/// Output: `rax`/`rdx` = pointer/length.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: json_ftoa (serialize_precision=-1 shortest round-trip) ---");
    emitter.label_global("__rt_json_ftoa");
    emit_short_decimal_fast_path_x86_64(emitter);

    emitter.instruction("push rbp");                                            // save caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("push rbx");                                            // save callee-saved rbx (precision counter)
    emitter.instruction("push r12");                                            // save callee-saved r12 (sign flag)
    emitter.instruction("push r13");                                            // save callee-saved r13 (exponent)
    emitter.instruction("push r14");                                            // save callee-saved r14 (result start pointer)
    emitter.instruction("sub rsp, 96");                                         // reserve scratch buffer, saved-double slot, exp char
    emitter.instruction("mov DWORD PTR [rsp + 80], edi");                       // stash exponent char param ('e' json / 'E' serialize)
    emitter.instruction("movsd QWORD PTR [rsp + 64], xmm0");                    // save the input double for re-formatting and compare
    emitter.instruction("xor ebx, ebx");                                        // p = 0 (precision passed to "%.*e")

    emitter.label("__rt_json_ftoa_probe_x");
    emitter.instruction("lea rdi, [rsp]");                                      // snprintf buffer = scratch area
    emitter.instruction("mov esi, 64");                                         // scratch buffer size
    abi::emit_symbol_address(emitter, "rdx", "_fmt_star_e");
    emitter.instruction("mov ecx, ebx");                                        // variadic int precision = p
    emitter.instruction("movsd xmm0, QWORD PTR [rsp + 64]");                    // variadic double = input value
    emitter.instruction("mov eax, 1");                                          // one vector register used by the variadic call
    emitter.instruction("call snprintf");                                       // format x at precision p into scratch
    emitter.instruction("lea rdi, [rsp]");                                      // strtod source = formatted scratch
    emitter.instruction("xor esi, esi");                                        // strtod endptr = NULL
    emitter.instruction("call strtod");                                         // parse the formatted string back to a double (xmm0)
    emitter.instruction("movsd xmm1, QWORD PTR [rsp + 64]");                    // reload the original input double
    emitter.instruction("ucomisd xmm0, xmm1");                                  // did the formatted string round-trip exactly?
    emitter.instruction("je __rt_json_ftoa_probe_done_x");                      // shortest precision found
    emitter.instruction("cmp ebx, 16");                                         // reached the 17-significant-digit ceiling?
    emitter.instruction("jge __rt_json_ftoa_probe_done_x");                     // accept p=16 (always round-trips)
    emitter.instruction("inc ebx");                                             // try one more digit of precision
    emitter.instruction("jmp __rt_json_ftoa_probe_x");                          // re-run the probe
    emitter.label("__rt_json_ftoa_probe_done_x");

    emitter.instruction("movzx eax, BYTE PTR [rsp]");                           // scratch[0]
    emitter.instruction("cmp al, 45");                                          // is the first byte a '-' sign?
    emitter.instruction("sete r12b");                                           // neg = 1 when the value is negative
    emitter.instruction("movzx r12d, r12b");                                    // zero-extend the sign flag
    emitter.instruction("lea rax, [r12 + 1]");                                  // index past optional sign and the leading digit
    emitter.instruction("test rbx, rbx");                                       // is the precision zero (no fractional part)?
    emitter.instruction("jz __rt_json_ftoa_have_eidx_x");                       // p==0 has no '.' to skip
    emitter.instruction("add rax, 1");                                          // skip the '.' separator
    emitter.instruction("add rax, rbx");                                        // skip the p fractional digits
    emitter.label("__rt_json_ftoa_have_eidx_x");
    emitter.instruction("lea rdi, [rsp + rax + 1]");                            // address of the exponent text after 'e'
    emitter.instruction("xor esi, esi");                                        // strtol endptr = NULL
    emitter.instruction("mov edx, 10");                                         // base 10
    emitter.instruction("call strtol");                                         // E = parsed decimal exponent
    emitter.instruction("mov r13, rax");                                        // keep E in a callee-saved register

    emitter.instruction("lea rax, [r13 + 1]");                                  // decpt = E + 1
    emitter.instruction("cmp rax, -3");                                         // compare decpt against -3
    emitter.instruction("jl __rt_json_ftoa_exp_x");                             // decpt < -3 -> exponential form
    emitter.instruction("cmp rax, 17");                                         // compare decpt against 17
    emitter.instruction("jg __rt_json_ftoa_exp_x");                             // decpt > 17 -> exponential form

    emitter.instruction("mov rcx, rbx");                                        // fracdigits = p ...
    emitter.instruction("sub rcx, r13");                                        // ... minus E
    emitter.instruction("test rcx, rcx");                                       // is the fractional digit count negative?
    emitter.instruction("jns __rt_json_ftoa_frac_ok_x");                        // non-negative count is fine
    emitter.instruction("xor ecx, ecx");                                        // clamp to zero (integer-valued)
    emitter.label("__rt_json_ftoa_frac_ok_x");
    abi::emit_load_symbol_to_reg(emitter, "r8", "_concat_off", 0);              // current concat offset
    abi::emit_symbol_address(emitter, "r9", "_concat_buf");
    emitter.instruction("lea rdi, [r9 + r8]");                                  // destination = concat_buf + offset
    emitter.instruction("mov esi, 48");                                         // destination size cap
    abi::emit_symbol_address(emitter, "rdx", "_fmt_star_f");
    emitter.instruction("movsd xmm0, QWORD PTR [rsp + 64]");                    // variadic double = input value
    emitter.instruction("mov eax, 1");                                          // one vector register used by the variadic call
    emitter.instruction("call snprintf");                                       // format the decimal digits into concat_buf
    emitter.instruction("mov rdx, rax");                                        // result length = bytes written
    abi::emit_load_symbol_to_reg(emitter, "r8", "_concat_off", 0);              // original offset (unchanged by snprintf)
    abi::emit_symbol_address(emitter, "r9", "_concat_buf");
    emitter.instruction("lea rax, [r9 + r8]");                                  // result pointer = concat_buf + offset
    emitter.instruction("add r8, rdx");                                         // advance the cursor past the digits
    abi::emit_store_reg_to_symbol(emitter, "r8", "_concat_off", 0);             // publish the new concat offset
    emitter.instruction("jmp __rt_json_ftoa_done_x");                           // finished decimal layout

    emitter.label("__rt_json_ftoa_exp_x");
    abi::emit_load_symbol_to_reg(emitter, "r8", "_concat_off", 0);              // current concat offset
    abi::emit_symbol_address(emitter, "r9", "_concat_buf");
    emitter.instruction("lea r10, [r9 + r8]");                                  // cursor = concat_buf + offset
    emitter.instruction("mov r14, r10");                                        // remember the result start pointer
    emitter.instruction("test r12, r12");                                       // is the value negative?
    emitter.instruction("jz __rt_json_ftoa_exp_first_x");                       // skip sign when non-negative
    emitter.instruction("mov BYTE PTR [r10], 45");                              // emit '-'
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.label("__rt_json_ftoa_exp_first_x");
    emitter.instruction("movzx ecx, BYTE PTR [rsp + r12]");                     // first significant digit (after optional sign)
    emitter.instruction("mov BYTE PTR [r10], cl");                              // emit the leading digit
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.instruction("mov BYTE PTR [r10], 46");                              // emit '.'
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.instruction("test rbx, rbx");                                       // does the mantissa have fractional digits?
    emitter.instruction("jnz __rt_json_ftoa_exp_frac_x");                       // p>0 copies the fractional digits
    emitter.instruction("mov BYTE PTR [r10], 48");                              // emit '0' (mantissa "d.0")
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.instruction("jmp __rt_json_ftoa_exp_e_x");                          // continue with the exponent marker
    emitter.label("__rt_json_ftoa_exp_frac_x");
    emitter.instruction("lea rsi, [rsp + r12 + 2]");                            // &scratch[neg+2] = first fractional digit
    emitter.instruction("xor edi, edi");                                        // fractional copy index
    emitter.label("__rt_json_ftoa_exp_frac_loop_x");
    emitter.instruction("cmp rdi, rbx");                                        // copied all p fractional digits?
    emitter.instruction("jge __rt_json_ftoa_exp_e_x");                          // mantissa fraction complete
    emitter.instruction("movzx ecx, BYTE PTR [rsi + rdi]");                     // load fractional digit
    emitter.instruction("mov BYTE PTR [r10], cl");                              // emit fractional digit
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.instruction("inc rdi");                                             // advance the fractional index
    emitter.instruction("jmp __rt_json_ftoa_exp_frac_loop_x");                  // copy the next fractional digit
    emitter.label("__rt_json_ftoa_exp_e_x");
    emitter.instruction("movzx eax, BYTE PTR [rsp + 80]");                      // exponent char param ('e' json / 'E' serialize)
    emitter.instruction("mov BYTE PTR [r10], al");                              // emit the exponent marker
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.instruction("test r13, r13");                                       // is the exponent negative?
    emitter.instruction("jns __rt_json_ftoa_exp_pos_x");                        // non-negative exponent uses '+'
    emitter.instruction("mov BYTE PTR [r10], 45");                              // emit '-'
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.instruction("neg r13");                                             // make the exponent magnitude positive
    emitter.instruction("jmp __rt_json_ftoa_exp_mag_x");                        // emit the magnitude digits
    emitter.label("__rt_json_ftoa_exp_pos_x");
    emitter.instruction("mov BYTE PTR [r10], 43");                              // emit '+'
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.label("__rt_json_ftoa_exp_mag_x");
    emitter.instruction("mov rax, r13");                                        // exponent magnitude into the dividend
    emitter.instruction("xor edx, edx");                                        // clear the high dividend half
    emitter.instruction("mov ecx, 100");                                        // divisor for the hundreds digit
    emitter.instruction("div rcx");                                             // rax = E/100 (hundreds), rdx = E%100
    emitter.instruction("mov r8, rax");                                         // save the hundreds digit
    emitter.instruction("mov rax, rdx");                                        // remainder (E % 100) into the dividend
    emitter.instruction("xor edx, edx");                                        // clear the high dividend half
    emitter.instruction("mov ecx, 10");                                         // divisor for the tens digit
    emitter.instruction("div rcx");                                             // rax = tens, rdx = ones
    emitter.instruction("mov r9, rax");                                         // save the tens digit
    emitter.instruction("mov r11, rdx");                                        // save the ones digit
    emitter.instruction("test r8, r8");                                         // is the hundreds digit zero?
    emitter.instruction("jz __rt_json_ftoa_exp_tens_x");                        // skip leading-zero hundreds
    emitter.instruction("lea ecx, [r8 + 48]");                                  // hundreds digit to ASCII
    emitter.instruction("mov BYTE PTR [r10], cl");                              // emit hundreds digit
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.instruction("lea ecx, [r9 + 48]");                                  // tens digit to ASCII
    emitter.instruction("mov BYTE PTR [r10], cl");                              // emit tens digit
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.instruction("jmp __rt_json_ftoa_exp_ones_x");                       // emit the ones digit
    emitter.label("__rt_json_ftoa_exp_tens_x");
    emitter.instruction("test r9, r9");                                         // is the tens digit zero?
    emitter.instruction("jz __rt_json_ftoa_exp_ones_x");                        // skip leading-zero tens
    emitter.instruction("lea ecx, [r9 + 48]");                                  // tens digit to ASCII
    emitter.instruction("mov BYTE PTR [r10], cl");                              // emit tens digit
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.label("__rt_json_ftoa_exp_ones_x");
    emitter.instruction("lea ecx, [r11 + 48]");                                 // ones digit to ASCII
    emitter.instruction("mov BYTE PTR [r10], cl");                              // emit ones digit
    emitter.instruction("inc r10");                                             // advance the cursor
    emitter.instruction("mov rax, r14");                                        // result pointer = start
    emitter.instruction("mov rdx, r10");                                        // cursor (one past the last byte)
    emitter.instruction("sub rdx, rax");                                        // result length = cursor - start
    abi::emit_load_symbol_to_reg(emitter, "r8", "_concat_off", 0);              // original concat offset
    emitter.instruction("add r8, rdx");                                         // advance past the emitted bytes
    abi::emit_store_reg_to_symbol(emitter, "r8", "_concat_off", 0);             // publish the new concat offset

    emitter.label("__rt_json_ftoa_done_x");
    emitter.instruction("add rsp, 96");                                         // release the scratch frame
    emitter.instruction("pop r14");                                             // restore callee-saved r14
    emitter.instruction("pop r13");                                             // restore callee-saved r13
    emitter.instruction("pop r12");                                             // restore callee-saved r12
    emitter.instruction("pop rbx");                                             // restore callee-saved rbx
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return result pointer (rax) and length (rdx)
}

/// Highest fraction-digit count the short-decimal fast path tries.
pub(crate) const JSON_FTOA_FAST_MAX_FRACTION_DIGITS: usize = 6;

/// Emits the AArch64 short-decimal fast path of `__rt_json_ftoa`.
///
/// For a non-zero `x` with `1e-4 <= |x| < 1e15` (the magnitudes PHP prints in decimal
/// layout), the smallest `k` in `0..=6` with `(double)round(x * 10^k) / 10^k == x` gives the
/// shortest round-trip digits: the division of two exact values is correctly rounded, so the
/// decimal `m / 10^k` parses back to `x`, and no shorter decimal exists because it would have
/// matched at a smaller `k`. The digits of `m` are written with a point `k` digits from the
/// right (no trailing `.0` for integers), so the output equals the `snprintf`/`strtod` probe's.
/// Other values fall through to the probe. Clobbers only caller-saved scratch registers and
/// keeps `w0` (the exponent marker) for the fallback.
fn emit_short_decimal_fast_path_aarch64(emitter: &mut Emitter) {
    emitter.instruction("fabs d1, d0");                                         // d1 = |x|
    abi::emit_symbol_address(emitter, "x9", "_json_ftoa_fast_consts");
    emitter.instruction("ldp d2, d3, [x9]");                                    // d2 = 1e-4 lower bound, d3 = 1e15 upper bound
    emitter.instruction("fcmp d1, d2");                                         // is |x| below the decimal-layout fast range (or zero)?
    emitter.instruction("b.lo __rt_json_ftoa_slow");                            // yes — use the general probe
    emitter.instruction("fcmp d1, d3");                                         // is |x| at or above 1e15?
    emitter.instruction("b.hs __rt_json_ftoa_slow");                            // yes — use the general probe
    emitter.instruction("add x9, x9, #16");                                     // x9 = powers-of-ten table (1, 10, ..., 1e6)
    emitter.instruction("mov x10, #0");                                         // k = 0 fraction digits
    emitter.label("__rt_json_ftoa_fast_try");
    emitter.instruction("ldr d2, [x9, x10, lsl #3]");                           // d2 = 10^k
    emitter.instruction("fmul d3, d0, d2");                                     // x * 10^k
    emitter.instruction("fcvtns x11, d3");                                      // m = round-to-nearest(x * 10^k)
    emitter.instruction("scvtf d3, x11");                                       // (double)m, exact because |m| < 2^53
    emitter.instruction("fdiv d3, d3, d2");                                     // correctly rounded m / 10^k
    emitter.instruction("fcmp d3, d0");                                         // does the k-digit decimal round-trip to x?
    emitter.instruction("b.eq __rt_json_ftoa_fast_found");                      // yes — it is the shortest representation
    emitter.instruction("add x10, x10, #1");                                    // try one more fraction digit
    emitter.instruction(&format!("cmp x10, #{}", JSON_FTOA_FAST_MAX_FRACTION_DIGITS)); // tried every fast fraction width?
    emitter.instruction("b.ls __rt_json_ftoa_fast_try");                        // no — keep trying
    emitter.instruction("b __rt_json_ftoa_slow");                               // no short decimal — use the general probe

    emitter.label("__rt_json_ftoa_fast_found");
    abi::emit_symbol_address(emitter, "x12", "_concat_off");
    emitter.instruction("ldr x13, [x12]");                                      // x13 = current concat offset
    abi::emit_symbol_address(emitter, "x14", "_concat_buf");
    emitter.instruction("add x1, x14, x13");                                    // x1 = result start inside the concat buffer
    emitter.instruction("mov x14, x1");                                         // x14 = write cursor
    emitter.instruction("tbz x11, #63, __rt_json_ftoa_fast_positive");          // non-negative m needs no sign
    emitter.instruction("mov w15, #45");                                        // '-'
    emitter.instruction("strb w15, [x14], #1");                                 // emit the sign
    emitter.instruction("neg x11, x11");                                        // m = |m|
    emitter.label("__rt_json_ftoa_fast_positive");
    // -- count the digits to print: at least k + 1 so a leading "0." appears --
    emitter.instruction("mov x15, x11");                                        // x15 = remaining value
    emitter.instruction("mov x16, #0");                                         // x16 = digit count
    emitter.instruction("mov x17, #10");                                        // divisor
    emitter.label("__rt_json_ftoa_fast_count");
    emitter.instruction("udiv x15, x15, x17");                                  // drop one decimal digit
    emitter.instruction("add x16, x16, #1");                                    // count it
    emitter.instruction("cbnz x15, __rt_json_ftoa_fast_count");                 // more significant digits remain
    emitter.instruction("add x15, x10, #1");                                    // minimum digits = k + 1
    emitter.instruction("cmp x16, x15");                                        // fewer digits than that?
    emitter.instruction("csel x16, x15, x16, lo");                              // pad with leading zeros up to k + 1 digits
    // -- write the digits right to left, with the point k digits from the right --
    emitter.instruction("add x15, x14, x16");                                   // x15 = end of the digit run
    emitter.instruction("cmp x10, #0");                                         // any fraction digits?
    emitter.instruction("cinc x15, x15, ne");                                   // reserve one byte for the decimal point
    emitter.instruction("mov x2, x15");                                         // x2 = end cursor (result end)
    emitter.instruction("mov x13, #0");                                         // x13 = digits written so far
    emitter.label("__rt_json_ftoa_fast_digit");
    emitter.instruction("cmp x13, x10");                                        // reached the decimal point position?
    emitter.instruction("b.ne __rt_json_ftoa_fast_no_point");                   // not yet
    emitter.instruction("cbz x10, __rt_json_ftoa_fast_no_point");               // integers have no point
    emitter.instruction("mov w12, #46");                                        // '.'
    emitter.instruction("strb w12, [x15, #-1]!");                               // emit the decimal point
    emitter.label("__rt_json_ftoa_fast_no_point");
    emitter.instruction("udiv x12, x11, x17");                                  // q = m / 10
    emitter.instruction("msub x3, x12, x17, x11");                              // digit = m - q * 10
    emitter.instruction("add w3, w3, #48");                                     // ASCII digit
    emitter.instruction("strb w3, [x15, #-1]!");                                // emit it right to left
    emitter.instruction("mov x11, x12");                                        // m = q
    emitter.instruction("add x13, x13, #1");                                    // one more digit written
    emitter.instruction("cmp x13, x16");                                        // all digits written?
    emitter.instruction("b.lo __rt_json_ftoa_fast_digit");                      // no — continue
    emitter.instruction("sub x2, x2, x1");                                      // x2 = result length
    abi::emit_symbol_address(emitter, "x12", "_concat_off");
    emitter.instruction("ldr x13, [x12]");                                      // original concat offset
    emitter.instruction("add x13, x13, x2");                                    // advance past the emitted bytes
    emitter.instruction("str x13, [x12]");                                      // publish the new concat offset
    emitter.instruction("ret");                                                 // return x1/x2
    emitter.label("__rt_json_ftoa_slow");
}

/// Emits the x86_64 short-decimal fast path of `__rt_json_ftoa` (see the AArch64 variant).
///
/// Clobbers only caller-saved scratch registers and keeps `dil` (the exponent marker) for
/// the fallback.
fn emit_short_decimal_fast_path_x86_64(emitter: &mut Emitter) {
    emitter.instruction("movq rax, xmm0");                                      // raw bits of x
    emitter.instruction("btr rax, 63");                                         // clear the sign bit
    emitter.instruction("movq xmm1, rax");                                      // xmm1 = |x|
    abi::emit_symbol_address(emitter, "r8", "_json_ftoa_fast_consts");
    emitter.instruction("ucomisd xmm1, QWORD PTR [r8]");                        // is |x| below 1e-4 (or zero)?
    emitter.instruction("jb __rt_json_ftoa_slow_x");                            // yes — use the general probe
    emitter.instruction("ucomisd xmm1, QWORD PTR [r8 + 8]");                    // is |x| at or above 1e15?
    emitter.instruction("jae __rt_json_ftoa_slow_x");                           // yes — use the general probe
    emitter.instruction("add r8, 16");                                          // r8 = powers-of-ten table (1, 10, ..., 1e6)
    emitter.instruction("xor ecx, ecx");                                        // k = 0 fraction digits
    emitter.label("__rt_json_ftoa_fast_try_x");
    emitter.instruction("movapd xmm2, xmm0");                                   // copy x
    emitter.instruction("mulsd xmm2, QWORD PTR [r8 + rcx * 8]");                // x * 10^k
    emitter.instruction("cvtsd2si r9, xmm2");                                   // m = round-to-nearest(x * 10^k)
    emitter.instruction("cvtsi2sd xmm3, r9");                                   // (double)m, exact because |m| < 2^53
    emitter.instruction("divsd xmm3, QWORD PTR [r8 + rcx * 8]");                // correctly rounded m / 10^k
    emitter.instruction("ucomisd xmm3, xmm0");                                  // does the k-digit decimal round-trip to x?
    emitter.instruction("je __rt_json_ftoa_fast_found_x");                      // yes — it is the shortest representation
    emitter.instruction("add ecx, 1");                                          // try one more fraction digit
    emitter.instruction(&format!("cmp ecx, {}", JSON_FTOA_FAST_MAX_FRACTION_DIGITS)); // tried every fast fraction width?
    emitter.instruction("jbe __rt_json_ftoa_fast_try_x");                       // no — keep trying
    emitter.instruction("jmp __rt_json_ftoa_slow_x");                           // no short decimal — use the general probe

    emitter.label("__rt_json_ftoa_fast_found_x");
    abi::emit_load_symbol_to_reg(emitter, "r10", "_concat_off", 0);             // r10 = current concat offset
    abi::emit_symbol_address(emitter, "r11", "_concat_buf");
    emitter.instruction("lea rax, [r11 + r10]");                                // rax = result start inside the concat buffer
    emitter.instruction("mov r11, rax");                                        // r11 = write cursor
    emitter.instruction("test r9, r9");                                         // is m negative?
    emitter.instruction("jns __rt_json_ftoa_fast_positive_x");                  // no — no sign
    emitter.instruction("mov BYTE PTR [r11], 45");                              // emit '-'
    emitter.instruction("add r11, 1");                                          // advance past the sign
    emitter.instruction("neg r9");                                              // m = |m|
    emitter.label("__rt_json_ftoa_fast_positive_x");
    emitter.instruction("push rsi");                                            // preserve rsi for the caller
    // -- count the digits to print: at least k + 1 so a leading "0." appears --
    emitter.instruction("mov r10, r9");                                         // r10 = remaining value
    emitter.instruction("xor esi, esi");                                        // rsi = digit count
    emitter.instruction("mov r8, 10");                                          // divisor
    emitter.label("__rt_json_ftoa_fast_count_x");
    emitter.instruction("push rax");                                            // keep the result start across the division
    emitter.instruction("mov rax, r10");                                        // dividend
    emitter.instruction("xor edx, edx");                                        // zero-extend the dividend
    emitter.instruction("div r8");                                              // drop one decimal digit
    emitter.instruction("mov r10, rax");                                        // remaining value
    emitter.instruction("pop rax");                                             // restore the result start
    emitter.instruction("add rsi, 1");                                          // count the digit
    emitter.instruction("test r10, r10");                                       // more significant digits remain?
    emitter.instruction("jnz __rt_json_ftoa_fast_count_x");                     // yes — keep counting
    emitter.instruction("lea r10, [rcx + 1]");                                  // minimum digits = k + 1
    emitter.instruction("cmp rsi, r10");                                        // fewer digits than that?
    emitter.instruction("cmovb rsi, r10");                                      // pad with leading zeros up to k + 1 digits
    // -- write the digits right to left, with the point k digits from the right --
    emitter.instruction("lea r10, [r11 + rsi]");                                // r10 = end of the digit run
    emitter.instruction("test ecx, ecx");                                       // any fraction digits?
    emitter.instruction("jz __rt_json_ftoa_fast_end_x");                        // integers have no point
    emitter.instruction("add r10, 1");                                          // reserve one byte for the decimal point
    emitter.label("__rt_json_ftoa_fast_end_x");
    emitter.instruction("push r10");                                            // remember the result end
    emitter.instruction("push rax");                                            // keep the result start across the divisions
    emitter.instruction("xor r11d, r11d");                                      // r11 = digits written so far
    emitter.label("__rt_json_ftoa_fast_digit_x");
    emitter.instruction("cmp r11, rcx");                                        // reached the decimal point position?
    emitter.instruction("jne __rt_json_ftoa_fast_no_point_x");                  // not yet
    emitter.instruction("test ecx, ecx");                                       // integers have no point
    emitter.instruction("jz __rt_json_ftoa_fast_no_point_x");                   // skip the point
    emitter.instruction("sub r10, 1");                                          // step left
    emitter.instruction("mov BYTE PTR [r10], 46");                              // emit '.'
    emitter.label("__rt_json_ftoa_fast_no_point_x");
    emitter.instruction("mov rax, r9");                                         // dividend
    emitter.instruction("xor edx, edx");                                        // zero-extend the dividend
    emitter.instruction("div r8");                                              // q = m / 10, rdx = digit
    emitter.instruction("add dl, 48");                                          // ASCII digit
    emitter.instruction("sub r10, 1");                                          // step left
    emitter.instruction("mov BYTE PTR [r10], dl");                              // emit the digit
    emitter.instruction("mov r9, rax");                                         // m = q
    emitter.instruction("add r11, 1");                                          // one more digit written
    emitter.instruction("cmp r11, rsi");                                        // all digits written?
    emitter.instruction("jb __rt_json_ftoa_fast_digit_x");                      // no — continue
    emitter.instruction("pop rax");                                             // rax = result start
    emitter.instruction("pop rdx");                                             // rdx = result end
    emitter.instruction("sub rdx, rax");                                        // rdx = result length
    emitter.instruction("pop rsi");                                             // restore the caller's rsi
    abi::emit_load_symbol_to_reg(emitter, "r10", "_concat_off", 0);             // original concat offset
    emitter.instruction("add r10, rdx");                                        // advance past the emitted bytes
    abi::emit_store_reg_to_symbol(emitter, "r10", "_concat_off", 0);            // publish the new concat offset
    emitter.instruction("ret");                                                 // return rax/rdx
    emitter.label("__rt_json_ftoa_slow_x");
}

/// Emits `__rt_var_dump_ftoa`, the float formatter behind `var_dump()`.
///
/// PHP renders `var_dump()` floats with `serialize_precision = -1`: the shortest
/// round-trip digits with an uppercase exponent (`0.30000000000000004`, `100`,
/// `1.0E+100`), and `INF`/`-INF`/`NAN` for non-finite values. Finite values tail-call
/// `__rt_json_ftoa` with the `'E'` marker; non-finite values tail-call `__rt_ftoa`,
/// which already spells them the PHP way.
///
/// Input: AArch64 `d0` / x86_64 `xmm0` = the double. Output: the same pointer/length
/// registers and `_concat_buf` contract as `__rt_ftoa`.
pub(crate) fn emit_var_dump_ftoa(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: var_dump_ftoa (serialize_precision=-1) ---");
    emitter.label_global("__rt_var_dump_ftoa");
    if emitter.target.arch == Arch::X86_64 {
        emitter.instruction("movq r9, xmm0");                                   // copy the double bits to test the exponent field
        emitter.instruction("shr r9, 52");                                      // shift the exponent (and sign) into the low bits
        emitter.instruction("and r9, 0x7ff");                                   // isolate the 11-bit exponent
        emitter.instruction("cmp r9, 0x7ff");                                   // an all-ones exponent means INF or NAN
        emitter.instruction("je __rt_ftoa");                                    // non-finite values keep the INF/-INF/NAN spelling
        emitter.instruction("mov edi, 69");                                     // exponent marker 'E' for the shortest round-trip layout
        emitter.instruction("jmp __rt_json_ftoa");                              // tail-call the serialize_precision=-1 formatter
        return;
    }
    emitter.instruction("fmov x9, d0");                                         // copy the double bits to test the exponent field
    emitter.instruction("lsr x9, x9, #52");                                     // shift the exponent (and sign) into the low bits
    emitter.instruction("and x9, x9, #0x7ff");                                  // isolate the 11-bit exponent
    emitter.instruction("cmp x9, #0x7ff");                                      // an all-ones exponent means INF or NAN
    emitter.instruction("b.ne __rt_var_dump_ftoa_finite");                      // finite values use the shortest round-trip layout
    // Mach-O cannot relocate a conditional branch to another helper's atom, so the
    // non-finite tail call is an unconditional `b`.
    emitter.instruction("b __rt_ftoa");                                         // non-finite values keep the INF/-INF/NAN spelling
    emitter.label("__rt_var_dump_ftoa_finite");
    emitter.instruction("mov w0, #69");                                         // exponent marker 'E' for the shortest round-trip layout
    emitter.instruction("b __rt_json_ftoa");                                    // tail-call the serialize_precision=-1 formatter
}
