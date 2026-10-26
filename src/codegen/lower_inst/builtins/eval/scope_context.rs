//! Purpose:
//! Manages persistent eval scope handles and active class scope.
//!
//! Called from:
//! - The eval lowering facade and sibling eval support modules.
//!
//! Key details:
//! - Late-static class lookup keeps target-specific lowering isolated here.

use super::*;

/// Loads the persistent eval context local into the selected integer argument register.
pub(super) fn load_eval_context_local_to_arg(
    ctx: &mut FunctionContext<'_>,
    context_offset: usize,
    arg_index: usize,
) {
    let arg_reg = abi::int_arg_reg_name(ctx.emitter.target, arg_index);
    abi::load_at_offset(ctx.emitter, arg_reg, context_offset);
}

/// Loads the current eval context handle into the selected integer argument register.
pub(super) fn load_eval_context_to_arg(ctx: &mut FunctionContext<'_>, arg_index: usize) {
    let arg_reg = abi::int_arg_reg_name(ctx.emitter.target, arg_index);
    abi::emit_load_temporary_stack_slot(ctx.emitter, arg_reg, EVAL_CONTEXT_HANDLE_OFFSET);
}

/// Reloads the saved eval source string into the bridge code pointer/length arguments.
pub(super) fn move_saved_eval_code_to_eval_args(ctx: &mut FunctionContext<'_>) {
    let code_ptr_arg = abi::int_arg_reg_name(ctx.emitter.target, 2);
    let code_len_arg = abi::int_arg_reg_name(ctx.emitter.target, 3);
    abi::emit_load_temporary_stack_slot(ctx.emitter, code_ptr_arg, EVAL_CODE_PTR_OFFSET);
    abi::emit_load_temporary_stack_slot(ctx.emitter, code_len_arg, EVAL_CODE_LEN_OFFSET);
}

/// Ensures a persistent eval scope exists and stores its handle in the scratch frame.
pub(super) fn ensure_eval_scope(ctx: &mut FunctionContext<'_>) -> Result<()> {
    let slot = eval_scope_slot(ctx)?;
    let offset = ctx.local_offset(slot)?;
    let ready = ctx.next_label("eval_scope_ready");
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::load_at_offset(ctx.emitter, result_reg, offset);
    abi::emit_branch_if_int_result_nonzero(ctx.emitter, &ready);
    let symbol = ctx.emitter.target.extern_symbol("__elephc_eval_scope_new");
    abi::emit_call_label(ctx.emitter, &symbol);
    abi::store_at_offset(ctx.emitter, result_reg, offset);
    ctx.emitter.label(&ready);
    abi::load_at_offset(ctx.emitter, result_reg, offset);
    abi::emit_store_to_sp(ctx.emitter, result_reg, EVAL_SCOPE_HANDLE_OFFSET);
    Ok(())
}

/// Ensures a persistent eval global-scope exists and stores its handle in scratch.
///
/// At true program top level (`is_main`) the calling scope IS the global scope: nothing tells
/// them apart the way a function's locals differ from the program's real globals. Reusing the
/// SAME persistent scope object `ensure_eval_scope` already ensures -- rather than the ordinarily
/// separate one this function allocates -- means a dynamically included or eval'd fragment's own
/// top-level names (never known to the AOT compiler, e.g. a dynamically selected vendor file) and
/// the native globals `flush_eval_global_scope`/`reload_eval_global_scope` sync into this handle
/// by name land in the SAME place. `context.global_scope_ptr()` (installed from this handle by
/// `set_eval_context_global_scope`) is what `$GLOBALS`, `global $x;`, and php's auto-global
/// superglobals resolve through from inside a function or method the fragment declares, so both
/// halves must agree on which object that is. Non-`is_main` call sites (eval()/include() written
/// inside a PHP function) keep the separate object: there, the calling scope is that function's
/// own locals, not the program's real global scope.
pub(super) fn ensure_eval_global_scope(ctx: &mut FunctionContext<'_>) -> Result<()> {
    if ctx.is_main {
        ensure_eval_scope(ctx)?;
        let slot = eval_scope_slot(ctx)?;
        let offset = ctx.local_offset(slot)?;
        let result_reg = abi::int_result_reg(ctx.emitter);
        abi::load_at_offset(ctx.emitter, result_reg, offset);
        abi::emit_store_to_sp(ctx.emitter, result_reg, EVAL_GLOBAL_SCOPE_HANDLE_OFFSET);
        return Ok(());
    }
    let slot = eval_global_scope_slot(ctx)?;
    let offset = ctx.local_offset(slot)?;
    let ready = ctx.next_label("eval_global_scope_ready");
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::load_at_offset(ctx.emitter, result_reg, offset);
    abi::emit_branch_if_int_result_nonzero(ctx.emitter, &ready);
    let symbol = ctx.emitter.target.extern_symbol("__elephc_eval_scope_new");
    abi::emit_call_label(ctx.emitter, &symbol);
    abi::store_at_offset(ctx.emitter, result_reg, offset);
    ctx.emitter.label(&ready);
    abi::load_at_offset(ctx.emitter, result_reg, offset);
    abi::emit_store_to_sp(ctx.emitter, result_reg, EVAL_GLOBAL_SCOPE_HANDLE_OFFSET);
    Ok(())
}

