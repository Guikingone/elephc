//! Purpose:
//! Touch timestamp ABI and pathinfo result lowering.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::io`.
//!
//! Key details:
//! - Preserves target-aware ABI handling, runtime calls, and result ownership.

use super::*;

/// Emits wrapper `stream_metadata()` dispatch for a loaded `touch()` call.
pub(super) fn emit_touch_wrapper_dispatch(ctx: &mut FunctionContext<'_>) {
    let wrapper = ctx.next_label("touch_wrapper");
    let after = ctx.next_label("touch_after");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("sub sp, sp, #48");                         // reserve path, time, flags, and result scratch storage
            ctx.emitter.instruction("str x1, [sp, #0]");                        // preserve the path pointer
            ctx.emitter.instruction("str x2, [sp, #8]");                        // preserve the path length
            ctx.emitter.instruction("str x3, [sp, #16]");                       // preserve mtime seconds
            ctx.emitter.instruction("str x4, [sp, #24]");                       // preserve atime seconds
            ctx.emitter.instruction("str x5, [sp, #32]");                       // preserve current-time flags
            ctx.emitter.instruction("mov x0, x1");                              // pass path pointer to wrapper-scheme probe
            ctx.emitter.instruction("mov x1, x2");                              // pass path length to wrapper-scheme probe
            abi::emit_call_label(ctx.emitter, "__rt_path_is_wrapper");
            ctx.emitter.instruction(&format!("cbnz x0, {}", wrapper));          // registered wrapper schemes use stream_metadata
            ctx.emitter.instruction("ldr x1, [sp, #0]");                        // pass path pointer to native touch
            ctx.emitter.instruction("ldr x2, [sp, #8]");                        // pass path length to native touch
            ctx.emitter.instruction("ldr x3, [sp, #16]");                       // pass mtime seconds to native touch
            ctx.emitter.instruction("ldr x4, [sp, #24]");                       // pass atime seconds to native touch
            ctx.emitter.instruction("ldr x5, [sp, #32]");                       // pass current-time flags to native touch
            ctx.emitter.instruction("add sp, sp, #48");                         // release scratch before native touch
            abi::emit_call_label(ctx.emitter, "__rt_touch");
            ctx.emitter.instruction(&format!("b {}", after));                   // skip wrapper stream_metadata after native touch
            ctx.emitter.label(&wrapper);
            ctx.emitter.instruction("ldr x0, [sp, #16]");                       // pass mtime to touch metadata array builder
            ctx.emitter.instruction("ldr x1, [sp, #24]");                       // pass atime to touch metadata array builder
            ctx.emitter.instruction("ldr x2, [sp, #32]");                       // pass current-time flags to metadata array builder
            abi::emit_call_label(ctx.emitter, "__rt_touch_meta_array");
            ctx.emitter.instruction("str x0, [sp, #16]");                       // preserve the boxed touch metadata value
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // pass wrapper path pointer
            ctx.emitter.instruction("ldr x1, [sp, #8]");                        // pass wrapper path length
            ctx.emitter.instruction(&format!("mov x2, #{}", STREAM_METADATA_SLOT)); // select stream_metadata vtable slot
            ctx.emitter.instruction(&format!("mov x3, #{}", STREAM_META_TOUCH)); // select STREAM_META_TOUCH
            ctx.emitter.instruction("ldr x4, [sp, #16]");                       // pass boxed touch metadata value
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_path_op");
            ctx.emitter.instruction("str x0, [sp, #0]");                        // preserve stream_metadata result across value release
            ctx.emitter.instruction("ldr x0, [sp, #16]");                       // reload the boxed touch metadata value
            abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
            ctx.emitter.instruction("ldr x0, [sp, #0]");                        // restore the stream_metadata boolean result
            ctx.emitter.instruction("add sp, sp, #48");                         // release wrapper touch scratch storage
            ctx.emitter.label(&after);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("sub rsp, 48");                             // reserve path, time, flags, and result scratch storage
            ctx.emitter.instruction("mov QWORD PTR [rsp + 0], rax");            // preserve the path pointer
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rdx");            // preserve the path length
            ctx.emitter.instruction("mov QWORD PTR [rsp + 16], rdi");           // preserve mtime seconds
            ctx.emitter.instruction("mov QWORD PTR [rsp + 24], rsi");           // preserve atime seconds
            ctx.emitter.instruction("mov QWORD PTR [rsp + 32], rcx");           // preserve current-time flags
            ctx.emitter.instruction("mov rdi, rax");                            // pass path pointer to wrapper-scheme probe
            ctx.emitter.instruction("mov rsi, rdx");                            // pass path length to wrapper-scheme probe
            abi::emit_call_label(ctx.emitter, "__rt_path_is_wrapper");
            ctx.emitter.instruction("test rax, rax");                           // test whether the path scheme matched a wrapper
            ctx.emitter.instruction(&format!("jnz {}", wrapper));               // registered wrapper schemes use stream_metadata
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 0]");            // pass path pointer to native touch
            ctx.emitter.instruction("mov rdx, QWORD PTR [rsp + 8]");            // pass path length to native touch
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 16]");           // pass mtime seconds to native touch
            ctx.emitter.instruction("mov rsi, QWORD PTR [rsp + 24]");           // pass atime seconds to native touch
            ctx.emitter.instruction("mov rcx, QWORD PTR [rsp + 32]");           // pass current-time flags to native touch
            ctx.emitter.instruction("add rsp, 48");                             // release scratch before native touch
            abi::emit_call_label(ctx.emitter, "__rt_touch");
            ctx.emitter.instruction(&format!("jmp {}", after));                 // skip wrapper stream_metadata after native touch
            ctx.emitter.label(&wrapper);
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 16]");           // pass mtime to touch metadata array builder
            ctx.emitter.instruction("mov rsi, QWORD PTR [rsp + 24]");           // pass atime to touch metadata array builder
            ctx.emitter.instruction("mov rdx, QWORD PTR [rsp + 32]");           // pass current-time flags to metadata array builder
            abi::emit_call_label(ctx.emitter, "__rt_touch_meta_array");
            ctx.emitter.instruction("mov QWORD PTR [rsp + 16], rax");           // preserve the boxed touch metadata value
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp + 0]");            // pass wrapper path pointer
            ctx.emitter.instruction("mov rsi, QWORD PTR [rsp + 8]");            // pass wrapper path length
            ctx.emitter.instruction(&format!("mov rdx, {}", STREAM_METADATA_SLOT)); // select stream_metadata vtable slot
            ctx.emitter.instruction(&format!("mov rcx, {}", STREAM_META_TOUCH)); // select STREAM_META_TOUCH
            ctx.emitter.instruction("mov r8, QWORD PTR [rsp + 16]");            // pass boxed touch metadata value
            abi::emit_call_label(ctx.emitter, "__rt_user_wrapper_path_op");
            ctx.emitter.instruction("mov QWORD PTR [rsp + 0], rax");            // preserve stream_metadata result across value release
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 16]");           // reload the boxed touch metadata value
            abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 0]");            // restore the stream_metadata boolean result
            ctx.emitter.instruction("add rsp, 48");                             // release wrapper touch scratch storage
            ctx.emitter.label(&after);
        }
    }
}

