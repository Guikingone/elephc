//! Purpose:
//! Emits the `__rt_file_put_contents`, `__rt_cstr` runtime helper assembly for file put contents.
//! Keeps PHP filesystem/resource behavior, libc calls, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - I/O helpers bridge PHP strings, resources, descriptors, and libc calls while returning runtime arrays or pointer/length strings.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Emits the `__rt_file_put_contents` runtime helper for PHP's `file_put_contents()`.
///
/// Dispatches to the target-specific implementation:
/// - ARM64: `emit_file_put_contents_arm64` (default)
/// - x86_64 Linux: `emit_file_put_contents_linux_x86_64`
///
/// # Input (ARM64 calling convention)
/// - x1/x2: filename string (pointer/length)
/// - x3/x4: data string (pointer/length)
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
    emit_refuse_data_scheme_aarch64(emitter);
    // php locates a wrapper for every path; a bare one is the plain-files wrapper.
    super::fopen::emit_refuse_when_file_wrapper_disabled_saying(
        emitter,
        super::fopen::DisabledWrapperAnswer::Predicate(-1),
        super::fopen::DisabledWrapperNotice::FailedToOpen {
            name_symbol: "_uww_name_file_put_contents",
            name_len: 17,
            directory: false,
        },
    );

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #64");                                     // allocate 64 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish new frame pointer

    // -- save data string for after cstr call --
    emitter.instruction("stp x3, x4, [sp, #16]");                               // save data ptr and len on stack
    emitter.instruction("str x5, [sp, #40]");                                   // hold $flags across __rt_cstr, which is a call

    // -- null-terminate the filename --
    emitter.instruction("bl __rt_path_cstr");                                   // convert filename to C string, x0=cstr path
    emitter.instruction("str x0, [sp, #0]");                                    // save null-terminated path pointer

    // -- open file with write+create+truncate --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload null-terminated path
    // FILE_APPEND (bit 3) selects O_APPEND. Without this the flag was accepted and ignored,
    // so a call meant to EXTEND the file truncated it instead — and still answered the byte
    // count, so the caller saw a success.
    emitter.instruction(&format!("mov x1, #0x{:X}", emitter.platform.o_wronly_creat_trunc())); // O_WRONLY|O_CREAT|O_TRUNC
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload $flags
    emitter.instruction("tbz x9, #3, __rt_fpc_flags_ready");                    // FILE_APPEND clear → keep truncating
    emitter.instruction(&format!("mov x1, #0x{:X}", emitter.platform.o_wronly_creat_append())); // O_WRONLY|O_CREAT|O_APPEND
    emitter.label("__rt_fpc_flags_ready");
    emitter.instruction("mov x2, #0x1A4");                                      // file mode 0644 (octal)
    emitter.syscall(5);
    // The open result was NEVER CHECKED. On macOS a failed open answers the ERRNO with the
    // carry set, so `file_put_contents("/no/such/dir/x", $payload)` wrote the payload to
    // descriptor 2 — the caller's stderr — and reported the byte count as a success. php warns
    // and answers false.
    if emitter.platform.needs_cmp_before_error_branch() {
        emitter.instruction("cmp x0, #0");                                      // Linux reports open failure as a negative result
    }
    let opened_branch = emitter.platform.branch_on_syscall_success("__rt_fpc_opened");
    emitter.instruction(&opened_branch);
    if emitter.platform.needs_cmp_before_error_branch() {
        emitter.instruction("neg x3, x0");                                      // Linux answers -errno
    } else {
        emitter.instruction("mov x3, x0");                                      // macOS answers the errno itself
    }
    emitter.instruction("ldr x2, [sp, #0]");                                    // the null-terminated path
    abi::emit_symbol_address(emitter, "x0", "_diag_open_failed_fpc_prefix");
    emitter.instruction(&format!("mov x1, #{}", "Warning: file_put_contents(".len()));
    // Prefer a name a delegating builtin published for the duration of its call. The
    // two values are loaded SEPARATELY: materializing a symbol borrows the scratch
    // register, so a length held there would not survive the pointer load.
    abi::emit_load_symbol_to_reg(emitter, "x9", "_rt_open_diag_prefix_len", 0);
    emitter.instruction("cbz x9, __rt_fpc_open_named");
    abi::emit_load_symbol_to_reg(emitter, "x0", "_rt_open_diag_prefix", 0);
    abi::emit_load_symbol_to_reg(emitter, "x1", "_rt_open_diag_prefix_len", 0);
    emitter.label("__rt_fpc_open_named");
    emitter.instruction("bl __rt_open_failed_warning");
    emitter.instruction("mov x0, #-1");                                         // php answers false for a path it cannot open
    emitter.instruction("b __rt_fpc_ret");
    emitter.label("__rt_fpc_opened");
    emitter.instruction("str x0, [sp, #8]");                                    // save fd on stack

    // -- write data to file --
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload fd
    emitter.instruction("ldr x1, [sp, #16]");                                   // reload data pointer
    emitter.instruction("ldr x2, [sp, #24]");                                   // reload data length
    emitter.syscall(4);
    emitter.instruction("str x0, [sp, #32]");                                   // save bytes written

    // -- close the file --
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload fd
    emitter.syscall(6);

    // -- return bytes written --
    emitter.instruction("ldr x0, [sp, #32]");                                   // return bytes written

    // -- restore frame and return --
    emitter.label("__rt_fpc_ret");
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the x86_64 Linux implementation of `__rt_file_put_contents`.
///
/// Uses the System V AMD64 ABI: rdi/rsi/rdx for the first three integer arguments.
/// Calls `__rt_cstr` to convert the filename, then libc `open`, `write`, and `close`.
///
/// # Input (System V AMD64 ABI)
/// - rdi/rsi: data string (pointer/length)
/// - rdx/rcx: filename string (pointer/length)
///
/// # Output
/// - rax: bytes written on success, -1 on error
fn emit_file_put_contents_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: file_put_contents ---");
    emitter.label_global("__rt_file_put_contents");
    emit_refuse_data_scheme_x86_64(emitter);
    // php locates a wrapper for every path; a bare one is the plain-files wrapper.
    super::fopen::emit_refuse_when_file_wrapper_disabled_saying(
        emitter,
        super::fopen::DisabledWrapperAnswer::Predicate(-1),
        super::fopen::DisabledWrapperNotice::FailedToOpen {
            name_symbol: "_uww_name_file_put_contents",
            name_len: 17,
            directory: false,
        },
    );

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while file_put_contents uses stack locals
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for saved pointers and lengths
    emitter.instruction("sub rsp, 48");                                         // reserve aligned stack space for data, path, fd, and byte-count temporaries

    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the data pointer while the filename is converted to a C string
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the data length while the filename is converted to a C string
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // hold $flags across __rt_cstr, which is a call
    emitter.instruction("call __rt_path_cstr");                                 // convert the elephc filename in rax/rdx into a null-terminated C path in rax
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the C filename pointer for the later open() call

    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // pass the C filename pointer as the first libc open() argument
    // See the AArch64 half: FILE_APPEND (bit 3) selects O_APPEND.
    emitter.instruction(&format!("mov rsi, 0x{:X}", emitter.platform.o_wronly_creat_trunc())); // pass O_WRONLY|O_CREAT|O_TRUNC as the open() flags
    emitter.instruction("mov r8, QWORD PTR [rbp - 48]");                        // reload $flags
    emitter.instruction("test r8, 8");                                          // FILE_APPEND?
    emitter.instruction("jz __rt_fpc_flags_ready_x");                           // clear → keep truncating
    emitter.instruction(&format!("mov rsi, 0x{:X}", emitter.platform.o_wronly_creat_append())); // O_WRONLY|O_CREAT|O_APPEND
    emitter.label("__rt_fpc_flags_ready_x");
    emitter.instruction("mov rdx, 0x1A4");                                      // pass mode 0644 for newly created files
    emitter.instruction("call open");                                           // open the destination file for overwriting through libc open()
    // See the AArch64 half: the open result was never checked, so an unopenable path wrote the
    // payload through a garbage descriptor and reported success. php warns and answers false.
    emitter.instruction("test eax, eax");                                       // libc open reports failure as a negative int
    emitter.instruction("jns __rt_fpc_opened_x");
    emitter.instruction("call __errno_location");
    emitter.instruction("movsxd rcx, DWORD PTR [rax]");                         // the errno to describe
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // the null-terminated path
    abi::emit_symbol_address(emitter, "rdi", "_diag_open_failed_fpc_prefix");
    emitter.instruction(&format!("mov esi, {}", "Warning: file_put_contents(".len()));
    // See the AArch64 arm: load both values from their slots, never park one in scratch.
    abi::emit_load_symbol_to_reg(emitter, "r11", "_rt_open_diag_prefix_len", 0);
    emitter.instruction("test r11, r11");
    emitter.instruction("jz __rt_fpc_open_named_x86");
    abi::emit_load_symbol_to_reg(emitter, "rdi", "_rt_open_diag_prefix", 0);
    abi::emit_load_symbol_to_reg(emitter, "rsi", "_rt_open_diag_prefix_len", 0);
    emitter.label("__rt_fpc_open_named_x86");
    emitter.instruction("call __rt_open_failed_warning");
    emitter.instruction("mov rax, -1");                                         // php answers false for a path it cannot open
    emitter.instruction("jmp __rt_fpc_ret_x");
    emitter.label("__rt_fpc_opened_x");
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the opened file descriptor for the later write() and close() calls

    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // pass the file descriptor as the first libc write() argument
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // pass the source data pointer as the second libc write() argument
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // pass the source data length as the third libc write() argument
    emitter.instruction("call write");                                          // write the requested bytes into the opened destination file
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the number of written bytes for the final return value

    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // pass the file descriptor as the first libc close() argument
    emitter.instruction("call close");                                          // close the destination file after the write completes

    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // return the number of bytes reported by libc write()
    emitter.label("__rt_fpc_ret_x");
    emitter.instruction("add rsp, 48");                                         // release the aligned stack locals used by file_put_contents
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the caller with the write byte count in rax
}

