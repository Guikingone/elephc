//! Purpose:
//! Builds EIR scaffolding and target-aware frames for synthetic helpers emitted once per module.
//! Lets repeated lowering sequences move out of call sites without changing their register contract.
//!
//! Called from:
//! - `crate::codegen::shared_mixed_string` for boxed Mixed string-context dispatch.
//!
//! Key details:
//! - The boxed value already occupies the integer result register; no argument shuffle occurs.
//! - The helper preserves the reserved nested-call register on every supported target.
//! - Throws unwind through this ordinary frame to the caller's active handler.

use crate::codegen::abi;
use crate::codegen::context::FunctionContext;
use crate::codegen::data_section::DataSection;
use crate::codegen::emit::Emitter;
use crate::codegen::frame;
use crate::codegen::platform::Arch;
use crate::codegen::shared_state::SharedCodegenState;
use crate::ir::{
    BasicBlock, BlockId, Function, FunctionParam, IrHeapKind, IrType, Module, Ownership, Value,
    ValueDef, ValueId,
};
use crate::types::PhpType;

use super::Result;

/// Returns the synthetic SSA value that describes the helper's boxed parameter.
pub(super) fn helper_value() -> ValueId {
    ValueId::from_raw(0)
}

/// Emits one shared helper with a minimal EIR function, a target frame, and the supplied body.
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_shared_helper(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
    regalloc_linear: bool,
    label: &str,
    return_php_type: PhpType,
    comment: &str,
    body: impl FnOnce(&mut FunctionContext<'_>) -> Result<()>,
) -> Result<()> {
    let function = helper_function(label, return_php_type);
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
        None,
    );

    ctx.emitter.blank();
    ctx.emitter.comment(comment);
    ctx.emitter.label(label);
    emit_helper_entry(&mut ctx);
    body(&mut ctx)?;
    emit_helper_exit(&mut ctx);
    Ok(())
}

/// Emits a zero-argument shared helper that returns one fresh object in the integer result register.
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_shared_object_helper(
    module: &Module,
    emitter: &mut Emitter,
    data: &mut DataSection,
    shared: &mut SharedCodegenState,
    label: &str,
    owner_class: &str,
    comment: &str,
    body: impl FnOnce(&mut FunctionContext<'_>) -> Result<()>,
) -> Result<()> {
    let function = object_helper_function(label, owner_class);
    let layout = frame::layout_for_function(&function, emitter.target, false);
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
        None,
    );

    ctx.emitter.blank();
    ctx.emitter.comment(comment);
    ctx.emitter.label(label);
    emit_helper_entry(&mut ctx);
    body(&mut ctx)?;
    emit_helper_exit(&mut ctx);
    Ok(())
}

/// Builds the minimal zero-argument EIR function used by object materializer helpers.
fn object_helper_function(label: &str, owner_class: &str) -> Function {
    let return_php_type = PhpType::Object(owner_class.to_string());
    let mut function = Function::new(
        label.to_string(),
        IrType::Heap(IrHeapKind::Object),
        return_php_type,
    );
    function.flags.is_synthetic = true;
    let entry = BlockId::from_raw(0);
    function
        .blocks
        .push(BasicBlock::new(entry, "entry".to_string(), Vec::new()));
    function.entry = entry;
    function
}

/// Builds the minimal synthetic EIR function required by `FunctionContext`.
fn helper_function(label: &str, return_php_type: PhpType) -> Function {
    let return_ir_type = match return_php_type {
        PhpType::Str => IrType::Str,
        _ => IrType::Void,
    };
    let mut function = Function::new(label.to_string(), return_ir_type, return_php_type);
    function.flags.is_synthetic = true;
    function.params.push(FunctionParam {
        name: "value".to_string(),
        ir_type: IrType::Heap(IrHeapKind::Mixed),
        php_type: PhpType::Mixed,
        by_ref: false,
        variadic: false,
    });
    let entry = BlockId::from_raw(0);
    function.blocks.push(BasicBlock::new(
        entry,
        "entry".to_string(),
        vec![ValueId::from_raw(0)],
    ));
    function.values.push(Value {
        ir_type: IrType::Heap(IrHeapKind::Mixed),
        php_type: PhpType::Mixed,
        def: ValueDef::BlockParam {
            block: entry,
            index: 0,
        },
        ownership: Ownership::Borrowed,
    });
    function.entry = entry;
    function
}

/// Saves the frame pointer, return address, and reserved nested-call register.
fn emit_helper_entry(ctx: &mut FunctionContext<'_>) {
    let nested = abi::nested_call_reg(ctx.emitter);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("stp x29, x30, [sp, #-16]!");               // preserve the caller frame and return address
            ctx.emitter.instruction("mov x29, sp");                             // establish the helper frame pointer
            ctx.emitter.instruction(&format!("str {}, [sp, #-16]!", nested));   // preserve the nested-call register
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("push rbp");                                // preserve the caller frame pointer
            ctx.emitter.instruction("mov rbp, rsp");                            // establish the helper frame pointer
            ctx.emitter.instruction(&format!("push {}", nested));               // preserve the nested-call register
            ctx.emitter.instruction("sub rsp, 8");                              // maintain sixteen-byte alignment before calls
        }
    }
}

/// Restores the helper frame while leaving the string or void result registers untouched.
fn emit_helper_exit(ctx: &mut FunctionContext<'_>) {
    let nested = abi::nested_call_reg(ctx.emitter);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("ldr {}, [sp], #16", nested));     // restore the nested-call register
            ctx.emitter.instruction("ldp x29, x30, [sp], #16");                 // restore the caller frame and return address
            ctx.emitter.instruction("ret");                                     // return the helper result
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("add rsp, 8");                              // release alignment padding
            ctx.emitter.instruction(&format!("pop {}", nested));                // restore the nested-call register
            ctx.emitter.instruction("pop rbp");                                 // restore the caller frame pointer
            ctx.emitter.instruction("ret");                                     // return the helper result
        }
    }
}
