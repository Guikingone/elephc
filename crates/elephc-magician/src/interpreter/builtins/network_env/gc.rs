//! Purpose:
//! Eval registry entries and implementations for PHP's cycle-collector controls —
//! `gc_enabled`, `gc_enable`, `gc_disable` and `gc_collect_cycles`.
//!
//! Called from:
//! - `crate::interpreter::builtins::network_env` direct and by-value dispatch.
//!
//! Key details:
//! - elephc frees by REFERENCE COUNT and has no cycle collector to turn on, so `gc_enabled()`
//!   is `false`, `gc_collect_cycles()` collects nothing and reports `0`, and the two switches
//!   are no-ops. Reporting `true` would be the more "PHP-default-like" answer and the wrong
//!   one: it would promise a collector that is not there.
//! - The family has to exist together, because callers use it as a pair — Symfony's
//!   `AbstractCloner::cloneVar` does `if ($gc = gc_enabled()) { gc_disable(); } … gc_enable();`
//!   — and a program that finds `gc_enabled()` and then no `gc_disable()` fails one line later
//!   than one that finds neither.

use super::*;

eval_builtin! {
    contract: "gc_enabled",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

eval_builtin! {
    contract: "gc_enable",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

eval_builtin! {
    contract: "gc_disable",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

eval_builtin! {
    contract: "gc_collect_cycles",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

/// Evaluates PHP `gc_enabled()`, which is always `false` without a cycle collector.
pub(in crate::interpreter) fn eval_builtin_gc_enabled(
    args: &[EvalExpr],
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    eval_gc_enabled_result(values)
}

/// Returns `false`: elephc has no circular reference collector to have enabled.
pub(in crate::interpreter) fn eval_gc_enabled_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    values.bool_value(false)
}

/// Evaluates PHP `gc_enable()` / `gc_disable()`, both no-ops here.
pub(in crate::interpreter) fn eval_builtin_gc_switch(
    args: &[EvalExpr],
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    eval_gc_switch_result(values)
}

/// Returns null: there is no collector state for either switch to change.
pub(in crate::interpreter) fn eval_gc_switch_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    values.null()
}

/// Evaluates PHP `gc_collect_cycles()`, which collects nothing here.
pub(in crate::interpreter) fn eval_builtin_gc_collect_cycles(
    args: &[EvalExpr],
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    eval_gc_collect_cycles_result(values)
}

/// Returns `0`: refcounting already freed everything it is able to free.
pub(in crate::interpreter) fn eval_gc_collect_cycles_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    values.int(0)
}
