//! Purpose:
//! Builds PHP's Tokenizer surface — `token_get_all()` and `token_name()` — as prelude
//! declarations, for programs that name either one.
//!
//! Called from:
//! - `crate::backend_gap_prelude::inject_if_used`, which prepends these declarations when the
//!   program references the tokenizer.
//!
//! Key details:
//! - The declarations are BUILT, never parsed: elephc does not parse PHP source outside tests.
//! - This is a TRANSCRIPTION of a reference implementation that was differential-tested against
//!   `php -n` 8.5.10's own `token_get_all()` over 3602 files (the whole Symfony vendor tree plus
//!   every PHP file under `examples/`) and matched the native token stream exactly: same ids,
//!   same texts, same line numbers, same array-vs-string shape for every token.
//! - The rules that look arbitrary were each measured, not guessed:
//!   * `<?php` swallows ONE following whitespace byte (or a `\r\n` pair) into T_OPEN_TAG, and
//!     `?>` swallows ONE following newline into T_CLOSE_TAG.
//!   * A line comment ends at `?>` as well as at a newline.
//!   * A double-quoted string with no interpolation is ONE T_CONSTANT_ENCAPSED_STRING; one with
//!     interpolation is a `"` character token, body pieces, and a closing `"`.
//!   * `->` and `?->` enter php's ST_LOOKING_FOR_PROPERTY: the next label is T_STRING whatever it
//!     spells, and only whitespace and comments may sit between.
//!   * `enum` is semi-reserved -- T_ENUM only when a name follows, so `function enum()` and
//!     `enum(1)` are both T_STRING.
//!   * `&` splits into T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG or the NOT_ variant by looking past
//!     whitespace AND comments for a `$` or a `...`.
//!   * `public(set)` and friends are ONE token whose text is the whole source span.
//! - The token ids come from [`crate::types::token_constants`], so the stream this produces and
//!   the `T_*` constants a program compares it against are read from one table.

use crate::parser::ast::{BinOp, Expr, Program, Stmt, TypeExpr};
use crate::synthetic_class::{
    e_array, e_binop, e_bool, e_call, e_const, e_index, e_int, e_neg, e_not, e_str, e_var, function,
    internal_declarations, s_array_assign, s_array_push, s_assign, s_break, s_continue, s_if,
    s_return, s_while, t_array,
};

/// PHP-visible entry point; everything else is prefixed so it cannot collide with user code.
const TOKEN_GET_ALL: &str = "token_get_all";
const TOKEN_NAME: &str = "token_name";
const IS_SPACE: &str = "__elephc_tok_is_space";
const IS_DIGIT: &str = "__elephc_tok_is_digit";
const IS_LABEL_START: &str = "__elephc_tok_is_label_start";
const IS_LABEL: &str = "__elephc_tok_is_label";
const KEYWORD: &str = "__elephc_tok_keyword";
const CAST: &str = "__elephc_tok_cast";
const OPERATOR: &str = "__elephc_tok_operator";
const HEREDOC_END: &str = "__elephc_tok_heredoc_end";
const INT_OVERFLOWS: &str = "__elephc_tok_int_overflows";

/// Identifiers php's lexer answers with a keyword token instead of T_STRING.
///
/// `die` is an alias of `exit`, and the magic constants are matched case-insensitively like every
/// other keyword, which is why the probe is against a lowercased word.
const KEYWORDS: &[(&str, &str)] = &[
    ("abstract", "T_ABSTRACT"),
    ("and", "T_LOGICAL_AND"),
    ("array", "T_ARRAY"),
    ("as", "T_AS"),
    ("break", "T_BREAK"),
    ("callable", "T_CALLABLE"),
    ("case", "T_CASE"),
    ("catch", "T_CATCH"),
    ("class", "T_CLASS"),
    ("clone", "T_CLONE"),
    ("const", "T_CONST"),
    ("continue", "T_CONTINUE"),
    ("declare", "T_DECLARE"),
    ("default", "T_DEFAULT"),
    ("die", "T_EXIT"),
    ("do", "T_DO"),
    ("echo", "T_ECHO"),
    ("else", "T_ELSE"),
    ("elseif", "T_ELSEIF"),
    ("empty", "T_EMPTY"),
    ("enddeclare", "T_ENDDECLARE"),
    ("endfor", "T_ENDFOR"),
    ("endforeach", "T_ENDFOREACH"),
    ("endif", "T_ENDIF"),
    ("endswitch", "T_ENDSWITCH"),
    ("endwhile", "T_ENDWHILE"),
    ("enum", "T_ENUM"),
    ("eval", "T_EVAL"),
    ("exit", "T_EXIT"),
    ("extends", "T_EXTENDS"),
    ("final", "T_FINAL"),
    ("finally", "T_FINALLY"),
    ("fn", "T_FN"),
    ("for", "T_FOR"),
    ("foreach", "T_FOREACH"),
    ("function", "T_FUNCTION"),
    ("global", "T_GLOBAL"),
    ("goto", "T_GOTO"),
    ("if", "T_IF"),
    ("implements", "T_IMPLEMENTS"),
    ("include", "T_INCLUDE"),
    ("include_once", "T_INCLUDE_ONCE"),
    ("instanceof", "T_INSTANCEOF"),
    ("insteadof", "T_INSTEADOF"),
    ("interface", "T_INTERFACE"),
    ("isset", "T_ISSET"),
    ("list", "T_LIST"),
    ("match", "T_MATCH"),
    ("namespace", "T_NAMESPACE"),
    ("new", "T_NEW"),
    ("or", "T_LOGICAL_OR"),
    ("print", "T_PRINT"),
    ("private", "T_PRIVATE"),
    ("protected", "T_PROTECTED"),
    ("public", "T_PUBLIC"),
    ("readonly", "T_READONLY"),
    ("require", "T_REQUIRE"),
    ("require_once", "T_REQUIRE_ONCE"),
    ("return", "T_RETURN"),
    ("static", "T_STATIC"),
    ("switch", "T_SWITCH"),
    ("throw", "T_THROW"),
    ("trait", "T_TRAIT"),
    ("try", "T_TRY"),
    ("unset", "T_UNSET"),
    ("use", "T_USE"),
    ("var", "T_VAR"),
    ("while", "T_WHILE"),
    ("xor", "T_LOGICAL_XOR"),
    ("yield", "T_YIELD"),
    ("__class__", "T_CLASS_C"),
    ("__dir__", "T_DIR"),
    ("__file__", "T_FILE"),
    ("__function__", "T_FUNC_C"),
    ("__halt_compiler", "T_HALT_COMPILER"),
    ("__line__", "T_LINE"),
    ("__method__", "T_METHOD_C"),
    ("__namespace__", "T_NS_C"),
    ("__property__", "T_PROPERTY_C"),
    ("__trait__", "T_TRAIT_C"),
];

/// Every tokenizer constant, for `token_name()`.
///
/// `T_PAAMAYIM_NEKUDOTAYIM` is deliberately absent: it shares `T_DOUBLE_COLON`'s value and php
/// answers `T_DOUBLE_COLON` for it.
const TOKEN_NAMES: &[&str] = &[
    "T_LNUMBER",
    "T_DNUMBER",
    "T_STRING",
    "T_NAME_FULLY_QUALIFIED",
    "T_NAME_RELATIVE",
    "T_NAME_QUALIFIED",
    "T_VARIABLE",
    "T_INLINE_HTML",
    "T_ENCAPSED_AND_WHITESPACE",
    "T_CONSTANT_ENCAPSED_STRING",
    "T_STRING_VARNAME",
    "T_NUM_STRING",
    "T_INCLUDE",
    "T_INCLUDE_ONCE",
    "T_EVAL",
    "T_REQUIRE",
    "T_REQUIRE_ONCE",
    "T_LOGICAL_OR",
    "T_LOGICAL_XOR",
    "T_LOGICAL_AND",
    "T_PRINT",
    "T_YIELD",
    "T_YIELD_FROM",
    "T_INSTANCEOF",
    "T_NEW",
    "T_CLONE",
    "T_EXIT",
    "T_IF",
    "T_ELSEIF",
    "T_ELSE",
    "T_ENDIF",
    "T_ECHO",
    "T_DO",
    "T_WHILE",
    "T_ENDWHILE",
    "T_FOR",
    "T_ENDFOR",
    "T_FOREACH",
    "T_ENDFOREACH",
    "T_DECLARE",
    "T_ENDDECLARE",
    "T_AS",
    "T_SWITCH",
    "T_ENDSWITCH",
    "T_CASE",
    "T_DEFAULT",
    "T_MATCH",
    "T_BREAK",
    "T_CONTINUE",
    "T_GOTO",
    "T_FUNCTION",
    "T_FN",
    "T_CONST",
    "T_RETURN",
    "T_TRY",
    "T_CATCH",
    "T_FINALLY",
    "T_THROW",
    "T_USE",
    "T_INSTEADOF",
    "T_GLOBAL",
    "T_STATIC",
    "T_ABSTRACT",
    "T_FINAL",
    "T_PRIVATE",
    "T_PROTECTED",
    "T_PUBLIC",
    "T_PRIVATE_SET",
    "T_PROTECTED_SET",
    "T_PUBLIC_SET",
    "T_READONLY",
    "T_VAR",
    "T_UNSET",
    "T_ISSET",
    "T_EMPTY",
    "T_HALT_COMPILER",
    "T_CLASS",
    "T_TRAIT",
    "T_INTERFACE",
    "T_ENUM",
    "T_EXTENDS",
    "T_IMPLEMENTS",
    "T_NAMESPACE",
    "T_LIST",
    "T_ARRAY",
    "T_CALLABLE",
    "T_LINE",
    "T_FILE",
    "T_DIR",
    "T_CLASS_C",
    "T_TRAIT_C",
    "T_METHOD_C",
    "T_FUNC_C",
    "T_PROPERTY_C",
    "T_NS_C",
    "T_ATTRIBUTE",
    "T_PLUS_EQUAL",
    "T_MINUS_EQUAL",
    "T_MUL_EQUAL",
    "T_DIV_EQUAL",
    "T_CONCAT_EQUAL",
    "T_MOD_EQUAL",
    "T_AND_EQUAL",
    "T_OR_EQUAL",
    "T_XOR_EQUAL",
    "T_SL_EQUAL",
    "T_SR_EQUAL",
    "T_COALESCE_EQUAL",
    "T_BOOLEAN_OR",
    "T_BOOLEAN_AND",
    "T_IS_EQUAL",
    "T_IS_NOT_EQUAL",
    "T_IS_IDENTICAL",
    "T_IS_NOT_IDENTICAL",
    "T_IS_SMALLER_OR_EQUAL",
    "T_IS_GREATER_OR_EQUAL",
    "T_SPACESHIP",
    "T_SL",
    "T_SR",
    "T_INC",
    "T_DEC",
    "T_INT_CAST",
    "T_DOUBLE_CAST",
    "T_STRING_CAST",
    "T_ARRAY_CAST",
    "T_OBJECT_CAST",
    "T_BOOL_CAST",
    "T_UNSET_CAST",
    "T_VOID_CAST",
    "T_OBJECT_OPERATOR",
    "T_NULLSAFE_OBJECT_OPERATOR",
    "T_DOUBLE_ARROW",
    "T_COMMENT",
    "T_DOC_COMMENT",
    "T_OPEN_TAG",
    "T_OPEN_TAG_WITH_ECHO",
    "T_CLOSE_TAG",
    "T_WHITESPACE",
    "T_START_HEREDOC",
    "T_END_HEREDOC",
    "T_DOLLAR_OPEN_CURLY_BRACES",
    "T_CURLY_OPEN",
    "T_DOUBLE_COLON",
    "T_NS_SEPARATOR",
    "T_ELLIPSIS",
    "T_COALESCE",
    "T_POW",
    "T_POW_EQUAL",
    "T_PIPE",
    "T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG",
    "T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG",
    "T_BAD_CHARACTER",
];

