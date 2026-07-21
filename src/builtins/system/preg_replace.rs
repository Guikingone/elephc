//! Purpose:
//! Home of the PHP `preg_replace` builtin: its declaration, check hook, and lowering.
//!
//! Called from:
//! - The builtin registry (declaration), the type checker (check hook), and the EIR
//!   backend (lower hook), all via `crate::builtins::registry`.
//!
//! Key details:
//! - PHP: `preg_replace(string|array $pattern, string|array $replacement,
//!   string|array $subject, int $limit = -1, int &$count = null): string|array|null`.
//!   `$count` is a by-ref OUT-parameter: it must be a variable and is not eagerly
//!   inferred (its value is produced by the call), so the spec sets `lazy_check: true`
//!   and the hook infers only the four input arguments.
//! - `lower` is a thin wrapper over `regex::lower_preg_replace` in the EIR backend.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::codegen::context::FunctionContext;
use crate::codegen::CodegenIrError;
use crate::errors::CompileError;
use crate::ir::Instruction;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    name: "preg_replace",
    area: System,
    params: [pattern: Str, replacement: Str, subject: Str, limit: Int = crate::builtins::spec::DefaultSpec::Int(-1), ref count: Mixed = crate::builtins::spec::DefaultSpec::Null],
    returns: Str,
    check: check,
    lazy_check: true,
    lower: lower,
    summary: "Performs a regular expression search and replace.",
}

/// Validates a `preg_replace` call: infers the four INPUT arguments (`$pattern`,
/// `$replacement`, `$subject`, `$limit`) and requires an explicit `$count` argument to be
/// a plain variable (PHP by-ref out-parameter; its value is produced by the call, so it is
/// never eagerly inferred). Arity (3–5) is pre-validated by the registry.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    for arg in cx.args.iter().take(4) {
        cx.checker.infer_type(arg, cx.env)?;
    }
    if cx.args.len() == 5 && !matches!(cx.args[4].kind, ExprKind::Variable(_)) {
        return Err(CompileError::new(
            cx.args[4].span,
            "preg_replace() parameter $count must be passed a variable",
        ));
    }
    Ok(PhpType::Str)
}

/// Lowers a `preg_replace` call by dispatching to the shared regex emitter.
fn lower(ctx: &mut FunctionContext, inst: &Instruction) -> Result<(), CodegenIrError> {
    crate::codegen::lower_inst::builtins::regex::lower_preg_replace(ctx, inst)
}