/// Materializes timestamp arguments for the `touch()` call on ARM64.
pub(super) fn lower_touch_args_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    match touch_time_shape(ctx, inst)? {
        TouchTimeShape::BothNow => {
            ctx.emitter.instruction("mov x3, #0");                              // ignored mtime seconds when runtime uses current time
            ctx.emitter.instruction("mov x4, #0");                              // ignored atime seconds when runtime uses current time
            ctx.emitter.instruction(&format!("mov x5, #{}", TOUCH_BOTH_NOW));   // mark mtime and atime as current-time fields
        }
        TouchTimeShape::MtimeAlsoAtime => {
            let mtime = expect_operand(inst, 1)?;
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            require_int(ctx.load_value_to_result(mtime)?.codegen_repr(), "touch mtime")?;
            ctx.emitter.instruction("mov x3, x0");                              // pass explicit mtime seconds
            ctx.emitter.instruction("mov x4, x0");                              // default atime to the explicit mtime seconds
            ctx.emitter.instruction("mov x5, #0");                              // mark both timestamp fields as explicit
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        }
        TouchTimeShape::ExplicitBoth => {
            let mtime = expect_operand(inst, 1)?;
            let atime = expect_operand(inst, 2)?;
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            require_int(ctx.load_value_to_result(mtime)?.codegen_repr(), "touch mtime")?;
            ctx.emitter.instruction("str x0, [sp, #-16]!");                     // save explicit mtime while atime is evaluated
            require_int(ctx.load_value_to_result(atime)?.codegen_repr(), "touch atime")?;
            ctx.emitter.instruction("mov x4, x0");                              // pass explicit atime seconds
            ctx.emitter.instruction("ldr x3, [sp], #16");                       // restore explicit mtime seconds
            ctx.emitter.instruction("mov x5, #0");                              // mark both timestamp fields as explicit
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        }
    }
    Ok(())
}

