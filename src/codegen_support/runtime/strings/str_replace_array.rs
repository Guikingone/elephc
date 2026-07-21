//! Purpose:
//! Emits the `__rt_str_replace_array` / `__rt_str_ireplace_array` runtime helpers, which apply
//! `str_replace`/`str_ireplace` over an array `$search` (with an array or single-string `$replace`)
//! against a string subject by iterating the search elements and reusing the per-element helper.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - Iterative PHP semantics: each search element is applied to the result of the prior element, so
//!   later searches observe earlier replacements; missing array-replacement elements become `""`.
//! - The inner `__rt_str_replace`/`__rt_str_ireplace` helper produces a concat-buffer result; this
//!   helper allocates nothing on the heap and borrows the input arrays/subject without freeing them.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits both the case-sensitive and case-insensitive array-search replacement helpers.
///
/// Each helper takes (in target ABI argument order) a search-array base pointer, a replacement
/// pointer (an array base, or a single-string pointer), a replacement length (`-1` marks an array
/// replacement, otherwise the single-string length), and the subject pointer/length. It returns the
/// replaced string as a concat-buffer pointer/length pair, matching the per-element string helpers.
pub fn emit_str_replace_array(emitter: &mut Emitter) {
    emit_array_helper(emitter, "__rt_str_replace_array", "__rt_str_replace");
    emit_array_helper(emitter, "__rt_str_ireplace_array", "__rt_str_ireplace");
}

/// Dispatches the array-search replacement helper to the target-specific emitter.
fn emit_array_helper(emitter: &mut Emitter, helper: &str, inner: &str) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_helper_x86_64(emitter, helper, inner);
    } else {
        emit_array_helper_aarch64(emitter, helper, inner);
    }
}

