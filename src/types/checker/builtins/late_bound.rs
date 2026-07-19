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
//!   `xdebug_is_enabled`, `igbinary_serialize`, `igbinary_unserialize`,
//!   `frankenphp_handle_request`. Non-extension-shaped undefined names surfaced by the same
//!   scan (`debug_backtrace`, `proc_open`, `eval`, `token_get_all`, `next`, `parse_ini_file`,
//!   `is_uploaded_file`, `move_uploaded_file`, `request_parse_body`, `headers_send`,
//!   `get_defined_functions`, `highlight_file`, `register_shutdown_function`) are OUT of
//!   scope: they are either core PHP functions elephc genuinely lacks (a real gap, not a
//!   late-bound guard pattern) or handled by unrelated work.
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
    "xdebug_is_enabled",
    "igbinary_serialize",
    "igbinary_unserialize",
    "frankenphp_handle_request",
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

    /// A name outside the curated allowlist entirely does not match, even when it shares a
    /// prefix with a curated family member.
    #[test]
    fn rejects_unrelated_extension_function() {
        assert!(!is_late_bound_undefined_function("apcu_fetch"));
        assert!(!is_late_bound_undefined_function("pcntl_fork"));
        assert!(!is_late_bound_undefined_function("fastcgi_finish_request"));
    }
}
