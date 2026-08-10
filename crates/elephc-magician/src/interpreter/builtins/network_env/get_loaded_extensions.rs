//! Purpose:
//! Eval registry entry and implementation for `get_loaded_extensions`.
//!
//! Called from:
//! - `crate::interpreter::builtins::network_env` direct and by-value dispatch.
//!
//! Key details:
//! - Returns a compile-time-known list of extension-name strings, matching native codegen.
//! - The optional `$zend_extensions` flag selects the Zend extension list; it defaults to false.

use super::*;

/// Regular (non-Zend) extension list returned by `get_loaded_extensions(false)`.
///
/// KEEP IN SYNC with `src/codegen/lower_inst/builtins.rs` (`CORE_LOADED_EXTENSIONS`).
const CORE_LOADED_EXTENSIONS: &[&str] = &[
    "Core",
    "standard",
    "SPL",
    "json",
    "pcre",
    "date",
    "ctype",
    "mbstring",
    "Reflection",
    "Zend OPcache",
];

/// Zend extension list returned by `get_loaded_extensions(true)`.
///
/// KEEP IN SYNC with `src/codegen/lower_inst/builtins.rs` (`ZEND_LOADED_EXTENSIONS`).
const ZEND_LOADED_EXTENSIONS: &[&str] = &["Zend OPcache"];

eval_builtin! {
    contract: "get_loaded_extensions",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

/// Evaluates PHP `get_loaded_extensions($zend_extensions = false)` over its eval expressions.
pub(in crate::interpreter) fn eval_builtin_get_loaded_extensions(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let zend_extensions = match args {
        [] => false,
        [flag] => {
            let flag = eval_expr(flag, context, scope, values)?;
            values.truthy(flag)?
        }
        _ => return Err(EvalStatus::RuntimeFatal),
    };
    eval_get_loaded_extensions_result(zend_extensions, values)
}

/// Builds the extension-name array for an already-resolved `$zend_extensions` flag.
pub(in crate::interpreter) fn eval_get_loaded_extensions_result(
    zend_extensions: bool,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let set = if zend_extensions {
        ZEND_LOADED_EXTENSIONS
    } else {
        CORE_LOADED_EXTENSIONS
    };
    let mut names = values.string_array_new(set.len().max(1))?;
    for name in set {
        names = values.string_array_push(names, name)?;
    }
    Ok(names)
}
