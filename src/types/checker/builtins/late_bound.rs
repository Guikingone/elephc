//! Purpose:
//! Curated, EXACT allowlist of extension function names elephc treats as PHP-faithful
//! late-bound undefined calls instead of a compile-time error. A call site naming one of
//! these functions compiles to a catchable `\Error` throw with PHP's exact
//! "Call to undefined function X()" message (see `crate::ir_lower::expr::late_bound_call`),
//! matching PHP's real behavior: calling an undefined function only fatals when the call
//! actually EXECUTES, so a call guarded behind `extension_loaded()`/`function_exists()` that
//! never runs costs nothing to compile.
//!
//! Called from:
//! - `crate::types::checker::functions::resolution` (`Checker::check_function_call`), which
//!   consults `is_late_bound_undefined_function` before falling back to the compile-time
//!   "Undefined function" diagnostic.
//! - `crate::ir_lower::expr::mod::lower_function_call`, which consults the same allowlist to
//!   lower a matching call to the `\Error` throw instead of an ordinary/builtin call.
//!
//! Key details:
//! - EXACT names only — deliberately NOT prefix-matched (unlike
//!   `crate::optimize::function_existence::NEVER_AVAILABLE_FUNCTION_PREFIXES`, which folds
//!   `function_exists()`/`extension_loaded()` broadly). A prefix match here would make a typo
//!   like `apcu_ftch` silently swallow into "compiles, throws at runtime" instead of the
//!   precious compile-time "Undefined function" diagnostic — jury-addendum-binding requirement.
//! - Harvested from the exact "Undefined function: ..." names produced by `--web` on
//!   `examples/symfony-app/public/index.php` (cycle 7), filtered to the extension-shaped
//!   families this project already treats as "never available"
//!   (`crate::optimize::function_existence::NEVER_AVAILABLE_FUNCTION_PREFIXES`'s families):
//!   `opcache_invalidate`, `opcache_compile_file`, `apcu_exists`, `apcu_store`, `apcu_delete`,
//!   `apcu_add`, `xdebug_is_enabled`, `xdebug_connect_to_client`, `igbinary_serialize`,
//!   `igbinary_unserialize`, `frankenphp_handle_request` (`xdebug_connect_to_client` is the
//!   xdebug-family sibling of `xdebug_is_enabled`, called doubly-guarded behind
//!   `extension_loaded('xdebug') && function_exists('xdebug_connect_to_client')` inside
//!   `Symfony\Component\Runtime\Runner\FrankenPhpWorkerRunner::run()`'s request handler closure;
//!   `apcu_add` added in the M1 easy sweep: `ApcuAdapter::clear()`
//!   only reaches it behind `!apcu_exists(...)`, and `apcu_exists` already throws before
//!   returning, so the site cannot depend on `apcu_add`'s return value).
//! - `token_get_all` was added later once its Symfony call site was verified to sit behind a
//!   guard that is genuinely DEAD under AOT: `PhpDumper::stripComments()` calls it only past
//!   `if (!\function_exists('token_get_all')) { return $source; }`, and elephc has NO
//!   `token_get_all` catalog entry, so `function_exists('token_get_all')` folds to `false`, the
//!   guard fires (early `return $source`), and the `token_get_all()` call never executes at
//!   runtime — but the checker still visits it and would otherwise reject the whole compile with
//!   "Undefined function". Late-binding it to PHP's own "Call to undefined function" `\Error` is
//!   therefore byte-faithful AND its throw is provably dead. It is a single EXACT name, so unlike
//!   a prefix family this costs no typo-detection signal.
//! - `proc_open` was deliberately NOT added, even though it too sits behind a
//!   `!\function_exists('proc_open')` guard in `Console\Terminal::readFromProcess()`. Unlike
//!   `token_get_all`, `proc_open` IS a catalog builtin (`CAMPAIGN_LEGACY_BUILTIN_FUNCTIONS`) with
//!   a signature but no EIR lowering, so `function_exists('proc_open')` folds to `true`; its
//!   guards do NOT fire, meaning a late-bound throw would be reachable (e.g. the profiler's
//!   `RequestDataCollector` path) rather than dead — a wrong runtime, not a PHP-faithful dead
//!   guard. The premise of this whole list ("dead under AOT") does not hold for `proc_open`, so
//!   it stays a loud compile-time error; the correct fix is either a real `proc_open`
//!   implementation or making it `function_exists`-invisible so its guards fire (both out of
//!   scope here).
//!   MEASURED ADDENDUM (backtrace-family cycle): `proc_open` is not merely un-lowered, it is the
//!   ONLY name in `CAMPAIGN_LEGACY_BUILTIN_FUNCTIONS` whose CALL the checker itself rejects — an
//!   exhaustive sweep compiling `<?php <name>();` for all 125 legacy catalog names produced
//!   exactly one "Undefined function" (this one), because no per-area `check_builtin` arm claims
//!   it and `Checker::check_builtin` therefore returns `Ok(None)`. That also rules out the
//!   "make it `function_exists`-invisible" half of the suggested fix as a NET win on its own:
//!   `Terminal::readFromProcess()` passes `$pipes` BY REFERENCE to `proc_open` and then reads
//!   `$pipes[1]`/`$pipes[2]` on the next lines, so the by-ref out-parameter knowledge in
//!   `crate::types::checker::inference::expr::by_ref_outputs` (`"proc_open" if index == 2`) is
//!   what currently initializes `$pipes`. Late-binding the name (or dropping it from the catalog)
//!   removes that knowledge and trades one "Undefined function" for a fresh "Undefined variable:
//!   $pipes". Only a real implementation clears this call site without relocating the error.
//! - The remaining non-extension-shaped undefined names surfaced by the same scan
//!   (`debug_backtrace`, `eval`, `next`, `highlight_file`, `extract`) stay OUT of scope: they are
//!   core PHP functions elephc genuinely lacks and are called UNCONDITIONALLY (not behind a dead
//!   guard), so keeping them a loud compile-time "Undefined function" is the honest real-gap
//!   signal. A dedicated cycle scoped each one against `php -n` 8.5.6 and against the compiler as
//!   it stands; the verdicts are recorded here so the next attempt starts from measurement rather
//!   than re-derives it. In every case, adding the NAME to `CAMPAIGN_LEGACY_BUILTIN_FUNCTIONS`
//!   would lower the `--web` error counter while relocating the failure one floor down into the
//!   EIR backend, where the `--web` run cannot see it (the run aborts at the checker) — the exact
//!   false-win shape this list exists to refuse. See the tripwire tests in
//!   `crate::types::checker::builtins::catalog`.
//!   * `debug_backtrace` — blocked on FILE IDENTITY, not just on call-stack metadata. Every frame
//!     PHP returns carries `file` and `line`, and five of the six Symfony call sites are built on
//!     those two keys: `VarDumper\...\SourceContextProvider::getContext()` reads
//!     `$trace[1]['file']`/`['line']` unqualified, while `ErrorHandler::call()`,
//!     `ErrorHandler::cleanTrace()` and `DependencyInjection\Kernel\KernelTrait` gate each frame
//!     on `isset($backtrace[$i]['file'], $backtrace[$i]['line'], ...)` and would skip every frame
//!     a file-less implementation produced. (The sixth,
//!     `DependencyInjection\ServiceLocator::createNotFoundException()`, wants `object` instead —
//!     feasible, since `$this` is in a known slot for methods.) `crate::span::Span` is line/col
//!     only and deliberately 16 bytes (a 32-byte span
//!     overflowed 2 MiB test-thread stacks), the resolver inlines every `include`/`require` into
//!     one AST, and `crate::resolver::scan_reflection_source_files` is called ONCE on the entry
//!     file (`crate::pipeline::compile`), so a declaration from an autoloaded file has no
//!     recorded path at all — which is why `ReflectionClass::getFileName()` returns PHP's `false`
//!     for them and why elephc's own `--web` diagnostics print `error[LINE:COL]` with no
//!     filename. A faithful `debug_backtrace` therefore needs, in order: (1) a complete
//!     declaration -> file map covering included/autoloaded files, (2) a shadow stack pushed in
//!     the callee prologue / popped in `crate::codegen_support::abi::frame`'s teardown with the
//!     call-site line supplied by the caller, restored across the setjmp/longjmp unwind in
//!     `crate::codegen_support::runtime::exceptions`, (3) pay-for-use gating on the
//!     `const_introspection` model in `crate::codegen_support::runtime_features`. The dormant
//!     `_exc_call_frame_top` chain is the pre-shaped hook (already fiber-aware, already walked by
//!     `__rt_exception_cleanup_frames`) but is never populated: the only writer,
//!     `lower_try_push_handler`, stores an immediate zero. `args` is out of reach regardless —
//!     `func_get_args()` works only by rewriting the CALLEE's own signature
//!     (`crate::types::checker::func_args_scan`), which cannot serve frames that did not opt in,
//!     so `DEBUG_BACKTRACE_IGNORE_ARGS` semantics are the honest ceiling. Both option constants
//!     already exist end to end (`crate::codegen_support::prescan`: `PROVIDE_OBJECT` = 1,
//!     `IGNORE_ARGS` = 2); only the function is missing.
//!   * `next` — the array internal-pointer family, and a trap. `reset`, `current` and `key` are
//!     ALREADY catalog builtins with checker arms, so `function_exists()` folds `true` for them,
//!     yet all three abort with "unsupported EIR backend feature: builtin call <name>" (verified
//!     on `$a=[1,2]; var_dump(reset($a));` and siblings). Only `end` is lowered, and it returns
//!     the last element without moving any pointer, because elephc arrays have no internal
//!     pointer: the 24-byte header built by `__rt_array_new`
//!     (`crate::codegen_support::runtime::arrays::array_new`) is length/capacity/elem_size with no
//!     cursor field. `Filesystem\Path::getLongestCommonBasePath()` — the one call site — drives
//!     `reset`/`next`/`key`/`current` in a single `for`, so registering `next` alone clears one
//!     checker error and immediately fails the same function in the backend on `reset`. Real work
//!     here is "add an internal pointer to the array representation", not "add a builtin".
//!   * `highlight_file` — needs a PHP tokenizer AND PHP's exact colorized HTML at runtime.
//!     elephc's lexer/parser exist only in the compiler and in the `elephc-magician` eval bridge,
//!     which a non-`eval` program does not link; `token_get_all` is itself late-bound precisely
//!     because elephc has no runtime tokenizer. `ErrorHandler\ErrorRenderer\HtmlErrorRenderer::
//!     fileExcerpt()` then `preg_replace`s the `<pre><code>` wrapper off and splits the emitted
//!     `<span>` tags, so anything short of PHP's real markup is worse than absent.
//!   * `extract` — genuinely impossible in elephc's model, not merely unbuilt. It materializes
//!     local variables whose NAMES are only known at runtime, and elephc rejects that class of
//!     construct by design: `$$name = 1;` is a compile error ("Variable variables (`$$name`) are
//!     not supported: variable names must be known at compile time"). Its lone call site,
//!     `HtmlErrorRenderer::include()`, does `extract($context, \EXTR_SKIP)` purely to hand
//!     variables to an `include`d template, which compounds the same problem. This one is a
//!     documented refusal rather than deferred work.
//!   `is_uploaded_file`/`move_uploaded_file` and `request_parse_body` were on this list for the
//!   same reason and have since been given REAL implementations rather than late-bound throws:
//!   `crate::upload_prelude` carries the rfc1867 upload registry the first two need (fed by the
//!   only producer of upload temp files, `crate::web_prelude`'s multipart parser), and
//!   `crate::web_prelude` implements `request_parse_body()` on top of the body parse it already
//!   performs per request.
//!   `register_shutdown_function` was ALSO in that original "out of scope" list but is now a real
//!   fix elsewhere — see `crate::name_resolver::PRELUDE_GLOBAL_FUNCTIONS` (an own-feature
//!   namespace-fallback gap, not a late-bound guard pattern: PHP's own
//!   `register_shutdown_function` genuinely exists, elephc's namespace resolver just failed to
//!   fall back to it). `get_defined_functions` and `parse_ini_file` remain OUT of scope after the
//!   M1 easy sweep too, but for a DIFFERENT reason than "not a late-bound guard pattern": both are
//!   reachable in real Symfony call sites (`UndefinedFunctionErrorEnhancer::enhance()`,
//!   `IniFileLoader::load()`) and a real (not late-bound-stub) implementation was scoped and
//!   evaluated, but deliberately deferred as a loud compile error rather than risked half-built —
//!   `get_defined_functions` needs a correct 'internal'/'user' split sourced from EIR
//!   `Function`/`FunctionFlags` (excluding methods/closures/synthetic wrapper flags and
//!   `PRELUDE_GLOBAL_FUNCTIONS` entries) plus a pay-for-use data-table gate mirroring
//!   `crate::codegen::runtime_features::RuntimeFeatures::const_introspection`'s multi-file
//!   pattern; `parse_ini_file` needs genuinely new "build a nested PHP array from parsed runtime
//!   data" machinery that must respect this codebase's ownership/COW/GC invariants — both are
//!   real, scoped feature work for a follow-up session, not a five-minute catalog entry.
//!   `headers_send` was ALSO in that original "out of scope" list, on the mistaken premise that it
//!   is a core PHP builtin elephc simply had not implemented. It is not: `headers_send()` is not a
//!   core function in any PHP release (verified against the local interpreter —
//!   `php -n -r 'var_dump(function_exists("headers_send"));'` prints `bool(false)` on PHP 8.5.6).
//!   It is provided only by SAPIs that support 1xx informational responses (FrankenPHP), which is
//!   exactly what its Symfony call site documents: `Response::sendHeaders()` writes
//!   `// skip informational responses if not supported by the SAPI` above its guard. So it is now
//!   late-bound here, for the SAME verified-dead-guard reason as `token_get_all` above and not on
//!   any "we owe an implementation" premise: elephc has no `headers_send` catalog entry, so
//!   `function_exists('headers_send')` folds to `false`; the guard at
//!   `HttpFoundation\Response::sendHeaders()` — `if ($informationalResponse &&
//!   !\function_exists('headers_send')) { return $this; }` — therefore fires whenever
//!   `$informationalResponse` is true, and the only `headers_send($statusCode)` call sits inside a
//!   later `if ($informationalResponse)`, i.e. on a path the guard has already returned from. The
//!   call is provably dead at runtime, while the checker still visits it and would otherwise
//!   reject the whole compile. Late-binding it to PHP's own "Call to undefined function" `\Error`
//!   is byte-faithful for elephc's non-FrankenPHP SAPI AND its throw is unreachable.
//! - `is_late_bound_undefined_function` matches on the LAST `\`-separated segment of the
//!   canonical name (case-insensitively): an unqualified call site written inside a namespace
//!   reaches this point already rewritten to its namespaced attempt form (e.g.
//!   `Symfony\Component\Cache\Adapter\apcu_exists`, matching PHP's own "namespace fallback
//!   failed too" error), and an explicitly fully-qualified call to the same bare name behaves
//!   identically in real PHP (any unresolvable function name is a late-bound runtime `\Error`
//!   regardless of qualification style) — both shapes are eligible, and the ORIGINAL canonical
//!   name (not the trimmed segment) is what must be embedded verbatim in the thrown message to
//!   stay byte-identical to PHP.
//! - Never applied inside a compile-time-evaluated context (`Checker::compile_time_const_depth
//!   > 0`: top-level `const` values, class/interface constant values) — PHP itself rejects ANY
//!   function call in those contexts, so elephc's pre-existing "Undefined function" diagnostic
//!   there is preserved rather than silently accepted. See `Checker::compile_time_const_depth`'s
//!   doc comment for why parameter/property default values are deliberately NOT covered by
//!   this guard (they are not compile-time-evaluated in elephc's model).

