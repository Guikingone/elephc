//! Purpose:
//! PHP's incremental-hashing object surface — the `HashContext` class plus the
//! `hash_init` / `hash_update` / `hash_final` / `hash_copy` functions — implemented
//! in elephc-PHP on top of the internal `__elephc_hash_ctx_*` builtins. PHP 8
//! migrated these from a resource to an object (the same migration GD made with
//! `GdImage`), and this prelude is what makes `hash_init()` return a real object
//! that `is_object`, `get_class`, `instanceof`, and `var_dump` all agree about.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`,
//!   after include resolution and before name resolution.
//!
//! Key details:
//! - WHY A PRELUDE AND NOT A NATIVE CLASS. Every hash-context `RuntimeFnId` declares
//!   `BuiltinRequirement::Bridge("elephc_crypto")` (`crate::ir::runtime_fn`), so the
//!   surface is bridge-gated: a program that never hashes must neither declare
//!   `HashContext` nor link `-lelephc_crypto`. Injecting only on demand preserves that,
//!   and the whole feature then compiles through the ordinary class/function pipeline
//!   with NO new assembly — so both architectures get it at the same time.
//! - WHY THE BUILTINS WERE RENAMED. A prelude function cannot shadow a builtin of the
//!   same name (`Cannot redeclare built-in function`), so the four raw builtins became
//!   `internal: true` `__elephc_hash_ctx_*` (see
//!   `crate::builtins::string::__elephc_hash_ctx_init`) and the PHP names are these
//!   wrappers.
//! - OWNERSHIP: the object holds the Mixed context cell in `$__elephc_ctx` and adds NO
//!   retain/release of its own. That cell already owns the native `elephc_crypto`
//!   context: the standard object-free path (`__rt_heap_free` → `object_free_deep`)
//!   releases property storage, which reaches `__rt_mixed_free_deep` and thence
//!   `__rt_hash_ctx_free`. `hash_copy()` boxes a *fresh* native context into a *fresh*
//!   cell in a *fresh* object, so each object frees exactly one context. There is
//!   deliberately no `__destruct`: adding one would introduce a second free path.
//! - `$__elephc_ctx` is public for the same reason `GdImage::$handle` is — the free
//!   functions read it — but it is hidden from `var_dump` because `__debugInfo()` below
//!   projects only `algo`, exactly as php-src's `HashContext` does.
//! - EVERY wrapper BINDS `$context->__elephc_ctx` TO A LOCAL (`$raw`) BEFORE CALLING.
//!   This is not style. Passing a `mixed` object property *inline* as a call argument
//!   trips a pre-existing codegen leak: the boxed temporary produced by the property
//!   read is never released, costing one heap block per call. It is not hash-specific
//!   and reproduces with no hashing at all —
//!   `class R { public mixed $m = null; } function f(R $r): string { return gettype($r->m); }`
//!   leaks one block per `f()` call under `--heap-debug`, while the identical body with
//!   `$v = $r->m;` first is `allocs == frees`. Binding to a local sidesteps it, so these
//!   wrappers are leak-free today; if that codegen bug is ever fixed the locals become
//!   redundant but stay harmless. `tests/codegen/runtime_gc/resource_scope_cleanup.rs`
//!   pins the clean-heap property so a careless "simplification" back to the inline form
//!   fails a test instead of silently leaking.
//! - The `flags`/`key` HMAC-streaming parameters of PHP's `hash_init()` remain
//!   unsupported (they were already blocked by the builtin's `max_args`). The wrapper
//!   keeps the one-parameter signature so passing them stays a COMPILE-TIME error
//!   (`Function 'hash_init' expects 1 arguments, got 3`) rather than being silently
//!   accepted. Use `hash_hmac()` for HMAC.
//! - `__serialize()` THROWS rather than returning a reduced array. PHP's `HashContext`
//!   really is serializable — it emits its full internal digest state
//!   (`O:11:"HashContext":5:{...}`) and round-trips — and elephc cannot reproduce that
//!   from the opaque bridge handle. Emitting a plausible-looking but state-free
//!   `O:11:"HashContext":1:{s:4:"algo";...}` would be the worst outcome: it looks like
//!   a serialized context and silently is not one. Failing loudly is the honest
//!   divergence, and `tests/resource_id_and_hash_context_tests.rs` pins it as a
//!   divergence rather than as parity.

mod detect;

/// The elephc-PHP hash prelude: the `HashContext` class and the four PHP functions
/// that produce and consume it.
pub(crate) const HASH_PRELUDE_SRC: &str = r#"<?php

final class HashContext {
    public string $algo = '';
    public mixed $__elephc_ctx = null;

    private function __construct() {}

    public static function __elephc_wrap(string $algo, mixed $raw): HashContext {
        $ctx = new self();
        $ctx->algo = $algo;
        $ctx->__elephc_ctx = $raw;
        return $ctx;
    }

    public function __debugInfo(): array {
        return ['algo' => $this->algo];
    }

    public function __serialize(): array {
        throw new \Exception("Serialization of 'HashContext' is not supported: the native hashing state cannot be captured");
    }
}

function hash_init(string $algo): HashContext {
    return HashContext::__elephc_wrap($algo, __elephc_hash_ctx_init($algo));
}

function hash_update(HashContext $context, string $data): bool {
    $raw = $context->__elephc_ctx;
    return __elephc_hash_ctx_update($raw, $data);
}

function hash_final(HashContext $context, bool $binary = false): string {
    $raw = $context->__elephc_ctx;
    return __elephc_hash_ctx_final($raw, $binary);
}

function hash_copy(HashContext $context): HashContext {
    $raw = $context->__elephc_ctx;
    return HashContext::__elephc_wrap($context->algo, __elephc_hash_ctx_copy($raw));
}
"#;

/// Injects the hash prelude when the program references the incremental-hashing
/// surface, leaving every other program untouched.
///
/// `force` comes from an explicit opt-in (the codegen harness); otherwise the
/// decision is `detect::program_uses_hash_context`. The prelude carries only
/// declarations, so prepending it is order-independent — PHP hoists them.
pub fn inject_if_used(
    program: crate::parser::ast::Program,
    force: bool,
    inventory: &mut crate::optimize::reachability::PreludeInventory,
) -> crate::parser::ast::Program {
    if !force && !detect::program_uses_hash_context(&program) {
        return program;
    }
    let tokens = crate::lexer::tokenize(HASH_PRELUDE_SRC).expect("hash prelude must tokenize");
    let mut combined = crate::parser::parse_internal(&tokens).expect("hash prelude must parse");
    inventory.record_program("hash", &combined);
    combined.extend(program);
    combined
}
