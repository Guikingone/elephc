//! Purpose:
//! Supplements an already-registered `Normalizer` class with the ext-intl `NFKC_CF`
//! class constant so vendor code that references it type-checks.
//!
//! Called from:
//! - `crate::types::checker::driver` init (after the other builtin-class injectors).
//!
//! Key details:
//! - Real ext-intl's `\Normalizer` defines `NFKC_CF = 48` (Unicode NFKC_Casefold
//!   normalization form). The symfony/polyfill-intl-normalizer stub loaded when
//!   ext-intl is absent omits it, so code guarded by `\defined('Normalizer::NFKC_CF')`
//!   (e.g. symfony/string's `AbstractUnicodeString::folded()`) still names it and the
//!   flattened stub lacks the constant.
//! - elephc emulates an ext-intl-present environment (it provides `normalizer_normalize`),
//!   so this SUPPLEMENTS an existing `Normalizer` class rather than injecting a new one:
//!   injecting would collide with the vendor stub, and a program without any `Normalizer`
//!   class already degrades the reference to `Mixed` through the absent-class path.

use std::collections::HashMap;

use crate::names::php_symbol_key;
use crate::parser::ast::{ClassConst, Expr, ExprKind, Visibility};
use crate::types::traits::FlattenedClass;

/// Real ext-intl value of `Normalizer::NFKC_CF` (verified against PHP 8.5:
/// `php -r 'echo Normalizer::NFKC_CF;'` prints `48`).
const NFKC_CF_VALUE: i64 = 48;

/// Adds `NFKC_CF = 48` to an existing `Normalizer` class (matched case-insensitively,
/// mirroring PHP's case-insensitive class names) that does not already declare it.
///
/// No-op when no `Normalizer` class is registered or when the constant is already
/// present, so it is safe to run unconditionally and idempotent across passes.
pub(crate) fn supplement_intl_normalizer_constants(
    class_map: &mut HashMap<String, FlattenedClass>,
) {
    let normalizer_key = php_symbol_key("Normalizer");
    let Some(normalizer) = class_map
        .values_mut()
        .find(|class| php_symbol_key(&class.name) == normalizer_key)
    else {
        return;
    };
    if normalizer
        .constants
        .iter()
        .any(|constant| constant.name == "NFKC_CF")
    {
        return;
    }
    normalizer.constants.push(ClassConst {
        name: "NFKC_CF".to_string(),
        visibility: Visibility::Public,
        is_final: false,
        type_expr: None,
        value: Expr::new(ExprKind::IntLiteral(NFKC_CF_VALUE), crate::span::Span::dummy()),
        span: crate::span::Span::dummy(),
        attributes: Vec::new(),
    });
}
