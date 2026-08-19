//! Purpose:
//! Emits module-wide helpers for repeated open Mixed callable dispatch.
//! Replaces per-site string, callable-array, object, and descriptor ladders with one body.
//!
//! Called from:
//! - `crate::codegen::block_emit::emit_module()` before ordinary function bodies.
//! - `crate::codegen::lower_inst::callables` at eligible descriptor-invoke sites.
//!
//! Key details:
//! - Only unknown-name Mixed callbacks with raw `array<mixed>` arguments are shared.
//! - Eval-backed functions and finite callable-name sets retain their specialized inline paths.
//! - Strict and extension-enabled profiles use distinct helpers and descriptor catalogs.

use crate::codegen::callable_reachability::CallableReachabilityAnalysis;
use crate::codegen::context::FunctionContext;
use crate::codegen::data_section::DataSection;
use crate::codegen::emit::Emitter;
use crate::codegen::frame;
use crate::codegen::lower_inst;
use crate::codegen::shared_state::SharedCodegenState;
use crate::ir::{
    BasicBlock, BlockId, Function, FunctionParam, Immediate, InstId, Instruction, IrHeapKind,
    IrType, LocalKind, Module, Op, Ownership, Terminator, Value, ValueDef, ValueId,
};
use crate::types::PhpType;

use super::Result;

/// Entry label for the extension-enabled open Mixed callable helper.
const MIXED_CALLABLE_LABEL: &str = "_eir_shared_mixed_callable_invoke";
/// Entry label for the strict-PHP open Mixed callable helper.
const STRICT_MIXED_CALLABLE_LABEL: &str = "_eir_shared_strict_mixed_callable_invoke";

/// Returns the shared helper label for an eligible call site.
pub(super) fn shared_helper_label(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    callable: ValueId,
    arg_container: ValueId,
) -> Option<&'static str> {
    let strict_php = lower_inst::instruction_strict_php_profile(inst);
    let label = helper_label(strict_php);
    if ctx.function.name == label
        || function_has_eval_context(ctx.function)
        || !is_open_mixed_callable_site(ctx, callable, arg_container)
    {
        return None;
    }
    ctx.shared
        .mixed_callable_sharing(profile_index(strict_php))
        .unwrap_or(false)
        .then_some(label)
}

/// Emits each strictness-profile helper required by at least two eligible sites.
pub(super) fn emit_shared_mixed_callable_helpers(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
    regalloc_linear: bool,
) -> Result<()> {
    for strict_php in [false, true] {
        let shares = mixed_callable_site_count(module, strict_php) >= 2;
        shared.set_mixed_callable_sharing(profile_index(strict_php), shares);
        if shares {
            emit_one_helper(
                module,
                emitter,
                data,
                shared,
                regalloc_linear,
                strict_php,
            )?;
        }
    }
    Ok(())
}

/// Counts open Mixed callable sites that share one complete descriptor catalog.
fn mixed_callable_site_count(module: &Module, strict_php: bool) -> usize {
    module_functions(module)
        .filter(|function| !function_has_eval_context(function))
        .map(|function| {
            let reachability = CallableReachabilityAnalysis::new(module, function);
            function
                .instructions
                .iter()
                .filter(|inst| {
                    if inst.op != Op::CallableDescriptorInvoke
                        || lower_inst::instruction_strict_php_profile(inst) != strict_php
                    {
                        return false;
                    }
                    let Some((&callable, &arg_container)) =
                        inst.operands.first().zip(inst.operands.get(1))
                    else {
                        return false;
                    };
                    site_types_are_shareable(function, callable, arg_container)
                        && reachability.candidates(callable).is_none()
                })
                .count()
        })
        .sum()
}

/// Iterates every ordinary, method, and closure body emitted by the backend.
fn module_functions(module: &Module) -> impl Iterator<Item = &Function> {
    module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .chain(module.closures.iter())
}

/// Returns whether a live context site has the shared helper's exact input shape.
fn is_open_mixed_callable_site(
    ctx: &FunctionContext<'_>,
    callable: ValueId,
    arg_container: ValueId,
) -> bool {
    site_types_are_shareable(ctx.function, callable, arg_container)
        && ctx.runtime_callable_candidates(callable).is_none()
}

/// Returns whether a site passes a boxed Mixed callback and raw Mixed-element array.
fn site_types_are_shareable(
    function: &Function,
    callable: ValueId,
    arg_container: ValueId,
) -> bool {
    let callback_ty = function
        .value(callable)
        .map(|value| value.php_type.codegen_repr());
    let container_ty = function
        .value(arg_container)
        .map(|value| value.php_type.codegen_repr());
    matches!(callback_ty, Some(PhpType::Mixed | PhpType::Union(_)))
        && matches!(
            container_ty,
            Some(PhpType::Array(element)) if element.codegen_repr() == PhpType::Mixed
        )
}

/// Returns whether a function owns a persistent eval context and needs its fallback path.
fn function_has_eval_context(function: &Function) -> bool {
    function
        .locals
        .iter()
        .any(|local| local.kind == LocalKind::EvalContext)
}

