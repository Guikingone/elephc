//! Purpose:
//! Emits the `__rt_fgetcsv` runtime helper assembly for CSV row parsing.
//! Supports custom separator, enclosure, and escape characters passed as a
//! packed `csv_opts` word from the EIR lowering.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - ARM64 and x86_64 variants share the same CSV state machine.
//! - `csv_opts = (esc << 16) | (enc << 8) | sep`; zero bytes select defaults
//!   (sep → ',', enc → '"', esc → 0 means RFC 4180 doubling mode).

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits the `__rt_fgetcsv` runtime helper, dispatching to the target-specific variant.
pub fn emit_fgetcsv(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_fgetcsv_linux_x86_64(emitter);
        return;
    }
    emit_fgetcsv_aarch64(emitter);
}

/// ARM64 variant of `__rt_fgetcsv`.
///
/// Signature: `__rt_fgetcsv(fd: x0, csv_opts: x1) -> array_ptr: x0`.
/// Returns `0` on EOF (PHP false), otherwise a heap array of owned string fields.
/// Supports RFC 4180 doubling mode (`esc == 0`) and escape-char mode (`esc != 0`).
fn emit_fgetcsv_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: fgetcsv ---");
    emitter.label_global("__rt_fgetcsv");

    // -- prologue: save callee-saved registers + fp/lr (112-byte frame) --
    emitter.instruction("stp x29, x30, [sp, #-112]!");                          // save fp, lr; allocate 112-byte frame
    emitter.instruction("stp x19, x20, [sp, #16]");                             // save x19 (temp/len), x20 (array_ptr)
    emitter.instruction("stp x21, x22, [sp, #32]");                             // save x21 (scan_ptr), x22 (end_ptr)
    emitter.instruction("stp x23, x24, [sp, #48]");                             // save x23 (field_start), x24 (write_ptr)
    emitter.instruction("stp x25, x26, [sp, #64]");                             // save w25 (state), w26 (sep)
    emitter.instruction("stp x27, x28, [sp, #80]");                             // save w27 (enc), w28 (esc)
    emitter.instruction("add x29, sp, #0");                                    // establish frame pointer

    // -- unpack csv_opts: sep=w1&0xFF, enc=(w1>>8)&0xFF, esc=(w1>>16)&0xFF --
    emitter.instruction("and w26, w1, #0xff");                                  // sep = csv_opts & 0xFF
    emitter.instruction("lsr w2, w1, #8");                                      // shift right 8 for enc field
    emitter.instruction("and w27, w2, #0xff");                                  // enc = (csv_opts >> 8) & 0xFF
    emitter.instruction("lsr w2, w1, #16");                                     // shift right 16 for esc field
    emitter.instruction("and w28, w2, #0xff");                                  // esc = (csv_opts >> 16) & 0xFF

    // -- apply defaults: sep==0 -> ',', enc==0 -> '"' --
    emitter.instruction("cbnz w26, __rt_fgetcsv_sep_done");                     // if sep != 0, skip default
    emitter.instruction("mov w26, #0x2c");                                       // sep = ',' (0x2C)
    emitter.label("__rt_fgetcsv_sep_done");
    emitter.instruction("cbnz w27, __rt_fgetcsv_enc_done");                     // if enc != 0, skip default
    emitter.instruction("mov w27, #0x22");                                      // enc = 0x22 double-quote
    emitter.label("__rt_fgetcsv_enc_done");

    // -- read one line via __rt_fgets -> x1=ptr, x2=len --
    emitter.instruction("bl __rt_fgets");                                       // x1 = line ptr, x2 = line len

    // -- EOF check: len == 0 -> return 0 (false) --
    emitter.instruction("cbz x2, __rt_fgetcsv_eof");                            // len == 0 -> EOF, return 0

    // -- set up scan pointers into the line buffer --
    emitter.instruction("mov x21, x1");                                         // scan_ptr = line_ptr
    emitter.instruction("add x22, x1, x2");                                     // end_ptr = ptr + len

    // -- create result array: cap=8, elem_size=16 (ptr+len pair) --
    emitter.instruction("mov x0, #8");                                          // capacity = 8 fields
    emitter.instruction("mov x1, #16");                                         // elem_size = 16 (ptr + len pair)
    emitter.instruction("bl __rt_array_new");                                   // x0 = new array ptr
    emitter.instruction("mov x20, x0");                                         // array_ptr = result

    // -- init field tracking: field_start = write_ptr = scan_ptr, state = 0 --
    emitter.instruction("mov x23, x21");                                        // field_start = scan_ptr
    emitter.instruction("mov x24, x21");                                        // write_ptr = scan_ptr
    emitter.instruction("mov w25, #0");                                         // state = 0 (OutsideField)

    // -- main parse loop --
    emitter.label("__rt_fgetcsv_loop");
    emitter.instruction("cmp x21, x22");                                       // scan_ptr >= end_ptr?
    emitter.instruction("b.ge __rt_fgetcsv_end_line");                          // yes -> push last field, return
    emitter.instruction("ldrb w0, [x21], #1");                                  // c = *scan_ptr++; (zero-extended)

    // -- dispatch on state (w25: 0..4) --
    emitter.instruction("cmp w25, #0");                                         // state == OutsideField?
    emitter.instruction("b.eq __rt_fgetcsv_st0");                               // -> state 0 handler
    emitter.instruction("cmp w25, #1");                                         // state == InField?
    emitter.instruction("b.eq __rt_fgetcsv_st1");                               // -> state 1 handler
    emitter.instruction("cmp w25, #2");                                         // state == InQuotedField?
    emitter.instruction("b.eq __rt_fgetcsv_st2");                               // -> state 2 handler
    emitter.instruction("cmp w25, #3");                                         // state == AfterEscape?
    emitter.instruction("b.eq __rt_fgetcsv_st3");                               // -> state 3 handler
    emitter.instruction("cmp w25, #4");                                         // state == AfterCloseQuote?
    emitter.instruction("b.eq __rt_fgetcsv_st4");                               // -> state 4 handler
    emitter.instruction("b __rt_fgetcsv_end_line");                             // unknown state -> safety exit

    // -- state 0: OutsideField --
    emitter.label("__rt_fgetcsv_st0");
    emitter.instruction("cmp w0, w26");                                         // c == sep?
    emitter.instruction("b.eq __rt_fgetcsv_push_reset");                         // -> push empty field, reset
    emitter.instruction("cmp w0, w27");                                         // c == enc (opening quote)?
    emitter.instruction("b.eq __rt_fgetcsv_s0_enc");                            // -> enter quoted field
    emitter.instruction("cmp w0, #0x0a");                                      // c == newline (0x0A)?
    emitter.instruction("b.eq __rt_fgetcsv_push_end");                          // -> push empty field, end
    emitter.instruction("cmp w0, #0x0d");                                      // c == carriage return (0x0D)?
    emitter.instruction("b.eq __rt_fgetcsv_push_end");                          // -> push empty field, end
    emitter.instruction("mov x23, x24");                                       // field_start = write_ptr
    emitter.instruction("strb w0, [x24], #1");                                 // *write_ptr++ = c (accumulate)
    emitter.instruction("mov w25, #1");                                         // state = InField
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    emitter.label("__rt_fgetcsv_s0_enc");
    emitter.instruction("mov x23, x24");                                       // field_start = write_ptr (skip opening quote)
    emitter.instruction("mov w25, #2");                                         // state = InQuotedField
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    // -- state 1: InField (unquoted, accumulating) --
    emitter.label("__rt_fgetcsv_st1");
    emitter.instruction("cmp w0, w26");                                         // c == sep?
    emitter.instruction("b.eq __rt_fgetcsv_push_reset");                        // -> push field, reset
    emitter.instruction("cmp w0, #0x0a");                                       // c == newline (0x0A)?
    emitter.instruction("b.eq __rt_fgetcsv_push_end");                         // -> push field, end
    emitter.instruction("cmp w0, #0x0d");                                       // c == carriage return (0x0D)?
    emitter.instruction("b.eq __rt_fgetcsv_push_end");                         // -> push field, end
    emitter.instruction("strb w0, [x24], #1");                                 // *write_ptr++ = c (accumulate)
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    // -- state 2: InQuotedField --
    emitter.label("__rt_fgetcsv_st2");
    emitter.instruction("cbz w28, __rt_fgetcsv_s2_chkenc");                     // esc == 0 -> doubling mode, skip esc check
    emitter.instruction("cmp w0, w28");                                         // c == esc?
    emitter.instruction("b.eq __rt_fgetcsv_s2_esc");                           // -> AfterEscape
    emitter.label("__rt_fgetcsv_s2_chkenc");
    emitter.instruction("cmp w0, w27");                                         // c == enc (close quote)?
    emitter.instruction("b.eq __rt_fgetcsv_s2_close");                          // -> AfterCloseQuote
    emitter.instruction("strb w0, [x24], #1");                                 // *write_ptr++ = c (accumulate)
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    emitter.label("__rt_fgetcsv_s2_esc");
    emitter.instruction("mov w25, #3");                                         // state = AfterEscape
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    emitter.label("__rt_fgetcsv_s2_close");
    emitter.instruction("mov w25, #4");                                         // state = AfterCloseQuote
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    // -- state 3: AfterEscape (esc mode only) --
    emitter.label("__rt_fgetcsv_st3");
    emitter.instruction("cmp w0, w27");                                         // c == enc?
    emitter.instruction("b.eq __rt_fgetcsv_s3_enc");                            // -> write c only (drop esc)
    emitter.instruction("strb w28, [x24], #1");                                 // *write_ptr++ = esc (keep esc for non-enc)
    emitter.label("__rt_fgetcsv_s3_enc");
    emitter.instruction("strb w0, [x24], #1");                                  // *write_ptr++ = c (literal)
    emitter.instruction("mov w25, #2");                                         // state = InQuotedField
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    // -- state 4: AfterCloseQuote --
    emitter.label("__rt_fgetcsv_st4");
    emitter.instruction("cmp w0, w27");                                         // c == enc (doubled quote)?
    emitter.instruction("b.eq __rt_fgetcsv_s4_dbl");                           // -> accumulate enc, back to quoted
    emitter.instruction("cmp w0, w26");                                         // c == sep?
    emitter.instruction("b.eq __rt_fgetcsv_push_reset");                        // -> push field, reset
    emitter.instruction("cmp w0, #0x0a");                                      // c == newline (0x0A)?
    emitter.instruction("b.eq __rt_fgetcsv_push_end");                         // -> push field, end
    emitter.instruction("cmp w0, #0x0d");                                      // c == carriage return (0x0D)?
    emitter.instruction("b.eq __rt_fgetcsv_push_end");                         // -> push field, end
    emitter.instruction("strb w27, [x24], #1");                                // *write_ptr++ = enc (restore close quote)
    emitter.instruction("strb w0, [x24], #1");                                 // *write_ptr++ = c (accumulate)
    emitter.instruction("mov w25, #1");                                         // state = InField
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    emitter.label("__rt_fgetcsv_s4_dbl");
    emitter.instruction("strb w27, [x24], #1");                                 // *write_ptr++ = enc (doubled -> single quote)
    emitter.instruction("mov w25, #2");                                         // state = InQuotedField
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    // -- push field and reset for next field (separator encountered) --
    emitter.label("__rt_fgetcsv_push_reset");
    emitter.instruction("sub x19, x24, x23");                                  // x19 = len = write_ptr - field_start
    emitter.instruction("mov x1, x23");                                         // ptr = field_start (raw slice into line buf)
    emitter.instruction("mov x2, x19");                                         // len
    emitter.instruction("bl __rt_str_persist");                                 // x0 = persisted string (heap copy)
    emitter.instruction("mov x1, x0");                                         // x1 = persisted string ptr
    emitter.instruction("mov x0, x20");                                         // x0 = array_ptr
    emitter.instruction("mov x2, x19");                                         // x2 = len (restored from callee-saved x19)
    emitter.instruction("bl __rt_array_push_str");                              // x0 = array_ptr (possibly reallocated)
    emitter.instruction("mov x20, x0");                                         // update array_ptr
    emitter.instruction("mov x23, x24");                                        // field_start = write_ptr (next field)
    emitter.instruction("mov w25, #0");                                         // state = OutsideField
    emitter.instruction("b __rt_fgetcsv_loop");                                 // continue loop

    // -- push field and end (newline or end-of-buffer) --
    emitter.label("__rt_fgetcsv_push_end");
    emitter.instruction("sub x19, x24, x23");                                  // x19 = len = write_ptr - field_start
    emitter.instruction("mov x1, x23");                                         // ptr = field_start (raw slice into line buf)
    emitter.instruction("mov x2, x19");                                         // len
    emitter.instruction("bl __rt_str_persist");                                 // x0 = persisted string (heap copy)
    emitter.instruction("mov x1, x0");                                         // x1 = persisted string ptr
    emitter.instruction("mov x0, x20");                                         // x0 = array_ptr
    emitter.instruction("mov x2, x19");                                         // x2 = len (restored from callee-saved x19)
    emitter.instruction("bl __rt_array_push_str");                              // x0 = array_ptr (possibly reallocated)
    emitter.instruction("mov x20, x0");                                        // update array_ptr
    emitter.instruction("b __rt_fgetcsv_done");                                 // -> epilogue

    // -- end of line (scan_ptr reached end_ptr without trailing newline) --
    emitter.label("__rt_fgetcsv_end_line");
    emitter.instruction("b __rt_fgetcsv_push_end");                            // push last field, then return

    // -- done: return array_ptr in x0 --
    emitter.label("__rt_fgetcsv_done");
    emitter.instruction("mov x0, x20");                                         // x0 = array_ptr (return value)
    emitter.instruction("b __rt_fgetcsv_epilogue");                             // -> common epilogue

    // -- EOF: return 0 (false) --
    emitter.label("__rt_fgetcsv_eof");
    emitter.instruction("mov x0, #0");                                         // x0 = 0 (false / EOF)

    // -- epilogue: restore registers and return --
    emitter.label("__rt_fgetcsv_epilogue");
    emitter.instruction("ldp x19, x20, [sp, #16]");                             // restore x19, x20
    emitter.instruction("ldp x21, x22, [sp, #32]");                             // restore x21, x22
    emitter.instruction("ldp x23, x24, [sp, #48]");                             // restore x23, x24
    emitter.instruction("ldp x25, x26, [sp, #64]");                             // restore x25, x26
    emitter.instruction("ldp x27, x28, [sp, #80]");                             // restore x27, x28
    emitter.instruction("ldp x29, x30, [sp], #112");                            // restore fp, lr; deallocate frame
    emitter.instruction("ret");                                                // return to caller
}

