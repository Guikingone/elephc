//! Purpose:
//! Emits the `__rt_preg_group_names` runtime helper that scans a delimiter-stripped
//! PCRE pattern and records the name → capture-group-index mapping for named groups
//! (`(?P<name>...)`, `(?<name>...)`, `(?'name'...)`).
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//! - Invoked at runtime by `__rt_preg_match_capture` on the capture path only.
//!
//! Key details:
//! - Returns a heap buffer laid out `[u32 count][ count × { u64 name_ptr, u32 name_len,
//!   u32 group_index } ]`. `name_ptr` points INTO the stripped pattern (no copy — the
//!   pattern outlives the call). Capture groups are numbered by opening `(` order, so the
//!   full match is index 0 and the first capturing group is index 1.
//! - Single-pass byte scanner with NORMAL / ESCAPE / CLASS / CLASS_ESCAPE states so that a
//!   `(` inside a character class or after a backslash is treated as a literal.
//! - The caller frees the buffer via `free()` on every return path. The helper allocates
//!   with `malloc`; a null (OOM) result reports zero named groups.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_preg_group_names` runtime helper for the current target.
///
/// Input:  x0/rdi = stripped pattern pointer, x1/rsi = stripped pattern length.
/// Output: x0/rax = pointer to a `malloc`'d name-map buffer (or null on allocation failure).
///
/// The buffer header holds a `u32` count; each subsequent 16-byte entry stores the name
/// pointer (into the stripped pattern), the name length, and the group index the name binds
/// to. The scanner increments the group index on every capturing `(` (named or unnamed) and
/// records an entry only for named groups.
pub(crate) fn emit_preg_group_names(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_preg_group_names_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: preg_group_names ---");
    emitter.label_global("__rt_preg_group_names");

    // -- set up stack frame and preserve the pattern arguments across malloc --
    emitter.instruction("sub sp, sp, #32");                                     // reserve frame for saved fp/lr and the pattern arguments
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish the group-names frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the stripped pattern pointer across malloc
    emitter.instruction("str x1, [sp, #8]");                                    // save the stripped pattern length across malloc

    // -- allocate a name-map buffer sized 8 + (len + 1) * 16 --
    emitter.instruction("add x9, x1, #1");                                      // reserve one entry slot per byte plus a spare for the count
    emitter.instruction("lsl x9, x9, #4");                                      // scale the worst-case entry count by the 16-byte entry stride
    emitter.instruction("add x0, x9, #8");                                      // add the 8-byte header holding the entry count
    emitter.bl_c("malloc");                                                     // allocate the scratch name-map buffer
    emitter.instruction("cbz x0, __rt_preg_group_names_oom");                   // an allocation failure reports zero named groups

    // -- initialize the scan state (buffer/count = 0, index/state/gidx = 0) --
    emitter.instruction("mov x11, x0");                                         // x11 = name-map buffer base
    emitter.instruction("str wzr, [x11]");                                      // publish an initial entry count of zero
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the stripped pattern pointer as the scan base
    emitter.instruction("ldr x10, [sp, #8]");                                   // reload the stripped pattern length as the scan bound
    emitter.instruction("mov x12, #0");                                         // x12 = current byte index
    emitter.instruction("mov x13, #0");                                         // x13 = scanner state (0 = NORMAL)
    emitter.instruction("mov x14, #0");                                         // x14 = running capture-group index
    emitter.instruction("mov x15, #0");                                         // x15 = recorded named-group count

    // -- main scan loop: dispatch on the scanner state --
    emitter.label("__rt_preg_group_names_loop");
    emitter.instruction("cmp x12, x10");                                        // have all pattern bytes been consumed?
    emitter.instruction("b.hs __rt_preg_group_names_finish");                   // stop once the scan cursor reaches the end
    emitter.instruction("ldrb w0, [x9, x12]");                                  // load the current pattern byte
    emitter.instruction("cmp x13, #0");                                         // are we in the NORMAL scanner state?
    emitter.instruction("b.eq __rt_preg_group_names_normal");                   // handle a normal byte
    emitter.instruction("cmp x13, #1");                                         // are we consuming an escaped byte?
    emitter.instruction("b.eq __rt_preg_group_names_escape");                   // skip the escaped byte and return to NORMAL
    emitter.instruction("cmp x13, #2");                                         // are we inside a character class?
    emitter.instruction("b.eq __rt_preg_group_names_class");                    // handle a character-class byte

    // -- state 3 (CLASS_ESCAPE): consume the escaped class byte, back to CLASS --
    emitter.instruction("add x12, x12, #1");                                    // advance past the escaped character-class byte
    emitter.instruction("mov x13, #2");                                         // resume scanning inside the character class
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan

    emitter.label("__rt_preg_group_names_escape");
    emitter.instruction("add x12, x12, #1");                                    // consume the escaped byte
    emitter.instruction("mov x13, #0");                                         // return to the NORMAL scanner state
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan

    emitter.label("__rt_preg_group_names_class");
    emitter.instruction("cmp w0, #93");                                         // is this the class close bracket ']'?
    emitter.instruction("b.ne __rt_preg_group_names_class_bs");                 // otherwise check for an escape inside the class
    emitter.instruction("add x12, x12, #1");                                    // advance past the ']'
    emitter.instruction("mov x13, #0");                                         // leaving the class returns to the NORMAL state
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan
    emitter.label("__rt_preg_group_names_class_bs");
    emitter.instruction("cmp w0, #92");                                         // is this a backslash inside the character class?
    emitter.instruction("b.ne __rt_preg_group_names_class_other");              // otherwise it is a plain class byte
    emitter.instruction("add x12, x12, #1");                                    // advance past the class backslash
    emitter.instruction("mov x13, #3");                                         // enter CLASS_ESCAPE to skip the next class byte
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan
    emitter.label("__rt_preg_group_names_class_other");
    emitter.instruction("add x12, x12, #1");                                    // advance past a plain character-class byte
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan

    emitter.label("__rt_preg_group_names_normal");
    emitter.instruction("cmp w0, #92");                                         // is this a backslash starting an escape?
    emitter.instruction("b.ne __rt_preg_group_names_normal_lb");                // otherwise check for a character-class opener
    emitter.instruction("add x12, x12, #1");                                    // advance past the backslash
    emitter.instruction("mov x13, #1");                                         // enter ESCAPE to skip the next byte
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan
    emitter.label("__rt_preg_group_names_normal_lb");
    emitter.instruction("cmp w0, #91");                                         // is this a character-class opener '['?
    emitter.instruction("b.ne __rt_preg_group_names_normal_lp");                // otherwise check for a group opener
    emitter.instruction("add x12, x12, #1");                                    // advance past the '['
    emitter.instruction("mov x13, #2");                                         // enter the CLASS scanner state
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan
    emitter.label("__rt_preg_group_names_normal_lp");
    emitter.instruction("cmp w0, #40");                                         // is this a group opener '('?
    emitter.instruction("b.eq __rt_preg_group_names_paren");                    // classify the group opener
    emitter.instruction("add x12, x12, #1");                                    // advance past an ordinary NORMAL byte
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan

    // -- classify a group opener '(' --
    emitter.label("__rt_preg_group_names_paren");
    emitter.instruction("add x1, x12, #1");                                     // index of the byte following '('
    emitter.instruction("cmp x1, x10");                                         // is there a byte after '('?
    emitter.instruction("b.hs __rt_preg_group_names_unnamed");                  // a trailing '(' counts as a capturing group
    emitter.instruction("ldrb w2, [x9, x1]");                                   // load the byte after '('
    emitter.instruction("cmp w2, #63");                                         // is it a '?' extension introducer?
    emitter.instruction("b.ne __rt_preg_group_names_unnamed");                  // '(' not followed by '?' is a capturing group
    emitter.instruction("add x3, x12, #2");                                     // index of the extension-type byte after '(?'
    emitter.instruction("cmp x3, x10");                                         // is there an extension-type byte?
    emitter.instruction("b.hs __rt_preg_group_names_noncap");                   // a truncated '(?' is treated as non-capturing
    emitter.instruction("ldrb w4, [x9, x3]");                                   // load the extension-type byte
    emitter.instruction("cmp w4, #80");                                         // is it 'P' (as in '(?P<name>')?
    emitter.instruction("b.eq __rt_preg_group_names_pp");                       // handle the Python-style named group
    emitter.instruction("cmp w4, #60");                                         // is it '<' (as in '(?<name>' or lookbehind)?
    emitter.instruction("b.eq __rt_preg_group_names_plt");                      // handle the angle-bracket named group
    emitter.instruction("cmp w4, #39");                                         // is it a single quote (as in '(?\x27name\x27')?
    emitter.instruction("b.eq __rt_preg_group_names_pquote");                   // handle the quoted named group
    emitter.instruction("b __rt_preg_group_names_noncap");                      // any other '(?...' is a non-capturing construct

    emitter.label("__rt_preg_group_names_pp");
    emitter.instruction("add x5, x12, #3");                                     // index of the byte after '(?P'
    emitter.instruction("cmp x5, x10");                                         // is there a byte after '(?P'?
    emitter.instruction("b.hs __rt_preg_group_names_noncap");                   // truncated '(?P' is non-capturing
    emitter.instruction("ldrb w6, [x9, x5]");                                   // load the byte after '(?P'
    emitter.instruction("cmp w6, #60");                                         // is it '<' (named group) rather than '=' or '>'?
    emitter.instruction("b.ne __rt_preg_group_names_noncap");                   // '(?P=' backref and '(?P>' recursion are non-capturing
    emitter.instruction("add x7, x12, #4");                                     // name starts after '(?P<'
    emitter.instruction("mov x3, #62");                                         // the name terminator is '>'
    emitter.instruction("b __rt_preg_group_names_named");                       // record this named group

    emitter.label("__rt_preg_group_names_plt");
    emitter.instruction("add x5, x12, #3");                                     // index of the byte after '(?<'
    emitter.instruction("cmp x5, x10");                                         // is there a byte after '(?<'?
    emitter.instruction("b.hs __rt_preg_group_names_noncap");                   // truncated '(?<' is non-capturing
    emitter.instruction("ldrb w6, [x9, x5]");                                   // load the byte after '(?<'
    emitter.instruction("cmp w6, #61");                                         // is it '=' (lookbehind assertion)?
    emitter.instruction("b.eq __rt_preg_group_names_noncap");                   // '(?<=' is a non-capturing lookbehind
    emitter.instruction("cmp w6, #33");                                         // is it '!' (negative lookbehind)?
    emitter.instruction("b.eq __rt_preg_group_names_noncap");                   // '(?<!' is a non-capturing lookbehind
    emitter.instruction("mov x7, x5");                                          // name starts right after '(?<'
    emitter.instruction("mov x3, #62");                                         // the name terminator is '>'
    emitter.instruction("b __rt_preg_group_names_named");                       // record this named group

    emitter.label("__rt_preg_group_names_pquote");
    emitter.instruction("add x7, x12, #3");                                     // name starts after '(?\x27'
    emitter.instruction("mov x3, #39");                                         // the name terminator is a single quote
    emitter.instruction("b __rt_preg_group_names_named");                       // record this named group

    emitter.label("__rt_preg_group_names_unnamed");
    emitter.instruction("add x14, x14, #1");                                    // an unnamed capturing group advances the group index
    emitter.instruction("add x12, x12, #1");                                    // advance past '(' so the body scans normally
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan

    emitter.label("__rt_preg_group_names_noncap");
    emitter.instruction("add x12, x12, #1");                                    // advance past '(' without counting a group
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan

    // -- record a named capturing group and scan its name --
    emitter.label("__rt_preg_group_names_named");
    emitter.instruction("add x14, x14, #1");                                    // a named capturing group advances the group index
    emitter.instruction("mov x4, x7");                                          // x4 = scan cursor starting at the name
    emitter.label("__rt_preg_group_names_name_loop");
    emitter.instruction("cmp x4, x10");                                         // did the name run off the end of the pattern?
    emitter.instruction("b.hs __rt_preg_group_names_finish");                   // a missing terminator is malformed; stop scanning
    emitter.instruction("ldrb w6, [x9, x4]");                                   // load the current name byte
    emitter.instruction("cmp w6, w3");                                          // is it the name terminator?
    emitter.instruction("b.eq __rt_preg_group_names_name_found");               // the name is complete
    emitter.instruction("add x4, x4, #1");                                      // advance to the next name byte
    emitter.instruction("b __rt_preg_group_names_name_loop");                   // continue scanning the name

    emitter.label("__rt_preg_group_names_name_found");
    emitter.instruction("mov x1, x15");                                         // copy the current entry count before scaling
    emitter.instruction("lsl x1, x1, #4");                                      // scale the entry index by the 16-byte entry stride
    emitter.instruction("add x1, x11, x1");                                     // advance from the buffer base to this entry
    emitter.instruction("add x1, x1, #8");                                      // skip the 8-byte header to the entry payload
    emitter.instruction("add x2, x9, x7");                                      // name_ptr = pattern base + name start offset
    emitter.instruction("str x2, [x1]");                                        // store the name pointer into the entry
    emitter.instruction("sub x6, x4, x7");                                      // name_len = terminator index - name start
    emitter.instruction("str w6, [x1, #8]");                                    // store the name length into the entry
    emitter.instruction("str w14, [x1, #12]");                                  // store the capture group index into the entry
    emitter.instruction("add x15, x15, #1");                                    // one more named group has been recorded
    emitter.instruction("add x12, x4, #1");                                     // resume scanning just past the name terminator
    emitter.instruction("b __rt_preg_group_names_loop");                        // continue the scan

    emitter.label("__rt_preg_group_names_finish");
    emitter.instruction("str w15, [x11]");                                      // publish the recorded named-group count
    emitter.instruction("mov x0, x11");                                         // return the name-map buffer pointer
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the group-names stack frame
    emitter.instruction("ret");                                                 // return the name-map buffer in x0

    emitter.label("__rt_preg_group_names_oom");
    emitter.instruction("mov x0, #0");                                          // report no name map when the allocation failed
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the group-names stack frame
    emitter.instruction("ret");                                                 // return a null name-map pointer in x0
}

