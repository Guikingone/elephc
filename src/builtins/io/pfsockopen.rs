//! Purpose:
//! Home of the PHP `pfsockopen` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` validates that `error_code` (arg[2]) and `error_message` (arg[3]), if provided,
//!   are plain variables (they are written by reference). Returns `Union(stream_resource, Bool)`.
//! - Shares the same params, check logic, and lower target as `fsockopen`; `cx.name` is used
//!   in error messages so diagnostics name `pfsockopen` correctly.
//! - Arguments are pre-inferred by the registry before the hook runs.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "pfsockopen",
    area: Io,
    params: [
        hostname: Str,
        port: Int,
        ref error_code: Mixed = DefaultSpec::Null,
        ref error_message: Mixed = DefaultSpec::Null,
        timeout: Mixed = DefaultSpec::Null
    ],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Pfsockopen,
    ),
    summary: "Open persistent Internet or Unix domain socket connection.",
    php_manual: "function.pfsockopen",
}

/// Validates ref output params are plain variables, then returns `Union(stream_resource, Bool)`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    super::connect_error_params::check_connect_error_params(cx, 2, 3)
}