/// Returns the hidden frame slot that owns this function's persistent eval scope.
pub(super) fn eval_scope_slot(ctx: &FunctionContext<'_>) -> Result<LocalSlotId> {
    // Main executes in the global symbol table, including names first created by
    // opaque eval. Keep one owning slot so frame cleanup frees the table once.
    if ctx.is_main {
        return eval_global_scope_slot(ctx);
    }
    ctx.function
        .locals
        .iter()
        .find(|local| local.kind == LocalKind::EvalScope)
        .map(|local| local.id)
        .ok_or_else(|| CodegenIrError::invalid_module("eval call missing eval scope local"))
}

/// Returns the hidden frame slot that owns this function's eval global scope.
pub(super) fn eval_global_scope_slot(ctx: &FunctionContext<'_>) -> Result<LocalSlotId> {
    ctx.function
        .locals
        .iter()
        .find(|local| local.kind == LocalKind::EvalGlobalScope)
        .map(|local| local.id)
        .ok_or_else(|| CodegenIrError::invalid_module("eval call missing eval global scope local"))
}

/// Loads the current eval scope handle into the selected integer argument register.
pub(super) fn load_eval_scope_to_arg(ctx: &mut FunctionContext<'_>, arg_index: usize) {
    let arg_reg = abi::int_arg_reg_name(ctx.emitter.target, arg_index);
    abi::emit_load_temporary_stack_slot(ctx.emitter, arg_reg, EVAL_SCOPE_HANDLE_OFFSET);
}

/// Loads the current eval global-scope handle into the selected integer argument register.
pub(super) fn load_eval_global_scope_to_arg(ctx: &mut FunctionContext<'_>, arg_index: usize) {
    let arg_reg = abi::int_arg_reg_name(ctx.emitter.target, arg_index);
    abi::emit_load_temporary_stack_slot(ctx.emitter, arg_reg, EVAL_GLOBAL_SCOPE_HANDLE_OFFSET);
}

/// Installs the current eval global-scope handle into the eval context.
pub(super) fn set_eval_context_global_scope(ctx: &mut FunctionContext<'_>) {
    load_eval_context_to_arg(ctx, 0);
    load_eval_global_scope_to_arg(ctx, 1);
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_context_set_global_scope");
    abi::emit_call_label(ctx.emitter, &symbol);
    emit_eval_status_check(ctx);
}

/// Returns whether this body is a closure whose scope `Closure::bind` may set at run time.
pub(in crate::codegen) fn has_runtime_closure_scope(ctx: &FunctionContext<'_>) -> bool {
    ctx.local_slot_by_name(crate::names::CLOSURE_SCOPE_LOCAL).is_some()
}

