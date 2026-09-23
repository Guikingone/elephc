//! Purpose:
//! Eval registry entries and implementations for `memory_get_usage` and
//! `memory_get_peak_usage`.
//!
//! Called from:
//! - `crate::interpreter::builtins::network_env` direct and by-value dispatch.
//!
//! Key details:
//! - The NUMBER IS NOT COMPUTED HERE. Both forward to `RuntimeValueOps::memory_usage_bytes`,
//!   which the runtime adapter answers from the generated runtime's `_gc_live` / `_gc_peak` /
//!   `_heap_off` counters — the same words the compiled lowering of these two builtins reads.
//!   An eval-local answer (a Rust-side allocation count, or process RSS) would describe a
//!   different heap than the program's, and the two execution paths would disagree about the
//!   same program. See `src/builtins/system/memory_usage.rs` for what the counters mean and
//!   where they diverge from php.
//! - `$real_usage` is read for truthiness exactly like php's weak-mode bool parameter, so
//!   `memory_get_usage(1)` and `memory_get_usage("x")` select the real figure as php does.

use super::*;

eval_builtin! {
    contract: "memory_get_usage",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

eval_builtin! {
    contract: "memory_get_peak_usage",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

/// Evaluates PHP `memory_get_usage($real_usage = false)`.
pub(in crate::interpreter) fn eval_builtin_memory_get_usage(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_builtin_memory_reporter(false, args, context, scope, values)
}

/// Evaluates PHP `memory_get_peak_usage($real_usage = false)`.
pub(in crate::interpreter) fn eval_builtin_memory_get_peak_usage(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_builtin_memory_reporter(true, args, context, scope, values)
}

/// Shared direct-call body: evaluate the optional flag, then report the byte count.
fn eval_builtin_memory_reporter(
    peak: bool,
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [] => eval_memory_usage_result(peak, None, values),
        [real_usage] => {
            let real_usage = eval_expr(real_usage, context, scope, values)?;
            eval_memory_usage_result(peak, Some(real_usage), values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Returns the reported byte count as a boxed int, reading the flag for truthiness.
pub(in crate::interpreter) fn eval_memory_usage_result(
    peak: bool,
    real_usage: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let real_usage = real_usage
        .map(|value| values.truthy(value))
        .transpose()?
        .unwrap_or(false);
    let bytes = values.memory_usage_bytes(peak, real_usage)?;
    values.int(bytes)
}
