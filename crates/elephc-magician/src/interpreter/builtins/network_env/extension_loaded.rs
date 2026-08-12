//! Purpose:
//! Eval registry entry and implementation for `extension_loaded`.
//!
//! Called from:
//! - `crate::interpreter::builtins::network_env` direct and by-value dispatch.
//!
//! Key details:
//! - Membership is resolved against a compile-time-known extension set, matching the native
//!   codegen behavior; there is no runtime extension state in this increment.
//! - Matching is case-insensitive over the canonical extension names.

use super::*;

/// Compile-time-known set of "loaded" PHP extensions for `extension_loaded()`.
///
/// KEEP IN SYNC with the core set in `src/codegen/lower_inst/builtins.rs`
/// (`CORE_LOADED_EXTENSIONS`).
///
/// Deliberate divergence from native codegen: the native backend also reports the bridge
/// staticlibs it links (e.g. `PDO`, `hash`, `openssl`), but the eval interpreter runs at compile
/// time with no AOT link step and therefore no linked bridges, so it reports only this core set.
/// `extension_loaded('PDO')` is thus `false` under eval even when the surrounding program is
/// compiled `--with-pdo` — same rationale as eval not seeing the link manifest.
const CORE_LOADED_EXTENSIONS: &[&str] = &[
    "Core",
    "standard",
    "SPL",
    "bcmath",
    "json",
    "pcre",
    "date",
    "ctype",
    "mbstring",
    "Reflection",
    "Zend OPcache",
];

eval_builtin! {
    contract: "extension_loaded",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

/// Evaluates PHP `extension_loaded($extension)` over one eval expression.
pub(in crate::interpreter) fn eval_builtin_extension_loaded(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [extension] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let extension = eval_expr(extension, context, scope, values)?;
    eval_extension_loaded_result(extension, values)
}

/// Reports whether an already-evaluated extension name is in the known extension set.
pub(in crate::interpreter) fn eval_extension_loaded_result(
    extension: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let name = values.string_bytes(extension)?;
    let name = String::from_utf8_lossy(&name);
    values.bool_value(eval_extension_is_loaded(name.as_ref()))
}

/// Returns whether `name` is in eval's known extension set, compared case-insensitively.
///
/// The single membership predicate for the eval interpreter: `extension_loaded()` and
/// `phpversion($extension)` both go through it, so the two can never disagree — the same
/// invariant `extension_is_loaded` enforces on the native side.
pub(in crate::interpreter) fn eval_extension_is_loaded(name: &str) -> bool {
    CORE_LOADED_EXTENSIONS
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(name))
}
