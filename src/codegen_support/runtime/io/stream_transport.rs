//! Purpose:
//! Emits the helpers that record which transport a socket stream was opened on, and read it back.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - The `stream_socket_client()`, `stream_socket_server()`, `stream_socket_accept()` and
//!   `stream_socket_pair()` lowerings, once the socket has been boxed.
//!
//! Key details:
//! - `stream_get_meta_data()['stream_type']` names the transport, and no descriptor property
//!   distinguishes them: a TCP, UDP, Unix-domain and socketpair endpoint are all non-seekable
//!   sockets. php-src answers `tcp_socket/ssl`, `udp_socket`, `unix_socket` and `generic_socket`.
//! - The transport is read from the address the caller wrote, which is where the socket helpers
//!   read it too, so one spelling decides both what is opened and what it is called.
//! - A stream that records nothing keeps whatever the metadata helper derived, so this cannot
//!   rename a file.

use crate::codegen_support::runtime::resources::layout::{
    STREAM_TRANSPORT_OFFSET, STREAM_TRANSPORT_TCP, STREAM_TRANSPORT_UDP, STREAM_TRANSPORT_UNIX,
};
use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits `__rt_stream_record_transport(handle, addr_ptr, addr_len, fallback) -> handle`.
///
/// AArch64 receives `x0`/`x1`/`x2`/`x3`; x86_64 receives `rdi`/`rsi`/`rdx`/`rcx`. A null address
/// records `fallback` unchanged, which is how a socket pair and an accepted connection — neither
/// of which has an address of its own — get their name.
pub fn emit_stream_record_transport(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_record_transport_aarch64(emitter),
        Arch::X86_64 => emit_record_transport_x86_64(emitter),
    }
}

/// Emits `__rt_stream_transport(handle) -> kind`, `0` when the stream records none.
pub fn emit_stream_transport(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.blank();
            emitter.comment("--- runtime: read a stream's recorded transport ---");
            emitter.label_global("__rt_stream_transport");
            emitter.instruction("sub sp, sp, #16");                             // frame for the saved linkage
            emitter.instruction("stp x29, x30, [sp, #0]");                      // save frame pointer and return address
            emitter.instruction("mov x29, sp");                                 // establish the helper frame pointer
            emitter.instruction("bl __rt_stream_state");                        // resolve the owning stream state
            emitter.instruction("cbz x0, __rt_str_none");                       // a stale handle records nothing
            emitter.instruction(&format!("ldr x0, [x0, #{STREAM_TRANSPORT_OFFSET}]"));
            emitter.instruction("b __rt_str_return");
            emitter.label("__rt_str_none");
            emitter.instruction("mov x0, #0");
            emitter.label("__rt_str_return");
            emitter.instruction("ldp x29, x30, [sp, #0]");                      // restore frame pointer and return address
            emitter.instruction("add sp, sp, #16");                             // release the helper frame
            emitter.instruction("ret");
        }
        Arch::X86_64 => {
            emitter.blank();
            emitter.comment("--- runtime: read a stream's recorded transport ---");
            emitter.label_global("__rt_stream_transport");
            emitter.instruction("push rbp");                                    // preserve the caller frame pointer
            emitter.instruction("mov rbp, rsp");                                // establish the helper frame
            emitter.instruction("call __rt_stream_state");                      // resolve the owning stream state
            emitter.instruction("test rax, rax");
            emitter.instruction("jz __rt_str_none_x86");                        // a stale handle records nothing
            emitter.instruction(&format!(
                "mov rax, QWORD PTR [rax + {STREAM_TRANSPORT_OFFSET}]"
            ));
            emitter.instruction("jmp __rt_str_return_x86");
            emitter.label("__rt_str_none_x86");
            emitter.instruction("xor eax, eax");
            emitter.label("__rt_str_return_x86");
            emitter.instruction("mov rsp, rbp");                                // release the frame from rbp
            emitter.instruction("pop rbp");                                     // restore the caller frame pointer
            emitter.instruction("ret");
        }
    }
}

