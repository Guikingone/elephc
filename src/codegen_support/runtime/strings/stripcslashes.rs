//! Purpose:
//! Emits the `__rt_stripcslashes` runtime helper assembly for stripcslashes.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.

use crate::codegen::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_stripcslashes` runtime helper for PHP's `stripcslashes()`.
///
/// Decodes C-style backslash escapes into the shared concat buffer: `\n \r \t \a
/// \v \b \f` map to their control bytes, `\\` collapses to one backslash, `\xHH`
/// consumes up to two hex digits, `\ooo` consumes up to three octal digits, and
/// any other `\c` yields the literal `c`. A trailing backslash with no following
/// byte is preserved. Octal/hex values are truncated to a single byte, matching
/// PHP's `(char)strtol(...)` cast.
///
/// ## ARM64 ABI (default)
/// - Input: `x1` = source pointer, `x2` = source length
/// - Output: `x1` = result pointer, `x2` = result length
/// - Uses the concat buffer (`_concat_buf` / `_concat_off`) for output storage
///
/// ## x86_64 Linux ABI
/// - Input: `rax` = source pointer, `rdx` = source length
/// - Output: `rax` = result pointer, `rdx` = result length
/// - Uses the concat buffer (`_concat_buf` / `_concat_off`) for output storage
pub fn emit_stripcslashes(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_stripcslashes_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: stripcslashes ---");
    emitter.label_global("__rt_stripcslashes");

    // -- set up concat_buf destination and scan cursor --
    abi::emit_symbol_address(emitter, "x6", "_concat_off");
    emitter.instruction("ldr x8, [x6]");                                        // load the current concat-buffer offset
    abi::emit_symbol_address(emitter, "x7", "_concat_buf");
    emitter.instruction("add x9, x7, x8");                                      // destination write pointer = buffer base + offset
    emitter.instruction("mov x10, x9");                                         // remember the result start pointer
    emitter.instruction("mov x11, x2");                                         // remaining source byte count

    emitter.label("__rt_stripcslashes_loop");
    emitter.instruction("cbz x11, __rt_stripcslashes_done");                    // stop once every source byte is consumed
    emitter.instruction("ldrb w12, [x1]");                                      // peek the current source byte
    emitter.instruction("cmp w12, #92");                                        // is it a backslash escape prefix?
    emitter.instruction("b.ne __rt_stripcslashes_copy");                        // ordinary bytes copy through unchanged
    emitter.instruction("cmp x11, #1");                                         // is the backslash the final source byte?
    emitter.instruction("b.eq __rt_stripcslashes_copy");                        // a trailing backslash stays literal
    emitter.instruction("add x1, x1, #1");                                      // consume the escape backslash
    emitter.instruction("sub x11, x11, #1");                                    // account for the consumed backslash
    emitter.instruction("ldrb w13, [x1]");                                      // load the escape selector byte

    emitter.instruction("cmp w13, #110");                                       // '\\n' newline?
    emitter.instruction("b.eq __rt_stripcslashes_n");                           // decode to a line feed
    emitter.instruction("cmp w13, #114");                                       // '\\r' carriage return?
    emitter.instruction("b.eq __rt_stripcslashes_r");                           // decode to a carriage return
    emitter.instruction("cmp w13, #97");                                        // '\\a' alert/bell?
    emitter.instruction("b.eq __rt_stripcslashes_a");                           // decode to a bell byte
    emitter.instruction("cmp w13, #116");                                       // '\\t' tab?
    emitter.instruction("b.eq __rt_stripcslashes_t");                           // decode to a horizontal tab
    emitter.instruction("cmp w13, #118");                                       // '\\v' vertical tab?
    emitter.instruction("b.eq __rt_stripcslashes_v");                           // decode to a vertical tab
    emitter.instruction("cmp w13, #98");                                        // '\\b' backspace?
    emitter.instruction("b.eq __rt_stripcslashes_b");                           // decode to a backspace byte
    emitter.instruction("cmp w13, #102");                                       // '\\f' form feed?
    emitter.instruction("b.eq __rt_stripcslashes_f");                           // decode to a form feed byte
    emitter.instruction("cmp w13, #92");                                        // '\\\\' escaped backslash?
    emitter.instruction("b.eq __rt_stripcslashes_bs");                          // decode to a single backslash
    emitter.instruction("cmp w13, #120");                                       // '\\x' hexadecimal escape?
    emitter.instruction("b.eq __rt_stripcslashes_hex");                         // decode hex digits
    emitter.instruction("sub w14, w13, #48");                                   // selector - '0' to test for an octal digit
    emitter.instruction("cmp w14, #7");                                         // is the selector one of '0'..'7'?
    emitter.instruction("b.hi __rt_stripcslashes_literal");                     // non-octal selectors emit their literal byte
    emitter.instruction("b __rt_stripcslashes_octal");                          // decode an octal escape run

    emitter.label("__rt_stripcslashes_n");
    emitter.instruction("mov w12, #10");                                        // decoded newline byte
    emitter.instruction("b __rt_stripcslashes_emit1");                          // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_r");
    emitter.instruction("mov w12, #13");                                        // decoded carriage-return byte
    emitter.instruction("b __rt_stripcslashes_emit1");                          // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_a");
    emitter.instruction("mov w12, #7");                                         // decoded bell byte
    emitter.instruction("b __rt_stripcslashes_emit1");                          // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_t");
    emitter.instruction("mov w12, #9");                                         // decoded tab byte
    emitter.instruction("b __rt_stripcslashes_emit1");                          // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_v");
    emitter.instruction("mov w12, #11");                                        // decoded vertical-tab byte
    emitter.instruction("b __rt_stripcslashes_emit1");                          // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_b");
    emitter.instruction("mov w12, #8");                                         // decoded backspace byte
    emitter.instruction("b __rt_stripcslashes_emit1");                          // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_f");
    emitter.instruction("mov w12, #12");                                        // decoded form-feed byte
    emitter.instruction("b __rt_stripcslashes_emit1");                          // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_bs");
    emitter.instruction("mov w12, #92");                                        // decoded single backslash
    emitter.instruction("b __rt_stripcslashes_emit1");                          // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_literal");
    emitter.instruction("mov w12, w13");                                        // unknown escape yields the literal selector byte

    emitter.label("__rt_stripcslashes_emit1");
    emitter.instruction("strb w12, [x9], #1");                                  // write the decoded byte to the output
    emitter.instruction("add x1, x1, #1");                                      // consume the single selector byte
    emitter.instruction("sub x11, x11, #1");                                    // account for the consumed selector
    emitter.instruction("b __rt_stripcslashes_loop");                           // continue scanning

    emitter.label("__rt_stripcslashes_copy");
    emitter.instruction("strb w12, [x9], #1");                                  // copy the current byte unchanged
    emitter.instruction("add x1, x1, #1");                                      // advance past the copied byte
    emitter.instruction("sub x11, x11, #1");                                    // account for the copied byte
    emitter.instruction("b __rt_stripcslashes_loop");                           // continue scanning

    // -- octal escape: up to three '0'..'7' digits starting at the selector --
    emitter.label("__rt_stripcslashes_octal");
    emitter.instruction("mov w15, w14");                                        // accumulator = first octal digit value
    emitter.instruction("add x1, x1, #1");                                      // consume the first octal digit
    emitter.instruction("sub x11, x11, #1");                                    // account for the first octal digit
    emitter.instruction("cbz x11, __rt_stripcslashes_octal_done");              // stop if no more source bytes remain
    emitter.instruction("ldrb w13, [x1]");                                      // peek the next byte
    emitter.instruction("sub w14, w13, #48");                                   // byte - '0' to test the second octal digit
    emitter.instruction("cmp w14, #7");                                         // is it '0'..'7'?
    emitter.instruction("b.hi __rt_stripcslashes_octal_done");                  // non-octal ends the run
    emitter.instruction("lsl w15, w15, #3");                                    // accumulator *= 8
    emitter.instruction("add w15, w15, w14");                                   // accumulator += second digit
    emitter.instruction("add x1, x1, #1");                                      // consume the second octal digit
    emitter.instruction("sub x11, x11, #1");                                    // account for the second octal digit
    emitter.instruction("cbz x11, __rt_stripcslashes_octal_done");              // stop if no more source bytes remain
    emitter.instruction("ldrb w13, [x1]");                                      // peek the next byte
    emitter.instruction("sub w14, w13, #48");                                   // byte - '0' to test the third octal digit
    emitter.instruction("cmp w14, #7");                                         // is it '0'..'7'?
    emitter.instruction("b.hi __rt_stripcslashes_octal_done");                  // non-octal ends the run
    emitter.instruction("lsl w15, w15, #3");                                    // accumulator *= 8
    emitter.instruction("add w15, w15, w14");                                   // accumulator += third digit
    emitter.instruction("add x1, x1, #1");                                      // consume the third octal digit
    emitter.instruction("sub x11, x11, #1");                                    // account for the third octal digit
    emitter.label("__rt_stripcslashes_octal_done");
    emitter.instruction("strb w15, [x9], #1");                                  // write the octal value truncated to one byte
    emitter.instruction("b __rt_stripcslashes_loop");                           // continue scanning

    // -- hex escape: up to two hex digits after the 'x' selector --
    emitter.label("__rt_stripcslashes_hex");
    emitter.instruction("add x1, x1, #1");                                      // consume the 'x' selector
    emitter.instruction("sub x11, x11, #1");                                    // account for the 'x' selector
    emitter.instruction("cbz x11, __rt_stripcslashes_hex_literal");             // no digit after 'x' emits a literal 'x'
    emitter.instruction("ldrb w13, [x1]");                                      // peek the first hex-digit candidate
    emitter.instruction("sub w14, w13, #48");                                   // candidate - '0'
    emitter.instruction("cmp w14, #9");                                         // is it a decimal digit '0'..'9'?
    emitter.instruction("b.ls __rt_stripcslashes_hex1_ok");                     // decimal digits map directly to their value
    emitter.instruction("orr w14, w13, #0x20");                                 // fold the candidate to lowercase
    emitter.instruction("sub w14, w14, #97");                                   // folded - 'a'
    emitter.instruction("cmp w14, #5");                                         // is it in 'a'..'f'?
    emitter.instruction("b.hi __rt_stripcslashes_hex_literal");                 // no hex digit after 'x' emits a literal 'x'
    emitter.instruction("add w14, w14, #10");                                   // hex letter value = 10 + (folded - 'a')
    emitter.label("__rt_stripcslashes_hex1_ok");
    emitter.instruction("add x1, x1, #1");                                      // consume the first hex digit
    emitter.instruction("sub x11, x11, #1");                                    // account for the first hex digit
    emitter.instruction("mov w15, w14");                                        // accumulator = first hex nibble
    emitter.instruction("cbz x11, __rt_stripcslashes_hex_emit");                // stop after a single hex digit
    emitter.instruction("ldrb w13, [x1]");                                      // peek the second hex-digit candidate
    emitter.instruction("sub w14, w13, #48");                                   // candidate - '0'
    emitter.instruction("cmp w14, #9");                                         // is it a decimal digit '0'..'9'?
    emitter.instruction("b.ls __rt_stripcslashes_hex2_ok");                     // decimal digits map directly to their value
    emitter.instruction("orr w14, w13, #0x20");                                 // fold the candidate to lowercase
    emitter.instruction("sub w14, w14, #97");                                   // folded - 'a'
    emitter.instruction("cmp w14, #5");                                         // is it in 'a'..'f'?
    emitter.instruction("b.hi __rt_stripcslashes_hex_emit");                    // a non-hex second byte ends the escape
    emitter.instruction("add w14, w14, #10");                                   // hex letter value = 10 + (folded - 'a')
    emitter.label("__rt_stripcslashes_hex2_ok");
    emitter.instruction("lsl w15, w15, #4");                                    // accumulator <<= 4 for the high nibble
    emitter.instruction("add w15, w15, w14");                                   // accumulator += second hex nibble
    emitter.instruction("add x1, x1, #1");                                      // consume the second hex digit
    emitter.instruction("sub x11, x11, #1");                                    // account for the second hex digit
    emitter.label("__rt_stripcslashes_hex_emit");
    emitter.instruction("strb w15, [x9], #1");                                  // write the decoded hex byte
    emitter.instruction("b __rt_stripcslashes_loop");                           // continue scanning
    emitter.label("__rt_stripcslashes_hex_literal");
    emitter.instruction("mov w15, #120");                                       // '\\x' with no hex digit emits a literal 'x'
    emitter.instruction("strb w15, [x9], #1");                                  // write the literal 'x'
    emitter.instruction("b __rt_stripcslashes_loop");                           // continue scanning

    emitter.label("__rt_stripcslashes_done");
    emitter.instruction("mov x1, x10");                                         // result pointer = saved start
    emitter.instruction("sub x2, x9, x10");                                     // result length = write pointer - start
    emitter.instruction("ldr x8, [x6]");                                        // reload the concat-buffer offset
    emitter.instruction("add x8, x8, x2");                                      // advance the offset by the result length
    emitter.instruction("str x8, [x6]");                                        // publish the updated concat-buffer offset
    emitter.instruction("ret");                                                 // return the decoded string slice
}

