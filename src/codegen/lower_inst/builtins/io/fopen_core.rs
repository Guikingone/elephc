//! Purpose:
//! Core fopen dispatch and php filter URL parsing.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;

/// Lowers `fopen(filename, mode)` and boxes stream resources or PHP false.
pub(crate) fn lower_fopen(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "fopen", 2, 4)?;
    let filename = expect_operand(inst, 0)?;
    let mode = expect_operand(inst, 1)?;
    let filename_literal = optional_const_string_operand(ctx, filename)?;
    begin_fopen_context_scope(ctx, inst.operands.get(3).copied())?;
    if let Some(path) = filename_literal.as_deref() {
        let open_mode = LiteralOpenMode::Operand(mode);
        if path.starts_with("php://filter/") {
            emit_literal_php_filter_fopen_result(ctx, open_mode, path)?;
        } else if let Some(underlying) = path.strip_prefix("compress.zlib://") {
            emit_literal_compress_wrapper_fopen_result(
                ctx,
                underlying,
                path,
                CompressWrapper::Zlib,
            )?;
        } else if let Some(underlying) = path.strip_prefix("compress.bzip2://") {
            emit_literal_compress_wrapper_fopen_result(
                ctx,
                underlying,
                path,
                CompressWrapper::Bzip2,
            )?;
        } else {
            emit_literal_fopen_result(ctx, open_mode, path)?;
        }
        emit_record_stream_mode_after_boxed(ctx, mode)?;
        finish_fopen_context_scope(ctx);
        store_if_result(ctx, inst)?;
        if path.starts_with("http://") {
            publish_http_response_headers(ctx);
        }
        return Ok(());
    }
    publish_dynamic_phar_function_pointers(ctx);
    publish_dynamic_phar_write_function_pointer(ctx);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_to_result(ctx, filename, "fopen filename")?;
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            load_string_to_result(ctx, mode, "fopen mode")?;
            ctx.emitter.instruction("mov x3, x1");                              // pass the mode pointer in the runtime helper's secondary string slot
            ctx.emitter.instruction("mov x4, x2");                              // pass the mode length in the runtime helper's secondary string slot
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        }
        Arch::X86_64 => {
            load_string_to_result(ctx, filename, "fopen filename")?;
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            load_string_to_result(ctx, mode, "fopen mode")?;
            ctx.emitter.instruction("mov rdi, rax");                            // pass the mode pointer while the filename remains on the stack
            ctx.emitter.instruction("mov rsi, rdx");                            // pass the mode length while the filename remains on the stack
            abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
        }
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => abi::emit_push_reg_pair(ctx.emitter, "x1", "x2"),
        Arch::X86_64 => abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx"),
    }
    emit_dynamic_fopen_result(ctx, inst)
}

