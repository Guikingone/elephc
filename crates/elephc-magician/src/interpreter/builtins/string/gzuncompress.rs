//! Purpose:
//! Declarative eval registry entry for `gzuncompress`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Runtime dispatch is declared here and implemented through the gzip/zlib hook.

eval_builtin! {
    contract: "gzuncompress",
    area: String,
    direct: Gzip,
    values: Gzip,
}

use super::super::super::*;

/// Evaluates PHP `gzuncompress(...)` over eval expressions.
pub(in crate::interpreter) fn eval_builtin_gzuncompress(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::gzcompress::eval_builtin_gzip_named("gzuncompress", args, context, scope, values)
}

/// Applies PHP `gzuncompress(...)` to already evaluated arguments.
pub(in crate::interpreter) fn eval_gzuncompress_result(
    evaluated_args: &[RuntimeCellHandle],
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::gzcompress::eval_gzip_named_result("gzuncompress", evaluated_args, values)
}
