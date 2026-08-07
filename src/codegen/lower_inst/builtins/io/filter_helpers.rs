//! Purpose:
//! Filter ids, params, user filters, and boxed stream resources.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;

/// Maps runtime-supported built-in stream filter names to byte-table ids.
pub(super) fn stream_filter_id(name: &str) -> Option<u8> {
    match name {
        "string.toupper" => Some(1),
        "string.tolower" => Some(2),
        "string.rot13" => Some(3),
        "string.strip_tags" => Some(4),
        "dechunk" => Some(5),
        "convert.base64-encode" => Some(6),
        "convert.base64-decode" => Some(7),
        "convert.quoted-printable-encode" => Some(8),
        "convert.quoted-printable-decode" => Some(9),
        _ => None,
    }
}

/// Reads a compile-time integer filter parameter from the fourth builtin operand.
pub(super) fn const_int_filter_param(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
    _key: &str,
    primary: bool,
    min: i64,
    max: i64,
) -> Result<Option<i64>> {
    if !primary {
        return Ok(None);
    }
    let Some(value) = inst.operands.get(3).copied() else {
        return Ok(None);
    };
    Ok(optional_const_i64_operand(ctx, value)?.map(|n| n.clamp(min, max)))
}

/// Returns a literal integer operand when the value was produced by `ConstI64`.
pub(super) fn optional_const_i64_operand(ctx: &FunctionContext<'_>, value: ValueId) -> Result<Option<i64>> {
    let value_ref = ctx
        .function
        .value(value)
        .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))?;
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Ok(None);
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    if inst_ref.op != Op::ConstI64 {
        return Ok(None);
    }
    match inst_ref.immediate {
        Some(Immediate::I64(value)) => Ok(Some(value)),
        _ => Err(CodegenIrError::invalid_module(
            "integer literal operand has no i64 immediate",
        )),
    }
}

/// Attaches a built-in stream filter by writing its id into per-fd direction tables.
pub(super) fn lower_builtin_stream_filter_attach(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    filter_id: u8,
    prepend: bool,
) -> Result<()> {
    let stream = expect_operand(inst, 0)?;
    load_open_stream_handle_to_result(ctx, stream, "stream_filter_append")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    materialize_stream_filter_mode(ctx, inst)?;
    emit_attach_filter_node(ctx, Some(filter_id), prepend);
    // Box the registry handle itself. `emit_boxed_stream_resource` mints a fresh
    // display id, which the legacy design could afford because its "filter
    // resource" was really the stream descriptor; a chain node has to be findable
    // again by `stream_filter_remove()`.
    emit_boxed_filter_handle(ctx);
    store_if_result(ctx, inst)
}

