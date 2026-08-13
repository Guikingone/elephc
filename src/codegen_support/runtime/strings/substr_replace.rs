//! Purpose:
//! Emits the `__rt_substr_replace`, `__rt_subrepl_pre` runtime helper assembly for substr replace.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers scan or transform byte ranges and return target ABI pointer/length pairs for generated call sites.
//! - The result can never exceed `subject_len + replacement_len`, and that bound is reserved
//!   through `__rt_concat_reserve` before the first store, so long subjects or replacements fall
//!   back to heap storage instead of running off the end of the 64 KiB concat scratch buffer.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_substr_replace` runtime helper for PHP's `substr_replace`.
///
/// Replaces a slice of the subject string with the replacement string, returning
/// the result via the standard string ABI (pointer in x1, length in x2).
///
/// ## Register conventions (ARM64)
/// - `x0`: offset into subject (negative = count from the end)
/// - `x1/x2`: subject string pointer/length
/// - `x3/x4`: replacement string pointer/length
/// - `x7`: replace length (negative = bytes omitted from the end; `i64::MAX` = to the end)
///
/// ## Behavior
/// 1. Clamps offset to [0, subject_len]. Negative offset is converted to a
///    tail-relative index; if still negative it is clamped to 0.
/// 2. Reads a NEGATIVE length as php does — bytes omitted from the end of the remaining tail —
///    clamping to 0 only when more were omitted than remain. The caller signals an omitted
///    length with `i64::MAX` rather than a value inside the valid range, so `-1` keeps its php
///    meaning of "stop one byte before the end".
/// 3. Bounds the length by the remaining tail, which is also what expands `i64::MAX`.
/// 4. Builds the result in storage reserved through `__rt_concat_reserve` as:
///    prefix (subject[0..offset]) + replacement + suffix (subject[slice_end..]),
///    then publishes the written length through `__rt_concat_publish`.
///
/// Clobbers every caller-saved register, because the reservation can reach `__rt_heap_alloc`.
/// A wrapped `subject_len + replacement_len` bound reports PHP's allocation-overflow fatal.
pub fn emit_substr_replace(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_substr_replace_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: substr_replace ---");
    emitter.label_global("__rt_substr_replace");
    emitter.instruction("sub sp, sp, #64");                                     // allocate stack frame with spill slots for the clamped bounds and both input strings
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // set frame pointer

    // -- clamp offset --
    emitter.instruction("cmp x0, #0");                                          // check if offset is negative
    emitter.instruction("b.ge 1f");                                             // skip if non-negative
    emitter.instruction("add x0, x2, x0");                                      // offset = len + offset
    emitter.instruction("cmp x0, #0");                                          // clamp to 0
    emitter.instruction("csel x0, xzr, x0, lt");                                // if still negative, use 0
    emitter.raw("1:");
    emitter.instruction("cmp x0, x2");                                          // clamp offset to string length
    emitter.instruction("csel x0, x2, x0, gt");                                 // min(offset, len)

    // -- compute replace length --
    // A NEGATIVE length is php's "stop this many bytes before the end of the subject", counted
    // from the remaining tail. It used to be clamped to zero, so `substr_replace("hello","X",1,-1)`
    // answered `"hX"` where php answers `"hXo"`. The omitted-length case no longer arrives as `-1`
    // — the caller passes `i64::MAX`, which the end clamp below turns into "through the end"
    // without a sentinel, so `-1` can be read as the real php length it is.
    emitter.instruction("cmp x7, #0");                                          // check whether the requested length is negative
    emitter.instruction("b.ge 2f");                                             // a non-negative length is already a byte count
    emitter.instruction("sub x9, x2, x0");                                      // bytes remaining from the clamped offset
    emitter.instruction("add x7, x9, x7");                                      // omit that many bytes from the end of the remaining tail
    emitter.instruction("cmp x7, #0");                                          // check whether more bytes were omitted than remain
    emitter.instruction("csel x7, xzr, x7, lt");                                // an over-long omission replaces nothing
    emitter.raw("2:");
    // Bound the length by what remains BEFORE adding it to the offset. That is what turns the
    // caller's `i64::MAX` into "through the end" with no sentinel test, and it also means the
    // sum below can never overflow: length <= remaining, so offset + length <= subject length.
    emitter.instruction("sub x9, x2, x0");                                      // bytes remaining from the clamped offset
    emitter.instruction("cmp x7, x9");                                          // compare the requested length against what remains
    emitter.instruction("csel x7, x9, x7, gt");                                 // min(length, remaining)
    emitter.instruction("add x8, x0, x7");                                      // end = offset + length

    // -- reserve the exact upper bound (subject + replacement) before writing anything --
    emitter.instruction("stp x0, x8, [sp, #0]");                                // save the clamped replacement offset and slice end across the reservation call
    emitter.instruction("stp x1, x2, [sp, #16]");                               // save the subject pointer and length across the reservation call
    emitter.instruction("stp x3, x4, [sp, #32]");                               // save the replacement pointer and length across the reservation call
    emitter.instruction("adds x0, x2, x4");                                     // the result can never exceed subject length plus replacement length
    emitter.instruction("b.cs __rt_subrepl_size_overflow");                     // reject a wrapped bound instead of reserving a too-small destination
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage for the replaced string
    emitter.instruction("mov x12, x0");                                         // destination pointer
    emitter.instruction("mov x13, x0");                                         // save result start
    emitter.instruction("ldp x0, x8, [sp, #0]");                                // reload the clamped replacement offset and slice end
    emitter.instruction("ldp x1, x2, [sp, #16]");                               // reload the subject pointer and length
    emitter.instruction("ldp x3, x4, [sp, #32]");                               // reload the replacement pointer and length

    // -- build result: prefix + replacement + suffix --

    // -- copy prefix: subject[0..offset] --
    emitter.instruction("mov x14, #0");                                         // copy index
    emitter.label("__rt_subrepl_pre");
    emitter.instruction("cmp x14, x0");                                         // copied offset bytes?
    emitter.instruction("b.ge __rt_subrepl_mid");                               // yes → copy replacement
    emitter.instruction("ldrb w15, [x1, x14]");                                 // load prefix byte
    emitter.instruction("strb w15, [x12], #1");                                 // store and advance
    emitter.instruction("add x14, x14, #1");                                    // next byte
    emitter.instruction("b __rt_subrepl_pre");                                  // continue

    // -- copy replacement --
    emitter.label("__rt_subrepl_mid");
    emitter.instruction("mov x14, #0");                                         // replacement copy index
    emitter.label("__rt_subrepl_rep");
    emitter.instruction("cmp x14, x4");                                         // all replacement bytes copied?
    emitter.instruction("b.ge __rt_subrepl_suf");                               // yes → copy suffix
    emitter.instruction("ldrb w15, [x3, x14]");                                 // load replacement byte
    emitter.instruction("strb w15, [x12], #1");                                 // store and advance
    emitter.instruction("add x14, x14, #1");                                    // next byte
    emitter.instruction("b __rt_subrepl_rep");                                  // continue

    // -- copy suffix: subject[end..len] --
    emitter.label("__rt_subrepl_suf");
    emitter.instruction("mov x14, x8");                                         // start from end position
    emitter.label("__rt_subrepl_suf_loop");
    emitter.instruction("cmp x14, x2");                                         // past end of subject?
    emitter.instruction("b.ge __rt_subrepl_done");                              // yes → done
    emitter.instruction("ldrb w15, [x1, x14]");                                 // load suffix byte
    emitter.instruction("strb w15, [x12], #1");                                 // store and advance
    emitter.instruction("add x14, x14, #1");                                    // next byte
    emitter.instruction("b __rt_subrepl_suf_loop");                             // continue

    emitter.label("__rt_subrepl_done");
    emitter.instruction("mov x1, x13");                                         // result pointer
    emitter.instruction("sub x2, x12, x13");                                    // result length
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame
    emitter.instruction("add sp, sp, #64");                                     // deallocate
    emitter.instruction("ret");                                                 // return

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_subrepl_size_overflow");
    emitter.instruction("b __rt_alloc_overflow");                               // unconditional branch keeps the fatal trampoline cross-atom safe
}

