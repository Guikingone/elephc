//! Purpose:
//! Stream get/copy scratch frames, bounds, and target loops.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;

/// Reserves temporary storage for `stream_get_contents` fd and length operands.
pub(super) fn emit_stream_get_contents_frame_enter(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("sub sp, sp, #32");                         // reserve aligned temporary storage for fd and length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("sub rsp, 32");                             // reserve aligned temporary storage for fd and length
        }
    }
}

/// Releases temporary storage used by `stream_get_contents`.
pub(super) fn emit_stream_get_contents_frame_leave(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("add sp, sp, #32");                         // release stream_get_contents temporary storage
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("add rsp, 32");                             // release stream_get_contents temporary storage
        }
    }
}

/// Saves the currently loaded stream descriptor in the temporary frame.
pub(super) fn emit_stream_get_contents_save_fd(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp, #0]");                        // save the stream descriptor across length and offset evaluation
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov QWORD PTR [rsp + 0], rax");            // save the stream descriptor across length and offset evaluation
        }
    }
}

/// Saves the currently loaded length value in the temporary frame.
pub(super) fn emit_stream_get_contents_save_length(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp, #8]");                        // save the requested byte count or unlimited sentinel
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rax");            // save the requested byte count or unlimited sentinel
        }
    }
}

/// Reloads the saved fd and releases the `stream_get_contents` temporary frame.
pub(super) fn lower_stream_get_contents_reload_fd_and_leave_frame(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the stream descriptor for the read-all path
            emit_stream_get_contents_frame_leave(ctx);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 0]");            // reload the stream descriptor for the read-all path
            emit_stream_get_contents_frame_leave(ctx);
        }
    }
}

/// Applies the optional `stream_get_contents` seek before reading.
pub(super) fn lower_stream_get_contents_seek(
    ctx: &mut FunctionContext<'_>,
    skip_seek: &str,
    wrap_seek: &str,
    seek_failed: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #0");                              // a negative offset means no seek is requested
            ctx.emitter.instruction(&format!("b.lt {}", skip_seek));            // keep the current position for negative offsets
            ctx.emitter.instruction("mov x1, x0");                              // pass offset as the second seek argument
            ctx.emitter.instruction("mov x2, #0");                              // pass SEEK_SET as the third seek argument
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the stream descriptor for seeking
            ctx.emitter.instruction("mov w9, #0x4000");                         // materialize the high half of USER_WRAPPER_FD_BASE
            ctx.emitter.instruction("lsl w9, w9, #16");                         // form the synthetic wrapper fd base 0x40000000
            ctx.emitter.instruction("cmp x0, x9");                              // test whether the handle is a synthetic wrapper fd
            ctx.emitter.instruction(&format!("b.ge {}", wrap_seek));            // dispatch synthetic handles to wrapper stream_seek
            ctx.emitter.syscall(199);
            if ctx.emitter.platform.needs_cmp_before_error_branch() {
                ctx.emitter.instruction("cmp x0, #0");                          // Linux reports lseek failure as a negative result
            }
            ctx.emitter.instruction(&ctx.emitter.platform.branch_on_syscall_success(skip_seek)); // continue only when lseek succeeded
            ctx.emitter.instruction(&format!("b {}", seek_failed));             // failed seek makes stream_get_contents return false
            ctx.emitter.label(wrap_seek);
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_fseek");
            ctx.emitter.instruction("cmp x0, #0");                              // wrapper fseek returns zero on success
            ctx.emitter.instruction(&format!("b.ne {}", seek_failed));          // failed wrapper seek makes stream_get_contents return false
            ctx.emitter.label(skip_seek);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 0");                              // a negative offset means no seek is requested
            ctx.emitter.instruction(&format!("jl {}", skip_seek));              // keep the current position for negative offsets
            ctx.emitter.instruction("mov rsi, rax");                            // pass offset as the second seek argument
            ctx.emitter.instruction("xor edx, edx");                            // pass SEEK_SET as the third seek argument
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // reload the stream descriptor for seeking
            ctx.emitter.instruction("mov r9d, 0x40000000");                     // materialize USER_WRAPPER_FD_BASE for synthetic handles
            ctx.emitter.instruction("cmp rdi, r9");                             // test whether the handle is a synthetic wrapper fd
            ctx.emitter.instruction(&format!("jge {}", wrap_seek));             // dispatch synthetic handles to wrapper stream_seek
            ctx.emitter.instruction("call lseek");                              // seek the native stream descriptor
            ctx.emitter.instruction("cmp rax, 0");                              // test whether lseek returned a non-negative offset
            ctx.emitter.instruction(&format!("jl {}", seek_failed));            // failed seek makes stream_get_contents return false
            ctx.emitter.instruction(&format!("jmp {}", skip_seek));             // continue after a successful native seek
            ctx.emitter.label(wrap_seek);
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_fseek");
            ctx.emitter.instruction("cmp rax, 0");                              // wrapper fseek returns zero on success
            ctx.emitter.instruction(&format!("jne {}", seek_failed));           // failed wrapper seek makes stream_get_contents return false
            ctx.emitter.label(skip_seek);
        }
    }
}

