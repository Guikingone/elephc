//! Purpose:
//! Home of the PHP `realpath_cache_size` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `realpath_cache_size` is a pure-data builtin whose
//!   return type (`Int`) is fully determined by its declaration.
//! - `arity_error` is overridden to preserve the legacy message
//!   "realpath_cache_size() takes exactly 0 arguments" (the registry default for
//!   0-arg builtins produces "takes no arguments").


builtin! {
    contract: "realpath_cache_size",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::RealpathCacheSize,
    ),
}
