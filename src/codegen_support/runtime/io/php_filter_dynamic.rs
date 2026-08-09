//! Purpose:
//! Emits the run-time half of `php://filter/...`: parsing a URL whose bytes are only known at
//! run time, and attaching the filter it names once the resource behind it is open.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//! - The dynamic `fopen()` lowering, which parses first and attaches after boxing.
//!
//! Key details:
//! - A filter URL is "open THIS, then filter it". The parse therefore hands the caller the
//!   RESOURCE and stops: the open path stays exactly the one a plain path takes, instead of the
//!   opener being re-implemented inside a helper that would also have to recurse for a
//!   `resource=php://temp` and carry the fopen mode down with it.
//! - Attaching needs nothing new. The literal path already goes through `__rt_filter_create` and
//!   `__rt_stream_filter_link`, two runtime helpers with plain arguments; the only difference
//!   here is that the id and direction are run-time values rather than immediates.
//! - An unrecognised filter name publishes direction 0, which opens the resource unfiltered —
//!   what the literal path does with the same URL.

use crate::codegen_support::abi;
use crate::codegen_support::{emit::Emitter, platform::Arch};

use crate::codegen_support::runtime::resources::layout::{
    STREAM_READ_FILTER_HEAD_OFFSET, STREAM_WRITE_FILTER_HEAD_OFFSET,
};

/// Emits `__rt_pf_match`, `__rt_php_filter_parse` and `__rt_php_filter_attach_pending`.
pub fn emit_php_filter_dynamic(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emit_prefix_match_aarch64(emitter);
            emit_filter_parse_aarch64(emitter);
            emit_filter_attach_aarch64(emitter);
        }
        Arch::X86_64 => {
            emit_prefix_match_x86_64(emitter);
            emit_filter_parse_x86_64(emitter);
            emit_filter_attach_x86_64(emitter);
        }
    }
}

/// `__rt_pf_match(x0 = haystack, x1 = length, x2 = needle, x3 = needle length) -> x0 = 0/1`.
fn emit_prefix_match_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: does a byte range start with a needle ---");
    emitter.label_global("__rt_pf_match");
    emitter.instruction("cmp x1, x3");                                          // enough bytes to hold the needle?
    emitter.instruction("b.lt __rt_pfm_no");                                    // too short to start with it
    emitter.instruction("mov x9, #0");                                          // comparison index
    emitter.label("__rt_pfm_byte");
    emitter.instruction("cmp x9, x3");                                          // compared the whole needle?
    emitter.instruction("b.hs __rt_pfm_yes");                                   // every byte agreed
    emitter.instruction("ldrb w10, [x0, x9]");                                  // one haystack byte
    emitter.instruction("ldrb w11, [x2, x9]");                                  // the corresponding needle byte
    emitter.instruction("cmp w10, w11");                                        // do they agree?
    emitter.instruction("b.ne __rt_pfm_no");                                    // a mismatch ends it
    emitter.instruction("add x9, x9, #1");                                      // advance the comparison index
    emitter.instruction("b __rt_pfm_byte");                                     // keep comparing
    emitter.label("__rt_pfm_yes");
    emitter.instruction("mov x0, #1");                                          // the range starts with the needle
    emitter.instruction("ret");
    emitter.label("__rt_pfm_no");
    emitter.instruction("mov x0, #0");                                          // it does not
    emitter.instruction("ret");
}

