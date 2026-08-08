//! Purpose:
//! Builtin, compressed, iconv, and user stream filter attachment.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;

/// Lowers `stream_filter_register(filter_name, class)` into the user-filter registry helper.
pub(crate) fn lower_stream_filter_register(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "stream_filter_register", 2)?;
    let filter_name = expect_operand(inst, 0)?;
    let class_name = expect_operand(inst, 1)?;
    load_string_to_result(ctx, filter_name, "stream_filter_register filter_name")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            load_string_to_result(ctx, class_name, "stream_filter_register class")?;
            ctx.emitter.instruction("mov x3, x2");                              // pass the class-name byte length as the fourth registry argument
            ctx.emitter.instruction("mov x2, x1");                              // pass the class-name pointer as the third registry argument
            abi::emit_pop_reg_pair(ctx.emitter, "x0", "x1");
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            load_string_to_result(ctx, class_name, "stream_filter_register class")?;
            ctx.emitter.instruction("mov rcx, rdx");                            // pass the class-name byte length as the fourth registry argument
            ctx.emitter.instruction("mov rdx, rax");                            // pass the class-name pointer as the third registry argument
            abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_stream_filter_register");
    store_if_result(ctx, inst)
}

/// Lowers `stream_filter_append` and `stream_filter_prepend`.
pub(crate) fn lower_stream_filter_attach(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
) -> Result<()> {
    ensure_arg_count_between(inst, name, 2, 4)?;
    // Ordering is the chain's job now: prepend inserts at the head instead of
    // shuffling two fixed table slots, so a third filter no longer falls off.
    let prepend = name == "stream_filter_prepend";
    let filter = expect_operand(inst, 1)?;
    if let Some(filter_name) = optional_const_string_operand(ctx, filter)? {
        if filter_name == "zlib.deflate" {
            return lower_zlib_deflate_stream_filter_attach(ctx, inst);
        }
        if filter_name == "zlib.inflate" {
            return lower_zlib_inflate_stream_filter_attach(ctx, inst);
        }
        if filter_name == "bzip2.compress" {
            return lower_bzip2_compress_stream_filter_attach(ctx, inst);
        }
        if filter_name == "bzip2.decompress" {
            return lower_bzip2_decompress_stream_filter_attach(ctx, inst);
        }
        if let Some(spec) = filter_name.strip_prefix("convert.iconv.") {
            return lower_iconv_stream_filter_attach(ctx, inst, spec);
        }
        if let Some(id) = stream_filter_id(&filter_name) {
            return lower_builtin_stream_filter_attach(ctx, inst, id, prepend);
        }
    }
    lower_user_stream_filter_attach(ctx, inst, prepend)
}

/// Lowers `stream_filter_append($stream, "zlib.deflate", ...)`.
pub(super) fn lower_zlib_deflate_stream_filter_attach(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let stream = expect_operand(inst, 0)?;
    load_stream_fd_to_result(ctx, stream, "stream_filter_append")?;
    let level = const_int_filter_param(ctx, inst, "level", true, -1, 9)?.unwrap_or(-1);
    let fwrite_label = ctx.next_label("zlib_deflate_fwrite");
    let close_label = ctx.next_label("zlib_deflate_close");
    let skip_label = ctx.next_label("zlib_deflate_skip_helpers");
    match ctx.emitter.target.arch {
        Arch::AArch64 => crate::codegen::stream_filters::zlib::emit_arm64(
            ctx.emitter,
            &fwrite_label,
            &close_label,
            &skip_label,
            level,
        ),
        Arch::X86_64 => crate::codegen::stream_filters::zlib::emit_x86_64(
            ctx.emitter,
            &fwrite_label,
            &close_label,
            &skip_label,
            level,
        ),
    }
    store_if_result(ctx, inst)
}

/// Lowers `stream_filter_append($stream, "zlib.inflate", ...)`.
pub(super) fn lower_zlib_inflate_stream_filter_attach(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let stream = expect_operand(inst, 0)?;
    load_stream_fd_to_result(ctx, stream, "stream_filter_append")?;
    emit_zlib_inflate_attach_in_place(ctx);
    store_if_result(ctx, inst)
}

