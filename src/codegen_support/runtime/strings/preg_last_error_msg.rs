//! Purpose:
//! Emits the `__rt_preg_last_error_msg` and `__rt_preg_last_error` runtime helpers.
//! The minimal implementation always returns `"No error"` and 0 respectively, since
//! elephc's PCRE engine does not record per-call error state.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - AArch64 output: x1=ptr, x2=len for string; x0=int for error code.
//! - x86_64 output: rax=ptr, rdx=len for string; rax=int for error code.
//! - The static string `_rt_preg_no_error_str` ("No error", 8 bytes) is defined in
//!   the fixed runtime data section and referenced by address here.

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Emits `__rt_preg_last_error_msg` — loads the address and length of the static
/// `"No error"` string and returns them in the string-result registers.
///
/// # Output registers
/// - AArch64: x1 = string pointer, x2 = string length (8)
/// - x86_64: rax = string pointer, rdx = string length (8)
pub fn emit_preg_last_error_msg(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_preg_last_error_msg_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: preg_last_error_msg ---");
    emitter.label_global("__rt_preg_last_error_msg");
    abi::emit_symbol_address(emitter, "x1", "_rt_preg_no_error_str");           // load address of "No error" string into string pointer register
    emitter.instruction("mov x2, #8");                                          // length of "No error" = 8 bytes
    emitter.instruction("ret");                                                 // return ptr in x1, len in x2
}

/// Emits the x86_64 `__rt_preg_last_error_msg`.
fn emit_preg_last_error_msg_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: preg_last_error_msg ---");
    emitter.label_global("__rt_preg_last_error_msg");
    abi::emit_symbol_address(emitter, "rax", "_rt_preg_no_error_str");          // load address of "No error" string into string pointer register (rax)
    emitter.instruction("mov rdx, 8");                                          // length of "No error" = 8 bytes
    emitter.instruction("ret");                                                 // return ptr in rax, len in rdx
}

/// Emits `__rt_preg_last_error` — always returns 0 (PREG_NO_ERROR).
///
/// # Output registers
/// - AArch64: x0 = 0
/// - x86_64: rax = 0
pub fn emit_preg_last_error(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: preg_last_error ---");
    emitter.label_global("__rt_preg_last_error");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("mov x0, #0");                                  // PREG_NO_ERROR = 0
            emitter.instruction("ret");                                         // return 0 in x0
        }
        Arch::X86_64 => {
            emitter.instruction("xor eax, eax");                                // PREG_NO_ERROR = 0
            emitter.instruction("ret");                                         // return 0 in rax
        }
    }
}
