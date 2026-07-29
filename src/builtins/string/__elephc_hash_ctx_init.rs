//! Purpose:
//! Home of the internal `__elephc_hash_ctx_init` builtin: the raw incremental-hash
//! constructor that the compiler-injected hash prelude wraps as PHP's `hash_init()`.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//! - The elephc-PHP body of `hash_init()` in `crate::hash_prelude`.
//!
//! Key details:
//! - WHY IT IS INTERNAL. PHP 8's `hash_init()` returns a `HashContext` *object*, not a
//!   resource. elephc models that object as an ordinary elephc-PHP class declared by
//!   `crate::hash_prelude`, and a prelude function cannot shadow a builtin of the same
//!   name (`Cannot redeclare built-in function`). Renaming the builtin frees `hash_init`
//!   for the prelude wrapper. `internal: true` keeps the raw name out of the builtin
//!   catalog, out of `function_exists()`, out of the generated docs, and out of the
//!   elephc/magician parity snapshots.
//! - The returned value is the opaque Mixed context cell the prelude stores in
//!   `HashContext::$__elephc_ctx`. It never reaches user code and consumes no PHP
//!   resource id (see `runtime::resource_ids`), so a `HashContext` does not shift the
//!   ids of surrounding `fopen()` streams the way the old resource model did.
//! - Only the algorithm name is accepted. PHP's `flags`/`key` HMAC-streaming parameters
//!   are declared and rejected by the prelude wrapper, so this builtin keeps the
//!   single-argument shape the runtime helper implements.

builtin! {
    name: "__elephc_hash_ctx_init",
    area: String,
    params: [algo: Str],
    returns: Mixed,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::HashInit,
    ),
    summary: "Opens a raw incremental hashing context for the hash prelude.",
    internal: true,
}
