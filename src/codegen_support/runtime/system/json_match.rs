//! Purpose:
//! Emits the JSON bracket-match table helpers: `__rt_json_match_build` records, in one pass
//! over a `json_decode()` document, the position of the closing bracket of every `[`/`{`,
//! and `__rt_json_match_free` releases it.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//! - `crate::codegen::lower_inst::builtins::json::lower_json_decode` (build before, free after).
//!
//! Key details:
//! - The array/object decoders find each element's end with a depth-counting scanner; on a
//!   nested container they jump to its recorded closing bracket instead of re-scanning it,
//!   which makes decoding linear instead of quadratic in the nesting depth.
//! - The table lives in libc memory (it never shows in `memory_get_usage()`), only exists for
//!   documents of at least 256 bytes whose brackets and strings balance, and the scanners
//!   only consult it for slices inside the indexed document. Any other document decodes
//!   exactly as before.
//! - A stale table left by an exception thrown out of `json_decode()` is freed by the next build.

use crate::codegen_support::abi::emit_symbol_address;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Smallest document length worth indexing.
const MIN_INDEXED_LEN: i64 = 256;

/// Emits `__rt_json_match_build` and `__rt_json_match_free` for the active target.
///
/// `__rt_json_match_build` takes the document in the string registers (x1/x2 on AArch64,
/// rax/rdx on x86_64) and returns with them unchanged.
pub fn emit_json_match(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

fn emit_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: json_match_build ---");
    emitter.label_global("__rt_json_match_build");
    emitter.instruction("stp x29, x30, [sp, #-48]!");                           // frame for the document across libc calls
    emitter.instruction("mov x29, sp");
    emitter.instruction("stp x1, x2, [sp, #16]");                               // keep the document registers for the caller
    emitter.instruction("str x19, [sp, #32]");                                  // callee-saved scratch for the table pointer
    emitter.instruction("bl __rt_json_match_free");                             // drop any table a thrown decode left behind
    emitter.instruction("ldr x2, [sp, #24]");                                   // document length
    emitter.instruction(&format!("cmp x2, #{}", MIN_INDEXED_LEN));              // small documents scan fast enough
    emitter.instruction("b.lt __rt_json_match_build_done");
    emitter.instruction("lsl x0, x2, #3");                                      // one 8-byte slot per document byte
    emitter.bl_c("malloc");
    emitter.instruction("cbz x0, __rt_json_match_build_done");                  // no memory: decode without the table
    emitter.instruction("mov x19, x0");                                         // the table
    emitter.instruction("ldp x1, x2, [sp, #16]");                               // document
    emitter.instruction("mov x9, #0");                                          // byte index
    emitter.instruction("mov x10, #-1");                                        // innermost open bracket (-1: none)
    emitter.instruction("mov x11, #0");                                         // inside a string?
    emitter.instruction("mov x12, #0");                                         // after a backslash?
    emitter.label("__rt_json_match_build_loop");
    emitter.instruction("cmp x9, x2");                                          // end of the document?
    emitter.instruction("b.ge __rt_json_match_build_end");
    emitter.instruction("ldrb w13, [x1, x9]");                                  // current byte
    emitter.instruction("cbz x12, 1f");                                         // not escaped
    emitter.instruction("mov x12, #0");                                         // the escaped byte is consumed
    emitter.instruction("b __rt_json_match_build_next");
    emitter.label("1");
    emitter.instruction("cbz x11, 2f");                                         // outside strings
    emitter.instruction("cmp w13, #92");                                        // backslash starts an escape
    emitter.instruction("cset x12, eq");
    emitter.instruction("cmp w13, #34");                                        // quote closes the string
    emitter.instruction("csel x11, xzr, x11, eq");
    emitter.instruction("b __rt_json_match_build_next");
    emitter.label("2");
    emitter.instruction("cmp w13, #34");                                        // quote opens a string
    emitter.instruction("b.ne 3f");
    emitter.instruction("mov x11, #1");
    emitter.instruction("b __rt_json_match_build_next");
    emitter.label("3");
    emitter.instruction("cmp w13, #91");                                        // '['
    emitter.instruction("b.eq 4f");
    emitter.instruction("cmp w13, #123");                                       // '{'
    emitter.instruction("b.ne 5f");
    emitter.label("4");
    emitter.instruction("str x10, [x19, x9, lsl #3]");                          // link to the enclosing open bracket
    emitter.instruction("mov x10, x9");                                         // this bracket is now innermost
    emitter.instruction("b __rt_json_match_build_next");
    emitter.label("5");
    emitter.instruction("cmp w13, #93");                                        // ']'
    emitter.instruction("b.eq 6f");
    emitter.instruction("cmp w13, #125");                                       // '}'
    emitter.instruction("b.ne __rt_json_match_build_next");
    emitter.label("6");
    emitter.instruction("tbnz x10, #63, __rt_json_match_build_bad");            // a close with nothing open
    emitter.instruction("ldr x14, [x19, x10, lsl #3]");                         // enclosing open bracket
    emitter.instruction("str x9, [x19, x10, lsl #3]");                          // record the match
    emitter.instruction("mov x10, x14");                                        // pop
    emitter.label("__rt_json_match_build_next");
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("b __rt_json_match_build_loop");
    emitter.label("__rt_json_match_build_end");
    emitter.instruction("tbz x10, #63, __rt_json_match_build_bad");             // an unclosed bracket
    emitter.instruction("cbnz x11, __rt_json_match_build_bad");                 // an unterminated string
    emit_symbol_address(emitter, "x9", "_json_match_base");
    emitter.instruction("str x1, [x9]");                                        // publish the indexed document
    emit_symbol_address(emitter, "x9", "_json_match_len");
    emitter.instruction("str x2, [x9]");
    emit_symbol_address(emitter, "x9", "_json_match_tbl");
    emitter.instruction("str x19, [x9]");
    emitter.instruction("b __rt_json_match_build_done");
    emitter.label("__rt_json_match_build_bad");
    emitter.instruction("mov x0, x19");                                         // unbalanced document: no table
    emitter.bl_c("free");
    emitter.label("__rt_json_match_build_done");
    emitter.instruction("ldr x19, [sp, #32]");
    emitter.instruction("ldp x1, x2, [sp, #16]");                               // the caller's document registers
    emitter.instruction("ldp x29, x30, [sp], #48");
    emitter.instruction("ret");

    emitter.blank();
    emitter.comment("--- runtime: json_match_free ---");
    emitter.label_global("__rt_json_match_free");
    emit_symbol_address(emitter, "x9", "_json_match_tbl");
    emitter.instruction("ldr x0, [x9]");                                        // current table
    emitter.instruction("cbz x0, 1f");                                          // none
    emitter.instruction("str xzr, [x9]");                                       // unpublish it first
    emitter.instruction("stp x29, x30, [sp, #-16]!");
    emitter.instruction("mov x29, sp");
    emitter.bl_c("free");
    emitter.instruction("ldp x29, x30, [sp], #16");
    emitter.label("1");
    emitter.instruction("ret");
}

fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: json_match_build ---");
    emitter.label_global("__rt_json_match_build");
    emitter.instruction("push rbp");                                            // frame for the document across libc calls
    emitter.instruction("mov rbp, rsp");
    emitter.instruction("push rbx");                                            // callee-saved scratch for the table pointer
    emitter.instruction("sub rsp, 24");                                         // document slots, 16-byte aligned frame
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // keep the document registers for the caller
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");
    emitter.instruction("call __rt_json_match_free");                           // drop any table a thrown decode left behind
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // document length
    emitter.instruction(&format!("cmp rdx, {}", MIN_INDEXED_LEN));              // small documents scan fast enough
    emitter.instruction("jl __rt_json_match_build_done");
    emitter.instruction("lea rdi, [rdx * 8]");                                  // one 8-byte slot per document byte
    emitter.instruction("call malloc");
    emitter.instruction("test rax, rax");                                       // no memory: decode without the table
    emitter.instruction("jz __rt_json_match_build_done");
    emitter.instruction("mov rbx, rax");                                        // the table
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // document
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");
    emitter.instruction("xor ecx, ecx");                                        // byte index
    emitter.instruction("mov r10, -1");                                         // innermost open bracket (-1: none)
    emitter.instruction("xor r11d, r11d");                                      // inside a string?
    emitter.instruction("xor r9d, r9d");                                        // after a backslash?
    emitter.label("__rt_json_match_build_loop");
    emitter.instruction("cmp rcx, rdx");                                        // end of the document?
    emitter.instruction("jge __rt_json_match_build_end");
    emitter.instruction("movzx r8d, BYTE PTR [rsi + rcx]");                     // current byte
    emitter.instruction("test r9, r9");                                         // escaped byte?
    emitter.instruction("jz 1f");
    emitter.instruction("xor r9d, r9d");                                        // the escaped byte is consumed
    emitter.instruction("jmp __rt_json_match_build_next");
    emitter.label("1");
    emitter.instruction("test r11, r11");                                       // inside a string?
    emitter.instruction("jz 2f");
    emitter.instruction("cmp r8d, 92");                                         // backslash starts an escape
    emitter.instruction("sete r9b");
    emitter.instruction("cmp r8d, 34");                                         // quote closes the string
    emitter.instruction("jne __rt_json_match_build_next");
    emitter.instruction("xor r11d, r11d");
    emitter.instruction("jmp __rt_json_match_build_next");
    emitter.label("2");
    emitter.instruction("cmp r8d, 34");                                         // quote opens a string
    emitter.instruction("jne 3f");
    emitter.instruction("mov r11d, 1");
    emitter.instruction("jmp __rt_json_match_build_next");
    emitter.label("3");
    emitter.instruction("cmp r8d, 91");                                         // '['
    emitter.instruction("je 4f");
    emitter.instruction("cmp r8d, 123");                                        // '{'
    emitter.instruction("jne 5f");
    emitter.label("4");
    emitter.instruction("mov QWORD PTR [rbx + rcx * 8], r10");                  // link to the enclosing open bracket
    emitter.instruction("mov r10, rcx");                                        // this bracket is now innermost
    emitter.instruction("jmp __rt_json_match_build_next");
    emitter.label("5");
    emitter.instruction("cmp r8d, 93");                                         // ']'
    emitter.instruction("je 6f");
    emitter.instruction("cmp r8d, 125");                                        // '}'
    emitter.instruction("jne __rt_json_match_build_next");
    emitter.label("6");
    emitter.instruction("test r10, r10");                                       // a close with nothing open
    emitter.instruction("js __rt_json_match_build_bad");
    emitter.instruction("mov rdi, QWORD PTR [rbx + r10 * 8]");                  // enclosing open bracket
    emitter.instruction("mov QWORD PTR [rbx + r10 * 8], rcx");                  // record the match
    emitter.instruction("mov r10, rdi");                                        // pop
    emitter.label("__rt_json_match_build_next");
    emitter.instruction("add rcx, 1");
    emitter.instruction("jmp __rt_json_match_build_loop");
    emitter.label("__rt_json_match_build_end");
    emitter.instruction("test r10, r10");                                       // an unclosed bracket
    emitter.instruction("jns __rt_json_match_build_bad");
    emitter.instruction("test r11, r11");                                       // an unterminated string
    emitter.instruction("jnz __rt_json_match_build_bad");
    emit_symbol_address(emitter, "r8", "_json_match_base");
    emitter.instruction("mov QWORD PTR [r8], rsi");                             // publish the indexed document
    emit_symbol_address(emitter, "r8", "_json_match_len");
    emitter.instruction("mov QWORD PTR [r8], rdx");
    emit_symbol_address(emitter, "r8", "_json_match_tbl");
    emitter.instruction("mov QWORD PTR [r8], rbx");
    emitter.instruction("jmp __rt_json_match_build_done");
    emitter.label("__rt_json_match_build_bad");
    emitter.instruction("mov rdi, rbx");                                        // unbalanced document: no table
    emitter.instruction("call free");
    emitter.label("__rt_json_match_build_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // the caller's document registers
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");
    emitter.instruction("mov rbx, QWORD PTR [rbp - 8]");
    emitter.instruction("leave");
    emitter.instruction("ret");

    emitter.blank();
    emitter.comment("--- runtime: json_match_free ---");
    emitter.label_global("__rt_json_match_free");
    emit_symbol_address(emitter, "r8", "_json_match_tbl");
    emitter.instruction("mov rdi, QWORD PTR [r8]");                             // current table
    emitter.instruction("test rdi, rdi");
    emitter.instruction("jz 1f");                                               // none
    emitter.instruction("mov QWORD PTR [r8], 0");                               // unpublish it first
    emitter.instruction("push rbp");                                            // align the stack for libc
    emitter.instruction("mov rbp, rsp");
    emitter.instruction("call free");
    emitter.instruction("pop rbp");
    emitter.label("1");
    emitter.instruction("ret");
}

