//! Purpose:
//! Emits the `__rt_stream_record_meta` runtime helper, which records the
//! wrapper id and URI string for a stream descriptor so that
//! `stream_get_meta_data()` can report the real `wrapper_type` and `uri`
//! instead of hardcoded `"plainfile"` / `""`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - `__rt_fopen`, `__rt_http_open`, `__rt_https_open`, `__rt_ftp_open`,
//!   `__rt_data_stream`, `__rt_tmpfile`, and other stream-open helpers,
//!   after a successful open that produced a raw fd.
//!
//! Key details:
//! - The URI is persisted via `__rt_str_persist` so it survives past the
//!   caller's string lifetime (the caller's buffer may be freed or reused).
//! - Tables: `_stream_wrapper_id[fd]` (u8), `_stream_uri_ptr[fd]` (u64),
//!   `_stream_uri_len[fd]` (u64), indexed by raw fd up to 256.
//! - fds >= 256 are silently skipped (no metadata recorded).

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// `__rt_stream_record_meta(fd, wrapper_id, uri_ptr, uri_len)`.
///
/// Persists the URI string and stores the wrapper id for the given fd so
/// `stream_get_meta_data()` can report accurate `wrapper_type` and `uri`.
///
/// Input:  AArch64 x0=fd, x1=wrapper_id, x2=uri_ptr, x3=uri_len.
///         x86_64  rdi=fd, rsi=wrapper_id, rdx=uri_ptr, rcx=uri_len.
/// Output: none (returns to caller; x0/rax is not preserved).
pub fn emit_stream_record_meta(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_stream_record_meta_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: stream_record_meta ---");
    emitter.label_global("__rt_stream_record_meta");

    // Frame: [0]=fd [8]=wrapper_id [16]=uri_ptr [24]=uri_len [32]=x29 [40]=x30
    emitter.instruction("sub sp, sp, #48");                                     // allocate the record-meta frame
    emitter.instruction("stp x29, x30, [sp, #32]");                              // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                     // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                     // save fd
    emitter.instruction("str x1, [sp, #8]");                                     // save wrapper_id
    emitter.instruction("str x2, [sp, #16]");                                    // save uri_ptr
    emitter.instruction("str x3, [sp, #24]");                                    // save uri_len

    // -- skip fds >= 256 (out of table range) --
    emitter.instruction("ldr x0, [sp, #0]");                                     // reload fd
    emitter.instruction("cmp x0, #256");                                         // fd >= 256?
    emitter.instruction("b.ge __rt_stream_record_meta_ret");                      // skip recording for out-of-range fds

    // -- persist the URI string so it survives past the caller's buffer --
    emitter.instruction("ldr x1, [sp, #16]");                                    // uri_ptr
    emitter.instruction("ldr x2, [sp, #24]");                                    // uri_len
    emitter.instruction("bl __rt_str_persist");                                  // x0 = persisted uri pointer
    emitter.instruction("str x0, [sp, #16]");                                    // save the persisted uri pointer

    // -- store wrapper_id at _stream_wrapper_id[fd] --
    emitter.instruction("ldr x0, [sp, #0]");                                     // reload fd
    emitter.instruction("ldr x1, [sp, #8]");                                     // reload wrapper_id
    abi::emit_symbol_address(emitter, "x2", "_stream_wrapper_id");               // wrapper-id table base
    emitter.instruction("strb w1, [x2, x0]");                                     // _stream_wrapper_id[fd] = wrapper_id (low byte)

    // -- store uri_ptr at _stream_uri_ptr[fd] --
    abi::emit_symbol_address(emitter, "x2", "_stream_uri_ptr");                   // uri-ptr table base
    emitter.instruction("ldr x3, [sp, #16]");                                    // reload persisted uri pointer
    emitter.instruction("str x3, [x2, x0, lsl #3]");                              // _stream_uri_ptr[fd] = uri_ptr

    // -- store uri_len at _stream_uri_len[fd] --
    abi::emit_symbol_address(emitter, "x2", "_stream_uri_len");                   // uri-len table base
    emitter.instruction("ldr x3, [sp, #24]");                                    // reload uri_len
    emitter.instruction("str x3, [x2, x0, lsl #3]");                              // _stream_uri_len[fd] = uri_len

    emitter.label("__rt_stream_record_meta_ret");
    emitter.instruction("ldp x29, x30, [sp, #32]");                               // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                        // release the record-meta frame
    emitter.instruction("ret");                                                  // return to the caller
}

/// x86_64 Linux variant of `__rt_stream_record_meta`.
fn emit_stream_record_meta_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: stream_record_meta ---");
    emitter.label_global("__rt_stream_record_meta");

    // Frame: rbp-relative, [-8]=fd [-16]=wrapper_id [-24]=uri_ptr [-32]=uri_len
    emitter.instruction("push rbp");                                             // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                         // establish the helper frame pointer
    emitter.instruction("sub rsp, 32");                                          // reserve the record-meta spill slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                          // save fd
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                         // save wrapper_id
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                         // save uri_ptr
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                         // save uri_len

    // -- skip fds >= 256 (out of table range) --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                          // reload fd
    emitter.instruction("cmp rax, 256");                                          // fd >= 256?
    emitter.instruction("jae __rt_stream_record_meta_ret_x86");                   // skip recording for out-of-range fds

    // -- persist the URI string so it survives past the caller's buffer --
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                         // uri_ptr
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                         // uri_len
    emitter.instruction("call __rt_str_persist");                                 // rax = persisted uri pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                         // save the persisted uri pointer

    // -- store wrapper_id at _stream_wrapper_id[fd] --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                          // reload fd
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                         // reload wrapper_id
    abi::emit_symbol_address(emitter, "r10", "_stream_wrapper_id");               // wrapper-id table base
    emitter.instruction("mov BYTE PTR [r10 + rax], sil");                        // _stream_wrapper_id[fd] = wrapper_id (low byte)

    // -- store uri_ptr at _stream_uri_ptr[fd] --
    abi::emit_symbol_address(emitter, "r10", "_stream_uri_ptr");                  // uri-ptr table base
    emitter.instruction("mov r11, QWORD PTR [rbp - 24]");                         // reload persisted uri pointer
    emitter.instruction("mov QWORD PTR [r10 + rax * 8], r11");                    // _stream_uri_ptr[fd] = uri_ptr

    // -- store uri_len at _stream_uri_len[fd] --
    abi::emit_symbol_address(emitter, "r10", "_stream_uri_len");                  // uri-len table base
    emitter.instruction("mov r11, QWORD PTR [rbp - 32]");                         // reload uri_len
    emitter.instruction("mov QWORD PTR [r10 + rax * 8], r11");                    // _stream_uri_len[fd] = uri_len

    emitter.label("__rt_stream_record_meta_ret_x86");
    emitter.instruction("leave");                                                 // restore rbp + rsp
    emitter.instruction("ret");                                                  // return to the caller
}