/// Attaches the `zlib.inflate` read filter to the stream descriptor already held
/// in the integer result register, leaving a resource-boxed `Mixed` in that
/// register. Shared by `stream_filter_append("zlib.inflate")` and the
/// `compress.zlib://` fopen wrapper.
pub(super) fn emit_zlib_inflate_attach_in_place(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let labels = vec![
                ctx.next_label("zlib_inflate_slurp"),
                ctx.next_label("zlib_inflate_slurped"),
                ctx.next_label("zlib_inflate_zero"),
                ctx.next_label("zlib_inflate_zeroed"),
                ctx.next_label("zlib_inflate_write"),
                ctx.next_label("zlib_inflate_written"),
            ];
            let mut labels = labels.into_iter();
            crate::codegen::stream_filters::inflate::emit_arm64(ctx.emitter, |_| {
                labels.next().expect("zlib inflate ARM64 label")
            });
        }
        Arch::X86_64 => {
            let labels = vec![
                ctx.next_label("zlib_inflate_slurp"),
                ctx.next_label("zlib_inflate_slurped"),
                ctx.next_label("zlib_inflate_sized"),
                ctx.next_label("zlib_inflate_zero"),
                ctx.next_label("zlib_inflate_zeroed"),
                ctx.next_label("zlib_inflate_write"),
                ctx.next_label("zlib_inflate_written"),
            ];
            let mut labels = labels.into_iter();
            crate::codegen::stream_filters::inflate::emit_x86_64(ctx.emitter, |_| {
                labels.next().expect("zlib inflate x86_64 label")
            });
        }
    }
}

/// Lowers `stream_filter_append($stream, "bzip2.compress", ...)`.
pub(super) fn lower_bzip2_compress_stream_filter_attach(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let stream = expect_operand(inst, 0)?;
    load_stream_fd_to_result(ctx, stream, "stream_filter_append")?;
    let block_size = const_int_filter_param(ctx, inst, "blocks", true, 1, 9)?.unwrap_or(9);
    let work_factor = const_int_filter_param(ctx, inst, "work", false, 0, 250)?.unwrap_or(0);
    let fwrite_label = ctx.next_label("bz2_compress_fwrite");
    let close_label = ctx.next_label("bz2_compress_close");
    let skip_label = ctx.next_label("bz2_compress_skip_helpers");
    match ctx.emitter.target.arch {
        Arch::AArch64 => crate::codegen::stream_filters::bzip2::emit_compress_arm64(
            ctx.emitter,
            &fwrite_label,
            &close_label,
            &skip_label,
            block_size,
            work_factor,
        ),
        Arch::X86_64 => crate::codegen::stream_filters::bzip2::emit_compress_x86_64(
            ctx.emitter,
            &fwrite_label,
            &close_label,
            &skip_label,
            block_size,
            work_factor,
        ),
    }
    store_if_result(ctx, inst)
}

/// Lowers `stream_filter_append($stream, "bzip2.decompress", ...)`.
pub(super) fn lower_bzip2_decompress_stream_filter_attach(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let stream = expect_operand(inst, 0)?;
    load_stream_fd_to_result(ctx, stream, "stream_filter_append")?;
    emit_bzip2_decompress_attach_in_place(ctx);
    store_if_result(ctx, inst)
}

/// Attaches the `bzip2.decompress` read filter to the stream descriptor already
/// held in the integer result register, leaving a resource-boxed `Mixed` in that
/// register. Shared by `stream_filter_append("bzip2.decompress")` and the
/// `compress.bzip2://` fopen wrapper.
pub(super) fn emit_bzip2_decompress_attach_in_place(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            let labels = vec![
                ctx.next_label("bz2_slurp"),
                ctx.next_label("bz2_slurped"),
                ctx.next_label("bz2_write"),
                ctx.next_label("bz2_written"),
                ctx.next_label("bz2_decompress_fail"),
                ctx.next_label("bz2_done_arm"),
            ];
            let mut labels = labels.into_iter();
            crate::codegen::stream_filters::bzip2::emit_decompress_arm64(ctx.emitter, |_| {
                labels.next().expect("bzip2 decompress ARM64 label")
            });
        }
        Arch::X86_64 => {
            let labels = vec![
                ctx.next_label("bz2_slurp_x"),
                ctx.next_label("bz2_slurped_x"),
                ctx.next_label("bz2_write_x"),
                ctx.next_label("bz2_written_x"),
                ctx.next_label("bz2_decompress_fail_x"),
                ctx.next_label("bz2_done_x"),
            ];
            let mut labels = labels.into_iter();
            crate::codegen::stream_filters::bzip2::emit_decompress_x86_64(ctx.emitter, |_| {
                labels.next().expect("bzip2 decompress x86_64 label")
            });
        }
    }
}

