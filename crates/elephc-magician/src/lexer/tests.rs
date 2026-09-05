//! Purpose:
//! Pins the token shapes the double-quoted string lexer produces.
//!
//! Before this suite existed the eval interpreter's lexer returned every double-quoted
//! literal as one opaque `TokenKind::String`, so a runtime-interpreted fragment printed
//! `a:$i:` where PHP 8.5.6 prints `a:7:`. These cases pin both halves of the contract:
//! a literal WITHOUT interpolation must still be exactly one `String` token, and a
//! literal WITH interpolation must be a parenthesized `.` concatenation.
//!
//! Called from:
//! - `cargo test -p elephc-magician --lib lexer::tests` through Rust's test harness.
//!
//! Key details:
//! - Assertions compare the WHOLE token vector, never a "contains" probe, so a stray
//!   extra or missing token fails.
//! - Malformed simple offsets are PHP parse errors (measured against PHP 8.5.6) and are
//!   pinned as refusals so a future rewrite cannot start silently accepting them.

use super::scan::tokenize;
use super::TokenKind;
use crate::errors::EvalParseError;

/// Tokenizes a fragment and returns the token kinds, failing the test on a parse error.
fn kinds(source: &str) -> Vec<TokenKind> {
    tokenize(source)
        .expect("fragment should tokenize")
        .into_iter()
        .map(super::Token::into_kind)
        .collect()
}

/// Returns the parse error a fragment produces, failing the test if it tokenizes.
fn error(source: &str) -> EvalParseError {
    tokenize(source).expect_err("fragment should be refused")
}

/// Builds a `TokenKind::String` from a literal for terser expectations.
fn string(value: &str) -> TokenKind {
    TokenKind::String(value.to_string())
}

/// Builds a `TokenKind::DollarIdent` from a literal for terser expectations.
fn var(name: &str) -> TokenKind {
    TokenKind::DollarIdent(name.to_string())
}

