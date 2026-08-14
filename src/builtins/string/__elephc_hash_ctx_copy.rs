//! Purpose:
//! Home of the internal `__elephc_hash_ctx_copy` builtin: the raw incremental-hash
//! clone that the compiler-injected hash prelude wraps as PHP's `hash_copy()`.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//! - The elephc-PHP body of `hash_copy()` in `crate::hash_prelude`.
//!
//! Key details:
//! - See `__elephc_hash_ctx_init` for why the hash-context builtins are internal.
//! - The clone is an independent context: feeding the source after copying does not
//!   affect the copy, and finalizing the source leaves the copy usable.
//! - The returned Mixed cell owns its own native context; the prelude wraps it in a
//!   fresh `HashContext`, so each object frees exactly one native context.
//! - Arity (exactly 1 arg) is validated by the registry's `check_arity` before the hook fires.

builtin! {
    contract: "__elephc_hash_ctx_copy",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::HashCopy,
    ),
}
