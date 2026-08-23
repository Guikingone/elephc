//! Purpose:
//! Home of the PHP `array_column` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` accepts indexed or associative collections of associative/gradual rows whose shape is
//!   runtime-known. The result preserves a known associative value type and otherwise becomes
//!   `array<mixed>`. Other shapes are rejected. A check hook is required because the return type
//!   depends on the inferred argument type.
//! - Arity (exactly 2 arguments) is validated by the registry's `check_arity` before
//!   the hook fires; the inline arity check from the legacy arm is not reproduced here.
//!   Note elephc only supports the 2-argument form (`array`, `column_key`).

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "array_column",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayColumn,
    ),
}

/// Returns the extracted-column array type for an `array_column` call.
///
/// The first argument must be an indexed or associative array of associative or gradual rows. Known
/// associative rows preserve their value type; gradual rows produce `array<mixed>`. Other
/// shapes are rejected. The argument is re-inferred here to drive the return type; the registry
/// already inferred every argument once for side effects, and arity (exactly 2) is pre-validated.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    match ty {
        PhpType::Array(inner) => match *inner {
            PhpType::AssocArray { value, .. } => Ok(PhpType::Array(value)),
            PhpType::Mixed | PhpType::Union(_) => {
                Ok(PhpType::Array(Box::new(PhpType::Mixed)))
            }
            _ => Err(CompileError::new(
                cx.span,
                "array_column() requires an array of associative arrays",
            )),
        },
        PhpType::AssocArray { value, .. } => match *value {
            PhpType::AssocArray { value, .. } => Ok(PhpType::Array(value)),
            PhpType::Mixed | PhpType::Union(_) => {
                Ok(PhpType::Array(Box::new(PhpType::Mixed)))
            }
            _ => Ok(PhpType::Array(Box::new(PhpType::Mixed))),
        },
        PhpType::Mixed | PhpType::Union(_) => Ok(PhpType::Array(Box::new(PhpType::Mixed))),
        _ => Err(CompileError::new(
            cx.span,
            "array_column() first argument must be array",
        )),
    }
}
