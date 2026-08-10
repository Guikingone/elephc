//! Purpose:
//! Disk, host, directory, process, and basic file path calls.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;
use crate::codegen_support::runtime::io::{SOCKET_WARNING_FSOCKOPEN};

/// Lowers `disk_free_space(path)` through the shared disk-space runtime helper.
pub(crate) fn lower_disk_free_space(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    lower_disk_space(ctx, inst, "disk_free_space", 0)
}

/// Lowers `disk_total_space(path)` through the shared disk-space runtime helper.
pub(crate) fn lower_disk_total_space(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    lower_disk_space(ctx, inst, "disk_total_space", 1)
}

/// Loads a path and disk-space mode into `__rt_disk_space`.
pub(super) fn lower_disk_space(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
    mode: i64,
) -> Result<()> {
    super::super::ensure_arg_count(inst, name, 1)?;
    let path = expect_operand(inst, 0)?;
    load_string_to_result(ctx, path, name)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_int_immediate(ctx.emitter, "x0", mode);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, rax");                            // pass the path pointer as the second disk-space argument
            abi::emit_load_int_immediate(ctx.emitter, "rdi", mode);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_disk_space");
    store_if_result(ctx, inst)
}

/// Lowers `gethostname()` through the shared runtime helper.
pub(crate) fn lower_gethostname(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "gethostname", 0)?;
    abi::emit_call_label(ctx.emitter, "__rt_gethostname");
    store_if_result(ctx, inst)
}

/// Lowers `gethostbyname(hostname)` through the shared runtime resolver.
pub(crate) fn lower_gethostbyname(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "gethostbyname", 1)?;
    let host = expect_operand(inst, 0)?;
    load_string_to_result(ctx, host, "gethostbyname host")?;
    abi::emit_call_label(ctx.emitter, "__rt_gethostbyname");
    store_if_result(ctx, inst)
}

/// Lowers `gethostbyaddr(address)` and boxes malformed addresses as PHP `false`.
pub(crate) fn lower_gethostbyaddr(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "gethostbyaddr", 1)?;
    let address = expect_operand(inst, 0)?;
    load_string_to_result(ctx, address, "gethostbyaddr address")?;
    abi::emit_call_label(ctx.emitter, "__rt_gethostbyaddr");
    box_owned_string_or_false_result(ctx, "gethostbyaddr");
    store_if_result(ctx, inst)
}

/// Lowers `getprotobyname(protocol)` and boxes a missing entry as PHP `false`.
pub(crate) fn lower_getprotobyname(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "getprotobyname", 1)?;
    let protocol = expect_operand(inst, 0)?;
    load_string_to_result(ctx, protocol, "getprotobyname protocol")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // pass the protocol pointer as the first runtime argument
            ctx.emitter.instruction("mov x1, x2");                              // pass the protocol byte length as the second runtime argument
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // pass the protocol pointer as the first runtime argument
            ctx.emitter.instruction("mov rsi, rdx");                            // pass the protocol byte length as the second runtime argument
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_getprotobyname");
    box_negative_int_or_false_result(ctx, "getprotobyname");
    store_if_result(ctx, inst)
}

/// Lowers `getprotobynumber(number)` and boxes a missing entry as PHP `false`.
pub(crate) fn lower_getprotobynumber(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "getprotobynumber", 1)?;
    let protocol = expect_operand(inst, 0)?;
    require_int(
        ctx.load_value_to_result(protocol)?.codegen_repr(),
        "getprotobynumber number",
    )?;
    if ctx.emitter.target.arch == Arch::X86_64 {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the protocol number as the runtime argument
    }
    abi::emit_call_label(ctx.emitter, "__rt_getprotobynumber");
    box_owned_string_or_false_result(ctx, "getprotobynumber");
    store_if_result(ctx, inst)
}

