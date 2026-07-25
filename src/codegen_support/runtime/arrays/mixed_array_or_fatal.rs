//! Purpose:
//! Emits `__rt_mixed_array_or_fatal`, the strict gradual-boundary assert used by array
//! builtins whose PHP-8 contract is a `TypeError` on a non-array argument
//! (`array_reverse`, `array_column`).
//!
//! Called from:
//! - `crate::codegen_support::runtime::arrays::emit_mixed_array_or_fatal()` via the top-level runtime emitter.
//! - The EIR `array_reverse` / `array_column` Mixed/union operand lowering in
//!   `crate::codegen::lower_inst::builtins::arrays`.
//!
//! Key details:
//! - Unboxes the Mixed cell; an indexed-array payload (runtime tag 4) is rebuilt into a fresh
//!   owned `array<mixed>` by delegating to `__rt_array_from_mixed`, so the result reads back
//!   through the ordinary boxed-Mixed indexed-array path.
//! - Every other payload (`false`/bool, null, scalar, associative array, object) fatals with a
//!   `TypeError`. Unlike `__rt_mixed_to_owned_hash`, this helper accepts indexed arrays only;
//!   unlike `__rt_mixed_count`, it never quietly maps a non-array to zero. This matches PHP 8,
//!   where `array_reverse(false)` and `array_column(false, …)` throw.
//! - The boxed Mixed argument is only borrowed; the returned array is independently owned
//!   (refcount 1), so a single release frees it.

use crate::codegen::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Length in bytes of the `_array_arg_type_error_msg` runtime fatal string.
const ARRAY_ARG_TYPE_ERROR_MSG_LEN: usize = 78;

/// Emits `__rt_mixed_array_or_fatal(mixed_ptr) -> array_ptr` for the active target.
///
/// Asserts that the boxed Mixed value is an indexed array, rebuilding it into a fresh owned
/// `array<mixed>`; any non-array payload fatals with a `TypeError` diagnostic and exits.
pub fn emit_mixed_array_or_fatal(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_array_or_fatal_x86_64(emitter);
        return;
    }
    emit_mixed_array_or_fatal_aarch64(emitter);
}

/// ARM64 implementation of `__rt_mixed_array_or_fatal` (input `x0`, result `x0`).
fn emit_mixed_array_or_fatal_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_or_fatal ---");
    emitter.label_global("__rt_mixed_array_or_fatal");

    // -- set up frame; [sp,#0] holds the borrowed boxed Mixed pointer --
    emitter.instruction("sub sp, sp, #32");                                     // reserve a save slot plus saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish the local frame
    emitter.instruction("str x0, [sp, #0]");                                    // save the borrowed boxed Mixed pointer

    // -- unbox and require an indexed-array payload --
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0 = concrete runtime tag of the boxed value
    emitter.instruction("cmp x0, #4");                                          // tag 4 = indexed array?
    emitter.instruction("b.ne __rt_mixed_array_or_fatal_bad");                  // any non-array payload is a TypeError

    // -- indexed array: rebuild a fresh owned array<mixed> via the (array)-cast dispatcher --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the boxed Mixed pointer for the rebuild
    emitter.instruction("bl __rt_array_from_mixed");                            // x0 = independently owned array<mixed>
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the local frame
    emitter.instruction("ret");                                                 // return the rebuilt owned array

    // -- non-array payload: emit the TypeError and terminate the process --
    emitter.label("__rt_mixed_array_or_fatal_bad");
    abi::emit_symbol_address(emitter, "x1", "_array_arg_type_error_msg");
    emitter.instruction(&format!("mov x2, #{}", ARRAY_ARG_TYPE_ERROR_MSG_LEN)); // pass the array-argument fatal message length
    emitter.instruction("mov x0, #2");                                          // write the diagnostic to stderr
    emitter.syscall(4);
    emitter.instruction("mov x0, #70");                                         // use EX_SOFTWARE as the process exit status
    emitter.syscall(1);
}

/// x86_64 implementation of `__rt_mixed_array_or_fatal` (input `rdi`, result `rax`).
fn emit_mixed_array_or_fatal_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_array_or_fatal ---");
    emitter.label_global("__rt_mixed_array_or_fatal");

    // -- set up frame; [rbp-8] holds the borrowed boxed Mixed pointer --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 16");                                         // reserve an aligned save slot
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the borrowed boxed Mixed pointer

    // -- unbox and require an indexed-array payload --
    emitter.instruction("mov rax, rdi");                                        // __rt_mixed_unbox reads the boxed pointer from rax
    emitter.instruction("call __rt_mixed_unbox");                               // rax = concrete runtime tag of the boxed value
    emitter.instruction("cmp rax, 4");                                          // tag 4 = indexed array?
    emitter.instruction("jne __rt_mixed_array_or_fatal_bad");                   // any non-array payload is a TypeError

    // -- indexed array: rebuild a fresh owned array<mixed> via the (array)-cast dispatcher --
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the boxed Mixed pointer for the rebuild
    emitter.instruction("call __rt_array_from_mixed");                          // rax = independently owned array<mixed>
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the rebuilt owned array

    // -- non-array payload: emit the TypeError and terminate the process --
    emitter.label("__rt_mixed_array_or_fatal_bad");
    emitter.instruction("mov edi, 2");                                          // write the diagnostic to Linux stderr
    abi::emit_symbol_address(emitter, "rsi", "_array_arg_type_error_msg");      // point the write() buffer at the fatal message
    emitter.instruction(&format!("mov edx, {}", ARRAY_ARG_TYPE_ERROR_MSG_LEN)); // pass the array-argument fatal message length
    emitter.instruction("mov eax, 1");                                          // Linux x86_64 syscall 1 = write
    emitter.instruction("syscall");                                             // emit the array-argument fatal before exiting
    emitter.instruction("mov edi, 70");                                         // use EX_SOFTWARE as the process exit status
    emitter.instruction("mov eax, 60");                                         // Linux x86_64 syscall 60 = exit
    emitter.instruction("syscall");                                             // terminate the process after the fatal diagnostic
}