/// Creates a filter node and links it into the direction chains the mode selects.
///
/// On entry the stream handle is on the stack and the mode bits are in the int
/// result register. On exit the new filter handle is in the result register, ready
/// to be boxed as the resource `stream_filter_append()` returns.
///
/// A node attached with `STREAM_FILTER_ALL` is linked into both chains, which is
/// why the direction bits are stored on the node rather than inferred from which
/// list it sits in.
fn emit_attach_filter_node(ctx: &mut FunctionContext<'_>, filter_id: Option<u8>, prepend: bool) {
    let skip_read = ctx.next_label("sf_chain_skip_read");
    let skip_write = ctx.next_label("sf_chain_skip_write");
    let prepend_flag = i64::from(prepend);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_pop_reg(ctx.emitter, "x4");                               // recover the owning stream handle
            abi::emit_reserve_temporary_stack(ctx.emitter, 32);
            ctx.emitter.instruction("str x4, [sp, #0]");                        // preserve the stream handle across the calls
            ctx.emitter.instruction("str x0, [sp, #8]");                        // preserve the requested direction bits
            ctx.emitter.instruction("mov x2, x0");                              // direction bits
            match filter_id {
                // A literal name resolves at compile time; a dynamic one arrives in
                // x9 from __rt_builtin_filter_id.
                Some(id) => ctx.emitter.instruction(&format!("mov x0, #{id}")),  // built-in filter id
                None => ctx.emitter.instruction("mov x0, x9"),                   // run-time resolved filter id
            }
            ctx.emitter.instruction("mov x1, #0");                              // built-ins carry no user-filter object
            ctx.emitter.instruction("mov x3, #0");                              // built-ins retain no params value
            abi::emit_call_label(ctx.emitter, "__rt_filter_create");            // x0 = the new filter handle
            ctx.emitter.instruction("str x0, [sp, #16]");                       // preserve the filter handle

            ctx.emitter.instruction("ldr x9, [sp, #8]");                        // direction bits
            ctx.emitter.instruction("tst x9, #1");                              // is STREAM_FILTER_READ set?
            ctx.emitter.instruction(&format!("b.eq {}", skip_read));
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // owning stream handle
            ctx.emitter.instruction("ldr x1, [sp, #16]");                       // filter handle
            ctx.emitter.instruction(&format!("mov x2, #{STREAM_READ_FILTER_HEAD_OFFSET}"));
            ctx.emitter.instruction(&format!("mov x3, #{prepend_flag}"));       // prepend selects head insertion
            abi::emit_call_label(ctx.emitter, "__rt_stream_filter_link");
            ctx.emitter.label(&skip_read);

            ctx.emitter.instruction("ldr x9, [sp, #8]");                        // direction bits
            ctx.emitter.instruction("tst x9, #2");                              // is STREAM_FILTER_WRITE set?
            ctx.emitter.instruction(&format!("b.eq {}", skip_write));
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // owning stream handle
            ctx.emitter.instruction("ldr x1, [sp, #16]");                       // filter handle
            ctx.emitter.instruction(&format!("mov x2, #{STREAM_WRITE_FILTER_HEAD_OFFSET}"));
            ctx.emitter.instruction(&format!("mov x3, #{prepend_flag}"));       // prepend selects head insertion
            abi::emit_call_label(ctx.emitter, "__rt_stream_filter_link");
            ctx.emitter.label(&skip_write);

            ctx.emitter.instruction("ldr x0, [sp, #16]");                       // return the filter handle
            abi::emit_release_temporary_stack(ctx.emitter, 32);
        }
        Arch::X86_64 => {
            abi::emit_pop_reg(ctx.emitter, "rcx");                              // recover the owning stream handle
            abi::emit_reserve_temporary_stack(ctx.emitter, 32);
            ctx.emitter.instruction("mov QWORD PTR [rsp + 0], rcx");            // preserve the stream handle across the calls
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rax");            // preserve the requested direction bits
            ctx.emitter.instruction("mov rdx, rax");                            // direction bits
            match filter_id {
                // A literal name resolves at compile time; a dynamic one arrives in
                // r13 from __rt_builtin_filter_id.
                Some(id) => ctx.emitter.instruction(&format!("mov rdi, {id}")),  // built-in filter id
                None => ctx.emitter.instruction("mov rdi, r13"),                 // run-time resolved filter id
            }
            ctx.emitter.instruction("xor esi, esi");                            // built-ins carry no user-filter object
            ctx.emitter.instruction("xor ecx, ecx");                            // built-ins retain no params value
            abi::emit_call_label(ctx.emitter, "__rt_filter_create");            // rax = the new filter handle
            ctx.emitter.instruction("mov QWORD PTR [rsp + 16], rax");           // preserve the filter handle

            ctx.emitter.instruction("mov r9, QWORD PTR [rsp + 8]");             // direction bits
            ctx.emitter.instruction("test r9, 1");                              // is STREAM_FILTER_READ set?
            ctx.emitter.instruction(&format!("jz {}", skip_read));
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // owning stream handle
            ctx.emitter.instruction("mov rsi, QWORD PTR [rsp + 16]");           // filter handle
            ctx.emitter.instruction(&format!("mov rdx, {STREAM_READ_FILTER_HEAD_OFFSET}"));
            ctx.emitter.instruction(&format!("mov rcx, {prepend_flag}"));       // prepend selects head insertion
            abi::emit_call_label(ctx.emitter, "__rt_stream_filter_link");
            ctx.emitter.label(&skip_read);

            ctx.emitter.instruction("mov r9, QWORD PTR [rsp + 8]");             // direction bits
            ctx.emitter.instruction("test r9, 2");                              // is STREAM_FILTER_WRITE set?
            ctx.emitter.instruction(&format!("jz {}", skip_write));
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // owning stream handle
            ctx.emitter.instruction("mov rsi, QWORD PTR [rsp + 16]");           // filter handle
            ctx.emitter.instruction(&format!("mov rdx, {STREAM_WRITE_FILTER_HEAD_OFFSET}"));
            ctx.emitter.instruction(&format!("mov rcx, {prepend_flag}"));       // prepend selects head insertion
            abi::emit_call_label(ctx.emitter, "__rt_stream_filter_link");
            ctx.emitter.label(&skip_write);

            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 16]");           // return the filter handle
            abi::emit_release_temporary_stack(ctx.emitter, 32);
        }
    }
}