/// Curated, exact, lowercase extension function names late-bound instead of compile-rejected.
/// Extend only when a name is harvested from a real `--web` scan and its family is one elephc
/// has zero catalog presence under (mirrors
/// `crate::optimize::function_existence::NEVER_AVAILABLE_FUNCTION_PREFIXES`'s families).
const LATE_BOUND_UNDEFINED_FUNCTIONS: &[&str] = &[
    "opcache_invalidate",
    "opcache_compile_file",
    "apcu_exists",
    "apcu_store",
    "apcu_delete",
    "apcu_add",
    "xdebug_is_enabled",
    "xdebug_connect_to_client",
    "igbinary_serialize",
    "igbinary_unserialize",
    "frankenphp_handle_request",
    // Tokenizer builtin elephc does not provide, reached only through a verified
    // `!function_exists('token_get_all')` early-return guard that IS dead under AOT
    // (elephc has no `token_get_all` catalog entry, so `function_exists` folds false and
    // the guard fires). See the module doc.
    "token_get_all",
    // SAPI-provided (FrankenPHP) 1xx-informational-response helper, absent from core PHP
    // (`function_exists('headers_send') === false` on PHP 8.5.6). Reached only through a verified
    // `!function_exists('headers_send')` early-return guard in
    // `HttpFoundation\Response::sendHeaders()` that IS dead under AOT. See the module doc.
    "headers_send",
];