/// Refuses a `data:` path the way php does, before the plain-file open is ever attempted.
///
/// php locates the wrapper first and the data wrapper OPENS in any mode — it simply has no write
/// function, so the write is what fails. MEASURED on `php -n` 8.5.6, both spellings:
///
///     file_put_contents('data://text/plain,cccc', 'x')  Notice: … Stream is not writable / false
///     file_put_contents('data:text/plain,cccc', 'x')    the same
///
/// elephc never consulted a wrapper here and opened the whole URL as a FILENAME, so it answered
/// `Warning: file_put_contents(data://text/plain,cccc): Failed to open stream: No such file or
/// directory` — a wrong reason for a stream that opens perfectly well.
///
/// The scheme is matched case-SENSITIVELY, because php is: `DATA://` and `Data://` are not the
/// data wrapper for either implementation (measured on both).
///
/// ⚠️ Only `data:` is routed. Every other wrapper still reaches the plain-file open, and each has
/// its own php answer; this is the one the corpus names.
fn emit_refuse_data_scheme_aarch64(emitter: &mut Emitter) {
    emitter.instruction("cmp x2, #5");                                          // "data:" is five bytes
    emitter.instruction("b.lt __rt_fpc_not_data");                              // too short to be the data wrapper
    abi::emit_symbol_address(emitter, "x9", "_data_n_prefix");                  // "data://" — the first five bytes are the scheme
    emitter.instruction("mov x10, #0");                                         // compare cursor
    emitter.label("__rt_fpc_data_scan");
    emitter.instruction("cmp x10, #5");                                         // compared the whole scheme?
    emitter.instruction("b.ge __rt_fpc_is_data");                               // it is the data wrapper
    emitter.instruction("ldrb w11, [x1, x10]");                                 // the path byte
    emitter.instruction("ldrb w12, [x9, x10]");                                 // the scheme byte
    emitter.instruction("cmp w11, w12");
    emitter.instruction("b.ne __rt_fpc_not_data");                              // some other wrapper, or a plain path
    emitter.instruction("add x10, x10, #1");
    emitter.instruction("b __rt_fpc_data_scan");
    emitter.label("__rt_fpc_is_data");
    emitter.instruction("sub sp, sp, #16");                                     // frame for the diagnostic call
    emitter.instruction("stp x29, x30, [sp, #0]");                              // save frame pointer and return address
    crate::codegen_support::runtime::io::emit_announce_read_fn_name(
        emitter,
        "_uww_name_file_put_contents",
        17,
    );                                                                          // php names the builtin the user wrote
    emitter.instruction("bl __rt_not_writable_notice");
    crate::codegen_support::runtime::io::emit_clear_read_fn_name(emitter);      // a later fwrite() is its own name again
    emitter.instruction("mov x0, #-1");                                         // negative: the caller boxes PHP false
    emitter.instruction("ldp x29, x30, [sp, #0]");                              // restore frame pointer and return address
    emitter.instruction("add sp, sp, #16");                                     // release the frame
    emitter.instruction("ret");
    emitter.label("__rt_fpc_not_data");
}

