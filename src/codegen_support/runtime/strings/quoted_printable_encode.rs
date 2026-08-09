//! Purpose:
//! Emits the `__rt_quoted_printable_encode` runtime helper assembly, a port of php-src's
//! `php_quot_print_encode` including its 75-column soft-line-break accounting.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - Three classes of byte, in php-src's order: an embedded `CRLF` pair is copied through
//!   verbatim and resets the column counter; a control byte, `0x7F`, any byte with the high
//!   bit set, `=` itself, or a space DIRECTLY BEFORE a `CR` becomes `=XX`; everything else is
//!   copied literally. A trailing space is therefore left alone (nothing follows it), while a
//!   trailing tab is a control byte and always becomes `=09`.
//! - The soft break is `=\r\n`, inserted BEFORE the byte that would cross column 75. php-src
//!   pre-charges the column counter by 3 and then adds a lookahead allowance for a UTF-8 lead
//!   byte (3 more for a 2-byte sequence, 6 for a 3-byte one, 9 for a 4-byte one) so a
//!   multi-byte character is not split across the break. Bytes above `0xF4` are never a valid
//!   lead byte and php-src's chain simply falls through without a break; that behavior is
//!   reproduced exactly rather than "fixed".
//! - Output storage comes from `__rt_concat_reserve`/`__rt_concat_publish`. The reservation is
//!   `4 * len + 8`, not php-src's `3 * len`: the measured worst case is 3.1 bytes per input
//!   byte (`str_repeat("=", 30)` encodes to 93 bytes) because a soft break adds 3 bytes that
//!   php-src's own bound does not account for.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_quoted_printable_encode` runtime helper.
///
/// ABI (AArch64): `x1` = subject pointer, `x2` = subject byte length; returns `x1`/`x2` =
/// encoded pointer/length.
///
/// Dispatches to `emit_quoted_printable_encode_linux_x86_64` on x86_64; uses inline AArch64
/// otherwise.
pub fn emit_quoted_printable_encode(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_quoted_printable_encode_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: quoted_printable_encode ---");
    emitter.label_global("__rt_quoted_printable_encode");

    // -- reserve worst-case storage before the first byte is classified --
    emitter.instruction("sub sp, sp, #32");                                     // allocate spill space for the borrowed subject string
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address across the reservation call
    emitter.instruction("add x29, sp, #16");                                    // establish the quoted-printable encoder frame pointer
    emitter.instruction("stp x1, x2, [sp]");                                    // save the subject pointer and length across the reservation call
    emitter.instruction("lsl x0, x2, #2");                                      // four bytes per input byte covers "=XX" plus its share of soft breaks
    emitter.instruction("add x0, x0, #8");                                      // add slack for a soft break emitted near the very end
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage for the encoded result
    emitter.instruction("mov x9, x0");                                          // keep the reservation start as the encoded string base
    emitter.instruction("mov x10, x0");                                         // seed the encoded-output write cursor
    emitter.instruction("ldp x1, x2, [sp]");                                    // reload the borrowed subject pointer and length
    emitter.instruction("mov x11, #0");                                         // lp: php-src's current output column counter

    emitter.label("__rt_qpenc_loop");
    emitter.instruction("cbz x2, __rt_qpenc_done");                             // stop once every subject byte has been classified
    emitter.instruction("ldrb w12, [x1], #1");                                  // load the current subject byte and advance the cursor
    emitter.instruction("sub x2, x2, #1");                                      // record that one subject byte has been consumed
    emitter.instruction("mov w13, #0");                                         // php-src reads the NUL terminator past the last byte
    emitter.instruction("cbz x2, __rt_qpenc_no_lookahead");                     // no following byte, so the lookahead stays zero
    emitter.instruction("ldrb w13, [x1]");                                      // peek the following byte without consuming it

    // -- an embedded CRLF pair is a hard line break and is copied through unchanged --
    emitter.label("__rt_qpenc_no_lookahead");
    emitter.instruction("cmp w12, #13");                                        // is the current byte a carriage return?
    emitter.instruction("b.ne __rt_qpenc_classify");                            // only CR can open a hard line break
    emitter.instruction("cbz x2, __rt_qpenc_classify");                         // a trailing CR has no LF to pair with
    emitter.instruction("cmp w13, #10");                                        // is the following byte a line feed?
    emitter.instruction("b.ne __rt_qpenc_classify");                            // a lone CR is encoded like any other control byte
    emitter.instruction("mov w14, #13");                                        // copy the carriage return verbatim
    emitter.instruction("strb w14, [x10], #1");                                 // write the carriage return to the output
    emitter.instruction("mov w14, #10");                                        // copy the line feed verbatim
    emitter.instruction("strb w14, [x10], #1");                                 // write the line feed to the output
    emitter.instruction("add x1, x1, #1");                                      // consume the line feed from the subject
    emitter.instruction("sub x2, x2, #1");                                      // record that the paired line feed was consumed
    emitter.instruction("mov x11, #0");                                         // a hard line break restarts the output column
    emitter.instruction("b __rt_qpenc_loop");                                   // classify the next subject byte

    // -- php-src's escape predicate, evaluated in its own order --
    emitter.label("__rt_qpenc_classify");
    emitter.instruction("cmp w12, #32");                                        // is the byte a C-locale control character?
    emitter.instruction("b.lo __rt_qpenc_encode");                              // control bytes are always escaped
    emitter.instruction("cmp w12, #127");                                       // is the byte DEL or does it have the high bit set?
    emitter.instruction("b.hs __rt_qpenc_encode");                              // DEL and every non-ASCII byte are always escaped
    emitter.instruction("cmp w12, #61");                                        // is the byte the '=' escape introducer itself?
    emitter.instruction("b.eq __rt_qpenc_encode");                              // '=' must be escaped or the output is ambiguous
    emitter.instruction("cmp w12, #32");                                        // is the byte a space?
    emitter.instruction("b.ne __rt_qpenc_literal");                             // any other printable byte is copied literally
    emitter.instruction("cmp w13, #13");                                        // does a carriage return follow this space?
    emitter.instruction("b.eq __rt_qpenc_encode");                              // a space at the end of a line must not be stripped in transit

    // -- literal byte: one column, with a soft break when it would cross column 75 --
    emitter.label("__rt_qpenc_literal");
    emitter.instruction("add x11, x11, #1");                                    // a literal byte occupies exactly one output column
    emitter.instruction("cmp x11, #75");                                        // would this byte still fit on the current line?
    emitter.instruction("b.ls __rt_qpenc_literal_write");                       // it fits, so no soft line break is needed
    emitter.instruction("mov w14, #61");                                        // a soft line break is written as "=\r\n"
    emitter.instruction("strb w14, [x10], #1");                                 // write the soft-break '='
    emitter.instruction("mov w14, #13");                                        // continue the soft break with a carriage return
    emitter.instruction("strb w14, [x10], #1");                                 // write the soft-break carriage return
    emitter.instruction("mov w14, #10");                                        // finish the soft break with a line feed
    emitter.instruction("strb w14, [x10], #1");                                 // write the soft-break line feed
    emitter.instruction("mov x11, #1");                                         // the moved byte is the first column of the new line

    emitter.label("__rt_qpenc_literal_write");
    emitter.instruction("strb w12, [x10], #1");                                 // write the literal subject byte to the output
    emitter.instruction("b __rt_qpenc_loop");                                   // classify the next subject byte

    // -- escaped byte: three columns, plus php-src's UTF-8 lookahead allowance --
    emitter.label("__rt_qpenc_encode");
    emitter.instruction("add x11, x11, #3");                                    // "=XX" occupies three output columns
    emitter.instruction("cmp w12, #127");                                       // is this an ASCII byte with no continuation bytes to keep together?
    emitter.instruction("b.hi __rt_qpenc_lead2");                               // a high-bit byte may lead a multi-byte character
    emitter.instruction("cmp x11, #75");                                        // would this escape still fit on the current line?
    emitter.instruction("b.hi __rt_qpenc_break");                               // break before the escape rather than past column 75
    emitter.instruction("b __rt_qpenc_write");                                  // the escape fits on the current line

    emitter.label("__rt_qpenc_lead2");
    emitter.instruction("cmp w12, #223");                                       // is this the lead byte of a two-byte UTF-8 sequence?
    emitter.instruction("b.hi __rt_qpenc_lead3");                               // no, try the three-byte lead range
    emitter.instruction("add x14, x11, #3");                                    // reserve room for the one continuation byte that follows
    emitter.instruction("cmp x14, #75");                                        // would the whole two-byte character still fit?
    emitter.instruction("b.hi __rt_qpenc_break");                               // break before the character rather than split it
    emitter.instruction("b __rt_qpenc_write");                                  // the whole character fits on the current line

    emitter.label("__rt_qpenc_lead3");
    emitter.instruction("cmp w12, #239");                                       // is this the lead byte of a three-byte UTF-8 sequence?
    emitter.instruction("b.hi __rt_qpenc_lead4");                               // no, try the four-byte lead range
    emitter.instruction("add x14, x11, #6");                                    // reserve room for the two continuation bytes that follow
    emitter.instruction("cmp x14, #75");                                        // would the whole three-byte character still fit?
    emitter.instruction("b.hi __rt_qpenc_break");                               // break before the character rather than split it
    emitter.instruction("b __rt_qpenc_write");                                  // the whole character fits on the current line

    emitter.label("__rt_qpenc_lead4");
    emitter.instruction("cmp w12, #244");                                       // is this the lead byte of a four-byte UTF-8 sequence?
    emitter.instruction("b.hi __rt_qpenc_write");                               // php-src never breaks for a byte above 0xF4
    emitter.instruction("add x14, x11, #9");                                    // reserve room for the three continuation bytes that follow
    emitter.instruction("cmp x14, #75");                                        // would the whole four-byte character still fit?
    emitter.instruction("b.ls __rt_qpenc_write");                               // the whole character fits on the current line

    emitter.label("__rt_qpenc_break");
    emitter.instruction("mov w14, #61");                                        // a soft line break is written as "=\r\n"
    emitter.instruction("strb w14, [x10], #1");                                 // write the soft-break '='
    emitter.instruction("mov w14, #13");                                        // continue the soft break with a carriage return
    emitter.instruction("strb w14, [x10], #1");                                 // write the soft-break carriage return
    emitter.instruction("mov w14, #10");                                        // finish the soft break with a line feed
    emitter.instruction("strb w14, [x10], #1");                                 // write the soft-break line feed
    emitter.instruction("mov x11, #3");                                         // the moved escape occupies the first three columns of the new line

    emitter.label("__rt_qpenc_write");
    emitter.instruction("mov w14, #61");                                        // every escape starts with '='
    emitter.instruction("strb w14, [x10], #1");                                 // write the escape introducer
    emitter.instruction("lsr w14, w12, #4");                                    // isolate the high nibble of the escaped byte
    emitter.instruction("cmp w14, #10");                                        // is the high nibble a decimal digit?
    emitter.instruction("b.lo __rt_qpenc_hi_digit");                            // digits map onto '0'-'9'
    emitter.instruction("add w14, w14, #55");                                   // map 10-15 onto the uppercase 'A'-'F' php-src uses
    emitter.instruction("b __rt_qpenc_hi_write");                               // the high nibble is ready to write

    emitter.label("__rt_qpenc_hi_digit");
    emitter.instruction("add w14, w14, #48");                                   // map 0-9 onto '0'-'9'

    emitter.label("__rt_qpenc_hi_write");
    emitter.instruction("strb w14, [x10], #1");                                 // write the high hex digit
    emitter.instruction("and w14, w12, #0xf");                                  // isolate the low nibble of the escaped byte
    emitter.instruction("cmp w14, #10");                                        // is the low nibble a decimal digit?
    emitter.instruction("b.lo __rt_qpenc_lo_digit");                            // digits map onto '0'-'9'
    emitter.instruction("add w14, w14, #55");                                   // map 10-15 onto the uppercase 'A'-'F' php-src uses
    emitter.instruction("b __rt_qpenc_lo_write");                               // the low nibble is ready to write

    emitter.label("__rt_qpenc_lo_digit");
    emitter.instruction("add w14, w14, #48");                                   // map 0-9 onto '0'-'9'

    emitter.label("__rt_qpenc_lo_write");
    emitter.instruction("strb w14, [x10], #1");                                 // write the low hex digit
    emitter.instruction("b __rt_qpenc_loop");                                   // classify the next subject byte

    emitter.label("__rt_qpenc_done");
    emitter.instruction("mov x1, x9");                                          // return the encoded payload pointer
    emitter.instruction("sub x2, x10, x9");                                     // return the number of encoded bytes actually written
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the quoted-printable encoder frame
    emitter.instruction("ret");                                                 // return the encoded string pair
}

