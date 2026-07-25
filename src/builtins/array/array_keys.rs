//! Purpose:
//! Home of the PHP `array_keys` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` reproduces the legacy return-type rule: an indexed array yields
//!   `Array<Int>` (positional keys) while an associative array yields `Array<key>`.
//!   A gradual array boundary yields `Array<Mixed>` because its runtime key shape
//!   is unknown. A check hook is required because the return type depends on the
//!   inferred argument type, which the `builtin!` `returns:` field cannot express.
//! - Arity (exactly 1 argument) is validated by the registry's `check_arity` before
//!   the hook fires; the inline arity check from the legacy arm is not reproduced here.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_keys",
    area: Array,
    params: [array: Mixed],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayKeys,
    ),
    summary: "Returns all the keys of an array.",
    php_manual: "https://www.php.net/manual/en/function.array-keys.php",
}

/// Returns the key-array type for an `array_keys` call.
///
/// An indexed array produces `Array<Int>`; an associative array produces
/// `Array<key>`. `Mixed` and unions containing an array produce `Array<Mixed>` and
/// defer the concrete check to the EIR runtime boundary. Other argument types are
/// rejected. The argument is re-inferred here to drive the return type; the registry
/// already inferred it once for side effects, and arity is pre-validated.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    match ty {
        PhpType::Array(_) => Ok(PhpType::Array(Box::new(PhpType::Int))),
        PhpType::AssocArray { key, .. } => Ok(PhpType::Array(key)),
        t if crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&t) => {
            Ok(PhpType::Array(Box::new(PhpType::Mixed)))
        }
        _ => Err(CompileError::new(
            cx.span,
            "array_keys() argument must be array",
        )),
    }
}
