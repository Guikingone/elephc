//! Purpose:
//! Eval-interpreter implementations of the five OPcache file/script functions
//! `opcache_is_script_cached()`, `opcache_invalidate()`, `opcache_compile_file()`,
//! `opcache_is_script_cached_in_file_cache()`, and `opcache_jit_blacklist()`.
//! Each mirrors the native prelude's disabled/CLI-default behavior: the eval interpreter
//! has no runtime SAPI and evaluates at compile time (CLI default), where the OPcache
//! *cache* is disabled — so they report the disabled result, exactly like
//! `php script.php`:
//! - `opcache_is_script_cached` → `false` (empty-cache interim; nothing is ever cached
//!   yet, disabled or not — matches native).
//! - `opcache_invalidate` → `false` (disabled cache; reference PHP returns `false` for
//!   any path when OPcache is off, even an existing file).
//! - `opcache_compile_file` → `false` (disabled; reference also emits an `E_NOTICE`, which
//!   the eval const-folder has no channel for — see below).
//! - `opcache_is_script_cached_in_file_cache` → `false` (php-src gates the whole body on
//!   `opcache.file_cache` being set, and that directive is registered with a C NULL
//!   default, so an unconfigured reference PHP returns `false` for every path — VERIFIED
//!   on PHP 8.5.6. Elephc has no file cache at all, so `false` is also its terminal
//!   answer, and it is the same disabled result the three siblings produce).
//! - `opcache_jit_blacklist` → `NULL` (a `void` function; php-src's body only mutates the
//!   JIT blacklist behind `#ifdef HAVE_JIT`, so a no-op returning null is the whole
//!   observable behavior — VERIFIED on PHP 8.5.6).
//!
//! Called from:
//! - `crate::interpreter::expressions::calls::eval_call` (direct dispatch).
//! - `crate::interpreter::builtins::registry::dispatch::eval_builtin_with_values`
//!   (dynamic-callable / by-values dispatch).
//! - `crate::interpreter::builtins::symbols::function_exists` (existence probe).
//!
//! Key details:
//! - All five are prelude-provided on the native side (real PHP functions whose bodies
//!   are baked from the compile-time enabled constant), NOT checker catalog builtins, so
//!   they must NOT be PHP-visible eval builtins either (that would break
//!   `builtin_parity_tests`, which require the two PHP-visible builtin sets to agree).
//!   They are dispatched as plain runtime handlers and made visible to `function_exists`
//!   through a small allowlist, exactly like `opcache_reset` / `opcache_get_status`.
//! - The disabled default is derived from the shared `opcache_cache_enabled` state
//!   function (`is_web_sapi = false`) rather than hard-coded, so it tracks the directive
//!   table verbatim (for the CLI default that is `opcache.enable_cli`, false). The
//!   state/directives files are shared verbatim from `src/opcache/` via `#[path]`
//!   includes (as sibling modules so `state`'s `use super::directives` resolves), giving a
//!   single source of truth across the two crates with no drift.
//! - Arity, matching PHP and the native prelude signatures: `is_script_cached` takes
//!   exactly 1 argument, `invalidate` takes 1 or 2, `compile_file` takes exactly 1,
//!   `is_script_cached_in_file_cache` takes exactly 1, and `jit_blacklist` takes exactly 1
//!   (VERIFIED on PHP 8.5.6 via `ReflectionFunction`: one required parameter named
//!   `closure`, typed `Closure`, return type `void`). Anything else is a runtime fatal.
//! - `opcache_compile_file`'s disabled `E_NOTICE`: the eval interpreter is a compile-time
//!   const-folder with no notice-level diagnostic channel (`RuntimeValueOps` exposes only
//!   a warning sink, at the wrong severity), and a const-fold should not synthesize a
//!   runtime notice anyway. Eval therefore returns `false` with no diagnostic; the notice
//!   is the native runtime's responsibility (rendered to STDERR by the prelude body). This
//!   mirrors how the sibling `opcache_reset` / `opcache_get_status` eval handlers are pure
//!   value producers.

use super::*;

// `state` only reads `opcache_directives`/`DirectiveValue` from this shared file, so the
// version-string/product-name items are unused in this crate; they are exercised by the
// native `opcache_get_configuration` copy and by this file's own tests.
#[path = "../../../../../../src/opcache/directives.rs"]
#[allow(dead_code)]
mod directives;

#[path = "../../../../../../src/opcache/state.rs"]
mod state;

use state::opcache_cache_enabled;

/// Returns whether `name` (already lowercased and unqualified) is one of the five OPcache
/// file/script functions, so `function_exists` reports it as existing even though none is a
/// PHP-visible eval builtin.
pub(in crate::interpreter) fn eval_opcache_file_function_exists(name: &str) -> bool {
    matches!(
        name,
        "opcache_is_script_cached"
            | "opcache_invalidate"
            | "opcache_compile_file"
            | "opcache_is_script_cached_in_file_cache"
            | "opcache_jit_blacklist"
    )
}

