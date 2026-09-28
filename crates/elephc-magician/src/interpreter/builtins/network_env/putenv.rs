//! Purpose:
//! Eval registry entry and implementation for `putenv`.
//!
//! Called from:
//! - `crate::interpreter::builtins::network_env` direct and by-value dispatch.
//!
//! Key details:
//! - Assignments mutate the host process environment for the current eval process.
//! - PHP's syntax guard raises a catchable `ValueError` before host environment APIs run.
//! - The platform calls and their status mirror the compiled `putenv()` lowering (#911): an
//!   argument containing `=` goes to `putenv(3)` with a persistent NUL-terminated copy, anything
//!   else to `unsetenv(3)`, and the result is `true` exactly when libc reports success. The
//!   environment holds C strings, so libc sees the argument up to its first NUL byte.

use super::*;

eval_builtin! {
    contract: "putenv",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

/// Evaluates PHP `putenv($assignment)` over one eval expression.
pub(in crate::interpreter) fn eval_builtin_putenv(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [assignment] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let assignment = eval_expr(assignment, context, scope, values)?;
    eval_putenv_result(assignment, context, values)
}

/// Validates and applies one `putenv()` assignment to the host environment.
pub(in crate::interpreter) fn eval_putenv_result(
    assignment: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let assignment = values.string_bytes(assignment)?;
    if assignment.is_empty() || assignment[0] == b'=' {
        return eval_throw_builtin_value_error(
            "putenv(): Argument #1 ($assignment) must have a valid syntax",
            context,
            values,
        );
    }
    values.bool_value(eval_putenv_host(&assignment))
}

/// Applies one syntax-checked assignment through libc and reports whether libc accepted it.
///
/// The `=` scan covers every byte, as the compiled lowering's does, while libc reads the copy
/// only up to its first NUL. `putenv(3)` keeps the pointer it is given as part of the
/// environment, so an accepted copy is deliberately never freed, like the compiled runtime's
/// persistent buffer; a refused one is freed at once.
fn eval_putenv_host(assignment: &[u8]) -> bool {
    let text = assignment.split(|byte| *byte == 0).next().unwrap_or_default();
    let Ok(text) = CString::new(text) else {
        return false;
    };
    if !assignment.contains(&b'=') {
        // SAFETY: `text` is NUL-terminated and outlives the call, which only reads it.
        return unsafe { libc::unsetenv(text.as_ptr()) } == 0;
    }
    let persistent = text.into_raw();
    // SAFETY: `persistent` is a NUL-terminated heap string that stays valid for as long as the
    // environment may reference it, because it is only freed when libc refused it.
    let accepted = unsafe { libc::putenv(persistent) } == 0;
    if !accepted {
        // SAFETY: libc refused the pointer, so nothing else references this allocation.
        drop(unsafe { CString::from_raw(persistent) });
    }
    accepted
}
