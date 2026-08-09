//! Purpose:
//! Emits `__rt_data_stream_dynamic`, which opens an RFC 2397 `data://` URI whose bytes are only
//! known at run time.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - The dynamic `fopen()` lowering, when the path carries the `data://` prefix.
//!
//! Key details:
//! - A literal `data://` URI is decoded during lowering and its bytes embedded, which is why a
//!   run-time URI had no path at all and answered `false`.
//! - Decoding needs nothing new: `__rt_base64_decode` and `__rt_urldecode` already exist and both
//!   take and return elephc's string pair. `__rt_urldecode` also maps `+` to a space, which is
//!   exactly what the compile-time decoder does for these URIs, so the two agree.
//! - The `;base64` marker is matched case-insensitively, as php-src matches it.

use crate::codegen_support::abi;
use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits `__rt_data_stream_dynamic`.
pub fn emit_data_stream_dynamic(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// `__rt_data_stream_dynamic(x0 = uri, x1 = length) -> x0 = descriptor, or -1`.
fn emit_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: open a run-time data:// URI ---");
    emitter.label_global("__rt_data_stream_dynamic");
    // Frame: [0]=cursor [8]=remaining [16]=comma offset
    emitter.instruction("sub sp, sp, #48");                                     // reserve the decode frame
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the helper frame pointer
    emitter.instruction("cmp x1, #8");                                          // "data://" plus at least a comma
    emitter.instruction("b.lt __rt_dsd_no");                                    // too short to be a data URI
    emitter.instruction("add x0, x0, #7");                                      // step past the scheme
    emitter.instruction("sub x1, x1, #7");                                      // and shorten the remaining count
    emitter.instruction("str x0, [sp, #0]");                                    // the media-type cursor
    emitter.instruction("str x1, [sp, #8]");

    // -- the comma separates the media type from the payload --
    emitter.instruction("mov x9, #0");                                          // scan index
    emitter.label("__rt_dsd_comma");
    emitter.instruction("cmp x9, x1");                                          // ran off the end?
    emitter.instruction("b.hs __rt_dsd_no");                                    // no comma: not a usable data URI
    emitter.instruction("ldrb w10, [x0, x9]");
    emitter.instruction("cmp w10, #44");                                        // ASCII ','
    emitter.instruction("b.eq __rt_dsd_split");                                 // found the separator
    emitter.instruction("add x9, x9, #1");
    emitter.instruction("b __rt_dsd_comma");

    emitter.label("__rt_dsd_split");
    emitter.instruction("str x9, [sp, #16]");                                   // the media type's length

    // -- does the media type end with ";base64"? Matched case-insensitively, as php-src does --
    emitter.instruction("cmp x9, #7");                                          // room for the marker at all?
    emitter.instruction("b.lt __rt_dsd_percent");                               // no marker: percent-decode
    emitter.instruction("sub x11, x9, #7");                                     // where the marker would start
    emitter.instruction("add x11, x0, x11");                                    // its address
    abi::emit_symbol_address(emitter, "x12", "_data_n_b64");
    emitter.instruction("mov x13, #0");                                         // comparison index
    emitter.label("__rt_dsd_b64_byte");
    emitter.instruction("cmp x13, #7");                                         // compared the whole marker?
    emitter.instruction("b.hs __rt_dsd_base64");                                // every byte agreed
    emitter.instruction("ldrb w14, [x11, x13]");                                // one candidate byte
    emitter.instruction("ldrb w15, [x12, x13]");                                // the marker byte, already lower case
    emitter.instruction("cmp w14, #65");                                        // fold an upper-case letter
    emitter.instruction("b.lt __rt_dsd_b64_cmp");
    emitter.instruction("cmp w14, #90");
    emitter.instruction("b.gt __rt_dsd_b64_cmp");
    emitter.instruction("add w14, w14, #32");                                   // to its lower-case form
    emitter.label("__rt_dsd_b64_cmp");
    emitter.instruction("cmp w14, w15");
    emitter.instruction("b.ne __rt_dsd_percent");                               // not the marker: percent-decode
    emitter.instruction("add x13, x13, #1");
    emitter.instruction("b __rt_dsd_b64_byte");

    emitter.label("__rt_dsd_base64");
    emitter.instruction("ldr x0, [sp, #0]");                                    // the media-type cursor
    emitter.instruction("ldr x1, [sp, #8]");                                    // bytes after the scheme
    emitter.instruction("ldr x9, [sp, #16]");                                   // the media type's length
    emitter.instruction("add x2, x0, x9");                                      // the comma
    emitter.instruction("add x1, x2, #1");                                      // the payload starts after it
    emitter.instruction("ldr x2, [sp, #8]");
    emitter.instruction("sub x2, x2, x9");                                      // bytes from the comma on
    emitter.instruction("sub x2, x2, #1");                                      // minus the comma itself
    emitter.instruction("mov x0, #0");                                          // tolerant decoding, as the URI form is
    emitter.instruction("bl __rt_base64_decode");                               // x1/x2 = the decoded payload
    emitter.instruction("b __rt_dsd_open");

    emitter.label("__rt_dsd_percent");
    emitter.instruction("ldr x0, [sp, #0]");                                    // the media-type cursor
    emitter.instruction("ldr x9, [sp, #16]");                                   // the media type's length
    emitter.instruction("add x1, x0, x9");                                      // the comma
    emitter.instruction("add x1, x1, #1");                                      // the payload starts after it
    emitter.instruction("ldr x2, [sp, #8]");
    emitter.instruction("sub x2, x2, x9");                                      // bytes from the comma on
    emitter.instruction("sub x2, x2, #1");                                      // minus the comma itself
    emitter.instruction("bl __rt_urldecode");                                   // x1/x2 = the decoded payload

    emitter.label("__rt_dsd_open");
    emitter.instruction("mov x0, x1");                                          // the decoded bytes
    emitter.instruction("mov x1, x2");                                          // and their length
    emitter.instruction("bl __rt_data_stream");                                 // x0 = the descriptor
    emitter.instruction("ldp x29, x30, [sp, #32]");
    emitter.instruction("add sp, sp, #48");
    emitter.instruction("ret");

    emitter.label("__rt_dsd_no");
    emitter.instruction("mov x0, #-1");                                         // an unusable data URI opens nothing
    emitter.instruction("ldp x29, x30, [sp, #32]");
    emitter.instruction("add sp, sp, #48");
    emitter.instruction("ret");
}