/// Words that make `( word )` a cast rather than a parenthesised constant.
const CASTS: &[(&str, &str)] = &[
    ("array", "T_ARRAY_CAST"),
    ("binary", "T_STRING_CAST"),
    ("bool", "T_BOOL_CAST"),
    ("boolean", "T_BOOL_CAST"),
    ("double", "T_DOUBLE_CAST"),
    ("float", "T_DOUBLE_CAST"),
    ("int", "T_INT_CAST"),
    ("integer", "T_INT_CAST"),
    ("object", "T_OBJECT_CAST"),
    ("real", "T_DOUBLE_CAST"),
    ("string", "T_STRING_CAST"),
    ("unset", "T_UNSET_CAST"),
    ("void", "T_VOID_CAST"),
];

/// Multi-character operators, THREE bytes first so the longest match wins.
const OPERATORS: &[(&str, &str)] = &[
    ("**=", "T_POW_EQUAL"),
    ("...", "T_ELLIPSIS"),
    ("<=>", "T_SPACESHIP"),
    ("===", "T_IS_IDENTICAL"),
    ("!==", "T_IS_NOT_IDENTICAL"),
    ("<<=", "T_SL_EQUAL"),
    (">>=", "T_SR_EQUAL"),
    ("??=", "T_COALESCE_EQUAL"),
    ("?->", "T_NULLSAFE_OBJECT_OPERATOR"),
    ("**", "T_POW"),
    ("++", "T_INC"),
    ("--", "T_DEC"),
    ("=>", "T_DOUBLE_ARROW"),
    ("<=", "T_IS_SMALLER_OR_EQUAL"),
    (">=", "T_IS_GREATER_OR_EQUAL"),
    ("==", "T_IS_EQUAL"),
    ("!=", "T_IS_NOT_EQUAL"),
    ("<>", "T_IS_NOT_EQUAL"),
    ("+=", "T_PLUS_EQUAL"),
    ("-=", "T_MINUS_EQUAL"),
    ("*=", "T_MUL_EQUAL"),
    ("/=", "T_DIV_EQUAL"),
    (".=", "T_CONCAT_EQUAL"),
    ("%=", "T_MOD_EQUAL"),
    ("&=", "T_AND_EQUAL"),
    ("|=", "T_OR_EQUAL"),
    ("^=", "T_XOR_EQUAL"),
    ("&&", "T_BOOLEAN_AND"),
    ("||", "T_BOOLEAN_OR"),
    ("??", "T_COALESCE"),
    ("|>", "T_PIPE"),
    ("::", "T_DOUBLE_COLON"),
    ("->", "T_OBJECT_OPERATOR"),
    ("<<", "T_SL"),
    (">>", "T_SR"),
];

/// Returns the tokenizer declarations for injection.
pub(crate) fn declarations() -> Program {
    internal_declarations(|| {
        vec![
            decl_fn_is_space(),
            decl_fn_is_digit(),
            decl_fn_is_label_start(),
            decl_fn_is_label(),
            decl_fn_keyword(),
            decl_fn_cast(),
            decl_fn_operator(),
            decl_fn_heredoc_end(),
            decl_fn_int_overflows(),
            decl_fn_token_get_all(),
            decl_fn_token_name(),
        ]
    })
}

// -- small expression helpers, so the transcription below reads like the PHP it mirrors --

/// Builds `$code[<index>]`.
fn code_at(index: Expr) -> Expr {
    e_index(e_var("code"), index)
}

/// Builds `$<name> + <n>`.
fn plus(name: &str, n: i64) -> Expr {
    e_binop(e_var(name), BinOp::Add, e_int(n))
}

/// Builds `$<name> = $<name> + <n>;`.
fn advance(name: &str, n: i64) -> Stmt {
    s_assign(name, plus(name, n))
}

/// Builds `$<name> < $len`.
fn in_bounds(name: &str) -> Expr {
    e_binop(e_var(name), BinOp::Lt, e_var("len"))
}

/// Builds `$<name> + <n> < $len`.
fn in_bounds_at(name: &str, n: i64) -> Expr {
    e_binop(plus(name, n), BinOp::Lt, e_var("len"))
}

/// Builds `<left> && <right>`.
fn and(left: Expr, right: Expr) -> Expr {
    e_binop(left, BinOp::And, right)
}

/// Builds `<left> || <right>`.
fn or(left: Expr, right: Expr) -> Expr {
    e_binop(left, BinOp::Or, right)
}

/// Builds `<value> === '<text>'`.
fn is(value: Expr, text: &str) -> Expr {
    e_binop(value, BinOp::StrictEq, e_str(text))
}

/// Builds `$code[$<name> + <n>] === '<text>'`.
fn char_is(name: &str, n: i64, text: &str) -> Expr {
    is(code_at(plus(name, n)), text)
}

/// Builds `substr($code, <from>, <length>)`.
fn slice(from: Expr, length: Expr) -> Expr {
    e_call("substr", vec![e_var("code"), from, length])
}

/// Builds `substr($code, $<from>, $<to> - $<from>)`.
fn span(from: &str, to: &str) -> Expr {
    slice(
        e_var(from),
        e_binop(e_var(to), BinOp::Sub, e_var(from)),
    )
}

/// Builds `$out[] = [<id>, <text>, $line];`.
fn push_token(id: Expr, text: Expr) -> Stmt {
    s_array_push("out", e_array(vec![id, text, e_var("line")]))
}

/// Builds `$line = $line + substr_count($<name>, "\n");`.
fn advance_line(name: &str) -> Stmt {
    s_assign(
        "line",
        e_binop(
            e_var("line"),
            BinOp::Add,
            e_call("substr_count", vec![e_var(name), e_str("\n")]),
        ),
    )
}

/// Builds `while ($<name> < $len && <extra>) { $<name> = $<name> + 1; }`.
fn scan_while(name: &str, extra: Expr) -> Stmt {
    s_while(and(in_bounds(name), extra), vec![advance(name, 1)])
}

/// `__elephc_tok_is_space()` — php's own whitespace set for T_WHITESPACE runs.
fn decl_fn_is_space() -> Stmt {
    let c = e_var("c");
    function(IS_SPACE)
        .param("c", TypeExpr::Str)
        .returns(TypeExpr::Bool)
        .body(vec![s_return(or(
            or(
                or(is(c.clone(), " "), is(c.clone(), "\t")),
                or(is(c.clone(), "\n"), is(c.clone(), "\r")),
            ),
            or(is(c.clone(), "\u{b}"), is(c, "\u{c}")),
        ))])
        .build()
}

/// `__elephc_tok_is_digit()`.
fn decl_fn_is_digit() -> Stmt {
    function(IS_DIGIT)
        .param("c", TypeExpr::Str)
        .returns(TypeExpr::Bool)
        .body(vec![s_return(and(
            e_binop(e_var("c"), BinOp::GtEq, e_str("0")),
            e_binop(e_var("c"), BinOp::LtEq, e_str("9")),
        ))])
        .build()
}

