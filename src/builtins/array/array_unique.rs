//! Purpose:
//! Home of the PHP `array_unique` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` preserves concrete array shapes and accepts gradual `Mixed`/array unions
//!   through a runtime array assertion, widening their result to `array<mixed>`.
//!   Concretely non-array arguments remain compile errors.
//! - Arity (exactly 1 argument) is validated by the registry's `check_arity` before
//!   the hook fires; the inline arity check from the legacy arm is not reproduced here.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_unique",
    area: Array,
    params: [array: Mixed, flags: Int = crate::builtins::spec::DefaultSpec::Int(2)],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayUnique,
    ),
    summary: "Removes duplicate values from an array.",
    php_manual: "https://www.php.net/manual/en/function.array-unique.php",
}

/// Returns the (shape-preserving) array type for an `array_unique` call.
///
/// De-duplication keeps concrete array shapes, while a `Mixed` or union-containing-array
/// argument crosses the gradual runtime boundary and produces `array<mixed>`. Concretely
/// non-array arguments are rejected. The argument is re-inferred here; the registry already
/// inferred it once for side effects, and arity is pre-validated.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    match ty {
        PhpType::Array(_) | PhpType::AssocArray { .. } => Ok(ty),
        gradual
            if crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(
                &gradual,
            ) =>
        {
            Ok(PhpType::Array(Box::new(PhpType::Mixed)))
        }
        _ => Err(CompileError::new(
            cx.span,
            "array_unique() argument must be array",
        )),
    }
}