/// x86_64 Linux variant of `__rt_fgetcsv` using the System V ABI.
///
/// Signature: `__rt_fgetcsv(fd: rdi, csv_opts: rsi) -> array_ptr: rax`.
/// Mirrors the ARM64 state machine; spills parser state to a rbp-relative frame.
fn emit_fgetcsv_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: fgetcsv ---");
    emitter.label_global("__rt_fgetcsv");

    // -- prologue: 96-byte frame with rbp --
    emitter.instruction("push rbp");                                            // preserve caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish frame base
    emitter.instruction("sub rsp, 96");                                         // reserve 96 bytes for parser state
    emitter.instruction("push rbx");                                            // save rbx (callee-saved, used for array_ptr)
    emitter.instruction("push r12");                                            // save r12 (scan_ptr)
    emitter.instruction("push r13");                                            // save r13 (end_ptr)
    emitter.instruction("push r14");                                            // save r14 (field_start)
    emitter.instruction("push r15");                                            // save r15 (write_ptr)

    // -- unpack csv_opts from rsi: sep=rsi&0xFF, enc=(rsi>>8)&0xFF, esc=(rsi>>16)&0xFF --
    emitter.instruction("movzx edx, sil");                                     // sep = csv_opts & 0xFF
    emitter.instruction("mov [rbp - 8], rdx");                                  // save sep at [rbp-8]
    emitter.instruction("shr rsi, 8");                                          // shift right 8 for enc
    emitter.instruction("movzx edx, sil");                                      // enc = (csv_opts >> 8) & 0xFF
    emitter.instruction("mov [rbp - 16], rdx");                                 // save enc at [rbp-16]
    emitter.instruction("shr rsi, 8");                                          // shift right 8 more for esc
    emitter.instruction("movzx edx, sil");                                      // esc = (csv_opts >> 16) & 0xFF
    emitter.instruction("mov [rbp - 24], rdx");                                 // save esc at [rbp-24]

    // -- apply defaults: sep==0 -> ',', enc==0 -> '"' --
    emitter.instruction("cmp QWORD PTR [rbp - 8], 0");                          // sep == 0?
    emitter.instruction("jne __rt_fgetcsv_x_sep_done");                         // if not, skip default
    emitter.instruction("mov QWORD PTR [rbp - 8], 0x2c");                       // sep = ',' (0x2C)
    emitter.label("__rt_fgetcsv_x_sep_done");
    emitter.instruction("cmp QWORD PTR [rbp - 16], 0");                         // enc == 0?
    emitter.instruction("jne __rt_fgetcsv_x_enc_done");                          // if not, skip default
    emitter.instruction("mov QWORD PTR [rbp - 16], 0x22");                      // enc = '"' (0x22)
    emitter.label("__rt_fgetcsv_x_enc_done");

    // -- read one line via __rt_fgets -> rax=ptr, rdx=len --
    emitter.instruction("call __rt_fgets");                                    // rax = line ptr, rdx = line len

    // -- EOF check: len == 0 -> return 0 (false) --
    emitter.instruction("test rdx, rdx");                                       // len == 0?
    emitter.instruction("jz __rt_fgetcsv_x_eof");                               // -> EOF, return 0

    // -- set up scan pointers --
    emitter.instruction("mov r12, rax");                                        // scan_ptr = line_ptr
    emitter.instruction("mov r13, rax");                                        // save line_ptr for end_ptr calc
    emitter.instruction("add r13, rdx");                                        // end_ptr = ptr + len

    // -- create result array: cap=8, elem_size=16 --
    emitter.instruction("mov edi, 8");                                          // capacity = 8 fields
    emitter.instruction("mov esi, 16");                                         // elem_size = 16 (ptr + len pair)
    emitter.instruction("call __rt_array_new");                                  // rax = new array ptr
    emitter.instruction("mov rbx, rax");                                        // array_ptr = result

    // -- init field tracking: field_start = write_ptr = scan_ptr, state = 0 --
    emitter.instruction("mov r14, r12");                                        // field_start = scan_ptr
    emitter.instruction("mov r15, r12");                                        // write_ptr = scan_ptr
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                         // state = 0 (OutsideField)

    // -- main parse loop --
    emitter.label("__rt_fgetcsv_x_loop");
    emitter.instruction("cmp r12, r13");                                        // scan_ptr >= end_ptr?
    emitter.instruction("jae __rt_fgetcsv_x_end_line");                          // yes -> push last field, return
    emitter.instruction("movzx eax, BYTE PTR [r12]");                            // c = *scan_ptr (zero-extended)
    emitter.instruction("add r12, 1");                                          // scan_ptr++

    // -- dispatch on state ([rbp-32]: 0..4) --
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // load state
    emitter.instruction("cmp rcx, 0");                                          // state == OutsideField?
    emitter.instruction("je __rt_fgetcsv_x_st0");                               // -> state 0 handler
    emitter.instruction("cmp rcx, 1");                                          // state == InField?
    emitter.instruction("je __rt_fgetcsv_x_st1");                               // -> state 1 handler
    emitter.instruction("cmp rcx, 2");                                          // state == InQuotedField?
    emitter.instruction("je __rt_fgetcsv_x_st2");                               // -> state 2 handler
    emitter.instruction("cmp rcx, 3");                                          // state == AfterEscape?
    emitter.instruction("je __rt_fgetcsv_x_st3");                               // -> state 3 handler
    emitter.instruction("cmp rcx, 4");                                          // state == AfterCloseQuote?
    emitter.instruction("je __rt_fgetcsv_x_st4");                               // -> state 4 handler
    emitter.instruction("jmp __rt_fgetcsv_x_end_line");                         // unknown state -> safety exit

    // -- state 0: OutsideField --
    emitter.label("__rt_fgetcsv_x_st0");
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 8]");                        // load sep
    emitter.instruction("cmp al, cl");                                          // c == sep?
    emitter.instruction("je __rt_fgetcsv_x_push_reset");                        // -> push empty field, reset
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 16]");                      // load enc
    emitter.instruction("cmp al, cl");                                          // c == enc (opening quote)?
    emitter.instruction("je __rt_fgetcsv_x_s0_enc");                            // -> enter quoted field
    emitter.instruction("cmp al, 0x0a");                                        // c == newline (0x0A)?
    emitter.instruction("je __rt_fgetcsv_x_push_end");                         // -> push empty field, end
    emitter.instruction("cmp al, 0x0d");                                        // c == carriage return (0x0D)?
    emitter.instruction("je __rt_fgetcsv_x_push_end");                         // -> push empty field, end
    emitter.instruction("mov r14, r15");                                        // field_start = write_ptr
    emitter.instruction("mov BYTE PTR [r15], al");                              // *write_ptr = c
    emitter.instruction("add r15, 1");                                          // write_ptr++
    emitter.instruction("mov QWORD PTR [rbp - 32], 1");                        // state = InField
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    emitter.label("__rt_fgetcsv_x_s0_enc");
    emitter.instruction("mov r14, r15");                                        // field_start = write_ptr (skip opening quote)
    emitter.instruction("mov QWORD PTR [rbp - 32], 2");                        // state = InQuotedField
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    // -- state 1: InField (unquoted, accumulating) --
    emitter.label("__rt_fgetcsv_x_st1");
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 8]");                        // load sep
    emitter.instruction("cmp al, cl");                                          // c == sep?
    emitter.instruction("je __rt_fgetcsv_x_push_reset");                        // -> push field, reset
    emitter.instruction("cmp al, 0x0a");                                        // c == newline (0x0A)?
    emitter.instruction("je __rt_fgetcsv_x_push_end");                         // -> push field, end
    emitter.instruction("cmp al, 0x0d");                                        // c == carriage return (0x0D)?
    emitter.instruction("je __rt_fgetcsv_x_push_end");                         // -> push field, end
    emitter.instruction("mov BYTE PTR [r15], al");                              // *write_ptr = c
    emitter.instruction("add r15, 1");                                          // write_ptr++
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    // -- state 2: InQuotedField --
    emitter.label("__rt_fgetcsv_x_st2");
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 24]");                      // load esc
    emitter.instruction("test cl, cl");                                         // esc == 0?
    emitter.instruction("jz __rt_fgetcsv_x_s2_chkenc");                          // -> doubling mode, skip esc check
    emitter.instruction("cmp al, cl");                                          // c == esc?
    emitter.instruction("je __rt_fgetcsv_x_s2_esc");                            // -> AfterEscape
    emitter.label("__rt_fgetcsv_x_s2_chkenc");
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 16]");                      // load enc
    emitter.instruction("cmp al, cl");                                          // c == enc (close quote)?
    emitter.instruction("je __rt_fgetcsv_x_s2_close");                          // -> AfterCloseQuote
    emitter.instruction("mov BYTE PTR [r15], al");                              // *write_ptr = c
    emitter.instruction("add r15, 1");                                          // write_ptr++
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    emitter.label("__rt_fgetcsv_x_s2_esc");
    emitter.instruction("mov QWORD PTR [rbp - 32], 3");                        // state = AfterEscape
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    emitter.label("__rt_fgetcsv_x_s2_close");
    emitter.instruction("mov QWORD PTR [rbp - 32], 4");                        // state = AfterCloseQuote
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    // -- state 3: AfterEscape (esc mode only) --
    emitter.label("__rt_fgetcsv_x_st3");
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 16]");                      // load enc
    emitter.instruction("cmp al, cl");                                          // c == enc?
    emitter.instruction("je __rt_fgetcsv_x_s3_enc");                           // -> write c only (drop esc)
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 24]");                      // load esc
    emitter.instruction("mov BYTE PTR [r15], cl");                              // *write_ptr = esc (keep esc for non-enc)
    emitter.instruction("add r15, 1");                                          // write_ptr++
    emitter.label("__rt_fgetcsv_x_s3_enc");
    emitter.instruction("mov BYTE PTR [r15], al");                              // *write_ptr = c (literal)
    emitter.instruction("add r15, 1");                                          // write_ptr++
    emitter.instruction("mov QWORD PTR [rbp - 32], 2");                        // state = InQuotedField
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    // -- state 4: AfterCloseQuote --
    emitter.label("__rt_fgetcsv_x_st4");
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 16]");                      // load enc
    emitter.instruction("cmp al, cl");                                          // c == enc (doubled quote)?
    emitter.instruction("je __rt_fgetcsv_x_s4_dbl");                           // -> accumulate enc, back to quoted
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 8]");                        // load sep
    emitter.instruction("cmp al, cl");                                          // c == sep?
    emitter.instruction("je __rt_fgetcsv_x_push_reset");                        // -> push field, reset
    emitter.instruction("cmp al, 0x0a");                                        // c == newline (0x0A)?
    emitter.instruction("je __rt_fgetcsv_x_push_end");                         // -> push field, end
    emitter.instruction("cmp al, 0x0d");                                        // c == carriage return (0x0D)?
    emitter.instruction("je __rt_fgetcsv_x_push_end");                         // -> push field, end
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 16]");                      // load enc
    emitter.instruction("mov BYTE PTR [r15], cl");                              // *write_ptr = enc (restore close quote)
    emitter.instruction("add r15, 1");                                          // write_ptr++
    emitter.instruction("mov BYTE PTR [r15], al");                              // *write_ptr = c
    emitter.instruction("add r15, 1");                                          // write_ptr++
    emitter.instruction("mov QWORD PTR [rbp - 32], 1");                        // state = InField
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    emitter.label("__rt_fgetcsv_x_s4_dbl");
    emitter.instruction("movzx ecx, BYTE PTR [rbp - 16]");                      // load enc
    emitter.instruction("mov BYTE PTR [r15], cl");                              // *write_ptr = enc (doubled -> single)
    emitter.instruction("add r15, 1");                                          // write_ptr++
    emitter.instruction("mov QWORD PTR [rbp - 32], 2");                        // state = InQuotedField
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    // -- push field and reset for next field (separator encountered) --
    emitter.label("__rt_fgetcsv_x_push_reset");
    emitter.instruction("mov rax, r15");                                        // rax = write_ptr
    emitter.instruction("sub rax, r14");                                        // rax = len = write_ptr - field_start
    emitter.instruction("mov [rbp - 40], rax");                                 // save len at [rbp-40]
    emitter.instruction("mov rsi, r14");                                        // ptr = field_start
    emitter.instruction("mov rdx, [rbp - 40]");                                 // len
    emitter.instruction("call __rt_str_persist");                               // rax = persisted string (heap copy)
    emitter.instruction("mov rsi, rax");                                        // rsi = persisted string ptr
    emitter.instruction("mov rdi, rbx");                                        // rdi = array_ptr
    emitter.instruction("mov rdx, [rbp - 40]");                                 // rdx = len
    emitter.instruction("call __rt_array_push_str");                            // rax = array_ptr (possibly reallocated)
    emitter.instruction("mov rbx, rax");                                        // update array_ptr
    emitter.instruction("mov r14, r15");                                        // field_start = write_ptr (next field)
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                        // state = OutsideField
    emitter.instruction("jmp __rt_fgetcsv_x_loop");                             // continue loop

    // -- push field and end (newline or end-of-buffer) --
    emitter.label("__rt_fgetcsv_x_push_end");
    emitter.instruction("mov rax, r15");                                        // rax = write_ptr
    emitter.instruction("sub rax, r14");                                        // rax = len = write_ptr - field_start
    emitter.instruction("mov [rbp - 40], rax");                                 // save len at [rbp-40]
    emitter.instruction("mov rsi, r14");                                        // ptr = field_start
    emitter.instruction("mov rdx, [rbp - 40]");                                 // len
    emitter.instruction("call __rt_str_persist");                               // rax = persisted string (heap copy)
    emitter.instruction("mov rsi, rax");                                        // rsi = persisted string ptr
    emitter.instruction("mov rdi, rbx");                                        // rdi = array_ptr
    emitter.instruction("mov rdx, [rbp - 40]");                                 // rdx = len
    emitter.instruction("call __rt_array_push_str");                            // rax = array_ptr (possibly reallocated)
    emitter.instruction("mov rbx, rax");                                        // update array_ptr
    emitter.instruction("jmp __rt_fgetcsv_x_done");                             // -> epilogue

    // -- end of line (scan_ptr reached end_ptr without trailing newline) --
    emitter.label("__rt_fgetcsv_x_end_line");
    emitter.instruction("jmp __rt_fgetcsv_x_push_end");                         // push last field, then return

    // -- done: return array_ptr in rax --
    emitter.label("__rt_fgetcsv_x_done");
    emitter.instruction("mov rax, rbx");                                        // rax = array_ptr (return value)
    emitter.instruction("jmp __rt_fgetcsv_x_epilogue");                         // -> common epilogue

    // -- EOF: return 0 (false) --
    emitter.label("__rt_fgetcsv_x_eof");
    emitter.instruction("xor eax, eax");                                        // rax = 0 (false / EOF)

    // -- epilogue: restore registers and return --
    emitter.label("__rt_fgetcsv_x_epilogue");
    emitter.instruction("pop r15");                                             // restore r15
    emitter.instruction("pop r14");                                             // restore r14
    emitter.instruction("pop r13");                                             // restore r13
    emitter.instruction("pop r12");                                             // restore r12
    emitter.instruction("pop rbx");                                             // restore rbx
    emitter.instruction("leave");                                               // restore rbp + rsp
    emitter.instruction("ret");                                                 // return to caller
}