/// Builds the shared disabled/CLI-default result: the OPcache cache is off under eval, so
/// all three file functions return `false`. Derived from the shared state function so it
/// tracks the directive table rather than a hard-coded literal.
fn eval_opcache_file_disabled_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let enabled = opcache_cache_enabled(crate::eval_php_profile::eval_php_version_id(), false);
    // Disabled cache (the eval/CLI default): every file function's correct result is
    // `false` — nothing is cached, invalidate reports not-found, compile cannot run.
    debug_assert!(!enabled, "eval reports the CLI default, which is disabled");
    values.bool_value(enabled)
}

/// Evaluates a direct `opcache_is_script_cached($filename)` call from an eval fragment.
/// Exactly one argument is required; the argument does not change the empty-cache result.
pub(in crate::interpreter) fn eval_opcache_is_script_cached_call(
    args: &[EvalCallArg],
    _context: &mut ElephcEvalContext,
    _scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if args.len() != 1 {
        return Err(EvalStatus::RuntimeFatal);
    }
    eval_opcache_is_script_cached_result(values)
}

/// Builds the `opcache_is_script_cached()` return value: `false` (empty-cache interim).
pub(in crate::interpreter) fn eval_opcache_is_script_cached_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_opcache_file_disabled_result(values)
}

/// Evaluates a direct `opcache_invalidate($filename, $force = false)` call from an eval
/// fragment. One or two arguments are accepted; under the disabled eval cache the result
/// is `false` regardless of the path or the `$force` flag.
pub(in crate::interpreter) fn eval_opcache_invalidate_call(
    args: &[EvalCallArg],
    _context: &mut ElephcEvalContext,
    _scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if args.is_empty() || args.len() > 2 {
        return Err(EvalStatus::RuntimeFatal);
    }
    eval_opcache_invalidate_result(values)
}

/// Builds the `opcache_invalidate()` return value: `false` (disabled eval cache).
pub(in crate::interpreter) fn eval_opcache_invalidate_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_opcache_file_disabled_result(values)
}

/// Evaluates a direct `opcache_compile_file($filename)` call from an eval fragment.
/// Exactly one argument is required. Under the disabled eval cache the result is `false`;
/// the reference `E_NOTICE` is not synthesized here (see the module docblock).
pub(in crate::interpreter) fn eval_opcache_compile_file_call(
    args: &[EvalCallArg],
    _context: &mut ElephcEvalContext,
    _scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if args.len() != 1 {
        return Err(EvalStatus::RuntimeFatal);
    }
    eval_opcache_compile_file_result(values)
}

/// Builds the `opcache_compile_file()` return value: `false` (disabled eval cache, no
/// runtime compiler). No notice is emitted (the eval const-folder has no notice channel).
pub(in crate::interpreter) fn eval_opcache_compile_file_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_opcache_file_disabled_result(values)
}

/// Evaluates a direct `opcache_is_script_cached_in_file_cache($filename)` call from an eval
/// fragment. Exactly one argument is required; the path does not change the result, because
/// reference PHP gates the whole body on `opcache.file_cache` being set and that directive
/// is registered with a C NULL default — so an unconfigured reference PHP answers `false`
/// for every path (VERIFIED on PHP 8.5.6), and elephc has no file cache at all.
pub(in crate::interpreter) fn eval_opcache_is_script_cached_in_file_cache_call(
    args: &[EvalCallArg],
    _context: &mut ElephcEvalContext,
    _scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if args.len() != 1 {
        return Err(EvalStatus::RuntimeFatal);
    }
    eval_opcache_is_script_cached_in_file_cache_result(values)
}

/// Builds the `opcache_is_script_cached_in_file_cache()` return value: `false` (no file
/// cache is configured under eval, the same disabled result the sibling file functions
/// produce).
pub(in crate::interpreter) fn eval_opcache_is_script_cached_in_file_cache_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_opcache_file_disabled_result(values)
}

/// Evaluates a direct `opcache_jit_blacklist($closure)` call from an eval fragment. Exactly
/// one argument is required (VERIFIED on PHP 8.5.6: one required parameter named `closure`,
/// typed `Closure`). The function is declared `void`, so the call evaluates to PHP `NULL`.
pub(in crate::interpreter) fn eval_opcache_jit_blacklist_call(
    args: &[EvalCallArg],
    _context: &mut ElephcEvalContext,
    _scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if args.len() != 1 {
        return Err(EvalStatus::RuntimeFatal);
    }
    eval_opcache_jit_blacklist_result(values)
}

/// Builds the `opcache_jit_blacklist()` return value: PHP `NULL`. Reference PHP only mutates
/// the JIT blacklist behind `#ifdef HAVE_JIT` and returns `void`, which `var_export`s as
/// `NULL` (VERIFIED on PHP 8.5.6); elephc has no JIT, so the no-op is the whole behavior.
pub(in crate::interpreter) fn eval_opcache_jit_blacklist_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    values.null()
}