/// Verifies eval numeric literals preserve PHP radix, separator, and exponent semantics.
#[test]
fn numeric_literals_preserve_php_radices_and_float_forms() {
    assert_eq!(
        kinds("0777; 0o701; 0x1f; 0b1010; 1_000; 012.5; 08e1;"),
        vec![
            TokenKind::Int(511),
            TokenKind::Semicolon,
            TokenKind::Int(449),
            TokenKind::Semicolon,
            TokenKind::Int(31),
            TokenKind::Semicolon,
            TokenKind::Int(10),
            TokenKind::Semicolon,
            TokenKind::Int(1_000),
            TokenKind::Semicolon,
            TokenKind::Float(12.5),
            TokenKind::Semicolon,
            TokenKind::Float(80.0),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies malformed radix digits and separator placement are rejected by eval parsing.
#[test]
fn malformed_numeric_literals_are_refused() {
    for source in ["078;", "0o78;", "0xfg;", "0b12;", "1__0;"] {
        assert_eq!(error(source), EvalParseError::InvalidNumber, "{source}");
    }
}

/// Verifies `??=` is emitted as one token without changing `??` or `=` tokenization.
#[test]
fn null_coalesce_assignment_stays_one_token() {
    assert_eq!(
        kinds("$value ??= $fallback ?? null;"),
        vec![
            var("value"),
            TokenKind::QuestionQuestionEqual,
            var("fallback"),
            TokenKind::QuestionQuestion,
            TokenKind::Ident("null".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a double-quoted literal without interpolation stays exactly ONE string token.
///
/// This is the invariant that keeps constant initializers, property defaults, enum
/// backing values and `declare()` arguments behaving identically: `parser::primary` maps
/// a lone `TokenKind::String` straight to `EvalConst::String`, and a parenthesized concat
/// would change that shape for every plain literal in every fragment.
#[test]
fn plain_double_quoted_literal_stays_one_string_token() {
    assert_eq!(kinds(r#""plain";"#), vec![string("plain"), TokenKind::Semicolon, TokenKind::Eof]);
}

/// Verifies PHP doc comments remain parser-visible while ordinary comments stay trivia.
#[test]
fn doc_comment_is_retained_as_metadata_token() {
    assert_eq!(
        kinds("/** retained */ class Documented {}"),
        vec![
            TokenKind::DocComment("/** retained */".to_string()),
            TokenKind::Ident("class".to_string()),
            TokenKind::Ident("Documented".to_string()),
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

/// Verifies single-quoted literals never interpolate and stay one token.
#[test]
fn single_quoted_literal_never_interpolates() {
    assert_eq!(
        kinds(r#"'$v and {$v}';"#),
        vec![string("$v and {$v}"), TokenKind::Semicolon, TokenKind::Eof]
    );
}

/// Verifies a flexible nowdoc is one literal token with PHP indentation stripping.
#[test]
fn quoted_nowdoc_is_literal_and_strips_closing_indent() {
    assert_eq!(
        kinds("$text = <<<'EOT'\n    $notInterpolated\\n\n    EOT;\n"),
        vec![
            var("text"),
            TokenKind::Equal,
            string("$notInterpolated\\n"),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies an interpolation-free heredoc shares nowdoc closing and indentation rules.
#[test]
fn unquoted_static_heredoc_is_a_literal_token() {
    assert_eq!(
        kinds("$text = <<<EOT\n    static text\n    EOT;\n"),
        vec![
            var("text"),
            TokenKind::Equal,
            string("static text"),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a flexible heredoc interpolates a simple `$name`, which the lexer used to refuse.
///
/// This is the reducer of the fourteen Symfony vendor files the interpreter parser rejected:
/// `$a = "X"; $s = <<<EOT\n    v=$a\n    EOT;` prints `v=X` under `php -n` 8.5.6, while the
/// lexer answered `UnsupportedConstruct` for any heredoc whose body held a `$` at all.
#[test]
fn flexible_heredoc_interpolates_a_simple_variable() {
    assert_eq!(
        kinds("$s = <<<EOT\n    v=$a\n    EOT;\n"),
        vec![
            var("s"),
            TokenKind::Equal,
            TokenKind::LParen,
            string("v="),
            TokenKind::Dot,
            var("a"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a heredoc `{$expr}` produces the same parenthesized fragment a `"…"` does.
///
/// `$arr = ['k' => 'KV']; echo <<<EOT\n    key={$arr['k']}\n    EOT;` prints `key=KV`.
#[test]
fn heredoc_complex_interpolation_matches_the_double_quoted_shape() {
    assert_eq!(
        kinds("<<<EOT\nkey={$a['k']}\nEOT;\n"),
        vec![
            TokenKind::LParen,
            string("key="),
            TokenKind::Dot,
            TokenKind::LParen,
            var("a"),
            TokenKind::LBracket,
            string("k"),
            TokenKind::RBracket,
            TokenKind::RParen,
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies heredoc simple syntax allows one `[offset]` or `->prop`, as a `"…"` does.
///
/// With `$a = ['k' => 'KV']` and an object whose `p` is `PROP`, PHP prints
/// `x KV y PROP` for the body `x $a[k] y $o->p`.
#[test]
fn heredoc_simple_access_matches_the_double_quoted_shape() {
    assert_eq!(
        kinds("<<<EOT\nx $a[k] y $o->p\nEOT;\n"),
        vec![
            TokenKind::LParen,
            string("x "),
            TokenKind::Dot,
            var("a"),
            TokenKind::LBracket,
            string("k"),
            TokenKind::RBracket,
            TokenKind::Dot,
            string(" y "),
            TokenKind::Dot,
            var("o"),
            TokenKind::Arrow,
            TokenKind::Ident("p".to_string()),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a heredoc body expands escapes, because PHP defines it as a double-quoted body.
///
/// `php -n` 8.5.6 prints a real tab for `\t`, a single backslash for `\\`, and `$v` for
/// `\$v`. The one divergence from a `"…"` literal is `\"`, which keeps its backslash: PHP
/// prints `quote:\"` for a heredoc line reading `quote:\"`, since a heredoc has no quote to
/// escape. Before this change the heredoc body was copied verbatim, so `\t` stayed text.
#[test]
fn heredoc_body_expands_escapes_except_the_quote() {
    assert_eq!(
        kinds("<<<EOT\ntab:\\t q:\\\" bs:\\\\ d:\\$v\nEOT;\n"),
        vec![
            string("tab:\t q:\\\" bs:\\ d:$v"),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies the same source inside a `"…"` literal unescapes the quote instead.
///
/// Pinned next to the heredoc case so the single deliberate divergence between the two
/// bodies stays visible in one place.
#[test]
fn double_quoted_body_unescapes_the_quote() {
    assert_eq!(
        kinds(r#""q:\"";"#),
        vec![string("q:\""), TokenKind::Semicolon, TokenKind::Eof]
    );
}

/// Verifies a bare `"` inside a heredoc is ordinary text rather than a terminator.
#[test]
fn heredoc_body_treats_a_quote_as_text() {
    assert_eq!(
        kinds("<<<EOT\nsay \"hi\" $v\nEOT;\n"),
        vec![
            TokenKind::LParen,
            string("say \"hi\" "),
            TokenKind::Dot,
            var("v"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a nowdoc keeps `$name` and `{$expr}` as text even though a heredoc now expands.
///
/// `php -n` 8.5.6 prints `raw $a {$b}` for this body, and the nowdoc arm must not follow
/// the heredoc arm into interpolation.
#[test]
fn nowdoc_with_a_dollar_stays_one_literal_token() {
    assert_eq!(
        kinds("$t = <<<'EOT'\n    raw $a {$b}\n    EOT;\n"),
        vec![
            var("t"),
            TokenKind::Equal,
            string("raw $a {$b}"),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies the legacy `${name}` form interpolates inside a heredoc as it does in `"…"`.
///
/// PHP 8.2 deprecates the spelling but still evaluates it, printing `legacy DOLLARBRACE end`
/// when `$name` holds `DOLLARBRACE`.
#[test]
fn heredoc_legacy_dollar_brace_interpolates() {
    assert_eq!(
        kinds("<<<EOT\nlegacy ${name} end\nEOT;\n"),
        vec![
            TokenKind::LParen,
            string("legacy "),
            TokenKind::Dot,
            TokenKind::LParen,
            var("name"),
            TokenKind::RParen,
            TokenKind::Dot,
            string(" end"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies the closing marker may be followed by a comma inside a call argument list.
///
/// `echo f(<<<EOT\n    one $a\n    EOT, 'two');` prints `one X|two` under `php -n` 8.5.6,
/// so the marker line ends the body and the rest of the line keeps tokenizing.
#[test]
fn heredoc_closing_marker_may_be_followed_by_a_comma() {
    assert_eq!(
        kinds("f(<<<EOT\n    a $v\n    EOT, 1);\n"),
        vec![
            TokenKind::Ident("f".to_string()),
            TokenKind::LParen,
            TokenKind::LParen,
            string("a "),
            TokenKind::Dot,
            var("v"),
            TokenKind::RParen,
            TokenKind::Comma,
            TokenKind::Int(1),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a body line that merely starts with the label does not close the heredoc.
///
/// PHP prints `EOTX is body`, because a label only closes when the next character cannot
/// continue an identifier.
#[test]
fn heredoc_label_prefix_in_the_body_does_not_close_it() {
    assert_eq!(
        kinds("<<<EOT\nEOTX is body\nEOT;\n"),
        vec![
            string("EOTX is body"),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a body line indented less than the closing marker is refused.
///
/// PHP reports `Invalid body indentation level`; this lexer has no wording for that yet and
/// refuses the fragment, which keeps the program from running with silently wrong text.
#[test]
fn heredoc_body_under_the_closing_indent_is_refused() {
    assert_eq!(
        error("<<<EOT\n  a\n    EOT;\n"),
        EvalParseError::UnexpectedToken
    );
}

/// Verifies a blank body line is exempt from the indentation rule.
///
/// PHP prints `a`, an empty line, then `b` for a two-space-indented body around a blank
/// line, so the blank line must not be measured against the closing marker's width.
#[test]
fn heredoc_blank_line_survives_indentation_stripping() {
    assert_eq!(
        kinds("<<<EOT\n  a\n\n  b\n  EOT;\n"),
        vec![string("a\n\nb"), TokenKind::Semicolon, TokenKind::Eof]
    );
}

/// Verifies a `"`-quoted heredoc label interpolates exactly like an unquoted one.
#[test]
fn double_quoted_heredoc_label_still_interpolates() {
    assert_eq!(
        kinds("<<<\"DQ\"\ndq $a\nDQ;\n"),
        vec![
            TokenKind::LParen,
            string("dq "),
            TokenKind::Dot,
            var("a"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies an unterminated heredoc is still refused rather than silently ending at EOF.
#[test]
fn unterminated_heredoc_is_refused() {
    assert_eq!(
        error("<<<EOT\nbody $a\n"),
        EvalParseError::UnterminatedString
    );
}

/// Verifies a non-ASCII identifier scans as one identifier, as PHP's byte rule requires.
///
/// PHP accepts every byte from `0x80` up in a name, so `class Café` and `$café` are legal.
/// The scanner used to stop at `_` and the ASCII letters and refused the rest.
#[test]
fn non_ascii_identifiers_scan_as_one_name() {
    assert_eq!(
        kinds("$café = 5;"),
        vec![
            var("café"),
            TokenKind::Equal,
            TokenKind::Int(5),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
    assert_eq!(
        kinds("class Café {}"),
        vec![
            TokenKind::Ident("class".to_string()),
            TokenKind::Ident("Café".to_string()),
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a binary marker is an identifier character, which is what makes `class ©` work.
///
/// A source byte that is not valid UTF-8 reaches the scanner as this marker. `php -n` 8.5.6
/// runs `class \xA9 { public const V = 5; } echo \xA9::V;` and prints `5`; before the marker
/// counted as an identifier character the fragment failed as invalid UTF-8 instead.
#[test]
fn a_binary_marker_is_an_identifier_character() {
    let marker = char::from_u32(0xF_0000 + 0xA9).expect("marker is a valid code point");
    assert_eq!(
        kinds(&format!("class {marker} {{}}")),
        vec![
            TokenKind::Ident("class".to_string()),
            TokenKind::Ident(marker.to_string()),
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
    assert_eq!(
        kinds(&format!("{marker}::V;")),
        vec![
            TokenKind::Ident(marker.to_string()),
            TokenKind::DoubleColon,
            TokenKind::Ident("V".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies `\$` yields a literal `$` and suppresses interpolation, still as one token.
#[test]
fn escaped_dollar_stays_literal_and_suppresses_interpolation() {
    assert_eq!(
        kinds(r#""\$v";"#),
        vec![string("$v"), TokenKind::Semicolon, TokenKind::Eof]
    );
}

/// Verifies the single-character half of the double-quoted escape table.
///
/// `\q` has no meaning, and PHP keeps both the backslash and the letter for it.
#[test]
fn double_quoted_escape_table_expands_the_named_escapes() {
    assert_eq!(
        kinds(r#""a\nb\tc\\d\"e\q\x41";"#),
        vec![
            string("a\nb\tc\\d\"e\\qA"),
            TokenKind::Semicolon,
            TokenKind::Eof
        ]
    );
}

/// Verifies `\x`, octal and `\u{}` escapes produce the bytes `php -n` 8.5.6 produces.
///
/// Every expectation is that build's `bin2hex()` of the same literal: `\x41`→`41`,
/// `\x4`→`04`, `\x4z`→`047a`, `\x414`→`4134`, `\X41`→`41`, `\xZZ`→`5c785a5a`,
/// `\101`→`41`, `\1`→`01`, `\1011`→`4131`, `\8`→`5c38`, `\u{41}`→`41`,
/// `\u{0000041}`→`41`, `\u41`→`5c753431` and `\U{41}`→`5c557b34317d`. Only a lowercase
/// `u` followed by a brace starts a code-point escape, while `\x` accepts both letter
/// cases, which is why `\X41` is a byte and `\U{41}` is text.
#[test]
fn numeric_escapes_match_php_byte_for_byte() {
    for (source, expected) in [
        (r#""\x41";"#, "A"),
        (r#""\x4";"#, "\u{4}"),
        (r#""\x4z";"#, "\u{4}z"),
        (r#""\x414";"#, "A4"),
        (r#""\X41";"#, "A"),
        (r#""\xZZ";"#, "\\xZZ"),
        (r#""\x";"#, "\\x"),
        (r#""\101";"#, "A"),
        (r#""\1";"#, "\u{1}"),
        (r#""\10";"#, "\u{8}"),
        (r#""\1011";"#, "A1"),
        (r#""\0";"#, "\0"),
        (r#""\8";"#, "\\8"),
        (r#""\u{41}";"#, "A"),
        (r#""\u{0000041}";"#, "A"),
        (r#""\u{1F600}";"#, "\u{1F600}"),
        (r#""\u41";"#, "\\u41"),
        (r#""\U{41}";"#, "\\U{41}"),
    ] {
        assert_eq!(
            kinds(source),
            vec![string(expected), TokenKind::Semicolon, TokenKind::Eof],
            "{source}"
        );
    }
}

/// Verifies a high byte from `\x` or octal becomes the marker the parser turns into bytes.
///
/// `bin2hex("\xFF")` is `ff` under `php -n` 8.5.6: the literal is one byte, not the two
/// bytes UTF-8 would need, so the scanner cannot store it as a `char` and hands the parser
/// the private-use marker for it instead. `\377` names the same byte.
#[test]
fn high_byte_escapes_become_binary_markers() {
    let marker = char::from_u32(0xF_0000 + 0xFF).expect("marker is a valid code point");
    for source in [r#""\xFF";"#, r#""\377";"#] {
        assert_eq!(
            kinds(source),
            vec![
                string(&marker.to_string()),
                TokenKind::Semicolon,
                TokenKind::Eof
            ],
            "{source}"
        );
    }
}

/// Verifies an octal escape past `\377` keeps only its low byte, as PHP does.
///
/// `php -n` 8.5.6 warns `Octal escape sequence overflow \400 is greater than \377` and
/// produces `bin2hex()` `00`; `\777` produces `ff`. This lexer has no warning channel, so
/// the value is pinned here and the missing diagnostic is a known gap.
#[test]
fn overflowing_octal_escape_keeps_its_low_byte() {
    assert_eq!(
        kinds(r#""\400";"#),
        vec![string("\0"), TokenKind::Semicolon, TokenKind::Eof]
    );
    let marker = char::from_u32(0xF_0000 + 0xFF).expect("marker is a valid code point");
    assert_eq!(
        kinds(r#""\777";"#),
        vec![
            string(&marker.to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof
        ]
    );
}

/// Verifies a lone surrogate is encoded rather than refused, matching PHP.
///
/// `bin2hex("\u{D800}")` is `eda080` under `php -n` 8.5.6. Rust's `char` cannot hold a
/// surrogate, so the three bytes arrive as markers.
#[test]
fn surrogate_code_point_escape_is_encoded_as_bytes() {
    let expected: String = [0xED, 0xA0, 0x80]
        .into_iter()
        .map(|byte: u32| char::from_u32(0xF_0000 + byte).expect("marker is a valid code point"))
        .collect();
    assert_eq!(
        kinds(r#""\u{D800}";"#),
        vec![string(&expected), TokenKind::Semicolon, TokenKind::Eof]
    );
}

/// Verifies the four malformed `\u{…}` forms PHP refuses are refused here too.
///
/// `php -n` 8.5.6 reports `Parse error: Invalid UTF-8 codepoint escape sequence` for
/// `"\u{}"`, `"\u{ZZ}"` and `"\u{41"`, and adds `: Codepoint too large` for
/// `"\u{110000}"`. All four must fail rather than degrade into text.
#[test]
fn malformed_code_point_escapes_are_refused() {
    for source in [
        r#""\u{}";"#,
        r#""\u{ZZ}";"#,
        r#""\u{41";"#,
        r#""\u{110000}";"#,
    ] {
        assert_eq!(error(source), EvalParseError::UnexpectedToken, "{source}");
    }
}

/// Verifies a simple `$name` expands to a parenthesized concatenation.
///
/// The leading empty string literal is deliberate: it is what makes the resulting `.`
/// chain string-typed, matching PHP's rule that a double-quoted literal is always a
/// string even when it holds nothing but an integer variable.
#[test]
fn simple_variable_expands_to_parenthesized_concat() {
    assert_eq!(
        kinds(r#""$v";"#),
        vec![
            TokenKind::LParen,
            string(""),
            TokenKind::Dot,
            var("v"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies literal text around an interpolation is concatenated on both sides.
#[test]
fn surrounding_literal_text_is_concatenated() {
    assert_eq!(
        kinds(r#""pre-$v-post";"#),
        vec![
            TokenKind::LParen,
            string("pre-"),
            TokenKind::Dot,
            var("v"),
            TokenKind::Dot,
            string("-post"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a bare `$a[key]` offset becomes a string key, which is PHP's simple syntax.
#[test]
fn bare_offset_key_becomes_a_string_key() {
    assert_eq!(
        kinds(r#""$a[k]";"#),
        vec![
            TokenKind::LParen,
            string(""),
            TokenKind::Dot,
            var("a"),
            TokenKind::LBracket,
            string("k"),
            TokenKind::RBracket,
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a negative simple offset lexes as one negative integer, not unary minus.
#[test]
fn negative_offset_key_becomes_one_negative_integer() {
    assert_eq!(
        kinds(r#""$a[-1]";"#),
        vec![
            TokenKind::LParen,
            string(""),
            TokenKind::Dot,
            var("a"),
            TokenKind::LBracket,
            TokenKind::Int(-1),
            TokenKind::RBracket,
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a variable simple offset keeps the variable rather than a bareword key.
#[test]
fn variable_offset_key_stays_a_variable() {
    assert_eq!(
        kinds(r#""$a[$k]";"#),
        vec![
            TokenKind::LParen,
            string(""),
            TokenKind::Dot,
            var("a"),
            TokenKind::LBracket,
            var("k"),
            TokenKind::RBracket,
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies `$obj->prop` simple syntax emits the property access tokens.
#[test]
fn simple_property_access_is_interpolated() {
    assert_eq!(
        kinds(r#""$o->p";"#),
        vec![
            TokenKind::LParen,
            string(""),
            TokenKind::Dot,
            var("o"),
            TokenKind::Arrow,
            TokenKind::Ident("p".to_string()),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies `->` followed by a digit is literal text, matching PHP 8.5.6.
///
/// Measured: `"$o->1"` interpolates `$o` alone and keeps `->1` as text (it fails with
/// "Object of class Obj could not be converted to string", proving the object itself was
/// the interpolated value). A property name may not start with a digit.
#[test]
fn arrow_followed_by_a_digit_is_literal_text() {
    assert_eq!(
        kinds(r#""$o->1";"#),
        vec![
            TokenKind::LParen,
            string(""),
            TokenKind::Dot,
            var("o"),
            TokenKind::Dot,
            string("->1"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies complex `{$expr}` interpolation nests the fragment inside its own parens.
#[test]
fn complex_interpolation_nests_the_fragment() {
    assert_eq!(
        kinds(r#""{$a['k']}";"#),
        vec![
            TokenKind::LParen,
            string(""),
            TokenKind::Dot,
            TokenKind::LParen,
            var("a"),
            TokenKind::LBracket,
            string("k"),
            TokenKind::RBracket,
            TokenKind::RParen,
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies a `{` not followed by `$` stays literal text, matching PHP's `"{ $x }"`.
#[test]
fn brace_without_dollar_is_literal_text() {
    assert_eq!(
        kinds(r#""{ $s }";"#),
        vec![
            TokenKind::LParen,
            string("{ "),
            TokenKind::Dot,
            var("s"),
            TokenKind::Dot,
            string(" }"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies the legacy `${name}` form interpolates the named variable.
#[test]
fn legacy_dollar_brace_form_interpolates() {
    assert_eq!(
        kinds(r#""${v}";"#),
        vec![
            TokenKind::LParen,
            string(""),
            TokenKind::Dot,
            TokenKind::LParen,
            var("v"),
            TokenKind::RParen,
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies `$` followed by a digit is literal text and the literal stays ONE token.
///
/// Measured on PHP 8.5.6: `echo "$2-$1";` prints `$2-$1`, because a PHP variable name
/// cannot start with a digit. This is not academic — it is exactly how `preg_replace()`
/// back-references are written inside a double-quoted replacement, so treating `$2` as a
/// variable silently breaks every such call. The magician's general `lex_ident()` DOES
/// accept a leading digit, so the interpolation path must gate on `is_ident_start`.
#[test]
fn dollar_followed_by_a_digit_is_literal_text() {
    assert_eq!(
        kinds(r#""$2-$1";"#),
        vec![string("$2-$1"), TokenKind::Semicolon, TokenKind::Eof]
    );
}

/// Verifies a digit-led simple offset variable is refused rather than read as a variable.
///
/// PHP 8.5.6 on `"$a[$2]"`: `syntax error, unexpected token "$"`.
#[test]
fn digit_led_offset_variable_is_refused() {
    assert_eq!(error(r#""$a[$2]";"#), EvalParseError::ExpectedVariable);
}

/// Verifies `$` with no identifier after it is literal text, so `"$$s"` yields `$` + `$s`.
#[test]
fn dollar_without_identifier_is_literal_text() {
    assert_eq!(
        kinds(r#""$$s";"#),
        vec![
            TokenKind::LParen,
            string("$"),
            TokenKind::Dot,
            var("s"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

/// Verifies every synthetic token of a multi-line literal carries the opening-quote line
/// and that the token after the literal resumes at the real line.
///
/// `__LINE__` inside a fragment reads these numbers, so an interpolation that stamped the
/// closing-quote line would silently shift diagnostics.
#[test]
fn interpolated_tokens_carry_the_opening_quote_line() {
    let tokens = tokenize("$x;\n\"a\n$v\nb\";\n$y;").expect("fragment should tokenize");
    let lines: Vec<i64> = tokens.iter().map(super::Token::line).collect();
    // `$x` `;` on line 1, then the literal's nine tokens
    // (`(` `"a\n"` `.` `$v` `.` `"\nb"` `)`) all stamped with its opening-quote line 2,
    // the `;` after the closing quote on line 4, then `$y` `;` on line 5 and EOF.
    assert_eq!(
        kinds("$x;\n\"a\n$v\nb\";\n$y;"),
        vec![
            var("x"),
            TokenKind::Semicolon,
            TokenKind::LParen,
            string("a\n"),
            TokenKind::Dot,
            var("v"),
            TokenKind::Dot,
            string("\nb"),
            TokenKind::RParen,
            TokenKind::Semicolon,
            var("y"),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
    assert_eq!(lines, vec![1, 1, 2, 2, 2, 2, 2, 2, 2, 4, 5, 5, 5]);
}

/// Verifies `"$a[]"` is refused. PHP 8.5.6: `syntax error, unexpected token "]"`.
///
/// The main compiler lexer accepts this and invents an empty-string key; the port must
/// not reproduce that over-acceptance.
#[test]
fn empty_simple_offset_is_refused() {
    assert_eq!(error(r#""$a[]";"#), EvalParseError::UnexpectedToken);
}

/// Verifies `"$a['k']"` is refused. PHP 8.5.6 rejects quoted keys in simple syntax.
#[test]
fn quoted_simple_offset_is_refused() {
    assert_eq!(error(r#""$a['k']";"#), EvalParseError::UnexpectedToken);
}

/// Verifies `"$a[ 0]"` is refused. PHP 8.5.6 rejects whitespace in simple syntax offsets.
#[test]
fn spaced_simple_offset_is_refused() {
    assert_eq!(error(r#""$a[ 0]";"#), EvalParseError::UnexpectedToken);
}

/// Verifies `"$a[-]"` is refused. PHP 8.5.6: `syntax error, unexpected token "]"`.
#[test]
fn lone_minus_simple_offset_is_refused() {
    assert_eq!(error(r#""$a[-]";"#), EvalParseError::InvalidNumber);
}

/// Verifies a simple offset that never closes is refused rather than silently truncated.
#[test]
fn unterminated_simple_offset_is_refused() {
    assert_eq!(error(r#""$a[0";"#), EvalParseError::UnterminatedString);
}

/// Verifies an unterminated `{$...}` is refused.
#[test]
fn unterminated_complex_interpolation_is_refused() {
    assert_eq!(error(r#""{$v";"#), EvalParseError::UnterminatedString);
}

/// Verifies errors from the recursive brace tokenizer propagate instead of being swallowed.
///
/// Swallowing them would turn arbitrary garbage inside `{...}` into silently accepted
/// text, which is the over-acceptance this port is most exposed to.
#[test]
fn garbage_inside_braces_propagates_the_inner_error() {
    assert_eq!(error(r#""{$ }";"#), EvalParseError::ExpectedVariable);
}

/// Verifies an unterminated double-quoted literal is still refused.
#[test]
fn unterminated_double_quoted_literal_is_refused() {
    assert_eq!(error(r#""abc"#), EvalParseError::UnterminatedString);
}

/// Verifies a nested literal inside `{$...}` is captured whole, braces and quotes included.
#[test]
fn nested_literal_inside_braces_is_captured_verbatim() {
    assert_eq!(
        kinds(r#""{$a['}{']}";"#),
        vec![
            TokenKind::LParen,
            string(""),
            TokenKind::Dot,
            TokenKind::LParen,
            var("a"),
            TokenKind::LBracket,
            string("}{"),
            TokenKind::RBracket,
            TokenKind::RParen,
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}
