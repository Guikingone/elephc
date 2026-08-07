//! Purpose:
//! Renders the CLI ini_get, ini_set, and ini_get_all compatibility surface.
//!
//! Called from:
//! - The OPcache prelude facade and sibling rendering modules.
//!
//! Key details:
//! - User declarations and module filters preserve pay-for-use injection.

#[allow(unused_imports)]
use super::*;

/// The CLI `ini_get` wrapper. On a plain CLI binary there is no session bridge (that is a
/// `--web`-only surface), so `ini_get` models exactly the `opcache.*` directive strings and
/// returns `false` for every other key — including `session.*` — matching reference PHP,
/// where a default `php script.php` has OPcache's directives registered but reports `false`
/// for a directive of an unloaded/absent extension. Being a real declared function is what
/// makes `function_exists('ini_get')` report `true`.
pub(super) const CLI_INI_GET_TEMPLATE: &str = r#"function ini_get(string $option): string|false {
    return __elephc_opcache_ini_string($option);
}
"#;

/// The CLI `ini_set` wrapper. Every `opcache.*` directive is compile-time-baked into the AOT
/// binary and cannot be mutated at runtime, and a CLI binary models nothing else settable, so
/// `ini_set` reports failure (`false`) for every key while `ini_get` keeps returning the baked
/// value. This is exact for the PHP_INI_SYSTEM majority; the 18 PHP_INI_ALL opcache directives
/// (which reference PHP would let you set) are a documented interim divergence. Both parameters
/// are consumed so the checker does not flag them unused.
pub(super) const CLI_INI_SET_TEMPLATE: &str = r#"function ini_set(string $option, $value): string|false {
    $value = (string) $value;
    if (__elephc_opcache_ini_string($option) === $value) { return false; }
    return false;
}
"#;

/// The CLI `ini_get_all` wrapper — the extension-filter dispatch, byte-modeled on php-src.
///
/// Reference PHP matches `$extension` VERBATIM against the module registry, whose keys are
/// lowercase, and does NOT case-fold (unlike `extension_loaded`, which does). So
/// `ini_get_all('zend opcache')` yields the 54 opcache entries while `ini_get_all('Zend OPcache')`
/// — the spelling `get_loaded_extensions()` reports — is "not found": an `E_WARNING`
/// (`ini_get_all(): Extension "…" cannot be found`) and `false`. A module that IS known but
/// registers no INI directives yields an EMPTY ARRAY, not `false` (verified on reference PHP
/// 8.5.6: `spl`/`json`/`ctype`/`reflection` → `[]`), which is what `__elephc_ini_module_known`
/// distinguishes.
///
/// `'core'` maps to the UNFILTERED surface, reproducing php-src's rule that Core's
/// `module_number` is 0 and the per-module filter is skipped for it. DOCUMENTED DIVERGENCE:
/// reference PHP's unfiltered surface is every registered directive of every loaded module
/// (403 on the reference build); elephc models only the directive blocks it actually owns, so
/// the unfiltered count is 54 on CLI (opcache only) and 87 under `--web` (session + opcache).
/// The rule is reproduced, the population is elephc's.
///
/// The RETURN TYPE HINT IS DELIBERATELY OMITTED (reference PHP is `array|false`): the
/// `GET_STATUS_TEMPLATE` precedent above applies verbatim — omitting the hint lets ordinary
/// union return inference handle the exits instead of leaning on union-return-type codegen.
///
/// SHAPE CONSTRAINT (an elephc codegen limitation, not a PHP-semantics one): a function that
/// writes an array-literal value on one branch and a scalar on the other into the SAME array
/// slot inside one loop miscompiles (SIGSEGV / heap exhaustion, no diagnostic). The
/// `$details` split therefore happens HERE, by dispatching to one of two single-shape
/// helpers, never inside a shared loop. See `render_opcache_ini_helpers`.
///
/// `__elephc_ini_module_known` takes `?string` rather than `string` because `$extension !== null`
/// does not currently narrow a `?string` parameter to `Str` in the checker; a nullable parameter
/// accepts the un-narrowed union while comparing identically against the string literals.
pub(super) const CLI_INI_GET_ALL_TEMPLATE: &str =
    r#"function ini_get_all(?string $extension = null, bool $details = true) {
    if ($extension !== null && $extension !== 'zend opcache' && $extension !== 'core') {
        if (__elephc_ini_module_known($extension)) { return []; }
        fwrite(STDERR, 'Warning: ini_get_all(): Extension "' . $extension . '" cannot be found' . "\n");
        return false;
    }
    if ($details) { return __elephc_opcache_ini_all_details(); }
    return __elephc_opcache_ini_all_plain();
}
"#;

/// Renders `__elephc_ini_module_known(?string $m): bool` — the KNOWN-MODULE predicate the
/// `ini_get_all` extension filter uses to tell "known module with no INI directives" (`[]`)
/// from "no such module" (`E_WARNING` + `false`).
///
/// The list is derived from [`CORE_LOADED_EXTENSIONS`] — the same compile-time set that backs
/// `extension_loaded()` / `get_loaded_extensions()` — LOWERCASED at render time, so the two
/// cannot drift and the comparison is verbatim against lowercase registry keys (reference PHP
/// does NOT case-fold this argument; do not share a comparison helper with `extension_loaded`,
/// which does). `web` adds `'session'`, the extra module a `--web` binary registers.
///
/// Bridge-linked extensions (`PDO`, `hash`, …) are deliberately NOT included: they are a
/// per-compilation link-set decision made in codegen, while this prelude is rendered before
/// codegen. A program compiled `--with-pdo` therefore reports `ini_get_all('pdo')` as
/// "cannot be found" rather than `[]` — a documented interim narrower than reference PHP.
pub(crate) fn render_ini_module_known(web: bool) -> String {
    let mut names: Vec<String> = crate::codegen::lower_inst::builtins::CORE_LOADED_EXTENSIONS
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect();
    if web {
        names.push("session".to_string());
    }
    let conditions: Vec<String> = names
        .iter()
        .map(|name| format!("$m === {}", render_php_single_quoted(name)))
        .collect();
    format!(
        "function __elephc_ini_module_known(?string $m): bool {{\n\
         \x20   return {};\n\
         }}\n",
        conditions.join("\n        || "),
    )
}