/// Emits the AArch64 array-search replacement helper named `helper`, calling `inner` per element.
///
/// Arguments arrive in x1 (search array base), x2 (replacement pointer), x3 (replacement length or
/// the `-1` array sentinel), x4 (subject pointer), and x5 (subject length); the replaced string is
/// returned in x1/x2. Loop state is spilled to the frame so it survives each `bl` to `inner`.
fn emit_array_helper_aarch64(emitter: &mut Emitter, helper: &str, inner: &str) {
    let loop_l = format!("{helper}_loop");
    let done_l = format!("{helper}_done");
    let have_rc = format!("{helper}_have_rcount");
    let single_l = format!("{helper}_single");
    let empty_rep_l = format!("{helper}_empty_rep");
    let apply_l = format!("{helper}_apply");
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} ---", helper));
    emitter.label_global(helper);
    // -- set up stack frame (80 bytes) --
    emitter.instruction("sub sp, sp, #80");                                     // allocate 80 bytes of replacement bookkeeping
    emitter.instruction("stp x29, x30, [sp, #64]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #64");                                    // establish new frame pointer
    // -- save the incoming arguments to the frame --
    emitter.instruction("str x1, [sp]");                                        // save search array base pointer
    emitter.instruction("str x2, [sp, #8]");                                    // save replacement pointer (array base or single string)
    emitter.instruction("str x3, [sp, #16]");                                   // save replacement length (-1 marks an array replacement)
    emitter.instruction("str x4, [sp, #24]");                                   // initialize the running subject pointer
    emitter.instruction("str x5, [sp, #32]");                                   // initialize the running subject length
    // -- load element counts --
    emitter.instruction("ldr x6, [x1]");                                        // load the search-array element count
    emitter.instruction("str x6, [sp, #40]");                                   // save the search-array element count
    emitter.instruction("cmn x3, #1");                                          // is the replacement an array (length sentinel -1)?
    emitter.instruction(&format!("b.ne {}", have_rc));                          // skip the replacement count for a single-string replacement
    emitter.instruction("ldr x6, [x2]");                                        // load the replacement-array element count
    emitter.instruction("str x6, [sp, #48]");                                   // save the replacement-array element count
    emitter.label(&have_rc);
    emitter.instruction("mov x6, #0");                                          // initialize the search-element index
    emitter.instruction("str x6, [sp, #56]");                                   // save the search-element index
    // -- empty search array: produce a concat-resident subject copy --
    emitter.instruction("ldr x6, [sp, #40]");                                   // reload the search-array element count
    emitter.instruction(&format!("cbnz x6, {}", loop_l));                       // run the replacement loop when the search array is non-empty
    emitter.instruction("mov x1, xzr");                                         // empty search pointer for the pass-through copy
    emitter.instruction("mov x2, xzr");                                         // empty search length makes the inner helper copy verbatim
    emitter.instruction("mov x3, xzr");                                         // empty replacement pointer for the pass-through copy
    emitter.instruction("mov x4, xzr");                                         // empty replacement length for the pass-through copy
    emitter.instruction("ldr x5, [sp, #24]");                                   // load the subject pointer for the pass-through copy
    emitter.instruction("ldr x6, [sp, #32]");                                   // load the subject length for the pass-through copy
    emitter.instruction(&format!("bl {}", inner));                              // copy the subject into the concat buffer
    emitter.instruction("str x1, [sp, #24]");                                   // save the copied-subject pointer as the result
    emitter.instruction("str x2, [sp, #32]");                                   // save the copied-subject length as the result
    emitter.instruction(&format!("b {}", done_l));                              // finish with the concat-resident subject copy
    emitter.label(&loop_l);
    emitter.instruction("ldr x6, [sp, #56]");                                   // reload the current search-element index
    emitter.instruction("ldr x7, [sp, #40]");                                   // reload the search-array element count
    emitter.instruction("cmp x6, x7");                                          // have all search elements been processed?
    emitter.instruction(&format!("b.ge {}", done_l));                           // finish once every search element has been applied
    // -- load search[i] string pointer/length --
    emitter.instruction("ldr x8, [sp]");                                        // reload the search array base pointer
    emitter.instruction("lsl x9, x6, #4");                                      // scale the index into a 16-byte string-slot offset
    emitter.instruction("add x9, x8, x9");                                      // address the search element slot within the payload
    emitter.instruction("add x9, x9, #24");                                     // skip the 24-byte indexed-array header
    emitter.instruction("ldr x13, [x9]");                                       // load the current search-element string pointer
    emitter.instruction("ldr x14, [x9, #8]");                                   // load the current search-element string length
    // -- select the replacement for this index --
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the replacement length/sentinel
    emitter.instruction("cmn x10, #1");                                         // is the replacement an array (length sentinel -1)?
    emitter.instruction(&format!("b.ne {}", single_l));                         // use the single replacement string when not an array
    emitter.instruction("ldr x11, [sp, #48]");                                  // reload the replacement-array element count
    emitter.instruction("cmp x6, x11");                                         // is there a positional replacement for this index?
    emitter.instruction(&format!("b.ge {}", empty_rep_l));                      // missing replacement elements become the empty string
    emitter.instruction("ldr x12, [sp, #8]");                                   // reload the replacement array base pointer
    emitter.instruction("lsl x9, x6, #4");                                      // scale the index into a 16-byte string-slot offset
    emitter.instruction("add x9, x12, x9");                                     // address the replacement element slot within the payload
    emitter.instruction("add x9, x9, #24");                                     // skip the 24-byte indexed-array header
    emitter.instruction("ldr x3, [x9]");                                        // load the positional replacement string pointer
    emitter.instruction("ldr x4, [x9, #8]");                                    // load the positional replacement string length
    emitter.instruction(&format!("b {}", apply_l));                             // apply the positional replacement string
    emitter.label(&empty_rep_l);
    emitter.instruction("mov x3, xzr");                                         // empty replacement pointer for the missing element
    emitter.instruction("mov x4, xzr");                                         // empty replacement length for the missing element
    emitter.instruction(&format!("b {}", apply_l));                             // apply the empty replacement string
    emitter.label(&single_l);
    emitter.instruction("ldr x3, [sp, #8]");                                    // reload the single replacement string pointer
    emitter.instruction("ldr x4, [sp, #16]");                                   // reload the single replacement string length
    emitter.label(&apply_l);
    emitter.instruction("mov x1, x13");                                         // pass the current search string as the inner search pointer
    emitter.instruction("mov x2, x14");                                         // pass the current search length as the inner search length
    emitter.instruction("ldr x5, [sp, #24]");                                   // load the running subject pointer as the inner subject pointer
    emitter.instruction("ldr x6, [sp, #32]");                                   // load the running subject length as the inner subject length
    emitter.instruction(&format!("bl {}", inner));                              // replace this search element throughout the running subject
    emitter.instruction("str x1, [sp, #24]");                                   // save the produced string as the new running subject pointer
    emitter.instruction("str x2, [sp, #32]");                                   // save the produced length as the new running subject length
    emitter.instruction("ldr x6, [sp, #56]");                                   // reload the current search-element index
    emitter.instruction("add x6, x6, #1");                                      // advance to the next search element
    emitter.instruction("str x6, [sp, #56]");                                   // save the advanced search-element index
    emitter.instruction(&format!("b {}", loop_l));                              // continue with the next search element
    emitter.label(&done_l);
    emitter.instruction("ldr x1, [sp, #24]");                                   // load the final result string pointer
    emitter.instruction("ldr x2, [sp, #32]");                                   // load the final result string length
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // deallocate the stack frame
    emitter.instruction("ret");                                                 // return the replaced string to the caller
}

