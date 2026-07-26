//! Purpose:
//! Curated, EXACT allowlist of platform-conditional PHP predefined constants that are genuinely
//! defined on a platform OTHER than the current AOT target (today: the Windows-only
//! `PHP_WINDOWS_VERSION_*` family). A bare reference to one of these on a non-Windows target
//! compiles instead of being rejected with a hard "Undefined constant", and is lowered to the same
//! runtime constant lookup PHP performs (`__rt_constant`), which throws PHP's own catchable `\Error`
//! on a miss (see `crate::ir_lower::expr::constants::lower_const_ref`).
//!
//! Called from:
//! - `crate::types::checker::inference::expr` (`ExprKind::ConstRef` inference), which consults
//!   `is_platform_conditional_constant` before falling back to the "Undefined constant" diagnostic
//!   and, on a match, degrades the reference to `PhpType::Mixed`.
//! - `crate::ir_lower::expr::constants::lower_const_ref`, which consults the same allowlist to lower
//!   a matching bare reference to a `constant("NAME")` runtime lookup instead of the runtime-`define`
//!   `LoadGlobal` fallback (there is no global slot for a constant that was never `define`d).
//!
//! Key details:
//! - EXACT names only — deliberately NOT prefix-matched. A prefix wildcard would let a typo like
//!   `PHP_WINDOWS_VERSON_MAJOR` silently swallow into "compiles, throws at runtime" instead of the
//!   precious compile-time "Undefined constant" diagnostic, and would defeat the whole point of the
//!   closed-world constant check (mirrors the same discipline as
//!   `crate::types::checker::builtins::late_bound`).
//! - PHP-faithful on BOTH axes: PHP does not reject an undefined-constant reference at compile time
//!   — it only throws a catchable `\Error` when the line actually EXECUTES. So a reference sitting in
//!   a Windows-only branch that is dead under a non-Windows AOT target (e.g. guarded by
//!   `'\\' === \DIRECTORY_SEPARATOR`) costs nothing to compile and, if somehow reached, throws the
//!   exact same `\Error` PHP would. A truly-undefined constant (a typo, or a constant this project
//!   genuinely lacks that is NOT platform-conditional) is NOT on this list and stays a loud
//!   compile-time error — the honest real-gap / typo signal.
//! - Harvested from the exact "Undefined constant: ..." names produced by `--web` on
//!   `examples/symfony-app/public/index.php`, filtered to real PHP predefined constants that are
//!   defined only on another platform. The current set is exactly the three referenced by
//!   `Symfony\Component\VarDumper\Dumper\CliDumper::isWindowsTrueColor()`:
//!   `PHP_WINDOWS_VERSION_MAJOR`, `PHP_WINDOWS_VERSION_MINOR`, `PHP_WINDOWS_VERSION_BUILD`. That
//!   method is reached only through a `'\\' === \DIRECTORY_SEPARATOR` guard in `hasColorSupport()`,
//!   which folds dead on macOS/Linux, so the references never execute at runtime — but the checker
//!   still visits the method body and would otherwise reject the whole compile.
//! - `is_platform_conditional_constant` matches on the LAST `\`-separated segment of the name
//!   (case-SENSITIVELY — PHP constant names are case-sensitive). An unqualified reference written
//!   inside a namespace reaches this point already rewritten to its namespaced attempt form (e.g.
//!   `Symfony\Component\VarDumper\Dumper\PHP_WINDOWS_VERSION_MAJOR`, matching PHP's own "namespace
//!   fallback to global failed" resolution), and an explicitly fully-qualified reference behaves
//!   identically in real PHP; both shapes resolve to the bare global name, which is what is matched
//!   and what is embedded verbatim in the runtime lookup so the thrown message stays byte-identical
//!   to PHP.
//! - Extend only when a name is harvested from a real `--web` scan AND is a genuine PHP predefined
//!   constant defined on a platform other than the current target. NEVER add a fake numeric value
//!   for one of these — the whole point is that this target does not define them.

/// Curated, exact, case-sensitive platform-conditional constant names tolerated as `Mixed` instead
/// of compile-rejected. Extend only with real PHP predefined constants that are defined on a
/// platform other than the current AOT target (see the module doc).
const PLATFORM_CONDITIONAL_CONSTANTS: &[&str] = &[
    // Windows-only PHP predefined constants, referenced by VarDumper's `CliDumper` behind a
    // `'\\' === \DIRECTORY_SEPARATOR` guard that is dead on macOS/Linux.
    "PHP_WINDOWS_VERSION_MAJOR",
    "PHP_WINDOWS_VERSION_MINOR",
    "PHP_WINDOWS_VERSION_BUILD",
];

/// Returns whether `name` (as resolved by the checker's constant lookup, possibly namespace-
/// prefixed) names one of the curated platform-conditional constants. Matches the last
/// `\`-separated segment case-sensitively; see the module doc for why the trailing segment (the
/// bare global name) is the right match target.
pub(crate) fn is_platform_conditional_constant(name: &str) -> bool {
    let bare = name.trim_start_matches('\\');
    let bare = bare.rsplit('\\').next().unwrap_or(bare);
    PLATFORM_CONDITIONAL_CONSTANTS
        .iter()
        .any(|candidate| *candidate == bare)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The curated Windows version constants match, both bare and through their namespaced
    /// attempt form (an unqualified reference inside a namespace, or a fully-qualified one).
    #[test]
    fn matches_curated_windows_constants() {
        assert!(is_platform_conditional_constant("PHP_WINDOWS_VERSION_MAJOR"));
        assert!(is_platform_conditional_constant("PHP_WINDOWS_VERSION_MINOR"));
        assert!(is_platform_conditional_constant("PHP_WINDOWS_VERSION_BUILD"));
        assert!(is_platform_conditional_constant(
            "Symfony\\Component\\VarDumper\\Dumper\\PHP_WINDOWS_VERSION_MAJOR"
        ));
        assert!(is_platform_conditional_constant(
            "\\PHP_WINDOWS_VERSION_BUILD"
        ));
    }

    /// PHP constant names are case-sensitive, so a differently-cased spelling does NOT match.
    #[test]
    fn is_case_sensitive() {
        assert!(!is_platform_conditional_constant("php_windows_version_major"));
        assert!(!is_platform_conditional_constant("Php_Windows_Version_Major"));
    }

    /// A typo in a curated name does NOT match — no prefix/fuzzy wildcards, so the compile-time
    /// "Undefined constant" diagnostic is preserved for genuine typos.
    #[test]
    fn rejects_typo_in_curated_name() {
        assert!(!is_platform_conditional_constant("PHP_WINDOWS_VERSON_MAJOR"));
        assert!(!is_platform_conditional_constant("PHP_WINDOWS_VERSION_MAJO"));
        assert!(!is_platform_conditional_constant("PHP_WINDOWS_VERSION"));
    }

    /// An unrelated undefined constant (a genuine gap or typo) stays out of the allowlist and
    /// therefore stays a loud compile-time error.
    #[test]
    fn rejects_unrelated_constant() {
        assert!(!is_platform_conditional_constant("APC_ITER_KEY"));
        assert!(!is_platform_conditional_constant("SOME_MISSING_CONSTANT"));
        assert!(!is_platform_conditional_constant("PHP_INT_MAX"));
    }
}