/// `__rt_php_filter_parse(x0 = path, x1 = length) -> x0 = 1 when a filter URL was parsed`.
fn emit_filter_parse_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: parse a run-time php://filter URL ---");
    emitter.label_global("__rt_php_filter_parse");
    // Frame: [0]=cursor [8]=remaining [16]=direction [24]=scan index, saved pair at [48].
    emitter.instruction("sub sp, sp, #64");                                     // reserve the parse frame
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // preserve the path
    emitter.instruction("str x1, [sp, #8]");                                    // preserve its length
    abi::emit_symbol_address(emitter, "x2", "_pf_n_prefix");
    emitter.instruction("mov x3, #13");                                         // "php://filter/"
    emitter.instruction("bl __rt_pf_match");                                    // is this a filter URL at all?
    emitter.instruction("cbz x0, __rt_pfp_no");                                 // no: leave the path alone
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the path
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload the length
    emitter.instruction("add x0, x0, #13");                                     // step past the scheme
    emitter.instruction("sub x1, x1, #13");                                     // and shorten the remaining count
    emitter.instruction("str x0, [sp, #0]");                                    // the cursor now sits on the direction
    emitter.instruction("str x1, [sp, #8]");

    emitter.instruction("mov x9, #3");                                          // no prefix means both directions
    emitter.instruction("str x9, [sp, #16]");
    abi::emit_symbol_address(emitter, "x2", "_pf_n_read");
    emitter.instruction("mov x3, #5");                                          // "read="
    emitter.instruction("bl __rt_pf_match");
    emitter.instruction("cbz x0, __rt_pfp_try_write");                          // not a read-only URL
    emitter.instruction("mov x9, #1");                                          // read direction
    emitter.instruction("str x9, [sp, #16]");
    emitter.instruction("ldr x0, [sp, #0]");
    emitter.instruction("ldr x1, [sp, #8]");
    emitter.instruction("add x0, x0, #5");                                      // step past "read="
    emitter.instruction("sub x1, x1, #5");
    emitter.instruction("str x0, [sp, #0]");
    emitter.instruction("str x1, [sp, #8]");
    emitter.instruction("b __rt_pfp_find_resource");

    emitter.label("__rt_pfp_try_write");
    emitter.instruction("ldr x0, [sp, #0]");
    emitter.instruction("ldr x1, [sp, #8]");
    abi::emit_symbol_address(emitter, "x2", "_pf_n_write");
    emitter.instruction("mov x3, #6");                                          // "write="
    emitter.instruction("bl __rt_pf_match");
    emitter.instruction("cbz x0, __rt_pfp_find_resource");                      // neither prefix: both directions
    emitter.instruction("mov x9, #2");                                          // write direction
    emitter.instruction("str x9, [sp, #16]");
    emitter.instruction("ldr x0, [sp, #0]");
    emitter.instruction("ldr x1, [sp, #8]");
    emitter.instruction("add x0, x0, #6");                                      // step past "write="
    emitter.instruction("sub x1, x1, #6");
    emitter.instruction("str x0, [sp, #0]");
    emitter.instruction("str x1, [sp, #8]");

    // -- scan for "/resource=", which separates the filter name from what it wraps --
    emitter.label("__rt_pfp_find_resource");
    emitter.instruction("mov x9, #0");                                          // scan index
    emitter.instruction("str x9, [sp, #24]");
    emitter.label("__rt_pfp_scan");
    emitter.instruction("ldr x9, [sp, #24]");                                   // the scan index
    emitter.instruction("ldr x1, [sp, #8]");                                    // bytes remaining after the direction
    emitter.instruction("add x10, x9, #10");                                    // does "/resource=" still fit here?
    emitter.instruction("cmp x10, x1");
    emitter.instruction("b.gt __rt_pfp_no");                                    // ran out: the URL names no resource
    emitter.instruction("ldr x0, [sp, #0]");                                    // the filter-name cursor
    emitter.instruction("add x0, x0, x9");                                      // the candidate separator position
    emitter.instruction("sub x1, x1, x9");                                      // bytes left from there
    abi::emit_symbol_address(emitter, "x2", "_pf_n_resource");
    emitter.instruction("mov x3, #10");                                         // "/resource="
    emitter.instruction("bl __rt_pf_match");
    emitter.instruction("cbnz x0, __rt_pfp_found");                             // the separator starts here
    emitter.instruction("ldr x9, [sp, #24]");
    emitter.instruction("add x9, x9, #1");                                      // keep scanning
    emitter.instruction("str x9, [sp, #24]");
    emitter.instruction("b __rt_pfp_scan");

    emitter.label("__rt_pfp_found");
    emitter.instruction("ldr x9, [sp, #24]");                                   // the separator offset IS the name length
    emitter.instruction("ldr x0, [sp, #0]");                                    // the filter name starts at the cursor
    emitter.instruction("ldr x1, [sp, #8]");                                    // bytes after the direction
    emitter.instruction("add x10, x0, x9");                                     // the separator
    emitter.instruction("add x10, x10, #10");                                   // the resource begins after it
    emitter.instruction("sub x11, x1, x9");                                     // bytes from the separator on
    emitter.instruction("sub x11, x11, #10");                                   // minus the separator itself
    emitter.instruction("cmp x11, #1");                                         // an empty resource names nothing
    emitter.instruction("b.lt __rt_pfp_no");                                    // reject it, as php-src does
    abi::emit_symbol_address(emitter, "x12", "_php_filter_res_ptr");
    emitter.instruction("str x10, [x12]");                                      // publish the resource pointer
    abi::emit_symbol_address(emitter, "x12", "_php_filter_res_len");
    emitter.instruction("str x11, [x12]");                                      // and its length
    // A resource that is itself a filter URL is what php-src refuses too.
    emitter.instruction("mov x0, x10");
    emitter.instruction("mov x1, x11");
    abi::emit_symbol_address(emitter, "x2", "_pf_n_prefix");
    emitter.instruction("mov x3, #12");                                         // "php://filter" without the slash
    emitter.instruction("bl __rt_pf_match");
    emitter.instruction("cbnz x0, __rt_pfp_no");                                // nested filters are not supported

    // Only the first filter of a `|`-separated list is applied, matching the literal path.
    emitter.instruction("ldr x9, [sp, #24]");                                   // the full name length
    emitter.instruction("ldr x0, [sp, #0]");                                    // the name
    emitter.instruction("mov x10, #0");                                         // scan index
    emitter.label("__rt_pfp_pipe");
    emitter.instruction("cmp x10, x9");                                         // reached the end of the name?
    emitter.instruction("b.hs __rt_pfp_resolve");                               // no pipe: use the whole name
    emitter.instruction("ldrb w11, [x0, x10]");
    emitter.instruction("cmp w11, #124");                                       // ASCII '|'
    emitter.instruction("b.eq __rt_pfp_resolve");                               // stop at the first one
    emitter.instruction("add x10, x10, #1");
    emitter.instruction("b __rt_pfp_pipe");

    emitter.label("__rt_pfp_resolve");
    emitter.instruction("mov x1, x10");                                         // the first filter's name length
    emitter.instruction("bl __rt_builtin_filter_id");                           // x0 = the built-in id, or 0
    abi::emit_symbol_address(emitter, "x12", "_php_filter_pending_id");
    emitter.instruction("str x0, [x12]");                                       // publish it
    emitter.instruction("ldr x9, [sp, #16]");                                   // the requested direction
    emitter.instruction("cmp x0, #0");                                          // did the name resolve?
    emitter.instruction("csel x9, xzr, x9, eq");                                // an unknown filter attaches nothing
    abi::emit_symbol_address(emitter, "x12", "_php_filter_pending_mode");
    emitter.instruction("str x9, [x12]");                                       // publish the direction
    emitter.instruction("mov x0, #1");                                          // the caller should open the resource
    emitter.instruction("ldp x29, x30, [sp, #48]");
    emitter.instruction("add sp, sp, #64");
    emitter.instruction("ret");

    emitter.label("__rt_pfp_no");
    abi::emit_symbol_address(emitter, "x12", "_php_filter_pending_id");
    emitter.instruction("str xzr, [x12]");                                      // nothing is pending
    abi::emit_symbol_address(emitter, "x12", "_php_filter_pending_mode");
    emitter.instruction("str xzr, [x12]");
    emitter.instruction("mov x0, #0");                                          // the path is not a usable filter URL
    emitter.instruction("ldp x29, x30, [sp, #48]");
    emitter.instruction("add sp, sp, #64");
    emitter.instruction("ret");
}