/// Performs a finite bounded read or jumps to read-all for null/negative length.
pub(super) fn lower_stream_get_contents_bounded_or_all(
    ctx: &mut FunctionContext<'_>,
    read_all: &str,
    done: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x9, [sp, #8]");                        // reload the requested byte count
            emit_branch_if_unlimited_length(ctx, "x9", "x10", read_all);
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the stream descriptor for bounded reading
            ctx.emitter.instruction("mov x1, x9");                              // pass the finite byte count to the bounded helper
            emit_stream_get_contents_frame_leave(ctx);
            abi::emit_call_label(ctx.emitter, "__rt_stream_get_contents_bounded");
            crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Str);
            ctx.emitter.instruction(&format!("b {}", done));                    // bounded read completed successfully
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov r9, QWORD PTR [rsp + 8]");             // reload the requested byte count
            emit_branch_if_unlimited_length(ctx, "r9", "r10", read_all);
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 0]");            // reload the stream descriptor for bounded reading
            ctx.emitter.instruction("mov rdi, rax");                            // pass the stream descriptor to the bounded helper
            ctx.emitter.instruction("mov rsi, r9");                             // pass the finite byte count to the bounded helper
            emit_stream_get_contents_frame_leave(ctx);
            abi::emit_call_label(ctx.emitter, "__rt_stream_get_contents_bounded");
            crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Str);
            ctx.emitter.instruction(&format!("jmp {}", done));                  // bounded read completed successfully
        }
    }
}

/// Branches when a length value means "read until EOF".
pub(super) fn emit_branch_if_unlimited_length(
    ctx: &mut FunctionContext<'_>,
    length_reg: &str,
    scratch_reg: &str,
    target_label: &str,
) {
    abi::emit_load_int_immediate(ctx.emitter, scratch_reg, NULL_SENTINEL);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cmp {}, {}", length_reg, scratch_reg)); // test whether length is PHP null
            ctx.emitter.instruction(&format!("b.eq {}", target_label));         // null length means read until EOF
            ctx.emitter.instruction(&format!("cmp {}, #0", length_reg));        // test whether length is negative
            ctx.emitter.instruction(&format!("b.lt {}", target_label));         // negative length means read until EOF
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp {}, {}", length_reg, scratch_reg)); // test whether length is PHP null
            ctx.emitter.instruction(&format!("je {}", target_label));           // null length means read until EOF
            ctx.emitter.instruction(&format!("cmp {}, 0", length_reg));         // test whether length is negative
            ctx.emitter.instruction(&format!("jl {}", target_label));           // negative length means read until EOF
        }
    }
}

