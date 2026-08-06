//! Purpose:
//! Registers the ext-intl `Normalizer` class constants so both the type checker and
//! codegen can resolve `Normalizer::FORM_C`, `Normalizer::NFKC_CF`, etc. elephc
//! emulates an ext-intl-present environment (it provides `normalizer_normalize`), so it
//! must expose the real `\Normalizer` class constants exactly as ext-intl does — mirroring
//! how the builtin `DateTime`/`PDO` classes and their constants are injected.
//!
//! Called from:
//! - `crate::types::checker::driver` init (after the other builtin-class injectors).
//!
//! Key details:
//! - Real ext-intl's `\Normalizer` (PHP 8.5) defines FORM_D/NFD=4, FORM_KD/NFKD=8,
//!   FORM_C/NFC=16, FORM_KC/NFKC=32, FORM_KC_CF/NFKC_CF=48. The deprecated `NONE`
//!   constant was removed in PHP 8, so it is intentionally NOT provided (referencing
//!   `Normalizer::NONE` is a fatal error under a real ext-intl PHP 8.5).
//! - When no `Normalizer` class is registered (e.g. a bare program with no
//!   symfony/polyfill-intl-normalizer stub), a builtin constants-only `Normalizer` class
//!   is injected so the constants resolve and fold to their literal value at codegen.
//! - When a `Normalizer` class already exists (the polyfill stub loaded when ext-intl is
//!   assumed absent), injecting a second one would collide, so the existing class's
//!   constant set is SUPPLEMENTED with any ext-intl constants it omits (the stub notably
//!   lacks `NFKC_CF`/`FORM_KC_CF`, which symfony/string's `AbstractUnicodeString::folded()`
//!   guards behind `\defined('Normalizer::NFKC_CF')`).
//! - Both paths share one source of truth (`NORMALIZER_CONSTANTS`), so the checker and the
//!   lowering fold read identical values.

use std::collections::HashMap;

use crate::names::php_symbol_key;
use crate::parser::ast::{ClassConst, Expr, ExprKind, Visibility};
use crate::types::traits::FlattenedClass;

/// Real ext-intl `\Normalizer` class constants and their integer values, verified against
/// PHP 8.5.6 with ext-intl on this machine (`(new ReflectionClass('Normalizer'))->getConstants()`).
/// The deprecated `NONE` constant (removed in PHP 8) is intentionally absent.
const NORMALIZER_CONSTANTS: &[(&str, i64)] = &[
    ("FORM_D", 4),
    ("NFD", 4),
    ("FORM_KD", 8),
    ("NFKD", 8),
    ("FORM_C", 16),
    ("NFC", 16),
    ("FORM_KC", 32),
    ("NFKC", 32),
    ("FORM_KC_CF", 48),
    ("NFKC_CF", 48),
];

/// Returns whether a class-constant initializer reads a constant off `Normalizer` itself.
///
/// That is how `symfony/polyfill-intl-normalizer`'s stub writes every constant
/// (`public const NFKC = \Normalizer::NFKC;`), aliasing the very class it stands in for. The
/// initializer is therefore circular and never folds to an integer, so the declared value must be
/// replaced with the authoritative ext-intl one rather than kept.
fn is_self_referential_normalizer_const(value: &Expr) -> bool {
    let ExprKind::ScopedConstantAccess {
        receiver: crate::parser::ast::StaticReceiver::Named(name),
        ..
    } = &value.kind
    else {
        return false;
    };
    php_symbol_key(&name.as_canonical()) == php_symbol_key("Normalizer")
}

/// Builds a public integer class constant for the synthetic `Normalizer` class.
fn int_class_const(name: &str, value: i64) -> ClassConst {
    ClassConst {
        name: name.to_string(),
        visibility: Visibility::Public,
        is_final: false,
        type_expr: None,
        value: Expr::new(ExprKind::IntLiteral(value), crate::span::Span::dummy()),
        span: crate::span::Span::dummy(),
        attributes: Vec::new(),
    }
}

/// Registers the ext-intl `Normalizer` class constants.
///
/// Injects a builtin constants-only `Normalizer` class when none is registered, or
/// supplements an existing (vendor-stub) `Normalizer` with any ext-intl constants it
/// omits. Matched case-insensitively (PHP class names are case-insensitive) and idempotent
/// across passes, so it is safe to run unconditionally.
pub(crate) fn inject_builtin_normalizer(class_map: &mut HashMap<String, FlattenedClass>) {
    let normalizer_key = php_symbol_key("Normalizer");
    if let Some(normalizer) = class_map
        .values_mut()
        .find(|class| php_symbol_key(&class.name) == normalizer_key)
    {
        // A `Normalizer` already exists (the polyfill stub): add the constants it lacks, and
        // REPLACE any it declares as a self-reference, so the checker and codegen see the full
        // ext-intl set with real integer values.
        //
        // `symfony/polyfill-intl-normalizer`'s stub writes each constant as
        // `public const NFKC = \Normalizer::NFKC;` — an alias of the very class it is standing in
        // for. Kept as-is that initializer is circular, so it does not fold to an integer, and the
        // value reaching `normalizer_normalize(?string $string, ?int $form)`'s `?int` parameter
        // typed as `Str` — which codegen refused with
        // `conversion from PHP type Str to PHP type TaggedScalar`. The polyfill DEFINES these
        // constants to be the ext-intl values, and `NORMALIZER_CONSTANTS` holds exactly those, so
        // substituting the authoritative value is what the stub itself asks for.
        for (name, value) in NORMALIZER_CONSTANTS {
            match normalizer
                .constants
                .iter_mut()
                .find(|constant| constant.name == *name)
            {
                Some(existing) if is_self_referential_normalizer_const(&existing.value) => {
                    *existing = int_class_const(name, *value);
                }
                Some(_) => {}
                None => normalizer.constants.push(int_class_const(name, *value)),
            }
        }
        return;
    }

    class_map.insert(
        "Normalizer".to_string(),
        FlattenedClass {
            name: "Normalizer".to_string(),
            span: crate::span::Span::dummy(),
            extends: None,
            implements: Vec::new(),
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            properties: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new(),
            constants: NORMALIZER_CONSTANTS
                .iter()
                .map(|(name, value)| int_class_const(name, *value))
                .collect(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );
}
