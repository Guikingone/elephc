//! Purpose:
//! Home of PHP's `preg_grep` builtin and its key-preserving regex-filter semantics.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The checked result uses associative mixed storage because original numeric and string keys are
//!   preserved rather than compacted.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "preg_grep",
    area: System,
    params: [pattern: Str, array: Mixed, flags: Int = DefaultSpec::Int(0)],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::PregGrep,
    ),
    summary: "Returns entries whose values match a regular expression while preserving keys.",
    php_manual: "function.preg-grep",
}

/// Returns the key-preserving hash layout materialized by the regex filter runtime.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(PhpType::Mixed),
    })
}
