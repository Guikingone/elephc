//! Purpose:
//! Home of the internal `__elephc_hash_ctx_final` builtin: the raw incremental-hash
//! finalization that the compiler-injected hash prelude wraps as PHP's `hash_final()`.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//! - The elephc-PHP body of `hash_final()` in `crate::hash_prelude`.
//!
//! Key details:
//! - See `__elephc_hash_ctx_init` for why the hash-context builtins are internal.
//! - `$binary` selects raw digest bytes over lowercase hex, exactly as PHP's `hash_final()`.
//! - The runtime helper rejects an already-finalized context with PHP's exact `\TypeError`
//!   *before* touching the context, so a rejected call has no side effect.
//! - Arity (1–2 args) is validated by the registry's `check_arity` before the hook fires.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "__elephc_hash_ctx_final",
    area: String,
    params: [context: Mixed, binary: Bool = DefaultSpec::Bool(false)],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::HashFinal,
    ),
    summary: "Finalizes a raw incremental hashing context for the hash prelude.",
    internal: true,
}
