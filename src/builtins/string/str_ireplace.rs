//! Purpose:
//! Home of the PHP `str_ireplace` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - The declared signature includes an optional `count` param, but `max_args: 3`
//!   caps arity so only three arguments are accepted, matching PHP's practical use.


builtin! {
    contract: "str_ireplace",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StrIreplace,
    ),
}
