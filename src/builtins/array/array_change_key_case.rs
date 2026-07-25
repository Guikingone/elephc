//! Purpose:
//! Home of the PHP `array_change_key_case` builtin and its typed runtime contract.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Indexed arrays preserve their shape because integer keys are unchanged.
//! - Associative arrays preserve their key/value types while the runtime rebuilds string keys.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_change_key_case",
    area: Array,
    params: [array: Mixed, case: Int = DefaultSpec::Int(0)],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayChangeKeyCase,
    ),
    summary: "Changes the case of all string keys in an array.",
    php_manual: "https://www.php.net/manual/en/function.array-change-key-case.php",
}

/// Preserves concrete input shape and accepts gradual array operands at runtime.
///
/// Integer keys remain unchanged and converting string-key case does not change
/// the key or value type. `Mixed` or array-containing unions return `Mixed`
/// because the runtime dispatcher preserves either the packed or associative
/// representation inside a result box.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    match ty {
        PhpType::Array(_) | PhpType::AssocArray { .. } => Ok(ty),
        gradual
            if crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(
                &gradual,
            ) =>
        {
            Ok(PhpType::Mixed)
        }
        _ => Err(CompileError::new(
            cx.span,
            "array_change_key_case() argument must be array",
        )),
    }
}