/// x86_64 form of [`emit_aarch64`].
///
/// `__rt_data_stream_dynamic(rdi = uri, rsi = length) -> rax = descriptor, or -1`.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: open a run-time data:// URI ---");
    emitter.label_global("__rt_data_stream_dynamic");
    // Frame: [rbp-8]=cursor [rbp-16]=remaining [rbp-24]=comma offset
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the decode frame
    emitter.instruction("sub rsp, 32");                                         // reserve the spill slots
    emitter.instruction("cmp rsi, 8");                                          // "data://" plus at least a comma
    emitter.instruction("jl __rt_dsd_no_x");                                    // too short to be a data URI
    emitter.instruction("add rdi, 7");                                          // step past the scheme
    emitter.instruction("sub rsi, 7");                                          // and shorten the remaining count
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // the media-type cursor
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");

    emitter.instruction("xor r9, r9");                                          // scan index
    emitter.label("__rt_dsd_comma_x");
    emitter.instruction("cmp r9, rsi");                                         // ran off the end?
    emitter.instruction("jae __rt_dsd_no_x");                                   // no comma: not a usable data URI
    emitter.instruction("movzx eax, BYTE PTR [rdi + r9]");
    emitter.instruction("cmp eax, 44");                                         // ASCII ','
    emitter.instruction("je __rt_dsd_split_x");                                 // found the separator
    emitter.instruction("add r9, 1");
    emitter.instruction("jmp __rt_dsd_comma_x");

    emitter.label("__rt_dsd_split_x");
    emitter.instruction("mov QWORD PTR [rbp - 24], r9");                        // the media type's length

    emitter.instruction("cmp r9, 7");                                           // room for the ";base64" marker?
    emitter.instruction("jl __rt_dsd_percent_x");                               // no marker: percent-decode
    emitter.instruction("mov r11, r9");
    emitter.instruction("sub r11, 7");                                          // where the marker would start
    emitter.instruction("add r11, rdi");                                        // its address
    abi::emit_symbol_address(emitter, "r10", "_data_n_b64");
    emitter.instruction("xor rcx, rcx");                                        // comparison index
    emitter.label("__rt_dsd_b64_byte_x");
    emitter.instruction("cmp rcx, 7");                                          // compared the whole marker?
    emitter.instruction("jae __rt_dsd_base64_x");                               // every byte agreed
    emitter.instruction("movzx eax, BYTE PTR [r11 + rcx]");                     // one candidate byte
    emitter.instruction("movzx r8d, BYTE PTR [r10 + rcx]");                     // the marker byte, already lower case
    emitter.instruction("cmp eax, 65");                                         // fold an upper-case letter
    emitter.instruction("jl __rt_dsd_b64_cmp_x");
    emitter.instruction("cmp eax, 90");
    emitter.instruction("jg __rt_dsd_b64_cmp_x");
    emitter.instruction("add eax, 32");                                         // to its lower-case form
    emitter.label("__rt_dsd_b64_cmp_x");
    emitter.instruction("cmp eax, r8d");
    emitter.instruction("jne __rt_dsd_percent_x");                              // not the marker: percent-decode
    emitter.instruction("add rcx, 1");
    emitter.instruction("jmp __rt_dsd_b64_byte_x");

    emitter.label("__rt_dsd_base64_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // the media-type cursor
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // the media type's length
    emitter.instruction("lea rax, [rdi + r9]");                                 // the comma
    emitter.instruction("add rax, 1");                                          // the payload starts after it
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");
    emitter.instruction("sub rdx, r9");                                         // bytes from the comma on
    emitter.instruction("sub rdx, 1");                                          // minus the comma itself
    emitter.instruction("xor edi, edi");                                        // tolerant decoding, as the URI form is
    emitter.instruction("call __rt_base64_decode");                             // rax/rdx = the decoded payload
    emitter.instruction("jmp __rt_dsd_open_x");

    emitter.label("__rt_dsd_percent_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // the media-type cursor
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // the media type's length
    emitter.instruction("lea rax, [rdi + r9]");                                 // the comma
    emitter.instruction("add rax, 1");                                          // the payload starts after it
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");
    emitter.instruction("sub rdx, r9");                                         // bytes from the comma on
    emitter.instruction("sub rdx, 1");                                          // minus the comma itself
    emitter.instruction("call __rt_urldecode");                                 // rax/rdx = the decoded payload

    emitter.label("__rt_dsd_open_x");
    emitter.instruction("mov rdi, rax");                                        // the decoded bytes
    emitter.instruction("mov rsi, rdx");                                        // and their length
    emitter.instruction("call __rt_data_stream");                               // rax = the descriptor
    emitter.instruction("mov rsp, rbp");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");

    emitter.label("__rt_dsd_no_x");
    emitter.instruction("mov rax, -1");                                         // an unusable data URI opens nothing
    emitter.instruction("mov rsp, rbp");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");
}
