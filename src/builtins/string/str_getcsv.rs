//! Purpose:
//! Home of the PHP `str_getcsv` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Parses ONE CSV record out of a string. It is not `fgetcsv()` over a line: a newline
//!   between enclosures, and a newline inside an unenclosed field, are ordinary data. Only a
//!   trailing newline is structural, and php-src strips one in two separate places — the
//!   runtime helper documents the exact rule.
//! - `escape` defaults to `"\\"` as in PHP 8.4/8.5; PHP 9.0 will change it to `""`.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "str_getcsv",
    area: String,
    params: [
        string: Str,
        separator: Str = DefaultSpec::Str(","),
        enclosure: Str = DefaultSpec::Str("\""),
        escape: Str = DefaultSpec::Str("\\")
    ],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StrGetcsv,
    ),
    summary: "Parse a CSV string into an array.",
    php_manual: "function.str-getcsv",
}

/// Returns `Array<Str>`: one record's fields, always at least one element.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