/// Replaces a run-time `php://filter/...` filename with the RESOURCE it wraps.
///
/// A filter URL is "open this, then filter it", so the open that follows is the ordinary one for
/// whatever the resource turns out to be — a file, `php://temp`, anything. The filter it named is
/// parked by the parse and attached after boxing, which is why nothing here needs to know how to
/// open the resource itself.
pub(super) fn emit_dynamic_php_filter_swap(ctx: &mut FunctionContext<'_>) {
    let unchanged = ctx.next_label("fopen_dynamic_not_filter");
    let no_resource = ctx.next_label("fopen_dynamic_filter_no_resource");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            // Both pairs must be saved, not just the mode: the parse takes its argument in x0/x1
            // and so destroys the filename pair the fall-through path still needs.
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");                   // the filename
            abi::emit_push_reg_pair(ctx.emitter, "x3", "x4");                   // the fopen mode
            ctx.emitter.instruction("mov x0, x1");                              // the candidate filter URL
            ctx.emitter.instruction("mov x1, x2");                              // and its length
            abi::emit_call_label(ctx.emitter, "__rt_php_filter_parse");
            ctx.emitter.instruction("mov x9, x0");                              // did it parse as a filter URL?
            abi::emit_pop_reg_pair(ctx.emitter, "x3", "x4");
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
            ctx.emitter.instruction("cmp x9, #2");                              // a filter URL that names no resource?
            ctx.emitter.instruction(&format!("b.eq {}", no_resource));          // php throws for it
            ctx.emitter.instruction(&format!("cbz x9, {}", unchanged));         // no: the filename stands
            abi::emit_symbol_address(ctx.emitter, "x9", "_php_filter_res_ptr");
            ctx.emitter.instruction("ldr x1, [x9]");                            // open the resource instead
            abi::emit_symbol_address(ctx.emitter, "x9", "_php_filter_res_len");
            ctx.emitter.instruction("ldr x2, [x9]");                            // with its length
        }
        Arch::X86_64 => {
            // See the AArch64 counterpart: the parse takes rdi/rsi, so the filename pair in
            // rax/rdx has to be saved as well as the mode.
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");                 // the filename
            abi::emit_push_reg_pair(ctx.emitter, "rdi", "rsi");                 // the fopen mode
            ctx.emitter.instruction("mov rdi, rax");                            // the candidate filter URL
            ctx.emitter.instruction("mov rsi, rdx");                            // and its length
            abi::emit_call_label(ctx.emitter, "__rt_php_filter_parse");
            ctx.emitter.instruction("mov r9, rax");                             // did it parse as a filter URL?
            abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
            abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
            ctx.emitter.instruction("cmp r9, 2");                               // a filter URL that names no resource?
            ctx.emitter.instruction(&format!("je {}", no_resource));            // php throws for it
            ctx.emitter.instruction("test r9, r9");
            ctx.emitter.instruction(&format!("jz {}", unchanged));              // no: the filename stands
            abi::emit_symbol_address(ctx.emitter, "r9", "_php_filter_res_ptr");
            ctx.emitter.instruction("mov rax, QWORD PTR [r9]");                 // open the resource instead
            abi::emit_symbol_address(ctx.emitter, "r9", "_php_filter_res_len");
            ctx.emitter.instruction("mov rdx, QWORD PTR [r9]");                 // with its length
        }
    }
    let past_throw = ctx.next_label("fopen_dynamic_filter_resourced");
    abi::emit_jump(ctx.emitter, &past_throw);
    ctx.emitter.label(&no_resource);
    // php's wording and class, and `@` does not soften it — the throw ignores the diagnostic
    // suppression depth, exactly as php's Error does.
    crate::codegen::lower_inst::exceptions::emit_error(ctx, "No URL resource specified");
    ctx.emitter.label(&past_throw);
    ctx.emitter.label(&unchanged);
}

/// Attaches the filter a run-time `php://filter` URL named, once the resource is open and boxed.
///
/// A no-op when nothing is pending, which is every open that did not come from a filter URL.
fn emit_dynamic_php_filter_attach(ctx: &mut FunctionContext<'_>) {
    abi::emit_call_label(ctx.emitter, "__rt_php_filter_attach_pending");
}

/// Opens a runtime filename that carries the `data://` prefix, falling through when it does not.
///
/// A literal `data://` URI is decoded during lowering and its bytes embedded, which left a
/// run-time URI with no path at all. `__rt_data_stream_dynamic` decodes from the bytes instead,
/// through the same base64 and percent decoders the rest of the runtime uses.
fn emit_dynamic_data_branch(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    done: &str,
) -> Result<()> {
    let not_data = ctx.next_label("fopen_dynamic_not_data");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x2, #8");                              // "data://" plus at least a comma
            ctx.emitter.instruction(&format!("b.lt {}", not_data));             // too short to carry the scheme
            for (offset, byte) in b"data://".iter().enumerate() {
                ctx.emitter.instruction(&format!("ldrb w9, [x1, #{}]", offset)); // load one candidate scheme byte
                ctx.emitter.instruction(&format!("cmp w9, #{}", byte));         // compare against the canonical data:// byte
                ctx.emitter.instruction(&format!("b.ne {}", not_data));         // a different prefix is not this wrapper
            }
            ctx.emitter.instruction("mov x0, x1");                              // pass the URI pointer
            ctx.emitter.instruction("mov x1, x2");                              // pass the URI length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rdx, 8");                              // "data://" plus at least a comma
            ctx.emitter.instruction(&format!("jl {}", not_data));               // too short to carry the scheme
            for (offset, byte) in b"data://".iter().enumerate() {
                ctx.emitter.instruction(&format!(
                    "cmp BYTE PTR [rax + {}], {}", offset, byte
                ));                                                             // compare one byte against the canonical data:// prefix
                ctx.emitter.instruction(&format!("jne {}", not_data));          // a different prefix is not this wrapper
            }
            ctx.emitter.instruction("mov rdi, rax");                            // pass the URI pointer
            ctx.emitter.instruction("mov rsi, rdx");                            // pass the URI length
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_data_stream_dynamic");
    box_stream_fd_or_false_result(ctx, "fopen_data_dynamic");
    emit_dynamic_php_filter_attach(ctx);                                        // a php://filter URL may wrap a data:// resource
    emit_record_stream_meta_after_boxed_stashed(ctx, 2);
    emit_record_stream_mode_after_boxed(ctx, expect_operand(inst, 1)?)?;
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    finish_fopen_context_scope(ctx);
    store_if_result(ctx, inst)?;
    abi::emit_jump(ctx.emitter, done);
    ctx.emitter.label(&not_data);
    Ok(())
}