/// `__rt_php_filter_attach_pending(x0 = boxed fopen result)`; returns it unchanged.
fn emit_filter_attach_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: attach the filter a php://filter URL named ---");
    emitter.label_global("__rt_php_filter_attach_pending");
    // Frame: [0]=boxed result [8]=stream handle [16]=filter handle [24]=direction
    emitter.instruction("sub sp, sp, #48");                                     // reserve the attach frame
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // preserve the boxed result
    abi::emit_symbol_address(emitter, "x9", "_php_filter_pending_mode");
    emitter.instruction("ldr x10, [x9]");                                       // the direction the URL asked for
    emitter.instruction("str xzr, [x9]");                                       // clear it: exactly one open consumes it
    emitter.instruction("str x10, [sp, #24]");
    abi::emit_symbol_address(emitter, "x9", "_php_filter_pending_id");
    emitter.instruction("ldr x11, [x9]");                                       // the filter it named
    emitter.instruction("str xzr, [x9]");                                       // cleared for the same reason
    emitter.instruction("cbz x10, __rt_pfa_done");                              // no direction: nothing to attach
    emitter.instruction("cbz x11, __rt_pfa_done");                              // no filter: the resource opened plain
    emitter.instruction("ldr x0, [sp, #0]");
    emitter.instruction("ldr x9, [x0]");                                        // the boxed tag
    emitter.instruction("cmp x9, #9");                                          // did the open produce a resource?
    emitter.instruction("b.ne __rt_pfa_done");                                  // a false result carries no stream
    emitter.instruction("ldr x9, [x0, #8]");                                    // the opaque stream handle
    emitter.instruction("str x9, [sp, #8]");
    emitter.instruction("mov x0, x11");                                         // the built-in filter id
    emitter.instruction("mov x1, #0");                                          // built-ins carry no user-filter object
    emitter.instruction("ldr x2, [sp, #24]");                                   // direction bits from the URL
    emitter.instruction("mov x3, #0");                                          // built-ins retain no params value
    abi::emit_call_label(emitter, "__rt_filter_create");
    emitter.instruction("str x0, [sp, #16]");                                   // preserve the filter handle
    emitter.instruction("ldr x10, [sp, #24]");
    emitter.instruction("tst x10, #1");                                         // does it filter reads?
    emitter.instruction("b.eq __rt_pfa_write");
    emitter.instruction("ldr x0, [sp, #8]");                                    // stream handle
    emitter.instruction("ldr x1, [sp, #16]");                                   // filter handle
    emitter.instruction(&format!("mov x2, #{STREAM_READ_FILTER_HEAD_OFFSET}"));
    emitter.instruction("mov x3, #0");                                          // append at the chain tail
    abi::emit_call_label(emitter, "__rt_stream_filter_link");
    emitter.label("__rt_pfa_write");
    emitter.instruction("ldr x10, [sp, #24]");
    emitter.instruction("tst x10, #2");                                         // does it filter writes?
    emitter.instruction("b.eq __rt_pfa_done");
    emitter.instruction("ldr x0, [sp, #8]");                                    // stream handle
    emitter.instruction("ldr x1, [sp, #16]");                                   // filter handle
    emitter.instruction(&format!("mov x2, #{STREAM_WRITE_FILTER_HEAD_OFFSET}"));
    emitter.instruction("mov x3, #0");                                          // append at the chain tail
    abi::emit_call_label(emitter, "__rt_stream_filter_link");
    emitter.label("__rt_pfa_done");
    emitter.instruction("ldr x0, [sp, #0]");                                    // hand the boxed result straight back
    emitter.instruction("ldp x29, x30, [sp, #32]");
    emitter.instruction("add sp, sp, #48");
    emitter.instruction("ret");
}