/// Lowers `getservbyname(service, protocol)` and boxes a missing entry as PHP `false`.
pub(crate) fn lower_getservbyname(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "getservbyname", 2)?;
    let service = expect_operand(inst, 0)?;
    let protocol = expect_operand(inst, 1)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_to_result(ctx, service, "getservbyname service")?;
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            load_string_to_result(ctx, protocol, "getservbyname protocol")?;
            ctx.emitter.instruction("mov x3, x1");                              // pass the protocol pointer as the third runtime argument
            ctx.emitter.instruction("mov x4, x2");                              // pass the protocol byte length as the fourth runtime argument
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        }
        Arch::X86_64 => {
            load_string_to_result(ctx, service, "getservbyname service")?;
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            load_string_to_result(ctx, protocol, "getservbyname protocol")?;
            ctx.emitter.instruction("mov rcx, rdx");                            // pass the protocol byte length as the fourth runtime argument
            ctx.emitter.instruction("mov rdx, rax");                            // pass the protocol pointer as the third runtime argument
            abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_getservbyname");
    box_negative_int_or_false_result(ctx, "getservbyname");
    store_if_result(ctx, inst)
}

/// Lowers `getservbyport(port, protocol)` and boxes a missing entry as PHP `false`.
pub(crate) fn lower_getservbyport(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "getservbyport", 2)?;
    let port = expect_operand(inst, 0)?;
    let protocol = expect_operand(inst, 1)?;
    require_int(
        ctx.load_value_to_result(port)?.codegen_repr(),
        "getservbyport port",
    )?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x0", "x0");
            load_string_to_result(ctx, protocol, "getservbyport protocol")?;
            abi::emit_pop_reg_pair(ctx.emitter, "x0", "x9");
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rax");
            load_string_to_result(ctx, protocol, "getservbyport protocol")?;
            ctx.emitter.instruction("mov rsi, rax");                            // pass the protocol pointer as the second runtime argument
            abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rcx");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_getservbyport");
    box_owned_string_or_false_result(ctx, "getservbyport");
    store_if_result(ctx, inst)
}

/// Lowers `opendir(path)` and boxes the directory stream as `resource|false`.
pub(crate) fn lower_opendir(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "opendir", 1)?;
    // PHP creates the request default stream context on the FIRST stream open of any
    // kind, not only on `fopen`: after `opendir(".")` the directory handle is id 5 and
    // `stream_context_get_default()` answers 4, so the context was minted first. Doing
    // it here keeps directory handles on PHP's numbering instead of letting them take
    // the id the context owns.
    emit_request_default_stream_context_handle(ctx);
    let path = expect_operand(inst, 0)?;
    load_string_to_result(ctx, path, "opendir path")?;
    abi::emit_call_label(ctx.emitter, "__rt_opendir");
    box_stream_fd_or_false_result_kind(ctx, "opendir", 4, true, true);
    store_if_result(ctx, inst)
}

/// Lowers `readdir(dir_handle)` for libc, glob, and userspace-wrapper handles.
pub(crate) fn lower_readdir(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "readdir", 1)?;
    let handle = expect_operand(inst, 0)?;
    load_open_stream_handle_to_result(ctx, handle, "readdir")?;
    if matches!(ctx.emitter.target.arch, Arch::X86_64) {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the opaque directory handle to state resolution
    }
    abi::emit_call_label(ctx.emitter, "__rt_stream_state");
    if matches!(ctx.emitter.target.arch, Arch::X86_64) {
        ctx.emitter.instruction("mov rdi, rax");                                // pass authoritative StreamState to typed directory iteration
    }
    abi::emit_call_label(ctx.emitter, "__rt_readdir");
    box_owned_string_or_false_result(ctx, "readdir");
    store_if_result(ctx, inst)
}

/// Lowers `closedir(dir_handle)` for libc, glob, and userspace-wrapper handles.
pub(crate) fn lower_closedir(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "closedir", 1)?;
    let handle = expect_operand(inst, 0)?;
    begin_stream_close(ctx, handle, "closedir")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the opaque handle after publishing Closing
            abi::emit_call_label(ctx.emitter, "__rt_stream_state");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp]");                // reload the opaque handle after publishing Closing
            abi::emit_call_label(ctx.emitter, "__rt_stream_state");
            ctx.emitter.instruction("mov rdi, rax");                            // pass authoritative StreamState to typed directory cleanup
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_closedir");
    finish_stream_close(ctx);
    store_if_result(ctx, inst)
}

