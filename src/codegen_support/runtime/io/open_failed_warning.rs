//! Purpose:
//! Emits `__rt_open_failed_warning`, the "Failed to open stream" diagnostic with the path
//! and the reason PHP puts in it.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - The `__rt_fopen` and `__rt_file_get_contents` failure paths.
//!
//! Key details:
//! - php-src writes `fopen(/no/such/file): Failed to open stream: No such file or directory`.
//!   elephc wrote `fopen(): Failed to open stream` — neither WHICH path failed nor WHY, which
//!   is most of what the message is for when several opens share one line.
//! - The caller supplies the prefix, so `fopen()` and `file_get_contents()` name themselves
//!   without a branch in here, and passes a POSITIVE errno: the two platforms report a failed
//!   syscall differently and normalising at the call site keeps that knowledge where the
//!   syscall is.
//! - Both the path and the reason are clamped: a diagnostic is never worth writing past the
//!   buffer into the neighbouring globals.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Bytes reserved for the composed message.
pub(crate) const OPEN_FAILED_MSG_CAPACITY: usize = 512;

/// The most path bytes copied into the message.
const PATH_CLAMP: usize = 300;

/// The most reason bytes copied into the message.
const REASON_CLAMP: usize = 120;

/// The fixed text between the path and the reason.
pub(crate) const OPEN_FAILED_MIDDLE: &str = "): Failed to open stream: ";

/// Emits `__rt_open_failed_warning(prefix_ptr, prefix_len, path_cstr, errno)`.
///
/// AArch64 takes `x0`/`x1`/`x2`/`x3`; x86_64 takes `rdi`/`rsi`/`rdx`/`rcx`.
pub fn emit_open_failed_warning(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// Emits the AArch64 composer.
fn emit_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: compose the failed-open warning ---");
    emitter.label_global("__rt_open_failed_warning");
    // Frame: [0] destination cursor, [8] saved errno, [16] saved path pointer, [32] linkage.
    emitter.instruction("sub sp, sp, #48");
    emitter.instruction("stp x29, x30, [sp, #32]");
    emitter.instruction("add x29, sp, #32");
    emitter.instruction("str x3, [sp, #8]");                                    // hold errno across the copies
    emitter.instruction("str x2, [sp, #16]");                                   // hold the path across the copies

    abi::emit_symbol_address(emitter, "x9", "_open_failed_msg");                // the destination buffer
    emitter.instruction("mov x10, #0");                                         // bytes written so far

    // -- the caller's prefix, ending just before the path --
    emitter.instruction("mov x11, #0");
    emitter.label("__rt_ofw_prefix");
    emitter.instruction("cmp x11, x1");
    emitter.instruction("b.hs __rt_ofw_path");
    emitter.instruction("ldrb w12, [x0, x11]");
    emitter.instruction("strb w12, [x9, x10]");
    emitter.instruction("add x10, x10, #1");
    emitter.instruction("add x11, x11, #1");
    emitter.instruction("b __rt_ofw_prefix");

    // -- the path, up to its NUL --
    emitter.label("__rt_ofw_path");
    emitter.instruction("ldr x13, [sp, #16]");                                  // the path pointer
    emitter.instruction("cbz x13, __rt_ofw_middle");                            // no path to name
    emitter.instruction("mov x11, #0");
    emitter.label("__rt_ofw_path_loop");
    emitter.instruction(&format!("cmp x11, #{PATH_CLAMP}"));
    emitter.instruction("b.hs __rt_ofw_middle");
    emitter.instruction("ldrb w12, [x13, x11]");
    emitter.instruction("cbz w12, __rt_ofw_middle");                            // the terminator ends the path
    emitter.instruction("strb w12, [x9, x10]");
    emitter.instruction("add x10, x10, #1");
    emitter.instruction("add x11, x11, #1");
    emitter.instruction("b __rt_ofw_path_loop");

    // -- "): Failed to open stream: " --
    emitter.label("__rt_ofw_middle");
    abi::emit_symbol_address(emitter, "x14", "_diag_open_failed_middle");
    emitter.instruction("mov x11, #0");
    emitter.label("__rt_ofw_middle_loop");
    emitter.instruction(&format!("cmp x11, #{}", OPEN_FAILED_MIDDLE.len()));
    emitter.instruction("b.hs __rt_ofw_reason");
    emitter.instruction("ldrb w12, [x14, x11]");
    emitter.instruction("strb w12, [x9, x10]");
    emitter.instruction("add x10, x10, #1");
    emitter.instruction("add x11, x11, #1");
    emitter.instruction("b __rt_ofw_middle_loop");

    // -- the platform's own wording for the errno --
    emitter.label("__rt_ofw_reason");
    emitter.instruction("str x9, [sp, #0]");                                    // the buffer base survives the libc call
    emitter.instruction("str x10, [sp, #24]");                                  // and so does the cursor
    emitter.instruction("ldr x0, [sp, #8]");                                    // the errno to describe
    emitter.bl_c("strerror");                                                   // x0 = static NUL-terminated reason
    emitter.instruction("mov x13, x0");                                         // the reason text
    emitter.instruction("ldr x9, [sp, #0]");                                    // the buffer base again
    emitter.instruction("ldr x10, [sp, #24]");                                  // and the cursor
    emitter.instruction("cbz x13, __rt_ofw_tail");                              // an unknown code has no text
    emitter.instruction("mov x11, #0");
    emitter.label("__rt_ofw_reason_loop");
    emitter.instruction(&format!("cmp x11, #{REASON_CLAMP}"));
    emitter.instruction("b.hs __rt_ofw_tail");
    emitter.instruction("ldrb w12, [x13, x11]");
    emitter.instruction("cbz w12, __rt_ofw_tail");
    emitter.instruction("strb w12, [x9, x10]");
    emitter.instruction("add x10, x10, #1");
    emitter.instruction("add x11, x11, #1");
    emitter.instruction("b __rt_ofw_reason_loop");

    emitter.label("__rt_ofw_tail");
    emitter.instruction("mov w12, #0x0a");                                      // '\n'
    emitter.instruction("strb w12, [x9, x10]");
    emitter.instruction("add x10, x10, #1");

    emitter.instruction("mov x1, x9");                                          // message pointer
    emitter.instruction("mov x2, x10");                                         // message length
    emitter.instruction("bl __rt_diag_warning");                                // stderr, and `@` suppresses it

    emitter.instruction("ldp x29, x30, [sp, #32]");
    emitter.instruction("add sp, sp, #48");
    emitter.instruction("ret");
}

