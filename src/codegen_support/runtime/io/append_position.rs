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

use crate::codegen_support::runtime::resources::layout::STREAM_APPEND_SKIP_OFFSET;
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
