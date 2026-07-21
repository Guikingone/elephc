//! Purpose:
//! Home of the PHP `min` builtin: its declaration, type-check hook, and lowering.
//!
//! Called from:
//! - The builtin registry (declaration), the type checker (check hook), and the EIR
//!   backend (lower hook), all via `crate::builtins::registry`.
//!
//! Key details:
//! - A `check` hook is required because the return type depends on argument types:
//!   any Float argument widens the result to Float; otherwise the result is Int.
//! - `min_args: 1` admits PHP's single-argument form `min(array $value_array)`; the
//!   check hook requires that lone argument to be an array (gradually acceptable).

use crate::builtins::spec::BuiltinCheckCtx;
use crate::codegen::context::FunctionContext;
use crate::codegen::CodegenIrError;
use crate::errors::CompileError;
use crate::ir::Instruction;
use crate::types::PhpType;

builtin! {
    name: "min",
    area: Math,
    params: [value: Mixed],
    variadic: "values",
    min_args: 1,
    arity_error: "min() requires at least 1 argument",
    returns: Mixed,
    check: check,
    lower: lower,
    summary: "Find lowest value.",
    php_manual: "https://www.php.net/manual/en/function.min.php",
}

/// Returns Float when any argument is Float, otherwise returns Int.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    // PHP overloads min/max as `min/max(array $value_array)` (a single array argument)
    // or `min/max(mixed $value, mixed ...$values)` (two or more scalar arguments).
    if cx.args.len() == 1 {
        // Single-argument form: the lone argument must be an array, accepted under the
        // gradual boundary (concrete array, `Mixed`, or a union containing an array); a
        // concrete scalar is a type error matching PHP's `min(1)` / `max(1)` TypeError.
        let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
        if !crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&ty) {
            return Err(CompileError::new(
                cx.span,
                &format!("{}() single argument must be array", cx.name),
            ));
        }
        return Ok(crate::types::checker::builtins::numeric::min_max_single_array_result(&ty));
    }
    // Two-or-more-argument scalar form: any argument types are accepted; a float
    // argument promotes the result to float.
    let mut has_float = false;
    for arg in cx.args {
        let t = cx.checker.infer_type(arg, cx.env)?;
        if t == PhpType::Float {
            has_float = true;
        }
    }
    if has_float {
        Ok(PhpType::Float)
    } else {
        Ok(PhpType::Int)
    }
}

/// Lowers a `min` call by dispatching to the shared min/max emitter with `want_max = false`.
fn lower(ctx: &mut FunctionContext, inst: &Instruction) -> Result<(), CodegenIrError> {
    crate::codegen::lower_inst::builtins::math::lower_min_max(ctx, inst, false)
}