/// Emits one ordinary framed helper around the existing inline lowering routine.
fn emit_one_helper(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
    regalloc_linear: bool,
    strict_php: bool,
) -> Result<()> {
    let label = helper_label(strict_php);
    let function = helper_function(label, strict_php);
    let inst = function.instructions[0].clone();
    let layout = frame::layout_for_function(&function, emitter.target, regalloc_linear);
    let mut ctx = FunctionContext::new(
        module,
        &function,
        emitter,
        data,
        shared,
        layout,
        false,
        false,
        false,
        Some(format!("{}_epilogue", label)),
    );
    ctx.emitter.blank();
    ctx.emitter
        .comment(&format!("--- shared open Mixed callable dispatch: {} ---", label));
    frame::emit_function_prologue_with_label(&mut ctx, label)?;
    load_helper_param_value(&mut ctx, 0, callback_value())?;
    load_helper_param_value(&mut ctx, 1, arg_container_value())?;
    lower_inst::lower_mixed_callable_descriptor_invoke_inline(
        &mut ctx,
        &inst,
        callback_value(),
        arg_container_value(),
        "callable_descriptor_invoke",
    )?;
    ctx.load_value_to_result(result_value())?;
    frame::emit_function_return_epilogue(&mut ctx, None);
    Ok(())
}

/// Loads one borrowed helper parameter local into its synthetic SSA value home.
fn load_helper_param_value(
    ctx: &mut FunctionContext<'_>,
    raw_slot: u32,
    value: ValueId,
) -> Result<()> {
    ctx.load_raw_local_to_result(crate::ir::LocalSlotId::from_raw(raw_slot))?;
    ctx.store_result_value(value)
}

/// Builds the minimal EIR metadata needed by normal frame and callable lowering.
fn helper_function(label: &str, strict_php: bool) -> Function {
    let mixed_ir = IrType::Heap(IrHeapKind::Mixed);
    let array_ir = IrType::Heap(IrHeapKind::Array);
    let array_ty = PhpType::Array(Box::new(PhpType::Mixed));
    let entry = BlockId::from_raw(0);
    let inst_id = InstId::from_raw(0);
    let mut function = Function::new(label.to_string(), mixed_ir.clone(), PhpType::Mixed);
    function.flags.is_synthetic = true;
    function.params = vec![
        FunctionParam {
            name: "callback".to_string(),
            ir_type: mixed_ir.clone(),
            php_type: PhpType::Mixed,
            by_ref: false,
            variadic: false,
        },
        FunctionParam {
            name: "arguments".to_string(),
            ir_type: array_ir.clone(),
            php_type: array_ty.clone(),
            by_ref: false,
            variadic: false,
        },
    ];
    let callback_slot = function.add_local(
        Some("callback".to_string()),
        mixed_ir.clone(),
        PhpType::Mixed,
        LocalKind::BorrowedTemp,
    );
    let arguments_slot = function.add_local(
        Some("arguments".to_string()),
        array_ir.clone(),
        array_ty.clone(),
        LocalKind::BorrowedTemp,
    );
    function.no_epilogue_cleanup_slots.insert(callback_slot);
    function.no_epilogue_cleanup_slots.insert(arguments_slot);
    function.values = vec![
        Value {
            ir_type: mixed_ir.clone(),
            php_type: PhpType::Mixed,
            def: ValueDef::BlockParam {
                block: entry,
                index: 0,
            },
            ownership: Ownership::Borrowed,
        },
        Value {
            ir_type: array_ir,
            php_type: array_ty,
            def: ValueDef::BlockParam {
                block: entry,
                index: 1,
            },
            ownership: Ownership::Borrowed,
        },
        Value {
            ir_type: mixed_ir.clone(),
            php_type: PhpType::Mixed,
            def: ValueDef::Instruction {
                block: entry,
                index: 0,
                inst: inst_id,
            },
            ownership: Ownership::Owned,
        },
    ];
    function.instructions.push(Instruction::new(
        Op::CallableDescriptorInvoke,
        vec![callback_value(), arg_container_value()],
        Some(Immediate::Bool(strict_php)),
        Some(result_value()),
        mixed_ir,
        PhpType::Mixed,
        Ownership::Owned,
        Op::CallableDescriptorInvoke.default_effects(),
        None,
    ));
    let mut block = BasicBlock::new(
        entry,
        "entry".to_string(),
        vec![callback_value(), arg_container_value()],
    );
    block.instructions.push(inst_id);
    block.terminator = Some(Terminator::Return {
        value: Some(result_value()),
    });
    function.blocks.push(block);
    function.entry = entry;
    function
}

/// Returns the helper callback SSA value.
fn callback_value() -> ValueId {
    ValueId::from_raw(0)
}

/// Returns the helper argument-container SSA value.
fn arg_container_value() -> ValueId {
    ValueId::from_raw(1)
}

/// Returns the helper's owned Mixed result SSA value.
fn result_value() -> ValueId {
    ValueId::from_raw(2)
}

/// Returns the helper label for one strictness profile.
fn helper_label(strict_php: bool) -> &'static str {
    if strict_php {
        STRICT_MIXED_CALLABLE_LABEL
    } else {
        MIXED_CALLABLE_LABEL
    }
}

/// Maps one strictness profile to its shared-state slot.
fn profile_index(strict_php: bool) -> usize {
    usize::from(strict_php)
}