/// Lowers `rewinddir(dir_handle)` for libc, glob, and userspace-wrapper handles.
pub(crate) fn lower_rewinddir(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "rewinddir", 1)?;
    let handle = expect_operand(inst, 0)?;
    load_open_stream_handle_to_result(ctx, handle, "rewinddir")?;
    if matches!(ctx.emitter.target.arch, Arch::X86_64) {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the opaque directory handle to state resolution
    }
    abi::emit_call_label(ctx.emitter, "__rt_stream_state");
    if matches!(ctx.emitter.target.arch, Arch::X86_64) {
        ctx.emitter.instruction("mov rdi, rax");                                // pass authoritative StreamState to typed directory rewind
    }
    abi::emit_call_label(ctx.emitter, "__rt_rewinddir");
    store_if_result(ctx, inst)
}

/// Lowers `popen(command, mode)` and boxes the process pipe as `resource|false`.
pub(crate) fn lower_popen(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "popen", 2)?;
    let command = expect_operand(inst, 0)?;
    let mode = expect_operand(inst, 1)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_to_result(ctx, command, "popen command")?;
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            load_string_to_result(ctx, mode, "popen mode")?;
            ctx.emitter.instruction("mov x3, x1");                              // pass the mode pointer as the third runtime argument
            ctx.emitter.instruction("mov x4, x2");                              // pass the mode byte length as the fourth runtime argument
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        }
        Arch::X86_64 => {
            load_string_to_result(ctx, command, "popen command")?;
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            load_string_to_result(ctx, mode, "popen mode")?;
            ctx.emitter.instruction("mov rcx, rdx");                            // pass the mode byte length as the fourth runtime argument
            ctx.emitter.instruction("mov rdx, rax");                            // pass the mode pointer as the third runtime argument
            abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_popen");
    box_stream_fd_or_false_result_kind(ctx, "popen", 3, true, false);
    store_if_result(ctx, inst)
}

