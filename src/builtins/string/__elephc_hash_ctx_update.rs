//! Purpose:
//! Home of the internal `__elephc_hash_ctx_update` builtin: the raw incremental-hash
//! update that the compiler-injected hash prelude wraps as PHP's `hash_update()`.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//! - The elephc-PHP body of `hash_update()` in `crate::hash_prelude`.
//!
//! Key details:
//! - See `__elephc_hash_ctx_init` for why the hash-context builtins are internal.
//! - `$context` is the opaque Mixed cell held in `HashContext::$__elephc_ctx`, never a
//!   user-visible value. The runtime helper rejects an already-finalized context with
//!   PHP's exact `\TypeError`, so that guard survives the object migration untouched.
//! - Arity (exactly 2 args) is validated by the registry's `check_arity` before the hook fires.

builtin! {
    name: "__elephc_hash_ctx_update",
    area: String,
    params: [context: Mixed, data: Str],
    returns: Bool,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::HashUpdate,
    ),
    summary: "Pumps data into a raw incremental hashing context for the hash prelude.",
    internal: true,
}