/// Selects which read-direction decompressor a `compress.*://` fopen wrapper attaches.
#[derive(Clone, Copy)]
pub(super) enum CompressWrapper {
    Zlib,
    Bzip2,
}

/// Re-adopts a compress-wrapper descriptor into the resource registry.
///
/// The `zlib.inflate` / `bzip2.decompress` attach emitters finish by re-boxing
/// the raw descriptor with `__rt_mixed_from_value(9, fd)` so they can also serve
/// `stream_filter_append`. On the `compress.*://` wrapper path that leaves an
/// unregistered descriptor, which `fclose()` cannot resolve. Unbox it and hand
/// the descriptor to the registry adoption used by every other literal open.
///
/// Input:  x0 / rax = Mixed cell boxing the raw descriptor.
/// Output: x0 / rax = Mixed cell boxing the registry handle, or PHP false.
fn emit_adopt_attached_compress_descriptor(ctx: &mut FunctionContext<'_>) {
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // recover the raw descriptor from the Mixed payload
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, rdi");                            // recover the raw descriptor from the Mixed payload
        }
    }
    box_stream_fd_or_false_result(ctx, "compress_adopt");
}

/// Opens `underlying` read-only through `__rt_fopen` and attaches the matching
/// decompressor so subsequent reads see plain bytes, boxing the filtered
/// descriptor as a resource. An empty path, or a failed open, boxes PHP false —
/// matching PHP's `compress.zlib://` / `compress.bzip2://` wrapper behavior.
pub(super) fn emit_literal_compress_wrapper_fopen_result(
    ctx: &mut FunctionContext<'_>,
    underlying: &str,
    full_uri: &str,
    kind: CompressWrapper,
) -> Result<()> {
    if underlying.is_empty() {
        emit_fd_result(ctx, -1);
        box_stream_fd_or_false_result(ctx, "fopen");
        return Ok(());
    }
    let (path_label, path_len) = ctx.data.add_string(underlying.as_bytes());
    let (mode_label, mode_len) = ctx.data.add_string(b"r");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &path_label);
            ctx.emitter.instruction(&format!("mov x2, #{}", path_len));         // pass the underlying path byte length
            abi::emit_symbol_address(ctx.emitter, "x3", &mode_label);
            ctx.emitter.instruction(&format!("mov x4, #{}", mode_len));         // pass the read-mode string byte length
            abi::emit_call_label(ctx.emitter, "__rt_fopen");
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rax", &path_label);
            ctx.emitter.instruction(&format!("mov rdx, {}", path_len));         // pass the underlying path byte length
            abi::emit_symbol_address(ctx.emitter, "rdi", &mode_label);
            ctx.emitter.instruction(&format!("mov rsi, {}", mode_len));         // pass the read-mode string byte length
            abi::emit_call_label(ctx.emitter, "__rt_fopen");
        }
    }
    let false_label = ctx.next_label("compress_fopen_false");
    let done_label = ctx.next_label("compress_fopen_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #0");                              // negative descriptor means the underlying open failed
            ctx.emitter.instruction(&format!("b.lt {}", false_label));          // box PHP false when the source could not be opened
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // negative descriptor means the underlying open failed
            ctx.emitter.instruction(&format!("js {}", false_label));            // box PHP false when the source could not be opened
        }
    }
    // The attach helpers end by re-boxing through `__rt_mixed_from_value(9, fd)`,
    // which wraps the RAW descriptor. That predates the resource registry: the
    // stream never got adopted, so `fclose()` could not resolve the handle and
    // raised. Unbox the descriptor back out and adopt it properly, matching
    // `emit_runtime_fopen_literal_result`'s box-then-record order.
    match kind {
        CompressWrapper::Zlib => {
            emit_zlib_inflate_attach_in_place(ctx);
            emit_adopt_attached_compress_descriptor(ctx);
            emit_record_stream_meta_after_boxed_literal(ctx, 8, full_uri);
        }
        CompressWrapper::Bzip2 => {
            emit_bzip2_decompress_attach_in_place(ctx);
            emit_adopt_attached_compress_descriptor(ctx);
            emit_record_stream_meta_after_boxed_literal(ctx, 9, full_uri);
        }
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!("b {}", done_label)), // skip false boxing after attaching the decompressor
        Arch::X86_64 => ctx.emitter.instruction(&format!("jmp {}", done_label)), // skip false boxing after attaching the decompressor
    }
    ctx.emitter.label(&false_label);
    box_stream_fd_or_false_result(ctx, "fopen");
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Lowers `stream_filter_append($stream, "convert.iconv.<from>/<to>", ...)`.
pub(super) fn lower_iconv_stream_filter_attach(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    spec: &str,
) -> Result<()> {
    let stream = expect_operand(inst, 0)?;
    load_stream_fd_to_result(ctx, stream, "stream_filter_append")?;
    let Some((from, to)) = spec.split_once('/') else {
        emit_boxed_stream_resource(ctx);
        return store_if_result(ctx, inst);
    };
    if from.is_empty() || to.is_empty() {
        emit_boxed_stream_resource(ctx);
        return store_if_result(ctx, inst);
    }
    let from_cstr = format!("{}\0", from);
    let to_cstr = format!("{}\0", to);
    let (from_sym, _) = ctx.data.add_string(from_cstr.as_bytes());
    let (to_sym, _) = ctx.data.add_string(to_cstr.as_bytes());
    let write_label = ctx.next_label("iconv_mode_write");
    let after_label = ctx.next_label("iconv_mode_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp, #-16]!");                     // preserve the descriptor across mode evaluation
            materialize_stream_filter_mode(ctx, inst)?;
            ctx.emitter.instruction("mov x9, x0");                              // hold the selected stream-filter mode
            ctx.emitter.instruction("ldr x0, [sp], #16");                       // restore the stream descriptor
            ctx.emitter.instruction("cmp x9, #2");                              // test for STREAM_FILTER_WRITE-only mode
            ctx.emitter.instruction(&format!("b.eq {}", write_label));          // install the streaming write transcoder
            emit_iconv_read_transform_for_current_fd(ctx, &from_sym, &to_sym);
            ctx.emitter.instruction(&format!("b {}", after_label));             // skip the write-filter attach path
            ctx.emitter.label(&write_label);
            emit_iconv_write_transform_for_current_fd(ctx, &from_sym, &to_sym);
            ctx.emitter.label(&after_label);
        }
        Arch::X86_64 => {
            abi::emit_push_reg(ctx.emitter, "rax");
            materialize_stream_filter_mode(ctx, inst)?;
            ctx.emitter.instruction("mov r9, rax");                             // hold the selected stream-filter mode
            abi::emit_pop_reg(ctx.emitter, "rax");
            ctx.emitter.instruction("cmp r9, 2");                               // test for STREAM_FILTER_WRITE-only mode
            ctx.emitter.instruction(&format!("je {}", write_label));            // install the streaming write transcoder
            emit_iconv_read_transform_for_current_fd(ctx, &from_sym, &to_sym);
            ctx.emitter.instruction(&format!("jmp {}", after_label));           // skip the write-filter attach path
            ctx.emitter.label(&write_label);
            emit_iconv_write_transform_for_current_fd(ctx, &from_sym, &to_sym);
            ctx.emitter.label(&after_label);
        }
    }
    store_if_result(ctx, inst)
}

