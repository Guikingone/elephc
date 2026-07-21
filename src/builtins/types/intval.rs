//! Purpose:
//! Home of the PHP `intval` builtin: its declaration and lowering.
//!
//! Called from:
//! - The builtin registry (declaration) and the EIR backend (lower hook), via `crate::builtins::registry`.
//!
//! Key details:
//! - Declared as `intval(mixed $value, int $base = 10)` matching PHP's signature.
//! - The check hook validates that an explicit `$base` argument is an `int`.
//! - `lower` is a thin wrapper over the shared intval emitter.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::codegen::context::FunctionContext;
use crate::codegen::CodegenIrError;
use crate::errors::CompileError;
use crate::ir::Instruction;
use crate::types::PhpType;

builtin! {
    name: "intval",
    area: Types,
    params: [value: Mixed, base: Int = crate::builtins::spec::DefaultSpec::Int(10)],
    returns: Int,
    check: check,
    lower: lower,
    summary: "Returns the integer value of a variable.",
    php_manual: "function.intval",
}

/// Validates an `intval` call: an explicit `$base` argument must be an `int`.
/// Arity (1–2) is pre-validated by the registry; argument types are inferred by the
/// common registry dispatch path before this hook fires.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    if let Some(base_arg) = cx.args.get(1) {
        let base_ty = cx.checker.infer_type(base_arg, cx.env)?;
        if !matches!(base_ty, PhpType::Int) {
            return Err(CompileError::new(
                base_arg.span,
                "intval() base argument must be int",
            ));
        }
    }
    Ok(PhpType::Int)
}

/// Lowers an `intval` call by dispatching to the shared intval emitter.
fn lower(ctx: &mut FunctionContext, inst: &Instruction) -> Result<(), CodegenIrError> {
    crate::codegen::lower_inst::builtins::lower_intval(ctx, inst)
}
