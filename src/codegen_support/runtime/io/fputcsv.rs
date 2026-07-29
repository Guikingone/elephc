//! Purpose:
//! Emits the `__rt_fputcsv` runtime helper assembly for writing a PHP string
//! array as a CSV row to a file descriptor. Supports custom separator,
//! enclosure, escape, and end-of-line characters passed as a packed `csv_opts`
//! word and an optional `(eol_ptr, eol_len)` pair from the EIR lowering.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - `csv_opts = (esc << 16) | (enc << 8) | sep`; zero bytes select defaults
//!   (sep → ',', enc → '"', esc → 0 means RFC 4180 doubling mode).
//! - `eol_ptr == 0` (or `eol_len == 0`) selects the default `"\n"` terminator.
//! - ARM64 and x86_64 variants mirror the same quoting and escaping logic.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits the `__rt_fputcsv` runtime helper, dispatching to the target-specific variant.
pub fn emit_fputcsv(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_fputcsv_linux_x86_64(emitter);
        return;
    }
    emit_fputcsv_aarch64(emitter);
}

/// ARM64 variant of `__rt_fputcsv`.
///
/// Signature: `__rt_fputcsv(fd: x0, arr: x1, csv_opts: x2, eol_ptr: x3, eol_len: x4)
/// -> bytes_written: x0`.
///
/// Writes each array element as a CSV field, quoting fields that contain the
/// separator, enclosure, escape, or whitespace characters. Internal quotes are
/// escaped by doubling (RFC 4180, `esc == 0`) or by the escape char (`esc != 0`).
/// A trailing `eol` (or `"\n"` default) is written after the last field.
fn emit_fputcsv_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: fputcsv ---");
    emitter.label_global("__rt_fputcsv");

    // -- set up stack frame: 128 bytes (fd, arr, total, index, sep, enc, esc, eol_ptr, eol_len, arrlen, field_ptr, field_len, scratch, scratch2, fp, lr) --
    emitter.instruction("sub sp, sp, #128");                                    // allocate 128 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #112]");                            // save frame pointer and return address
    emitter.instruction("add x29, sp, #112");                                   // establish new frame pointer

    // -- save inputs --
    emitter.instruction("str x0, [sp, #0]");                                    // save fd
    emitter.instruction("str x1, [sp, #8]");                                    // save array pointer
    emitter.instruction("str xzr, [sp, #16]");                                   // total bytes written = 0
    emitter.instruction("str xzr, [sp, #24]");                                   // current element index = 0

    // -- unpack csv_opts: sep = x2 & 0xFF, enc = (x2 >> 8) & 0xFF, esc = (x2 >> 16) & 0xFF --
    emitter.instruction("and w5, w2, #0xff");                                   // sep = csv_opts & 0xFF
    emitter.instruction("lsr w6, w2, #8");                                        // shift right 8 for enc
    emitter.instruction("and w6, w6, #0xff");                                   // enc = (csv_opts >> 8) & 0xFF
    emitter.instruction("lsr w7, w2, #16");                                      // shift right 16 for esc
    emitter.instruction("and w7, w7, #0xff");                                   // esc = (csv_opts >> 16) & 0xFF

    // -- apply defaults: sep==0 -> 0x2C, enc==0 -> 0x22 --
    emitter.instruction("cbnz w5, __rt_fputcsv_sep_ok");                        // if sep != 0, skip default
    emitter.instruction("mov w5, #0x2c");                                        // sep = ',' (0x2C)
    emitter.label("__rt_fputcsv_sep_ok");
    emitter.instruction("cbnz w6, __rt_fputcsv_enc_ok");                        // if enc != 0, skip default
    emitter.instruction("mov w6, #0x22");                                        // enc = '"' (0x22)
    emitter.label("__rt_fputcsv_enc_ok");

    // -- save sep/enc/esc and eol --
    emitter.instruction("str w5, [sp, #32]");                                    // save sep
    emitter.instruction("str w6, [sp, #40]");                                    // save enc
    emitter.instruction("str w7, [sp, #48]");                                    // save esc
    emitter.instruction("str x3, [sp, #56]");                                    // save eol_ptr
    emitter.instruction("str x4, [sp, #64]");                                    // save eol_len

    // -- get array length from header --
    emitter.instruction("ldr x9, [x1]");                                        // load array length from header
    emitter.instruction("str x9, [sp, #72]");                                   // save array length

    // -- main loop: iterate over array elements --
    emitter.label("__rt_fputcsv_loop");
    emitter.instruction("ldr x9, [sp, #24]");                                    // load current index
    emitter.instruction("ldr x10, [sp, #72]");                                    // load array length
    emitter.instruction("cmp x9, x10");                                         // check if we've processed all elements
    emitter.instruction("b.hs __rt_fputcsv_eol");                                // if done, write trailing eol

    // -- write separator before 2nd+ fields --
    emitter.instruction("cbz x9, __rt_fputcsv_field");                           // skip separator for first field
    emitter.instruction("ldr x0, [sp, #0]");                                     // reload fd
    emitter.instruction("ldr w1, [sp, #32]");                                    // load sep byte
    emitter.instruction("and x1, x1, #0xff");                                     // zero-extend sep
    emitter.instruction("strb w1, [sp, #96]");                                    // store sep byte in scratch slot
    emitter.instruction("add x1, sp, #96");                                        // ptr = scratch slot
    emitter.instruction("mov x2, #1");                                            // write 1 byte (sep)
    emitter.instruction("bl __rt_fd_write");                                      // write the separator
    emitter.instruction("ldr x9, [sp, #16]");                                    // reload total bytes
    emitter.instruction("add x9, x9, x0");                                        // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                     // save updated total

    // -- load current field from array --
    emitter.label("__rt_fputcsv_field");
    emitter.instruction("ldr x9, [sp, #24]");                                    // reload current index
    emitter.instruction("ldr x10, [sp, #8]");                                    // reload array pointer
    emitter.instruction("lsl x11, x9, #4");                                      // byte offset = index * 16
    emitter.instruction("add x11, x10, x11");                                    // element address = array + offset
    emitter.instruction("ldr x3, [x11, #24]");                                    // load string pointer (skip 24-byte header)
    emitter.instruction("ldr x4, [x11, #32]");                                    // load string length

    // -- check if field needs quoting (contains sep, enc, esc, or whitespace) --
    emitter.instruction("stp x3, x4, [sp, #80]");                                 // save field ptr and len (overlapping frame top is fine; we saved fp/lr at #80 but this is scratch above fp)
    emitter.instruction("mov x5, #0");                                            // needs_quote flag = 0
    emitter.instruction("mov x6, #0");                                            // scan index = 0
    emitter.label("__rt_fputcsv_scan");
    emitter.instruction("cmp x6, x4");                                            // check if scan complete
    emitter.instruction("b.hs __rt_fputcsv_scan_done");                            // if done scanning, proceed to write
    emitter.instruction("ldrb w7, [x3, x6]");                                    // load byte at current position
    emitter.instruction("ldr w8, [sp, #32]");                                     // load sep
    emitter.instruction("cmp w7, w8");                                            // byte == sep?
    emitter.instruction("b.eq __rt_fputcsv_need_q");                              // needs quoting
    emitter.instruction("ldr w8, [sp, #40]");                                     // load enc
    emitter.instruction("cmp w7, w8");                                            // byte == enc?
    emitter.instruction("b.eq __rt_fputcsv_need_q");                              // needs quoting
    emitter.instruction("ldr w8, [sp, #48]");                                     // load esc
    emitter.instruction("cbz w8, __rt_fputcsv_scan_ws");                          // esc == 0 -> skip esc check
    emitter.instruction("cmp w7, w8");                                            // byte == esc?
    emitter.instruction("b.eq __rt_fputcsv_need_q");                              // needs quoting
    emitter.label("__rt_fputcsv_scan_ws");
    emitter.instruction("cmp w7, #0x20");                                         // byte == space?
    emitter.instruction("b.eq __rt_fputcsv_need_q");                              // needs quoting
    emitter.instruction("cmp w7, #0x09");                                         // byte == tab?
    emitter.instruction("b.eq __rt_fputcsv_need_q");                              // needs quoting
    emitter.instruction("cmp w7, #0x0a");                                         // byte == newline?
    emitter.instruction("b.eq __rt_fputcsv_need_q");                              // needs quoting
    emitter.instruction("cmp w7, #0x0d");                                         // byte == carriage return?
    emitter.instruction("b.eq __rt_fputcsv_need_q");                              // needs quoting
    emitter.instruction("add x6, x6, #1");                                        // increment scan index
    emitter.instruction("b __rt_fputcsv_scan");                                    // continue scanning

    emitter.label("__rt_fputcsv_need_q");
    emitter.instruction("mov x5, #1");                                            // set needs_quote flag

    // -- write the field (quoted or unquoted) --
    emitter.label("__rt_fputcsv_scan_done");
    emitter.instruction("ldp x3, x4, [sp, #80]");                                 // reload field ptr and len
    emitter.instruction("cbz x5, __rt_fputcsv_plain");                            // if no quoting needed, write directly

    // -- write opening quote (enc) --
    emitter.instruction("ldr x0, [sp, #0]");                                       // reload fd
    emitter.instruction("ldr w1, [sp, #40]");                                      // load enc
    emitter.instruction("and x1, x1, #0xff");                                       // zero-extend enc
    emitter.instruction("strb w1, [sp, #96]");                                        // store enc byte in scratch slot
    emitter.instruction("add x1, sp, #96");                                          // ptr = scratch slot
    emitter.instruction("mov x2, #1");                                              // write 1 byte (enc)
    emitter.instruction("bl __rt_fd_write");                                        // write opening quote
    emitter.instruction("ldr x9, [sp, #16]");                                       // reload total bytes
    emitter.instruction("add x9, x9, x0");                                          // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                       // save updated total

    // -- write field contents, escaping internal quotes/escapes --
    emitter.instruction("ldp x3, x4, [sp, #80]");                                   // reload field ptr and len
    emitter.instruction("mov x6, #0");                                              // byte index = 0
    emitter.label("__rt_fputcsv_qloop");
    emitter.instruction("cmp x6, x4");                                              // check if all bytes written
    emitter.instruction("b.hs __rt_fputcsv_close_q");                                // if done, write closing quote
    emitter.instruction("ldrb w7, [x3, x6]");                                        // load current byte
    emitter.instruction("add x6, x6, #1");                                           // advance index
    emitter.instruction("str x6, [sp, #104]");                                        // save current index (scratch2 slot, safe: not used by loop bounds)
    emitter.instruction("ldr w8, [sp, #40]");                                        // load enc
    emitter.instruction("cmp w7, w8");                                                // byte == enc?
    emitter.instruction("b.ne __rt_fputcsv_qesc_chk");                                // if not enc, check esc
    // -- byte is enc: escape via doubling or escape char --
    emitter.instruction("ldr w8, [sp, #48]");                                        // load esc
    emitter.instruction("cbnz w8, __rt_fputcsv_q_escape");                            // esc != 0 -> write esc + enc
    // -- doubling mode: write enc twice --
    emitter.instruction("ldr x0, [sp, #0]");                                          // reload fd
    emitter.instruction("ldr w1, [sp, #40]");                                         // load enc
    emitter.instruction("and x1, x1, #0xff");                                          // zero-extend enc
    emitter.instruction("strb w1, [sp, #96]");                                         // store first enc in scratch
    emitter.instruction("strb w1, [sp, #97]");                                         // store second enc in scratch
    emitter.instruction("add x1, sp, #96");                                            // ptr = scratch slot
    emitter.instruction("mov x2, #2");                                                // write 2 bytes (enc enc)
    emitter.instruction("bl __rt_fd_write");                                          // write doubled quote
    emitter.instruction("ldr x9, [sp, #16]");                                         // reload total bytes
    emitter.instruction("add x9, x9, x0");                                            // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                         // save updated total
    emitter.instruction("b __rt_fputcsv_qloop_next");                                 // continue loop

    emitter.label("__rt_fputcsv_q_escape");
    // -- escape mode: write esc + enc --
    emitter.instruction("ldr x0, [sp, #0]");                                          // reload fd
    emitter.instruction("ldr w1, [sp, #48]");                                         // load esc
    emitter.instruction("and x1, x1, #0xff");                                          // zero-extend esc
    emitter.instruction("ldr w2, [sp, #40]");                                         // load enc
    emitter.instruction("and x2, x2, #0xff");                                          // zero-extend enc
    emitter.instruction("strb w1, [sp, #96]");                                         // store esc in scratch
    emitter.instruction("strb w2, [sp, #97]");                                         // store enc in scratch
    emitter.instruction("add x1, sp, #96");                                            // ptr = scratch slot
    emitter.instruction("mov x2, #2");                                                // write 2 bytes (esc enc)
    emitter.instruction("bl __rt_fd_write");                                          // write escaped quote
    emitter.instruction("ldr x9, [sp, #16]");                                         // reload total bytes
    emitter.instruction("add x9, x9, x0");                                            // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                         // save updated total
    emitter.instruction("b __rt_fputcsv_qloop_next");                                 // continue loop

    emitter.label("__rt_fputcsv_qesc_chk");
    // -- check if byte == esc (and esc != 0): double the esc --
    emitter.instruction("ldr w8, [sp, #48]");                                          // load esc
    emitter.instruction("cbz w8, __rt_fputcsv_qchar");                                 // esc == 0 -> no esc doubling
    emitter.instruction("cmp w7, w8");                                                // byte == esc?
    emitter.instruction("b.ne __rt_fputcsv_qchar");                                    // if not esc, write byte as-is
    // -- byte is esc: write esc twice --
    emitter.instruction("ldr x0, [sp, #0]");                                          // reload fd
    emitter.instruction("ldr w1, [sp, #48]");                                         // load esc
    emitter.instruction("and x1, x1, #0xff");                                          // zero-extend esc
    emitter.instruction("strb w1, [sp, #96]");                                         // store first esc in scratch
    emitter.instruction("strb w1, [sp, #97]");                                         // store second esc in scratch
    emitter.instruction("add x1, sp, #96");                                            // ptr = scratch slot
    emitter.instruction("mov x2, #2");                                                // write 2 bytes (esc esc)
    emitter.instruction("bl __rt_fd_write");                                          // write doubled escape
    emitter.instruction("ldr x9, [sp, #16]");                                         // reload total bytes
    emitter.instruction("add x9, x9, x0");                                            // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                         // save updated total
    emitter.instruction("b __rt_fputcsv_qloop_next");                                 // continue loop

    emitter.label("__rt_fputcsv_qchar");
    // -- write the actual character --
    emitter.instruction("ldp x3, x4, [sp, #80]");                                     // reload field ptr and len
    emitter.instruction("ldr x6, [sp, #104]");                                         // reload byte index
    emitter.instruction("sub x9, x6, #1");                                             // index of byte to write (we advanced x6 earlier)
    emitter.instruction("add x1, x3, x9");                                             // pointer to the byte
    emitter.instruction("ldr x0, [sp, #0]");                                          // reload fd
    emitter.instruction("mov x2, #1");                                                 // write 1 byte
    emitter.instruction("bl __rt_fd_write");                                           // write this byte
    emitter.instruction("ldr x9, [sp, #16]");                                          // reload total bytes
    emitter.instruction("add x9, x9, x0");                                             // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                          // save updated total

    emitter.label("__rt_fputcsv_qloop_next");
    emitter.instruction("ldr x6, [sp, #104]");                                         // reload byte index
    emitter.instruction("ldp x3, x4, [sp, #80]");                                     // reload field ptr and len
    emitter.instruction("b __rt_fputcsv_qloop");                                       // continue writing

    // -- write closing quote (enc) --
    emitter.label("__rt_fputcsv_close_q");
    emitter.instruction("ldr x0, [sp, #0]");                                          // reload fd
    emitter.instruction("ldr w1, [sp, #40]");                                         // load enc
    emitter.instruction("and x1, x1, #0xff");                                          // zero-extend enc
    emitter.instruction("strb w1, [sp, #96]");                                         // store enc byte in scratch
    emitter.instruction("add x1, sp, #96");                                            // ptr = scratch slot
    emitter.instruction("mov x2, #1");                                                // write 1 byte (enc)
    emitter.instruction("bl __rt_fd_write");                                          // write closing quote
    emitter.instruction("ldr x9, [sp, #16]");                                         // reload total bytes
    emitter.instruction("add x9, x9, x0");                                            // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                         // save updated total
    emitter.instruction("b __rt_fputcsv_next");                                       // proceed to next field

    // -- write plain field (no quoting needed) --
    emitter.label("__rt_fputcsv_plain");
    emitter.instruction("ldr x0, [sp, #0]");                                          // reload fd
    emitter.instruction("mov x1, x3");                                                // field pointer
    emitter.instruction("mov x2, x4");                                                // field length
    emitter.instruction("bl __rt_fd_write");                                          // write the plain field
    emitter.instruction("ldr x9, [sp, #16]");                                         // reload total bytes
    emitter.instruction("add x9, x9, x0");                                            // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                         // save updated total

    // -- advance to next element --
    emitter.label("__rt_fputcsv_next");
    emitter.instruction("ldr x9, [sp, #24]");                                         // reload current index
    emitter.instruction("add x9, x9, #1");                                             // increment index
    emitter.instruction("str x9, [sp, #24]");                                         // save updated index
    emitter.instruction("b __rt_fputcsv_loop");                                        // continue loop

    // -- write trailing eol (custom or default "\n") --
    emitter.label("__rt_fputcsv_eol");
    emitter.instruction("ldr x3, [sp, #56]");                                         // reload eol_ptr
    emitter.instruction("ldr x4, [sp, #64]");                                         // reload eol_len
    emitter.instruction("cbz x3, __rt_fputcsv_eol_default");                           // eol_ptr == 0 -> default "\n"
    emitter.instruction("cbz x4, __rt_fputcsv_eol_default");                           // eol_len == 0 -> default "\n"
    // -- write custom eol --
    emitter.instruction("ldr x0, [sp, #0]");                                          // reload fd
    emitter.instruction("mov x1, x3");                                                // eol pointer
    emitter.instruction("mov x2, x4");                                                // eol length
    emitter.instruction("bl __rt_fd_write");                                          // write the eol
    emitter.instruction("ldr x9, [sp, #16]");                                         // reload total bytes
    emitter.instruction("add x9, x9, x0");                                            // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                         // save final total
    emitter.instruction("b __rt_fputcsv_ret");                                        // return

    emitter.label("__rt_fputcsv_eol_default");
    emitter.instruction("ldr x0, [sp, #0]");                                          // reload fd
    emitter.adrp("x1", "__rt_fputcsv_nl_lit");                                        // load newline literal address
    emitter.add_lo12("x1", "x1", "__rt_fputcsv_nl_lit");                              // resolve exact address
    emitter.instruction("mov x2, #1");                                                // write 1 byte (newline)
    emitter.instruction("bl __rt_fd_write");                                          // write the newline
    emitter.instruction("ldr x9, [sp, #16]");                                         // reload total bytes
    emitter.instruction("add x9, x9, x0");                                            // add final bytes written
    emitter.instruction("str x9, [sp, #16]");                                         // save final total

    // -- return total bytes written --
    emitter.label("__rt_fputcsv_ret");
    emitter.instruction("ldr x0, [sp, #16]");                                         // return total bytes written

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #112]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #128");                                    // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller

    // -- literal data for newline --
    emitter.label("__rt_fputcsv_nl_lit");
    emitter.instruction(".ascii \"\\n\"");                                            // newline character literal
}

