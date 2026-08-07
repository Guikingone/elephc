//! Purpose:
//! Folds bounded ASCII-only string builtins for literal eval.
//!
//! Called from:
//! - The eval AOT facade and sibling analysis modules.
//!
//! Key details:
//! - Byte-order, trim masks, and predicate behavior remain intentionally narrow.

use super::*;

/// ASCII-only case conversion supported by literal eval builtin folding.
pub(super) enum AsciiCaseFold {
    Lower,
    Upper,
}

/// First-byte ASCII case conversion supported by literal eval builtin folding.
pub(super) enum FirstCharCaseFold {
    Lower,
    Upper,
}

/// Side selected by default-mask ASCII trim folding.
pub(super) enum TrimSide {
    Left,
    Right,
    Both,
}

/// Two-string ASCII predicates supported by literal eval builtin folding.
pub(super) enum StringPredicate {
    Contains,
    StartsWith,
    EndsWith,
}

/// Folds ASCII-only `strtolower()` and `strtoupper()` literal calls.
pub(super) fn fold_ascii_case(args: &[Expr], mode: AsciiCaseFold) -> Option<String> {
    if args.len() != 1 {
        return None;
    }
    let ExprKind::StringLiteral(value) = &args[0].kind else {
        return None;
    };
    if !value.is_ascii() {
        return None;
    }
    let folded = match mode {
        AsciiCaseFold::Lower => value.to_ascii_lowercase(),
        AsciiCaseFold::Upper => value.to_ascii_uppercase(),
    };
    Some(folded)
}

/// Folds ASCII-only `ucfirst()` and `lcfirst()` literal calls.
pub(super) fn fold_ascii_first_char_case(args: &[Expr], mode: FirstCharCaseFold) -> Option<String> {
    if args.len() != 1 {
        return None;
    }
    let ExprKind::StringLiteral(value) = &args[0].kind else {
        return None;
    };
    if !value.is_ascii() {
        return None;
    }
    let mut bytes = value.as_bytes().to_vec();
    if let Some(first) = bytes.first_mut() {
        match mode {
            FirstCharCaseFold::Lower => first.make_ascii_lowercase(),
            FirstCharCaseFold::Upper => first.make_ascii_uppercase(),
        }
    }
    String::from_utf8(bytes).ok()
}

/// Folds ASCII-only `strrev()` literal calls with PHP byte-order behavior.
pub(super) fn fold_ascii_strrev(args: &[Expr]) -> Option<String> {
    if args.len() != 1 {
        return None;
    }
    let ExprKind::StringLiteral(value) = &args[0].kind else {
        return None;
    };
    if !value.is_ascii() {
        return None;
    }
    Some(value.bytes().rev().map(char::from).collect())
}

/// Folds ASCII-only `substr()` literal calls with non-negative offset and length.
pub(super) fn fold_ascii_substr(args: &[Expr]) -> Option<String> {
    if !(2..=3).contains(&args.len()) {
        return None;
    }
    let ExprKind::StringLiteral(value) = &args[0].kind else {
        return None;
    };
    if !value.is_ascii() {
        return None;
    }
    let offset = usize::try_from(const_int_expr(&args[1])?).ok()?;
    let start = offset.min(value.len());
    let end = if let Some(length_arg) = args.get(2) {
        let length = usize::try_from(const_int_expr(length_arg)?).ok()?;
        start.saturating_add(length).min(value.len())
    } else {
        value.len()
    };
    Some(value[start..end].to_string())
}

/// Folds ASCII-only `str_repeat()` literal calls with a bounded static result.
pub(super) fn fold_ascii_str_repeat(args: &[Expr]) -> Option<String> {
    if args.len() != 2 {
        return None;
    }
    let ExprKind::StringLiteral(value) = &args[0].kind else {
        return None;
    };
    if !value.is_ascii() {
        return None;
    }
    let times = usize::try_from(const_int_expr(&args[1])?).ok()?;
    let bytes = value.len().checked_mul(times)?;
    if bytes > MAX_STATIC_STRING_FOLD_BYTES {
        return None;
    }
    Some(value.repeat(times))
}

/// Folds one-argument ASCII `trim()`/`ltrim()`/`rtrim()` calls using PHP's default mask.
pub(super) fn fold_ascii_default_trim(args: &[Expr], side: TrimSide) -> Option<String> {
    if args.len() != 1 {
        return None;
    }
    let ExprKind::StringLiteral(value) = &args[0].kind else {
        return None;
    };
    if !value.is_ascii() {
        return None;
    }
    let trimmed = match side {
        TrimSide::Left => value.trim_start_matches(is_php_default_trim_char),
        TrimSide::Right => value.trim_end_matches(is_php_default_trim_char),
        TrimSide::Both => value.trim_matches(is_php_default_trim_char),
    };
    Some(trimmed.to_string())
}

/// Returns true for characters removed by PHP's default trim character mask.
pub(super) fn is_php_default_trim_char(ch: char) -> bool {
    matches!(ch, '\0' | '\t' | '\n' | '\r' | '\x0b' | ' ')
}

/// Folds ASCII-only two-string predicate calls to their boolean result.
pub(super) fn fold_ascii_string_predicate(args: &[Expr], predicate: StringPredicate) -> Option<bool> {
    if args.len() != 2 {
        return None;
    }
    let (ExprKind::StringLiteral(haystack), ExprKind::StringLiteral(needle)) =
        (&args[0].kind, &args[1].kind)
    else {
        return None;
    };
    if !haystack.is_ascii() || !needle.is_ascii() {
        return None;
    }
    Some(match predicate {
        StringPredicate::Contains => haystack.contains(needle),
        StringPredicate::StartsWith => haystack.starts_with(needle),
        StringPredicate::EndsWith => haystack.ends_with(needle),
    })
}