/// The x86_64 counterpart of [`emit_refuse_data_scheme_aarch64`]; the filename is in `rax`/`rdx`.
fn emit_refuse_data_scheme_x86_64(emitter: &mut Emitter) {
    emitter.instruction("cmp rdx, 5");                                          // "data:" is five bytes
    emitter.instruction("jl __rt_fpc_not_data_x86");                            // too short to be the data wrapper
    abi::emit_symbol_address(emitter, "r9", "_data_n_prefix");                  // "data://" — the first five bytes are the scheme
    emitter.instruction("xor r10, r10");                                        // compare cursor
    emitter.label("__rt_fpc_data_scan_x86");
    emitter.instruction("cmp r10, 5");                                          // compared the whole scheme?
    emitter.instruction("jge __rt_fpc_is_data_x86");                            // it is the data wrapper
    emitter.instruction("movzx r11d, BYTE PTR [rax + r10]");                    // the path byte
    emitter.instruction("movzx r8d, BYTE PTR [r9 + r10]");                      // the scheme byte
    emitter.instruction("cmp r11b, r8b");
    emitter.instruction("jne __rt_fpc_not_data_x86");                           // some other wrapper, or a plain path
    emitter.instruction("add r10, 1");
    emitter.instruction("jmp __rt_fpc_data_scan_x86");
    emitter.label("__rt_fpc_is_data_x86");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame pointer
    crate::codegen_support::runtime::io::emit_announce_read_fn_name(
        emitter,
        "_uww_name_file_put_contents",
        17,
    );                                                                          // php names the builtin the user wrote
    emitter.instruction("call __rt_not_writable_notice");
    crate::codegen_support::runtime::io::emit_clear_read_fn_name(emitter);      // a later fwrite() is its own name again
    emitter.instruction("mov rax, -1");                                         // negative: the caller boxes PHP false
    emitter.instruction("mov rsp, rbp");                                        // release the frame from rbp
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");
    emitter.label("__rt_fpc_not_data_x86");
}