/// Creates the `stream_copy_to_stream` scratch frame after source/destination fd loading.
pub(super) fn emit_stream_copy_frame_enter(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, x0");                              // preserve the destination descriptor while restoring the source
            abi::emit_pop_reg(ctx.emitter, "x0");
            ctx.emitter.instruction("sub sp, sp, #48");                         // reserve source, destination, total, chunk, and max-length slots
            ctx.emitter.instruction("str x0, [sp, #0]");                        // save the source descriptor
            ctx.emitter.instruction("str x1, [sp, #8]");                        // save the destination descriptor
            ctx.emitter.instruction("str xzr, [sp, #16]");                      // initialize copied-byte total to zero
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, rax");                            // preserve the destination descriptor while restoring the source
            abi::emit_pop_reg(ctx.emitter, "rdi");
            ctx.emitter.instruction("sub rsp, 48");                             // reserve source, destination, total, chunk, and max-length slots
            ctx.emitter.instruction("mov QWORD PTR [rsp + 0], rdi");            // save the source descriptor
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rsi");            // save the destination descriptor
            ctx.emitter.instruction("mov QWORD PTR [rsp + 16], 0");             // initialize copied-byte total to zero
        }
    }
}

/// Releases the `stream_copy_to_stream` scratch frame.
pub(super) fn emit_stream_copy_frame_leave(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("add sp, sp, #48");                         // release stream_copy_to_stream temporary storage
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("add rsp, 48");                             // release stream_copy_to_stream temporary storage
        }
    }
}

/// Saves the optional `stream_copy_to_stream` length or the unlimited sentinel.
pub(super) fn materialize_stream_copy_length(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 3 {
        let length = expect_operand(inst, 2)?;
        require_optional_int(
            ctx.load_value_to_result(length)?.codegen_repr(),
            "stream_copy_to_stream length",
        )?;
    } else {
        abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), -1);
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp, #32]");                       // save requested max bytes or the unlimited sentinel
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov QWORD PTR [rsp + 32], rax");           // save requested max bytes or the unlimited sentinel
        }
    }
    Ok(())
}

/// Applies the optional `stream_copy_to_stream` source seek before copying.
pub(super) fn lower_stream_copy_seek(
    ctx: &mut FunctionContext<'_>,
    skip_seek: &str,
    wrap_seek: &str,
    seek_failed: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #0");                              // a negative offset means no seek is requested
            ctx.emitter.instruction(&format!("b.lt {}", skip_seek));            // keep the current source position for negative offsets
            ctx.emitter.instruction("mov x1, x0");                              // pass offset as the second seek argument
            ctx.emitter.instruction("mov x2, #0");                              // pass SEEK_SET as the third seek argument
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the source descriptor for seeking
            ctx.emitter.instruction("mov w9, #0x4000");                         // materialize the high half of USER_WRAPPER_FD_BASE
            ctx.emitter.instruction("lsl w9, w9, #16");                         // form the synthetic wrapper fd base 0x40000000
            ctx.emitter.instruction("cmp x0, x9");                              // test whether the source is a synthetic wrapper fd
            ctx.emitter.instruction(&format!("b.ge {}", wrap_seek));            // dispatch synthetic handles to wrapper stream_seek
            ctx.emitter.syscall(199);
            if ctx.emitter.platform.needs_cmp_before_error_branch() {
                ctx.emitter.instruction("cmp x0, #0");                          // Linux reports lseek failure as a negative result
            }
            ctx.emitter.instruction(&ctx.emitter.platform.branch_on_syscall_success(skip_seek)); // continue only when lseek succeeded
            ctx.emitter.instruction(&format!("b {}", seek_failed));             // failed native seek returns PHP false
            ctx.emitter.label(wrap_seek);
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_fseek");
            ctx.emitter.instruction("cmp x0, #0");                              // wrapper fseek returns zero on success
            ctx.emitter.instruction(&format!("b.ne {}", seek_failed));          // failed wrapper seek returns PHP false
            ctx.emitter.label(skip_seek);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 0");                              // a negative offset means no seek is requested
            ctx.emitter.instruction(&format!("jl {}", skip_seek));              // keep the current source position for negative offsets
            ctx.emitter.instruction("mov rsi, rax");                            // pass offset as the second seek argument
            ctx.emitter.instruction("xor edx, edx");                            // pass SEEK_SET as the third seek argument
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // reload the source descriptor for seeking
            ctx.emitter.instruction("mov r9d, 0x40000000");                     // materialize USER_WRAPPER_FD_BASE for synthetic handles
            ctx.emitter.instruction("cmp rdi, r9");                             // test whether the source is a synthetic wrapper fd
            ctx.emitter.instruction(&format!("jge {}", wrap_seek));             // dispatch synthetic handles to wrapper stream_seek
            ctx.emitter.instruction("call lseek");                              // seek the native stream descriptor
            ctx.emitter.instruction("cmp rax, 0");                              // test whether lseek returned a non-negative offset
            ctx.emitter.instruction(&format!("jl {}", seek_failed));            // failed native seek returns PHP false
            ctx.emitter.instruction(&format!("jmp {}", skip_seek));             // continue after a successful native seek
            ctx.emitter.label(wrap_seek);
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_fseek");
            ctx.emitter.instruction("cmp rax, 0");                              // wrapper fseek returns zero on success
            ctx.emitter.instruction(&format!("jne {}", seek_failed));           // failed wrapper seek returns PHP false
            ctx.emitter.label(skip_seek);
        }
    }
}

