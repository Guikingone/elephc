//! Purpose:
//! Home of the PHP `diskfreespace` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `diskfreespace` is an alias for `disk_free_space`.

builtin! {
    name: "diskfreespace",
    area: Io,
    params: [directory: Str],
    returns: Float,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::DiskFreeSpace,
    ),
    summary: "Returns available space in filesystem or disk partition (alias of disk_free_space).",
    php_manual: "function.disk-free-space",
}