/// `__elephc_tok_is_label_start()` — php labels also admit every byte at or above 0x80.
fn decl_fn_is_label_start() -> Stmt {
    function(IS_LABEL_START)
        .param("c", TypeExpr::Str)
        .returns(TypeExpr::Bool)
        .body(vec![
            s_if(is(e_var("c"), ""), vec![s_return(e_bool(false))], vec![], None),
            s_if(is(e_var("c"), "_"), vec![s_return(e_bool(true))], vec![], None),
            s_if(
                and(
                    e_binop(e_var("c"), BinOp::GtEq, e_str("a")),
                    e_binop(e_var("c"), BinOp::LtEq, e_str("z")),
                ),
                vec![s_return(e_bool(true))],
                vec![],
                None,
            ),
            s_if(
                and(
                    e_binop(e_var("c"), BinOp::GtEq, e_str("A")),
                    e_binop(e_var("c"), BinOp::LtEq, e_str("Z")),
                ),
                vec![s_return(e_bool(true))],
                vec![],
                None,
            ),
            s_return(e_binop(
                e_call("ord", vec![e_var("c")]),
                BinOp::GtEq,
                e_int(128),
            )),
        ])
        .build()
}

/// `__elephc_tok_is_label()`.
fn decl_fn_is_label() -> Stmt {
    function(IS_LABEL)
        .param("c", TypeExpr::Str)
        .returns(TypeExpr::Bool)
        .body(vec![
            s_if(is(e_var("c"), ""), vec![s_return(e_bool(false))], vec![], None),
            s_if(
                e_call(IS_DIGIT, vec![e_var("c")]),
                vec![s_return(e_bool(true))],
                vec![],
                None,
            ),
            s_return(e_call(IS_LABEL_START, vec![e_var("c")])),
        ])
        .build()
}

/// Builds one `if ($w === '<word>') { return <CONST>; }` chain plus a `-1` miss.
fn lookup_chain(name: &str, param: &str, table: &[(&str, &str)], lowercase: bool) -> Stmt {
    let mut body = Vec::new();
    let probe = if lowercase {
        body.push(s_assign(
            "w",
            e_call("strtolower", vec![e_var(param)]),
        ));
        "w"
    } else {
        param
    };
    for (text, constant) in table {
        body.push(s_if(
            is(e_var(probe), text),
            vec![s_return(e_const(constant))],
            vec![],
            None,
        ));
    }
    body.push(s_return(e_neg(e_int(1))));
    function(name)
        .param(param, TypeExpr::Str)
        .returns(TypeExpr::Int)
        .body(body)
        .build()
}

/// `__elephc_tok_keyword()` — the keyword token for one identifier, or -1 for a plain T_STRING.
fn decl_fn_keyword() -> Stmt {
    lookup_chain(KEYWORD, "word", KEYWORDS, true)
}

/// `__elephc_tok_cast()` — the cast token for the word inside `( … )`, or -1.
fn decl_fn_cast() -> Stmt {
    lookup_chain(CAST, "word", CASTS, true)
}

/// `__elephc_tok_operator()` — the token for a two- or three-byte operator, or -1.
fn decl_fn_operator() -> Stmt {
    lookup_chain(OPERATOR, "op", OPERATORS, false)
}

/// `__elephc_tok_heredoc_end()` — the indent+label length ending a heredoc at `$pos`, or -1.
///
/// php 7.3 let the closing label be indented, and that indentation is part of T_END_HEREDOC's
/// text (the body keeps its own, un-stripped). A label immediately followed by another label byte
/// is not the terminator -- `EOTX` does not close `<<<EOT`.
fn decl_fn_heredoc_end() -> Stmt {
    function(HEREDOC_END)
        .param("code", TypeExpr::Str)
        .param("pos", TypeExpr::Int)
        .param("len", TypeExpr::Int)
        .param("label", TypeExpr::Str)
        .returns(TypeExpr::Int)
        .body(vec![
            s_assign("j", e_var("pos")),
            scan_while(
                "j",
                or(is(code_at(e_var("j")), " "), is(code_at(e_var("j")), "\t")),
            ),
            s_assign("n", e_call("strlen", vec![e_var("label")])),
            s_if(
                e_binop(
                    e_binop(e_var("j"), BinOp::Add, e_var("n")),
                    BinOp::Gt,
                    e_var("len"),
                ),
                vec![s_return(e_neg(e_int(1)))],
                vec![],
                None,
            ),
            s_if(
                e_binop(
                    slice(e_var("j"), e_var("n")),
                    BinOp::StrictNotEq,
                    e_var("label"),
                ),
                vec![s_return(e_neg(e_int(1)))],
                vec![],
                None,
            ),
            s_assign("after", e_binop(e_var("j"), BinOp::Add, e_var("n"))),
            s_if(
                and(
                    in_bounds("after"),
                    e_call(IS_LABEL, vec![code_at(e_var("after"))]),
                ),
                vec![s_return(e_neg(e_int(1)))],
                vec![],
                None,
            ),
            s_return(e_binop(e_var("after"), BinOp::Sub, e_var("pos"))),
        ])
        .build()
}

/// `__elephc_tok_int_overflows()` — php answers T_DNUMBER for a decimal integer past PHP_INT_MAX.
fn decl_fn_int_overflows() -> Stmt {
    function(INT_OVERFLOWS)
        .param("digits", TypeExpr::Str)
        .returns(TypeExpr::Bool)
        .body(vec![
            s_assign("d", e_call("ltrim", vec![e_var("digits"), e_str("0")])),
            s_if(is(e_var("d"), ""), vec![s_return(e_bool(false))], vec![], None),
            s_assign("max", e_str("9223372036854775807")),
            s_if(
                e_binop(
                    e_call("strlen", vec![e_var("d")]),
                    BinOp::Gt,
                    e_call("strlen", vec![e_var("max")]),
                ),
                vec![s_return(e_bool(true))],
                vec![],
                None,
            ),
            s_if(
                e_binop(
                    e_call("strlen", vec![e_var("d")]),
                    BinOp::Lt,
                    e_call("strlen", vec![e_var("max")]),
                ),
                vec![s_return(e_bool(false))],
                vec![],
                None,
            ),
            s_return(e_binop(
                e_call("strcmp", vec![e_var("d"), e_var("max")]),
                BinOp::Gt,
                e_int(0),
            )),
        ])
        .build()
}

/// `token_get_all()` — the PHP-visible entry point.
fn decl_fn_token_get_all() -> Stmt {
    let mut body = token_prologue();
    let mut loop_body = vec![s_if(
        e_binop(e_var("mode"), BinOp::StrictEq, e_int(0)),
        inline_html_arm(),
        vec![],
        None,
    )];
    loop_body.push(s_if(
        e_binop(e_var("mode"), BinOp::StrictEq, e_int(1)),
        php_code_arm(),
        vec![],
        None,
    ));
    loop_body.extend(string_body_arm());
    body.push(s_while(in_bounds("i"), loop_body));
    body.push(s_return(e_var("out")));
    function(TOKEN_GET_ALL)
        .param("code", TypeExpr::Str)
        .param_default("flags", TypeExpr::Int, e_int(0))
        .returns(t_array())
        .body(body)
        .build()
}

/// The scanner's whole mutable state, named once.
///
/// `mode` is 0 inline html, 1 php code, 2 inside `"…"`, 3 inside a heredoc body, 4 inside a
/// backtick body. `sp` indexes the four parallel stacks that remember which string a `{$ … }`
/// interpolation has to return to; `depth` counts the braces opened INSIDE that interpolation, so
/// the `}` that closes it can be told from the ones that close an array or a closure within it.
fn token_prologue() -> Vec<Stmt> {
    vec![
        s_assign("out", e_array(Vec::new())),
        s_assign("len", e_call("strlen", vec![e_var("code")])),
        s_assign("i", e_int(0)),
        s_assign("line", e_int(1)),
        s_assign("mode", e_int(0)),
        s_assign("hd_label", e_str("")),
        s_assign("hd_nowdoc", e_bool(false)),
        s_assign("hd_atline", e_bool(true)),
        s_assign("property", e_bool(false)),
        s_assign("depth", e_int(0)),
        s_assign("sp", e_int(0)),
        s_assign("stack_mode", e_array(Vec::new())),
        s_assign("stack_label", e_array(Vec::new())),
        s_assign("stack_nowdoc", e_array(Vec::new())),
        s_assign("stack_depth", e_array(Vec::new())),
    ]
}

