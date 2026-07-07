//! Purpose:
//! Emits the `__rt_proc_open` runtime helper. C1a ships a loud stub that reports
//! failure so the PHP surface compiles and links on every supported target; the
//! real fork/pipe/exec implementation lands in C1b (Linux/macOS) and C1c (Windows).
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::io`.
//!
//! Key details:
//! - The stub ignores its arguments and returns -1 so the EIR boxer boxes PHP false.
//! - The C1b/C1c implementation must honor the ABI: AArch64 x0=descriptor_spec
//!   array ptr, x1=command ptr, x2=command len, x3=pipes array ptr; x86_64
//!   rdi/rsi/rdx/rcx.

use crate::codegen::{emit::Emitter, platform::Arch};

/// Emits the `__rt_proc_open` stub: returns -1 on every target (C1a surface only).
///
/// Input ABI (honored by C1b/C1c, ignored by the stub): AArch64 x0=descriptor_spec
/// array pointer, x1=command string pointer, x2=command length, x3=pipes array
/// pointer; x86_64 rdi/rsi/rdx/rcx. Output: the process descriptor, or -1 on failure.
pub fn emit_proc_open(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_proc_open_stub_x86_64(emitter);
        return;
    }
    emitter.blank();
    emitter.comment("--- runtime: proc_open (C1a stub) ---");
    emitter.label_global("__rt_proc_open");
    emitter.instruction("mov x0, #-1");                                         // stub failure: boxes as PHP false until C1b/C1c
    emitter.instruction("ret");                                                 // return the failure sentinel
}

/// Emits the Linux/Windows x86_64 `__rt_proc_open` stub (target-agnostic failure).
fn emit_proc_open_stub_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: proc_open (C1a stub) ---");
    emitter.label_global("__rt_proc_open");
    emitter.instruction("mov rax, -1");                                         // stub failure: boxes as PHP false until C1b/C1c
    emitter.instruction("ret");                                                 // return the failure sentinel
}