/// Leaves this frame's EFFECTIVE class scope name in `ptr_reg`/`len_reg` (empty = global scope).
///
/// That is `fallback` -- the frame's lexical class, as each caller spells it -- unless the body is
/// a closure carrying the hidden scope capture (`crate::names::CLOSURE_SCOPE_LOCAL`) and
/// `Closure::bind` stamped it:
/// - a positive stamp is a compiled class id plus one, named by `_class_name_entries`;
/// - a negative stamp is an interned `[name_ptr, name_len]` record with bit 63 set, made by
///   `__elephc_eval_scope_name_token` for a scope class only the interpreter declared.
///
/// Every bridge that hands the interpreter a calling scope reads it here: with the lexical class
/// alone, a closure bound to an interpreted class wrote that class's private properties "from
/// global scope" and was refused. Clobbers x10-x12 on AArch64 and r10/r11 on x86_64.
pub(in crate::codegen) fn emit_effective_scope_name(
    ctx: &mut FunctionContext<'_>,
    fallback: &str,
    ptr_reg: &str,
    len_reg: &str,
) -> Result<()> {
    let (fallback_label, fallback_len) = ctx.data.add_string(fallback.as_bytes());
    let Some(slot) = ctx.local_slot_by_name(crate::names::CLOSURE_SCOPE_LOCAL) else {
        abi::emit_symbol_address(ctx.emitter, ptr_reg, &fallback_label);
        abi::emit_load_int_immediate(ctx.emitter, len_reg, fallback_len as i64);
        return Ok(());
    };
    let offset = ctx.local_offset(slot)?;
    let lexical = ctx.next_label("effective_scope_lexical");
    let interpreted = ctx.next_label("effective_scope_interpreted");
    let done = ctx.next_label("effective_scope_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::load_at_offset(ctx.emitter, "x12", offset);
            ctx.emitter.instruction(&format!("cbz x12, {lexical}"));             // never rebound: the lexical scope
            ctx.emitter.instruction(&format!("tbnz x12, #63, {interpreted}"));   // bit 63: an interned interpreter class name
            ctx.emitter.instruction("sub x12, x12, #1");                          // undo the +1 encoding: the bound class id
            abi::emit_load_symbol_to_reg(ctx.emitter, "x10", "_class_name_count", 0);
            ctx.emitter.instruction("cmp x12, x10");                             // an id past the table names nothing
            ctx.emitter.instruction(&format!("b.hs {lexical}"));                // keep the lexical scope
            abi::emit_symbol_address(ctx.emitter, "x11", "_class_name_entries");
            ctx.emitter.instruction("add x11, x11, x12, lsl #4");                // select the 16-byte class-name row
            ctx.emitter.instruction(&format!("ldr {ptr_reg}, [x11]"));          // bound class name pointer
            ctx.emitter.instruction(&format!("ldr {len_reg}, [x11, #8]"));      // bound class name length
            ctx.emitter.instruction(&format!("b {done}"));                      // scope ready
            ctx.emitter.label(&interpreted);
            ctx.emitter.instruction("and x12, x12, #0x7fffffffffffffff");        // strip the marker: the name record address
            ctx.emitter.instruction(&format!("ldr {ptr_reg}, [x12]"));           // interned name pointer
            ctx.emitter.instruction(&format!("ldr {len_reg}, [x12, #8]"));       // interned name length
            ctx.emitter.instruction(&format!("b {done}"));                      // scope ready
        }
        Arch::X86_64 => {
            abi::load_at_offset(ctx.emitter, "r10", offset);
            ctx.emitter.instruction("test r10, r10");                           // never rebound?
            ctx.emitter.instruction(&format!("jz {lexical}"));                  // the lexical scope
            ctx.emitter.instruction(&format!("js {interpreted}"));              // bit 63: an interned interpreter class name
            ctx.emitter.instruction("sub r10, 1");                              // undo the +1 encoding: the bound class id
            abi::emit_load_symbol_to_reg(ctx.emitter, "r11", "_class_name_count", 0);
            ctx.emitter.instruction("cmp r10, r11");                            // an id past the table names nothing
            ctx.emitter.instruction(&format!("jae {lexical}"));                 // keep the lexical scope
            abi::emit_symbol_address(ctx.emitter, "r11", "_class_name_entries");
            ctx.emitter.instruction("shl r10, 4");                              // 16-byte class-name rows
            ctx.emitter.instruction("add r11, r10");                            // select the bound class's row
            ctx.emitter.instruction(&format!("mov {ptr_reg}, QWORD PTR [r11]")); // bound class name pointer
            ctx.emitter.instruction(&format!("mov {len_reg}, QWORD PTR [r11 + 8]")); // bound class name length
            ctx.emitter.instruction(&format!("jmp {done}"));                    // scope ready
            ctx.emitter.label(&interpreted);
            ctx.emitter.instruction("btr r10, 63");                             // strip the marker: the name record address
            ctx.emitter.instruction(&format!("mov {ptr_reg}, QWORD PTR [r10]")); // interned name pointer
            ctx.emitter.instruction(&format!("mov {len_reg}, QWORD PTR [r10 + 8]")); // interned name length
            ctx.emitter.instruction(&format!("jmp {done}"));                    // scope ready
        }
    }
    ctx.emitter.label(&lexical);
    abi::emit_symbol_address(ctx.emitter, ptr_reg, &fallback_label);
    abi::emit_load_int_immediate(ctx.emitter, len_reg, fallback_len as i64);
    ctx.emitter.label(&done);
    Ok(())
}