/// x86_64 form of [`emit_prefix_match_aarch64`].
///
/// `__rt_pf_match(rdi = haystack, rsi = length, rdx = needle, rcx = needle length) -> rax = 0/1`.
fn emit_prefix_match_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: does a byte range start with a needle ---");
    emitter.label_global("__rt_pf_match");
    emitter.instruction("cmp rsi, rcx");                                        // enough bytes to hold the needle?
    emitter.instruction("jl __rt_pfm_no_x");                                    // too short to start with it
    emitter.instruction("xor r9, r9");                                          // comparison index
    emitter.label("__rt_pfm_byte_x");
    emitter.instruction("cmp r9, rcx");                                         // compared the whole needle?
    emitter.instruction("jae __rt_pfm_yes_x");                                  // every byte agreed
    emitter.instruction("movzx eax, BYTE PTR [rdi + r9]");                      // one haystack byte
    emitter.instruction("movzx r10d, BYTE PTR [rdx + r9]");                     // the corresponding needle byte
    emitter.instruction("cmp al, r10b");                                        // do they agree?
    emitter.instruction("jne __rt_pfm_no_x");                                   // a mismatch ends it
    emitter.instruction("add r9, 1");                                           // advance the comparison index
    emitter.instruction("jmp __rt_pfm_byte_x");                                 // keep comparing
    emitter.label("__rt_pfm_yes_x");
    emitter.instruction("mov rax, 1");                                          // the range starts with the needle
    emitter.instruction("ret");
    emitter.label("__rt_pfm_no_x");
    emitter.instruction("xor eax, eax");                                        // it does not
    emitter.instruction("ret");
}