/// Returns whether `canonical_name` (as resolved by name-resolver/checker call lookup, possibly
/// namespace-prefixed) names one of the curated late-bound extension functions. Matches the
/// last `\`-separated segment case-insensitively; see the module doc for why the trailing
/// segment (not the whole canonical name) is the right match target.
pub(crate) fn is_late_bound_undefined_function(canonical_name: &str) -> bool {
    let bare = canonical_name
        .rsplit('\\')
        .next()
        .unwrap_or(canonical_name);
    LATE_BOUND_UNDEFINED_FUNCTIONS
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(bare))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A curated bare name matches regardless of case.
    #[test]
    fn matches_curated_name_case_insensitively() {
        assert!(is_late_bound_undefined_function("apcu_exists"));
        assert!(is_late_bound_undefined_function("APCU_EXISTS"));
        assert!(is_late_bound_undefined_function("Apcu_Exists"));
    }

    /// `apcu_add` (added in the M1 easy sweep alongside its `apcu_exists`/`apcu_store`/
    /// `apcu_delete` siblings) matches case-insensitively and is distinct from the unrelated
    /// `apcu_add_something`-shaped name.
    #[test]
    fn matches_apcu_add() {
        assert!(is_late_bound_undefined_function("apcu_add"));
        assert!(is_late_bound_undefined_function("APCU_ADD"));
        assert!(!is_late_bound_undefined_function("apcu_add_multi"));
    }

    /// `xdebug_connect_to_client` (the xdebug-family sibling of `xdebug_is_enabled`, called
    /// doubly-guarded inside `FrankenPhpWorkerRunner::run()`'s handler closure) is late-bound,
    /// including through its Symfony-namespaced attempt form, and does not match a typo.
    #[test]
    fn matches_xdebug_connect_to_client() {
        assert!(is_late_bound_undefined_function("xdebug_connect_to_client"));
        assert!(is_late_bound_undefined_function("XDEBUG_CONNECT_TO_CLIENT"));
        assert!(is_late_bound_undefined_function(
            "Symfony\\Component\\Runtime\\Runner\\xdebug_connect_to_client"
        ));
        assert!(!is_late_bound_undefined_function("xdebug_connect_to_clien"));
    }

    /// A curated name reached through a namespaced attempt (unqualified call inside a
    /// namespace, or an explicit fully-qualified call) still matches on its trailing segment.
    #[test]
    fn matches_namespaced_attempt_form() {
        assert!(is_late_bound_undefined_function(
            "Symfony\\Component\\Cache\\Adapter\\apcu_exists"
        ));
        assert!(is_late_bound_undefined_function("Foo\\Bar\\igbinary_serialize"));
    }

    /// A same-family typo does NOT match — no prefix wildcards (jury addendum #1).
    #[test]
    fn rejects_same_family_typo() {
        assert!(!is_late_bound_undefined_function("apcu_ftch"));
        assert!(!is_late_bound_undefined_function("apcu_tpyo"));
        assert!(!is_late_bound_undefined_function("opcache_invalidat"));
    }

    /// `token_get_all` (the verified dead-guarded tokenizer addition) is late-bound, including
    /// through its Symfony-namespaced attempt form, and does not match a typo. `proc_open` is
    /// deliberately NOT late-bound (it is a catalog builtin whose guards do not fire — see the
    /// module doc), so it must stay a compile-time error, i.e. NOT match here.
    #[test]
    fn matches_dead_guarded_token_get_all_not_proc_open() {
        assert!(is_late_bound_undefined_function("token_get_all"));
        assert!(is_late_bound_undefined_function("TOKEN_GET_ALL"));
        assert!(is_late_bound_undefined_function(
            "Symfony\\Component\\DependencyInjection\\Dumper\\token_get_all"
        ));
        assert!(!is_late_bound_undefined_function("token_get_al"));
        assert!(!is_late_bound_undefined_function("proc_open"));
        assert!(!is_late_bound_undefined_function(
            "Symfony\\Component\\Console\\proc_open"
        ));
    }

    /// `headers_send` (the SAPI-only, dead-guarded 1xx-response helper) is late-bound, including
    /// through its Symfony-namespaced attempt form, and does not match a typo or its unrelated
    /// core-PHP near-neighbour `headers_sent` (which elephc really does implement, so late-binding
    /// it would swallow a genuine builtin).
    #[test]
    fn matches_dead_guarded_headers_send_not_headers_sent() {
        assert!(is_late_bound_undefined_function("headers_send"));
        assert!(is_late_bound_undefined_function("HEADERS_SEND"));
        assert!(is_late_bound_undefined_function(
            "Symfony\\Component\\HttpFoundation\\headers_send"
        ));
        assert!(!is_late_bound_undefined_function("headers_sent"));
        assert!(!is_late_bound_undefined_function("headers_sen"));
    }

    /// A name outside the curated allowlist entirely does not match, even when it shares a
    /// prefix with a curated family member.
    #[test]
    fn rejects_unrelated_extension_function() {
        assert!(!is_late_bound_undefined_function("apcu_fetch"));
        assert!(!is_late_bound_undefined_function("pcntl_fork"));
        assert!(!is_late_bound_undefined_function("fastcgi_finish_request"));
    }
}
