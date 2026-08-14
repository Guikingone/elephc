//! Purpose:
//! Home of the PHP `log` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `log` is a pure-data builtin whose return type
//!   (`Float`) is fully determined by its declaration.
//! - The second parameter `base` is optional with a default of `M_E`, matching
//!   PHP's `log(num, base = M_E)` signature. The registry enforces 1-2 args.


builtin! {
    contract: "log",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Log,
    ),
}
