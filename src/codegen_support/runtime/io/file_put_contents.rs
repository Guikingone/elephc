//! Purpose:
//! Emits the `__rt_file_put_contents` / `__rt_file_put_contents_flags` runtime helper assembly.
//! Keeps PHP filesystem/resource behavior, libc calls, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - `__rt_file_put_contents_flags` honours PHP's `FILE_APPEND` (8) and `LOCK_EX` (2): the file
//!   is opened WITHOUT truncation, locked when asked, and only then truncated unless appending,
//!   so a concurrent reader never sees it emptied before the lock is held (php's own order).
//! - `__rt_file_put_contents` is the flag-less entry the copy and PHAR writers use; it zeroes
//!   the flags register and falls through.
//! - A failed `open()` returns -1 without touching any descriptor. On macOS the raw syscall
//!   reports failure through the carry flag with a POSITIVE errno in x0, which the previous
//!   version wrote to as if it were a descriptor.

use crate::codegen_support::{emit::Emitter, platform::{Arch, Platform}};

/// PHP's `FILE_APPEND` flag bit.
const FILE_APPEND: u32 = 8;
/// PHP's `LOCK_EX` flag bit (also the POSIX `flock()` exclusive-lock operation).
const LOCK_EX: u32 = 2;

/// Emits the `__rt_file_put_contents` runtime helpers for PHP's `file_put_contents()`.
///
/// # Input (ARM64)
/// - x1/x2: filename string (pointer/length)
/// - x3/x4: data string (pointer/length)
/// - x5: PHP flags (`__rt_file_put_contents_flags` only)
///
/// # Output
/// - x0: bytes written on success, -1 on error
pub fn emit_file_put_contents(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_file_put_contents_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: file_put_contents ---");
    emitter.label_global("__rt_file_put_contents");
    emitter.instruction("mov x5, #0");                                          // the flag-less entry writes with flags = 0
    emitter.label_global("__rt_file_put_contents_flags");

    // -- frame: [0]=C path [8]=fd [16..32)=data ptr/len [32]=bytes written [40]=flags --
    emitter.instruction("sub sp, sp, #64");                                     // allocate 64 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish new frame pointer
    emitter.instruction("stp x3, x4, [sp, #16]");                               // save data ptr and len on stack
    emitter.instruction("str x5, [sp, #40]");                                   // save the PHP flags across the calls below

    // -- null-terminate the filename --
    emitter.instruction("bl __rt_cstr");                                        // convert filename to C string, x0=cstr path
    emitter.instruction("str x0, [sp, #0]");                                    // save null-terminated path pointer

    // -- open for writing without truncation, appending when FILE_APPEND is set --
    emitter.instruction(&format!("mov x1, #0x{:X}", emitter.platform.o_wronly_creat())); // O_WRONLY|O_CREAT
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload the PHP flags
    emitter.instruction(&format!("tst x9, #{}", FILE_APPEND));                  // was FILE_APPEND requested?
    emitter.instruction("b.eq __rt_fpc_open");                                  // no: keep the plain write mode
    emitter.instruction(&format!("mov x1, #0x{:X}", emitter.platform.o_wronly_creat_append())); // O_WRONLY|O_CREAT|O_APPEND
    emitter.label("__rt_fpc_open");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload null-terminated path
    emitter.instruction("mov x2, #0x1A4");                                      // file mode 0644 (octal)
    emitter.syscall(5);
    match emitter.platform {
        Platform::MacOS => {
            emitter.instruction("b.cs __rt_fpc_fail");                          // carry set: open failed and x0 holds errno, not a descriptor
        }
        _ => {
            emitter.instruction("tbnz x0, #63, __rt_fpc_fail");                 // negative result: open failed
        }
    }
    emitter.instruction("str x0, [sp, #8]");                                    // save fd on stack

    // -- LOCK_EX: take an exclusive lock before touching the contents --
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload the PHP flags
    emitter.instruction(&format!("tst x9, #{}", LOCK_EX));                      // was LOCK_EX requested?
    emitter.instruction("b.eq __rt_fpc_locked");                                // no lock requested
    emitter.instruction("ldr x0, [sp, #8]");                                    // fd to lock
    emitter.instruction(&format!("mov x1, #{}", LOCK_EX));                      // POSIX LOCK_EX
    emitter.bl_c("flock");                                                      // libc flock(fd, LOCK_EX)
    emitter.instruction("cbnz w0, __rt_fpc_close_fail");                        // a failed lock fails the whole write, as in php
    emitter.label("__rt_fpc_locked");

    // -- truncate now unless appending --
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload the PHP flags
    emitter.instruction(&format!("tst x9, #{}", FILE_APPEND));                  // appending keeps the existing contents
    emitter.instruction("b.ne __rt_fpc_write");                                 // FILE_APPEND: skip truncation
    emitter.instruction("ldr x0, [sp, #8]");                                    // fd to truncate
    emitter.instruction("mov x1, #0");                                          // new length 0
    emitter.bl_c("ftruncate");                                                  // libc ftruncate(fd, 0)

    // -- write data to file --
    emitter.label("__rt_fpc_write");
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload fd
    emitter.instruction("ldr x1, [sp, #16]");                                   // reload data pointer
    emitter.instruction("ldr x2, [sp, #24]");                                   // reload data length
    emitter.syscall(4);
    emitter.instruction("str x0, [sp, #32]");                                   // save bytes written

    // -- close the file (releases the lock) --
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload fd
    emitter.syscall(6);

    // -- return bytes written --
    emitter.instruction("ldr x0, [sp, #32]");                                   // return bytes written
    emitter.instruction("b __rt_fpc_return");                                   // skip the failure paths

    emitter.label("__rt_fpc_close_fail");
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload fd to release it
    emitter.syscall(6);
    emitter.label("__rt_fpc_fail");
    emitter.instruction("mov x0, #-1");                                         // PHP false sentinel
    emitter.label("__rt_fpc_return");
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the x86_64 Linux implementation of the `__rt_file_put_contents` helpers.
///
/// # Input
/// - rax/rdx: filename string (pointer/length)
/// - rdi/rsi: data string (pointer/length)
/// - r8: PHP flags (`__rt_file_put_contents_flags` only)
///
/// # Output
/// - rax: bytes written on success, -1 on error
fn emit_file_put_contents_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: file_put_contents ---");
    emitter.label_global("__rt_file_put_contents");
    emitter.instruction("xor r8d, r8d");                                        // the flag-less entry writes with flags = 0
    emitter.label_global("__rt_file_put_contents_flags");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while file_put_contents uses stack locals
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for saved pointers and lengths
    emitter.instruction("sub rsp, 48");                                         // reserve aligned stack space for data, path, fd, flags, and byte count

    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the data pointer while the filename is converted to a C string
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the data length while the filename is converted to a C string
    emitter.instruction("mov QWORD PTR [rbp - 48], r8");                        // save the PHP flags across the libc calls
    emitter.instruction("call __rt_cstr");                                      // convert the elephc filename in rax/rdx into a null-terminated C path in rax
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the C filename pointer for the later open() call

    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // pass the C filename pointer as the first libc open() argument
    emitter.instruction(&format!("mov esi, 0x{:X}", emitter.platform.o_wronly_creat())); // O_WRONLY|O_CREAT, no truncation yet
    emitter.instruction(&format!("mov ecx, 0x{:X}", emitter.platform.o_wronly_creat_append())); // O_WRONLY|O_CREAT|O_APPEND
    emitter.instruction(&format!("test QWORD PTR [rbp - 48], {}", FILE_APPEND)); // was FILE_APPEND requested?
    emitter.instruction("jz __rt_fpc_open_x86");                                // no: keep the plain write mode
    emitter.instruction("mov esi, ecx");                                        // open in append mode
    emitter.label("__rt_fpc_open_x86");
    emitter.instruction("mov edx, 0x1A4");                                      // pass mode 0644 for newly created files
    emitter.instruction("xor eax, eax");                                        // variadic call: no vector registers used
    emitter.instruction("call open");                                           // open the destination file through libc open()
    emitter.instruction("test eax, eax");                                       // did open() fail?
    emitter.instruction("js __rt_fpc_fail_x86");                                // -1: return PHP false without touching a descriptor
    emitter.instruction("movsxd rax, eax");                                     // widen the descriptor
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the opened file descriptor

    emitter.instruction(&format!("test QWORD PTR [rbp - 48], {}", LOCK_EX));    // was LOCK_EX requested?
    emitter.instruction("jz __rt_fpc_locked_x86");                              // no lock requested
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // fd to lock
    emitter.instruction(&format!("mov esi, {}", LOCK_EX));                      // POSIX LOCK_EX
    emitter.instruction("call flock");                                          // libc flock(fd, LOCK_EX)
    emitter.instruction("test eax, eax");                                       // did the lock succeed?
    emitter.instruction("jnz __rt_fpc_close_fail_x86");                         // a failed lock fails the whole write, as in php
    emitter.label("__rt_fpc_locked_x86");

    emitter.instruction(&format!("test QWORD PTR [rbp - 48], {}", FILE_APPEND)); // appending keeps the existing contents
    emitter.instruction("jnz __rt_fpc_write_x86");                              // FILE_APPEND: skip truncation
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // fd to truncate
    emitter.instruction("xor esi, esi");                                        // new length 0
    emitter.instruction("call ftruncate");                                      // libc ftruncate(fd, 0)

    emitter.label("__rt_fpc_write_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // pass the file descriptor as the first libc write() argument
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // pass the source data pointer as the second libc write() argument
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // pass the source data length as the third libc write() argument
    emitter.instruction("call write");                                          // write the requested bytes into the opened destination file
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the number of written bytes for the final return value

    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // pass the file descriptor as the first libc close() argument
    emitter.instruction("call close");                                          // close the destination file (releases the lock)

    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // return the number of bytes reported by libc write()
    emitter.instruction("jmp __rt_fpc_return_x86");                             // skip the failure paths

    emitter.label("__rt_fpc_close_fail_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // release the descriptor whose lock failed
    emitter.instruction("call close");                                          // close it before reporting failure
    emitter.label("__rt_fpc_fail_x86");
    emitter.instruction("mov rax, -1");                                         // PHP false sentinel
    emitter.label("__rt_fpc_return_x86");
    emitter.instruction("add rsp, 48");                                         // release the aligned stack locals used by file_put_contents
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the caller with the write byte count in rax
}
