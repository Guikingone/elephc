//! Purpose:
//! Implements PHP's `get_defined_vars()` for the eval interpreter.
//!
//! Called from:
//! - `crate::interpreter::expressions::calls::eval_call`, ahead of the undefined-function refusal.
//!
//! Key details:
//! - Like `func_get_args()`, this is a scope intrinsic and not a shared-catalog builtin: the answer
//!   is the RUNNING scope's variables, which only the interpreter's call site can see, so no
//!   contract entry is added.
//! - The result is an associative array of every visible local, in the order the variables were
//!   last bound, with each value copied (arrays and strings share storage copy-on-write).
//!   `$this` is not a variable in php's view and is never included. Unset markers are skipped.
//! - Laravel's compiled Blade views call it for `@include`
//!   (`array_diff_key(get_defined_vars(), ['__data' => 1, '__path' => 1])`).

use super::super::super::*;

/// Names the interpreter answers itself as scope intrinsics rather than looking them up.
pub(in crate::interpreter) fn eval_is_get_defined_vars_intrinsic(name: &str) -> bool {
    name.trim_start_matches('\\').eq_ignore_ascii_case("get_defined_vars")
}

/// Evaluates `get_defined_vars()`.
pub(in crate::interpreter) fn eval_get_defined_vars_intrinsic(
    args: &[EvalCallArg],
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !args.is_empty() {
        return Err(EvalStatus::RuntimeFatal);
    }
    let visible = scope.visible_entries_in_binding_order();
    let mut array = values.assoc_new(visible.len())?;
    for (name, cell) in visible {
        if name == "this" {
            continue;
        }
        let key = values.string_bytes_value(name.as_bytes())?;
        let value = values.retain(cell)?;
        array = values.array_set(array, key, value)?;
    }
    Ok(array)
}