/// x86_64 form of [`emit_filter_parse_aarch64`].
///
/// `__rt_php_filter_parse(rdi = path, rsi = length) -> rax = 1 when a filter URL was parsed`.
fn emit_filter_parse_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: parse a run-time php://filter URL ---");
    emitter.label_global("__rt_php_filter_parse");
    // Frame: [rbp-8]=cursor [rbp-16]=remaining [rbp-24]=direction [rbp-32]=scan index
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the parse frame
    emitter.instruction("sub rsp, 48");                                         // reserve the spill slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the path
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // preserve its length
    abi::emit_symbol_address(emitter, "rdx", "_pf_n_prefix");
    emitter.instruction("mov rcx, 13");                                         // "php://filter/"
    emitter.instruction("call __rt_pf_match");                                  // is this a filter URL at all?
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_pfp_no_x");                                    // no: leave the path alone
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");
    emitter.instruction("add rdi, 13");                                         // step past the scheme
    emitter.instruction("sub rsi, 13");                                         // and shorten the remaining count
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // the cursor now sits on the direction
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");

    emitter.instruction("mov QWORD PTR [rbp - 24], 3");                         // no prefix means both directions
    abi::emit_symbol_address(emitter, "rdx", "_pf_n_read");
    emitter.instruction("mov rcx, 5");                                          // "read="
    emitter.instruction("call __rt_pf_match");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_pfp_try_write_x");                             // not a read-only URL
    emitter.instruction("mov QWORD PTR [rbp - 24], 1");                         // read direction
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");
    emitter.instruction("add rdi, 5");                                          // step past "read="
    emitter.instruction("sub rsi, 5");
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");
    emitter.instruction("jmp __rt_pfp_find_resource_x");

    emitter.label("__rt_pfp_try_write_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");
    abi::emit_symbol_address(emitter, "rdx", "_pf_n_write");
    emitter.instruction("mov rcx, 6");                                          // "write="
    emitter.instruction("call __rt_pf_match");
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_pfp_find_resource_x");                         // neither prefix: both directions
    emitter.instruction("mov QWORD PTR [rbp - 24], 2");                         // write direction
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");
    emitter.instruction("add rdi, 6");                                          // step past "write="
    emitter.instruction("sub rsi, 6");
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");

    emitter.label("__rt_pfp_find_resource_x");
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                         // scan index
    emitter.label("__rt_pfp_scan_x");
    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");                        // the scan index
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // bytes remaining after the direction
    emitter.instruction("lea r10, [r9 + 10]");                                  // does "/resource=" still fit here?
    emitter.instruction("cmp r10, rsi");
    emitter.instruction("jg __rt_pfp_no_x");                                    // ran out: the URL names no resource
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // the filter-name cursor
    emitter.instruction("add rdi, r9");                                         // the candidate separator position
    emitter.instruction("sub rsi, r9");                                         // bytes left from there
    abi::emit_symbol_address(emitter, "rdx", "_pf_n_resource");
    emitter.instruction("mov rcx, 10");                                         // "/resource="
    emitter.instruction("call __rt_pf_match");
    emitter.instruction("test rax, rax");
    emitter.instruction("jnz __rt_pfp_found_x");                                // the separator starts here
    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");
    emitter.instruction("add r9, 1");                                           // keep scanning
    emitter.instruction("mov QWORD PTR [rbp - 32], r9");
    emitter.instruction("jmp __rt_pfp_scan_x");

    emitter.label("__rt_pfp_found_x");
    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");                        // the separator offset IS the name length
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // the filter name starts at the cursor
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // bytes after the direction
    emitter.instruction("lea r10, [rdi + r9]");                                 // the separator
    emitter.instruction("add r10, 10");                                         // the resource begins after it
    emitter.instruction("mov r11, rsi");
    emitter.instruction("sub r11, r9");                                         // bytes from the separator on
    emitter.instruction("sub r11, 10");                                         // minus the separator itself
    emitter.instruction("cmp r11, 1");                                          // an empty resource names nothing
    emitter.instruction("jl __rt_pfp_no_x");                                    // reject it, as php-src does
    abi::emit_symbol_address(emitter, "r8", "_php_filter_res_ptr");
    emitter.instruction("mov QWORD PTR [r8], r10");                             // publish the resource pointer
    abi::emit_symbol_address(emitter, "r8", "_php_filter_res_len");
    emitter.instruction("mov QWORD PTR [r8], r11");                             // and its length
    emitter.instruction("mov rdi, r10");
    emitter.instruction("mov rsi, r11");
    abi::emit_symbol_address(emitter, "rdx", "_pf_n_prefix");
    emitter.instruction("mov rcx, 12");                                         // "php://filter" without the slash
    emitter.instruction("call __rt_pf_match");
    emitter.instruction("test rax, rax");
    emitter.instruction("jnz __rt_pfp_no_x");                                   // nested filters are not supported

    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");                        // the full name length
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // the name
    emitter.instruction("xor r10, r10");                                        // scan index
    emitter.label("__rt_pfp_pipe_x");
    emitter.instruction("cmp r10, r9");                                         // reached the end of the name?
    emitter.instruction("jae __rt_pfp_resolve_x");                              // no pipe: use the whole name
    emitter.instruction("movzx eax, BYTE PTR [rdi + r10]");
    emitter.instruction("cmp eax, 124");                                        // ASCII '|'
    emitter.instruction("je __rt_pfp_resolve_x");                               // stop at the first one
    emitter.instruction("add r10, 1");
    emitter.instruction("jmp __rt_pfp_pipe_x");

    emitter.label("__rt_pfp_resolve_x");
    emitter.instruction("mov rsi, r10");                                        // the first filter's name length
    emitter.instruction("call __rt_builtin_filter_id");                         // rax = the built-in id, or 0
    abi::emit_symbol_address(emitter, "r8", "_php_filter_pending_id");
    emitter.instruction("mov QWORD PTR [r8], rax");                             // publish it
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // the requested direction
    emitter.instruction("xor r10, r10");
    emitter.instruction("test rax, rax");                                       // did the name resolve?
    emitter.instruction("cmove r9, r10");                                       // an unknown filter attaches nothing
    abi::emit_symbol_address(emitter, "r8", "_php_filter_pending_mode");
    emitter.instruction("mov QWORD PTR [r8], r9");                              // publish the direction
    emitter.instruction("mov rax, 1");                                          // the caller should open the resource
    emitter.instruction("mov rsp, rbp");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");

    emitter.label("__rt_pfp_no_x");
    abi::emit_symbol_address(emitter, "r8", "_php_filter_pending_id");
    emitter.instruction("mov QWORD PTR [r8], 0");                               // nothing is pending
    abi::emit_symbol_address(emitter, "r8", "_php_filter_pending_mode");
    emitter.instruction("mov QWORD PTR [r8], 0");
    emitter.instruction("xor eax, eax");                                        // the path is not a usable filter URL
    emitter.instruction("mov rsp, rbp");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");
}

