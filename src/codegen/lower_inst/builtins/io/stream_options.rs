//! Purpose:
//! Stream predicates, options, select, and include-path resolution.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;

/// Lowers `stream_isatty(stream)`.
pub(crate) fn lower_stream_isatty(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "stream_isatty", 1)?;
    let stream = expect_operand(inst, 0)?;
    load_stream_fd_to_result(ctx, stream, "stream_isatty")?;
    if ctx.emitter.target.arch == Arch::X86_64 {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the descriptor to the runtime terminal probe
    }
    abi::emit_call_label(ctx.emitter, "__rt_stream_isatty");
    store_if_result(ctx, inst)
}

/// Lowers `stream_set_blocking(stream, enable)`.
pub(crate) fn lower_stream_set_blocking(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "stream_set_blocking", 2)?;
    let stream = expect_operand(inst, 0)?;
    let enable = expect_operand(inst, 1)?;
    load_stream_fd_to_result(ctx, stream, "stream_set_blocking")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    require_int_or_bool(
        ctx.load_value_to_result(enable)?.codegen_repr(),
        "stream_set_blocking enable",
    )?;
    let wrapper = ctx.next_label("set_blocking_wrapper");
    let after = ctx.next_label("set_blocking_after");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, x0");                              // pass the blocking flag as the native helper's second argument
            abi::emit_pop_reg(ctx.emitter, "x0");
            ctx.emitter.instruction("mov w9, #0x4000");                         // materialize the high half of USER_WRAPPER_FD_BASE
            ctx.emitter.instruction("lsl w9, w9, #16");                         // form the synthetic wrapper fd base 0x40000000
            ctx.emitter.instruction("cmp x0, x9");                              // test whether the handle is a synthetic wrapper fd
            ctx.emitter.instruction(&format!("b.ge {}", wrapper));              // dispatch synthetic handles to stream_set_option
            abi::emit_call_label(ctx.emitter, "__rt_stream_set_blocking");
            ctx.emitter.instruction(&format!("b {}", after));                   // skip wrapper dispatch after the native fd update
            ctx.emitter.label(&wrapper);
            ctx.emitter.instruction("mov x2, x1");                              // pass the blocking flag as wrapper option arg1
            ctx.emitter.instruction(&format!("mov x1, #{}", STREAM_OPTION_BLOCKING)); // select STREAM_OPTION_BLOCKING
            ctx.emitter.instruction("mov x3, #0");                              // pass zero as wrapper option arg2
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_set_option");
            ctx.emitter.label(&after);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, rax");                            // pass the blocking flag as the native helper's second argument
            abi::emit_pop_reg(ctx.emitter, "rdi");
            ctx.emitter.instruction("mov r9d, 0x40000000");                     // materialize USER_WRAPPER_FD_BASE for synthetic handles
            ctx.emitter.instruction("cmp rdi, r9");                             // test whether the handle is a synthetic wrapper fd
            ctx.emitter.instruction(&format!("jge {}", wrapper));               // dispatch synthetic handles to stream_set_option
            abi::emit_call_label(ctx.emitter, "__rt_stream_set_blocking");
            ctx.emitter.instruction(&format!("jmp {}", after));                 // skip wrapper dispatch after the native fd update
            ctx.emitter.label(&wrapper);
            ctx.emitter.instruction("mov rdx, rsi");                            // pass the blocking flag as wrapper option arg1
            ctx.emitter.instruction(&format!("mov rsi, {}", STREAM_OPTION_BLOCKING)); // select STREAM_OPTION_BLOCKING
            ctx.emitter.instruction("xor ecx, ecx");                            // pass zero as wrapper option arg2
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_set_option");
            ctx.emitter.label(&after);
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers `stream_set_chunk_size(stream, size)` and returns the previous size.
pub(crate) fn lower_stream_set_chunk_size(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "stream_set_chunk_size", 2)?;
    let stream = expect_operand(inst, 0)?;
    let size = expect_operand(inst, 1)?;
    load_stream_handle_to_result(ctx, stream, "stream_set_chunk_size")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    require_int(
        ctx.load_value_to_result(size)?.codegen_repr(),
        "stream_set_chunk_size size",
    )?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, x0");                              // pass the requested chunk size as the second runtime argument
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, rax");                            // pass the requested chunk size as the second runtime argument
            abi::emit_pop_reg(ctx.emitter, "rdi");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_stream_set_chunk_size");
    let open_label = ctx.next_label("stream_chunk_open");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbnz x1, {}", open_label));       // continue only when the opaque handle resolved to a live stream
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rdx, rdx");                           // did the opaque handle resolve to a live stream?
            ctx.emitter.instruction(&format!("jnz {}", open_label));            // continue only after successful state replacement
        }
    }
    emit_closed_stream_type_error(ctx, "stream_set_chunk_size");
    ctx.emitter.label(&open_label);
    store_if_result(ctx, inst)
}