/// x86_64 Linux variant of `__rt_fputcsv`.
///
/// Signature: `__rt_fputcsv(fd: rdi, arr: rsi, csv_opts: rdx, eol_ptr: rcx,
/// eol_len: r8) -> bytes_written: rax`. Mirrors the ARM64 quoting/escaping
/// logic using the System V ABI and a rbp-relative frame.
fn emit_fputcsv_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: fputcsv ---");
    emitter.label_global("__rt_fputcsv");

    // -- prologue: 112-byte frame with rbp --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 112");                                         // reserve aligned stack space for writer state

    // -- save inputs --
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                         // preserve the destination file descriptor
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                        // preserve the source string-array pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                          // total written bytes start at zero
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                          // current field index starts at zero
    // -- save eol_ptr (rcx) and eol_len (r8) BEFORE unpacking csv_opts (which clobbers rcx) --
    emitter.instruction("mov QWORD PTR [rbp - 64], rcx");                        // preserve eol_ptr before rcx is clobbered
    emitter.instruction("mov QWORD PTR [rbp - 72], r8");                         // preserve eol_len (r8 not clobbered by unpack)

    // -- unpack csv_opts from rdx: sep=dl, enc=(rdx>>8)&0xFF, esc=(rdx>>16)&0xFF --
    emitter.instruction("movzx ecx, dl");                                       // sep = csv_opts & 0xFF
    emitter.instruction("mov QWORD PTR [rbp - 40], rcx");                         // save sep
    emitter.instruction("shr rdx, 8");                                          // shift right 8 for enc
    emitter.instruction("movzx ecx, dl");                                       // enc = (csv_opts >> 8) & 0xFF
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                         // save enc
    emitter.instruction("shr rdx, 8");                                          // shift right 8 more for esc
    emitter.instruction("movzx ecx, dl");                                       // esc = (csv_opts >> 16) & 0xFF
    emitter.instruction("mov QWORD PTR [rbp - 56], rcx");                         // save esc

    // -- apply defaults: sep==0 -> 0x2C, enc==0 -> 0x22 --
    emitter.instruction("cmp QWORD PTR [rbp - 40], 0");                           // sep == 0?
    emitter.instruction("jne __rt_fputcsv_x_sep_ok");                            // if not, skip default
    emitter.instruction("mov QWORD PTR [rbp - 40], 0x2c");                        // sep = ',' (0x2C)
    emitter.label("__rt_fputcsv_x_sep_ok");
    emitter.instruction("cmp QWORD PTR [rbp - 48], 0");                          // enc == 0?
    emitter.instruction("jne __rt_fputcsv_x_enc_ok");                             // if not, skip default
    emitter.instruction("mov QWORD PTR [rbp - 48], 0x22");                       // enc = '"' (0x22)
    emitter.label("__rt_fputcsv_x_enc_ok");

    // -- get array length from header --
    emitter.instruction("mov r10, QWORD PTR [rsi]");                             // load array length before entering the loop
    emitter.instruction("mov QWORD PTR [rbp - 80], r10");                        // preserve the source array length

    // -- main loop: iterate over array elements --
    emitter.label("__rt_fputcsv_x_loop");
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                         // reload the current field index
    emitter.instruction("cmp r10, QWORD PTR [rbp - 80]");                         // have we already emitted every field?
    emitter.instruction("jae __rt_fputcsv_x_eol");                                // write the trailing eol once every field has been emitted

    // -- write separator before 2nd+ fields --
    emitter.instruction("test r10, r10");                                         // is the current field index zero?
    emitter.instruction("jz __rt_fputcsv_x_field");                              // skip the separator before the first field
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                          // pass the destination fd for the separator
    emitter.instruction("lea rsi, [rbp - 40]");                                  // ptr = address of sep byte on stack
    emitter.instruction("mov edx, 1");                                           // write exactly one separator byte
    emitter.instruction("call __rt_fd_write");                                   // emit the separator through __rt_fd_write()
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                         // accumulate the separator byte count

    // -- load current field from array --
    emitter.label("__rt_fputcsv_x_field");
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                         // reload the current field index
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                         // reload the source string-array pointer
    emitter.instruction("mov rcx, r10");                                         // copy the field index before scaling
    emitter.instruction("shl rcx, 4");                                           // convert the field index into the byte offset
    emitter.instruction("lea rcx, [r11 + rcx + 24]");                            // compute the current string-slot address
    emitter.instruction("mov r8, QWORD PTR [rcx]");                              // load the current field string pointer
    emitter.instruction("mov r9, QWORD PTR [rcx + 8]");                           // load the current field string length
    emitter.instruction("mov QWORD PTR [rbp - 88], r8");                         // preserve the current field pointer
    emitter.instruction("mov QWORD PTR [rbp - 96], r9");                         // preserve the current field length
    emitter.instruction("mov QWORD PTR [rbp - 104], 0");                         // needs_quote starts false
    emitter.instruction("xor ecx, ecx");                                         // start scanning from byte index zero

    // -- scan field for quote-triggering bytes --
    emitter.label("__rt_fputcsv_x_scan");
    emitter.instruction("cmp rcx, r9");                                          // have we scanned every byte?
    emitter.instruction("jae __rt_fputcsv_x_scan_done");                         // proceed to field emission once scan completes
    emitter.instruction("movzx edx, BYTE PTR [r8 + rcx]");                        // load the current field byte
    emitter.instruction("cmp dl, BYTE PTR [rbp - 40]");                           // byte == sep?
    emitter.instruction("je __rt_fputcsv_x_need_q");                             // quote the field when it contains the separator
    emitter.instruction("cmp dl, BYTE PTR [rbp - 48]");                          // byte == enc?
    emitter.instruction("je __rt_fputcsv_x_need_q");                             // quote the field when it contains the enclosure
    emitter.instruction("cmp QWORD PTR [rbp - 56], 0");                          // esc == 0?
    emitter.instruction("jz __rt_fputcsv_x_scan_ws");                            // skip esc check if esc is disabled
    emitter.instruction("cmp dl, BYTE PTR [rbp - 56]");                          // byte == esc?
    emitter.instruction("je __rt_fputcsv_x_need_q");                             // quote the field when it contains the escape
    emitter.label("__rt_fputcsv_x_scan_ws");
    emitter.instruction("cmp dl, 0x20");                                         // byte == space?
    emitter.instruction("je __rt_fputcsv_x_need_q");                             // quote the field when it contains whitespace
    emitter.instruction("cmp dl, 0x09");                                         // byte == tab?
    emitter.instruction("je __rt_fputcsv_x_need_q");                             // quote the field when it contains a tab
    emitter.instruction("cmp dl, 0x0a");                                         // byte == newline?
    emitter.instruction("je __rt_fputcsv_x_need_q");                             // quote the field when it contains a newline
    emitter.instruction("cmp dl, 0x0d");                                         // byte == carriage return?
    emitter.instruction("je __rt_fputcsv_x_need_q");                             // quote the field when it contains a carriage return
    emitter.instruction("add rcx, 1");                                           // advance to the next field byte
    emitter.instruction("jmp __rt_fputcsv_x_scan");                              // continue scanning

    emitter.label("__rt_fputcsv_x_need_q");
    emitter.instruction("mov QWORD PTR [rbp - 104], 1");                         // remember that the current field must be quoted

    // -- write the field (quoted or unquoted) --
    emitter.label("__rt_fputcsv_x_scan_done");
    emitter.instruction("cmp QWORD PTR [rbp - 104], 0");                         // does the current field require quoting?
    emitter.instruction("je __rt_fputcsv_x_plain");                              // write the field directly when no quoting needed

    // -- write opening quote (enc) --
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                          // pass the destination fd for the opening quote
    emitter.instruction("lea rsi, [rbp - 48]");                                  // ptr = address of enc byte on stack
    emitter.instruction("mov edx, 1");                                          // write exactly one opening quote byte
    emitter.instruction("call __rt_fd_write");                                   // emit the opening quote
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                        // accumulate the opening-quote byte count
    emitter.instruction("mov QWORD PTR [rbp - 112], 0");                         // current byte index inside the quoted field

    // -- write field contents, escaping internal quotes/escapes --
    emitter.label("__rt_fputcsv_x_qloop");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 112]");                       // reload the current byte index
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 96]");                         // have we emitted every byte from the field?
    emitter.instruction("jae __rt_fputcsv_x_close_q");                           // write the closing quote once all bytes emitted
    emitter.instruction("mov r8, QWORD PTR [rbp - 88]");                         // reload the current field string pointer
    emitter.instruction("movzx edx, BYTE PTR [r8 + rcx]");                        // load the current field byte
    emitter.instruction("add QWORD PTR [rbp - 112], 1");                         // advance the byte index
    emitter.instruction("cmp dl, BYTE PTR [rbp - 48]");                          // is the byte the enclosure?
    emitter.instruction("jne __rt_fputcsv_x_qesc_chk");                          // skip to esc check if not enclosure
    // -- byte is enc: escape via doubling or escape char --
    emitter.instruction("cmp QWORD PTR [rbp - 56], 0");                          // esc == 0?
    emitter.instruction("jne __rt_fputcsv_x_q_escape");                          // esc != 0 -> write esc + enc
    // -- doubling mode: write enc twice --
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                         // pass the destination fd
    emitter.instruction("lea rsi, [rbp - 48]");                                  // ptr = address of enc byte
    emitter.instruction("mov edx, 1");                                          // write one enc
    emitter.instruction("call __rt_fd_write");                                  // emit first enc
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                        // accumulate
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                         // pass the destination fd
    emitter.instruction("lea rsi, [rbp - 48]");                                 // ptr = address of enc byte
    emitter.instruction("mov edx, 1");                                          // write second enc
    emitter.instruction("call __rt_fd_write");                                  // emit second enc
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate
    emitter.instruction("jmp __rt_fputcsv_x_qloop");                            // continue the quoted field loop

    emitter.label("__rt_fputcsv_x_q_escape");
    // -- escape mode: write esc + enc --
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                         // pass the destination fd
    emitter.instruction("lea rsi, [rbp - 56]");                                  // ptr = address of esc byte
    emitter.instruction("mov edx, 1");                                          // write one esc
    emitter.instruction("call __rt_fd_write");                                  // emit esc
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                         // pass the destination fd
    emitter.instruction("lea rsi, [rbp - 48]");                                 // ptr = address of enc byte
    emitter.instruction("mov edx, 1");                                          // write one enc
    emitter.instruction("call __rt_fd_write");                                  // emit enc
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate
    emitter.instruction("jmp __rt_fputcsv_x_qloop");                            // continue the quoted field loop

    emitter.label("__rt_fputcsv_x_qesc_chk");
    // -- check if byte == esc (and esc != 0): double the esc --
    emitter.instruction("cmp QWORD PTR [rbp - 56], 0");                         // esc == 0?
    emitter.instruction("jz __rt_fputcsv_x_qchar");                             // skip esc doubling if esc disabled
    emitter.instruction("cmp dl, BYTE PTR [rbp - 56]");                         // byte == esc?
    emitter.instruction("jne __rt_fputcsv_x_qchar");                            // if not esc, write byte as-is
    // -- byte is esc: write esc twice --
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                        // pass the destination fd
    emitter.instruction("lea rsi, [rbp - 56]");                                 // ptr = address of esc byte
    emitter.instruction("mov edx, 1");                                         // write one esc
    emitter.instruction("call __rt_fd_write");                                 // emit first esc
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                      // accumulate
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                        // pass the destination fd
    emitter.instruction("lea rsi, [rbp - 56]");                                // ptr = address of esc byte
    emitter.instruction("mov edx, 1");                                         // write second esc
    emitter.instruction("call __rt_fd_write");                                 // emit second esc
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                      // accumulate
    emitter.instruction("jmp __rt_fputcsv_x_qloop");                           // continue the quoted field loop

    emitter.label("__rt_fputcsv_x_qchar");
    // -- write the actual character --
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                        // pass the destination fd
    emitter.instruction("mov r8, QWORD PTR [rbp - 88]");                        // reload the current field string pointer
    emitter.instruction("mov rcx, QWORD PTR [rbp - 112]");                      // reload the byte index (already advanced)
    emitter.instruction("sub rcx, 1");                                        // index of byte to write
    emitter.instruction("lea rsi, [r8 + rcx]");                                // pointer to the byte
    emitter.instruction("mov edx, 1");                                         // write exactly one byte
    emitter.instruction("call __rt_fd_write");                                 // emit the byte
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                      // accumulate
    emitter.instruction("jmp __rt_fputcsv_x_qloop");                           // continue the quoted field loop

    // -- write closing quote (enc) --
    emitter.label("__rt_fputcsv_x_close_q");
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                       // pass the destination fd
    emitter.instruction("lea rsi, [rbp - 48]");                                // ptr = address of enc byte
    emitter.instruction("mov edx, 1");                                         // write one closing quote
    emitter.instruction("call __rt_fd_write");                                // emit the closing quote
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                      // accumulate
    emitter.instruction("jmp __rt_fputcsv_x_next");                            // advance to the next field

    // -- write plain field (no quoting needed) --
    emitter.label("__rt_fputcsv_x_plain");
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                       // pass the destination fd
    emitter.instruction("mov rsi, QWORD PTR [rbp - 88]");                       // pass the field pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 96]");                      // pass the field length
    emitter.instruction("call __rt_fd_write");                                 // emit the plain field
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                      // accumulate

    // -- advance to next element --
    emitter.label("__rt_fputcsv_x_next");
    emitter.instruction("add QWORD PTR [rbp - 32], 1");                        // advance the field index
    emitter.instruction("jmp __rt_fputcsv_x_loop");                           // continue emitting the remaining fields

    // -- write trailing eol (custom or default "\n") --
    emitter.label("__rt_fputcsv_x_eol");
    emitter.instruction("mov rax, QWORD PTR [rbp - 64]");                      // reload eol_ptr
    emitter.instruction("test rax, rax");                                      // eol_ptr == 0?
    emitter.instruction("jz __rt_fputcsv_x_eol_default");                      // use default "\n"
    emitter.instruction("mov rcx, QWORD PTR [rbp - 72]");                      // reload eol_len
    emitter.instruction("test rcx, rcx");                                     // eol_len == 0?
    emitter.instruction("jz __rt_fputcsv_x_eol_default");                      // use default "\n"
    // -- write custom eol --
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                      // pass the destination fd
    emitter.instruction("mov rsi, rax");                                      // pass eol pointer
    emitter.instruction("mov rdx, rcx");                                      // pass eol length
    emitter.instruction("call __rt_fd_write");                                // emit the eol
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                      // accumulate
    emitter.instruction("jmp __rt_fputcsv_x_ret");                            // return

    emitter.label("__rt_fputcsv_x_eol_default");
    emitter.instruction("mov edi, DWORD PTR [rbp - 8]");                      // pass the destination fd
    emitter.instruction("lea rsi, [rip + __rt_fputcsv_nl_lit]");               // pass the newline literal address
    emitter.instruction("mov edx, 1");                                        // write exactly one trailing newline byte
    emitter.instruction("call __rt_fd_write");                                // emit the trailing newline
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                      // accumulate the trailing newline byte count

    // -- return total bytes written --
    emitter.label("__rt_fputcsv_x_ret");
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                      // return the total written byte count
    emitter.instruction("leave");                                             // restore rbp + rsp
    emitter.instruction("ret");                                               // return to caller

    // -- literal data for newline --
    emitter.label("__rt_fputcsv_nl_lit");
    emitter.instruction(".ascii \"\\n\"");                                    // trailing newline character literal
}