/// Emits the x86_64 Linux variant of `__rt_substr_replace`.
///
/// Identical semantics to the ARM64 variant, but uses the x86_64 System V ABI:
/// - `rdi/rsi`: subject string pointer/length
/// - `rdx/rcx`: replacement string pointer/length
/// - `r8`: replacement offset (clamped to [0, subject_len]; negative = from the end)
/// - `r9`: replace length (negative = bytes omitted from the end; `i64::MAX` = to the end)
///
/// ## Output
/// - `rax`: result string pointer (concat buffer start)
/// - `rdx`: result string length
///
/// The concat buffer offset symbol (`_concat_off`) is updated to reflect bytes written.
fn emit_substr_replace_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: substr_replace ---");
    emitter.label_global("__rt_substr_replace");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving substr_replace() spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the saved subject, replacement, and slice bounds
    emitter.instruction("sub rsp, 64");                                         // reserve aligned spill slots for the input strings plus concat-buffer bookkeeping
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve the subject string pointer across offset clamping and concat-buffer copying
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // preserve the subject string length across offset clamping and concat-buffer copying
    emitter.instruction("mov QWORD PTR [rbp - 24], rdi");                       // preserve the replacement string pointer across the concat-buffer copy loops
    emitter.instruction("mov QWORD PTR [rbp - 32], rsi");                       // preserve the replacement string length across the concat-buffer copy loops
    emitter.instruction("mov r9, rcx");                                         // start clamping from the requested replacement offset
    emitter.instruction("cmp r9, 0");                                           // check whether the requested replacement offset is negative
    emitter.instruction("jge __rt_substr_replace_off_ready_linux_x86_64");      // skip the tail-relative offset fixup when the requested offset is already non-negative
    emitter.instruction("add r9, QWORD PTR [rbp - 16]");                        // convert the negative offset into a tail-relative byte index
    emitter.instruction("cmp r9, 0");                                           // check whether the tail-relative replacement offset still points before the string start
    emitter.instruction("mov rcx, 0");                                          // materialize zero for the final negative-offset clamp
    emitter.instruction("cmovl r9, rcx");                                       // clamp the adjusted replacement offset back to zero when it still underflows
    emitter.label("__rt_substr_replace_off_ready_linux_x86_64");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // reload the full subject-string length before clamping offsets past the end
    emitter.instruction("cmp r9, rcx");                                         // compare the requested replacement offset against the full subject-string length
    emitter.instruction("cmovg r9, rcx");                                       // clamp the replacement offset to the end of the subject string when needed
    // A NEGATIVE length is php's "stop this many bytes before the end of the subject", counted
    // from the remaining tail — it used to be clamped to zero, answering `"hX"` for
    // `substr_replace("hello","X",1,-1)` where php answers `"hXo"`. The omitted-length case no
    // longer arrives as `-1` but as `i64::MAX`, which the remaining-bound below turns into
    // "through the end" by the ordinary path, so `-1` reads as the real php length it is.
    emitter.instruction("mov r10, r8");                                         // start from the requested replacement length before bounds clamping
    emitter.instruction("cmp r10, 0");                                          // check whether the requested replacement length is negative
    emitter.instruction("jge __rt_substr_replace_len_known_linux_x86_64");      // a non-negative length is already a byte count
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // reload the subject-string length to size the remaining tail
    emitter.instruction("sub rcx, r9");                                         // bytes remaining from the clamped replacement offset
    emitter.instruction("add r10, rcx");                                        // omit that many bytes from the end of the remaining tail
    emitter.instruction("mov rcx, 0");                                          // materialize zero for the over-omission clamp
    emitter.instruction("cmovl r10, rcx");                                      // an over-long omission replaces nothing
    emitter.label("__rt_substr_replace_len_known_linux_x86_64");
    // Bound the length by what remains BEFORE adding it to the offset: that is what turns the
    // caller's `i64::MAX` into "through the end", and it keeps the sum below from overflowing.
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // reload the full subject-string length
    emitter.instruction("sub rcx, r9");                                         // bytes remaining from the clamped replacement offset
    emitter.instruction("cmp r10, rcx");                                        // compare the requested length against what remains
    emitter.instruction("cmovg r10, rcx");                                      // min(length, remaining)
    emitter.instruction("mov r11, r9");                                         // seed the suffix start from the clamped replacement offset
    emitter.instruction("add r11, r10");                                        // compute the byte offset immediately after the replaced slice
    emitter.instruction("mov QWORD PTR [rbp - 40], r9");                        // preserve the clamped replacement offset for the prefix copy loop
    emitter.instruction("mov QWORD PTR [rbp - 48], r11");                       // preserve the clamped suffix start for the suffix copy loop

    // -- reserve the exact upper bound (subject + replacement) before writing anything --
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // seed the reservation bound from the subject-string length
    emitter.instruction("add rax, QWORD PTR [rbp - 32]");                       // the result can never exceed subject length plus replacement length
    emitter.instruction("jc __rt_substr_replace_size_overflow_linux_x86_64");   // reject a wrapped bound instead of reserving a too-small destination
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage for the replaced string
    emitter.instruction("mov r8, rax");                                         // compute the destination pointer where the replaced string begins
    emitter.instruction("mov QWORD PTR [rbp - 56], r8");                        // preserve the replaced-string start pointer for the final x86_64 string return pair
    emitter.instruction("xor rcx, rcx");                                        // start the prefix copy loop from byte offset zero

    emitter.label("__rt_substr_replace_prefix_linux_x86_64");
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 40]");                       // have we already copied every prefix byte before the replacement offset?
    emitter.instruction("jge __rt_substr_replace_replacement_linux_x86_64");    // jump to the replacement payload copy once the full prefix is emitted
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the subject string pointer before copying the next prefix byte
    emitter.instruction("mov al, BYTE PTR [r10 + rcx]");                        // load the current prefix byte from the subject string
    emitter.instruction("mov BYTE PTR [r8], al");                               // store the current prefix byte into the concat-buffer destination
    emitter.instruction("add r8, 1");                                           // advance the concat-buffer destination after storing one prefix byte
    emitter.instruction("add rcx, 1");                                          // advance to the next prefix byte before repeating the loop
    emitter.instruction("jmp __rt_substr_replace_prefix_linux_x86_64");         // continue copying prefix bytes until the replacement offset is reached

    emitter.label("__rt_substr_replace_replacement_linux_x86_64");
    emitter.instruction("xor rcx, rcx");                                        // start copying the replacement payload from byte offset zero

    emitter.label("__rt_substr_replace_replacement_loop_linux_x86_64");
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 32]");                       // have we already copied every byte of the replacement string?
    emitter.instruction("jge __rt_substr_replace_suffix_linux_x86_64");         // jump to the suffix copy once the full replacement string is emitted
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the replacement string pointer before copying the next replacement byte
    emitter.instruction("mov al, BYTE PTR [r10 + rcx]");                        // load the current replacement byte from the replacement string
    emitter.instruction("mov BYTE PTR [r8], al");                               // store the current replacement byte into the concat-buffer destination
    emitter.instruction("add r8, 1");                                           // advance the concat-buffer destination after storing one replacement byte
    emitter.instruction("add rcx, 1");                                          // advance to the next replacement byte before repeating the loop
    emitter.instruction("jmp __rt_substr_replace_replacement_loop_linux_x86_64"); // continue copying replacement bytes until the full replacement string is emitted

    emitter.label("__rt_substr_replace_suffix_linux_x86_64");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // start the suffix copy from the clamped byte offset immediately after the replaced slice

    emitter.label("__rt_substr_replace_suffix_loop_linux_x86_64");
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 16]");                       // have we already copied every suffix byte through the end of the subject string?
    emitter.instruction("jge __rt_substr_replace_done_linux_x86_64");           // finalize the returned string once the suffix is fully emitted
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the subject string pointer before copying the next suffix byte
    emitter.instruction("mov al, BYTE PTR [r10 + rcx]");                        // load the current suffix byte from the subject string
    emitter.instruction("mov BYTE PTR [r8], al");                               // store the current suffix byte into the concat-buffer destination
    emitter.instruction("add r8, 1");                                           // advance the concat-buffer destination after storing one suffix byte
    emitter.instruction("add rcx, 1");                                          // advance to the next suffix byte before repeating the loop
    emitter.instruction("jmp __rt_substr_replace_suffix_loop_linux_x86_64");    // continue copying suffix bytes until the subject-string end is reached

    emitter.label("__rt_substr_replace_done_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 56]");                       // return the reserved start pointer of the replaced string in the primary x86_64 string result register
    emitter.instruction("mov rdx, r8");                                         // copy the destination end pointer so the final replaced-string length can be derived
    emitter.instruction("sub rdx, rax");                                        // derive the replaced-string length from the destination start/end pointers
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("add rsp, 64");                                         // release the substr_replace() spill slots before returning the replaced string
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning to the caller
    emitter.instruction("ret");                                                 // return the replaced string in the standard x86_64 string result registers

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_substr_replace_size_overflow_linux_x86_64");
    emitter.instruction("jmp __rt_alloc_overflow");                             // unconditional branch keeps the fatal trampoline reachable from every caller
}