/// x86_64 form of [`emit_filter_attach_aarch64`].
///
/// `__rt_php_filter_attach_pending(rax = boxed fopen result)`; returns it unchanged in rax.
fn emit_filter_attach_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: attach the filter a php://filter URL named ---");
    emitter.label_global("__rt_php_filter_attach_pending");
    // Frame: [rbp-8]=boxed result [rbp-16]=stream handle [rbp-24]=filter handle [rbp-32]=direction
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the attach frame
    emitter.instruction("sub rsp, 48");                                         // reserve the spill slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve the boxed result
    abi::emit_symbol_address(emitter, "r9", "_php_filter_pending_mode");
    emitter.instruction("mov r10, QWORD PTR [r9]");                             // the direction the URL asked for
    emitter.instruction("mov QWORD PTR [r9], 0");                               // clear it: exactly one open consumes it
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");
    abi::emit_symbol_address(emitter, "r9", "_php_filter_pending_id");
    emitter.instruction("mov r11, QWORD PTR [r9]");                             // the filter it named
    emitter.instruction("mov QWORD PTR [r9], 0");                               // cleared for the same reason
    emitter.instruction("test r10, r10");
    emitter.instruction("jz __rt_pfa_done_x");                                  // no direction: nothing to attach
    emitter.instruction("test r11, r11");
    emitter.instruction("jz __rt_pfa_done_x");                                  // no filter: the resource opened plain
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");
    emitter.instruction("cmp QWORD PTR [rax], 9");                              // did the open produce a resource?
    emitter.instruction("jne __rt_pfa_done_x");                                 // a false result carries no stream
    emitter.instruction("mov r9, QWORD PTR [rax + 8]");                         // the opaque stream handle
    emitter.instruction("mov QWORD PTR [rbp - 16], r9");
    emitter.instruction("mov rdi, r11");                                        // the built-in filter id
    emitter.instruction("xor esi, esi");                                        // built-ins carry no user-filter object
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // direction bits from the URL
    emitter.instruction("xor ecx, ecx");                                        // built-ins retain no params value
    abi::emit_call_label(emitter, "__rt_filter_create");
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // preserve the filter handle
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");
    emitter.instruction("test r10, 1");                                         // does it filter reads?
    emitter.instruction("jz __rt_pfa_write_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // stream handle
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // filter handle
    emitter.instruction(&format!("mov rdx, {STREAM_READ_FILTER_HEAD_OFFSET}"));
    emitter.instruction("xor ecx, ecx");                                        // append at the chain tail
    abi::emit_call_label(emitter, "__rt_stream_filter_link");
    emitter.label("__rt_pfa_write_x");
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");
    emitter.instruction("test r10, 2");                                         // does it filter writes?
    emitter.instruction("jz __rt_pfa_done_x");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // stream handle
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // filter handle
    emitter.instruction(&format!("mov rdx, {STREAM_WRITE_FILTER_HEAD_OFFSET}"));
    emitter.instruction("xor ecx, ecx");                                        // append at the chain tail
    abi::emit_call_label(emitter, "__rt_stream_filter_link");
    emitter.label("__rt_pfa_done_x");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // hand the boxed result straight back
    emitter.instruction("mov rsp, rbp");
    emitter.instruction("pop rbp");
    emitter.instruction("ret");
}
