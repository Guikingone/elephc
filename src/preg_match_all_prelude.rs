//! Purpose:
//! Builds `preg_match_all()`'s `$flags` / `$offset` forms as a prelude declaration, on top of the
//! native `preg_match()` and the native flag-less `preg_match_all()`.
//!
//! Called from:
//! - `crate::backend_gap_prelude::inject_if_used`, which prepends the declaration when the
//!   program references `preg_match_all`.
//!
//! Key details:
//! - The declarations are BUILT, never parsed: elephc does not parse PHP source outside tests.
//! - The native `__rt_preg_match_all_capture` builds PATTERN order and ignores `$flags`, so
//!   `PREG_OFFSET_CAPTURE` came back as plain strings and indexing a match gave single characters.
//!   MEASURED against php 8.5.10 on `preg_match_all('#<(…)>#ix', 'a <info>b</info> c', $m,
//!   PREG_OFFSET_CAPTURE)`: php `$m[0][0] === ['<info>', 2]`, elephc `$m[0][0] === '<info>'` and
//!   `$m[0][0][1] === 'i'`. Symfony's console `OutputFormatter::formatAndWrap()` walks its style
//!   tags exactly that way, so every command printed its tags literally and repeated each line.
//! - `preg_match()` already honours `PREG_OFFSET_CAPTURE` and a start offset, so the helper finds
//!   the matches one at a time with it and arranges them itself. The native flag-less call is
//!   still made once: it is the only thing that knows the pattern's FULL group list, including
//!   groups that never participated (a single `preg_match()` omits trailing unmatched ones) and
//!   the empty columns PHP returns when nothing matches at all.
//! - After an EMPTY match the search resumes one byte later. PHP first retries at the same
//!   position with `PCRE2_NOTEMPTY_ATSTART`, which this cannot express; the two differ only for a
//!   pattern that can match both empty and non-empty at the same position.

use crate::parser::ast::{BinOp, Program, Stmt, TypeExpr};
use crate::synthetic_class::{
    e_array, e_binop, e_bool, e_call, e_index, e_int, e_null, e_str, e_ternary, e_var, function,
    e_not as e_unary_not, internal_declarations, s_array_assign, s_array_push, s_assign, s_break, s_foreach, s_if,
    s_return, s_while,
};
/// Reserved helper for `preg_match_all()` calls that pass `$flags` (and possibly `$offset`).
pub(crate) const PREG_MATCH_ALL_FLAGS_NAME: &str = "__elephc_preg_match_all_flags";

const PREG_SET_ORDER: i64 = 2;
const PREG_OFFSET_CAPTURE: i64 = 256;
const PREG_UNMATCHED_AS_NULL: i64 = 512;

/// Returns the `preg_match_all()` flags declaration for injection.
pub(crate) fn declarations() -> Program {
    internal_declarations(|| vec![preg_match_all_flags()])
}

fn has_flag(flag: i64) -> crate::parser::ast::Expr {
    e_binop(
        e_binop(e_var("flags"), BinOp::BitAnd, e_int(flag)),
        BinOp::StrictNotEq,
        e_int(0),
    )
}

