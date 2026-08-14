//! Purpose:
//! Home of the PHP `stream_resolve_include_path` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the common registry path infers the filename argument and returns `Mixed`.
//! - `returns: Mixed` reflects the `string|false` PHP return type.


builtin! {
    contract: "stream_resolve_include_path",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamResolveIncludePath,
    ),
}