/// Emits the x86_64 array-search replacement helper named `helper`, calling `inner` per element.
///
/// Arguments arrive in rdi (search array base), rsi (replacement pointer), rdx (replacement length
/// or the `-1` array sentinel), rcx (subject pointer), and r8 (subject length); the replaced string
/// is returned in rax/rdx. Loop state is spilled to the frame so it survives each `call` to `inner`.
fn emit_array_helper_x86_64(emitter: &mut Emitter, helper: &str, inner: &str) {
    let loop_l = format!("{helper}_loop");
    let done_l = format!("{helper}_done");
    let have_rc = format!("{helper}_have_rcount");
    let single_l = format!("{helper}_single");
    let empty_rep_l = format!("{helper}_empty_rep");
    let apply_l = format!("{helper}_apply");
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} ---", helper));
    emitter.label_global(helper);
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame base
    emitter.instruction("sub rsp, 64");                                         // reserve spill slots for the replacement bookkeeping
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the search array base pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the replacement pointer (array base or single string)
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save the replacement length (-1 marks an array replacement)
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // initialize the running subject pointer
    emitter.instruction("mov QWORD PTR [rbp - 40], r8");                        // initialize the running subject length
    emitter.instruction("mov rax, QWORD PTR [rdi]");                            // load the search-array element count
    emitter.instruction("mov QWORD PTR [rbp - 48], rax");                       // save the search-array element count
    emitter.instruction("cmp rdx, -1");                                         // is the replacement an array (length sentinel -1)?
    emitter.instruction(&format!("jne {}", have_rc));                           // skip the replacement count for a single-string replacement
    emitter.instruction("mov rax, QWORD PTR [rsi]");                            // load the replacement-array element count
    emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // save the replacement-array element count
    emitter.label(&have_rc);
    emitter.instruction("mov QWORD PTR [rbp - 64], 0");                         // initialize the search-element index
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // reload the search-array element count
    emitter.instruction("test rax, rax");                                       // is the search array empty?
    emitter.instruction(&format!("jnz {}", loop_l));                            // run the replacement loop when the search array is non-empty
    emitter.instruction("xor eax, eax");                                        // empty search pointer for the pass-through copy
    emitter.instruction("xor edx, edx");                                        // empty search length makes the inner helper copy verbatim
    emitter.instruction("xor edi, edi");                                        // empty replacement pointer for the pass-through copy
    emitter.instruction("xor esi, esi");                                        // empty replacement length for the pass-through copy
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // load the subject pointer for the pass-through copy
    emitter.instruction("mov r8, QWORD PTR [rbp - 40]");                        // load the subject length for the pass-through copy
    emitter.instruction(&format!("call {}", inner));                            // copy the subject into the concat buffer
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the copied-subject pointer as the result
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // save the copied-subject length as the result
    emitter.instruction(&format!("jmp {}", done_l));                            // finish with the concat-resident subject copy
    emitter.label(&loop_l);
    emitter.instruction("mov rax, QWORD PTR [rbp - 64]");                       // reload the current search-element index
    emitter.instruction("cmp rax, QWORD PTR [rbp - 48]");                       // have all search elements been processed?
    emitter.instruction(&format!("jge {}", done_l));                            // finish once every search element has been applied
    // -- load search[i] string pointer/length into scratch registers --
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the search array base pointer
    emitter.instruction("mov rcx, rax");                                        // copy the index before scaling it to a slot offset
    emitter.instruction("shl rcx, 4");                                          // scale the index into a 16-byte string-slot offset
    emitter.instruction("lea rcx, [rdi + rcx + 24]");                           // address the search element slot after the array header
    emitter.instruction("mov r9, QWORD PTR [rcx]");                             // load the current search-element string pointer
    emitter.instruction("mov r10, QWORD PTR [rcx + 8]");                        // load the current search-element string length
    // -- select the replacement for this index --
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // reload the replacement length/sentinel
    emitter.instruction("cmp rdx, -1");                                         // is the replacement an array (length sentinel -1)?
    emitter.instruction(&format!("jne {}", single_l));                          // use the single replacement string when not an array
    emitter.instruction("mov rax, QWORD PTR [rbp - 64]");                       // reload the current index for the replacement lookup
    emitter.instruction("cmp rax, QWORD PTR [rbp - 56]");                       // is there a positional replacement for this index?
    emitter.instruction(&format!("jge {}", empty_rep_l));                       // missing replacement elements become the empty string
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // reload the replacement array base pointer
    emitter.instruction("mov rcx, rax");                                        // copy the index before scaling it to a slot offset
    emitter.instruction("shl rcx, 4");                                          // scale the index into a 16-byte string-slot offset
    emitter.instruction("lea rcx, [rsi + rcx + 24]");                           // address the replacement element slot after the array header
    emitter.instruction("mov rdi, QWORD PTR [rcx]");                            // load the positional replacement string pointer
    emitter.instruction("mov rsi, QWORD PTR [rcx + 8]");                        // load the positional replacement string length
    emitter.instruction(&format!("jmp {}", apply_l));                           // apply the positional replacement string
    emitter.label(&empty_rep_l);
    emitter.instruction("xor edi, edi");                                        // empty replacement pointer for the missing element
    emitter.instruction("xor esi, esi");                                        // empty replacement length for the missing element
    emitter.instruction(&format!("jmp {}", apply_l));                           // apply the empty replacement string
    emitter.label(&single_l);
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the single replacement string pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // reload the single replacement string length
    emitter.label(&apply_l);
    emitter.instruction("mov rax, r9");                                         // pass the current search string as the inner search pointer
    emitter.instruction("mov rdx, r10");                                        // pass the current search length as the inner search length
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // load the running subject pointer as the inner subject pointer
    emitter.instruction("mov r8, QWORD PTR [rbp - 40]");                        // load the running subject length as the inner subject length
    emitter.instruction(&format!("call {}", inner));                            // replace this search element throughout the running subject
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the produced string as the new running subject pointer
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // save the produced length as the new running subject length
    emitter.instruction("mov rax, QWORD PTR [rbp - 64]");                       // reload the current search-element index
    emitter.instruction("add rax, 1");                                          // advance to the next search element
    emitter.instruction("mov QWORD PTR [rbp - 64], rax");                       // save the advanced search-element index
    emitter.instruction(&format!("jmp {}", loop_l));                            // continue with the next search element
    emitter.label(&done_l);
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // load the final result string pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // load the final result string length
    emitter.instruction("add rsp, 64");                                         // release the spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the replaced string to the caller
}
