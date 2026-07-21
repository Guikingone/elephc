//! Purpose:
//! Emits the `__rt_mkdir_recursive` runtime helper backing `mkdir($dir,
//! $permissions, recursive: true)`: walks `/`-separated path prefixes,
//! best-effort-creating each parent directory, then creates the full
//! (trailing-slash-trimmed) path as the leaf.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - `crate::codegen::lower_inst::builtins::io::lower_mkdir()` (the
//!   `mkdir()` EIR lowering) when `$recursive` is truthy.
//!
//! Key details:
//! - php-verified (PHP 8.5.6 local, `php -n`) semantics this mirrors exactly:
//!   parent-prefix `mkdir()` calls tolerate ANY failure (in particular
//!   `EEXIST` — an already-existing parent directory is fine), but the FINAL
//!   `mkdir()` on the full (trimmed) path does NOT tolerate failure — its own
//!   success/failure is the function's return value. This is why
//!   `mkdir("existing", 0777, true)` on an existing leaf directory still
//!   returns `false` (php-verified) even though every parent already exists,
//!   while `mkdir("a/b/c", 0777, true)` on a brand-new nested path returns
//!   `true`. A trailing slash (`"a/b/"`) is trimmed before the walk so the
//!   loop never attempts to `mkdir()` an empty trailing segment.
//! - Delegates each individual `mkdir()` attempt to `__rt_mkdir_mode`, so the
//!   real caller-supplied mode applies at every level (matching PHP: the same
//!   `$permissions` argument is used for every directory the recursive walk
//!   creates, not just the leaf).
//! - Does NOT dispatch through the stream-wrapper path-op probe used by the
//!   1-arg `mkdir()` EIR lowering (a scoped, documented residual — see
//!   `crate::codegen::lower_inst::builtins::io::lower_mkdir` — recursive
//!   mkdir against a registered userspace stream wrapper is not implemented).

use crate::codegen::{emit::Emitter, platform::Arch};