/// Opens a runtime filename that carries the `php://` prefix, falling through when it does not.
///
/// The literal path resolves its wrapper at compile time; a runtime path had no such dispatch and
/// went to the file opener, so `fopen($path, 'r')` with `php://memory` searched for a FILE of
/// that name. This branch gives the runtime bytes the same treatment, through
/// `__rt_php_wrapper_open`, and leaves everything else — including a `php://` URL the helper does
/// not recognise, which answers `-1` and boxes as `false` — to the paths below.
fn emit_dynamic_php_wrapper_branch(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    done: &str,
) -> Result<()> {
    let not_php = ctx.next_label("fopen_dynamic_not_php");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x2, #6");                              // is the filename long enough for php://?
            ctx.emitter.instruction(&format!("b.lt {}", not_php));              // shorter filenames cannot carry the scheme
            for (offset, byte) in b"php://".iter().enumerate() {
                ctx.emitter.instruction(&format!("ldrb w9, [x1, #{}]", offset)); // load one candidate scheme byte
                ctx.emitter.instruction(&format!("cmp w9, #{}", byte));         // compare against the canonical php:// byte
                ctx.emitter.instruction(&format!("b.ne {}", not_php));          // a different prefix is not this wrapper
            }
            ctx.emitter.instruction("mov x0, x1");                              // pass the filename pointer
            ctx.emitter.instruction("mov x1, x2");                              // pass the filename length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rdx, 6");                              // is the filename long enough for php://?
            ctx.emitter.instruction(&format!("jl {}", not_php));                // shorter filenames cannot carry the scheme
            for (offset, byte) in b"php://".iter().enumerate() {
                ctx.emitter.instruction(&format!(
                    "cmp BYTE PTR [rax + {}], {}", offset, byte
                ));                                                             // compare one byte against the canonical php:// prefix
                ctx.emitter.instruction(&format!("jne {}", not_php));           // a different prefix is not this wrapper
            }
            ctx.emitter.instruction("mov rdi, rax");                            // pass the filename pointer
            ctx.emitter.instruction("mov rsi, rdx");                            // pass the filename length
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_php_wrapper_open");
    box_stream_fd_or_false_result(ctx, "fopen_php_dynamic");
    emit_dynamic_php_filter_attach(ctx);                                        // a php://filter URL parked its filter; attach it now the stream exists
    emit_record_stream_meta_after_boxed_stashed(ctx, 6);
    emit_record_stream_mode_after_boxed(ctx, expect_operand(inst, 1)?)?;
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    finish_fopen_context_scope(ctx);
    store_if_result(ctx, inst)?;
    abi::emit_jump(ctx.emitter, done);
    ctx.emitter.label(&not_php);
    Ok(())
}

/// Dispatches a runtime filename to the streaming HTTP opener or generic fopen helper.
fn emit_dynamic_fopen_result(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let plain = ctx.next_label("fopen_dynamic_plain");
    let done = ctx.next_label("fopen_dynamic_done");
    emit_dynamic_php_filter_swap(ctx);
    emit_dynamic_php_wrapper_branch(ctx, inst, &done)?;
    emit_dynamic_data_branch(ctx, inst, &done)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x2, #7");                              // is the dynamic filename long enough for http://?
            ctx.emitter.instruction(&format!("b.lt {}", plain));                // shorter filenames use the generic opener
            for (offset, byte) in b"http://".iter().enumerate() {
                ctx.emitter.instruction(&format!("ldrb w9, [x1, #{}]", offset)); // load one dynamic wrapper-prefix byte
                ctx.emitter.instruction(&format!("cmp w9, #{}", byte));         // compare against the canonical http:// byte
                ctx.emitter.instruction(&format!("b.ne {}", plain));            // a different prefix uses the generic opener
            }
            abi::emit_call_label(ctx.emitter, "__rt_http_open_url");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rdx, 7");                              // is the dynamic filename long enough for http://?
            ctx.emitter.instruction(&format!("jl {}", plain));                  // shorter filenames use the generic opener
            for (offset, byte) in b"http://".iter().enumerate() {
                ctx.emitter.instruction(&format!(
                    "cmp BYTE PTR [rax + {}], {}", offset, byte
                ));                                                             // compare one byte against the canonical http:// prefix
                ctx.emitter.instruction(&format!("jne {}", plain));             // a different prefix uses the generic opener
            }
            abi::emit_call_label(ctx.emitter, "__rt_http_open_url");
        }
    }
    box_stream_fd_or_false_result(ctx, "fopen_http_dynamic");
    emit_dynamic_php_filter_attach(ctx);                                        // a php://filter URL parked its filter; attach it now the stream exists
    emit_record_stream_meta_after_boxed_stashed(ctx, 1);
    emit_record_stream_mode_after_boxed(ctx, expect_operand(inst, 1)?)?;
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    finish_fopen_context_scope(ctx);
    store_if_result(ctx, inst)?;
    publish_http_response_headers(ctx);
    abi::emit_jump(ctx.emitter, &done);

    ctx.emitter.label(&plain);
    abi::emit_call_label(ctx.emitter, "__rt_fopen_maybe_phar");
    box_stream_fd_or_false_result(ctx, "fopen");
    emit_dynamic_php_filter_attach(ctx);                                        // a php://filter URL parked its filter; attach it now the stream exists
    emit_record_stream_meta_after_boxed_stashed(ctx, 0);
    emit_record_stream_mode_after_boxed(ctx, expect_operand(inst, 1)?)?;
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    finish_fopen_context_scope(ctx);
    store_if_result(ctx, inst)?;
    ctx.emitter.label(&done);
    Ok(())
}