/// Emits the `__rt_quoted_printable_encode` runtime helper for the Linux x86_64 target.
///
/// ABI (x86_64): `rax` = subject pointer, `rdx` = subject byte length; returns `rax`/`rdx` =
/// encoded pointer/length.
///
/// Same classification order and column accounting as the AArch64 path.
/// Called exclusively from `emit_quoted_printable_encode` when
/// `emitter.target.arch == Arch::X86_64`.
fn emit_quoted_printable_encode_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: quoted_printable_encode ---");
    emitter.label_global("__rt_quoted_printable_encode");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the reservation and publish calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the borrowed subject string
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spill slots for the subject pointer and length
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the subject pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the subject length across the reservation call
    emitter.instruction("mov rax, rdx");                                        // start the reservation size from the subject length
    emitter.instruction("shl rax, 2");                                          // four bytes per input byte covers "=XX" plus its share of soft breaks
    emitter.instruction("add rax, 8");                                          // add slack for a soft break emitted near the very end
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage for the encoded result
    emitter.instruction("mov r9, rax");                                         // keep the reservation start as the encoded string base
    emitter.instruction("mov r10, rax");                                        // seed the encoded-output write cursor
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // reload the borrowed subject pointer into the read cursor
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // reload the subject length into the loop counter
    emitter.instruction("xor r11d, r11d");                                      // lp: php-src's current output column counter

    emitter.label("__rt_qpenc_loop_x86");
    emitter.instruction("test rcx, rcx");                                       // stop once every subject byte has been classified
    emitter.instruction("jz __rt_qpenc_done_x86");                              // leave the loop at the end of the subject
    emitter.instruction("movzx eax, BYTE PTR [rsi]");                           // load the current subject byte and widen it for comparisons
    emitter.instruction("add rsi, 1");                                          // advance the subject read cursor
    emitter.instruction("sub rcx, 1");                                          // record that one subject byte has been consumed
    emitter.instruction("xor r8d, r8d");                                        // php-src reads the NUL terminator past the last byte
    emitter.instruction("test rcx, rcx");                                       // is there a following byte to peek at?
    emitter.instruction("jz __rt_qpenc_no_lookahead_x86");                      // no following byte, so the lookahead stays zero
    emitter.instruction("movzx r8d, BYTE PTR [rsi]");                           // peek the following byte without consuming it

    emitter.label("__rt_qpenc_no_lookahead_x86");
    emitter.instruction("cmp eax, 13");                                         // is the current byte a carriage return?
    emitter.instruction("jne __rt_qpenc_classify_x86");                         // only CR can open a hard line break
    emitter.instruction("test rcx, rcx");                                       // is there a following byte at all?
    emitter.instruction("jz __rt_qpenc_classify_x86");                          // a trailing CR has no LF to pair with
    emitter.instruction("cmp r8d, 10");                                         // is the following byte a line feed?
    emitter.instruction("jne __rt_qpenc_classify_x86");                         // a lone CR is encoded like any other control byte
    emitter.instruction("mov BYTE PTR [r10], 13");                              // copy the carriage return verbatim
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the carriage return
    emitter.instruction("mov BYTE PTR [r10], 10");                              // copy the line feed verbatim
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the line feed
    emitter.instruction("add rsi, 1");                                          // consume the line feed from the subject
    emitter.instruction("sub rcx, 1");                                          // record that the paired line feed was consumed
    emitter.instruction("xor r11d, r11d");                                      // a hard line break restarts the output column
    emitter.instruction("jmp __rt_qpenc_loop_x86");                             // classify the next subject byte

    emitter.label("__rt_qpenc_classify_x86");
    emitter.instruction("cmp eax, 32");                                         // is the byte a C-locale control character?
    emitter.instruction("jb __rt_qpenc_encode_x86");                            // control bytes are always escaped
    emitter.instruction("cmp eax, 127");                                        // is the byte DEL or does it have the high bit set?
    emitter.instruction("jae __rt_qpenc_encode_x86");                           // DEL and every non-ASCII byte are always escaped
    emitter.instruction("cmp eax, 61");                                         // is the byte the '=' escape introducer itself?
    emitter.instruction("je __rt_qpenc_encode_x86");                            // '=' must be escaped or the output is ambiguous
    emitter.instruction("cmp eax, 32");                                         // is the byte a space?
    emitter.instruction("jne __rt_qpenc_literal_x86");                          // any other printable byte is copied literally
    emitter.instruction("cmp r8d, 13");                                         // does a carriage return follow this space?
    emitter.instruction("je __rt_qpenc_encode_x86");                            // a space at the end of a line must not be stripped in transit

    emitter.label("__rt_qpenc_literal_x86");
    emitter.instruction("add r11, 1");                                          // a literal byte occupies exactly one output column
    emitter.instruction("cmp r11, 75");                                         // would this byte still fit on the current line?
    emitter.instruction("jbe __rt_qpenc_literal_write_x86");                    // it fits, so no soft line break is needed
    emitter.instruction("mov BYTE PTR [r10], 61");                              // write the soft-break '='
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the soft-break '='
    emitter.instruction("mov BYTE PTR [r10], 13");                              // write the soft-break carriage return
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the soft-break carriage return
    emitter.instruction("mov BYTE PTR [r10], 10");                              // write the soft-break line feed
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the soft-break line feed
    emitter.instruction("mov r11, 1");                                          // the moved byte is the first column of the new line

    emitter.label("__rt_qpenc_literal_write_x86");
    emitter.instruction("mov BYTE PTR [r10], al");                              // write the literal subject byte to the output
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the literal byte
    emitter.instruction("jmp __rt_qpenc_loop_x86");                             // classify the next subject byte

    emitter.label("__rt_qpenc_encode_x86");
    emitter.instruction("add r11, 3");                                          // "=XX" occupies three output columns
    emitter.instruction("cmp eax, 127");                                        // is this an ASCII byte with no continuation bytes to keep together?
    emitter.instruction("ja __rt_qpenc_lead2_x86");                             // a high-bit byte may lead a multi-byte character
    emitter.instruction("cmp r11, 75");                                         // would this escape still fit on the current line?
    emitter.instruction("ja __rt_qpenc_break_x86");                             // break before the escape rather than past column 75
    emitter.instruction("jmp __rt_qpenc_write_x86");                            // the escape fits on the current line

    emitter.label("__rt_qpenc_lead2_x86");
    emitter.instruction("cmp eax, 223");                                        // is this the lead byte of a two-byte UTF-8 sequence?
    emitter.instruction("ja __rt_qpenc_lead3_x86");                             // no, try the three-byte lead range
    emitter.instruction("mov rdx, r11");                                        // copy the column counter before adding the lookahead allowance
    emitter.instruction("add rdx, 3");                                          // reserve room for the one continuation byte that follows
    emitter.instruction("cmp rdx, 75");                                         // would the whole two-byte character still fit?
    emitter.instruction("ja __rt_qpenc_break_x86");                             // break before the character rather than split it
    emitter.instruction("jmp __rt_qpenc_write_x86");                            // the whole character fits on the current line

    emitter.label("__rt_qpenc_lead3_x86");
    emitter.instruction("cmp eax, 239");                                        // is this the lead byte of a three-byte UTF-8 sequence?
    emitter.instruction("ja __rt_qpenc_lead4_x86");                             // no, try the four-byte lead range
    emitter.instruction("mov rdx, r11");                                        // copy the column counter before adding the lookahead allowance
    emitter.instruction("add rdx, 6");                                          // reserve room for the two continuation bytes that follow
    emitter.instruction("cmp rdx, 75");                                         // would the whole three-byte character still fit?
    emitter.instruction("ja __rt_qpenc_break_x86");                             // break before the character rather than split it
    emitter.instruction("jmp __rt_qpenc_write_x86");                            // the whole character fits on the current line

    emitter.label("__rt_qpenc_lead4_x86");
    emitter.instruction("cmp eax, 244");                                        // is this the lead byte of a four-byte UTF-8 sequence?
    emitter.instruction("ja __rt_qpenc_write_x86");                             // php-src never breaks for a byte above 0xF4
    emitter.instruction("mov rdx, r11");                                        // copy the column counter before adding the lookahead allowance
    emitter.instruction("add rdx, 9");                                          // reserve room for the three continuation bytes that follow
    emitter.instruction("cmp rdx, 75");                                         // would the whole four-byte character still fit?
    emitter.instruction("jbe __rt_qpenc_write_x86");                            // the whole character fits on the current line

    emitter.label("__rt_qpenc_break_x86");
    emitter.instruction("mov BYTE PTR [r10], 61");                              // write the soft-break '='
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the soft-break '='
    emitter.instruction("mov BYTE PTR [r10], 13");                              // write the soft-break carriage return
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the soft-break carriage return
    emitter.instruction("mov BYTE PTR [r10], 10");                              // write the soft-break line feed
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the soft-break line feed
    emitter.instruction("mov r11, 3");                                          // the moved escape occupies the first three columns of the new line

    emitter.label("__rt_qpenc_write_x86");
    emitter.instruction("mov BYTE PTR [r10], 61");                              // write the escape introducer
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the escape introducer
    emitter.instruction("mov edx, eax");                                        // copy the escaped byte before isolating its high nibble
    emitter.instruction("shr edx, 4");                                          // isolate the high nibble of the escaped byte
    emitter.instruction("cmp edx, 10");                                         // is the high nibble a decimal digit?
    emitter.instruction("jb __rt_qpenc_hi_digit_x86");                          // digits map onto '0'-'9'
    emitter.instruction("add edx, 55");                                         // map 10-15 onto the uppercase 'A'-'F' php-src uses
    emitter.instruction("jmp __rt_qpenc_hi_write_x86");                         // the high nibble is ready to write

    emitter.label("__rt_qpenc_hi_digit_x86");
    emitter.instruction("add edx, 48");                                         // map 0-9 onto '0'-'9'

    emitter.label("__rt_qpenc_hi_write_x86");
    emitter.instruction("mov BYTE PTR [r10], dl");                              // write the high hex digit
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the high hex digit
    emitter.instruction("mov edx, eax");                                        // copy the escaped byte before isolating its low nibble
    emitter.instruction("and edx, 15");                                         // isolate the low nibble of the escaped byte
    emitter.instruction("cmp edx, 10");                                         // is the low nibble a decimal digit?
    emitter.instruction("jb __rt_qpenc_lo_digit_x86");                          // digits map onto '0'-'9'
    emitter.instruction("add edx, 55");                                         // map 10-15 onto the uppercase 'A'-'F' php-src uses
    emitter.instruction("jmp __rt_qpenc_lo_write_x86");                         // the low nibble is ready to write

    emitter.label("__rt_qpenc_lo_digit_x86");
    emitter.instruction("add edx, 48");                                         // map 0-9 onto '0'-'9'

    emitter.label("__rt_qpenc_lo_write_x86");
    emitter.instruction("mov BYTE PTR [r10], dl");                              // write the low hex digit
    emitter.instruction("add r10, 1");                                          // advance the output cursor past the low hex digit
    emitter.instruction("jmp __rt_qpenc_loop_x86");                             // classify the next subject byte

    emitter.label("__rt_qpenc_done_x86");
    emitter.instruction("mov rax, r9");                                         // return the encoded payload pointer
    emitter.instruction("mov rdx, r10");                                        // copy the output cursor into the length scratch register
    emitter.instruction("sub rdx, r9");                                         // return the number of encoded bytes actually written
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("add rsp, 32");                                         // release the quoted-printable encoder spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the encoded string pair
}
