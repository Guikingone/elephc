//! Purpose:
//! Emits the `__rt_mkdir_mode` runtime helper: a single-directory `mkdir(path,
//! mode)` with a REAL caller-supplied permission mode, backing `mkdir($dir,
//! $permissions)` and used as the leaf/prefix primitive by `__rt_mkdir_recursive`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - `crate::codegen::lower_inst::builtins::io::lower_mkdir()` (the `mkdir()`
//!   EIR lowering) when `$permissions`/`$recursive` are explicitly passed.
//! - `crate::codegen_support::runtime::io::mkdir_recursive::emit_mkdir_recursive()`.
//!
//! Key details:
//! - Distinct from the shared `__rt_mkdir(path)` (single-arg, hardcoded 0777
//!   default, still used by the frozen legacy backend and the wrapper-aware
//!   1-arg EIR path) — this helper threads a real mode through instead.
//! - php-verified (PHP 8.5.6 local, `php -n`): `mkdir()` on an existing
//!   directory fails (`EEXIST`) and returns `false`; the kernel applies the
//!   process umask to the requested mode, matching PHP exactly (elephc just
//!   passes the raw requested mode to the syscall/libc call).

use crate::codegen::{emit::Emitter, platform::Arch};

/// Emits `__rt_mkdir_mode` for the host target.
///
/// AArch64: input x1=ptr, x2=len, x3=mode. Output: x0=1 on success, 0 on failure.
/// x86_64: input rax=ptr, rdx=len, rdi=mode. Output: rax=1 on success, 0 on failure.
pub fn emit_mkdir_mode(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mkdir_mode_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: mkdir_mode ---");
    emitter.label_global("__rt_mkdir_mode");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #32");                                     // allocate frame + spill slot for mode
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish new frame pointer
    emitter.instruction("str x3, [sp, #0]");                                    // preserve the requested mode across the cstr call

    // -- null-terminate path and call mkdir(path, mode) --
    emitter.instruction("bl __rt_cstr");                                        // convert path to C string, x0=cstr
    emitter.instruction("ldr x1, [sp, #0]");                                    // restore the requested mode into the second syscall argument
    emitter.syscall(136);

    // -- return success/failure (no EEXIST tolerance: matches PHP's leaf-mkdir semantics) --
    emitter.instruction("cmp x0, #0");                                          // check syscall result
    emitter.instruction("cset x0, eq");                                         // x0 = 1 if mkdir succeeded

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the Linux x86_64 `__rt_mkdir_mode` variant using libc `mkdir(path, mode)`.
fn emit_mkdir_mode_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mkdir_mode ---");
    emitter.label_global("__rt_mkdir_mode");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while mode is spilled across __rt_cstr
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 16");                                         // reserve an aligned spill slot for the requested mode
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the requested mode (arrived via rdi) across __rt_cstr
    emitter.instruction("call __rt_cstr");                                      // convert the elephc path in rax/rdx into a null-terminated C path in rax
    emitter.instruction("mov rdi, rax");                                        // first libc mkdir() argument = C path
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // second libc mkdir() argument = the requested mode
    emitter.instruction("call mkdir");                                          // libc mkdir(path, mode)
    emitter.instruction("cmp eax, 0");                                          // did libc mkdir() return success as a C int?
    emitter.instruction("sete al");                                             // convert the success flag into a boolean byte
    emitter.instruction("movzx rax, al");                                       // widen the boolean byte into the canonical integer result register
    emitter.instruction("add rsp, 16");                                         // release the mode spill slot
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the mkdir() success predicate to the caller
}