/// Saves the active context bridges, selects and retains the fopen context, and publishes its state.
///
/// Shared with the path-based readers: any builtin taking a `$context` argument must
/// publish it for the duration of its own call, or the wrapper reads whatever context
/// happened to be published last.
pub(super) fn begin_fopen_context_scope(
    ctx: &mut FunctionContext<'_>,
    explicit_context: Option<ValueId>,
) -> Result<()> {
    abi::emit_reserve_temporary_stack(ctx.emitter, 48);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_context_options");
            ctx.emitter.instruction("ldr x10, [x9]");                           // save the previously active borrowed options pointer
            ctx.emitter.instruction("str x10, [sp, #0]");                       // preserve options for nested fopen restoration
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_notification_callback");
            ctx.emitter.instruction("ldr x10, [x9]");                           // save the previously active borrowed notifier descriptor
            ctx.emitter.instruction("str x10, [sp, #8]");                       // preserve notifier state for nested fopen restoration
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_current_context_handle");
            ctx.emitter.instruction("ldr x10, [x9]");                           // save the previously active borrowed context handle
            ctx.emitter.instruction("str x10, [sp, #32]");                      // preserve the active handle for nested wrapper restoration
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_context_options");
            ctx.emitter.instruction("mov r10, QWORD PTR [r9]");                 // save the previously active borrowed options pointer
            ctx.emitter.instruction("mov QWORD PTR [rsp + 0], r10");            // preserve options for nested fopen restoration
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_notification_callback");
            ctx.emitter.instruction("mov r10, QWORD PTR [r9]");                 // save the previously active borrowed notifier descriptor
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], r10");            // preserve notifier state for nested fopen restoration
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_current_context_handle");
            ctx.emitter.instruction("mov r10, QWORD PTR [r9]");                 // save the previously active borrowed context handle
            ctx.emitter.instruction("mov QWORD PTR [rsp + 32], r10");           // preserve the active handle for nested wrapper restoration
        }
    }

    let use_default = ctx.next_label("fopen_context_use_default");
    let selected = ctx.next_label("fopen_context_selected");
    match explicit_context {
        None => abi::emit_jump(ctx.emitter, &use_default),
        Some(context) => {
            let raw_ty = ctx.raw_value_php_type(context)?;
            match raw_ty {
                PhpType::Void | PhpType::Never => {
                    abi::emit_jump(ctx.emitter, &use_default);
                }
                // NOTE: `PhpType::Int` deliberately does NOT join this arm. A resource
                // bound to an untyped parameter is narrowed to Int by `codegen_repr()`,
                // and while the handle value survives the call, routing it here still
                // fails `__rt_context_state` validation at runtime. Accepting Int would
                // turn an explicit unsupported-feature diagnostic into an uncaught
                // exception. The real fix is preserving Resource across untyped
                // parameter binding in the checker.
                PhpType::Resource(_) => {
                    ctx.load_value_to_result(context)?;
                    abi::emit_jump(ctx.emitter, &selected);
                }
                PhpType::Mixed | PhpType::Union(_) => {
                    let resource_payload =
                        ctx.next_label("fopen_context_resource_payload");
                    ctx.load_value_to_result(context)?;
                    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
                    match ctx.emitter.target.arch {
                        Arch::AArch64 => {
                            ctx.emitter.instruction("cmp x0, #8");              // does the explicit Mixed context contain null?
                            ctx.emitter.instruction(&format!("b.eq {}", use_default)); // explicit null selects the request default
                            ctx.emitter.instruction("cmp x0, #9");              // does the explicit Mixed context contain a resource?
                            ctx.emitter.instruction(&format!("b.eq {}", resource_payload)); // resource payload is available in x1
                        }
                        Arch::X86_64 => {
                            ctx.emitter.instruction("cmp rax, 8");              // does the explicit Mixed context contain null?
                            ctx.emitter.instruction(&format!("je {}", use_default)); // explicit null selects the request default
                            ctx.emitter.instruction("cmp rax, 9");              // does the explicit Mixed context contain a resource?
                            ctx.emitter.instruction(&format!("je {}", resource_payload)); // resource payload is available in rdi
                        }
                    }
                    emit_stream_type_error(ctx, "fopen");
                    ctx.emitter.label(&resource_payload);
                    match ctx.emitter.target.arch {
                        Arch::AArch64 => {
                            ctx.emitter.instruction("mov x0, x1");              // expose the unboxed context handle
                        }
                        Arch::X86_64 => {
                            ctx.emitter.instruction("mov rax, rdi");            // expose the unboxed context handle
                        }
                    }
                    abi::emit_jump(ctx.emitter, &selected);
                }
                other => {
                    return Err(CodegenIrError::unsupported(format!(
                        "fopen context argument PHP type {:?}",
                        other
                    )));
                }
            }
        }
    }

    ctx.emitter.label(&use_default);
    emit_request_default_stream_context_handle(ctx);
    abi::emit_jump(ctx.emitter, &selected);

    ctx.emitter.label(&selected);
    let resolved_context = ctx.next_label("fopen_context_resolved");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp, #16]");                       // preserve the selected handle for attach and release
            abi::emit_call_label(ctx.emitter, "__rt_resource_retain");
            abi::emit_call_label(ctx.emitter, "__rt_context_state");
            ctx.emitter.instruction(&format!("cbnz x0, {}", resolved_context)); // continue only with a live ContextState
            emit_closed_stream_type_error(ctx, "fopen");
            ctx.emitter.label(&resolved_context);
            ctx.emitter.instruction("ldr x10, [x0, #0]");                       // load the selected context options pointer
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_context_options");
            ctx.emitter.instruction("str x10, [x9]");                           // publish options, including an explicit empty context
            ctx.emitter.instruction("ldr x10, [x0, #16]");                      // load the selected context notifier descriptor
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_notification_callback");
            ctx.emitter.instruction("str x10, [x9]");                           // publish notifier, including an explicit empty context
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_current_context_handle");
            ctx.emitter.instruction("ldr x10, [sp, #16]");                      // reload the selected context handle
            ctx.emitter.instruction("str x10, [x9]");                           // expose the borrowed handle to user-wrapper construction
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov QWORD PTR [rsp + 16], rax");           // preserve the selected handle for attach and release
            ctx.emitter.instruction("mov rdi, rax");                            // pass the selected handle to registry retain
            abi::emit_call_label(ctx.emitter, "__rt_resource_retain");
            ctx.emitter.instruction("mov rdi, rax");                            // pass the retained handle to typed context lookup
            abi::emit_call_label(ctx.emitter, "__rt_context_state");
            ctx.emitter.instruction("test rax, rax");                           // did the selected handle resolve to ContextState?
            ctx.emitter.instruction(&format!("jnz {}", resolved_context));      // continue only with a live ContextState
            emit_closed_stream_type_error(ctx, "fopen");
            ctx.emitter.label(&resolved_context);
            ctx.emitter.instruction("mov r10, QWORD PTR [rax + 0]");            // load the selected context options pointer
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_context_options");
            ctx.emitter.instruction("mov QWORD PTR [r9], r10");                 // publish options, including an explicit empty context
            ctx.emitter.instruction("mov r10, QWORD PTR [rax + 16]");           // load the selected context notifier descriptor
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_notification_callback");
            ctx.emitter.instruction("mov QWORD PTR [r9], r10");                 // publish notifier, including an explicit empty context
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_current_context_handle");
            ctx.emitter.instruction("mov r10, QWORD PTR [rsp + 16]");           // reload the selected context handle
            ctx.emitter.instruction("mov QWORD PTR [r9], r10");                 // expose the borrowed handle to user-wrapper construction
        }
    }
    Ok(())
}