/// Lowers `stream_set_read_buffer(stream, size)`.
pub(crate) fn lower_stream_set_read_buffer(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    lower_stream_set_buffer(
        ctx,
        inst,
        "stream_set_read_buffer",
        STREAM_OPTION_READ_BUFFER,
        NATIVE_READ_BUFFER_RESULT,
    )
}

/// Lowers `stream_set_write_buffer(stream, size)`.
pub(crate) fn lower_stream_set_write_buffer(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    lower_stream_set_buffer(
        ctx,
        inst,
        "stream_set_write_buffer",
        STREAM_OPTION_WRITE_BUFFER,
        NATIVE_WRITE_BUFFER_RESULT,
    )
}

/// Lowers a stream read/write buffer setter, dispatching wrappers to `stream_set_option()`.
///
/// Both setters used to be a no-op returning 0, so a userspace wrapper never learned the buffer
/// changed and every stream reported success. Measured on php 8.5.6:
///
/// - the wrapper's `stream_set_option($option, $arg1, $arg2)` is called with `$option` 3 for write
///   and 2 for read; `$arg1` is `PHP_STREAM_BUFFER_NONE` (0) for size 0 and `_FULL` (2) otherwise;
///   `$arg2` is the size, or the default chunk size 1024 when the size is 0;
/// - the call answers 0 when the hook returns true and -1 otherwise — including when the hook is
///   absent, which additionally warns;
/// - a stream that is NOT a wrapper answers -1 for the write buffer and 0 for the read buffer,
///   alike for a real file and for `php://memory`, which is what `native_result` carries.
fn lower_stream_set_buffer(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
    option: usize,
    native_result: i64,
) -> Result<()> {
    ensure_arg_count_between(inst, name, 2, 2)?;
    let stream = expect_operand(inst, 0)?;
    let size = expect_operand(inst, 1)?;
    load_stream_fd_to_result(ctx, stream, name)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    require_int(ctx.load_value_to_result(size)?.codegen_repr(), name)?;
    let wrapper = ctx.next_label("set_buffer_wrapper");
    let after = ctx.next_label("set_buffer_after");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, x0");                              // hold the requested size while the handle is restored
            abi::emit_pop_reg(ctx.emitter, "x0");
            ctx.emitter.instruction("mov w9, #0x4000");                         // materialize the high half of USER_WRAPPER_FD_BASE
            ctx.emitter.instruction("lsl w9, w9, #16");                         // form the synthetic wrapper fd base 0x40000000
            ctx.emitter.instruction("cmp x0, x9");                              // test whether the handle is a synthetic wrapper fd
            ctx.emitter.instruction(&format!("b.ge {}", wrapper));              // only wrappers see stream_set_option
            ctx.emitter.instruction(&format!("mov x0, #{}", native_result));    // php's answer for a non-wrapper stream
            ctx.emitter.instruction(&format!("b {}", after));
            ctx.emitter.label(&wrapper);
            ctx.emitter.instruction("cmp x1, #0");                              // a zero size means "unbuffered"
            ctx.emitter.instruction(&format!("mov x2, #{}", STREAM_BUFFER_FULL)); // buffered is the default mode
            ctx.emitter.instruction("mov x3, x1");                              // and the requested size travels as arg2
            ctx.emitter.instruction(&format!("mov x9, #{}", STREAM_BUFFER_NONE));
            ctx.emitter.instruction(&format!("mov x10, #{}", DEFAULT_CHUNK_SIZE));
            ctx.emitter.instruction("csel x2, x9, x2, eq");                     // size 0 → PHP_STREAM_BUFFER_NONE
            ctx.emitter.instruction("csel x3, x10, x3, eq");                    // size 0 → php sends the default chunk size
            ctx.emitter.instruction(&format!("mov x1, #{}", option));           // select the read/write buffer option
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_set_option");
            ctx.emitter.instruction("cmp x0, #0");                              // the hook's bool result decides the answer
            ctx.emitter.instruction("mov x9, #-1");
            ctx.emitter.instruction("mov x10, #0");
            ctx.emitter.instruction("csel x0, x10, x9, ne");                    // true → 0, false or missing hook → -1
            ctx.emitter.label(&after);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, rax");                            // hold the requested size while the handle is restored
            abi::emit_pop_reg(ctx.emitter, "rdi");
            ctx.emitter.instruction("mov r9d, 0x40000000");                     // materialize USER_WRAPPER_FD_BASE for synthetic handles
            ctx.emitter.instruction("cmp rdi, r9");                             // test whether the handle is a synthetic wrapper fd
            ctx.emitter.instruction(&format!("jge {}", wrapper));               // only wrappers see stream_set_option
            ctx.emitter.instruction(&format!("mov rax, {}", native_result));    // php's answer for a non-wrapper stream
            ctx.emitter.instruction(&format!("jmp {}", after));
            ctx.emitter.label(&wrapper);
            ctx.emitter.instruction(&format!("mov rdx, {}", STREAM_BUFFER_FULL)); // buffered is the default mode
            ctx.emitter.instruction("mov rcx, rsi");                            // and the requested size travels as arg2
            ctx.emitter.instruction("test rsi, rsi");                           // a zero size means "unbuffered"
            ctx.emitter.instruction(&format!("mov r10, {}", STREAM_BUFFER_NONE));
            ctx.emitter.instruction(&format!("mov r11, {}", DEFAULT_CHUNK_SIZE));
            ctx.emitter.instruction("cmove rdx, r10");                          // size 0 → PHP_STREAM_BUFFER_NONE
            ctx.emitter.instruction("cmove rcx, r11");                          // size 0 → php sends the default chunk size
            ctx.emitter.instruction(&format!("mov rsi, {}", option));           // select the read/write buffer option
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_set_option");
            ctx.emitter.instruction("test rax, rax");                           // the hook's bool result decides the answer
            ctx.emitter.instruction("mov rax, 0");
            ctx.emitter.instruction("mov r9, -1");
            ctx.emitter.instruction("cmove rax, r9");                           // true → 0, false or missing hook → -1
            ctx.emitter.label(&after);
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers `stream_set_timeout(stream, seconds, microseconds?)`.
pub(crate) fn lower_stream_set_timeout(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    ensure_arg_count_between(inst, "stream_set_timeout", 2, 3)?;
    let stream = expect_operand(inst, 0)?;
    let seconds = expect_operand(inst, 1)?;
    load_stream_fd_to_result(ctx, stream, "stream_set_timeout")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    require_int(
        ctx.load_value_to_result(seconds)?.codegen_repr(),
        "stream_set_timeout seconds",
    )?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            if inst.operands.len() == 3 {
                let usec = expect_operand(inst, 2)?;
                require_int(
                    ctx.load_value_to_result(usec)?.codegen_repr(),
                    "stream_set_timeout microseconds",
                )?;
                ctx.emitter.instruction("mov x2, x0");                          // pass explicit microseconds as the third runtime argument
            } else {
                ctx.emitter.instruction("mov x2, #0");                          // default omitted microseconds to zero
            }
            abi::emit_pop_reg(ctx.emitter, "x1");
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            if inst.operands.len() == 3 {
                let usec = expect_operand(inst, 2)?;
                require_int(
                    ctx.load_value_to_result(usec)?.codegen_repr(),
                    "stream_set_timeout microseconds",
                )?;
                ctx.emitter.instruction("mov rdx, rax");                        // pass explicit microseconds as the third runtime argument
            } else {
                ctx.emitter.instruction("xor edx, edx");                        // default omitted microseconds to zero
            }
            abi::emit_pop_reg(ctx.emitter, "rsi");
            abi::emit_pop_reg(ctx.emitter, "rdi");
        }
    }
    lower_stream_timeout_dispatch(ctx);
    store_if_result(ctx, inst)
}

