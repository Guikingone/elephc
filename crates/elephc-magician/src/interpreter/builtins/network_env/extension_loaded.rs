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

/// Eval's compile-time-known set of "loaded" PHP extensions for `extension_loaded()`.
///
/// Most entries mirror AOT's `CORE_LOADED_EXTENSIONS`. BCMath deliberately differs: Magician
/// always implements every `bc*` function, so eval always reports `bcmath`; AOT reports it only
/// when `elephc_bcmath` is linked through static detection or `--with-bcmath`.
///
/// The native backend also reports the other bridge
/// staticlibs it links (e.g. `PDO`, `hash`, `openssl`), but the eval interpreter runs at compile
/// time with no AOT link manifest and therefore does not expose those extensions.
/// `extension_loaded('PDO')` is thus `false` under eval even when the surrounding program is
/// compiled `--with-pdo`.
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