/// Materializes the stream-filter mode operand, defaulting to STREAM_FILTER_ALL.
pub(super) fn materialize_stream_filter_mode(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() >= 3 {
        let mode = expect_operand(inst, 2)?;
        require_int_or_bool(
            ctx.load_value_to_result(mode)?.codegen_repr(),
            "stream_filter_append mode",
        )?;
        return Ok(());
    }
    emit_fd_result(ctx, 3);
    Ok(())
}

/// Materializes the optional stream-filter params operand as an owned boxed
/// Mixed cell, defaulting to PHP null when the caller omitted it.
pub(super) fn materialize_stream_filter_params(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() < 4 {
        emit_null_mixed(ctx);
        return Ok(());
    }
    let params = expect_operand(inst, 3)?;
    let params_ty = ctx.value_php_type(params)?.codegen_repr();
    ctx.load_value_to_result(params)?;
    if matches!(params_ty, PhpType::Mixed | PhpType::Union(_)) {
        if !ctx.value_can_own_mixed_box_source(params)? {
            abi::emit_incref_if_refcounted(ctx.emitter, &params_ty);
        }
    } else {
        emit_box_current_value_as_mixed(ctx.emitter, &params_ty);
    }
    Ok(())
}

/// Attaches a user-defined stream filter through the runtime registry.
pub(super) fn lower_user_stream_filter_attach(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    prepend: bool,
) -> Result<()> {
    let stream = expect_operand(inst, 0)?;
    let filter = expect_operand(inst, 1)?;
    // A dynamic name may still be a built-in. Resolving it here keeps such
    // filters on the chain instead of the legacy per-descriptor slots, which hold
    // only 256 descriptors.
    let user_path = ctx.next_label("sfa_user_path");
    let attached = ctx.next_label("sfa_attached");
    load_string_to_result(ctx, filter, "stream_filter_append filter")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // name pointer
            ctx.emitter.instruction("mov x1, x2");                              // name length
            abi::emit_call_label(ctx.emitter, "__rt_builtin_filter_id");
            ctx.emitter.instruction(&format!("cbz x0, {}", user_path));         // not a built-in: use the user-filter path
            ctx.emitter.instruction("str x0, [sp, #-16]!");                     // preserve the resolved id across the operand loads
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // name pointer
            ctx.emitter.instruction("mov rsi, rdx");                            // name length
            abi::emit_call_label(ctx.emitter, "__rt_builtin_filter_id");
            ctx.emitter.instruction("test rax, rax");
            ctx.emitter.instruction(&format!("jz {}", user_path));              // not a built-in: use the user-filter path
            ctx.emitter.instruction("push rax");                                // preserve the resolved id across the operand loads
        }
    }
    load_open_stream_handle_to_result(ctx, stream, "stream_filter_append")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    materialize_stream_filter_mode(ctx, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            // The node creator reads the run-time id from x9 and pops the handle itself.
            ctx.emitter.instruction("ldr x9, [sp, #16]");                       // resolved id, below the pushed handle
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov r13, QWORD PTR [rsp + 8]");            // resolved id, below the pushed handle
        }
    }
    emit_attach_filter_node(ctx, None, prepend);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("add sp, sp, #16");                         // drop the preserved id
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("add rsp, 8");                              // drop the preserved id
        }
    }
    emit_boxed_filter_handle(ctx);
    abi::emit_jump(ctx.emitter, &attached);
    ctx.emitter.label(&user_path);
    lower_user_stream_filter_attach_legacy(ctx, inst)?;
    ctx.emitter.label(&attached);
    return Ok(());
}

