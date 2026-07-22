//! Purpose:
//! Home of the PHP `array_reverse` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` reproduces the legacy rule: reversing preserves the array shape, so the
//!   return type is the (array-or-assoc) input type unchanged. A check hook is
//!   required both to reject non-array arguments and to echo the input type back.
//! - Arity (exactly 1 argument) is validated by the registry's `check_arity` before
//!   the hook fires; the inline arity check from the legacy arm is not reproduced here.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_reverse",
    area: Array,
    params: [array: Mixed, preserve_keys: Bool = crate::builtins::spec::DefaultSpec::Bool(false)],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayReverse,
    ),
    summary: "Returns an array with the elements in reverse order.",
    php_manual: "https://www.php.net/manual/en/function.array-reverse.php",
}

/// Returns the (shape-preserving) array type for an `array_reverse` call.
///
/// Reversing keeps the array shape, so the input array/assoc type is returned
/// unchanged. Non-array arguments are rejected. The argument is re-inferred here;
/// the registry already inferred it once for side effects, and arity is pre-validated.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    // Accept a concrete array or a gradual operand (`Mixed`/union containing an array),
    // exactly like `count`. EIR emits a runtime unbox + assert-array boundary guard, so a
    // runtime non-array still fatals (PHP-8 `TypeError`).
    if !crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&ty) {
        return Err(CompileError::new(
            cx.span,
            "array_reverse() argument must be array",
        ));
    }
    match ty {
        // A concrete array keeps its precise element type in the result.
        PhpType::Array(_) | PhpType::AssocArray { .. } => Ok(ty),
        // A `Mixed`/union operand has an unknown element type, so the result is a list of
        // `Mixed`, mirroring `array_keys`/`array_values`.
        _ => Ok(PhpType::Array(Box::new(PhpType::Mixed))),
    }
}