/// Enters the current AOT method's class scope in the eval context, if any.
pub(super) fn push_eval_context_class_scope(ctx: &mut FunctionContext<'_>) -> Result<bool> {
    let class_name = current_eval_method_class(ctx).map(str::to_string);
    let Some(class_name) = class_name.or_else(|| has_runtime_closure_scope(ctx).then(String::new))
    else {
        return Ok(false);
    };
    emit_eval_called_class_name_result(ctx, &class_name)?;
    let (called_ptr_reg, called_len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_store_to_sp(ctx.emitter, called_ptr_reg, EVAL_CALLED_CLASS_PTR_OFFSET);
    abi::emit_store_to_sp(ctx.emitter, called_len_reg, EVAL_CALLED_CLASS_LEN_OFFSET);
    load_eval_context_to_arg(ctx, 0);
    let target = ctx.emitter.target;
    emit_effective_scope_name(
        ctx,
        &class_name,
        abi::int_arg_reg_name(target, 1),
        abi::int_arg_reg_name(target, 2),
    )?;
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 3),
        EVAL_CALLED_CLASS_PTR_OFFSET,
    );
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 4),
        EVAL_CALLED_CLASS_LEN_OFFSET,
    );
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_context_push_class_scope");
    abi::emit_call_label(ctx.emitter, &symbol);
    emit_eval_status_check(ctx);
    Ok(true)
}

/// Leaves a pushed eval class scope while preserving the original eval status.
///
/// The status is parked in `EVAL_STATUS_SAVE_OFFSET` rather than the generic temp cell: a caller
/// that boxed a statically typed receiver left that box's only pointer in
/// `EVAL_TEMP_CELL_OFFSET` and still owes it a release once the status has been checked, so
/// borrowing that slot here would silently drop the box and strand its object reference.
pub(super) fn pop_eval_context_class_scope(ctx: &mut FunctionContext<'_>, pushed: bool) {
    if !pushed {
        return;
    }
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_store_to_sp(ctx.emitter, result_reg, EVAL_STATUS_SAVE_OFFSET);
    load_eval_context_to_arg(ctx, 0);
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_context_pop_class_scope");
    abi::emit_call_label(ctx.emitter, &symbol);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, EVAL_STATUS_SAVE_OFFSET);
}

/// Publishes this compiled frame's lexical class for one bridge call that carries no context.
///
/// The owned-receiver path hands the bridge a NULL caller context on purpose -- the owner
/// context decides dispatch -- and a body with no eval context local of its own cannot forward
/// scopes at all. The interpreter then read no calling scope and refused a protected override
/// "from global scope", which is how `Twig\Template::yield()` calling `$this->doDisplay()`
/// died. Publishing the lexical class costs two calls and needs neither a context nor a wider
/// method-call ABI.
///
/// Returns whether anything was pushed, for [`pop_native_caller_class`].
pub(super) fn push_native_caller_class(ctx: &mut FunctionContext<'_>) -> Result<bool> {
    let class_name = current_eval_method_class(ctx).map(str::to_string);
    let Some(class_name) = class_name.or_else(|| has_runtime_closure_scope(ctx).then(String::new))
    else {
        return Ok(false);
    };
    let target = ctx.emitter.target;
    emit_effective_scope_name(
        ctx,
        &class_name,
        abi::int_arg_reg_name(target, 0),
        abi::int_arg_reg_name(target, 1),
    )?;
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_push_native_caller_class");
    abi::emit_call_label(ctx.emitter, &symbol);
    Ok(true)
}

/// Drops the published lexical class, preserving the status the bridge call just returned.
///
/// The status is parked in `EVAL_STATUS_SAVE_OFFSET` for the same reason
/// [`pop_eval_context_class_scope`] parks it there: the generic temp cell may still hold a box
/// this frame owes a release.
pub(super) fn pop_native_caller_class(ctx: &mut FunctionContext<'_>, pushed: bool) {
    if !pushed {
        return;
    }
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_store_to_sp(ctx.emitter, result_reg, EVAL_STATUS_SAVE_OFFSET);
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_pop_native_caller_class");
    abi::emit_call_label(ctx.emitter, &symbol);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, EVAL_STATUS_SAVE_OFFSET);
}