/// Restores the prior context bridges and transfers one retained owner to a successful stream.
pub(super) fn finish_fopen_context_scope(ctx: &mut FunctionContext<'_>) {
    let restore = ctx.next_label("fopen_context_restore");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x0, [sp, #24]");                       // preserve the boxed fopen result across context cleanup
            ctx.emitter.instruction("ldr x9, [x0]");                            // load the boxed fopen result tag
            ctx.emitter.instruction("cmp x9, #9");                              // did fopen return a stream resource?
            ctx.emitter.instruction(&format!("b.ne {}", restore));              // false results have no StreamState to attach
            ctx.emitter.instruction("ldr x0, [x0, #8]");                        // load the opaque stream handle payload
            ctx.emitter.instruction("ldr x1, [sp, #16]");                       // load the selected context handle
            abi::emit_call_label(ctx.emitter, "__rt_stream_attach_context");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov QWORD PTR [rsp + 24], rax");           // preserve the boxed fopen result across context cleanup
            ctx.emitter.instruction("cmp QWORD PTR [rax], 9");                  // did fopen return a stream resource?
            ctx.emitter.instruction(&format!("jne {}", restore));               // false results have no StreamState to attach
            ctx.emitter.instruction("mov rdi, QWORD PTR [rax + 8]");            // load the opaque stream handle payload
            ctx.emitter.instruction("mov rsi, QWORD PTR [rsp + 16]");           // load the selected context handle
            abi::emit_call_label(ctx.emitter, "__rt_stream_attach_context");
        }
    }
    ctx.emitter.label(&restore);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_context_options");
            ctx.emitter.instruction("ldr x10, [sp, #0]");                       // reload the previously active options pointer
            ctx.emitter.instruction("str x10, [x9]");                           // restore the outer options bridge before release
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_notification_callback");
            ctx.emitter.instruction("ldr x10, [sp, #8]");                       // reload the previously active notifier descriptor
            ctx.emitter.instruction("str x10, [x9]");                           // restore the outer notifier bridge before release
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_current_context_handle");
            ctx.emitter.instruction("ldr x10, [sp, #32]");                      // reload the previously active borrowed context handle
            ctx.emitter.instruction("str x10, [x9]");                           // restore the outer wrapper context bridge
            ctx.emitter.instruction("ldr x0, [sp, #16]");                       // load the temporary selected-context owner
            abi::emit_call_label(ctx.emitter, "__rt_resource_release");
            ctx.emitter.instruction("ldr x0, [sp, #24]");                       // restore the boxed fopen result
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_context_options");
            ctx.emitter.instruction("mov r10, QWORD PTR [rsp + 0]");            // reload the previously active options pointer
            ctx.emitter.instruction("mov QWORD PTR [r9], r10");                 // restore the outer options bridge before release
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_notification_callback");
            ctx.emitter.instruction("mov r10, QWORD PTR [rsp + 8]");            // reload the previously active notifier descriptor
            ctx.emitter.instruction("mov QWORD PTR [r9], r10");                 // restore the outer notifier bridge before release
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_current_context_handle");
            ctx.emitter.instruction("mov r10, QWORD PTR [rsp + 32]");           // reload the previously active borrowed context handle
            ctx.emitter.instruction("mov QWORD PTR [r9], r10");                 // restore the outer wrapper context bridge
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 16]");           // load the temporary selected-context owner
            abi::emit_call_label(ctx.emitter, "__rt_resource_release");
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 24]");           // restore the boxed fopen result
        }
    }
    abi::emit_release_temporary_stack(ctx.emitter, 48);
}