/// Emits `__rt_mkdir_recursive` for the host target.
///
/// AArch64: input x1=ptr, x2=len, x3=mode. Output: x0=1 on success, 0 on failure
/// (the final leaf `mkdir()`'s own result; parent-prefix failures are ignored).
/// x86_64: input rax=ptr, rdx=len, rdi=mode. Output: rax=1 on success, 0 on failure.
pub fn emit_mkdir_recursive(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mkdir_recursive_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: mkdir_recursive ---");
    emitter.label_global("__rt_mkdir_recursive");

    // Frame (48 bytes): [0]=ptr [8]=effective_len [16]=mode [24]=cursor [32]x29 [40]x30.
    emitter.instruction("sub sp, sp, #48");                                     // allocate the recursive-walk frame
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the walk frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save the path pointer
    emitter.instruction("str x2, [sp, #8]");                                    // save the path length (trimmed below)
    emitter.instruction("str x3, [sp, #16]");                                   // save the requested mode

    // -- trim any trailing '/' bytes so the walk never targets an empty final segment --
    emitter.label("__rt_mkdir_rec_trim");
    emitter.instruction("ldr x9, [sp, #8]");                                    // reload the effective length
    emitter.instruction("cbz x9, __rt_mkdir_rec_trim_done");                    // defensive: an all-slash/empty string has nothing left to trim
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload the path pointer
    emitter.instruction("sub x11, x9, #1");                                     // index of the last byte
    emitter.instruction("ldrb w12, [x10, x11]");                                // load that last byte
    emitter.instruction("cmp w12, #0x2F");                                      // is it '/'?
    emitter.instruction("b.ne __rt_mkdir_rec_trim_done");                       // no trailing slash left: done trimming
    emitter.instruction("str x11, [sp, #8]");                                   // trim one trailing slash and re-check
    emitter.instruction("b __rt_mkdir_rec_trim");
    emitter.label("__rt_mkdir_rec_trim_done");

    // -- walk interior '/' separators, best-effort mkdir() each parent prefix --
    emitter.instruction("mov x4, #1");                                          // cursor i = 1 (skip index 0: never mkdir("") for a leading '/')
    emitter.instruction("str x4, [sp, #24]");                                   // save the cursor

    emitter.label("__rt_mkdir_rec_walk_loop");
    emitter.instruction("ldr x4, [sp, #24]");                                   // reload the cursor
    emitter.instruction("ldr x9, [sp, #8]");                                    // reload the effective length
    emitter.instruction("cmp x4, x9");                                          // reached the (trimmed) end of the path?
    emitter.instruction("b.ge __rt_mkdir_rec_walk_done");                       // no more interior bytes to scan
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload the path pointer
    emitter.instruction("ldrb w11, [x10, x4]");                                 // load the byte at the cursor
    emitter.instruction("cmp w11, #0x2F");                                      // is it a '/' separator?
    emitter.instruction("b.ne __rt_mkdir_rec_walk_next");                       // not a separator: keep scanning

    // -- found an interior separator: best-effort mkdir() the prefix path[0..cursor) --
    emitter.instruction("mov x1, x10");                                         // prefix pointer = the path base
    emitter.instruction("mov x2, x4");                                          // prefix length = the cursor position
    emitter.instruction("ldr x3, [sp, #16]");                                   // reload the requested mode
    emitter.instruction("bl __rt_mkdir_mode");                                  // best-effort: EEXIST/any failure here is fine

    emitter.label("__rt_mkdir_rec_walk_next");
    emitter.instruction("ldr x4, [sp, #24]");                                   // reload the cursor
    emitter.instruction("add x4, x4, #1");                                      // advance the cursor
    emitter.instruction("str x4, [sp, #24]");                                   // save the advanced cursor
    emitter.instruction("b __rt_mkdir_rec_walk_loop");                          // continue the walk

    emitter.label("__rt_mkdir_rec_walk_done");
    // -- final: mkdir() the full (trimmed) path; its own result is the return value --
    emitter.instruction("ldr x1, [sp, #0]");                                    // reload the path pointer
    emitter.instruction("ldr x2, [sp, #8]");                                    // reload the effective (trimmed) length
    emitter.instruction("ldr x3, [sp, #16]");                                   // reload the requested mode
    emitter.instruction("bl __rt_mkdir_mode");                                  // x0 = this call's own success/failure (no EEXIST tolerance)

    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // deallocate the recursive-walk frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the Linux x86_64 `__rt_mkdir_recursive` variant.
fn emit_mkdir_recursive_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mkdir_recursive ---");
    emitter.label_global("__rt_mkdir_recursive");

    // rbp-relative frame: [-8]=ptr [-16]=effective_len [-24]=mode [-32]=cursor.
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the walk frame pointer
    emitter.instruction("sub rsp, 32");                                         // reserve the recursive-walk local slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the path pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the path length (trimmed below)
    emitter.instruction("mov QWORD PTR [rbp - 24], rdi");                       // save the requested mode

    // -- trim any trailing '/' bytes so the walk never targets an empty final segment --
    emitter.label("__rt_mkdir_rec_trim_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the effective length
    emitter.instruction("test rax, rax");                                       // defensive: an all-slash/empty string has nothing left to trim
    emitter.instruction("jz __rt_mkdir_rec_trim_done_x86_64");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // reload the path pointer
    emitter.instruction("lea rcx, [rax - 1]");                                  // index of the last byte
    emitter.instruction("movzx edx, BYTE PTR [rsi + rcx]");                     // load that last byte
    emitter.instruction("cmp dl, 0x2F");                                        // is it '/'?
    emitter.instruction("jne __rt_mkdir_rec_trim_done_x86_64");                 // no trailing slash left: done trimming
    emitter.instruction("mov QWORD PTR [rbp - 16], rcx");                       // trim one trailing slash and re-check
    emitter.instruction("jmp __rt_mkdir_rec_trim_x86_64");
    emitter.label("__rt_mkdir_rec_trim_done_x86_64");

    // -- walk interior '/' separators, best-effort mkdir() each parent prefix --
    emitter.instruction("mov QWORD PTR [rbp - 32], 1");                         // cursor i = 1 (skip index 0: never mkdir("") for a leading '/')

    emitter.label("__rt_mkdir_rec_walk_loop_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // reload the cursor
    emitter.instruction("cmp rax, QWORD PTR [rbp - 16]");                       // reached the (trimmed) end of the path?
    emitter.instruction("jge __rt_mkdir_rec_walk_done_x86_64");                 // no more interior bytes to scan
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // reload the path pointer
    emitter.instruction("movzx edx, BYTE PTR [rsi + rax]");                     // load the byte at the cursor
    emitter.instruction("cmp dl, 0x2F");                                        // is it a '/' separator?
    emitter.instruction("jne __rt_mkdir_rec_walk_next_x86_64");                 // not a separator: keep scanning

    // -- found an interior separator: best-effort mkdir() the prefix path[0..cursor) --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // prefix pointer = the path base
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // prefix length = the cursor position
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // reload the requested mode
    emitter.instruction("call __rt_mkdir_mode");                                // best-effort: EEXIST/any failure here is fine

    emitter.label("__rt_mkdir_rec_walk_next_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // reload the cursor
    emitter.instruction("inc rax");                                             // advance the cursor
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the advanced cursor
    emitter.instruction("jmp __rt_mkdir_rec_walk_loop_x86_64");                 // continue the walk

    emitter.label("__rt_mkdir_rec_walk_done_x86_64");
    // -- final: mkdir() the full (trimmed) path; its own result is the return value --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the path pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload the effective (trimmed) length
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // reload the requested mode
    emitter.instruction("call __rt_mkdir_mode");                                // rax = this call's own success/failure (no EEXIST tolerance)

    emitter.instruction("add rsp, 32");                                         // release the recursive-walk local slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to caller
}
