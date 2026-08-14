//! Purpose:
//! Home of the PHP `number_format` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Accepts a required `num` float and optional `decimals`, `decimal_separator`,
//!   and `thousands_separator` params with PHP-compatible defaults.


builtin! {
    contract: "number_format",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::NumberFormat,
    ),
}