/// Lazily creates the request-default context and leaves its global-owned handle in the result.
pub(super) fn emit_request_default_stream_context_handle(ctx: &mut FunctionContext<'_>) {
    let ready = ctx.next_label("fopen_default_context_ready");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_default_context_handle");
            ctx.emitter.instruction("ldr x0, [x9]");                            // load the request-global default context handle
            ctx.emitter.instruction(&format!("cbnz x0, {}", ready));            // reuse the existing request default
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_default_context_handle");
            ctx.emitter.instruction("mov rax, QWORD PTR [r9]");                 // load the request-global default context handle
            ctx.emitter.instruction("test rax, rax");                           // has the request default been allocated?
            ctx.emitter.instruction(&format!("jnz {}", ready));                 // reuse the existing request default
        }
    }
    clear_stream_context_options(ctx);
    clear_stream_notification_callback(ctx);
    emit_dynamic_stream_context_allocation(ctx, "fopen_default_context");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x9", "_stream_default_context_handle");
            ctx.emitter.instruction("str x0, [x9]");                            // transfer the creator reference to the request-global owner
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "r9", "_stream_default_context_handle");
            ctx.emitter.instruction("mov QWORD PTR [r9], rax");                 // transfer the creator reference to the request-global owner
        }
    }
    ctx.emitter.label(&ready);
}