/// Lowers `pclose(handle)` and returns the child process status.
pub(crate) fn lower_pclose(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "pclose", 1)?;
    let handle = expect_operand(inst, 0)?;
    begin_stream_close(ctx, handle, "pclose")?;
    let popen_label = ctx.next_label("pclose_process");
    let invalid_label = ctx.next_label("pclose_invalid");
    let backend_done_label = ctx.next_label("pclose_backend_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // resolve the opaque process-pipe handle
            abi::emit_call_label(ctx.emitter, "__rt_stream_state");
            ctx.emitter.instruction(&format!("cbz x0, {}", invalid_label));     // reject an unexpectedly missing Closing StreamState
            ctx.emitter.instruction(&format!(
                "ldr x9, [x0, #{}]", STREAM_BACKEND_KIND_OFFSET
            ));                                                                 // inspect the authoritative backend discriminator
            ctx.emitter.instruction(&format!("cmp x9, #{}", STREAM_BACKEND_POPEN)); // does this stream own a child process?
            ctx.emitter.instruction(&format!("b.eq {}", popen_label));          // process pipes return their decoded child status
            ctx.emitter.instruction("ldr x0, [x0, #16]");                       // ordinary streams close their native descriptor
            ctx.emitter.instruction("cmp x0, #0");                              // is a descriptor available to close?
            let non_process_closed = ctx.next_label("pclose_non_process_closed");
            ctx.emitter.instruction(&format!("b.lt {}", non_process_closed));   // descriptor-less ordinary streams still return zero
            ctx.emitter.syscall(6);                                             // close the non-process stream through its native descriptor
            ctx.emitter.label(&non_process_closed);
            ctx.emitter.instruction("mov x0, #0");                              // PHP returns zero when pclose receives an ordinary stream
            ctx.emitter.instruction(&format!("b {}", backend_done_label));      // skip process-owner cleanup
            ctx.emitter.label(&popen_label);
            ctx.emitter.instruction(&format!(
                "ldr x9, [x0, #{}]", STREAM_BACKEND_AUX_OFFSET
            ));                                                                 // load the owning FILE* from stable StreamState
            ctx.emitter.instruction(&format!(
                "str xzr, [x0, #{}]", STREAM_BACKEND_AUX_OFFSET
            ));                                                                 // detach ownership before libc may re-enter cleanup
            ctx.emitter.instruction("mov x0, x9");                              // pass the owning FILE* to the runtime close helper
            abi::emit_call_label(ctx.emitter, "__rt_pclose");
            ctx.emitter.instruction(&format!("b {}", backend_done_label));      // join lifecycle publication after process cleanup
            ctx.emitter.label(&invalid_label);
            emit_closed_stream_type_error(ctx, "pclose");
            ctx.emitter.label(&backend_done_label);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp]");                // resolve the opaque process-pipe handle
            abi::emit_call_label(ctx.emitter, "__rt_stream_state");
            ctx.emitter.instruction("test rax, rax");                           // did Closing state resolution succeed?
            ctx.emitter.instruction(&format!("jz {}", invalid_label));          // reject an unexpectedly missing StreamState
            ctx.emitter.instruction(&format!(
                "mov r9, QWORD PTR [rax + {}]", STREAM_BACKEND_KIND_OFFSET
            ));                                                                 // inspect the authoritative backend discriminator
            ctx.emitter.instruction(&format!("cmp r9, {}", STREAM_BACKEND_POPEN)); // does this stream own a child process?
            ctx.emitter.instruction(&format!("je {}", popen_label));            // process pipes return their decoded child status
            ctx.emitter.instruction("mov rdi, QWORD PTR [rax + 16]");           // ordinary streams close their native descriptor
            ctx.emitter.instruction("test rdi, rdi");                           // is a descriptor available to close?
            let non_process_closed = ctx.next_label("pclose_non_process_closed");
            ctx.emitter.instruction(&format!("js {}", non_process_closed));     // descriptor-less ordinary streams still return zero
            ctx.emitter.instruction("call close");                              // close the non-process stream through libc
            ctx.emitter.label(&non_process_closed);
            ctx.emitter.instruction("xor eax, eax");                            // PHP returns zero when pclose receives an ordinary stream
            ctx.emitter.instruction(&format!("jmp {}", backend_done_label));    // skip process-owner cleanup
            ctx.emitter.label(&popen_label);
            ctx.emitter.instruction(&format!(
                "mov rdi, QWORD PTR [rax + {}]", STREAM_BACKEND_AUX_OFFSET
            ));                                                                 // load the owning FILE* from stable StreamState
            ctx.emitter.instruction(&format!(
                "mov QWORD PTR [rax + {}], 0", STREAM_BACKEND_AUX_OFFSET
            ));                                                                 // detach ownership before libc may re-enter cleanup
            abi::emit_call_label(ctx.emitter, "__rt_pclose");
            ctx.emitter.instruction(&format!("jmp {}", backend_done_label));    // join lifecycle publication after process cleanup
            ctx.emitter.label(&invalid_label);
            emit_closed_stream_type_error(ctx, "pclose");
            ctx.emitter.label(&backend_done_label);
        }
    }
    finish_stream_close(ctx);
    store_if_result(ctx, inst)
}

