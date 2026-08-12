//! Purpose:
//! Emits the two helpers that let `ftell()` answer PHP's position for an append stream, and
//! `fseek()` put it back in agreement with the descriptor.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - The `ftell()`, `fseek()` and `rewind()` lowerings.
//!
//! Key details:
//! - PHP maintains an append stream's position itself: `fopen($f, 'a')` on a four-byte file
//!   answers `1` after one byte is written, not `5`. The descriptor really is at 5 — `O_APPEND`
//!   puts every write at the end — so the two disagree by however much was jumped over, which
//!   `__rt_fwrite` accumulates on the stream state.
//! - Only the reported number diverges. PHP's own reads use the descriptor's offset too, so a read
//!   after an append write hits EOF on both sides and the file contents are identical.
//! - A raw descriptor has no state and no total, which is what the zero answers cover.

use crate::codegen_support::runtime::resources::layout::{
    STREAM_APPEND_SKIP_OFFSET, STREAM_WRAPPER_POS_OFFSET,
};
use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits `__rt_stream_append_skip(handle)`, answering the bytes `O_APPEND` has jumped over.
///
/// AArch64 takes and answers `x0`; x86_64 takes `rdi` and answers `rax`. Zero for every stream
/// that is not an append one, which makes the subtraction at `ftell()` unconditional.
pub fn emit_stream_append_skip(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: stream_append_skip ---");
    emitter.label_global("__rt_stream_append_skip");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("sub sp, sp, #16");
            emitter.instruction("stp x29, x30, [sp, #0]");                      // save frame pointer and return address
            emitter.instruction("mov x29, sp");
            emitter.instruction("bl __rt_stream_state");                        // x0 = the owning state, zero for a raw descriptor
            emitter.instruction("cbz x0, __rt_sas_none");
            emitter.instruction(&format!("ldr x0, [x0, #{STREAM_APPEND_SKIP_OFFSET}]"));
            emitter.instruction("ldp x29, x30, [sp, #0]");
            emitter.instruction("add sp, sp, #16");
            emitter.instruction("ret");
            emitter.label("__rt_sas_none");
            emitter.instruction("mov x0, #0");                                  // nothing was jumped over
            emitter.instruction("ldp x29, x30, [sp, #0]");
            emitter.instruction("add sp, sp, #16");
            emitter.instruction("ret");
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");
            emitter.instruction("mov rbp, rsp");
            emitter.instruction("call __rt_stream_state");                      // rax = the owning state, zero for a raw descriptor
            emitter.instruction("test rax, rax");
            emitter.instruction("jz __rt_sas_none_x86");
            emitter.instruction(&format!("mov rax, QWORD PTR [rax + {STREAM_APPEND_SKIP_OFFSET}]"));
            emitter.instruction("pop rbp");
            emitter.instruction("ret");
            emitter.label("__rt_sas_none_x86");
            emitter.instruction("xor eax, eax");                                // nothing was jumped over
            emitter.instruction("pop rbp");
            emitter.instruction("ret");
        }
    }
}

/// Emits `__rt_stream_clear_append_skip(handle)`, called after a successful seek.
///
/// A seek puts PHP's position and the descriptor back in agreement — PHP answers `0` right after
/// `fseek($h, 0)` on an append stream, and `1` after writing one more byte — so the running total
/// starts again from there. Without this the answer goes negative on the write after a seek.
pub fn emit_stream_clear_append_skip(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: stream_clear_append_skip ---");
    emitter.label_global("__rt_stream_clear_append_skip");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("sub sp, sp, #16");
            emitter.instruction("stp x29, x30, [sp, #0]");
            emitter.instruction("mov x29, sp");
            emitter.instruction("bl __rt_stream_state");                        // x0 = the owning state, zero for a raw descriptor
            emitter.instruction("cbz x0, __rt_scas_none");
            emitter.instruction(&format!("str xzr, [x0, #{STREAM_APPEND_SKIP_OFFSET}]"));
            emitter.label("__rt_scas_none");
            emitter.instruction("ldp x29, x30, [sp, #0]");
            emitter.instruction("add sp, sp, #16");
            emitter.instruction("ret");
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");
            emitter.instruction("mov rbp, rsp");
            emitter.instruction("call __rt_stream_state");                      // rax = the owning state, zero for a raw descriptor
            emitter.instruction("test rax, rax");
            emitter.instruction("jz __rt_scas_none_x86");
            emitter.instruction(&format!("mov QWORD PTR [rax + {STREAM_APPEND_SKIP_OFFSET}], 0"));
            emitter.label("__rt_scas_none_x86");
            emitter.instruction("pop rbp");
            emitter.instruction("ret");
        }
    }
}