/// Emits the nested-container jump of a JSON boundary scanner: when the scanner meets an open
/// bracket at depth 0 inside the indexed document, the cursor moves to the recorded closing
/// bracket and control continues at `advance` (which steps past it, depth unchanged).
/// Otherwise control falls through to the caller's ordinary open-bracket handling.
pub(crate) fn emit_scanner_jump(
    emitter: &mut Emitter,
    slice_reg: &str,
    cursor_reg: &str,
    depth_reg: &str,
    advance: &str,
) {
    let fallthrough = format!("{}_nojump", advance);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("cbnz {}, {}", depth_reg, fallthrough)); // only a top-level element can jump
            emit_symbol_address(emitter, "x16", "_json_match_tbl");
            emitter.instruction("ldr x16, [x16]");                              // the match table
            emitter.instruction(&format!("cbz x16, {}", fallthrough));          // none for this document
            emit_symbol_address(emitter, "x17", "_json_match_base");
            emitter.instruction("ldr x17, [x17]");
            emitter.instruction(&format!("sub x17, {}, x17", slice_reg));       // slice offset in the document
            emitter.instruction(&format!("add x17, x17, {}", cursor_reg));      // document index of the bracket
            emit_symbol_address(emitter, "x15", "_json_match_len");
            emitter.instruction("ldr x15, [x15]");
            emitter.instruction("cmp x17, x15");                                // inside the indexed document?
            emitter.instruction(&format!("b.hs {}", fallthrough));
            emitter.instruction("ldr x15, [x16, x17, lsl #3]");                 // its closing bracket
            emitter.instruction("sub x15, x15, x17");                           // distance to it
            emitter.instruction(&format!("add {}, {}, x15", cursor_reg, cursor_reg)); // jump over the container
            emitter.instruction(&format!("b {}", advance));
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("test {}, {}", depth_reg, depth_reg));  // only a top-level element can jump
            emitter.instruction(&format!("jnz {}", fallthrough));
            emit_symbol_address(emitter, "r9", "_json_match_tbl");
            emitter.instruction("mov r9, QWORD PTR [r9]");                      // the match table
            emitter.instruction("test r9, r9");
            emitter.instruction(&format!("jz {}", fallthrough));                // none for this document
            emit_symbol_address(emitter, "r8", "_json_match_base");
            emitter.instruction("mov r8, QWORD PTR [r8]");
            emitter.instruction("neg r8");
            emitter.instruction(&format!("add r8, {}", slice_reg));             // slice offset in the document
            emitter.instruction(&format!("add r8, {}", cursor_reg));            // document index of the bracket
            emitter.instruction("push r9");                                     // keep the table across the length load
            emit_symbol_address(emitter, "r9", "_json_match_len");
            emitter.instruction("cmp r8, QWORD PTR [r9]");                      // inside the indexed document?
            emitter.instruction("pop r9");
            emitter.instruction(&format!("jae {}", fallthrough));
            emitter.instruction("mov r9, QWORD PTR [r9 + r8 * 8]");             // its closing bracket
            emitter.instruction("sub r9, r8");                                  // distance to it
            emitter.instruction(&format!("add {}, r9", cursor_reg));            // jump over the container
            emitter.instruction(&format!("jmp {}", advance));
        }
    }
    emitter.label(&fallthrough);
}