/// Materializes timestamp arguments for the `touch()` call on x86_64.
pub(super) fn lower_touch_args_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    match touch_time_shape(ctx, inst)? {
        TouchTimeShape::BothNow => {
            ctx.emitter.instruction("mov rdi, 0");                              // ignored mtime seconds when runtime uses current time
            ctx.emitter.instruction("mov rsi, 0");                              // ignored atime seconds when runtime uses current time
            ctx.emitter.instruction(&format!("mov rcx, {}", TOUCH_BOTH_NOW));   // mark mtime and atime as current-time fields
        }
        TouchTimeShape::MtimeAlsoAtime => {
            let mtime = expect_operand(inst, 1)?;
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            require_int(ctx.load_value_to_result(mtime)?.codegen_repr(), "touch mtime")?;
            ctx.emitter.instruction("mov rdi, rax");                            // pass explicit mtime seconds
            ctx.emitter.instruction("mov rsi, rax");                            // default atime to the explicit mtime seconds
            ctx.emitter.instruction("mov rcx, 0");                              // mark both timestamp fields as explicit
            abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
        }
        TouchTimeShape::ExplicitBoth => {
            let mtime = expect_operand(inst, 1)?;
            let atime = expect_operand(inst, 2)?;
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            require_int(ctx.load_value_to_result(mtime)?.codegen_repr(), "touch mtime")?;
            ctx.emitter.instruction("sub rsp, 16");                             // reserve aligned temporary storage for mtime
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");                // save explicit mtime while atime is evaluated
            require_int(ctx.load_value_to_result(atime)?.codegen_repr(), "touch atime")?;
            ctx.emitter.instruction("mov rsi, rax");                            // pass explicit atime seconds
            ctx.emitter.instruction("mov rdi, QWORD PTR [rsp]");                // restore explicit mtime seconds
            ctx.emitter.instruction("add rsp, 16");                             // release the aligned mtime temporary
            ctx.emitter.instruction("mov rcx, 0");                              // mark both timestamp fields as explicit
            abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
        }
    }
    Ok(())
}

/// Classifies optional `touch()` timestamp operands after EIR type checking.
pub(super) fn touch_time_shape(ctx: &FunctionContext<'_>, inst: &Instruction) -> Result<TouchTimeShape> {
    match inst.operands.len() {
        1 => Ok(TouchTimeShape::BothNow),
        2 if is_nullish_value(ctx, expect_operand(inst, 1)?)? => Ok(TouchTimeShape::BothNow),
        2 => Ok(TouchTimeShape::MtimeAlsoAtime),
        _ if is_nullish_value(ctx, expect_operand(inst, 1)?)?
            && is_nullish_value(ctx, expect_operand(inst, 2)?)? =>
        {
            Ok(TouchTimeShape::BothNow)
        }
        _ if is_nullish_value(ctx, expect_operand(inst, 2)?)? => {
            Ok(TouchTimeShape::MtimeAlsoAtime)
        }
        _ => Ok(TouchTimeShape::ExplicitBoth),
    }
}

