//! Purpose:
//! Home of the PHP `stream_context_set_option` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the common registry path infers all arguments and returns `Bool`.
//!   PHP accepts two call shapes — (ctx, options_array) or (ctx, wrapper, option, value) —
//!   both accepted inertly.


builtin! {
    contract: "stream_context_set_option",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamContextSetOption,
    ),
}