/// Inline-html mode: everything up to the next real open tag is one T_INLINE_HTML.
///
/// `<?` alone is NOT an open tag under the default `short_open_tag=Off`, and `<?php` counts only
/// when whitespace follows it, so the search steps over anything else it finds.
fn inline_html_arm() -> Vec<Stmt> {
    vec![
        s_assign("tagPos", e_neg(e_int(1))),
        s_assign("tagKind", e_int(0)),
        s_assign("p", e_var("i")),
        s_while(
            e_binop(e_var("p"), BinOp::Lt, e_var("len")),
            vec![
                s_assign(
                    "q",
                    e_call("strpos", vec![e_var("code"), e_str("<?"), e_var("p")]),
                ),
                s_if(
                    e_binop(e_var("q"), BinOp::StrictEq, e_bool(false)),
                    vec![s_break(1)],
                    vec![],
                    None,
                ),
                s_if(
                    is(
                        e_call("strtolower", vec![slice(e_var("q"), e_int(5))]),
                        "<?php",
                    ),
                    vec![
                        s_assign("after", e_str("\n")),
                        s_if(
                            in_bounds_at("q", 5),
                            vec![s_assign("after", code_at(plus("q", 5)))],
                            vec![],
                            None,
                        ),
                        s_if(
                            e_call(IS_SPACE, vec![e_var("after")]),
                            vec![
                                s_assign("tagPos", e_var("q")),
                                s_assign("tagKind", e_int(1)),
                                s_break(1),
                            ],
                            vec![],
                            None,
                        ),
                    ],
                    vec![],
                    None,
                ),
                s_if(
                    is(slice(e_var("q"), e_int(3)), "<?="),
                    vec![
                        s_assign("tagPos", e_var("q")),
                        s_assign("tagKind", e_int(2)),
                        s_break(1),
                    ],
                    vec![],
                    None,
                ),
                s_assign("p", plus("q", 2)),
            ],
        ),
        s_assign("end", e_var("len")),
        s_if(
            e_binop(e_var("tagPos"), BinOp::GtEq, e_int(0)),
            vec![s_assign("end", e_var("tagPos"))],
            vec![],
            None,
        ),
        s_if(
            e_binop(e_var("end"), BinOp::Gt, e_var("i")),
            vec![
                s_assign("text", span("i", "end")),
                push_token(e_const("T_INLINE_HTML"), e_var("text")),
                advance_line("text"),
                s_assign("i", e_var("end")),
            ],
            vec![],
            None,
        ),
        s_if(
            e_binop(e_var("tagPos"), BinOp::Lt, e_int(0)),
            vec![s_assign("i", e_var("len")), s_continue(1)],
            vec![],
            None,
        ),
        s_if(
            e_binop(e_var("tagKind"), BinOp::StrictEq, e_int(1)),
            vec![
                s_assign("text", slice(e_var("i"), e_int(5))),
                s_assign("j", plus("i", 5)),
                s_if(
                    in_bounds("j"),
                    vec![s_if(
                        and(
                            is(code_at(e_var("j")), "\r"),
                            and(in_bounds_at("j", 1), char_is("j", 1, "\n")),
                        ),
                        vec![
                            s_assign(
                                "text",
                                e_binop(e_var("text"), BinOp::Concat, e_str("\r\n")),
                            ),
                            advance("j", 2),
                        ],
                        vec![(
                            e_call(IS_SPACE, vec![code_at(e_var("j"))]),
                            vec![
                                s_assign(
                                    "text",
                                    e_binop(e_var("text"), BinOp::Concat, code_at(e_var("j"))),
                                ),
                                advance("j", 1),
                            ],
                        )],
                        None,
                    )],
                    vec![],
                    None,
                ),
                push_token(e_const("T_OPEN_TAG"), e_var("text")),
                advance_line("text"),
                s_assign("i", e_var("j")),
            ],
            vec![],
            Some(vec![
                push_token(e_const("T_OPEN_TAG_WITH_ECHO"), e_str("<?=")),
                advance("i", 3),
            ]),
        ),
        s_assign("mode", e_int(1)),
        s_continue(1),
    ]
}

/// PHP-code mode: one token per pass, each arm ending in `continue`.
///
/// The arms are ordered the way php's own lexer resolves ambiguity: `#[` before `#`, `?>` before
/// the `?` operators, a cast before a bare `(`, and the three-byte operators before the two-byte
/// ones.
fn php_code_arm() -> Vec<Stmt> {
    let mut arm = vec![
        s_assign("c", code_at(e_var("i"))),
        // php's ST_LOOKING_FOR_PROPERTY: the label right after `->`/`?->` is a T_STRING whatever
        // it spells. Whitespace and comments do not end the state; anything else does.
        s_assign("wasProperty", e_var("property")),
        s_assign("property", e_bool(false)),
    ];
    arm.push(whitespace_arm());
    arm.push(close_tag_arm());
    arm.push(attribute_arm());
    arm.push(line_comment_arm());
    arm.push(block_comment_arm());
    arm.push(variable_arm());
    arm.push(number_arm());
    arm.push(single_quoted_arm());
    arm.push(double_quoted_arm());
    arm.push(backtick_arm());
    arm.push(heredoc_open_arm());
    arm.push(cast_or_paren_arm());
    arm.push(open_brace_arm());
    arm.push(close_brace_arm());
    arm.push(name_arm());
    arm.push(ampersand_arm());
    arm.extend(operator_tail());
    arm
}

