//! Purpose:
//! Emits the `__rt_proc_close` runtime helper. C1a ships a loud stub that reports
//! failure so the PHP surface compiles and links on every supported target; the real
//! waitpid/reap implementation lands in C1b (Linux/macOS) and C1c (Windows).
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::io`,
//!   and from `__rt_mixed_free_deep` as the kind-5 resource destructor.
//!
//! Key details:
//! - The stub ignores its argument and returns -1. It is also the kind-5 destructor
//!   target, so a leaked proc resource is a no-op until C1b/C1c.

use crate::codegen::{emit::Emitter, platform::Arch};

/// Emits the `__rt_proc_close` stub: returns -1 on every target (C1a surface only).
///
/// Input ABI (honored by C1b/C1c, ignored by the stub): AArch64 x0=process
/// descriptor; x86_64 rdi=process descriptor. Output: the child exit status, or -1.
pub fn emit_proc_close(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_proc_close_stub_x86_64(emitter);
        return;
    }
    emitter.blank();
    emitter.comment("--- runtime: proc_close (C1a stub) ---");
    emitter.label_global("__rt_proc_close");
    emitter.instruction("mov x0, #-1");                                         // stub failure until C1b/C1c
    emitter.instruction("ret");                                                 // return the failure sentinel
}

/// Emits the Linux/Windows x86_64 `__rt_proc_close` stub (target-agnostic failure).
fn emit_proc_close_stub_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: proc_close (C1a stub) ---");
    emitter.label_global("__rt_proc_close");
    emitter.instruction("mov rax, -1");                                         // stub failure until C1b/C1c
    emitter.instruction("ret");                                                 // return the failure sentinel
}