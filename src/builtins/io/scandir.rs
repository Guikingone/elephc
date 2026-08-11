//! Purpose:
//! Home of the PHP `scandir` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Array<Str>` (the directory entries). A check hook is required
//!   because the array return type cannot be expressed through the scalar `returns:`
//!   field.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "scandir",
    area: Io,
    params: [directory: Str],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Scandir,
    ),
    summary: "Lists files and directories inside the specified path.",
    php_manual: "function.scandir",
}

/// Returns `Array<Str>` reflecting that `scandir` yields directory entry names.
///
/// PHP's signature is `array|false`, but declaring the union costs more than it buys here:
/// `Mixed` makes `in_array($name, scandir($dir))` — ordinary, correct PHP — fail to compile,
/// because the array-taking builtins require a declared array. The failure case is kept
/// non-fatal instead (an empty listing rather than a crash) and the `false` divergence is
/// left for a decision that can weigh it against that cost.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
