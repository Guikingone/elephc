//! Purpose:
//! Emits the `__rt_chunk_split` runtime helper assembly for PHP's `chunk_split`: copies the
//! subject in fixed-size pieces, appending the separator after every piece including the last.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - php-src appends the separator after the trailing partial chunk too, and its
//!   `chunklen > srclen` back-compat branch means an EMPTY subject still yields exactly one
//!   separator (`chunk_split("", 3, "-") === "-"`). The copy loop is therefore a do-while:
//!   one iteration always runs, copying `min(chunklen, remaining)` bytes plus the separator.
//! - The exact result size is `srclen + parts * seplen` where `parts` is the number of loop
//!   iterations (`ceil(srclen / chunklen)`, floored at 1). It is reserved through
//!   `__rt_concat_reserve` before the first store, so results past the 64 KiB concat scratch
//!   fall back to owned heap storage instead of running off the buffer.
//! - `$length < 1` never reaches this helper: the EIR lowering raises php-src's `ValueError`
//!   before the call, which is also what keeps the division below well defined.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_chunk_split` runtime helper.
///
/// ABI (AArch64):
///   Input:  `x1` = subject pointer, `x2` = subject length, `x3` = chunk length (>= 1),
///           `x4` = separator pointer, `x5` = separator length.
///   Output: `x1` = result pointer, `x2` = result length.
///
/// ABI (x86_64 System V):
///   Input:  `rax` = subject pointer, `rdx` = subject length, `rdi` = chunk length (>= 1),
///           `rcx` = separator pointer, `r8` = separator length.
///   Output: `rax` = result pointer, `rdx` = result length.
///
/// Clobbers every caller-saved register, because the reservation can reach `__rt_heap_alloc`.
/// A size computation that would wrap reports PHP's allocation-overflow fatal instead.
pub fn emit_chunk_split(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_chunk_split_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: chunk_split ---");
    emitter.label_global("__rt_chunk_split");

    // -- preserve the borrowed subject and separator across the reservation call --
    emitter.instruction("sub sp, sp, #80");                                     // allocate spill space for the subject, separator, and result start
    emitter.instruction("stp x29, x30, [sp, #64]");                             // save the frame pointer and return address across the reservation call
    emitter.instruction("add x29, sp, #64");                                    // establish the chunk_split helper frame pointer
    emitter.instruction("stp x1, x2, [sp]");                                    // save the subject pointer and length
    emitter.instruction("stp x3, x4, [sp, #16]");                               // save the chunk length and separator pointer
    emitter.instruction("str x5, [sp, #32]");                                   // save the separator length

    // -- count the separators the copy loop will emit --
    emitter.instruction("udiv x9, x2, x3");                                     // whole chunks = subject length / chunk length
    emitter.instruction("msub x10, x9, x3, x2");                                // trailing remainder = subject length - chunks * chunk length
    emitter.instruction("cmp x10, #0");                                         // is there a trailing partial chunk?
    emitter.instruction("cinc x9, x9, ne");                                     // a partial chunk contributes one more separator
    emitter.instruction("cmp x9, #0");                                          // did an empty subject leave no chunks at all?
    emitter.instruction("csinc x9, x9, xzr, ne");                               // php-src's back-compat branch still emits one separator for an empty subject

    // -- reserve the exact subject + separators result before writing anything --
    emitter.instruction("umulh x11, x9, x5");                                   // capture the high half of the separators * separator length product
    emitter.instruction("cbnz x11, __rt_chunk_split_size_overflow");            // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("mul x12, x9, x5");                                     // total separator bytes in the finished result
    emitter.instruction("adds x0, x2, x12");                                    // result size = subject length + total separator bytes
    emitter.instruction("b.cs __rt_chunk_split_size_overflow");                 // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage for the split result
    emitter.instruction("mov x13, x0");                                         // destination cursor
    emitter.instruction("str x0, [sp, #40]");                                   // save the result start for the published pointer
    emitter.instruction("ldp x1, x2, [sp]");                                    // reload the borrowed subject pointer and remaining length
    emitter.instruction("ldp x3, x4, [sp, #16]");                               // reload the chunk length and separator pointer
    emitter.instruction("ldr x5, [sp, #32]");                                   // reload the separator length

    // -- do-while: one iteration always runs so an empty subject still emits a separator --
    emitter.label("__rt_chunk_split_chunk");
    emitter.instruction("cmp x2, x3");                                          // does a whole chunk still fit in the remaining subject?
    emitter.instruction("csel x9, x3, x2, hs");                                 // copy a whole chunk, or the shorter trailing remainder
    emitter.instruction("mov x10, #0");                                         // start copying at the first byte of this chunk

    emitter.label("__rt_chunk_split_copy");
    emitter.instruction("cmp x10, x9");                                         // has the whole chunk been copied?
    emitter.instruction("b.hs __rt_chunk_split_copied");                        // move on to the separator once the chunk is copied
    emitter.instruction("ldrb w11, [x1, x10]");                                 // load the next subject byte
    emitter.instruction("strb w11, [x13, x10]");                                // store it at the same offset inside the result
    emitter.instruction("add x10, x10, #1");                                    // advance the copy index
    emitter.instruction("b __rt_chunk_split_copy");                             // copy the next subject byte

    emitter.label("__rt_chunk_split_copied");
    emitter.instruction("add x1, x1, x9");                                      // advance the subject cursor past the copied chunk
    emitter.instruction("add x13, x13, x9");                                    // advance the destination cursor past the copied chunk
    emitter.instruction("sub x2, x2, x9");                                      // record how much subject is still unconsumed
    emitter.instruction("mov x10, #0");                                         // start copying at the first separator byte

    emitter.label("__rt_chunk_split_sep");
    emitter.instruction("cmp x10, x5");                                         // has the whole separator been copied?
    emitter.instruction("b.hs __rt_chunk_split_sep_done");                      // the chunk plus its separator are complete
    emitter.instruction("ldrb w11, [x4, x10]");                                 // load the next separator byte
    emitter.instruction("strb w11, [x13, x10]");                                // store it at the same offset inside the result
    emitter.instruction("add x10, x10, #1");                                    // advance the copy index
    emitter.instruction("b __rt_chunk_split_sep");                              // copy the next separator byte

    emitter.label("__rt_chunk_split_sep_done");
    emitter.instruction("add x13, x13, x5");                                    // advance the destination cursor past the separator
    emitter.instruction("cbnz x2, __rt_chunk_split_chunk");                     // keep splitting while subject bytes remain

    emitter.instruction("ldr x1, [sp, #40]");                                   // return the split string start pointer
    emitter.instruction("sub x2, x13, x1");                                     // the written byte count is the result length
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore the frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // release the chunk_split helper frame
    emitter.instruction("ret");                                                 // return the split string as a PHP string pair

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_chunk_split_size_overflow");
    emitter.instruction("b __rt_alloc_overflow");                               // unconditional branch keeps the fatal trampoline cross-atom safe
}

