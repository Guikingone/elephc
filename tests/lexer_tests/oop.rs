//! Purpose:
//! Integration or regression tests for lexer tokenization coverage of object-oriented PHP, including lex double colon, lex this, and lex clone.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP source is tokenized and assertions check exact token kinds, literals, and source structure.

use super::*;

/// Verifies `::` (double colon) tokenizes as `DoubleColon` for static access.
#[test]
fn test_lex_double_colon() {
    let t = tokens("<?php Point::origin();");
    assert!(t.contains(&Token::DoubleColon));
}

/// Verifies `$this` tokenizes as `This`.
#[test]
fn test_lex_this() {
    let t = tokens("<?php $this->value;");
    assert_eq!(t[1], Token::This);
}

/// Verifies the `clone` keyword tokenizes as `Token::Clone` for the clone expression.
#[test]
fn test_lex_clone() {
    let t = tokens("<?php $b = clone $a;");
    // OpenTag, Variable("b"), Equals, Clone, Variable("a"), Semicolon
    assert_eq!(t[3], Token::Clone);
}

/// Verifies `self::${$n}` now tokenizes (no lexer hard error): the bare `$` before `{` becomes
/// `Token::Dollar`, so the sequence is `self :: $ { $n }`. The parser decides validity.
#[test]
fn test_lex_dynamic_static_property_dollar_brace() {
    let t = tokens("<?php self::${$n};");
    assert_eq!(
        &t[1..7],
        &[
            Token::Self_,
            Token::DoubleColon,
            Token::Dollar,
            Token::LBrace,
            Token::Variable("n".to_string()),
            Token::RBrace,
        ]
    );
}

/// Verifies a bare `$` immediately before another `$` (the `$$var` form) tokenizes as
/// `Token::Dollar` followed by the variable, rather than erroring in the lexer.
#[test]
fn test_lex_bare_dollar_before_variable() {
    let t = tokens("<?php $$name;");
    assert_eq!(t[1], Token::Dollar);
    assert_eq!(t[2], Token::Variable("name".to_string()));
}

// --- Spaceship operator ---