/// Copies source chunks into the destination and boxes the int-or-false result.
pub(super) fn lower_stream_copy_loop_and_box(
    ctx: &mut FunctionContext<'_>,
    seek_failed: &str,
    boxed_done: &str,
) {
    let loop_label = ctx.next_label("scs_loop");
    let done_label = ctx.next_label("scs_done");
    let length_unlimited = ctx.next_label("scs_length_unlimited");
    let after_length_check = ctx.next_label("scs_after_length_check");
    let request_unlimited = ctx.next_label("scs_request_unlimited");
    let after_request = ctx.next_label("scs_after_request");
    let chunk_unlimited = ctx.next_label("scs_chunk_unlimited");
    let after_chunk = ctx.next_label("scs_after_chunk");
    ctx.emitter.label(&loop_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_stream_copy_loop_aarch64(
            ctx,
            &loop_label,
            &done_label,
            &length_unlimited,
            &after_length_check,
            &request_unlimited,
            &after_request,
            &chunk_unlimited,
            &after_chunk,
        ),
        Arch::X86_64 => lower_stream_copy_loop_x86_64(
            ctx,
            &loop_label,
            &done_label,
            &length_unlimited,
            &after_length_check,
            &request_unlimited,
            &after_request,
            &chunk_unlimited,
            &after_chunk,
        ),
    }
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Int);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("b {}", boxed_done));              // successful copy skips the seek-failure false result
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("jmp {}", boxed_done));            // successful copy skips the seek-failure false result
        }
    }
    ctx.emitter.label(seek_failed);
    emit_stream_copy_frame_leave(ctx);
    emit_bool_result(ctx, false);
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Bool);
    ctx.emitter.label(boxed_done);
}

