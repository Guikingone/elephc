//! Purpose:
//! Recognition-layer tests for the intl grapheme (`grapheme_*`), intl normalizer
//! (`normalizer_*`), and iconv (`iconv*`) PHP builtins. These builtins are registered for type
//! checking and first-class-callable resolution only; runtime lowering is deferred, so these
//! tests assert type-check recognition (never `compile_and_run`).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Verifies arity diagnostics, PHP-accurate return types (the `|false` / `|false|null`
//!   unions), and first-class-callable syntax. A regressing catalog/signature entry surfaces
//!   here as a spurious "Undefined function" or a wrong arity/return-type error.

use super::*;

/// Verifies that every registered grapheme/normalizer/iconv builtin is recognized
/// (no "Undefined function") in a representative valid call form.
#[test]
fn test_intl_builtins_recognized() {
    assert!(
        check_source(
            r#"<?php
$gl = grapheme_strlen("héllo");
$gs = grapheme_substr("héllo", 1, 3);
$gi = grapheme_stripos("héllo", "L");
$gr = grapheme_strripos("héllo", "L");
$gsp = grapheme_str_split("héllo");
$ge = grapheme_extract("héllo", 2);
$nn = normalizer_normalize("héllo");
$ni = normalizer_is_normalized("héllo");
$ic = iconv("UTF-8", "ASCII//TRANSLIT", "héllo");
$il = iconv_strlen("héllo");
$ip = iconv_strpos("héllo", "l");
$ir = iconv_strrpos("héllo", "l");
$isub = iconv_substr("héllo", 1, 3);
$im = iconv_mime_decode("=?UTF-8?B?aGk=?=");
echo $gs . $ge . $nn . $ic . $isub . $im;
echo $ni ? "y" : "n";
"#
        )
        .is_ok(),
        "all registered grapheme/normalizer/iconv builtins should type-check",
    );
}

/// Verifies the canonical catalog enables PHP's case-insensitive global-builtin
/// fallback for an `iconv()` call written inside a namespace.
#[test]
fn test_iconv_catalog_supports_case_insensitive_namespace_fallback() {
    assert!(
        check_source(
            r#"<?php
namespace Symfony\Component\String;
$converted = ICONV("UTF-8", "ASCII", "abc");
echo $converted;
"#
        )
        .is_ok(),
        "namespaced case-insensitive iconv() should resolve through the global builtin catalog",
    );
}

/// Verifies the `|false`-union return types: `grapheme_stripos`/`iconv_strpos` yield
/// `int|false`, `grapheme_substr`/`iconv`/`normalizer_normalize` yield `string|false`,
/// and `grapheme_str_split` yields `array|false`, so a strict `=== false` check
/// type-checks against each union.
#[test]
fn test_intl_union_returns_recognized() {
    assert!(
        check_source(
            r#"<?php
$pos = grapheme_stripos("abc", "b");
if ($pos === false) { echo "no"; } else { echo $pos; }
$sub = grapheme_substr("abc", 1);
if ($sub === false) { echo "no"; } else { echo $sub; }
$sp = grapheme_str_split("abc");
if ($sp === false) { echo "no"; } else { echo $sp[0]; }
$conv = iconv("UTF-8", "ASCII", "abc");
if ($conv === false) { echo "no"; } else { echo $conv; }
$ipos = iconv_strpos("abc", "b");
if ($ipos === false) { echo "no"; } else { echo $ipos; }
$norm = normalizer_normalize("abc");
if ($norm === false) { echo "no"; } else { echo $norm; }
"#
        )
        .is_ok(),
        "intl/iconv union return types (int|false, string|false, array|false) should type-check",
    );
}

/// Verifies the case-sensitive grapheme search pair `grapheme_strpos`/`grapheme_strrpos`
/// (unmasked by this batch) is recognized, accepts the optional `$offset`, and yields an
/// `int|false` union that a strict `=== false` check type-checks against.
#[test]
fn test_grapheme_strpos_recognized() {
    assert!(
        check_source(
            r#"<?php
$p = grapheme_strpos("héllo", "l");
if ($p === false) { echo "no"; } else { echo $p; }
$r = grapheme_strrpos("héllo", "l", 1);
if ($r === false) { echo "no"; } else { echo $r; }
"#
        )
        .is_ok(),
        "grapheme_strpos/grapheme_strrpos should type-check with an int|false return",
    );
}

expect_builtin_arity_error!(
    test_error_grapheme_strpos_too_few_args,
    "<?php grapheme_strpos(\"a\");",
    "grapheme_strpos() takes 2 or 3 arguments"
);

