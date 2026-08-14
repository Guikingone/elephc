//! Purpose:
//! Home of the PHP `wordwrap` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Accepts a required `string` param plus optional `width`, `break`, and
//!   `cut_long_words` params with PHP-compatible defaults. The `break` param
//!   uses the raw identifier `r#break` because `break` is a Rust keyword.


builtin! {
    contract: "wordwrap",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Wordwrap,
    ),
}
