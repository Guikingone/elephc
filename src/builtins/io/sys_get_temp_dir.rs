//! Purpose:
//! Home of the PHP `sys_get_temp_dir` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook: `sys_get_temp_dir` is a pure-data builtin whose `Str` return
//!   type is fully determined by its declaration. The registry common path enforces
//!   its 0-argument arity before falling back to `returns`.


builtin! {
    contract: "sys_get_temp_dir",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::SysGetTempDir,
    ),
}
