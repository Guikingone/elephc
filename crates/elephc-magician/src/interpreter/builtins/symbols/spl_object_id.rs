//! Purpose:
//! Eval registry entry and implementation for `spl_object_id`.
//!
//! Called from:
//! - `crate::interpreter::builtins::symbols`.
//!
//! Key details:
//! - `spl_object_hash()` shares the same object-identity implementation.

eval_builtin! {
    contract: "spl_object_id",
    area: Symbols,
    direct: Symbols,
    values: Symbols,
}

use super::super::super::*;

/// Evaluates direct `spl_object_id(...)` calls.
pub(in crate::interpreter) fn eval_spl_object_id_declared_call(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_builtin_spl_object_identity("spl_object_id", args, context, scope, values)
}

/// Evaluates materialized `spl_object_id(...)` arguments.
pub(in crate::interpreter) fn eval_spl_object_id_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    _context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match evaluated_args {
        [object] => eval_spl_object_identity_result("spl_object_id", *object, values),
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Evaluates PHP's SPL object identity builtins over one eval object expression.
pub(in crate::interpreter) fn eval_builtin_spl_object_identity(
    name: &str,
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [object] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let object = eval_expr(object, context, scope, values)?;
    eval_spl_object_identity_result(name, object, values)
}

/// Returns the PHP object handle in the native SPL builtin spelling.
///
/// `spl_object_id()` is the handle itself — the same small dense integer the AOT
/// engine reports and `var_dump()` prints as `object(C)#N`. `spl_object_hash()` is
/// PHP's 32-character rendering of that handle: 16 zero-padded hex digits followed
/// by 16 zeros, so handle `1` gives `"00000000000000010000000000000000"` (verified
/// against PHP 8.5.6).
pub(in crate::interpreter) fn eval_spl_object_identity_result(
    name: &str,
    object: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if values.type_tag(object)? != EVAL_TAG_OBJECT {
        return Err(EvalStatus::RuntimeFatal);
    }
    let handle = values.php_object_handle(object)?;
    match name {
        "spl_object_id" => values.int(handle as i64),
        "spl_object_hash" => values.string(&format!("{:016x}{:016x}", handle, 0)),
        _ => Err(EvalStatus::UnsupportedConstruct),
    }
}