/// Returns the lexical class encoded in the current EIR callable name.
pub(super) fn current_eval_method_class<'a>(ctx: &'a FunctionContext<'_>) -> Option<&'a str> {
    current_eval_function_class(ctx.function)
}

/// Returns the PHP lexical class of a callable body, including a synthetic closure.
pub(super) fn current_eval_function_class(function: &crate::ir::Function) -> Option<&str> {
    function
        .name
        .rsplit_once("::")
        .map(|(class_name, _)| class_name)
        .or(function.lexical_class.as_deref())
}

/// Returns the physical source file that owns a compiled callable body.
///
/// A module has one entry path, but included declarations and class-scoped closures retain their
/// own source identities. Eval/include bridges must use the body's source so relative includes,
/// magic constants, and diagnostics keep PHP's file-local semantics.
pub(super) fn eval_source_path_for_function<'a>(
    module: &'a crate::ir::Module,
    function: &crate::ir::Function,
) -> Option<&'a str> {
    if let Some(class_name) = current_eval_function_class(function) {
        if let Some(path) = module
            .declared_class_source_files
            .get(&crate::names::php_symbol_key(class_name.trim_start_matches('\\')))
        {
            return Some(path);
        }
    }
    module
        .declared_function_source_files
        .get(&crate::names::php_symbol_key(&function.name))
        .map(String::as_str)
        .or(module.source_path.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::Target;
    use crate::ir::{Function, IrType, Module};
    use crate::types::PhpType;

    #[test]
    fn synthetic_closure_keeps_its_lexical_class_for_eval_scope() {
        let mut closure = Function::new("{closure:Owner:0}".to_string(), IrType::Void, PhpType::Void);
        closure.lexical_class = Some("Owner".to_string());

        assert_eq!(current_eval_function_class(&closure), Some("Owner"));
    }

    #[test]
    fn eval_source_path_prefers_lexical_class_declaration_file() {
        let mut module = Module::new(Target::detect_host());
        module.source_path = Some("/entry.php".to_string());
        module.declared_class_source_files.insert(
            crate::names::php_symbol_key("Owner"),
            "/included/Owner.php".to_string(),
        );
        let mut closure = Function::new("{closure:Owner:0}".to_string(), IrType::Void, PhpType::Void);
        closure.lexical_class = Some("Owner".to_string());

        assert_eq!(
            eval_source_path_for_function(&module, &closure),
            Some("/included/Owner.php")
        );
    }
}

/// Materializes the runtime called-class name for eval `static::` resolution.
pub(super) fn emit_eval_called_class_name_result(
    ctx: &mut FunctionContext<'_>,
    fallback_class: &str,
) -> Result<()> {
    if eval_late_static_class_id_available(ctx) {
        match ctx.emitter.target.arch {
            Arch::AArch64 => emit_eval_called_class_name_result_aarch64(ctx),
            Arch::X86_64 => emit_eval_called_class_name_result_x86_64(ctx),
        }
    } else {
        emit_eval_static_string_result(ctx, fallback_class.as_bytes());
        Ok(())
    }
}

/// Emits the AArch64 class-id table lookup for eval's called class.
pub(super) fn emit_eval_called_class_name_result_aarch64(ctx: &mut FunctionContext<'_>) -> Result<()> {
    let missing = ctx.next_label("eval_called_class_missing");
    let done = ctx.next_label("eval_called_class_done");
    emit_eval_late_static_class_id_to_reg(ctx, "x12")?;
    abi::emit_load_symbol_to_reg(ctx.emitter, "x10", "_class_name_count", 0);
    ctx.emitter.instruction("cmp x12, x10");                                    // reject called-class ids outside the class-name table
    ctx.emitter.instruction(&format!("b.hs {}", missing));                      // fall back to the lexical eval class when metadata is missing
    abi::emit_symbol_address(ctx.emitter, "x11", "_class_name_entries");
    ctx.emitter.instruction("lsl x12, x12, #4");                                // convert class id to a 16-byte class-name table offset
    ctx.emitter.instruction("add x11, x11, x12");                               // select the called-class metadata row
    ctx.emitter.instruction("ldr x1, [x11]");                                   // load the called-class name pointer
    ctx.emitter.instruction("ldr x2, [x11, #8]");                               // load the called-class name length
    ctx.emitter.instruction(&format!("b {}", done));                            // skip the missing-metadata fallback
    ctx.emitter.label(&missing);
    abi::emit_symbol_address(ctx.emitter, "x1", "_class_name_missing");
    ctx.emitter.instruction("mov x2, #0");                                      // empty called-class name triggers lexical fallback in eval
    ctx.emitter.label(&done);
    Ok(())
}