/// Emits the Linux x86_64 variant of `__rt_preg_group_names`.
///
/// Uses the System V AMD64 ABI: rdi = stripped pattern pointer, rsi = stripped pattern
/// length; returns the `malloc`'d name-map buffer in rax (null on allocation failure).
/// Preserves rbx/r12/r13/r14/r15 as callee-saved scan state (pattern base, length, buffer,
/// group index, and recorded count) and uses rcx/rdx as the scan cursor and scanner state.
fn emit_preg_group_names_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: preg_group_names ---");
    emitter.label_global("__rt_preg_group_names");

    // -- set up the frame and preserve callee-saved scan registers --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("push rbx");                                            // preserve rbx for the pattern base pointer
    emitter.instruction("push r12");                                            // preserve r12 for the pattern length
    emitter.instruction("push r13");                                            // preserve r13 for the name-map buffer
    emitter.instruction("push r14");                                            // preserve r14 for the group index
    emitter.instruction("push r15");                                            // preserve r15 for the recorded count
    emitter.instruction("sub rsp, 16");                                         // reserve aligned spill slots for the pattern pointer and length
    emitter.instruction("mov QWORD PTR [rbp - 48], rdi");                       // save the stripped pattern pointer across malloc
    emitter.instruction("mov QWORD PTR [rbp - 56], rsi");                       // save the stripped pattern length across malloc

    // -- allocate a name-map buffer sized 8 + (len + 1) * 16 --
    emitter.instruction("lea rdi, [rsi + 1]");                                  // reserve one entry slot per byte plus a spare for the count
    emitter.instruction("shl rdi, 4");                                          // scale the worst-case entry count by the 16-byte entry stride
    emitter.instruction("add rdi, 8");                                          // add the 8-byte header holding the entry count
    emitter.bl_c("malloc");                                                     // allocate the scratch name-map buffer
    emitter.instruction("test rax, rax");                                       // did the allocation succeed?
    emitter.instruction("jz __rt_preg_group_names_oom_x");                      // report no name map when the allocation failed

    // -- initialize scan state --
    emitter.instruction("mov r13, rax");                                        // r13 = name-map buffer base
    emitter.instruction("mov DWORD PTR [r13], 0");                              // publish an initial entry count of zero
    emitter.instruction("mov rbx, QWORD PTR [rbp - 48]");                       // rbx = stripped pattern base pointer
    emitter.instruction("mov r12, QWORD PTR [rbp - 56]");                       // r12 = stripped pattern length
    emitter.instruction("xor rcx, rcx");                                        // rcx = current byte index
    emitter.instruction("xor rdx, rdx");                                        // rdx = scanner state (0 = NORMAL)
    emitter.instruction("xor r14, r14");                                        // r14 = running capture-group index
    emitter.instruction("xor r15, r15");                                        // r15 = recorded named-group count

    emitter.label("__rt_preg_group_names_loop_x");
    emitter.instruction("cmp rcx, r12");                                        // have all pattern bytes been consumed?
    emitter.instruction("jae __rt_preg_group_names_finish_x");                  // stop once the scan cursor reaches the end
    emitter.instruction("movzx eax, BYTE PTR [rbx + rcx]");                     // load the current pattern byte
    emitter.instruction("cmp rdx, 0");                                          // are we in the NORMAL scanner state?
    emitter.instruction("je __rt_preg_group_names_normal_x");                   // handle a normal byte
    emitter.instruction("cmp rdx, 1");                                          // are we consuming an escaped byte?
    emitter.instruction("je __rt_preg_group_names_escape_x");                   // skip the escaped byte and return to NORMAL
    emitter.instruction("cmp rdx, 2");                                          // are we inside a character class?
    emitter.instruction("je __rt_preg_group_names_class_x");                    // handle a character-class byte
    emitter.instruction("add rcx, 1");                                          // state 3 CLASS_ESCAPE: consume the escaped class byte
    emitter.instruction("mov rdx, 2");                                          // resume scanning inside the character class
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan

    emitter.label("__rt_preg_group_names_escape_x");
    emitter.instruction("add rcx, 1");                                          // consume the escaped byte
    emitter.instruction("xor rdx, rdx");                                        // return to the NORMAL scanner state
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan

    emitter.label("__rt_preg_group_names_class_x");
    emitter.instruction("cmp eax, 93");                                         // is this the class close bracket ']'?
    emitter.instruction("jne __rt_preg_group_names_class_bs_x");                // otherwise check for an escape inside the class
    emitter.instruction("add rcx, 1");                                          // advance past the ']'
    emitter.instruction("xor rdx, rdx");                                        // leaving the class returns to the NORMAL state
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan
    emitter.label("__rt_preg_group_names_class_bs_x");
    emitter.instruction("cmp eax, 92");                                         // is this a backslash inside the character class?
    emitter.instruction("jne __rt_preg_group_names_class_other_x");             // otherwise it is a plain class byte
    emitter.instruction("add rcx, 1");                                          // advance past the class backslash
    emitter.instruction("mov rdx, 3");                                          // enter CLASS_ESCAPE to skip the next class byte
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan
    emitter.label("__rt_preg_group_names_class_other_x");
    emitter.instruction("add rcx, 1");                                          // advance past a plain character-class byte
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan

    emitter.label("__rt_preg_group_names_normal_x");
    emitter.instruction("cmp eax, 92");                                         // is this a backslash starting an escape?
    emitter.instruction("jne __rt_preg_group_names_normal_lb_x");               // otherwise check for a character-class opener
    emitter.instruction("add rcx, 1");                                          // advance past the backslash
    emitter.instruction("mov rdx, 1");                                          // enter ESCAPE to skip the next byte
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan
    emitter.label("__rt_preg_group_names_normal_lb_x");
    emitter.instruction("cmp eax, 91");                                         // is this a character-class opener '['?
    emitter.instruction("jne __rt_preg_group_names_normal_lp_x");               // otherwise check for a group opener
    emitter.instruction("add rcx, 1");                                          // advance past the '['
    emitter.instruction("mov rdx, 2");                                          // enter the CLASS scanner state
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan
    emitter.label("__rt_preg_group_names_normal_lp_x");
    emitter.instruction("cmp eax, 40");                                         // is this a group opener '('?
    emitter.instruction("je __rt_preg_group_names_paren_x");                    // classify the group opener
    emitter.instruction("add rcx, 1");                                          // advance past an ordinary NORMAL byte
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan

    emitter.label("__rt_preg_group_names_paren_x");
    emitter.instruction("lea rsi, [rcx + 1]");                                  // index of the byte following '('
    emitter.instruction("cmp rsi, r12");                                        // is there a byte after '('?
    emitter.instruction("jae __rt_preg_group_names_unnamed_x");                 // a trailing '(' counts as a capturing group
    emitter.instruction("movzx r8d, BYTE PTR [rbx + rsi]");                     // load the byte after '('
    emitter.instruction("cmp r8d, 63");                                         // is it a '?' extension introducer?
    emitter.instruction("jne __rt_preg_group_names_unnamed_x");                 // '(' not followed by '?' is a capturing group
    emitter.instruction("lea rdi, [rcx + 2]");                                  // index of the extension-type byte after '(?'
    emitter.instruction("cmp rdi, r12");                                        // is there an extension-type byte?
    emitter.instruction("jae __rt_preg_group_names_noncap_x");                  // a truncated '(?' is treated as non-capturing
    emitter.instruction("movzx r9d, BYTE PTR [rbx + rdi]");                     // load the extension-type byte
    emitter.instruction("cmp r9d, 80");                                         // is it 'P' (as in '(?P<name>')?
    emitter.instruction("je __rt_preg_group_names_pp_x");                       // handle the Python-style named group
    emitter.instruction("cmp r9d, 60");                                         // is it '<' (as in '(?<name>' or lookbehind)?
    emitter.instruction("je __rt_preg_group_names_plt_x");                      // handle the angle-bracket named group
    emitter.instruction("cmp r9d, 39");                                         // is it a single quote (as in '(?\x27name\x27')?
    emitter.instruction("je __rt_preg_group_names_pquote_x");                   // handle the quoted named group
    emitter.instruction("jmp __rt_preg_group_names_noncap_x");                  // any other '(?...' is a non-capturing construct

    emitter.label("__rt_preg_group_names_pp_x");
    emitter.instruction("lea r10, [rcx + 3]");                                  // index of the byte after '(?P'
    emitter.instruction("cmp r10, r12");                                        // is there a byte after '(?P'?
    emitter.instruction("jae __rt_preg_group_names_noncap_x");                  // truncated '(?P' is non-capturing
    emitter.instruction("movzx r11d, BYTE PTR [rbx + r10]");                    // load the byte after '(?P'
    emitter.instruction("cmp r11d, 60");                                        // is it '<' (named group) rather than '=' or '>'?
    emitter.instruction("jne __rt_preg_group_names_noncap_x");                  // '(?P=' backref and '(?P>' recursion are non-capturing
    emitter.instruction("lea r10, [rcx + 4]");                                  // name starts after '(?P<'
    emitter.instruction("mov r11, 62");                                         // the name terminator is '>'
    emitter.instruction("jmp __rt_preg_group_names_named_x");                   // record this named group

    emitter.label("__rt_preg_group_names_plt_x");
    emitter.instruction("lea r10, [rcx + 3]");                                  // index of the byte after '(?<'
    emitter.instruction("cmp r10, r12");                                        // is there a byte after '(?<'?
    emitter.instruction("jae __rt_preg_group_names_noncap_x");                  // truncated '(?<' is non-capturing
    emitter.instruction("movzx r11d, BYTE PTR [rbx + r10]");                    // load the byte after '(?<'
    emitter.instruction("cmp r11d, 61");                                        // is it '=' (lookbehind assertion)?
    emitter.instruction("je __rt_preg_group_names_noncap_x");                   // '(?<=' is a non-capturing lookbehind
    emitter.instruction("cmp r11d, 33");                                        // is it '!' (negative lookbehind)?
    emitter.instruction("je __rt_preg_group_names_noncap_x");                   // '(?<!' is a non-capturing lookbehind
    emitter.instruction("mov r11, 62");                                         // the name terminator is '>'
    emitter.instruction("jmp __rt_preg_group_names_named_x");                   // record this named group

    emitter.label("__rt_preg_group_names_pquote_x");
    emitter.instruction("lea r10, [rcx + 3]");                                  // name starts after '(?\x27'
    emitter.instruction("mov r11, 39");                                         // the name terminator is a single quote
    emitter.instruction("jmp __rt_preg_group_names_named_x");                   // record this named group

    emitter.label("__rt_preg_group_names_unnamed_x");
    emitter.instruction("add r14, 1");                                          // an unnamed capturing group advances the group index
    emitter.instruction("add rcx, 1");                                          // advance past '(' so the body scans normally
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan

    emitter.label("__rt_preg_group_names_noncap_x");
    emitter.instruction("add rcx, 1");                                          // advance past '(' without counting a group
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan

    // -- record a named capturing group; r10 = name start, r11 = terminator byte --
    emitter.label("__rt_preg_group_names_named_x");
    emitter.instruction("add r14, 1");                                          // a named capturing group advances the group index
    emitter.instruction("mov r8, r10");                                         // r8 = scan cursor starting at the name
    emitter.label("__rt_preg_group_names_name_loop_x");
    emitter.instruction("cmp r8, r12");                                         // did the name run off the end of the pattern?
    emitter.instruction("jae __rt_preg_group_names_finish_x");                  // a missing terminator is malformed; stop scanning
    emitter.instruction("movzx r9d, BYTE PTR [rbx + r8]");                      // load the current name byte
    emitter.instruction("cmp r9, r11");                                         // is it the name terminator?
    emitter.instruction("je __rt_preg_group_names_name_found_x");               // the name is complete
    emitter.instruction("add r8, 1");                                           // advance to the next name byte
    emitter.instruction("jmp __rt_preg_group_names_name_loop_x");               // continue scanning the name

    emitter.label("__rt_preg_group_names_name_found_x");
    emitter.instruction("mov rax, r15");                                        // copy the current entry count before scaling
    emitter.instruction("shl rax, 4");                                          // scale the entry index by the 16-byte entry stride
    emitter.instruction("lea rax, [r13 + rax + 8]");                            // advance to this entry past the 8-byte header
    emitter.instruction("lea rsi, [rbx + r10]");                                // name_ptr = pattern base + name start offset
    emitter.instruction("mov QWORD PTR [rax], rsi");                            // store the name pointer into the entry
    emitter.instruction("mov rsi, r8");                                         // copy the terminator index for the length computation
    emitter.instruction("sub rsi, r10");                                        // name_len = terminator index - name start
    emitter.instruction("mov DWORD PTR [rax + 8], esi");                        // store the name length into the entry
    emitter.instruction("mov DWORD PTR [rax + 12], r14d");                      // store the capture group index into the entry
    emitter.instruction("add r15, 1");                                          // one more named group has been recorded
    emitter.instruction("lea rcx, [r8 + 1]");                                   // resume scanning just past the name terminator
    emitter.instruction("jmp __rt_preg_group_names_loop_x");                    // continue the scan

    emitter.label("__rt_preg_group_names_finish_x");
    emitter.instruction("mov DWORD PTR [r13], r15d");                           // publish the recorded named-group count
    emitter.instruction("mov rax, r13");                                        // return the name-map buffer pointer
    emitter.instruction("add rsp, 16");                                         // release the pattern spill slots
    emitter.instruction("pop r15");                                             // restore the caller's r15
    emitter.instruction("pop r14");                                             // restore the caller's r14
    emitter.instruction("pop r13");                                             // restore the caller's r13
    emitter.instruction("pop r12");                                             // restore the caller's r12
    emitter.instruction("pop rbx");                                             // restore the caller's rbx
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the name-map buffer in rax

    emitter.label("__rt_preg_group_names_oom_x");
    emitter.instruction("xor eax, eax");                                        // report no name map when the allocation failed
    emitter.instruction("add rsp, 16");                                         // release the pattern spill slots
    emitter.instruction("pop r15");                                             // restore the caller's r15
    emitter.instruction("pop r14");                                             // restore the caller's r14
    emitter.instruction("pop r13");                                             // restore the caller's r13
    emitter.instruction("pop r12");                                             // restore the caller's r12
    emitter.instruction("pop rbx");                                             // restore the caller's rbx
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return a null name-map pointer in rax
}