/// Emits the AArch64 stream-copy read/write loop.
pub(super) fn lower_stream_copy_loop_aarch64(
    ctx: &mut FunctionContext<'_>,
    loop_label: &str,
    done_label: &str,
    length_unlimited: &str,
    after_length_check: &str,
    request_unlimited: &str,
    after_request: &str,
    chunk_unlimited: &str,
    after_chunk: &str,
) {
    ctx.emitter.instruction("ldr x9, [sp, #16]");                               // load copied-byte total so far
    ctx.emitter.instruction("ldr x10, [sp, #32]");                              // load requested max byte count
    emit_branch_if_unlimited_length(ctx, "x10", "x11", length_unlimited);
    ctx.emitter.instruction("cmp x9, x10");                                     // test whether requested byte count is complete
    ctx.emitter.instruction(&format!("b.ge {}", done_label));                   // finish once length bytes have been copied
    ctx.emitter.instruction(&format!("b {}", after_length_check));              // continue with a finite request
    ctx.emitter.label(length_unlimited);
    ctx.emitter.label(after_length_check);
    ctx.emitter.instruction("ldr x0, [sp, #0]");                                // reload the source descriptor
    ctx.emitter.instruction("mov x1, #4096");                                   // request up to 4096 bytes by default
    ctx.emitter.instruction("ldr x10, [sp, #32]");                              // load requested max byte count
    emit_branch_if_unlimited_length(ctx, "x10", "x11", request_unlimited);
    ctx.emitter.instruction("ldr x9, [sp, #16]");                               // reload copied-byte total so far
    ctx.emitter.instruction("sub x10, x10, x9");                                // compute remaining finite bytes
    ctx.emitter.instruction("cmp x10, x1");                                     // check whether remaining bytes are below the chunk cap
    ctx.emitter.instruction("csel x1, x10, x1, lt");                            // clamp the read request to remaining bytes
    ctx.emitter.instruction(&format!("b {}", after_request));                   // finite request size is ready
    ctx.emitter.label(request_unlimited);
    ctx.emitter.label(after_request);
    abi::emit_call_label(ctx.emitter, "__rt_fread");
    ctx.emitter.instruction(&format!("cbz x2, {}", done_label));                // stop when the source returns an empty chunk
    ctx.emitter.instruction("ldr x10, [sp, #32]");                              // load requested max byte count
    emit_branch_if_unlimited_length(ctx, "x10", "x11", chunk_unlimited);
    ctx.emitter.instruction("ldr x9, [sp, #16]");                               // reload copied-byte total so far
    ctx.emitter.instruction("sub x10, x10, x9");                                // compute remaining finite bytes
    ctx.emitter.instruction("cmp x2, x10");                                     // check whether the read returned too many bytes
    ctx.emitter.instruction("csel x2, x2, x10, ls");                            // clamp wrapper chunks to remaining bytes
    ctx.emitter.instruction(&format!("b {}", after_chunk));                     // finite chunk length is ready
    ctx.emitter.label(chunk_unlimited);
    ctx.emitter.label(after_chunk);
    ctx.emitter.instruction("str x1, [sp, #24]");                               // save the owned chunk pointer for release
    ctx.emitter.instruction("ldr x9, [sp, #16]");                               // load copied-byte total so far
    ctx.emitter.instruction("add x9, x9, x2");                                  // add this chunk length to the total
    ctx.emitter.instruction("str x9, [sp, #16]");                               // store updated copied-byte total
    ctx.emitter.instruction("ldr x0, [sp, #8]");                                // reload the destination descriptor
    abi::emit_call_label(ctx.emitter, "__rt_fwrite");
    ctx.emitter.instruction("ldr x0, [sp, #24]");                               // reload the owned chunk pointer
    abi::emit_call_label(ctx.emitter, "__rt_decref_any");
    ctx.emitter.instruction(&format!("b {}", loop_label));                      // copy the next chunk
    ctx.emitter.label(done_label);
    ctx.emitter.instruction("ldr x0, [sp, #16]");                               // return the copied-byte total
    emit_stream_copy_frame_leave(ctx);
}