/// Emits the boxed `fopen()` result for a compile-time literal path without storing it.
/// Where a literal open gets its mode.
///
/// `fopen()` reads it from the call's second argument. `file_get_contents()` has no such
/// argument, and its second operand is `$use_include_path` — reading that as a mode is how
/// this helper stayed unusable from there.
#[derive(Clone, Copy)]
pub(super) enum LiteralOpenMode {
    /// The mode string the call passed.
    Operand(ValueId),
    /// A read-only open, fixed by a caller that has no `$mode` argument.
    ReadOnly,
}

impl LiteralOpenMode {
    /// Whether this open writes. A caller-fixed read-only open never does.
    fn is_write(self, ctx: &mut FunctionContext<'_>) -> Result<bool> {
        match self {
            LiteralOpenMode::Operand(mode) => literal_fopen_mode_is_write(ctx, mode),
            LiteralOpenMode::ReadOnly => Ok(false),
        }
    }
}

pub(super) fn emit_literal_fopen_result(
    ctx: &mut FunctionContext<'_>,
    mode: LiteralOpenMode,
    path: &str,
) -> Result<()> {
    if let Some(fd) = php_standard_stream_fd(path).or_else(|| php_fd_stream(path)) {
        emit_dup_fd_result(ctx, fd);
        box_stream_fd_or_false_result(ctx, "fopen");
        emit_record_stream_meta_after_boxed_literal(ctx, 6, path);
        return Ok(());
    }
    if is_php_memory_stream(path) {
        abi::emit_call_label(ctx.emitter, "__rt_tmpfile");
        box_stream_fd_or_false_result(ctx, "fopen");
        emit_record_stream_meta_after_boxed_literal(ctx, 6, path);
        return Ok(());
    }
    if path.starts_with("data://") {
        return emit_literal_data_fopen_result(ctx, path);
    }
    if path.starts_with("ftp://") {
        return emit_literal_ftp_fopen_result(ctx, path);
    }
    if path.starts_with("phar://") {
        if mode.is_write(ctx)? {
            return emit_literal_phar_fopen_write_result(ctx, path);
        }
        return emit_literal_phar_fopen_read_result(ctx, path);
    }
    if path.starts_with("http://") {
        return emit_literal_http_fopen_result(ctx, path);
    }
    emit_runtime_fopen_literal_result(ctx, path, mode)
}

/// Emits a runtime `fopen()` call for a literal path and the caller's mode operand.
/// Loads the open mode into the string result registers, materializing `"r"` for a
/// caller-fixed read-only open.
fn emit_literal_open_mode_string(
    ctx: &mut FunctionContext<'_>,
    mode: LiteralOpenMode,
) -> Result<()> {
    match mode {
        LiteralOpenMode::Operand(mode) => load_string_to_result(ctx, mode, "fopen mode"),
        LiteralOpenMode::ReadOnly => {
            let (label, len) = ctx.data.add_string(b"r");
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    abi::emit_symbol_address(ctx.emitter, "x1", &label);
                    ctx.emitter.instruction(&format!("mov x2, #{}", len));      // the fixed read-only mode
                }
                Arch::X86_64 => {
                    abi::emit_symbol_address(ctx.emitter, "rax", &label);
                    ctx.emitter.instruction(&format!("mov rdx, {}", len));      // the fixed read-only mode
                }
            }
            Ok(())
        }
    }
}

pub(super) fn emit_runtime_fopen_literal_result(
    ctx: &mut FunctionContext<'_>,
    path: &str,
    mode: LiteralOpenMode,
) -> Result<()> {
    let (path_label, path_len) = ctx.data.add_string(path.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &path_label);
            ctx.emitter.instruction(&format!("mov x2, #{}", path_len));         // pass the literal fopen path byte length
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            emit_literal_open_mode_string(ctx, mode)?;
            ctx.emitter.instruction("mov x3, x1");                              // pass the fopen mode pointer with the literal path
            ctx.emitter.instruction("mov x4, x2");                              // pass the fopen mode length with the literal path
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rax", &path_label);
            ctx.emitter.instruction(&format!("mov rdx, {}", path_len));         // pass the literal fopen path byte length
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            emit_literal_open_mode_string(ctx, mode)?;
            ctx.emitter.instruction("mov rdi, rax");                            // pass the fopen mode pointer with the literal path
            ctx.emitter.instruction("mov rsi, rdx");                            // pass the fopen mode length with the literal path
            abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_fopen_maybe_phar");
    box_stream_fd_or_false_result(ctx, "fopen");
    emit_record_stream_meta_after_boxed_literal(ctx, 0, path);
    Ok(())
}

