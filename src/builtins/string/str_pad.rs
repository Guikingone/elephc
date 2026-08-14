//! Purpose:
//! Home of the PHP `str_pad` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Accepts required `string` and `length` params, plus optional `pad_string`
//!   and `pad_type` params with PHP-compatible defaults.


builtin! {
    contract: "str_pad",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StrPad,
    ),
}