/// Lowers `fsockopen(host, port, errno?, errstr?, timeout?)`.
pub(crate) fn lower_fsockopen(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "fsockopen", 2, 5)?;
    let host = expect_operand(inst, 0)?;
    let port = expect_operand(inst, 1)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_to_result(ctx, host, "fsockopen host")?;
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            require_int(ctx.load_value_to_result(port)?.codegen_repr(), "fsockopen port")?;
            abi::emit_push_reg(ctx.emitter, "x0");
            if inst.operands.len() >= 5 {
                let timeout = expect_operand(inst, 4)?;
                ctx.load_value_to_result(timeout)?;
            }
            abi::emit_pop_reg(ctx.emitter, "x9");
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
            ctx.emitter.instruction("mov x0, x1");                              // pass hostname pointer as the first runtime argument
            ctx.emitter.instruction("mov x1, x2");                              // pass hostname byte length as the second runtime argument
            ctx.emitter.instruction("mov x2, x9");                              // pass TCP port as the third runtime argument
        }
        Arch::X86_64 => {
            load_string_to_result(ctx, host, "fsockopen host")?;
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            require_int(ctx.load_value_to_result(port)?.codegen_repr(), "fsockopen port")?;
            abi::emit_push_reg(ctx.emitter, "rax");
            if inst.operands.len() >= 5 {
                let timeout = expect_operand(inst, 4)?;
                ctx.load_value_to_result(timeout)?;
            }
            abi::emit_pop_reg(ctx.emitter, "r8");
            abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
            ctx.emitter.instruction("mov rdx, r8");                             // pass TCP port as the third runtime argument
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_fsockopen");
    emit_socket_open_failure_warning(ctx, host, Some(port), SOCKET_WARNING_FSOCKOPEN)?;
    store_socket_error_outputs(ctx, inst, 2, 3, true)?;
    box_stream_fd_or_false_result(ctx, "fsockopen");
    store_if_result(ctx, inst)
}

/// Lowers `file(path)` through the target-aware runtime line-array helper.
/// Lowers `file(path, flags)` through the target-aware runtime line-array helper.
///
/// PHP's `$flags` bitmask is an ordinary run-time integer, so it needs no literal: the helper
/// applies `FILE_IGNORE_NEW_LINES` / `FILE_SKIP_EMPTY_LINES` while it produces each line. The
/// flags are resolved and spilled BEFORE the path, because coercing a non-string path calls a
/// conversion helper that clobbers the caller-saved register the flags would otherwise sit in.
pub(crate) fn lower_file(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count_between(inst, "file", 1, 3)?;
    let path = expect_operand(inst, 0)?;
    // Publish `$context` for this read, like fopen()/file_get_contents()/readfile().
    let explicit_context = inst.operands.get(2).copied();
    begin_fopen_context_scope(ctx, explicit_context)?;
    match inst.operands.get(1).copied() {
        None => {
            load_string_to_result(ctx, path, "file")?;
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    ctx.emitter.instruction("mov x0, #0");                      // no $flags argument: request PHP's default behavior
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction("xor edi, edi");                    // no $flags argument: request PHP's default behavior
                }
            }
        }
        Some(flags) => {
            resolve_int_operand_to_result(ctx, flags, "file flags")?;
            abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
            load_string_to_result(ctx, path, "file")?;
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    abi::emit_pop_reg(ctx.emitter, "x0"); // restore the resolved $flags bitmask into the first runtime argument
                }
                Arch::X86_64 => {
                    abi::emit_pop_reg(ctx.emitter, "rdi"); // restore the resolved $flags bitmask into the first runtime argument
                }
            }
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_file");
    finish_fopen_context_scope(ctx);
    store_if_result(ctx, inst)
}

/// Lowers `realpath(path)` and boxes the owned runtime string-or-false result.
pub(crate) fn lower_realpath(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "realpath", 1)?;
    let path = expect_operand(inst, 0)?;
    load_string_to_result(ctx, path, "realpath")?;
    abi::emit_call_label(ctx.emitter, "__rt_realpath");
    box_owned_string_or_false_result(ctx, "realpath");
    store_if_result(ctx, inst)
}

/// Lowers `realpath_cache_get()` to elephc's empty realpath-cache view.
pub(crate) fn lower_realpath_cache_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "realpath_cache_get", 0)?;
    emit_empty_mixed_hash(ctx);
    store_if_result(ctx, inst)
}

/// Lowers `realpath_cache_size()` to zero because elephc has no realpath cache.
pub(crate) fn lower_realpath_cache_size(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "realpath_cache_size", 0)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, #0");                              // report an empty realpath cache size
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("xor rax, rax");                            // report an empty realpath cache size
        }
    }
    store_if_result(ctx, inst)
}