/// A run of whitespace is one T_WHITESPACE and leaves the property state alone.
fn whitespace_arm() -> Stmt {
    s_if(
        e_call(IS_SPACE, vec![e_var("c")]),
        vec![
            s_assign("j", e_var("i")),
            scan_while("j", e_call(IS_SPACE, vec![code_at(e_var("j"))])),
            s_assign("text", span("i", "j")),
            push_token(e_const("T_WHITESPACE"), e_var("text")),
            advance_line("text"),
            s_assign("i", e_var("j")),
            s_assign("property", e_var("wasProperty")),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `?>` ends php mode and swallows ONE following newline.
fn close_tag_arm() -> Stmt {
    s_if(
        and(
            is(e_var("c"), "?"),
            and(in_bounds_at("i", 1), char_is("i", 1, ">")),
        ),
        vec![
            s_assign("text", e_str("?>")),
            s_assign("j", plus("i", 2)),
            s_if(
                in_bounds("j"),
                vec![s_if(
                    and(
                        is(code_at(e_var("j")), "\r"),
                        and(in_bounds_at("j", 1), char_is("j", 1, "\n")),
                    ),
                    vec![
                        s_assign("text", e_binop(e_var("text"), BinOp::Concat, e_str("\r\n"))),
                        advance("j", 2),
                    ],
                    vec![(
                        is(code_at(e_var("j")), "\n"),
                        vec![
                            s_assign("text", e_binop(e_var("text"), BinOp::Concat, e_str("\n"))),
                            advance("j", 1),
                        ],
                    )],
                    None,
                )],
                vec![],
                None,
            ),
            push_token(e_const("T_CLOSE_TAG"), e_var("text")),
            advance_line("text"),
            s_assign("i", e_var("j")),
            s_assign("mode", e_int(0)),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `#[` opens an attribute; the matching `]` is an ordinary character token, so the brace depth
/// is bumped for it the same way `{` is.
fn attribute_arm() -> Stmt {
    s_if(
        and(
            is(e_var("c"), "#"),
            and(in_bounds_at("i", 1), char_is("i", 1, "[")),
        ),
        vec![
            push_token(e_const("T_ATTRIBUTE"), e_str("#[")),
            advance("i", 2),
            advance("depth", 1),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `//` and `#` run to the end of the line OR to a `?>`, whichever comes first.
fn line_comment_arm() -> Stmt {
    s_if(
        or(
            is(e_var("c"), "#"),
            and(
                is(e_var("c"), "/"),
                and(in_bounds_at("i", 1), char_is("i", 1, "/")),
            ),
        ),
        vec![
            s_assign("j", e_var("i")),
            s_while(
                in_bounds("j"),
                vec![
                    s_if(is(code_at(e_var("j")), "\n"), vec![s_break(1)], vec![], None),
                    s_if(
                        and(
                            is(code_at(e_var("j")), "?"),
                            and(in_bounds_at("j", 1), char_is("j", 1, ">")),
                        ),
                        vec![s_break(1)],
                        vec![],
                        None,
                    ),
                    advance("j", 1),
                ],
            ),
            push_token(e_const("T_COMMENT"), span("i", "j")),
            s_assign("i", e_var("j")),
            s_assign("property", e_var("wasProperty")),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `/* … */`, and T_DOC_COMMENT when it opens `/**` and is longer than the empty `/**/`.
fn block_comment_arm() -> Stmt {
    s_if(
        and(
            is(e_var("c"), "/"),
            and(in_bounds_at("i", 1), char_is("i", 1, "*")),
        ),
        vec![
            s_assign(
                "close",
                e_call("strpos", vec![e_var("code"), e_str("*/"), plus("i", 2)]),
            ),
            s_assign("j", e_var("len")),
            s_if(
                e_binop(e_var("close"), BinOp::StrictNotEq, e_bool(false)),
                vec![s_assign("j", plus("close", 2))],
                vec![],
                None,
            ),
            s_assign("text", span("i", "j")),
            s_assign("id", e_const("T_COMMENT")),
            s_if(
                and(
                    e_binop(e_call("strlen", vec![e_var("text")]), BinOp::Gt, e_int(4)),
                    is(
                        e_call("substr", vec![e_var("text"), e_int(0), e_int(3)]),
                        "/**",
                    ),
                ),
                vec![s_assign("id", e_const("T_DOC_COMMENT"))],
                vec![],
                None,
            ),
            push_token(e_var("id"), e_var("text")),
            advance_line("text"),
            s_assign("i", e_var("j")),
            s_assign("property", e_var("wasProperty")),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `$name`. A bare `$` (as in `$$a`) falls through to the single-character tail.
fn variable_arm() -> Stmt {
    s_if(
        and(
            is(e_var("c"), "$"),
            and(
                in_bounds_at("i", 1),
                e_call(IS_LABEL_START, vec![code_at(plus("i", 1))]),
            ),
        ),
        vec![
            s_assign("j", plus("i", 1)),
            scan_while("j", e_call(IS_LABEL, vec![code_at(e_var("j"))])),
            push_token(e_const("T_VARIABLE"), span("i", "j")),
            s_assign("i", e_var("j")),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// Numbers: hex, binary, explicit octal, and decimal with an optional fraction and exponent.
///
/// A decimal integer past PHP_INT_MAX is T_DNUMBER, which is why the overflow probe runs on the
/// underscore-stripped text. `1..2` must not eat the first `.`, so a `.` is only taken as the
/// decimal point when it is not followed by another one.
fn number_arm() -> Stmt {
    let digit_or_underscore = or(
        e_call(IS_DIGIT, vec![code_at(e_var("j"))]),
        is(code_at(e_var("j")), "_"),
    );
    let decimal_tail = vec![
        scan_while("j", digit_or_underscore.clone()),
        s_if(
            and(
                and(in_bounds("j"), is(code_at(e_var("j")), ".")),
                and(
                    in_bounds_at("j", 1),
                    e_call(IS_DIGIT, vec![code_at(plus("j", 1))]),
                ),
            ),
            vec![
                s_assign("isFloat", e_bool(true)),
                advance("j", 1),
                scan_while("j", digit_or_underscore.clone()),
            ],
            vec![(
                and(
                    and(in_bounds("j"), is(code_at(e_var("j")), ".")),
                    e_not(and(in_bounds_at("j", 1), char_is("j", 1, "."))),
                ),
                vec![s_assign("isFloat", e_bool(true)), advance("j", 1)],
            )],
            None,
        ),
        s_if(
            and(
                in_bounds("j"),
                or(is(code_at(e_var("j")), "e"), is(code_at(e_var("j")), "E")),
            ),
            vec![
                s_assign("k", plus("j", 1)),
                s_if(
                    and(
                        in_bounds("k"),
                        or(is(code_at(e_var("k")), "+"), is(code_at(e_var("k")), "-")),
                    ),
                    vec![advance("k", 1)],
                    vec![],
                    None,
                ),
                s_if(
                    and(in_bounds("k"), e_call(IS_DIGIT, vec![code_at(e_var("k"))])),
                    vec![
                        s_assign("isFloat", e_bool(true)),
                        s_assign("j", e_var("k")),
                        scan_while("j", digit_or_underscore),
                    ],
                    vec![],
                    None,
                ),
            ],
            vec![],
            None,
        ),
    ];
    s_if(
        or(
            e_call(IS_DIGIT, vec![e_var("c")]),
            and(
                is(e_var("c"), "."),
                and(
                    in_bounds_at("i", 1),
                    e_call(IS_DIGIT, vec![code_at(plus("i", 1))]),
                ),
            ),
        ),
        vec![
            s_assign("j", e_var("i")),
            s_assign("isFloat", e_bool(false)),
            s_assign("radix", e_int(10)),
            s_if(
                and(
                    is(e_var("c"), "0"),
                    and(
                        in_bounds_at("i", 1),
                        or(char_is("i", 1, "x"), char_is("i", 1, "X")),
                    ),
                ),
                vec![
                    s_assign("radix", e_int(16)),
                    s_assign("j", plus("i", 2)),
                    scan_while(
                        "j",
                        or(
                            or(
                                e_call(IS_DIGIT, vec![code_at(e_var("j"))]),
                                is(code_at(e_var("j")), "_"),
                            ),
                            or(
                                and(
                                    e_binop(code_at(e_var("j")), BinOp::GtEq, e_str("a")),
                                    e_binop(code_at(e_var("j")), BinOp::LtEq, e_str("f")),
                                ),
                                and(
                                    e_binop(code_at(e_var("j")), BinOp::GtEq, e_str("A")),
                                    e_binop(code_at(e_var("j")), BinOp::LtEq, e_str("F")),
                                ),
                            ),
                        ),
                    ),
                ],
                vec![
                    (
                        and(
                            is(e_var("c"), "0"),
                            and(
                                in_bounds_at("i", 1),
                                or(char_is("i", 1, "b"), char_is("i", 1, "B")),
                            ),
                        ),
                        vec![
                            s_assign("radix", e_int(2)),
                            s_assign("j", plus("i", 2)),
                            scan_while(
                                "j",
                                or(
                                    or(
                                        is(code_at(e_var("j")), "0"),
                                        is(code_at(e_var("j")), "1"),
                                    ),
                                    is(code_at(e_var("j")), "_"),
                                ),
                            ),
                        ],
                    ),
                    (
                        and(
                            is(e_var("c"), "0"),
                            and(
                                in_bounds_at("i", 1),
                                or(char_is("i", 1, "o"), char_is("i", 1, "O")),
                            ),
                        ),
                        vec![
                            s_assign("radix", e_int(8)),
                            s_assign("j", plus("i", 2)),
                            scan_while(
                                "j",
                                or(
                                    and(
                                        e_binop(code_at(e_var("j")), BinOp::GtEq, e_str("0")),
                                        e_binop(code_at(e_var("j")), BinOp::LtEq, e_str("7")),
                                    ),
                                    is(code_at(e_var("j")), "_"),
                                ),
                            ),
                        ],
                    ),
                ],
                Some(decimal_tail),
            ),
            s_assign("text", span("i", "j")),
            s_assign("id", e_const("T_LNUMBER")),
            s_if(
                e_var("isFloat"),
                vec![s_assign("id", e_const("T_DNUMBER"))],
                vec![(
                    and(
                        e_binop(e_var("radix"), BinOp::StrictEq, e_int(10)),
                        e_call(
                            INT_OVERFLOWS,
                            vec![e_call(
                                "str_replace",
                                vec![e_str("_"), e_str(""), e_var("text")],
                            )],
                        ),
                    ),
                    vec![s_assign("id", e_const("T_DNUMBER"))],
                )],
                None,
            ),
            push_token(e_var("id"), e_var("text")),
            s_assign("i", e_var("j")),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `'…'` never interpolates, so it is always one T_CONSTANT_ENCAPSED_STRING.
fn single_quoted_arm() -> Stmt {
    s_if(
        is(e_var("c"), "'"),
        vec![
            s_assign("j", plus("i", 1)),
            s_while(
                in_bounds("j"),
                vec![
                    s_if(
                        and(is(code_at(e_var("j")), "\\"), in_bounds_at("j", 1)),
                        vec![advance("j", 2), s_continue(1)],
                        vec![],
                        None,
                    ),
                    s_if(
                        is(code_at(e_var("j")), "'"),
                        vec![advance("j", 1), s_break(1)],
                        vec![],
                        None,
                    ),
                    advance("j", 1),
                ],
            ),
            s_assign("text", span("i", "j")),
            push_token(e_const("T_CONSTANT_ENCAPSED_STRING"), e_var("text")),
            advance_line("text"),
            s_assign("i", e_var("j")),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `"…"` is ONE token when nothing inside it interpolates, and a `"` plus body pieces when
/// something does. The look-ahead below decides which, honouring `\$` and `\{` escapes.
fn double_quoted_arm() -> Stmt {
    s_if(
        is(e_var("c"), "\""),
        vec![
            s_assign("j", plus("i", 1)),
            s_assign("interpolates", e_bool(false)),
            s_while(
                in_bounds("j"),
                vec![
                    s_assign("d", code_at(e_var("j"))),
                    s_if(
                        and(is(e_var("d"), "\\"), in_bounds_at("j", 1)),
                        vec![advance("j", 2), s_continue(1)],
                        vec![],
                        None,
                    ),
                    s_if(is(e_var("d"), "\""), vec![s_break(1)], vec![], None),
                    s_if(
                        and(
                            is(e_var("d"), "$"),
                            and(
                                in_bounds_at("j", 1),
                                or(
                                    e_call(IS_LABEL_START, vec![code_at(plus("j", 1))]),
                                    char_is("j", 1, "{"),
                                ),
                            ),
                        ),
                        vec![s_assign("interpolates", e_bool(true)), s_break(1)],
                        vec![],
                        None,
                    ),
                    s_if(
                        and(
                            is(e_var("d"), "{"),
                            and(in_bounds_at("j", 1), char_is("j", 1, "$")),
                        ),
                        vec![s_assign("interpolates", e_bool(true)), s_break(1)],
                        vec![],
                        None,
                    ),
                    advance("j", 1),
                ],
            ),
            s_if(
                e_not(e_var("interpolates")),
                vec![
                    s_assign("end", e_var("j")),
                    s_if(in_bounds("end"), vec![advance("end", 1)], vec![], None),
                    s_assign("text", span("i", "end")),
                    push_token(e_const("T_CONSTANT_ENCAPSED_STRING"), e_var("text")),
                    advance_line("text"),
                    s_assign("i", e_var("end")),
                    s_continue(1),
                ],
                vec![],
                None,
            ),
            s_array_push("out", e_str("\"")),
            advance("i", 1),
            s_assign("mode", e_int(2)),
            s_assign("hd_nowdoc", e_bool(false)),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// A backtick body always splits, even when nothing interpolates.
fn backtick_arm() -> Stmt {
    s_if(
        is(e_var("c"), "`"),
        vec![
            s_array_push("out", e_str("`")),
            advance("i", 1),
            s_assign("mode", e_int(4)),
            s_assign("hd_nowdoc", e_bool(false)),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `<<<LABEL`, `<<<"LABEL"` and the nowdoc `<<<'LABEL'`. T_START_HEREDOC carries the newline that
/// ends its own line, so the body starts on the next one.
fn heredoc_open_arm() -> Stmt {
    s_if(
        and(
            is(e_var("c"), "<"),
            is(slice(e_var("i"), e_int(3)), "<<<"),
        ),
        vec![
            s_assign("j", plus("i", 3)),
            scan_while(
                "j",
                or(is(code_at(e_var("j")), " "), is(code_at(e_var("j")), "\t")),
            ),
            s_assign("quote", e_str("")),
            s_if(
                and(
                    in_bounds("j"),
                    or(
                        is(code_at(e_var("j")), "\""),
                        is(code_at(e_var("j")), "'"),
                    ),
                ),
                vec![
                    s_assign("quote", code_at(e_var("j"))),
                    advance("j", 1),
                ],
                vec![],
                None,
            ),
            s_assign("labelStart", e_var("j")),
            scan_while("j", e_call(IS_LABEL, vec![code_at(e_var("j"))])),
            s_assign("hd_label", span("labelStart", "j")),
            s_assign("hd_nowdoc", is(e_var("quote"), "'")),
            s_if(
                and(
                    e_binop(e_var("quote"), BinOp::StrictNotEq, e_str("")),
                    and(
                        in_bounds("j"),
                        e_binop(code_at(e_var("j")), BinOp::StrictEq, e_var("quote")),
                    ),
                ),
                vec![advance("j", 1)],
                vec![],
                None,
            ),
            s_if(
                and(in_bounds("j"), is(code_at(e_var("j")), "\r")),
                vec![advance("j", 1)],
                vec![],
                None,
            ),
            s_if(
                and(in_bounds("j"), is(code_at(e_var("j")), "\n")),
                vec![advance("j", 1)],
                vec![],
                None,
            ),
            s_assign("text", span("i", "j")),
            push_token(e_const("T_START_HEREDOC"), e_var("text")),
            advance_line("text"),
            s_assign("i", e_var("j")),
            s_assign("mode", e_int(3)),
            s_assign("hd_atline", e_bool(true)),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `(int)` and friends are ONE token; `(Foo)` is three. The word has to be a cast name AND be
/// closed by `)` with nothing but spaces around it.
fn cast_or_paren_arm() -> Stmt {
    s_if(
        is(e_var("c"), "("),
        vec![
            s_assign("j", plus("i", 1)),
            scan_while(
                "j",
                or(is(code_at(e_var("j")), " "), is(code_at(e_var("j")), "\t")),
            ),
            s_assign("wordStart", e_var("j")),
            scan_while("j", e_call(IS_LABEL, vec![code_at(e_var("j"))])),
            s_assign("word", span("wordStart", "j")),
            s_assign("k", e_var("j")),
            scan_while(
                "k",
                or(is(code_at(e_var("k")), " "), is(code_at(e_var("k")), "\t")),
            ),
            s_if(
                and(
                    e_binop(e_var("word"), BinOp::StrictNotEq, e_str("")),
                    and(in_bounds("k"), is(code_at(e_var("k")), ")")),
                ),
                vec![
                    s_assign("castId", e_call(CAST, vec![e_var("word")])),
                    s_if(
                        e_binop(e_var("castId"), BinOp::GtEq, e_int(0)),
                        vec![
                            push_token(
                                e_var("castId"),
                                slice(
                                    e_var("i"),
                                    e_binop(plus("k", 1), BinOp::Sub, e_var("i")),
                                ),
                            ),
                            s_assign("i", plus("k", 1)),
                            s_continue(1),
                        ],
                        vec![],
                        None,
                    ),
                ],
                vec![],
                None,
            ),
            s_array_push("out", e_str("(")),
            advance("i", 1),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `{` deepens the interpolation brace count so its `}` is not mistaken for the one that closes
/// a `{$ … }`.
fn open_brace_arm() -> Stmt {
    s_if(
        is(e_var("c"), "{"),
        vec![
            s_array_push("out", e_str("{")),
            advance("i", 1),
            advance("depth", 1),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `}` closes a nested brace, or -- at depth zero with a string waiting -- resumes that string.
fn close_brace_arm() -> Stmt {
    s_if(
        is(e_var("c"), "}"),
        vec![
            s_array_push("out", e_str("}")),
            advance("i", 1),
            s_if(
                e_binop(e_var("depth"), BinOp::Gt, e_int(0)),
                vec![advance("depth", -1)],
                vec![(
                    e_binop(e_var("sp"), BinOp::Gt, e_int(0)),
                    vec![
                        advance("sp", -1),
                        s_assign("mode", e_index(e_var("stack_mode"), e_var("sp"))),
                        s_assign("hd_label", e_index(e_var("stack_label"), e_var("sp"))),
                        s_assign("hd_nowdoc", e_index(e_var("stack_nowdoc"), e_var("sp"))),
                        s_assign("depth", e_index(e_var("stack_depth"), e_var("sp"))),
                        s_assign("hd_atline", e_bool(false)),
                    ],
                )],
                None,
            ),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// Identifiers, namespaced names, and the three lexer rules that turn a keyword back into a name.
///
/// A leading `\` makes the whole thing T_NAME_FULLY_QUALIFIED; a `\` INSIDE makes it
/// T_NAME_QUALIFIED, or T_NAME_RELATIVE when the first segment is `namespace`. `yield from` is
/// one token carrying the whitespace between its halves.
fn name_arm() -> Stmt {
    s_if(
        or(
            e_call(IS_LABEL_START, vec![e_var("c")]),
            is(e_var("c"), "\\"),
        ),
        vec![
            s_assign("j", e_var("i")),
            s_assign("fully", e_bool(false)),
            s_if(
                is(e_var("c"), "\\"),
                vec![s_if(
                    and(
                        in_bounds_at("i", 1),
                        e_call(IS_LABEL_START, vec![code_at(plus("i", 1))]),
                    ),
                    vec![
                        s_assign("fully", e_bool(true)),
                        s_assign("j", plus("i", 1)),
                    ],
                    vec![],
                    Some(vec![
                        push_token(e_const("T_NS_SEPARATOR"), e_str("\\")),
                        advance("i", 1),
                        s_continue(1),
                    ]),
                )],
                vec![],
                None,
            ),
            scan_while("j", e_call(IS_LABEL, vec![code_at(e_var("j"))])),
            s_assign("qualified", e_bool(false)),
            s_while(
                and(
                    and(in_bounds("j"), is(code_at(e_var("j")), "\\")),
                    and(
                        in_bounds_at("j", 1),
                        e_call(IS_LABEL_START, vec![code_at(plus("j", 1))]),
                    ),
                ),
                vec![
                    s_assign("qualified", e_bool(true)),
                    advance("j", 1),
                    scan_while("j", e_call(IS_LABEL, vec![code_at(e_var("j"))])),
                ],
            ),
            s_assign("text", span("i", "j")),
            s_if(
                e_var("fully"),
                vec![
                    push_token(e_const("T_NAME_FULLY_QUALIFIED"), e_var("text")),
                    s_assign("i", e_var("j")),
                    s_continue(1),
                ],
                vec![],
                None,
            ),
            s_if(
                e_var("qualified"),
                vec![
                    s_assign(
                        "head",
                        e_call(
                            "strtolower",
                            vec![e_call(
                                "substr",
                                vec![
                                    e_var("text"),
                                    e_int(0),
                                    e_call("strpos", vec![e_var("text"), e_str("\\")]),
                                ],
                            )],
                        ),
                    ),
                    s_if(
                        is(e_var("head"), "namespace"),
                        vec![push_token(e_const("T_NAME_RELATIVE"), e_var("text"))],
                        vec![],
                        Some(vec![push_token(e_const("T_NAME_QUALIFIED"), e_var("text"))]),
                    ),
                    s_assign("i", e_var("j")),
                    s_continue(1),
                ],
                vec![],
                None,
            ),
            s_if(
                is(e_call("strtolower", vec![e_var("text")]), "yield"),
                vec![
                    s_assign("k", e_var("j")),
                    scan_while("k", e_call(IS_SPACE, vec![code_at(e_var("k"))])),
                    s_if(
                        and(
                            e_binop(e_var("k"), BinOp::Gt, e_var("j")),
                            and(
                                is(
                                    e_call(
                                        "strtolower",
                                        vec![slice(e_var("k"), e_int(4))],
                                    ),
                                    "from",
                                ),
                                or(
                                    e_binop(plus("k", 4), BinOp::GtEq, e_var("len")),
                                    e_not(e_call(IS_LABEL, vec![code_at(plus("k", 4))])),
                                ),
                            ),
                        ),
                        vec![
                            s_assign(
                                "full",
                                slice(
                                    e_var("i"),
                                    e_binop(plus("k", 4), BinOp::Sub, e_var("i")),
                                ),
                            ),
                            push_token(e_const("T_YIELD_FROM"), e_var("full")),
                            advance_line("full"),
                            s_assign("i", plus("k", 4)),
                            s_continue(1),
                        ],
                        vec![],
                        None,
                    ),
                ],
                vec![],
                None,
            ),
            s_assign("id", e_const("T_STRING")),
            s_if(
                e_not(e_var("wasProperty")),
                vec![
                    s_assign("id", e_call(KEYWORD, vec![e_var("text")])),
                    s_if(
                        e_binop(e_var("id"), BinOp::Lt, e_int(0)),
                        vec![s_assign("id", e_const("T_STRING"))],
                        vec![],
                        None,
                    ),
                ],
                vec![],
                None,
            ),
            asymmetric_visibility_arm(),
            semi_reserved_enum_arm(),
            push_token(e_var("id"), e_var("text")),
            s_assign("i", e_var("j")),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// php 8.4's asymmetric visibility: `public(set)` is ONE token whose text is the whole source
/// span, whitespace inside it included.
fn asymmetric_visibility_arm() -> Stmt {
    s_if(
        or(
            e_binop(e_var("id"), BinOp::StrictEq, e_const("T_PUBLIC")),
            or(
                e_binop(e_var("id"), BinOp::StrictEq, e_const("T_PRIVATE")),
                e_binop(e_var("id"), BinOp::StrictEq, e_const("T_PROTECTED")),
            ),
        ),
        vec![
            s_assign("k", e_var("j")),
            scan_while("k", e_call(IS_SPACE, vec![code_at(e_var("k"))])),
            s_if(
                and(in_bounds("k"), is(code_at(e_var("k")), "(")),
                vec![
                    advance("k", 1),
                    scan_while("k", e_call(IS_SPACE, vec![code_at(e_var("k"))])),
                    s_if(
                        and(
                            is(
                                e_call("strtolower", vec![slice(e_var("k"), e_int(3))]),
                                "set",
                            ),
                            or(
                                e_binop(plus("k", 3), BinOp::GtEq, e_var("len")),
                                e_not(e_call(IS_LABEL, vec![code_at(plus("k", 3))])),
                            ),
                        ),
                        vec![
                            advance("k", 3),
                            scan_while("k", e_call(IS_SPACE, vec![code_at(e_var("k"))])),
                            s_if(
                                and(in_bounds("k"), is(code_at(e_var("k")), ")")),
                                vec![
                                    advance("k", 1),
                                    s_assign("setId", e_const("T_PUBLIC_SET")),
                                    s_if(
                                        e_binop(
                                            e_var("id"),
                                            BinOp::StrictEq,
                                            e_const("T_PRIVATE"),
                                        ),
                                        vec![s_assign("setId", e_const("T_PRIVATE_SET"))],
                                        vec![(
                                            e_binop(
                                                e_var("id"),
                                                BinOp::StrictEq,
                                                e_const("T_PROTECTED"),
                                            ),
                                            vec![s_assign("setId", e_const("T_PROTECTED_SET"))],
                                        )],
                                        None,
                                    ),
                                    s_assign("full", span("i", "k")),
                                    push_token(e_var("setId"), e_var("full")),
                                    advance_line("full"),
                                    s_assign("i", e_var("k")),
                                    s_continue(1),
                                ],
                                vec![],
                                None,
                            ),
                        ],
                        vec![],
                        None,
                    ),
                ],
                vec![],
                None,
            ),
        ],
        vec![],
        None,
    )
}

/// `enum` is semi-reserved: it only opens a declaration when a name follows. `function enum()`
/// and `enum(1)` are both ordinary T_STRING.
fn semi_reserved_enum_arm() -> Stmt {
    s_if(
        e_binop(e_var("id"), BinOp::StrictEq, e_const("T_ENUM")),
        vec![
            s_assign("k", e_var("j")),
            scan_while("k", e_call(IS_SPACE, vec![code_at(e_var("k"))])),
            s_if(
                or(
                    e_binop(e_var("k"), BinOp::StrictEq, e_var("j")),
                    or(
                        e_binop(e_var("k"), BinOp::GtEq, e_var("len")),
                        e_not(e_call(IS_LABEL_START, vec![code_at(e_var("k"))])),
                    ),
                ),
                vec![s_assign("id", e_const("T_STRING"))],
                vec![],
                None,
            ),
        ],
        vec![],
        None,
    )
}

/// A lone `&` splits by what follows it, past whitespace AND comments: a `$` or a `...` makes it
/// the by-reference token, anything else the bitwise one. `&&` and `&=` are operators and are
/// left to the operator tail.
fn ampersand_arm() -> Stmt {
    s_if(
        and(
            is(e_var("c"), "&"),
            e_not(and(
                in_bounds_at("i", 1),
                or(char_is("i", 1, "&"), char_is("i", 1, "=")),
            )),
        ),
        vec![
            s_assign("k", plus("i", 1)),
            s_while(
                in_bounds("k"),
                vec![
                    s_if(
                        e_call(IS_SPACE, vec![code_at(e_var("k"))]),
                        vec![advance("k", 1), s_continue(1)],
                        vec![],
                        None,
                    ),
                    s_if(
                        and(
                            is(code_at(e_var("k")), "/"),
                            and(in_bounds_at("k", 1), char_is("k", 1, "*")),
                        ),
                        vec![
                            s_assign(
                                "close",
                                e_call("strpos", vec![e_var("code"), e_str("*/"), plus("k", 2)]),
                            ),
                            s_if(
                                e_binop(e_var("close"), BinOp::StrictEq, e_bool(false)),
                                vec![s_assign("k", e_var("len")), s_break(1)],
                                vec![],
                                None,
                            ),
                            s_assign("k", plus("close", 2)),
                            s_continue(1),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        or(
                            and(
                                is(code_at(e_var("k")), "/"),
                                and(in_bounds_at("k", 1), char_is("k", 1, "/")),
                            ),
                            and(
                                is(code_at(e_var("k")), "#"),
                                e_not(and(in_bounds_at("k", 1), char_is("k", 1, "["))),
                            ),
                        ),
                        vec![
                            s_while(
                                and(
                                    in_bounds("k"),
                                    e_binop(
                                        code_at(e_var("k")),
                                        BinOp::StrictNotEq,
                                        e_str("\n"),
                                    ),
                                ),
                                vec![advance("k", 1)],
                            ),
                            s_continue(1),
                        ],
                        vec![],
                        None,
                    ),
                    s_break(1),
                ],
            ),
            s_assign("byRef", e_bool(false)),
            s_if(
                and(in_bounds("k"), is(code_at(e_var("k")), "$")),
                vec![s_assign("byRef", e_bool(true))],
                vec![(
                    is(slice(e_var("k"), e_int(3)), "..."),
                    vec![s_assign("byRef", e_bool(true))],
                )],
                None,
            ),
            s_assign("id", e_const("T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG")),
            s_if(
                e_var("byRef"),
                vec![s_assign(
                    "id",
                    e_const("T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG"),
                )],
                vec![],
                None,
            ),
            push_token(e_var("id"), e_str("&")),
            advance("i", 1),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// Three-byte operators, then two-byte ones, then the single character itself.
///
/// `->` and `?->` are what arm php's property-name state for the next label.
fn operator_tail() -> Vec<Stmt> {
    vec![
        s_assign("three", slice(e_var("i"), e_int(3))),
        s_assign("id", e_call(OPERATOR, vec![e_var("three")])),
        s_if(
            e_binop(e_var("id"), BinOp::GtEq, e_int(0)),
            vec![
                push_token(e_var("id"), e_var("three")),
                advance("i", 3),
                s_if(
                    e_binop(
                        e_var("id"),
                        BinOp::StrictEq,
                        e_const("T_NULLSAFE_OBJECT_OPERATOR"),
                    ),
                    vec![s_assign("property", e_bool(true))],
                    vec![],
                    None,
                ),
                s_continue(1),
            ],
            vec![],
            None,
        ),
        s_assign("two", slice(e_var("i"), e_int(2))),
        s_assign("id", e_call(OPERATOR, vec![e_var("two")])),
        s_if(
            e_binop(e_var("id"), BinOp::GtEq, e_int(0)),
            vec![
                push_token(e_var("id"), e_var("two")),
                advance("i", 2),
                s_if(
                    e_binop(e_var("id"), BinOp::StrictEq, e_const("T_OBJECT_OPERATOR")),
                    vec![s_assign("property", e_bool(true))],
                    vec![],
                    None,
                ),
                s_continue(1),
            ],
            vec![],
            None,
        ),
        s_array_push("out", e_var("c")),
        advance("i", 1),
        s_continue(1),
    ]
}

/// String-like bodies: 2 = `"…"`, 3 = a heredoc body, 4 = a backtick body.
///
/// One pass emits the literal run up to whatever interrupts it, then handles that interruption.
/// A nowdoc body has no interruptions at all except its own end label, which is why every probe
/// below is guarded on `hd_nowdoc`.
fn string_body_arm() -> Vec<Stmt> {
    let mut arm = vec![
        s_assign("start", e_var("i")),
        s_assign("stop", e_neg(e_int(1))),
        // 1 terminator, 2 a simple `$var`, 3 a `${`, 4 a `{$`.
        s_assign("kind", e_int(0)),
        s_while(in_bounds("i"), string_body_scan()),
        s_if(
            e_binop(e_var("stop"), BinOp::Lt, e_int(0)),
            vec![s_assign("stop", e_var("len"))],
            vec![],
            None,
        ),
        s_if(
            e_binop(e_var("stop"), BinOp::Gt, e_var("start")),
            vec![
                s_assign("text", span("start", "stop")),
                push_token(e_const("T_ENCAPSED_AND_WHITESPACE"), e_var("text")),
                advance_line("text"),
            ],
            vec![],
            None,
        ),
        s_assign("i", e_var("stop")),
        s_if(
            e_binop(e_var("i"), BinOp::GtEq, e_var("len")),
            vec![s_break(1)],
            vec![],
            None,
        ),
        string_terminator_arm(),
        simple_interpolation_arm(),
        dollar_brace_arm(),
    ];
    arm.extend(curly_open_arm());
    arm
}

/// Walks the literal run, stopping at the terminator or at an interpolation.
fn string_body_scan() -> Vec<Stmt> {
    vec![
        s_if(
            and(
                e_binop(e_var("mode"), BinOp::StrictEq, e_int(3)),
                e_var("hd_atline"),
            ),
            vec![
                s_assign(
                    "endLen",
                    e_call(
                        HEREDOC_END,
                        vec![e_var("code"), e_var("i"), e_var("len"), e_var("hd_label")],
                    ),
                ),
                s_if(
                    e_binop(e_var("endLen"), BinOp::GtEq, e_int(0)),
                    vec![
                        s_assign("stop", e_var("i")),
                        s_assign("kind", e_int(1)),
                        s_break(1),
                    ],
                    vec![],
                    None,
                ),
            ],
            vec![],
            None,
        ),
        s_assign("hd_atline", e_bool(false)),
        s_assign("d", code_at(e_var("i"))),
        s_if(
            and(
                and(is(e_var("d"), "\\"), in_bounds_at("i", 1)),
                e_not(e_var("hd_nowdoc")),
            ),
            vec![advance("i", 2), s_continue(1)],
            vec![],
            None,
        ),
        s_if(
            and(
                e_binop(e_var("mode"), BinOp::StrictEq, e_int(2)),
                is(e_var("d"), "\""),
            ),
            vec![
                s_assign("stop", e_var("i")),
                s_assign("kind", e_int(1)),
                s_break(1),
            ],
            vec![],
            None,
        ),
        s_if(
            and(
                e_binop(e_var("mode"), BinOp::StrictEq, e_int(4)),
                is(e_var("d"), "`"),
            ),
            vec![
                s_assign("stop", e_var("i")),
                s_assign("kind", e_int(1)),
                s_break(1),
            ],
            vec![],
            None,
        ),
        s_if(
            and(
                e_not(e_var("hd_nowdoc")),
                and(
                    is(e_var("d"), "$"),
                    and(
                        in_bounds_at("i", 1),
                        e_call(IS_LABEL_START, vec![code_at(plus("i", 1))]),
                    ),
                ),
            ),
            vec![
                s_assign("stop", e_var("i")),
                s_assign("kind", e_int(2)),
                s_break(1),
            ],
            vec![],
            None,
        ),
        s_if(
            and(
                e_not(e_var("hd_nowdoc")),
                and(
                    is(e_var("d"), "$"),
                    and(in_bounds_at("i", 1), char_is("i", 1, "{")),
                ),
            ),
            vec![
                s_assign("stop", e_var("i")),
                s_assign("kind", e_int(3)),
                s_break(1),
            ],
            vec![],
            None,
        ),
        s_if(
            and(
                e_not(e_var("hd_nowdoc")),
                and(
                    is(e_var("d"), "{"),
                    and(in_bounds_at("i", 1), char_is("i", 1, "$")),
                ),
            ),
            vec![
                s_assign("stop", e_var("i")),
                s_assign("kind", e_int(4)),
                s_break(1),
            ],
            vec![],
            None,
        ),
        s_if(
            is(e_var("d"), "\n"),
            vec![s_assign("hd_atline", e_bool(true))],
            vec![],
            None,
        ),
        advance("i", 1),
    ]
}

/// The body's own terminator: T_END_HEREDOC with its indentation, or the closing quote.
fn string_terminator_arm() -> Stmt {
    s_if(
        e_binop(e_var("kind"), BinOp::StrictEq, e_int(1)),
        vec![
            s_if(
                e_binop(e_var("mode"), BinOp::StrictEq, e_int(3)),
                vec![
                    s_assign(
                        "endLen",
                        e_call(
                            HEREDOC_END,
                            vec![e_var("code"), e_var("i"), e_var("len"), e_var("hd_label")],
                        ),
                    ),
                    push_token(
                        e_const("T_END_HEREDOC"),
                        slice(e_var("i"), e_var("endLen")),
                    ),
                    s_assign("i", e_binop(e_var("i"), BinOp::Add, e_var("endLen"))),
                ],
                vec![(
                    e_binop(e_var("mode"), BinOp::StrictEq, e_int(2)),
                    vec![s_array_push("out", e_str("\"")), advance("i", 1)],
                )],
                Some(vec![s_array_push("out", e_str("`")), advance("i", 1)]),
            ),
            s_assign("mode", e_int(1)),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `$name`, optionally followed by ONE `[…]` index or ONE `->prop`. php stops there: `"$a->b->c"`
/// interpolates `$a->b` and leaves `->c` as literal text.
fn simple_interpolation_arm() -> Stmt {
    s_if(
        e_binop(e_var("kind"), BinOp::StrictEq, e_int(2)),
        vec![
            s_assign("j", plus("i", 1)),
            scan_while("j", e_call(IS_LABEL, vec![code_at(e_var("j"))])),
            push_token(e_const("T_VARIABLE"), span("i", "j")),
            s_assign("i", e_var("j")),
            s_if(
                and(in_bounds("i"), is(code_at(e_var("i")), "[")),
                vec![
                    s_array_push("out", e_str("[")),
                    advance("i", 1),
                    s_if(
                        and(in_bounds("i"), is(code_at(e_var("i")), "$")),
                        vec![
                            s_assign("k", plus("i", 1)),
                            scan_while("k", e_call(IS_LABEL, vec![code_at(e_var("k"))])),
                            push_token(e_const("T_VARIABLE"), span("i", "k")),
                            s_assign("i", e_var("k")),
                        ],
                        vec![],
                        Some(vec![
                            s_assign("k", e_var("i")),
                            s_if(
                                and(in_bounds("k"), is(code_at(e_var("k")), "-")),
                                vec![advance("k", 1)],
                                vec![],
                                None,
                            ),
                            s_assign(
                                "numeric",
                                and(in_bounds("k"), e_call(IS_DIGIT, vec![code_at(e_var("k"))])),
                            ),
                            s_while(
                                and(
                                    in_bounds("k"),
                                    e_binop(code_at(e_var("k")), BinOp::StrictNotEq, e_str("]")),
                                ),
                                vec![advance("k", 1)],
                            ),
                            s_assign("inner", span("i", "k")),
                            s_assign("id", e_const("T_STRING")),
                            s_if(
                                e_var("numeric"),
                                vec![s_assign("id", e_const("T_NUM_STRING"))],
                                vec![],
                                None,
                            ),
                            s_if(
                                e_binop(e_var("inner"), BinOp::StrictNotEq, e_str("")),
                                vec![push_token(e_var("id"), e_var("inner"))],
                                vec![],
                                None,
                            ),
                            s_assign("i", e_var("k")),
                        ]),
                    ),
                    s_if(
                        and(in_bounds("i"), is(code_at(e_var("i")), "]")),
                        vec![s_array_push("out", e_str("]")), advance("i", 1)],
                        vec![],
                        None,
                    ),
                ],
                vec![(
                    and(
                        and(in_bounds_at("i", 1), is(code_at(e_var("i")), "-")),
                        and(
                            char_is("i", 1, ">"),
                            and(
                                in_bounds_at("i", 2),
                                e_call(IS_LABEL_START, vec![code_at(plus("i", 2))]),
                            ),
                        ),
                    ),
                    vec![
                        push_token(e_const("T_OBJECT_OPERATOR"), e_str("->")),
                        advance("i", 2),
                        s_assign("k", e_var("i")),
                        scan_while("k", e_call(IS_LABEL, vec![code_at(e_var("k"))])),
                        push_token(e_const("T_STRING"), span("i", "k")),
                        s_assign("i", e_var("k")),
                    ],
                )],
                None,
            ),
            s_continue(1),
        ],
        vec![],
        None,
    )
}

/// `${name}` is a T_STRING_VARNAME between braces; `${expr}` re-enters php code instead.
fn dollar_brace_arm() -> Stmt {
    let mut body = vec![
        s_assign("j", plus("i", 2)),
        s_assign("nameStart", e_var("j")),
        scan_while("j", e_call(IS_LABEL, vec![code_at(e_var("j"))])),
        s_if(
            and(
                e_binop(e_var("j"), BinOp::Gt, e_var("nameStart")),
                and(in_bounds("j"), is(code_at(e_var("j")), "}")),
            ),
            vec![
                push_token(e_const("T_DOLLAR_OPEN_CURLY_BRACES"), e_str("${")),
                push_token(e_const("T_STRING_VARNAME"), span("nameStart", "j")),
                s_array_push("out", e_str("}")),
                s_assign("i", plus("j", 1)),
                s_continue(1),
            ],
            vec![],
            None,
        ),
        push_token(e_const("T_DOLLAR_OPEN_CURLY_BRACES"), e_str("${")),
    ];
    body.extend(push_interpolation_frame(2));
    s_if(
        e_binop(e_var("kind"), BinOp::StrictEq, e_int(3)),
        body,
        vec![],
        None,
    )
}

/// `{$ … }` re-enters php code until its own `}`.
fn curly_open_arm() -> Vec<Stmt> {
    let mut arm = vec![push_token(e_const("T_CURLY_OPEN"), e_str("{"))];
    arm.extend(push_interpolation_frame(1));
    arm
}

/// Remembers which string to come back to, then switches to php-code mode.
fn push_interpolation_frame(consumed: i64) -> Vec<Stmt> {
    vec![
        s_array_assign("stack_mode", e_var("sp"), e_var("mode")),
        s_array_assign("stack_label", e_var("sp"), e_var("hd_label")),
        s_array_assign("stack_nowdoc", e_var("sp"), e_var("hd_nowdoc")),
        s_array_assign("stack_depth", e_var("sp"), e_var("depth")),
        advance("sp", 1),
        s_assign("mode", e_int(1)),
        s_assign("depth", e_int(0)),
        advance("i", consumed),
        s_continue(1),
    ]
}

/// `token_name()` — the constant's spelling for one token id, or `'UNKNOWN'`.
///
/// php answers `'UNKNOWN'` for every value that is not a token id, single ASCII characters
/// included, so `token_name(ord(';'))` is `'UNKNOWN'` and not `';'`.
fn decl_fn_token_name() -> Stmt {
    let mut body = Vec::new();
    for name in TOKEN_NAMES {
        body.push(s_if(
            e_binop(e_var("id"), BinOp::StrictEq, e_const(name)),
            vec![s_return(e_str(name))],
            vec![],
            None,
        ));
    }
    body.push(s_return(e_str("UNKNOWN")));
    function(TOKEN_NAME)
        .param("id", TypeExpr::Int)
        .returns(TypeExpr::Str)
        .body(body)
        .build()
}
