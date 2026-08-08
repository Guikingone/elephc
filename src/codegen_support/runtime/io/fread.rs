//! Purpose:
//! Emits the `__rt_fread`, `__rt_fread_done` runtime helper assembly for fread.
//! Keeps PHP filesystem/resource behavior, libc calls, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - Native reads resolve their descriptor from an opaque stream handle and
//!   publish EOF on the corresponding `StreamState`, including seekable short reads.

use crate::codegen_support::{emit::Emitter, platform::Arch};
use crate::codegen_support::abi;

/// Capacity of the shared `_concat_buf` accumulation buffer.
const CONCAT_BUF_CAPACITY: u64 = 65536;

/// Emits the `__rt_fread` runtime helper for reading bytes from a stream handle.
///
/// On ARM64: reads into the concat buffer, updates `_concat_off`, sets StreamState EOF,
/// and returns (pointer, byte_count) in x1:x2.
///
/// On x86_64: same semantics but uses libc `read()` and returns (pointer, byte_count) in rax:rdx.
///
/// # Inputs
/// - x0/rdi: opaque stream handle
/// - x1/rsi: number of bytes to read
///
/// # Outputs
/// - x1/x86_64 rax: pointer to bytes in concat buffer (borrowed, not owned)
/// - x2/rdx: actual bytes read (0 on EOF/error)
///
/// # Side effects
/// - Advances `_concat_off` by actual bytes read.
/// - Sets state-owned EOF on zero-byte reads and seekable short reads.
pub fn emit_fread(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_fread_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: fread ---");
    emitter.label_global("__rt_fread");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #64");                                     // allocate stream, descriptor, and read-result spill slots
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish new frame pointer

    // -- save handle and requested length, then resolve the backend descriptor --
    emitter.instruction("str x0, [sp, #0]");                                    // save the opaque stream handle
    emitter.instruction("str x1, [sp, #8]");                                    // save requested read length
    emitter.instruction("bl __rt_stream_fd");                                   // resolve the backend descriptor through StreamState
    emitter.instruction("str x0, [sp, #32]");                                   // preserve the resolved backend descriptor
    emitter.instruction("mov w9, #0x4000");                                     // load the high half of USER_WRAPPER_FD_BASE = 0x40000000
    emitter.instruction("lsl w9, w9, #16");                                     // shift into bits 30..16 to form 0x40000000
    emitter.instruction("cmp x0, x9");                                          // is the backend below the wrapper range?
    emitter.instruction("b.lo __rt_fread_real_fd");                             // native descriptors continue to the syscall path
    // The wrapper fd range ends at the allocated handle capacity, not a
    // fixed 256: a slot beyond the bound would be misread as a native fd.
    super::emit_load_handles_cap(emitter, "x10");
    emitter.instruction("add x10, x9, x10");                                    // wrapper range end = USER_WRAPPER_FD_BASE + handle capacity
    emitter.instruction("cmp x0, x10");                                         // is the backend above the wrapper range?
    emitter.instruction("b.hs __rt_fread_real_fd");                             // non-wrapper synthetic backends stay on the native path
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload the requested byte count for stream_read
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore the caller frame before wrapper tail dispatch
    emitter.instruction("add sp, sp, #64");                                     // release native-read scratch storage
    emitter.instruction("b __rt_user_wrapper_fread");                           // wrapper backend tail-calls stream_read
    emitter.label("__rt_fread_real_fd");

    // -- get concat_buf write position --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("ldr x10, [x9]");                                       // load current write offset
    // Clamp the request to what the shared buffer can still hold. Without this a
    // read past the 64 KiB capacity ran straight off the end; `_concat_off` is
    // declared immediately after `_concat_buf`, so the first bytes overwritten
    // were the accumulator itself, which both corrupted memory and truncated the
    // result at an arbitrary length.
    crate::codegen_support::abi::emit_symbol_address(emitter, "x11", "_concat_buf");
    emitter.instruction("add x12, x11, x10");                                   // compute write pointer: buf + offset
    // The start pointer must be published before the capacity check: the
    // buffer-full exit joins the common epilogue, which reads it back.
    emitter.instruction("str x12, [sp, #16]");                                  // save start pointer for return value
    emitter.instruction(&format!("mov x13, #{CONCAT_BUF_CAPACITY}"));           // shared concat-buffer capacity
    emitter.instruction("subs x13, x13, x10");                                  // bytes still available after the write cursor
    emitter.instruction("b.le __rt_fread_buffer_full");                         // no capacity left: report a zero-length read
    emitter.instruction("ldr x14, [sp, #8]");                                   // requested byte count
    emitter.instruction("cmp x14, x13");                                        // does the request exceed the remaining capacity?
    emitter.instruction("csel x14, x13, x14, gt");                              // clamp to the remaining capacity
    emitter.instruction("str x14, [sp, #8]");                                   // publish the clamped request

    // -- TLS dispatch: the session hangs off the StreamState, so it is keyed by
    //    the generation-checked handle rather than by a reusable descriptor. --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the opaque stream handle
    emitter.instruction("bl __rt_stream_tls_session");                          // x0 = attached session, zero when plain
    emitter.instruction("cbz x0, __rt_fread_do_syscall");                       // no TLS attached → fall through to read syscall
    emitter.instruction("ldr x1, [sp, #16]");                                   // buf ptr: x12 is caller-saved and the call clobbered it
    emitter.instruction("ldr x2, [sp, #8]");                                    // len
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_elephc_tls_read_fn");
    emitter.instruction("ldr x9, [x9]");                                        // load elephc_tls_read entry pointer
    emitter.instruction("blr x9");                                              // x0 = bytes read (>=0) or -1
    emitter.instruction("cmp x0, #0");                                          // value-based check after the TLS call
    emitter.instruction("b.ge __rt_fread_read_ok");                             // continue when TLS read returned >= 0
    emitter.instruction("str xzr, [sp, #24]");                                  // TLS error: zero-length result
    emitter.instruction("b __rt_fread_mark_eof");                               // mark the stream exhausted

    emitter.label("__rt_fread_do_syscall");
    // -- perform read syscall --
    emitter.instruction("ldr x0, [sp, #32]");                                   // fd for read syscall
    emitter.instruction("ldr x1, [sp, #16]");                                   // buffer pointer: the TLS probe clobbers caller-saved x12
    emitter.instruction("ldr x2, [sp, #8]");                                    // number of bytes to read
    emitter.syscall(3);
    if emitter.platform.needs_cmp_before_error_branch() {
        emitter.instruction("cmp x0, #0");                                      // Linux: negative read result means failure
    }
    emitter.instruction(&emitter.platform.branch_on_syscall_success("__rt_fread_read_ok")); // continue only when the read syscall succeeded
    if emitter.platform.needs_cmp_before_error_branch() {
        emitter.instruction(&format!("cmn x0, #{}", emitter.platform.would_block_errno())); // Linux: is this -EAGAIN/-EWOULDBLOCK from a nonblocking fd?
    } else {
        emitter.instruction(&format!("cmp x0, #{}", emitter.platform.would_block_errno())); // macOS: is this EAGAIN/EWOULDBLOCK from a nonblocking fd?
    }
    emitter.instruction("b.eq __rt_fread_would_block");                         // a transient nonblocking miss is not EOF
    emitter.instruction("str xzr, [sp, #24]");                                  // failed reads return an empty result
    emitter.instruction("b __rt_fread_mark_eof");                               // mark the stream as exhausted after a read failure
    emitter.label("__rt_fread_read_ok");

    // -- update concat_off by actual bytes read --
    emitter.instruction("str x0, [sp, #24]");                                   // save actual bytes read
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_concat_off");
    emitter.instruction("ldr x10, [x9]");                                       // load current offset
    emitter.instruction("add x10, x10, x0");                                    // advance offset by bytes read
    emitter.instruction("str x10, [x9]");                                       // store updated offset

    // -- set EOF when the read returned fewer bytes than requested --
    emitter.instruction("ldr x0, [sp, #24]");                                   // reload bytes read
    emitter.instruction("ldr x9, [sp, #8]");                                    // reload the requested byte count
    emitter.instruction("cmp x0, x9");                                          // did the backend satisfy the complete request?
    emitter.instruction("b.ge __rt_fread_done");                                // a full read does not prove EOF
    emitter.instruction("cbz x0, __rt_fread_mark_eof");                         // a zero-byte read is EOF for every blocking backend
    emitter.instruction("ldr x0, [sp, #32]");                                   // reload the backend descriptor for a seekability probe
    emitter.instruction("mov x1, #0");                                          // probe the current position without moving it
    emitter.instruction("mov x2, #1");                                          // select SEEK_CUR for the non-mutating probe
    emitter.syscall(199);
    if emitter.platform.needs_cmp_before_error_branch() {
        emitter.instruction("cmp x0, #0");                                      // Linux reports a non-seekable descriptor as a negative result
    }
    emitter.instruction(&emitter.platform.branch_on_syscall_success("__rt_fread_mark_eof")); // only seekable short reads prove EOF
    emitter.instruction("b __rt_fread_done");                                   // sockets and pipes may legally return partial data
    emitter.label("__rt_fread_mark_eof");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the opaque stream handle
    emitter.instruction("mov x1, #1");                                          // publish the EOF state
    emitter.instruction("bl __rt_stream_eof_set");                              // update only this stream's stable state
    emitter.instruction("b __rt_fread_done");                                   // preserve a successful short-read result

    emitter.label("__rt_fread_buffer_full");
    emitter.instruction("str xzr, [sp, #24]");                                  // a full shared buffer yields an empty read rather than an overflow
    emitter.instruction("b __rt_fread_done");                                   // join the common return path

    emitter.label("__rt_fread_would_block");
    emitter.instruction("str xzr, [sp, #24]");                                  // return an empty read without setting EOF for EAGAIN/EWOULDBLOCK

    // -- return pointer and length --
    emitter.label("__rt_fread_done");
    emitter.instruction("ldr x1, [sp, #16]");                                   // return string start pointer
    emitter.instruction("ldr x2, [sp, #24]");                                   // return actual bytes read as length

    // -- apply the attached read filter chain to the bytes just read --
    // The chain is an arbitrarily long linked list rooted in StreamState, so this
    // replaces the old two-slot table that silently dropped a third filter.
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the opaque stream handle
    emitter.instruction(&format!(
        "mov x3, #{}",
        crate::codegen_support::runtime::resources::layout::STREAM_READ_FILTER_HEAD_OFFSET
    ));                                                                         // select the read-direction chain
    emitter.instruction("bl __rt_stream_apply_filter_chain");                   // x1/x2 <- filtered buffer and length

    // Legacy per-descriptor slots still serve the filter families not yet moved
    // onto the chain (user filters, zlib/bzip2/iconv). A filter lives in exactly
    // one mechanism, so running both is correct for the duration of the migration.
    // -- apply attached read filters to the bytes just read (2-slot chain) --
    //    Slot 0 = _stream_read_filters[fd], slot 1 = _stream_read_filters[fd+256]
    emitter.instruction("ldr x0, [sp, #32]");                                   // reload the file descriptor
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_stream_read_filters");
    // -- slot 0 --
    emitter.instruction("ldrb w3, [x9, x0]");                                   // read filter id for slot 0
    emitter.instruction("cbz w3, __rt_fread_slot1");                            // skip slot 0 when empty
    emitter.instruction("cmp w3, #128");                                        // user-filter id range?
    emitter.instruction("b.lt __rt_fread_builtin_slot0");                       // built-in filter
    emitter.instruction("mov x3, #0");                                          // direction = 0 (read)
    emitter.instruction("bl __rt_apply_user_stream_filter");                    // x1/x2 ← transformed
    emitter.instruction("b __rt_fread_slot1");                                  // proceed to slot 1
    emitter.label("__rt_fread_builtin_slot0");
    emitter.instruction("bl __rt_apply_stream_filter");                         // transform in place
    // -- slot 1 --
    emitter.label("__rt_fread_slot1");
    crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_stream_read_filters");
    emitter.instruction("ldr x0, [sp, #32]");                                   // reload fd
    emitter.instruction("add x10, x0, #256");                                   // fd+256 (slot 1 index)
    emitter.instruction("ldrb w3, [x9, x10]");                                  // read filter id for slot 1
    emitter.instruction("cbz w3, __rt_fread_ret");                              // skip slot 1 when empty
    emitter.instruction("cmp w3, #128");                                        // user-filter id range?
    emitter.instruction("b.lt __rt_fread_builtin_slot1");                       // built-in filter
    emitter.instruction("mov x3, #0");                                          // direction = 0 (read)
    emitter.instruction("bl __rt_apply_user_stream_filter");                    // x1/x2 ← transformed
    emitter.instruction("b __rt_fread_ret");                                    // common epilogue
    emitter.label("__rt_fread_builtin_slot1");
    emitter.instruction("bl __rt_apply_stream_filter");                         // transform in place


    // -- restore frame and return --
    emitter.label("__rt_fread_ret");
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the x86_64 Linux variant of `__rt_fread` using libc `read()`.
fn emit_fread_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: fread ---");
    emitter.label_global("__rt_fread");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while fread() uses local spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the saved file descriptor, length, and concat-buffer start pointer
    emitter.instruction("sub rsp, 48");                                         // reserve aligned stream, descriptor, and read-result spill slots

    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the opaque stream handle
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // preserve the requested byte count across the concat-buffer address computation and libc read() call
    emitter.instruction("call __rt_stream_fd");                                 // resolve the backend descriptor through StreamState
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // preserve the resolved backend descriptor
    emitter.instruction("mov r9d, 0x40000000");                                 // materialize USER_WRAPPER_FD_BASE
    emitter.instruction("cmp rax, r9");                                         // is the backend below the wrapper range?
    emitter.instruction("jb __rt_fread_real_fd_x86");                           // native descriptors continue to libc read
    // The wrapper fd range ends at the allocated handle capacity, not a
    // fixed 256: a slot beyond the bound would be misread as a native fd.
    super::emit_load_handles_cap(emitter, "r10");
    emitter.instruction("add r10, r9");                                         // wrapper range end = USER_WRAPPER_FD_BASE + handle capacity
    emitter.instruction("cmp rax, r10");                                        // is the backend above the wrapper range?
    emitter.instruction("jae __rt_fread_real_fd_x86");                          // non-wrapper synthetic backends stay on the native path
    emitter.instruction("mov rdi, rax");                                        // pass the synthetic backend descriptor to stream_read
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // reload the requested byte count
    emitter.instruction("add rsp, 48");                                         // release native-read scratch storage
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("jmp __rt_user_wrapper_fread");                         // wrapper backend tail-calls stream_read
    emitter.label("__rt_fread_real_fd_x86");
    emitter.instruction("cmp rax, 0");                                          // did descriptor resolution produce a valid backend?
    emitter.instruction("jge __rt_fread_fd_ok_x86");                            // continue to the normal read path for non-negative descriptors
    emitter.instruction("xor eax, eax");                                        // return an empty string pointer for an invalid stream
    emitter.instruction("xor edx, edx");                                        // return an empty string length for an invalid stream
    emitter.instruction("add rsp, 48");                                         // release native-read scratch storage
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // skip the read path for invalid stream handles

    emitter.label("__rt_fread_fd_ok_x86");
    abi::emit_load_symbol_to_reg(emitter, "r10", "_concat_off", 0);             // load the current concat-buffer absolute offset before appending the fread() result
    abi::emit_symbol_address(emitter, "r11", "_concat_buf");                    // materialize the concat-buffer base address once for the x86_64 fread() helper
    emitter.instruction("lea rax, [r11 + r10]");                                // compute the start pointer for the bytes that libc read() will append
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // preserve the concat-buffer start pointer for the final elephc string result

    // -- TLS dispatch: the session hangs off the StreamState, so it is keyed by
    //    the generation-checked handle rather than by a reusable descriptor. --
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the opaque stream handle
    emitter.instruction("call __rt_stream_tls_session");                        // rax = attached session, zero when plain
    emitter.instruction("test rax, rax");                                       // is a TLS session attached?
    emitter.instruction("jz __rt_fread_do_syscall_x86");                        // no TLS attached → use libc read
    emitter.instruction("mov rdi, rax");                                        // handle as first arg
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // buf ptr
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // len
    abi::emit_load_symbol_to_reg(emitter, "r9", "_elephc_tls_read_fn", 0);      // prepare SysV call argument
    emitter.instruction("call r9");                                             // rax = bytes read (>=0) or -1
    emitter.instruction("cmp rax, 0");                                          // did the TLS bridge return bytes?
    emitter.instruction("jle __rt_fread_eof_x86");                              // TLS errors and EOF still mark the stream as exhausted
    emitter.instruction("jmp __rt_fread_read_ok_x86");                          // publish the successful TLS read
    emitter.label("__rt_fread_do_syscall_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // pass the file descriptor as the first libc read() argument
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // pass the concat-buffer write pointer as the second libc read() argument
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // pass the requested byte count as the third libc read() argument
    emitter.instruction("call read");                                           // read the requested bytes into the concat-buffer append window through libc read()
    emitter.instruction("cmp rax, 0");                                          // classify libc read() as bytes, EOF, or failure
    emitter.instruction("jg __rt_fread_read_ok_x86");                           // positive byte count: publish the successful read
    emitter.instruction("jl __rt_fread_read_failed_x86");                       // negative result: inspect errno before treating it as EOF
    emitter.instruction("jmp __rt_fread_eof_x86");                              // zero-byte read means real EOF

    emitter.label("__rt_fread_read_ok_x86");
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // preserve the actual byte count across EOF publication
    abi::emit_load_symbol_to_reg(emitter, "r10", "_concat_off", 0);             // reload the previous concat-buffer absolute offset before publishing the fread() append
    emitter.instruction("add r10, rax");                                        // advance the concat-buffer offset by the number of bytes libc read() returned
    abi::emit_store_reg_to_symbol(emitter, "r10", "_concat_off", 0);            // publish the updated concat-buffer offset for later string appenders
    emitter.instruction("cmp rax, QWORD PTR [rbp - 16]");                       // did the backend satisfy the complete request?
    emitter.instruction("jge __rt_fread_publish_x86");                          // a full read does not prove EOF
    emitter.instruction("test rax, rax");                                       // was this a universal zero-byte EOF read?
    emitter.instruction("jz __rt_fread_mark_eof_x86");                          // zero bytes mark every blocking backend exhausted
    emitter.instruction("mov rdi, QWORD PTR [rbp - 32]");                       // reload the backend descriptor for a seekability probe
    emitter.instruction("xor esi, esi");                                        // probe the current position without moving it
    emitter.instruction("mov edx, 1");                                          // select SEEK_CUR for the non-mutating probe
    emitter.instruction("call lseek");                                          // seekable short reads prove the regular stream was exhausted
    emitter.instruction("test rax, rax");                                       // did the position probe fail?
    emitter.instruction("js __rt_fread_publish_x86");                           // sockets and pipes may legally return partial data
    emitter.label("__rt_fread_mark_eof_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the opaque stream handle
    emitter.instruction("mov esi, 1");                                          // publish the EOF state
    emitter.instruction("call __rt_stream_eof_set");                            // update only this stream's stable state
    emitter.label("__rt_fread_publish_x86");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // return the successful byte count in the string-length register
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // return the concat-buffer start pointer in the x86_64 elephc string-pointer result register
    // -- apply the attached read filter chain to the bytes just read --
    // The chain is an arbitrarily long linked list rooted in StreamState, so this
    // replaces the old two-slot table that silently dropped a third filter.
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the opaque stream handle
    emitter.instruction(&format!(
        "mov rsi, {}",
        crate::codegen_support::runtime::resources::layout::STREAM_READ_FILTER_HEAD_OFFSET
    ));                                                                         // select the read-direction chain
    emitter.instruction("call __rt_stream_apply_filter_chain");                 // rax/rdx <- filtered buffer and length

    // Legacy per-descriptor slots still serve the filter families not yet moved
    // onto the chain (user filters, zlib/bzip2/iconv). A filter lives in exactly
    // one mechanism, so running both is correct for the duration of the migration.
    // -- apply attached read filters (2-slot chain) --
    //    Slot 0 = _stream_read_filters[fd], slot 1 = _stream_read_filters[fd+256]
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload the file descriptor
    abi::emit_symbol_address(emitter, "r11", "_stream_read_filters");           // materialize the read-filter table base
    // -- slot 0 --
    emitter.instruction("movzx ecx, BYTE PTR [r11 + r10]");                     // read filter id for slot 0
    emitter.instruction("test rcx, rcx");                                       // is slot 0 empty?
    emitter.instruction("jz __rt_fread_slot1_x86");                             // skip slot 0 when empty
    emitter.instruction("cmp rcx, 128");                                        // user-filter id range?
    emitter.instruction("jl __rt_fread_builtin_slot0_x86");                     // built-in filter
    emitter.instruction("mov rdi, r10");                                        // fd
    emitter.instruction("mov rsi, rax");                                        // buf ptr
    // rdx already holds the byte count
    emitter.instruction("xor ecx, ecx");                                        // direction = 0 (read)
    emitter.instruction("call __rt_apply_user_stream_filter");                  // rax/rdx ← transformed
    emitter.instruction("jmp __rt_fread_slot1_x86");                            // proceed to slot 1
    emitter.label("__rt_fread_builtin_slot0_x86");
    emitter.instruction("call __rt_apply_stream_filter");                       // transform in place
    // -- slot 1 --
    emitter.label("__rt_fread_slot1_x86");
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload fd
    abi::emit_symbol_address(emitter, "r11", "_stream_read_filters");           // materialize the read-filter table base
    emitter.instruction("lea rcx, [r10 + 256]");                                // fd+256 (slot 1 index)
    emitter.instruction("movzx ecx, BYTE PTR [r11 + rcx]");                     // read filter id for slot 1
    emitter.instruction("test rcx, rcx");                                       // is slot 1 empty?
    emitter.instruction("jz __rt_fread_ret_x86");                               // skip slot 1 when empty
    emitter.instruction("cmp rcx, 128");                                        // user-filter id range?
    emitter.instruction("jl __rt_fread_builtin_slot1_x86");                     // built-in filter
    emitter.instruction("mov rdi, r10");                                        // fd
    emitter.instruction("mov rsi, rax");                                        // buf ptr (post slot-0)
    // rdx already holds the byte count (post slot-0)
    emitter.instruction("xor ecx, ecx");                                        // direction = 0 (read)
    emitter.instruction("call __rt_apply_user_stream_filter");                  // rax/rdx ← transformed
    emitter.instruction("jmp __rt_fread_ret_x86");                              // common epilogue
    emitter.label("__rt_fread_builtin_slot1_x86");
    emitter.instruction("call __rt_apply_stream_filter");                       // transform in place
    emitter.label("__rt_fread_ret_x86");
    emitter.instruction("add rsp, 48");                                         // release the fread() spill slots before returning the successful string slice
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer after the successful fread() path
    emitter.instruction("ret");                                                 // return the borrowed concat-buffer string slice to the caller

    emitter.label("__rt_fread_read_failed_x86");
    emitter.instruction("call __errno_location");                               // fetch errno after libc read() failed
    emitter.instruction("mov r10d, DWORD PTR [rax]");                           // load the thread-local errno value
    emitter.instruction("cmp r10d, 11");                                        // is this EAGAIN/EWOULDBLOCK from a nonblocking fd?
    emitter.instruction("je __rt_fread_would_block_x86");                       // transient nonblocking miss returns empty without EOF
    emitter.instruction("jmp __rt_fread_eof_x86");                              // other read failures behave like an exhausted stream

    emitter.label("__rt_fread_would_block_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // return the concat-buffer start pointer for an empty transient read
    emitter.instruction("xor edx, edx");                                        // return a zero-length read result without setting EOF
    emitter.instruction("add rsp, 48");                                         // release the fread() spill slots before returning the empty string
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer after the would-block fread() path
    emitter.instruction("ret");                                                 // return the empty non-EOF read result

    emitter.label("__rt_fread_eof_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the opaque stream handle
    emitter.instruction("mov esi, 1");                                          // publish the EOF state
    emitter.instruction("call __rt_stream_eof_set");                            // mark this stream exhausted after the zero-byte or failed read
    emitter.instruction("xor eax, eax");                                        // return an empty string pointer when libc read() reports EOF or failure
    emitter.instruction("xor edx, edx");                                        // return an empty string length when libc read() reports EOF or failure
    emitter.instruction("add rsp, 48");                                         // release the fread() spill slots before returning the empty-string result
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer after the EOF/error fread() path
    emitter.instruction("ret");                                                 // return the empty string result for the exhausted or failed stream read
}