/// Emits `__rt_stream_wrapper_pos(handle)`, answering the position PHP keeps for a wrapper stream.
///
/// php-src has no tell op for userspace wrappers: `stream_tell` is called only from inside
/// `php_userstreamop_seek`, to reconcile after a seek. The position itself is PHP's own, advanced
/// by whatever each read and write moved — which is why `ftell()` answers `7` after seven bytes
/// even when the wrapper's `stream_tell()` is written to return something else.
pub fn emit_stream_wrapper_pos(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: stream_wrapper_pos ---");
    emitter.label_global("__rt_stream_wrapper_pos");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("sub sp, sp, #16");
            emitter.instruction("stp x29, x30, [sp, #0]");
            emitter.instruction("mov x29, sp");
            emitter.instruction("bl __rt_stream_state");                        // x0 = the owning state, zero for a raw descriptor
            emitter.instruction("cbz x0, __rt_swp_none");
            emitter.instruction(&format!("ldr x0, [x0, #{STREAM_WRAPPER_POS_OFFSET}]"));
            emitter.instruction("ldp x29, x30, [sp, #0]");
            emitter.instruction("add sp, sp, #16");
            emitter.instruction("ret");
            emitter.label("__rt_swp_none");
            emitter.instruction("mov x0, #0");                                  // no state: the stream is at its start
            emitter.instruction("ldp x29, x30, [sp, #0]");
            emitter.instruction("add sp, sp, #16");
            emitter.instruction("ret");
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");
            emitter.instruction("mov rbp, rsp");
            emitter.instruction("call __rt_stream_state");                      // rax = the owning state, zero for a raw descriptor
            emitter.instruction("test rax, rax");
            emitter.instruction("jz __rt_swp_none_x86");
            emitter.instruction(&format!("mov rax, QWORD PTR [rax + {STREAM_WRAPPER_POS_OFFSET}]"));
            emitter.instruction("pop rbp");
            emitter.instruction("ret");
            emitter.label("__rt_swp_none_x86");
            emitter.instruction("xor eax, eax");                                // no state: the stream is at its start
            emitter.instruction("pop rbp");
            emitter.instruction("ret");
        }
    }
}

/// Emits `__rt_stream_wrapper_pos_advance(handle, delta)`, moving that position.
///
/// Called from the wrapper read and write paths with the bytes they actually moved, which is
/// exactly how php-src maintains `stream->position` for these streams.
pub fn emit_stream_wrapper_pos_advance(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: stream_wrapper_pos_advance ---");
    emitter.label_global("__rt_stream_wrapper_pos_advance");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("sub sp, sp, #32");
            emitter.instruction("stp x29, x30, [sp, #16]");
            emitter.instruction("add x29, sp, #16");
            emitter.instruction("str x1, [sp, #0]");                            // hold the delta across the lookup
            emitter.instruction("bl __rt_stream_state");                        // x0 = the owning state
            emitter.instruction("cbz x0, __rt_swpa_done");
            emitter.instruction(&format!("ldr x9, [x0, #{STREAM_WRAPPER_POS_OFFSET}]"));
            emitter.instruction("ldr x10, [sp, #0]");
            emitter.instruction("add x9, x9, x10");
            emitter.instruction(&format!("str x9, [x0, #{STREAM_WRAPPER_POS_OFFSET}]"));
            emitter.label("__rt_swpa_done");
            emitter.instruction("ldp x29, x30, [sp, #16]");
            emitter.instruction("add sp, sp, #32");
            emitter.instruction("ret");
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");
            emitter.instruction("mov rbp, rsp");
            emitter.instruction("sub rsp, 16");
            emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                // hold the delta across the lookup
            emitter.instruction("call __rt_stream_state");                      // rax = the owning state
            emitter.instruction("test rax, rax");
            emitter.instruction("jz __rt_swpa_done_x86");
            emitter.instruction("mov r10, QWORD PTR [rbp - 8]");
            emitter.instruction(&format!("add QWORD PTR [rax + {STREAM_WRAPPER_POS_OFFSET}], r10"));
            emitter.label("__rt_swpa_done_x86");
            emitter.instruction("mov rsp, rbp");
            emitter.instruction("pop rbp");
            emitter.instruction("ret");
        }
    }
}

/// Emits `__rt_stream_wrapper_pos_set(handle, position)`, called after a wrapper seek.
///
/// php-src reconciles here by calling `stream_tell` and taking its answer when it is an integer.
/// elephc uses the offset the seek moved to, which is the same number for any wrapper whose
/// `stream_tell()` reports its real position; a wrapper that reports something else is the one
/// case still to cover, and needs the boxed-return mask that `a7c92d475` never landed here.
pub fn emit_stream_wrapper_pos_set(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: stream_wrapper_pos_set ---");
    emitter.label_global("__rt_stream_wrapper_pos_set");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("sub sp, sp, #32");
            emitter.instruction("stp x29, x30, [sp, #16]");
            emitter.instruction("add x29, sp, #16");
            emitter.instruction("str x1, [sp, #0]");                            // hold the position across the lookup
            emitter.instruction("bl __rt_stream_state");
            emitter.instruction("cbz x0, __rt_swps_done");
            emitter.instruction("ldr x9, [sp, #0]");
            emitter.instruction(&format!("str x9, [x0, #{STREAM_WRAPPER_POS_OFFSET}]"));
            emitter.label("__rt_swps_done");
            emitter.instruction("ldp x29, x30, [sp, #16]");
            emitter.instruction("add sp, sp, #32");
            emitter.instruction("ret");
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");
            emitter.instruction("mov rbp, rsp");
            emitter.instruction("sub rsp, 16");
            emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                // hold the position across the lookup
            emitter.instruction("call __rt_stream_state");
            emitter.instruction("test rax, rax");
            emitter.instruction("jz __rt_swps_done_x86");
            emitter.instruction("mov r10, QWORD PTR [rbp - 8]");
            emitter.instruction(&format!("mov QWORD PTR [rax + {STREAM_WRAPPER_POS_OFFSET}], r10"));
            emitter.label("__rt_swps_done_x86");
            emitter.instruction("mov rsp, rbp");
            emitter.instruction("pop rbp");
            emitter.instruction("ret");
        }
    }
}