/// Lowers `stream_select(read, write, except, seconds, microseconds?)`.
pub(crate) fn lower_stream_select(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    ensure_arg_count_between(inst, "stream_select", 4, 5)?;
    let result_reg = abi::int_result_reg(ctx.emitter);
    for idx in 0..4 {
        let value = expect_operand(inst, idx)?;
        ctx.load_value_to_result(value)?;
        abi::emit_push_reg(ctx.emitter, result_reg);
    }
    if inst.operands.len() == 5 {
        let microseconds = expect_operand(inst, 4)?;
        ctx.load_value_to_result(microseconds)?;
    } else {
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction("mov x0, #0");                          // default omitted microseconds to zero
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("xor eax, eax");                        // default omitted microseconds to zero
            }
        }
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x4, x0");                              // pass microseconds as the fifth runtime argument
            abi::emit_pop_reg(ctx.emitter, "x3");
            abi::emit_pop_reg(ctx.emitter, "x2");
            abi::emit_pop_reg(ctx.emitter, "x1");
            abi::emit_pop_reg(ctx.emitter, "x0");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov r8, rax");                             // pass microseconds as the fifth runtime argument
            abi::emit_pop_reg(ctx.emitter, "rcx");
            abi::emit_pop_reg(ctx.emitter, "rdx");
            abi::emit_pop_reg(ctx.emitter, "rsi");
            abi::emit_pop_reg(ctx.emitter, "rdi");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_stream_select");
    store_if_result(ctx, inst)
}

/// Lowers `stream_resolve_include_path(filename)` as realpath-backed `string|false`.
pub(crate) fn lower_stream_resolve_include_path(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "stream_resolve_include_path", 1)?;
    let filename = expect_operand(inst, 0)?;
    load_string_to_result(ctx, filename, "stream_resolve_include_path")?;
    abi::emit_call_label(ctx.emitter, "__rt_realpath");
    box_owned_string_or_false_result(ctx, "stream_resolve_include_path");
    store_if_result(ctx, inst)
}