/// Builds `__elephc_preg_match_all_flags($pattern, $subject, &$matches, $flags, $offset)`.
fn preg_match_all_flags() -> Stmt {
    function(PREG_MATCH_ALL_FLAGS_NAME)
        .param("pattern", TypeExpr::Str)
        .param("subject", TypeExpr::Str)
        .param_by_ref("matches", None)
        .param("flags", TypeExpr::Int)
        .param("offset", TypeExpr::Int)
        .body(vec![
            s_assign(
                "count",
                e_call(
                    "preg_match_all",
                    vec![e_var("pattern"), e_var("subject"), e_var("plain")],
                ),
            ),
            s_if(
                e_binop(e_var("count"), BinOp::StrictEq, e_bool(false)),
                vec![s_assign("matches", e_array(Vec::new())), s_return(e_bool(false))],
                Vec::new(),
                None,
            ),
            s_assign("keys", e_call("array_keys", vec![e_var("plain")])),
            s_assign("withOffset", has_flag(PREG_OFFSET_CAPTURE)),
            s_assign("asNull", has_flag(PREG_UNMATCHED_AS_NULL)),
            // SET order reports a row the way `preg_match()` does: trailing groups that did not
            // participate are absent, unless PREG_UNMATCHED_AS_NULL asks for them as nulls.
            // PATTERN order needs every column, so it fills them in.
            s_assign(
                "trimTrailing",
                e_binop(
                    has_flag(PREG_SET_ORDER),
                    BinOp::And,
                    e_unary_not(e_var("asNull")),
                ),
            ),
            s_assign("sets", e_array(Vec::new())),
            s_assign("len", e_call("strlen", vec![e_var("subject")])),
            // A negative offset counts from the end of the subject, as in `preg_match()`.
            s_assign(
                "pos",
                e_ternary(
                    e_binop(e_var("offset"), BinOp::Lt, e_int(0)),
                    e_call(
                        "max",
                        vec![
                            e_int(0),
                            e_binop(e_var("len"), BinOp::Add, e_var("offset")),
                        ],
                    ),
                    e_var("offset"),
                ),
            ),
            s_while(
                e_binop(e_var("pos"), BinOp::LtEq, e_var("len")),
                vec![
                    s_assign("m", e_array(Vec::new())),
                    s_if(
                        e_binop(
                            e_call(
                                "preg_match",
                                vec![
                                    e_var("pattern"),
                                    e_var("subject"),
                                    e_var("m"),
                                    e_int(PREG_OFFSET_CAPTURE),
                                    e_var("pos"),
                                ],
                            ),
                            BinOp::StrictNotEq,
                            e_int(1),
                        ),
                        vec![s_break(1)],
                        Vec::new(),
                        None,
                    ),
                    s_assign("set", e_array(Vec::new())),
                    s_foreach(e_var("keys"), None, "k", group_into_set()),
                    s_array_push("sets", e_var("set")),
                    s_assign("start", e_index(e_index(e_var("m"), e_int(0)), e_int(1))),
                    s_assign(
                        "mlen",
                        e_call(
                            "strlen",
                            vec![e_index(e_index(e_var("m"), e_int(0)), e_int(0))],
                        ),
                    ),
                    s_if(
                        e_binop(e_var("mlen"), BinOp::StrictEq, e_int(0)),
                        vec![s_assign("pos", e_binop(e_var("start"), BinOp::Add, e_int(1)))],
                        Vec::new(),
                        Some(vec![s_assign(
                            "pos",
                            e_binop(e_var("start"), BinOp::Add, e_var("mlen")),
                        )]),
                    ),
                ],
            ),
            s_if(
                has_flag(PREG_SET_ORDER),
                vec![s_assign("matches", e_var("sets"))],
                Vec::new(),
                Some(pattern_order()),
            ),
            s_return(e_call("count", vec![e_var("sets")])),
        ])
        .build()
}

/// `$set[$k] = <this group's pair, or its text>` for one group key of one match.
fn group_into_set() -> Vec<Stmt> {
    vec![
        s_if(
            e_binop(
                e_var("trimTrailing"),
                BinOp::And,
                e_unary_not(e_call("array_key_exists", vec![e_var("k"), e_var("m")])),
            ),
            vec![crate::synthetic_class::s_continue(1)],
            Vec::new(),
            None,
        ),
        s_if(
            e_binop(
                e_call("array_key_exists", vec![e_var("k"), e_var("m")]),
                BinOp::And,
                e_binop(
                    e_index(e_index(e_var("m"), e_var("k")), e_int(1)),
                    BinOp::StrictNotEq,
                    e_int(-1),
                ),
            ),
            vec![s_assign("pair", e_index(e_var("m"), e_var("k")))],
            Vec::new(),
            Some(vec![s_assign(
                "pair",
                e_array(vec![
                    e_ternary(e_var("asNull"), e_null(), e_str("")),
                    e_int(-1),
                ]),
            )]),
        ),
        s_if(
            e_var("withOffset"),
            vec![s_array_assign("set", e_var("k"), e_var("pair"))],
            Vec::new(),
            Some(vec![s_array_assign(
                "set",
                e_var("k"),
                e_index(e_var("pair"), e_int(0)),
            )]),
        ),
    ]
}

/// PATTERN order: one column per group, each holding that group across every match.
fn pattern_order() -> Vec<Stmt> {
    vec![
        s_assign("out", e_array(Vec::new())),
        s_foreach(
            e_var("keys"),
            None,
            "k",
            vec![
                s_assign("col", e_array(Vec::new())),
                s_foreach(
                    e_var("sets"),
                    None,
                    "s",
                    vec![s_array_push("col", e_index(e_var("s"), e_var("k")))],
                ),
                s_array_assign("out", e_var("k"), e_var("col")),
            ],
        ),
        s_assign("matches", e_var("out")),
    ]
}