/// Emits the attach-time READ transform for the current iconv stream descriptor.
pub(super) fn emit_iconv_read_transform_for_current_fd(
    ctx: &mut FunctionContext<'_>,
    from_sym: &str,
    to_sym: &str,
) {
    let labels = vec![
        ctx.next_label("iconv_slurp"),
        ctx.next_label("iconv_slurped"),
        ctx.next_label("iconv_sized"),
        ctx.next_label("iconv_skip"),
        ctx.next_label("iconv_write"),
        ctx.next_label("iconv_written"),
    ];
    let mut labels = labels.into_iter();
    match ctx.emitter.target.arch {
        Arch::AArch64 => crate::codegen::stream_filters::iconv::emit_read_arm64(
            ctx.emitter,
            from_sym,
            to_sym,
            |_| labels.next().expect("iconv read transform label"),
        ),
        Arch::X86_64 => crate::codegen::stream_filters::iconv::emit_read_x86_64(
            ctx.emitter,
            from_sym,
            to_sym,
            |_| labels.next().expect("iconv read transform label"),
        ),
    }
}

/// Emits the WRITE transform attachment for the current iconv stream descriptor.
pub(super) fn emit_iconv_write_transform_for_current_fd(
    ctx: &mut FunctionContext<'_>,
    from_sym: &str,
    to_sym: &str,
) {
    let labels = vec![
        ctx.next_label("iconv_w_fwrite"),
        ctx.next_label("iconv_w_close"),
        ctx.next_label("iconv_w_skip_helpers"),
        ctx.next_label("iconv_w_loop"),
        ctx.next_label("iconv_w_after_write"),
        ctx.next_label("iconv_w_done"),
        ctx.next_label("iconv_w_skip_store"),
    ];
    let mut labels = labels.into_iter();
    crate::codegen::stream_filters::iconv_write::emit_iconv_write_attach_with_labels(
        ctx.emitter,
        from_sym,
        to_sym,
        |_| labels.next().expect("iconv write transform label"),
    );
}

