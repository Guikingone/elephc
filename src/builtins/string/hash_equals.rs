//! Purpose:
//! Home of the PHP `hash_equals` builtin: single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook is needed: `returns: Bool` expresses the return type inline and no
//!   bridge library is required (this is a pure timing-safe byte comparison).
//! - Arity (exactly 2 args) is validated by the registry.


builtin! {
    contract: "hash_equals",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::HashEquals,
    ),
}