/// Verifies that the optional-tail forms type-check: `normalizer_normalize` with an
/// explicit `$form`, `iconv_strpos` with an `$offset`, and `grapheme_extract` with a
/// `$type`/`$offset`.
#[test]
fn test_intl_optional_args_recognized() {
    assert!(
        check_source(
            r#"<?php
$a = normalizer_normalize("héllo", 16);
$b = iconv_strpos("héllo", "l", 1, "UTF-8");
$c = grapheme_extract("héllo", 2, 0, 0);
echo $a . $b . $c;
"#
        )
        .is_ok(),
        "intl/iconv optional trailing arguments should type-check",
    );
}

/// Verifies that grapheme/iconv builtins work through first-class-callable syntax,
/// since Symfony passes such functions as callables (e.g. to `array_map`).
#[test]
fn test_intl_first_class_callable_recognized() {
    assert!(
        check_source(
            "<?php $f = grapheme_strlen(...); $g = iconv_strlen(...); $h = normalizer_normalize(...); echo is_callable($f) && is_callable($g) && is_callable($h);"
        )
        .is_ok(),
        "grapheme_strlen(...)/iconv_strlen(...)/normalizer_normalize(...) first-class callable syntax should type-check",
    );
}

expect_builtin_arity_error!(
    test_error_grapheme_strlen_wrong_args,
    "<?php grapheme_strlen(\"a\", \"b\");",
    "grapheme_strlen() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_grapheme_substr_too_few_args,
    "<?php grapheme_substr(\"a\");",
    "grapheme_substr() takes 2 or 3 arguments"
);

expect_builtin_arity_error!(
    test_error_iconv_wrong_args,
    "<?php iconv(\"UTF-8\", \"ASCII\");",
    "iconv() takes exactly 3 arguments"
);

expect_builtin_arity_error!(
    test_error_iconv_strrpos_too_many_args,
    "<?php iconv_strrpos(\"a\", \"b\", \"UTF-8\", 1);",
    "iconv_strrpos() takes 2 or 3 arguments"
);

expect_builtin_arity_error!(
    test_error_normalizer_normalize_too_many_args,
    "<?php normalizer_normalize(\"a\", 16, 1);",
    "normalizer_normalize() takes 1 or 2 arguments"
);

/// Regression: elephc supplements a registered `Normalizer` class (e.g. the
/// symfony/polyfill-intl-normalizer stub, which omits it) with the ext-intl
/// `NFKC_CF = 48` constant, since elephc emulates an intl-present environment.
/// Code guarded by `\defined('Normalizer::NFKC_CF')` (symfony/string's
/// `AbstractUnicodeString::folded()`) then type-checks. See
/// `builtin_types::normalizer::inject_builtin_normalizer`.
#[test]
fn test_normalizer_nfkc_cf_constant_resolves() {
    expect_ok(
        r#"<?php
class Normalizer { public const FORM_C = 16; public const NFC = 16; }
echo Normalizer::NFKC_CF;
echo Normalizer::FORM_C;
"#,
    );
}

/// Negative control for the `Normalizer::NFKC_CF` supplement: a genuinely undefined
/// constant on the same class still errors, so the supplement adds only the real
/// intl constants and never blanket-accepts arbitrary class constants.
#[test]
fn test_normalizer_fake_constant_still_errors() {
    expect_error(
        r#"<?php
class Normalizer { public const FORM_C = 16; }
echo Normalizer::TOTALLY_FAKE;
"#,
        "Undefined class constant: Normalizer::TOTALLY_FAKE",
    );
}

/// Regression: with no user/vendor `Normalizer` in scope, elephc injects a builtin
/// constants-only `Normalizer` class (it emulates an ext-intl-present environment), so
/// the ext-intl constants resolve without any declaration. Previously such a reference
/// degraded to an absent-optional-dependency `ScopedConstantGet(Mixed)` the backend
/// rejected. See `builtin_types::normalizer::inject_builtin_normalizer`.
#[test]
fn test_normalizer_injected_constants_resolve() {
    expect_ok(r#"<?php echo Normalizer::NFKC_CF, Normalizer::FORM_C, Normalizer::FORM_KC_CF;"#);
}

/// Negative control for the injected (no-declaration) `Normalizer`: a genuinely undefined
/// constant still errors loudly, and the deprecated PHP-8-removed `NONE` constant is not
/// resurrected — the injection provides only the real PHP 8.5 ext-intl constant set.
#[test]
fn test_normalizer_injected_fake_constant_still_errors() {
    expect_error(
        r#"<?php echo Normalizer::NO_SUCH_CONST;"#,
        "Undefined class constant: Normalizer::NO_SUCH_CONST",
    );
    expect_error(
        r#"<?php echo Normalizer::NONE;"#,
        "Undefined class constant: Normalizer::NONE",
    );
}
