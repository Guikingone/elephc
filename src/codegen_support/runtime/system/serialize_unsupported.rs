//! Purpose:
//! Emits the `__rt_serialize_unsupported` runtime helper assembly for the deferred
//! serialize()/unserialize() fatal stub.
//! Keeps the unsupported-serialization fatal path target-aware alongside the other runtime fatals.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - serialize()/unserialize() are recognized at the type level so call sites resolve, but the
//!   full PHP serialization format is not implemented yet. Reaching this helper means the program
//!   exercised the unsupported path, so it writes a diagnostic to stderr and terminates.

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Emits the `__rt_serialize_unsupported` runtime helper for both AArch64 and x86_64.
///
/// This fatal handler writes a hardcoded 60-byte error message to stderr and terminates the
/// process with exit code 70 (EX_SOFTWARE). It is invoked by generated code when a program
/// actually calls `serialize()` or `unserialize()`, neither of which is fully implemented yet.
///
/// AArch64 path: uses syscall 4 (sys_write) then syscall 1 (sys_exit).
/// x86_64 path: uses syscall 1 (write) then syscall 60 (exit).
pub fn emit_serialize_unsupported(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: serialize_unsupported ---");
    emitter.label_global("__rt_serialize_unsupported");
    match emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(emitter, "x1", "_serialize_unsupported_msg"); // load the unsupported-serialization error message page for the AArch64 fatal path
            emitter.instruction("mov x2, #60");                                 // byte length of the unsupported-serialization error message
            emitter.instruction("mov x0, #2");                                  // write diagnostics to stderr on the AArch64 fatal path
            emitter.syscall(4);
            emitter.instruction("mov x0, #70");                                 // use EX_SOFTWARE as the process exit status on the AArch64 fatal path
            emitter.syscall(1);
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(emitter, "rsi", "_serialize_unsupported_msg"); // materialize the unsupported-serialization error message address for the x86_64 fatal path
            emitter.instruction("mov edx, 60");                                 // byte length of the unsupported-serialization error message
            emitter.instruction("mov edi, 2");                                  // write diagnostics to stderr on the x86_64 fatal path
            emitter.instruction("mov eax, 1");                                  // Linux x86_64 syscall number 1 = write
            emitter.instruction("syscall");                                     // emit the unsupported-serialization fatal diagnostic on x86_64
            emitter.instruction("mov edi, 70");                                 // use EX_SOFTWARE as the process exit status on the x86_64 fatal path
            emitter.instruction("mov eax, 60");                                 // Linux x86_64 syscall number 60 = exit
            emitter.instruction("syscall");                                     // terminate the process after reporting the unsupported serialization call
        }
    }
}