/// Attaches a user-registered filter through the legacy per-descriptor slots.
fn lower_user_stream_filter_attach_legacy(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let stream = expect_operand(inst, 0)?;
    let filter = expect_operand(inst, 1)?;
    load_stream_fd_to_result(ctx, stream, "stream_filter_append")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    load_string_to_result(ctx, filter, "stream_filter_append filter")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            materialize_stream_filter_mode(ctx, inst)?;
            abi::emit_push_reg(ctx.emitter, "x0");
            materialize_stream_filter_params(ctx, inst)?;
            ctx.emitter.instruction("mov x4, x0");                              // pass the boxed stream-filter params to the attach helper
            abi::emit_pop_reg(ctx.emitter, "x3");
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
            ctx.emitter.instruction("ldr x0, [sp]");                            // pass the saved stream descriptor without popping it yet
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            materialize_stream_filter_mode(ctx, inst)?;
            abi::emit_push_reg(ctx.emitter, "rax");
            materialize_stream_filter_params(ctx, inst)?;
            ctx.emitter.instruction("mov r8, rax");                             // pass the boxed stream-filter params to the attach helper
            abi::emit_pop_reg(ctx.emitter, "rcx");
            abi::emit_pop_reg_pair(ctx.emitter, "rsi", "rdx");
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp]");                // pass the saved stream descriptor without popping it yet
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_stream_filter_attach_user");
    let fail = ctx.next_label("sfau_false");
    let done = ctx.next_label("sfau_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x0, {}", fail));              // unknown filter or failed onCreate returns PHP false
            ctx.emitter.instruction("ldr x0, [sp]");                            // reload the descriptor for the returned filter resource
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            emit_boxed_stream_resource(ctx);
            ctx.emitter.instruction(&format!("b {}", done));                    // skip the PHP false boxing path
            ctx.emitter.label(&fail);
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            emit_boxed_bool(ctx, false);
            ctx.emitter.label(&done);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // did the attach helper report success?
            ctx.emitter.instruction(&format!("jz {}", fail));                   // unknown filter or failed onCreate returns PHP false
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp]");                // reload the descriptor for the returned filter resource
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            emit_boxed_stream_resource(ctx);
            ctx.emitter.instruction(&format!("jmp {}", done));                  // skip the PHP false boxing path
            ctx.emitter.label(&fail);
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            emit_boxed_bool(ctx, false);
            ctx.emitter.label(&done);
        }
    }
    store_if_result(ctx, inst)
}

/// Boxes the current integer result as a PHP stream resource Mixed cell.
///
/// Mints a fresh resource id first: like a descriptor, a filter handle can repeat a
/// number a previous, now-released filter used, and PHP never hands the same
/// resource id out twice.
pub(super) fn emit_boxed_stream_resource(ctx: &mut FunctionContext<'_>) {
    abi::emit_call_label(ctx.emitter, "__rt_resource_id_mint");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x1, x0");                              // use the descriptor as the resource payload
            ctx.emitter.instruction("mov x2, #0");                              // resource Mixed payloads do not use the high word
            ctx.emitter.instruction("mov x0, #9");                              // runtime tag 9 = resource
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // use the descriptor as the resource payload
            ctx.emitter.instruction("xor esi, esi");                            // resource Mixed payloads do not use the high word
            ctx.emitter.instruction("mov eax, 9");                              // runtime tag 9 = resource
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
        }
    }
}

/// Boxes a PHP boolean Mixed cell in the current result register.
pub(super) fn emit_boxed_bool(ctx: &mut FunctionContext<'_>, value: bool) {
    emit_bool_result(ctx, value);
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Bool);
}

/// Boxes a PHP null Mixed cell in the current result register.
pub(super) fn emit_null_mixed(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, #0");                              // null has no payload
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("xor eax, eax");                            // null has no payload
        }
    }
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Void);
}

