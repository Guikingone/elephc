//! Purpose:
//! Emits the `__rt_headers_sent` runtime helper: reads the `_headers_sent` flag stamped by
//! `__rt_stdout_write`'s real-output path and returns it as `headers_sent()`'s bool.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - `_headers_sent` is a `.comm` word emitted by `data::fixed`; the flag is set to 1 the first
//!   time real (non-buffered) output leaves the process, matching PHP's headers_sent() model.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::abi;

/// Emits `__rt_headers_sent`: returns `_headers_sent` (1 once real output has
/// left the output-buffering stack, 0 otherwise) in the int result register.
/// `_headers_sent` itself is stamped by `__rt_stdout_write`'s real-write path
/// (see `crate::codegen::runtime::io::stdout_write`), not here.
pub fn emit_headers_sent(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_headers_sent_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: headers_sent ---");
    emitter.label_global("__rt_headers_sent");
    abi::emit_symbol_address(emitter, "x9", "_headers_sent");
    emitter.instruction("ldr x0, [x9]");                                        // load the headers-sent flag as the result
    emitter.instruction("ret");
}

/// x86_64 (Linux) counterpart of `emit_headers_sent`.
fn emit_headers_sent_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: headers_sent ---");
    emitter.label_global("__rt_headers_sent");
    abi::emit_symbol_address(emitter, "r8", "_headers_sent");
    emitter.instruction("mov rax, QWORD PTR [r8]");                             // load the headers-sent flag as the result
    emitter.instruction("ret");
}