/// Returns true when an EIR value represents PHP `null`.
pub(super) fn is_nullish_value(ctx: &FunctionContext<'_>, value: ValueId) -> Result<bool> {
    Ok(matches!(
        ctx.value_php_type(value)?.codegen_repr(),
        PhpType::Void
    ))
}

/// Calls the single-component `pathinfo()` helper after materializing an integer flag.
pub(super) fn lower_pathinfo_string(ctx: &mut FunctionContext<'_>, flag: ValueId) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            require_int(ctx.load_value_to_result(flag)?.codegen_repr(), "pathinfo flags")?;
            ctx.emitter.instruction("mov x3, x0");                              // pass the selected PATHINFO_* flag to the string helper
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            require_int(ctx.load_value_to_result(flag)?.codegen_repr(), "pathinfo flags")?;
            ctx.emitter.instruction("mov rdi, rax");                            // pass the selected PATHINFO_* flag to the string helper
            abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_pathinfo_str");
    Ok(())
}

/// Lowers dynamic `pathinfo(path, flag)` and boxes string or array results as Mixed.
pub(super) fn lower_pathinfo_mixed(ctx: &mut FunctionContext<'_>, flag: ValueId) -> Result<()> {
    let array_label = ctx.next_label("pathinfo_dynamic_array");
    let done_label = ctx.next_label("pathinfo_dynamic_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
            require_int(ctx.load_value_to_result(flag)?.codegen_repr(), "pathinfo flags")?;
            ctx.emitter.instruction("mov x3, x0");                              // keep the evaluated flag in the string-helper flag register
            abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
            ctx.emitter.instruction("cmp x3, #15");                             // does the runtime flag request PATHINFO_ALL exactly?
            ctx.emitter.instruction(&format!("b.eq {}", array_label));          // runtime PATHINFO_ALL must produce the array shape
            abi::emit_call_label(ctx.emitter, "__rt_pathinfo_str");
            ctx.emitter.instruction("mov x0, #1");                              // select runtime tag 1 for a string Mixed value
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.instruction(&format!("b {}", done_label));              // skip array boxing after building the string result
            ctx.emitter.label(&array_label);
            abi::emit_call_label(ctx.emitter, "__rt_pathinfo_array");
            box_owned_pathinfo_array_as_mixed(ctx);
            ctx.emitter.label(&done_label);
        }
        Arch::X86_64 => {
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
            require_int(ctx.load_value_to_result(flag)?.codegen_repr(), "pathinfo flags")?;
            ctx.emitter.instruction("mov rdi, rax");                            // keep the evaluated flag in the string-helper flag register
            abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
            ctx.emitter.instruction("cmp rdi, 15");                             // does the runtime flag request PATHINFO_ALL exactly?
            ctx.emitter.instruction(&format!("je {}", array_label));            // runtime PATHINFO_ALL must produce the array shape
            abi::emit_call_label(ctx.emitter, "__rt_pathinfo_str");
            ctx.emitter.instruction("mov rdi, rax");                            // pass the component string pointer as the Mixed low payload word
            ctx.emitter.instruction("mov rsi, rdx");                            // pass the component string length as the Mixed high payload word
            ctx.emitter.instruction("mov eax, 1");                              // select runtime tag 1 for a string Mixed value
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.instruction(&format!("jmp {}", done_label));            // skip array boxing after building the string result
            ctx.emitter.label(&array_label);
            abi::emit_call_label(ctx.emitter, "__rt_pathinfo_array");
            box_owned_pathinfo_array_as_mixed(ctx);
            ctx.emitter.label(&done_label);
        }
    }
    Ok(())
}