/// Emits `__rt_chunk_split` for x86_64 Linux using the System V ABI.
///
/// The spill slots start at `[rbp-32]` so they stay clear of the `[rbp-8]`..`[rbp-24]` window
/// other runtime emitters reserve for pushed callee-saved registers.
fn emit_chunk_split_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: chunk_split ---");
    emitter.label_global("__rt_chunk_split");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the reservation and publish calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the borrowed strings
    emitter.instruction("sub rsp, 80");                                         // reserve aligned spill slots for the subject, separator, and result start
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the subject pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // save the subject length across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 48], rdi");                       // save the chunk length across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 56], rcx");                       // save the separator pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 64], r8");                        // save the separator length across the reservation call

    // -- count the separators the copy loop will emit --
    emitter.instruction("mov rax, rdx");                                        // the subject length is the dividend low half
    emitter.instruction("xor edx, edx");                                        // clear the dividend high half before the unsigned division
    emitter.instruction("div rdi");                                             // whole chunks in rax, trailing remainder in rdx
    emitter.instruction("test rdx, rdx");                                       // is there a trailing partial chunk?
    emitter.instruction("jz __rt_chunk_split_no_rest_linux_x86_64");            // skip the extra separator when the subject divides evenly
    emitter.instruction("add rax, 1");                                          // a partial chunk contributes one more separator

    emitter.label("__rt_chunk_split_no_rest_linux_x86_64");
    emitter.instruction("test rax, rax");                                       // did an empty subject leave no chunks at all?
    emitter.instruction("jnz __rt_chunk_split_have_parts_linux_x86_64");        // a non-empty subject already has its separator count
    emitter.instruction("mov rax, 1");                                          // php-src's back-compat branch still emits one separator for an empty subject

    emitter.label("__rt_chunk_split_have_parts_linux_x86_64");
    // -- reserve the exact subject + separators result before writing anything --
    emitter.instruction("mul r8");                                              // total separator bytes = separators * separator length
    emitter.instruction("jc __rt_chunk_split_size_overflow_linux_x86_64");      // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("add rax, QWORD PTR [rbp - 40]");                       // result size = subject length + total separator bytes
    emitter.instruction("jc __rt_chunk_split_size_overflow_linux_x86_64");      // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage for the split result
    emitter.instruction("mov r9, rax");                                         // destination cursor
    emitter.instruction("mov QWORD PTR [rbp - 72], rax");                       // save the result start for the published pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // reload the borrowed subject pointer as a read cursor
    emitter.instruction("mov rcx, QWORD PTR [rbp - 40]");                       // reload the subject length as the remaining counter
    emitter.instruction("mov rdi, QWORD PTR [rbp - 48]");                       // reload the chunk length
    emitter.instruction("mov r10, QWORD PTR [rbp - 56]");                       // reload the separator pointer
    emitter.instruction("mov r11, QWORD PTR [rbp - 64]");                       // reload the separator length

    // -- do-while: one iteration always runs so an empty subject still emits a separator --
    emitter.label("__rt_chunk_split_chunk_linux_x86_64");
    emitter.instruction("mov rax, rcx");                                        // assume the shorter trailing remainder is the piece to copy
    emitter.instruction("cmp rcx, rdi");                                        // does a whole chunk still fit in the remaining subject?
    emitter.instruction("cmovae rax, rdi");                                     // copy a whole chunk when one still fits
    emitter.instruction("xor edx, edx");                                        // start copying at the first byte of this chunk

    emitter.label("__rt_chunk_split_copy_linux_x86_64");
    emitter.instruction("cmp rdx, rax");                                        // has the whole chunk been copied?
    emitter.instruction("jae __rt_chunk_split_copied_linux_x86_64");            // move on to the separator once the chunk is copied
    emitter.instruction("mov r8b, BYTE PTR [rsi + rdx]");                       // load the next subject byte
    emitter.instruction("mov BYTE PTR [r9 + rdx], r8b");                        // store it at the same offset inside the result
    emitter.instruction("add rdx, 1");                                          // advance the copy index
    emitter.instruction("jmp __rt_chunk_split_copy_linux_x86_64");              // copy the next subject byte

    emitter.label("__rt_chunk_split_copied_linux_x86_64");
    emitter.instruction("add rsi, rax");                                        // advance the subject cursor past the copied chunk
    emitter.instruction("add r9, rax");                                         // advance the destination cursor past the copied chunk
    emitter.instruction("sub rcx, rax");                                        // record how much subject is still unconsumed
    emitter.instruction("xor edx, edx");                                        // start copying at the first separator byte

    emitter.label("__rt_chunk_split_sep_linux_x86_64");
    emitter.instruction("cmp rdx, r11");                                        // has the whole separator been copied?
    emitter.instruction("jae __rt_chunk_split_sep_done_linux_x86_64");          // the chunk plus its separator are complete
    emitter.instruction("mov r8b, BYTE PTR [r10 + rdx]");                       // load the next separator byte
    emitter.instruction("mov BYTE PTR [r9 + rdx], r8b");                        // store it at the same offset inside the result
    emitter.instruction("add rdx, 1");                                          // advance the copy index
    emitter.instruction("jmp __rt_chunk_split_sep_linux_x86_64");               // copy the next separator byte

    emitter.label("__rt_chunk_split_sep_done_linux_x86_64");
    emitter.instruction("add r9, r11");                                         // advance the destination cursor past the separator
    emitter.instruction("test rcx, rcx");                                       // are there subject bytes left to split?
    emitter.instruction("jnz __rt_chunk_split_chunk_linux_x86_64");             // keep splitting while subject bytes remain

    emitter.instruction("mov rax, QWORD PTR [rbp - 72]");                       // return the split string start pointer
    emitter.instruction("mov rdx, r9");                                         // copy the destination cursor into the length scratch register
    emitter.instruction("sub rdx, rax");                                        // the written byte count is the result length
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("add rsp, 80");                                         // release the chunk_split spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the split string as a PHP string pair

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_chunk_split_size_overflow_linux_x86_64");
    emitter.instruction("jmp __rt_alloc_overflow");                             // unconditional branch keeps the fatal trampoline reachable from every caller
}
