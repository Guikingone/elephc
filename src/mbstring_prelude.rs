//! Purpose:
//! Builds the UTF-8 mbstring functions elephc has no native implementation for, as prelude
//! declarations.
//!
//! Called from:
//! - `crate::backend_gap_prelude::inject_if_used`, which prepends exactly the functions the
//!   program calls and does not declare itself.
//!
//! Key details:
//! - Only `mb_strlen`, `mb_strtolower`, `mb_strtoupper` and `mb_ereg_match` are native. Every
//!   other mbstring call compiled and died at run time with `Call to undefined function
//!   mb_substr()`, while `php -n` ships all of them.
//! - Each function walks UTF-8 code points by their lead byte (`__elephc_mb_advance`) and hands
//!   byte offsets to the byte-string builtins, so positions and lengths count characters as php
//!   counts them for valid UTF-8. The `$encoding` arguments are accepted and ignored: the only
//!   encoding supported is UTF-8, as `mb_internal_encoding()` reports.
//! - A program declaring its own function of the same name (`symfony/polyfill-mbstring` declares
//!   them behind `function_exists()`) keeps it: that name is not injected, so nothing is declared
//!   twice. `extension_loaded('mbstring')` still answers false, so polyfills stay active.
//! - The declarations are BUILT, never parsed: elephc does not parse PHP source outside tests.

use crate::parser::ast::{BinOp, Expr, Program, Stmt, StmtKind, TypeExpr};
use crate::synthetic_class::{
    e_binop, e_bool, e_call, e_concat, e_index, e_int, e_new, e_not, e_null, e_str, e_ternary,
    e_var, function, internal_declarations, s_array_push, s_assign, s_if, s_return, s_throw,
    s_while, t_array, t_mixed, t_nullable,
};

/// Internal helper every function uses to turn a character count into a byte offset.
const ADVANCE: &str = "__elephc_mb_advance";
/// Internal helper `mb_str_pad()` uses to repeat a padding string over a character count.
const REPEAT: &str = "__elephc_mb_repeat";

/// The public functions this prelude provides, in declaration order.
pub(crate) const FUNCTIONS: &[&str] = &[
    "mb_substr",
    "mb_strpos",
    "mb_stripos",
    "mb_strrpos",
    "mb_strripos",
    "mb_str_split",
    "mb_substr_count",
    "mb_strstr",
    "mb_stristr",
    "mb_strrchr",
    "mb_internal_encoding",
    "mb_check_encoding",
    "mb_str_pad",
    "mb_ucfirst",
    "mb_lcfirst",
];

/// Returns the declarations for the named public functions plus the helpers they need.
pub(crate) fn declarations(selected: &[&str]) -> Program {
    internal_declarations(|| {
        let mut out = vec![advance()];
        if selected.contains(&"mb_str_pad") {
            out.push(repeat());
        }
        for name in FUNCTIONS {
            if !selected.contains(name) {
                continue;
            }
            out.push(match *name {
                "mb_substr" => mb_substr(),
                "mb_strpos" => mb_strpos("mb_strpos", "strpos", false),
                "mb_stripos" => mb_strpos("mb_stripos", "strpos", true),
                "mb_strrpos" => mb_strpos("mb_strrpos", "strrpos", false),
                "mb_strripos" => mb_strpos("mb_strripos", "strrpos", true),
                "mb_str_split" => mb_str_split(),
                "mb_substr_count" => mb_substr_count(),
                "mb_strstr" => mb_strstr("mb_strstr", "strpos", false),
                "mb_stristr" => mb_strstr("mb_stristr", "strpos", true),
                "mb_strrchr" => mb_strstr("mb_strrchr", "strrpos", false),
                "mb_internal_encoding" => mb_internal_encoding(),
                "mb_check_encoding" => mb_check_encoding(),
                "mb_str_pad" => mb_str_pad(),
                "mb_ucfirst" => mb_case_first("mb_ucfirst", "mb_strtoupper"),
                "mb_lcfirst" => mb_case_first("mb_lcfirst", "mb_strtolower"),
                other => unreachable!("mbstring prelude has no builder for {other}"),
            });
        }
        out
    })
}