/// Emits the x86_64 Linux implementation of `__rt_stripcslashes`.
///
/// Same decoding semantics as the ARM64 variant using System V registers:
/// - Input: `rax` = source pointer, `rdx` = source length
/// - Output: `rax` = result pointer, `rdx` = result length
fn emit_stripcslashes_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: stripcslashes ---");
    emitter.label_global("__rt_stripcslashes");

    abi::emit_load_symbol_to_reg(emitter, "r8", "_concat_off", 0);              // load the current concat-buffer offset
    abi::emit_symbol_address(emitter, "r9", "_concat_buf");                     // materialize the concat-buffer base pointer
    emitter.instruction("add r9, r8");                                          // destination write pointer = base + offset
    emitter.instruction("mov r10, r9");                                         // remember the result start pointer
    emitter.instruction("mov rcx, rdx");                                        // remaining source byte count

    emitter.label("__rt_stripcslashes_loop_x86_64");
    emitter.instruction("test rcx, rcx");                                       // stop once every source byte is consumed
    emitter.instruction("je __rt_stripcslashes_done_x86_64");                   // finish when no bytes remain
    emitter.instruction("movzx r11d, BYTE PTR [rax]");                          // peek the current source byte
    emitter.instruction("cmp r11b, 92");                                        // is it a backslash escape prefix?
    emitter.instruction("jne __rt_stripcslashes_copy_x86_64");                  // ordinary bytes copy through unchanged
    emitter.instruction("cmp rcx, 1");                                          // is the backslash the final source byte?
    emitter.instruction("je __rt_stripcslashes_copy_x86_64");                   // a trailing backslash stays literal
    emitter.instruction("add rax, 1");                                          // consume the escape backslash
    emitter.instruction("sub rcx, 1");                                          // account for the consumed backslash
    emitter.instruction("movzx esi, BYTE PTR [rax]");                           // load the escape selector byte into a caller-saved scratch

    emitter.instruction("cmp sil, 110");                                        // '\\n' newline?
    emitter.instruction("je __rt_stripcslashes_n_x86_64");                      // decode to a line feed
    emitter.instruction("cmp sil, 114");                                        // '\\r' carriage return?
    emitter.instruction("je __rt_stripcslashes_r_x86_64");                      // decode to a carriage return
    emitter.instruction("cmp sil, 97");                                         // '\\a' alert/bell?
    emitter.instruction("je __rt_stripcslashes_a_x86_64");                      // decode to a bell byte
    emitter.instruction("cmp sil, 116");                                        // '\\t' tab?
    emitter.instruction("je __rt_stripcslashes_t_x86_64");                      // decode to a horizontal tab
    emitter.instruction("cmp sil, 118");                                        // '\\v' vertical tab?
    emitter.instruction("je __rt_stripcslashes_v_x86_64");                      // decode to a vertical tab
    emitter.instruction("cmp sil, 98");                                         // '\\b' backspace?
    emitter.instruction("je __rt_stripcslashes_b_x86_64");                      // decode to a backspace byte
    emitter.instruction("cmp sil, 102");                                        // '\\f' form feed?
    emitter.instruction("je __rt_stripcslashes_f_x86_64");                      // decode to a form feed byte
    emitter.instruction("cmp sil, 92");                                         // '\\\\' escaped backslash?
    emitter.instruction("je __rt_stripcslashes_bs_x86_64");                     // decode to a single backslash
    emitter.instruction("cmp sil, 120");                                        // '\\x' hexadecimal escape?
    emitter.instruction("je __rt_stripcslashes_hex_x86_64");                    // decode hex digits
    emitter.instruction("lea edi, [rsi - 48]");                                 // selector - '0' to test for an octal digit
    emitter.instruction("cmp edi, 7");                                          // is the selector one of '0'..'7'?
    emitter.instruction("ja __rt_stripcslashes_literal_x86_64");                // non-octal selectors emit their literal byte
    emitter.instruction("jmp __rt_stripcslashes_octal_x86_64");                 // decode an octal escape run

    emitter.label("__rt_stripcslashes_n_x86_64");
    emitter.instruction("mov r11d, 10");                                        // decoded newline byte
    emitter.instruction("jmp __rt_stripcslashes_emit1_x86_64");                 // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_r_x86_64");
    emitter.instruction("mov r11d, 13");                                        // decoded carriage-return byte
    emitter.instruction("jmp __rt_stripcslashes_emit1_x86_64");                 // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_a_x86_64");
    emitter.instruction("mov r11d, 7");                                         // decoded bell byte
    emitter.instruction("jmp __rt_stripcslashes_emit1_x86_64");                 // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_t_x86_64");
    emitter.instruction("mov r11d, 9");                                         // decoded tab byte
    emitter.instruction("jmp __rt_stripcslashes_emit1_x86_64");                 // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_v_x86_64");
    emitter.instruction("mov r11d, 11");                                        // decoded vertical-tab byte
    emitter.instruction("jmp __rt_stripcslashes_emit1_x86_64");                 // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_b_x86_64");
    emitter.instruction("mov r11d, 8");                                         // decoded backspace byte
    emitter.instruction("jmp __rt_stripcslashes_emit1_x86_64");                 // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_f_x86_64");
    emitter.instruction("mov r11d, 12");                                        // decoded form-feed byte
    emitter.instruction("jmp __rt_stripcslashes_emit1_x86_64");                 // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_bs_x86_64");
    emitter.instruction("mov r11d, 92");                                        // decoded single backslash
    emitter.instruction("jmp __rt_stripcslashes_emit1_x86_64");                 // emit the decoded byte and consume the selector
    emitter.label("__rt_stripcslashes_literal_x86_64");
    emitter.instruction("mov r11d, esi");                                       // unknown escape yields the literal selector byte

    emitter.label("__rt_stripcslashes_emit1_x86_64");
    emitter.instruction("mov BYTE PTR [r9], r11b");                             // write the decoded byte to the output
    emitter.instruction("add r9, 1");                                           // advance the output write pointer
    emitter.instruction("add rax, 1");                                          // consume the single selector byte
    emitter.instruction("sub rcx, 1");                                          // account for the consumed selector
    emitter.instruction("jmp __rt_stripcslashes_loop_x86_64");                  // continue scanning

    emitter.label("__rt_stripcslashes_copy_x86_64");
    emitter.instruction("mov BYTE PTR [r9], r11b");                             // copy the current byte unchanged
    emitter.instruction("add r9, 1");                                           // advance the output write pointer
    emitter.instruction("add rax, 1");                                          // advance past the copied byte
    emitter.instruction("sub rcx, 1");                                          // account for the copied byte
    emitter.instruction("jmp __rt_stripcslashes_loop_x86_64");                  // continue scanning

    emitter.label("__rt_stripcslashes_octal_x86_64");
    emitter.instruction("mov r11d, edi");                                       // accumulator = first octal digit value
    emitter.instruction("add rax, 1");                                          // consume the first octal digit
    emitter.instruction("sub rcx, 1");                                          // account for the first octal digit
    emitter.instruction("test rcx, rcx");                                       // are more source bytes available?
    emitter.instruction("je __rt_stripcslashes_octal_done_x86_64");             // stop the run when no bytes remain
    emitter.instruction("movzx esi, BYTE PTR [rax]");                           // peek the next byte
    emitter.instruction("lea edi, [rsi - 48]");                                 // byte - '0' to test the second octal digit
    emitter.instruction("cmp edi, 7");                                          // is it '0'..'7'?
    emitter.instruction("ja __rt_stripcslashes_octal_done_x86_64");             // non-octal ends the run
    emitter.instruction("shl r11d, 3");                                         // accumulator *= 8
    emitter.instruction("add r11d, edi");                                       // accumulator += second digit
    emitter.instruction("add rax, 1");                                          // consume the second octal digit
    emitter.instruction("sub rcx, 1");                                          // account for the second octal digit
    emitter.instruction("test rcx, rcx");                                       // are more source bytes available?
    emitter.instruction("je __rt_stripcslashes_octal_done_x86_64");             // stop the run when no bytes remain
    emitter.instruction("movzx esi, BYTE PTR [rax]");                           // peek the next byte
    emitter.instruction("lea edi, [rsi - 48]");                                 // byte - '0' to test the third octal digit
    emitter.instruction("cmp edi, 7");                                          // is it '0'..'7'?
    emitter.instruction("ja __rt_stripcslashes_octal_done_x86_64");             // non-octal ends the run
    emitter.instruction("shl r11d, 3");                                         // accumulator *= 8
    emitter.instruction("add r11d, edi");                                       // accumulator += third digit
    emitter.instruction("add rax, 1");                                          // consume the third octal digit
    emitter.instruction("sub rcx, 1");                                          // account for the third octal digit
    emitter.label("__rt_stripcslashes_octal_done_x86_64");
    emitter.instruction("mov BYTE PTR [r9], r11b");                             // write the octal value truncated to one byte
    emitter.instruction("add r9, 1");                                           // advance the output write pointer
    emitter.instruction("jmp __rt_stripcslashes_loop_x86_64");                  // continue scanning

    emitter.label("__rt_stripcslashes_hex_x86_64");
    emitter.instruction("add rax, 1");                                          // consume the 'x' selector
    emitter.instruction("sub rcx, 1");                                          // account for the 'x' selector
    emitter.instruction("test rcx, rcx");                                       // is any byte available after 'x'?
    emitter.instruction("je __rt_stripcslashes_hex_literal_x86_64");            // no digit after 'x' emits a literal 'x'
    emitter.instruction("movzx esi, BYTE PTR [rax]");                           // peek the first hex-digit candidate
    emitter.instruction("lea edi, [rsi - 48]");                                 // candidate - '0'
    emitter.instruction("cmp edi, 9");                                          // is it a decimal digit '0'..'9'?
    emitter.instruction("jbe __rt_stripcslashes_hex1_ok_x86_64");               // decimal digits map directly to their value
    emitter.instruction("mov edi, esi");                                        // reload the candidate for case folding
    emitter.instruction("or edi, 0x20");                                        // fold the candidate to lowercase
    emitter.instruction("sub edi, 97");                                         // folded - 'a'
    emitter.instruction("cmp edi, 5");                                          // is it in 'a'..'f'?
    emitter.instruction("ja __rt_stripcslashes_hex_literal_x86_64");            // no hex digit after 'x' emits a literal 'x'
    emitter.instruction("add edi, 10");                                         // hex letter value = 10 + (folded - 'a')
    emitter.label("__rt_stripcslashes_hex1_ok_x86_64");
    emitter.instruction("add rax, 1");                                          // consume the first hex digit
    emitter.instruction("sub rcx, 1");                                          // account for the first hex digit
    emitter.instruction("mov r11d, edi");                                       // accumulator = first hex nibble
    emitter.instruction("test rcx, rcx");                                       // is a second hex digit available?
    emitter.instruction("je __rt_stripcslashes_hex_emit_x86_64");               // stop after a single hex digit
    emitter.instruction("movzx esi, BYTE PTR [rax]");                           // peek the second hex-digit candidate
    emitter.instruction("lea edi, [rsi - 48]");                                 // candidate - '0'
    emitter.instruction("cmp edi, 9");                                          // is it a decimal digit '0'..'9'?
    emitter.instruction("jbe __rt_stripcslashes_hex2_ok_x86_64");               // decimal digits map directly to their value
    emitter.instruction("mov edi, esi");                                        // reload the candidate for case folding
    emitter.instruction("or edi, 0x20");                                        // fold the candidate to lowercase
    emitter.instruction("sub edi, 97");                                         // folded - 'a'
    emitter.instruction("cmp edi, 5");                                          // is it in 'a'..'f'?
    emitter.instruction("ja __rt_stripcslashes_hex_emit_x86_64");               // a non-hex second byte ends the escape
    emitter.instruction("add edi, 10");                                         // hex letter value = 10 + (folded - 'a')
    emitter.label("__rt_stripcslashes_hex2_ok_x86_64");
    emitter.instruction("shl r11d, 4");                                         // accumulator <<= 4 for the high nibble
    emitter.instruction("add r11d, edi");                                       // accumulator += second hex nibble
    emitter.instruction("add rax, 1");                                          // consume the second hex digit
    emitter.instruction("sub rcx, 1");                                          // account for the second hex digit
    emitter.label("__rt_stripcslashes_hex_emit_x86_64");
    emitter.instruction("mov BYTE PTR [r9], r11b");                             // write the decoded hex byte
    emitter.instruction("add r9, 1");                                           // advance the output write pointer
    emitter.instruction("jmp __rt_stripcslashes_loop_x86_64");                  // continue scanning
    emitter.label("__rt_stripcslashes_hex_literal_x86_64");
    emitter.instruction("mov BYTE PTR [r9], 120");                              // '\\x' with no hex digit emits a literal 'x'
    emitter.instruction("add r9, 1");                                           // advance the output write pointer
    emitter.instruction("jmp __rt_stripcslashes_loop_x86_64");                  // continue scanning

    emitter.label("__rt_stripcslashes_done_x86_64");
    emitter.instruction("mov rax, r10");                                        // result pointer = saved start
    emitter.instruction("mov rdx, r9");                                         // snapshot the final write pointer
    emitter.instruction("sub rdx, r10");                                        // result length = write pointer - start
    abi::emit_load_symbol_to_reg(emitter, "r8", "_concat_off", 0);             // reload the concat-buffer offset
    emitter.instruction("add r8, rdx");                                         // advance the offset by the result length
    abi::emit_store_reg_to_symbol(emitter, "r8", "_concat_off", 0);            // publish the updated concat-buffer offset
    emitter.instruction("ret");                                                 // return the decoded string slice
}