/// Boxes an opaque filter handle as a PHP resource value.
///
/// The payload is the registry handle, so `stream_filter_remove()` can resolve
/// the node again and `is_resource()` observes the registry lifetime.
pub(super) fn emit_boxed_filter_handle(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, x0");                              // resource payload = the filter handle
            // The high word is the registry-ownership marker, not padding: only the
            // marked values make is_resource()/get_resource_type() consult the
            // registry. Leaving it zero routed them down the legacy branch, which
            // answers "stream" and "open" unconditionally.
            ctx.emitter.instruction("mov x2, #1");                              // registry-owned resource marker
            ctx.emitter.instruction("mov x0, #9");                              // runtime tag 9 = resource
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // resource payload = the filter handle
            // The high word is the registry-ownership marker, not padding: only the
            // marked values make is_resource()/get_resource_type() consult the
            // registry. Leaving it zero routed them down the legacy branch, which
            // answers "stream" and "open" unconditionally.
            ctx.emitter.instruction("mov esi, 1");                              // registry-owned resource marker
            ctx.emitter.instruction("mov eax, 9");                              // runtime tag 9 = resource
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
}

/// Lowers `stream_filter_remove(resource)`.
pub(crate) fn lower_stream_filter_remove(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    super::super::ensure_arg_count(inst, "stream_filter_remove", 1)?;
    let filter = expect_operand(inst, 0)?;
    // A chain node resolves as a filter resource. Anything else is still a legacy
    // per-descriptor filter, so the previous teardown stays reachable until the
    // remaining families move over.
    let legacy = ctx.next_label("sfr_legacy");
    let done = ctx.next_label("sfr_done");
    load_stream_handle_to_result(ctx, filter, "stream_filter_remove")?;
    abi::emit_reserve_temporary_stack(ctx.emitter, 16);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp, #0]");                        // preserve the candidate handle
            abi::emit_call_label(ctx.emitter, "__rt_filter_state");
            ctx.emitter.instruction(&format!("cbz x0, {}", legacy));            // not a chain filter: use the legacy teardown
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the filter handle
            ctx.emitter.instruction(&format!("mov x1, #{STREAM_READ_FILTER_HEAD_OFFSET}"));
            abi::emit_call_label(ctx.emitter, "__rt_stream_filter_unlink");     // detach from the read chain
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the filter handle
            ctx.emitter.instruction(&format!("mov x1, #{STREAM_WRITE_FILTER_HEAD_OFFSET}"));
            abi::emit_call_label(ctx.emitter, "__rt_stream_filter_unlink");     // detach from the write chain
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the filter handle
            // PHP closes a removed filter, so `onClose()` fires here rather than only
            // when the stream goes: the node is off both chains and the teardown that
            // sweeps them will never see it again.
            abi::emit_call_label(ctx.emitter, "__rt_filter_node_close_obj");    // onClose(), exactly once
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the filter handle
            abi::emit_call_label(ctx.emitter, "__rt_resource_mark_closed");     // publish Closed so is_resource() reports false
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the filter handle
            abi::emit_call_label(ctx.emitter, "__rt_resource_release");         // drop the reference stream_filter_append() handed out
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            ctx.emitter.instruction("mov x0, #1");                              // stream_filter_remove() reports success
            abi::emit_jump(ctx.emitter, &done);
            ctx.emitter.label(&legacy);
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // reload the candidate for the legacy path
            abi::emit_release_temporary_stack(ctx.emitter, 16);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov QWORD PTR [rsp + 0], rax");            // preserve the candidate handle
            ctx.emitter.instruction("mov rdi, rax");                            // pass it to the filter-state probe
            abi::emit_call_label(ctx.emitter, "__rt_filter_state");
            ctx.emitter.instruction("test rax, rax");
            ctx.emitter.instruction(&format!("jz {}", legacy));                 // not a chain filter: use the legacy teardown
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // reload the filter handle
            ctx.emitter.instruction(&format!("mov rsi, {STREAM_READ_FILTER_HEAD_OFFSET}"));
            abi::emit_call_label(ctx.emitter, "__rt_stream_filter_unlink");     // detach from the read chain
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // reload the filter handle
            ctx.emitter.instruction(&format!("mov rsi, {STREAM_WRITE_FILTER_HEAD_OFFSET}"));
            abi::emit_call_label(ctx.emitter, "__rt_stream_filter_unlink");     // detach from the write chain
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // reload the filter handle
            // See the AArch64 counterpart: a removed node is off both chains, so the
            // chain teardown can no longer fire its `onClose()`.
            abi::emit_call_label(ctx.emitter, "__rt_filter_node_close_obj");    // onClose(), exactly once
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // reload the filter handle
            abi::emit_call_label(ctx.emitter, "__rt_resource_mark_closed");     // publish Closed so is_resource() reports false
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // reload the filter handle
            abi::emit_call_label(ctx.emitter, "__rt_resource_release");         // drop the reference stream_filter_append() handed out
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            ctx.emitter.instruction("mov eax, 1");                              // stream_filter_remove() reports success
            abi::emit_jump(ctx.emitter, &done);
            ctx.emitter.label(&legacy);
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 0]");            // reload the candidate for the legacy path
            abi::emit_release_temporary_stack(ctx.emitter, 16);
        }
    }
    load_stream_fd_to_result(ctx, filter, "stream_filter_remove")?;
    if matches!(ctx.emitter.target.arch, Arch::X86_64) {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the descriptor to the user-filter teardown helper
    }
    abi::emit_call_label(ctx.emitter, "__rt_user_filter_release_fd");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_read_filters");
            ctx.emitter.instruction("strb wzr, [x9, x0]");                      // clear the read-direction slot 0
            ctx.emitter.instruction("add x10, x0, #256");                       // fd+256 (slot 1)
            ctx.emitter.instruction("strb wzr, [x9, x10]");                     // clear the read-direction slot 1
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_write_filters");
            ctx.emitter.instruction("strb wzr, [x9, x0]");                      // clear the write-direction slot 0
            ctx.emitter.instruction("strb wzr, [x9, x10]");                     // clear the write-direction slot 1
            ctx.emitter.instruction("mov x0, #1");                              // return true after removing the filter state
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_read_filters"); // read-filter table base
            ctx.emitter.instruction("mov BYTE PTR [r9 + rax], 0");              // clear the read-direction slot 0
            ctx.emitter.instruction("lea r10, [rax + 256]");                    // fd+256 (slot 1)
            ctx.emitter.instruction("mov BYTE PTR [r9 + r10], 0");              // clear the read-direction slot 1
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_write_filters"); // write-filter table base
            ctx.emitter.instruction("mov BYTE PTR [r9 + rax], 0");              // clear the write-direction slot 0
            ctx.emitter.instruction("mov BYTE PTR [r9 + r10], 0");              // clear the write-direction slot 1
            ctx.emitter.instruction("mov eax, 1");                              // return true after removing the filter state
        }
    }
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)
}

