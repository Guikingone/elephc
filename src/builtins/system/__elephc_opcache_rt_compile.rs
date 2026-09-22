//! Purpose:
//! Home of the internal `__elephc_opcache_rt_compile` builtin: the runtime script cache's
//! compile operation, reached by path. Answers `0` for a file that does not parse.
//!
//! Called from:
//! - The injected OPcache prelude, so a NATIVELY compiled `opcache_*` body answers about the
//!   dynamic tier instead of only the compile-time manifest.
//!
//! Key details:
//! - `internal: true`: never PHP-visible, so `function_exists()` does not report it.
//! - Takes the path as a PHP string, which is what makes the native and eval surfaces agree
//!   on a DYNAMICALLY included file — the manifest cannot answer for one.
//! - The lowering folds the call to `0` when this binary has no eval bridge, and for THIS
//!   name that fold is a KNOWN DIVERGENCE rather than the right answer. Its siblings
//!   `_is_cached` and `_discard` answer `false` truthfully there — nothing can have been
//!   cached without a dynamic tier. `opcache_compile_file()`'s job is to CREATE the entry,
//!   so reference PHP answers `true` and caches the file whether or not the program uses
//!   `eval()`. MEASURED: reference `c1=1 c2=1`, elephc `c1=0 c2=0`.
//!
//!   Closing it means linking the interpreter when the program calls this function, which is
//!   the correct trade — measured at 2.5 MB against a 69 KB disabled build, and it lands only
//!   on programs that both call it AND enable OPcache, since the prelude's own gate
//!   short-circuits before the call is emitted otherwise.
//!
//!   It is NOT closed here because the switch is not clean: `required_runtime_features
//!   .eval_bridge` links the archive, while a separate `module_uses_eval` predicate —
//!   duplicated across the `eval_*_helpers` families — decides whether the callback symbols
//!   that archive references are emitted. Raising the first without the second produces a
//!   binary that does not link (`__elephc_eval_reflection_attribute_new` and friends
//!   undefined). Separating "links the interpreter" from "the program contains eval()" is
//!   its own change.

builtin! {
    contract: "__elephc_opcache_rt_compile",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ElephcOpcacheRtCompile,
    ),
}