/// Emits a literal `fopen("php://filter/...", ...)` result without storing it.
pub(super) fn emit_literal_php_filter_fopen_result(
    ctx: &mut FunctionContext<'_>,
    mode: LiteralOpenMode,
    path: &str,
) -> Result<()> {
    let Some((mode_bits, filter_ids, resource)) = parse_php_filter_url(path) else {
        // php THROWS for a filter URL that names no resource — `Error: No URL resource
        // specified`, not a warning, and `@` does not soften it. A NESTED resource is the
        // other reason the parse declines; php recurses there, which is a separate,
        // still-open divergence, so it keeps the loud failed-open path.
        if literal_filter_url_names_no_resource(path) {
            crate::codegen::lower_inst::exceptions::emit_error(ctx, "No URL resource specified");
            return Ok(());
        }
        emit_fd_result(ctx, -1);
        box_stream_fd_or_false_result(ctx, "fopen_php_filter");
        return Ok(());
    };
    emit_literal_fopen_result(ctx, mode, &resource)?;
    if mode_bits != 0 {
        emit_php_filter_table_stamps(ctx, mode_bits, &filter_ids);
    }
    Ok(())
}

/// Returns whether a literal `php://filter/...` URL names NO resource — missing or empty.
///
/// This is the case php answers with `Error: No URL resource specified`; a nested resource
/// also makes [`parse_php_filter_url`] decline, but that is a different php behavior.
pub(super) fn literal_filter_url_names_no_resource(path: &str) -> bool {
    match path
        .strip_prefix("php://filter/")
        .map(|spec| spec.split_once("/resource="))
    {
        Some(None) => true,                                  // no separator at all
        Some(Some((_, resource))) => resource.is_empty(),    // an empty resource names nothing
        None => false,                                       // not a filter URL: not this error
    }
}

/// Parses `php://filter/[read=|write=]a|b|.../resource=path` for literal `fopen`.
///
/// Every name in the `|` chain is resolved, in order: php-src runs the record through all
/// of them. An unrecognised name is skipped and the rest still apply, which is what
/// `php -n` does — it is not an error and it does not cancel the chain.
pub(super) fn parse_php_filter_url(path: &str) -> Option<(u8, Vec<u8>, String)> {
    let spec = path.strip_prefix("php://filter/")?;
    let (filter_part, resource) = spec.split_once("/resource=")?;
    if resource.is_empty() {
        return None;
    }
    let (mode_bits, filters) = if let Some(filters) = filter_part.strip_prefix("read=") {
        (1u8, filters)
    } else if let Some(filters) = filter_part.strip_prefix("write=") {
        (2u8, filters)
    } else {
        (3u8, filter_part)
    };
    // An unrecognised name is SKIPPED, not fatal, and does not cancel its neighbours:
    // `php -n` 8.5.6 opens `read=string.toupper|no.such.filter` successfully and returns the
    // uppercased bytes. Measured, because the opposite reading is just as plausible.
    let filter_ids: Vec<u8> = filters.split('|').filter_map(stream_filter_id).collect();
    let mode_bits = if filter_ids.is_empty() { 0 } else { mode_bits };
    // A NESTED resource recurses, as php does: the inner level sits closest to the bytes, so
    // its chain applies FIRST and the outer chain sees what the inner one produced —
    // `read=string.rot13/resource=php://filter/read=string.toupper/resource=x` uppercases and
    // THEN rot13s, measured. Levels with conflicting explicit directions stay refused (the
    // pending hand-off carries one direction), which keeps that exotic spelling loudly failing
    // rather than half-filtered.
    if resource.starts_with("php://filter/") {
        let (inner_bits, inner_ids, innermost) = parse_php_filter_url(resource)?;
        if inner_bits != 0 && mode_bits != 0 && inner_bits != mode_bits {
            return None;
        }
        let bits = if mode_bits == 0 { inner_bits } else { mode_bits };
        let mut ids = inner_ids;
        ids.extend(filter_ids);
        return Some((bits, ids, innermost));
    }
    Some((mode_bits, filter_ids, resource.to_string()))
}