/// Emits the AArch64 recorder.
fn emit_record_transport_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: record the transport a socket was opened on ---");
    emitter.label_global("__rt_stream_record_transport");
    emitter.instruction("sub sp, sp, #48");                                     // frame for the handle, the address and the saved linkage
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // preserve the opaque stream handle
    emitter.instruction("mov x9, x1");                                          // the address the caller wrote
    emitter.instruction("mov x10, x2");                                         // and its byte length
    emitter.instruction("mov x11, x3");                                         // the fallback transport

    emitter.instruction("cbz x9, __rt_srt_resolved");                           // no address: the fallback stands
    emit_prefix_probe_aarch64(emitter, "udp", b"udp://", STREAM_TRANSPORT_UDP);
    emit_prefix_probe_aarch64(emitter, "unix", b"unix://", STREAM_TRANSPORT_UNIX);
    emit_prefix_probe_aarch64(emitter, "udg", b"udg://", STREAM_TRANSPORT_UNIX);
    emitter.instruction(&format!("mov x11, #{STREAM_TRANSPORT_TCP}"));          // every other spelling opens TCP
    emitter.label("__rt_srt_resolved");

    emitter.instruction("cbz x11, __rt_srt_done");                              // nothing to record
    emitter.instruction("ldr x0, [sp, #0]");                                    // the handle again
    emitter.instruction("str x11, [sp, #8]");                                   // preserve the resolved transport across the lookup
    emitter.instruction("bl __rt_stream_state");                                // resolve the owning stream state
    emitter.instruction("cbz x0, __rt_srt_done");                               // a stale handle records nothing
    emitter.instruction("ldr x11, [sp, #8]");                                   // reload the resolved transport
    emitter.instruction(&format!("str x11, [x0, #{STREAM_TRANSPORT_OFFSET}]")); // publish it for the metadata namer
    emitter.label("__rt_srt_done");
    emitter.instruction("ldr x0, [sp, #0]");                                    // hand the handle back unchanged
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the helper frame
    emitter.instruction("ret");
}

/// Emits one AArch64 prefix probe: on a match, `x11` takes `value` and the scan is over.
fn emit_prefix_probe_aarch64(emitter: &mut Emitter, tag: &str, prefix: &[u8], value: u64) {
    let miss = format!("__rt_srt_not_{tag}");
    emitter.instruction(&format!("cmp x10, #{}", prefix.len()));                // long enough to carry the scheme?
    emitter.instruction(&format!("b.lt {miss}"));
    for (offset, byte) in prefix.iter().enumerate() {
        emitter.instruction(&format!("ldrb w12, [x9, #{}]", offset));           // one candidate scheme byte
        emitter.instruction(&format!("cmp w12, #{}", byte));
        emitter.instruction(&format!("b.ne {miss}"));
    }
    emitter.instruction(&format!("mov x11, #{value}"));                         // the scheme names this transport
    emitter.instruction("b __rt_srt_resolved");
    emitter.label(&miss);
}

/// Emits the x86_64 recorder.
fn emit_record_transport_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: record the transport a socket was opened on ---");
    emitter.label_global("__rt_stream_record_transport");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame
    emitter.instruction("sub rsp, 32");                                         // reserve the handle and transport slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the opaque stream handle
    emitter.instruction("mov r9, rsi");                                         // the address the caller wrote
    emitter.instruction("mov r10, rdx");                                        // and its byte length
    emitter.instruction("mov r11, rcx");                                        // the fallback transport

    emitter.instruction("test r9, r9");
    emitter.instruction("jz __rt_srt_resolved_x86");                            // no address: the fallback stands
    emit_prefix_probe_x86_64(emitter, "udp", b"udp://", STREAM_TRANSPORT_UDP);
    emit_prefix_probe_x86_64(emitter, "unix", b"unix://", STREAM_TRANSPORT_UNIX);
    emit_prefix_probe_x86_64(emitter, "udg", b"udg://", STREAM_TRANSPORT_UNIX);
    emitter.instruction(&format!("mov r11, {STREAM_TRANSPORT_TCP}"));           // every other spelling opens TCP
    emitter.label("__rt_srt_resolved_x86");

    emitter.instruction("test r11, r11");
    emitter.instruction("jz __rt_srt_done_x86");                                // nothing to record
    emitter.instruction("mov QWORD PTR [rbp - 16], r11");                       // preserve the resolved transport across the lookup
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // the handle again
    emitter.instruction("call __rt_stream_state");                              // resolve the owning stream state
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_srt_done_x86");                                // a stale handle records nothing
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                       // reload the resolved transport
    emitter.instruction(&format!(
        "mov QWORD PTR [rax + {STREAM_TRANSPORT_OFFSET}], r11"
    ));                                                                         // publish it for the metadata namer
    emitter.label("__rt_srt_done_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // hand the handle back unchanged
    emitter.instruction("add rsp, 32");                                         // release the helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");
}

/// Emits one x86_64 prefix probe: on a match, `r11` takes `value` and the scan is over.
fn emit_prefix_probe_x86_64(emitter: &mut Emitter, tag: &str, prefix: &[u8], value: u64) {
    let miss = format!("__rt_srt_not_{tag}_x86");
    emitter.instruction(&format!("cmp r10, {}", prefix.len()));                 // long enough to carry the scheme?
    emitter.instruction(&format!("jl {miss}"));
    for (offset, byte) in prefix.iter().enumerate() {
        emitter.instruction(&format!("cmp BYTE PTR [r9 + {}], {}", offset, byte)); // one candidate scheme byte
        emitter.instruction(&format!("jne {miss}"));
    }
    emitter.instruction(&format!("mov r11, {value}"));                          // the scheme names this transport
    emitter.instruction("jmp __rt_srt_resolved_x86");
    emitter.label(&miss);
}