/// Returns whether the program declares a function named `name` ANYWHERE, a conditional
/// declaration included.
///
/// A polyfill declares its functions behind `if (!function_exists(...))`; a top-level-only scan
/// missed them, both bodies were emitted, and the link failed on a duplicate symbol.
pub(crate) fn program_declares_anywhere(program: &[Stmt], name: &str) -> bool {
    program.iter().any(|stmt| stmt_declares_anywhere(stmt, name))
}

fn names_function(declared: &str, name: &str) -> bool {
    declared
        .rsplit('\\')
        .next()
        .is_some_and(|last| last.eq_ignore_ascii_case(name))
}

fn stmt_declares_anywhere(stmt: &Stmt, name: &str) -> bool {
    let any = |body: &[Stmt]| body.iter().any(|stmt| stmt_declares_anywhere(stmt, name));
    match &stmt.kind {
        StmtKind::FunctionDecl { name: declared, .. }
        | StmtKind::FunctionVariantGroup { name: declared, .. }
        | StmtKind::FunctionVariantMark { name: declared, .. } => names_function(declared, name),
        StmtKind::If { then_body, elseif_clauses, else_body, .. } => {
            any(then_body)
                || elseif_clauses.iter().any(|(_, body)| any(body))
                || else_body.as_deref().is_some_and(any)
        }
        StmtKind::IfDef { then_body, else_body, .. } => {
            any(then_body) || else_body.as_deref().is_some_and(any)
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => any(body),
        StmtKind::Switch { cases, default, .. } => {
            cases.iter().any(|(_, body)| any(body)) || default.as_deref().is_some_and(any)
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            any(try_body)
                || catches.iter().any(|catch_clause| any(&catch_clause.body))
                || finally_body.as_deref().is_some_and(any)
        }
        _ => false,
    }
}

fn v(name: &str) -> Expr {
    e_var(name)
}

fn bin(left: Expr, op: BinOp, right: Expr) -> Expr {
    e_binop(left, op, right)
}

fn int(value: i64) -> Expr {
    e_int(value)
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    e_call(name, args)
}

fn if_then(condition: Expr, body: Vec<Stmt>) -> Stmt {
    s_if(condition, body, Vec::new(), None)
}

fn value_error(message: &str) -> Stmt {
    s_throw(e_new("ValueError", vec![e_str(message)]))
}

fn nullable_string() -> TypeExpr {
    t_nullable(TypeExpr::Str)
}

/// `__elephc_mb_advance(string $s, int $byte, int $chars): int` — the byte offset reached after
/// `$chars` code points from `$byte`, clamped to the string's length.
fn advance() -> Stmt {
    let step = |n: i64| vec![s_assign("byte", bin(v("byte"), BinOp::Add, int(n)))];
    function(ADVANCE)
        .param("s", TypeExpr::Str)
        .param("byte", TypeExpr::Int)
        .param("chars", TypeExpr::Int)
        .returns(TypeExpr::Int)
        .body(vec![
            s_assign("len", call("strlen", vec![v("s")])),
            s_while(
                bin(
                    bin(v("chars"), BinOp::Gt, int(0)),
                    BinOp::And,
                    bin(v("byte"), BinOp::Lt, v("len")),
                ),
                vec![
                    s_assign("b", call("ord", vec![e_index(v("s"), v("byte"))])),
                    s_if(
                        bin(v("b"), BinOp::Lt, int(0x80)),
                        step(1),
                        vec![
                            (bin(v("b"), BinOp::GtEq, int(0xF0)), step(4)),
                            (bin(v("b"), BinOp::GtEq, int(0xE0)), step(3)),
                            (bin(v("b"), BinOp::GtEq, int(0xC0)), step(2)),
                        ],
                        Some(step(1)),
                    ),
                    s_assign("chars", bin(v("chars"), BinOp::Sub, int(1))),
                ],
            ),
            if_then(bin(v("byte"), BinOp::Gt, v("len")), vec![s_assign("byte", v("len"))]),
            s_return(v("byte")),
        ])
        .build()
}

/// `__elephc_mb_repeat(string $pad, int $padChars, int $count): string` — `$count` characters of
/// `$pad` repeated.
fn repeat() -> Stmt {
    function(REPEAT)
        .param("pad", TypeExpr::Str)
        .param("padChars", TypeExpr::Int)
        .param("count", TypeExpr::Int)
        .returns(TypeExpr::Str)
        .body(vec![
            if_then(bin(v("count"), BinOp::LtEq, int(0)), vec![s_return(e_str(""))]),
            s_return(e_concat(
                call(
                    "str_repeat",
                    vec![v("pad"), call("intdiv", vec![v("count"), v("padChars")])],
                ),
                call(
                    "mb_substr",
                    vec![v("pad"), int(0), bin(v("count"), BinOp::Mod, v("padChars"))],
                ),
            )),
        ])
        .build()
}

/// `mb_substr(string $string, int $start, ?int $length = null, ?string $encoding = null): string`
fn mb_substr() -> Stmt {
    function("mb_substr")
        .param("string", TypeExpr::Str)
        .param("start", TypeExpr::Int)
        .param_default("length", t_nullable(TypeExpr::Int), e_null())
        .param_default("encoding", nullable_string(), e_null())
        .returns(TypeExpr::Str)
        .body(vec![
            s_assign("n", call("mb_strlen", vec![v("string")])),
            if_then(
                bin(v("start"), BinOp::Lt, int(0)),
                vec![
                    s_assign("start", bin(v("n"), BinOp::Add, v("start"))),
                    if_then(bin(v("start"), BinOp::Lt, int(0)), vec![s_assign("start", int(0))]),
                ],
            ),
            if_then(bin(v("start"), BinOp::Gt, v("n")), vec![s_return(e_str(""))]),
            s_if(
                bin(v("length"), BinOp::StrictEq, e_null()),
                vec![s_assign("end", v("n"))],
                vec![(
                    bin(v("length"), BinOp::Lt, int(0)),
                    vec![s_assign("end", bin(v("n"), BinOp::Add, v("length")))],
                )],
                Some(vec![
                    s_assign("end", bin(v("start"), BinOp::Add, v("length"))),
                    if_then(bin(v("end"), BinOp::Gt, v("n")), vec![s_assign("end", v("n"))]),
                ]),
            ),
            if_then(bin(v("end"), BinOp::LtEq, v("start")), vec![s_return(e_str(""))]),
            s_assign("from", call(ADVANCE, vec![v("string"), int(0), v("start")])),
            s_assign(
                "to",
                call(
                    ADVANCE,
                    vec![v("string"), v("from"), bin(v("end"), BinOp::Sub, v("start"))],
                ),
            ),
            s_return(call(
                "substr",
                vec![v("string"), v("from"), bin(v("to"), BinOp::Sub, v("from"))],
            )),
        ])
        .build()
}

/// `mb_strpos` / `mb_stripos` / `mb_strrpos` / `mb_strripos`: the byte search over the character
/// offset, its result converted back to a character position.
fn mb_strpos(name: &str, byte_search: &str, fold_case: bool) -> Stmt {
    let mut body = Vec::new();
    if fold_case {
        body.push(s_assign("haystack", call("mb_strtolower", vec![v("haystack")])));
        body.push(s_assign("needle", call("mb_strtolower", vec![v("needle")])));
    }
    let reverse = byte_search == "strrpos";
    body.push(s_assign("n", call("mb_strlen", vec![v("haystack")])));
    body.push(s_assign(
        "at",
        e_ternary(
            bin(v("offset"), BinOp::Lt, int(0)),
            bin(v("n"), BinOp::Add, v("offset")),
            v("offset"),
        ),
    ));
    body.push(if_then(
        bin(
            bin(v("at"), BinOp::Lt, int(0)),
            BinOp::Or,
            bin(v("at"), BinOp::Gt, v("n")),
        ),
        vec![value_error(&format!(
            "{name}(): Argument #3 ($offset) must be contained in argument #1 ($haystack)"
        ))],
    ));
    body.push(s_assign("byte", call(ADVANCE, vec![v("haystack"), int(0), v("at")])));
    if reverse {
        // A negative offset bounds where a match may start, counted from the end: the same
        // position as a negative byte offset to `strrpos()`.
        body.push(if_then(
            bin(v("offset"), BinOp::Lt, int(0)),
            vec![s_assign(
                "byte",
                bin(v("byte"), BinOp::Sub, call("strlen", vec![v("haystack")])),
            )],
        ));
    }
    body.push(s_assign(
        "pos",
        call(byte_search, vec![v("haystack"), v("needle"), v("byte")]),
    ));
    body.push(if_then(
        bin(v("pos"), BinOp::StrictEq, e_bool(false)),
        vec![s_return(e_bool(false))],
    ));
    body.push(s_return(call(
        "mb_strlen",
        vec![call("substr", vec![v("haystack"), int(0), v("pos")])],
    )));
    function(name)
        .param("haystack", TypeExpr::Str)
        .param("needle", TypeExpr::Str)
        .param_default("offset", TypeExpr::Int, int(0))
        .param_default("encoding", nullable_string(), e_null())
        .returns(t_mixed())
        .body(body)
        .build()
}

/// `mb_str_split(string $string, int $length = 1, ?string $encoding = null): array`
fn mb_str_split() -> Stmt {
    function("mb_str_split")
        .param("string", TypeExpr::Str)
        .param_default("length", TypeExpr::Int, int(1))
        .param_default("encoding", nullable_string(), e_null())
        .returns(t_array())
        .body(vec![
            if_then(
                bin(v("length"), BinOp::Lt, int(1)),
                vec![value_error(
                    "mb_str_split(): Argument #2 ($length) must be greater than 0",
                )],
            ),
            s_assign("parts", crate::synthetic_class::e_array(Vec::new())),
            s_assign("len", call("strlen", vec![v("string")])),
            s_assign("byte", int(0)),
            s_while(
                bin(v("byte"), BinOp::Lt, v("len")),
                vec![
                    s_assign("next", call(ADVANCE, vec![v("string"), v("byte"), v("length")])),
                    s_array_push(
                        "parts",
                        call(
                            "substr",
                            vec![v("string"), v("byte"), bin(v("next"), BinOp::Sub, v("byte"))],
                        ),
                    ),
                    s_assign("byte", v("next")),
                ],
            ),
            s_return(v("parts")),
        ])
        .build()
}

/// `mb_substr_count(string $haystack, string $needle, ?string $encoding = null): int` — counting
/// non-overlapping byte matches is counting character matches for valid UTF-8.
fn mb_substr_count() -> Stmt {
    function("mb_substr_count")
        .param("haystack", TypeExpr::Str)
        .param("needle", TypeExpr::Str)
        .param_default("encoding", nullable_string(), e_null())
        .returns(TypeExpr::Int)
        .body(vec![
            if_then(
                bin(v("needle"), BinOp::StrictEq, e_str("")),
                vec![value_error("mb_substr_count(): Argument #2 ($needle) must not be empty")],
            ),
            s_return(call("substr_count", vec![v("haystack"), v("needle")])),
        ])
        .build()
}

/// `mb_strstr` / `mb_stristr` / `mb_strrchr`: the part of the haystack from (or before) the
/// match; byte offsets are character boundaries for valid UTF-8.
fn mb_strstr(name: &str, byte_search: &str, fold_case: bool) -> Stmt {
    let search_in = if fold_case {
        call("mb_strtolower", vec![v("haystack")])
    } else {
        v("haystack")
    };
    let search_for = if fold_case {
        call("mb_strtolower", vec![v("needle")])
    } else {
        v("needle")
    };
    function(name)
        .param("haystack", TypeExpr::Str)
        .param("needle", TypeExpr::Str)
        .param_default("before_needle", TypeExpr::Bool, e_bool(false))
        .param_default("encoding", nullable_string(), e_null())
        .returns(t_mixed())
        .body(vec![
            s_assign("pos", call(byte_search, vec![search_in, search_for])),
            if_then(
                bin(v("pos"), BinOp::StrictEq, e_bool(false)),
                vec![s_return(e_bool(false))],
            ),
            s_if(
                v("before_needle"),
                vec![s_return(call("substr", vec![v("haystack"), int(0), v("pos")]))],
                Vec::new(),
                None,
            ),
            s_return(call("substr", vec![v("haystack"), v("pos")])),
        ])
        .build()
}

/// `mb_internal_encoding(?string $encoding = null): string|bool` — UTF-8 is the only encoding.
fn mb_internal_encoding() -> Stmt {
    function("mb_internal_encoding")
        .param_default("encoding", nullable_string(), e_null())
        .returns(t_mixed())
        .body(vec![
            if_then(
                bin(v("encoding"), BinOp::StrictEq, e_null()),
                vec![s_return(e_str("UTF-8"))],
            ),
            s_return(bin(
                call("strtoupper", vec![call("str_replace", vec![e_str("-"), e_str(""), v("encoding")])]),
                BinOp::StrictEq,
                e_str("UTF8"),
            )),
        ])
        .build()
}

/// `mb_check_encoding(mixed $value = null, ?string $encoding = null): bool` — whether a string is
/// well-formed UTF-8 (no overlong forms, surrogates or code points past U+10FFFF).
fn mb_check_encoding() -> Stmt {
    let byte_at = |offset: Expr| call("ord", vec![e_index(v("value"), offset)]);
    function("mb_check_encoding")
        .param_default("value", t_mixed(), e_null())
        .param_default("encoding", nullable_string(), e_null())
        .returns(TypeExpr::Bool)
        .body(vec![
            if_then(
                e_not(call("is_string", vec![v("value")])),
                vec![s_return(e_bool(true))],
            ),
            s_assign("len", call("strlen", vec![v("value")])),
            s_assign("i", int(0)),
            s_while(
                bin(v("i"), BinOp::Lt, v("len")),
                vec![
                    s_assign("b", byte_at(v("i"))),
                    s_if(
                        bin(v("b"), BinOp::Lt, int(0x80)),
                        vec![s_assign("need", int(0))],
                        vec![
                            (
                                bin(
                                    bin(v("b"), BinOp::GtEq, int(0xC2)),
                                    BinOp::And,
                                    bin(v("b"), BinOp::LtEq, int(0xDF)),
                                ),
                                vec![s_assign("need", int(1))],
                            ),
                            (
                                bin(
                                    bin(v("b"), BinOp::GtEq, int(0xE0)),
                                    BinOp::And,
                                    bin(v("b"), BinOp::LtEq, int(0xEF)),
                                ),
                                vec![s_assign("need", int(2))],
                            ),
                            (
                                bin(
                                    bin(v("b"), BinOp::GtEq, int(0xF0)),
                                    BinOp::And,
                                    bin(v("b"), BinOp::LtEq, int(0xF4)),
                                ),
                                vec![s_assign("need", int(3))],
                            ),
                        ],
                        Some(vec![s_return(e_bool(false))]),
                    ),
                    if_then(
                        bin(bin(v("i"), BinOp::Add, v("need")), BinOp::GtEq, v("len")),
                        vec![if_then(
                            bin(v("need"), BinOp::Gt, int(0)),
                            vec![s_return(e_bool(false))],
                        )],
                    ),
                    s_assign("k", int(1)),
                    s_while(
                        bin(v("k"), BinOp::LtEq, v("need")),
                        vec![
                            s_assign("c", byte_at(bin(v("i"), BinOp::Add, v("k")))),
                            if_then(
                                bin(
                                    bin(v("c"), BinOp::Lt, int(0x80)),
                                    BinOp::Or,
                                    bin(v("c"), BinOp::Gt, int(0xBF)),
                                ),
                                vec![s_return(e_bool(false))],
                            ),
                            s_assign("k", bin(v("k"), BinOp::Add, int(1))),
                        ],
                    ),
                    if_then(
                        bin(v("need"), BinOp::Gt, int(1)),
                        vec![
                            s_assign("second", byte_at(bin(v("i"), BinOp::Add, int(1)))),
                            if_then(
                                bin(
                                    bin(
                                        bin(
                                            bin(v("b"), BinOp::StrictEq, int(0xE0)),
                                            BinOp::And,
                                            bin(v("second"), BinOp::Lt, int(0xA0)),
                                        ),
                                        BinOp::Or,
                                        bin(
                                            bin(v("b"), BinOp::StrictEq, int(0xED)),
                                            BinOp::And,
                                            bin(v("second"), BinOp::Gt, int(0x9F)),
                                        ),
                                    ),
                                    BinOp::Or,
                                    bin(
                                        bin(
                                            bin(v("b"), BinOp::StrictEq, int(0xF0)),
                                            BinOp::And,
                                            bin(v("second"), BinOp::Lt, int(0x90)),
                                        ),
                                        BinOp::Or,
                                        bin(
                                            bin(v("b"), BinOp::StrictEq, int(0xF4)),
                                            BinOp::And,
                                            bin(v("second"), BinOp::Gt, int(0x8F)),
                                        ),
                                    ),
                                ),
                                vec![s_return(e_bool(false))],
                            ),
                        ],
                    ),
                    s_assign(
                        "i",
                        bin(bin(v("i"), BinOp::Add, v("need")), BinOp::Add, int(1)),
                    ),
                ],
            ),
            s_return(e_bool(true)),
        ])
        .build()
}

/// `mb_str_pad(string $string, int $length, string $pad_string = " ", int $pad_type =
/// STR_PAD_RIGHT, ?string $encoding = null): string` — padding counted in characters.
fn mb_str_pad() -> Stmt {
    function("mb_str_pad")
        .param("string", TypeExpr::Str)
        .param("length", TypeExpr::Int)
        .param_default("pad_string", TypeExpr::Str, e_str(" "))
        .param_default("pad_type", TypeExpr::Int, int(1))
        .param_default("encoding", nullable_string(), e_null())
        .returns(TypeExpr::Str)
        .body(vec![
            if_then(
                bin(v("pad_string"), BinOp::StrictEq, e_str("")),
                vec![value_error(
                    "mb_str_pad(): Argument #3 ($pad_string) must be a non-empty string",
                )],
            ),
            if_then(
                bin(
                    bin(v("pad_type"), BinOp::Lt, int(0)),
                    BinOp::Or,
                    bin(v("pad_type"), BinOp::Gt, int(2)),
                ),
                vec![value_error(
                    "mb_str_pad(): Argument #4 ($pad_type) must be STR_PAD_LEFT, STR_PAD_RIGHT, or STR_PAD_BOTH",
                )],
            ),
            s_assign(
                "total",
                bin(v("length"), BinOp::Sub, call("mb_strlen", vec![v("string")])),
            ),
            if_then(bin(v("total"), BinOp::LtEq, int(0)), vec![s_return(v("string"))]),
            s_assign("padChars", call("mb_strlen", vec![v("pad_string")])),
            s_if(
                bin(v("pad_type"), BinOp::StrictEq, int(1)),
                vec![s_assign("left", int(0))],
                vec![(
                    bin(v("pad_type"), BinOp::StrictEq, int(0)),
                    vec![s_assign("left", v("total"))],
                )],
                Some(vec![s_assign("left", call("intdiv", vec![v("total"), int(2)]))]),
            ),
            s_return(e_concat(
                e_concat(
                    call(REPEAT, vec![v("pad_string"), v("padChars"), v("left")]),
                    v("string"),
                ),
                call(
                    REPEAT,
                    vec![
                        v("pad_string"),
                        v("padChars"),
                        bin(v("total"), BinOp::Sub, v("left")),
                    ],
                ),
            )),
        ])
        .build()
}

/// `mb_ucfirst` / `mb_lcfirst`: the first character re-cased, the rest unchanged.
fn mb_case_first(name: &str, recase: &str) -> Stmt {
    function(name)
        .param("string", TypeExpr::Str)
        .param_default("encoding", nullable_string(), e_null())
        .returns(TypeExpr::Str)
        .body(vec![s_return(e_concat(
            call(recase, vec![call("mb_substr", vec![v("string"), int(0), int(1)])]),
            call("mb_substr", vec![v("string"), int(1)]),
        ))])
        .build()
}
