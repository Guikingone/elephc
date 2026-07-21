//! Purpose:
//! Emits `__rt_mixed_array_payload_or_fatal`, the strict gradual-boundary assert that returns
//! the *borrowed* indexed-array payload of a boxed `Mixed` value. Used by `array_column`, whose
//! runtime helper reads the original hash rows in place rather than a boxed-Mixed rebuild.
//!
//! Called from:
//! - `crate::codegen_support::runtime::arrays::emit_mixed_array_payload_or_fatal()` via the top-level runtime emitter.
//! - The EIR `array_column` Mixed/union operand lowering in
//!   `crate::codegen::lower_inst::builtins::arrays::column`.
//!
//! Key details:
//! - Unboxes the Mixed cell; an indexed-array payload (runtime tag 4) is returned unchanged as a
//!   borrowed pointer, so the source array and its rows keep their existing refcounts.
//! - Every other payload (`false`/bool, null, scalar, associative array, object) fatals with a
//!   `TypeError`, matching PHP 8's `array_column(false, …)` rather than silently yielding `[]`.
//! - Unlike `__rt_mixed_array_or_fatal`, the payload is *not* rebuilt into boxed-Mixed slots: the
//!   caller only reads the rows, so the raw hash rows must be preserved for the hash lookups.

use crate::codegen::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Length in bytes of the `_array_arg_type_error_msg` runtime fatal string.
const ARRAY_ARG_TYPE_ERROR_MSG_LEN: usize = 78;

/// Emits `__rt_mixed_array_payload_or_fatal(mixed_ptr) -> array_ptr` for the active target.
///
/// Asserts the boxed Mixed value is an indexed array and returns its borrowed payload pointer;
/// any non-array payload fatals with a `TypeError` diagnostic and exits.
pub fn emit_mixed_array_payload_or_fatal(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_array_payload_or_fatal_x86_64(emitter);
        return;
    }
    emit_mixed_array_payload_or_fatal_aarch64(emitter);
}

/// ARM64 implementation of `__rt_mixed_array_payload_or_fatal` (input `x0`, result `x0`).
fn emit_mixed_array_payload_or_fatal_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_payload_or_fatal ---");
    emitter.label_global("__rt_mixed_array_payload_or_fatal");

    emitter.instruction("sub sp, sp, #16");                                     // reserve space for the saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #0]");                              // save frame pointer and return address across the unbox call

    // -- unbox and require an indexed-array payload --
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0 = runtime tag, x1 = payload low word
    emitter.instruction("cmp x0, #4");                                          // tag 4 = indexed array?
    emitter.instruction("b.ne __rt_mixed_array_payload_or_fatal_bad");          // any non-array payload is a TypeError
    emitter.instruction("cbz x1, __rt_mixed_array_payload_or_fatal_bad");       // a null array payload is also a TypeError
    emitter.instruction("mov x0, x1");                                          // return the borrowed indexed-array payload pointer
    emitter.instruction("ldp x29, x30, [sp, #0]");                              // restore frame pointer and return address
    emitter.instruction("add sp, sp, #16");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the borrowed payload

    // -- non-array payload: emit the TypeError and terminate the process --
    emitter.label("__rt_mixed_array_payload_or_fatal_bad");
    abi::emit_symbol_address(emitter, "x1", "_array_arg_type_error_msg");
    emitter.instruction(&format!("mov x2, #{}", ARRAY_ARG_TYPE_ERROR_MSG_LEN)); // pass the array-argument fatal message length
    emitter.instruction("mov x0, #2");                                          // write the diagnostic to stderr
    emitter.syscall(4);
    emitter.instruction("mov x0, #70");                                         // use EX_SOFTWARE as the process exit status
    emitter.syscall(1);
}

/// x86_64 implementation of `__rt_mixed_array_payload_or_fatal` (input `rdi`, result `rax`).
fn emit_mixed_array_payload_or_fatal_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_payload_or_fatal ---");
    emitter.label_global("__rt_mixed_array_payload_or_fatal");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base

    // -- unbox and require an indexed-array payload --
    emitter.instruction("mov rax, rdi");                                        // __rt_mixed_unbox reads the boxed pointer from rax
    emitter.instruction("call __rt_mixed_unbox");                               // rax = runtime tag, rdi = payload low word
    emitter.instruction("cmp rax, 4");                                          // tag 4 = indexed array?
    emitter.instruction("jne __rt_mixed_array_payload_or_fatal_bad");           // any non-array payload is a TypeError
    emitter.instruction("test rdi, rdi");                                       // is the array payload null?
    emitter.instruction("je __rt_mixed_array_payload_or_fatal_bad");            // a null array payload is also a TypeError
    emitter.instruction("mov rax, rdi");                                        // return the borrowed indexed-array payload pointer
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the borrowed payload

    // -- non-array payload: emit the TypeError and terminate the process --
    emitter.label("__rt_mixed_array_payload_or_fatal_bad");
    emitter.instruction("mov edi, 2");                                          // write the diagnostic to Linux stderr
    abi::emit_symbol_address(emitter, "rsi", "_array_arg_type_error_msg");      // point the write() buffer at the fatal message
    emitter.instruction(&format!("mov edx, {}", ARRAY_ARG_TYPE_ERROR_MSG_LEN)); // pass the array-argument fatal message length
    emitter.instruction("mov eax, 1");                                          // Linux x86_64 syscall 1 = write
    emitter.instruction("syscall");                                             // emit the array-argument fatal before exiting
    emitter.instruction("mov edi, 70");                                         // use EX_SOFTWARE as the process exit status
    emitter.instruction("mov eax, 60");                                         // Linux x86_64 syscall 60 = exit
    emitter.instruction("syscall");                                             // terminate the process after the fatal diagnostic
}