/// Emits the x86_64 stream-copy read/write loop.
pub(super) fn lower_stream_copy_loop_x86_64(
    ctx: &mut FunctionContext<'_>,
    loop_label: &str,
    done_label: &str,
    length_unlimited: &str,
    after_length_check: &str,
    request_unlimited: &str,
    after_request: &str,
    chunk_unlimited: &str,
    after_chunk: &str,
) {
    ctx.emitter.instruction("mov r8, QWORD PTR [rsp + 16]");                    // load copied-byte total so far
    ctx.emitter.instruction("mov r9, QWORD PTR [rsp + 32]");                    // load requested max byte count
    emit_branch_if_unlimited_length(ctx, "r9", "r10", length_unlimited);
    ctx.emitter.instruction("cmp r8, r9");                                      // test whether requested byte count is complete
    ctx.emitter.instruction(&format!("jge {}", done_label));                    // finish once length bytes have been copied
    ctx.emitter.instruction(&format!("jmp {}", after_length_check));            // continue with a finite request
    ctx.emitter.label(length_unlimited);
    ctx.emitter.label(after_length_check);
    ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");                    // reload the source descriptor
    ctx.emitter.instruction("mov rsi, 4096");                                   // request up to 4096 bytes by default
    ctx.emitter.instruction("mov r9, QWORD PTR [rsp + 32]");                    // load requested max byte count
    emit_branch_if_unlimited_length(ctx, "r9", "r10", request_unlimited);
    ctx.emitter.instruction("mov r8, QWORD PTR [rsp + 16]");                    // reload copied-byte total so far
    ctx.emitter.instruction("sub r9, r8");                                      // compute remaining finite bytes
    ctx.emitter.instruction("cmp r9, rsi");                                     // check whether remaining bytes are below the chunk cap
    ctx.emitter.instruction("cmovl rsi, r9");                                   // clamp the read request to remaining bytes
    ctx.emitter.instruction(&format!("jmp {}", after_request));                 // finite request size is ready
    ctx.emitter.label(request_unlimited);
    ctx.emitter.label(after_request);
    abi::emit_call_label(ctx.emitter, "__rt_fread");
    ctx.emitter.instruction("test rdx, rdx");                                   // check whether the source returned an empty chunk
    ctx.emitter.instruction(&format!("jz {}", done_label));                     // stop when the source returns an empty chunk
    ctx.emitter.instruction("mov r9, QWORD PTR [rsp + 32]");                    // load requested max byte count
    emit_branch_if_unlimited_length(ctx, "r9", "r10", chunk_unlimited);
    ctx.emitter.instruction("mov r8, QWORD PTR [rsp + 16]");                    // reload copied-byte total so far
    ctx.emitter.instruction("sub r9, r8");                                      // compute remaining finite bytes
    ctx.emitter.instruction("cmp rdx, r9");                                     // check whether the read returned too many bytes
    ctx.emitter.instruction("cmova rdx, r9");                                   // clamp wrapper chunks to remaining bytes
    ctx.emitter.instruction(&format!("jmp {}", after_chunk));                   // finite chunk length is ready
    ctx.emitter.label(chunk_unlimited);
    ctx.emitter.label(after_chunk);
    ctx.emitter.instruction("mov QWORD PTR [rsp + 24], rax");                   // save the owned chunk pointer for release
    ctx.emitter.instruction("mov r8, QWORD PTR [rsp + 16]");                    // load copied-byte total so far
    ctx.emitter.instruction("add r8, rdx");                                     // add this chunk length to the total
    ctx.emitter.instruction("mov QWORD PTR [rsp + 16], r8");                    // store updated copied-byte total
    ctx.emitter.instruction("mov rsi, rax");                                    // pass the chunk pointer to fwrite
    ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 8]");                    // reload the destination descriptor
    abi::emit_call_label(ctx.emitter, "__rt_fwrite");
    ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 24]");                   // reload the owned chunk pointer
    abi::emit_call_label(ctx.emitter, "__rt_decref_any");
    ctx.emitter.instruction(&format!("jmp {}", loop_label));                    // copy the next chunk
    ctx.emitter.label(done_label);
    ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 16]");                   // return the copied-byte total
    emit_stream_copy_frame_leave(ctx);
}