/// Emits the x86_64 class-id table lookup for eval's called class.
pub(super) fn emit_eval_called_class_name_result_x86_64(ctx: &mut FunctionContext<'_>) -> Result<()> {
    let missing = ctx.next_label("eval_called_class_missing");
    let done = ctx.next_label("eval_called_class_done");
    emit_eval_late_static_class_id_to_reg(ctx, "r8")?;
    abi::emit_load_symbol_to_reg(ctx.emitter, "r9", "_class_name_count", 0);
    ctx.emitter.instruction("cmp r8, r9");                                      // reject called-class ids outside the class-name table
    ctx.emitter.instruction(&format!("jae {}", missing));                       // fall back to the lexical eval class when metadata is missing
    abi::emit_symbol_address(ctx.emitter, "r10", "_class_name_entries");
    ctx.emitter.instruction("shl r8, 4");                                       // convert class id to a 16-byte class-name table offset
    ctx.emitter.instruction("add r10, r8");                                     // select the called-class metadata row
    ctx.emitter.instruction("mov rax, QWORD PTR [r10]");                        // load the called-class name pointer
    ctx.emitter.instruction("mov rdx, QWORD PTR [r10 + 8]");                    // load the called-class name length
    ctx.emitter.instruction(&format!("jmp {}", done));                          // skip the missing-metadata fallback
    ctx.emitter.label(&missing);
    abi::emit_symbol_address(ctx.emitter, "rax", "_class_name_missing");
    ctx.emitter.instruction("mov rdx, 0");                                      // empty called-class name triggers lexical fallback in eval
    ctx.emitter.label(&done);
    Ok(())
}

/// Returns true when the current method frame can provide a late-static class id.
pub(super) fn eval_late_static_class_id_available(ctx: &FunctionContext<'_>) -> bool {
    ctx.local_slot_by_name(crate::names::CALLED_CLASS_ID_LOCAL).is_some()
        || ctx.local_slot_by_name("this").is_some()
}

/// Loads the late-static class id from the hidden static slot or `$this`.
pub(super) fn emit_eval_late_static_class_id_to_reg(ctx: &mut FunctionContext<'_>, reg: &str) -> Result<()> {
    if let Some(slot) = ctx.local_slot_by_name(crate::names::CALLED_CLASS_ID_LOCAL) {
        let offset = ctx.local_offset(slot)?;
        abi::load_at_offset(ctx.emitter, reg, offset);
        return Ok(());
    }
    if let Some(slot) = ctx.local_slot_by_name("this") {
        match ctx.local_php_type(slot)? {
            PhpType::Mixed | PhpType::Union(_) => {
                ctx.load_local_to_result(slot)?;
                abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
                let object_reg = eval_mixed_unbox_low_payload_reg(ctx);
                abi::emit_load_from_address(ctx.emitter, reg, object_reg, 0);
            }
            PhpType::Object(_) => {
                let offset = ctx.local_offset(slot)?;
                abi::load_at_offset(ctx.emitter, reg, offset);
                abi::emit_load_from_address(ctx.emitter, reg, reg, 0);
            }
            other => {
                return Err(CodegenIrError::invalid_module(format!(
                    "eval class scope this local has PHP type {:?}",
                    other
                )))
            }
        }
        return Ok(());
    }
    Err(CodegenIrError::invalid_module(format!(
        "eval class scope without called-class source in {}",
        ctx.function.name
    )))
}

/// Emits a static string result for eval class-scope setup fallback paths.
pub(super) fn emit_eval_static_string_result(ctx: &mut FunctionContext<'_>, bytes: &[u8]) {
    let (label, len) = ctx.data.add_string(bytes);
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_symbol_address(ctx.emitter, ptr_reg, &label);
    abi::emit_load_int_immediate(ctx.emitter, len_reg, len as i64);
}