/// Emits the Linux x86_64 composer.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: compose the failed-open warning ---");
    emitter.label_global("__rt_open_failed_warning");
    emitter.instruction("push rbp");
    emitter.instruction("mov rbp, rsp");
    emitter.instruction("sub rsp, 48");
    emitter.instruction("mov QWORD PTR [rbp - 8], rcx");                        // hold errno across the copies
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // hold the path across the copies

    abi::emit_symbol_address(emitter, "r9", "_open_failed_msg");                // the destination buffer
    emitter.instruction("xor r10d, r10d");                                      // bytes written so far

    // -- the caller's prefix, ending just before the path --
    emitter.instruction("xor r11d, r11d");
    emitter.label("__rt_ofw_prefix_x86");
    emitter.instruction("cmp r11, rsi");
    emitter.instruction("jae __rt_ofw_path_x86");
    emitter.instruction("movzx eax, BYTE PTR [rdi + r11]");
    emitter.instruction("mov BYTE PTR [r9 + r10], al");
    emitter.instruction("inc r10");
    emitter.instruction("inc r11");
    emitter.instruction("jmp __rt_ofw_prefix_x86");

    // -- the path, up to its NUL --
    emitter.label("__rt_ofw_path_x86");
    emitter.instruction("mov r8, QWORD PTR [rbp - 16]");
    emitter.instruction("test r8, r8");
    emitter.instruction("jz __rt_ofw_middle_x86");
    emitter.instruction("xor r11d, r11d");
    emitter.label("__rt_ofw_path_loop_x86");
    emitter.instruction(&format!("cmp r11, {PATH_CLAMP}"));
    emitter.instruction("jae __rt_ofw_middle_x86");
    emitter.instruction("movzx eax, BYTE PTR [r8 + r11]");
    emitter.instruction("test al, al");
    emitter.instruction("jz __rt_ofw_middle_x86");
    emitter.instruction("mov BYTE PTR [r9 + r10], al");
    emitter.instruction("inc r10");
    emitter.instruction("inc r11");
    emitter.instruction("jmp __rt_ofw_path_loop_x86");

    // -- "): Failed to open stream: " --
    emitter.label("__rt_ofw_middle_x86");
    abi::emit_symbol_address(emitter, "r8", "_diag_open_failed_middle");
    emitter.instruction("xor r11d, r11d");
    emitter.label("__rt_ofw_middle_loop_x86");
    emitter.instruction(&format!("cmp r11, {}", OPEN_FAILED_MIDDLE.len()));
    emitter.instruction("jae __rt_ofw_reason_x86");
    emitter.instruction("movzx eax, BYTE PTR [r8 + r11]");
    emitter.instruction("mov BYTE PTR [r9 + r10], al");
    emitter.instruction("inc r10");
    emitter.instruction("inc r11");
    emitter.instruction("jmp __rt_ofw_middle_loop_x86");

    // -- the platform's own wording for the errno --
    emitter.label("__rt_ofw_reason_x86");
    emitter.instruction("mov QWORD PTR [rbp - 24], r9");                        // the buffer base survives the libc call
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");                       // and so does the cursor
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // the errno to describe
    emitter.instruction("call strerror");                                       // rax = static NUL-terminated reason
    emitter.instruction("mov r8, rax");                                         // the reason text
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // the buffer base again
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // and the cursor
    emitter.instruction("test r8, r8");
    emitter.instruction("jz __rt_ofw_tail_x86");
    emitter.instruction("xor r11d, r11d");
    emitter.label("__rt_ofw_reason_loop_x86");
    emitter.instruction(&format!("cmp r11, {REASON_CLAMP}"));
    emitter.instruction("jae __rt_ofw_tail_x86");
    emitter.instruction("movzx eax, BYTE PTR [r8 + r11]");
    emitter.instruction("test al, al");
    emitter.instruction("jz __rt_ofw_tail_x86");
    emitter.instruction("mov BYTE PTR [r9 + r10], al");
    emitter.instruction("inc r10");
    emitter.instruction("inc r11");
    emitter.instruction("jmp __rt_ofw_reason_loop_x86");

    emitter.label("__rt_ofw_tail_x86");
    emitter.instruction("mov BYTE PTR [r9 + r10], 0x0a");                       // '\n'
    emitter.instruction("inc r10");

    emitter.instruction("mov rdi, r9");                                         // message pointer
    emitter.instruction("mov rsi, r10");                                        // message length
    emitter.instruction("call __rt_diag_warning");                              // stderr, and `@` suppresses it

    emitter.instruction("mov rsp, rbp");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");
}
