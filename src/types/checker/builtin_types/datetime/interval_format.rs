//! Purpose:
//! Synthetic AST implementation of `DateInterval::format`.
//!
//! Called from:
//! - DateInterval declaration injection.
//!
//! Key details:
//! - PHP padding, sign, total-days, literal, and unknown-specifier behavior is retained.

use super::*;

/// `DateInterval::format(string $format): string` — render the interval using PHP's `%` specifiers.
///
/// Scans `$format`; `%` introduces a specifier and every other character is copied literally.
/// Supports `%y/%Y %m/%M %d/%D %h/%H %i/%I %s/%S` (lowercase = no padding, uppercase = at least two
/// digits, zero-padded), `%a` (total days, or `(unknown)` for intervals not produced by `diff()`),
/// `%R` (`-`/`+`), `%r` (`-`/empty), and `%%`. An unrecognized specifier is copied verbatim.
pub(super) fn date_interval_format() -> ClassMethod {
    let var = |n: &str| Expr::new(ExprKind::Variable(n.to_string()), dummy());
    let int = |n: i64| Expr::new(ExprKind::IntLiteral(n), dummy());
    let strlit = |s: &str| Expr::new(ExprKind::StringLiteral(s.to_string()), dummy());
    let binop = |l: Expr, op: BinOp, r: Expr| {
        Expr::new(ExprKind::BinaryOp { left: Box::new(l), op, right: Box::new(r) }, dummy())
    };
    // $r = $r . <e>;
    let cat = |e: Expr| Stmt::assign("r", binop(var("r"), BinOp::Concat, e));
    // $p = $p + 1;
    let p_inc = || Stmt::assign("p", binop(var("p"), BinOp::Add, int(1)));
    // $spec === "<ch>"
    let spec_is = |ch: &str| binop(var("spec"), BinOp::StrictEq, strlit(ch));
    // append $this-><prop> with no padding.
    let nopad = |prop: &str| vec![cat(this_property(prop))];
    // append $this-><prop> zero-padded to at least two digits.
    let padded = |prop: &str| {
        vec![
            Stmt::new(
                StmtKind::If {
                    condition: binop(this_property(prop), BinOp::Lt, int(10)),
                    then_body: vec![cat(strlit("0"))],
                    elseif_clauses: Vec::new(),
                    else_body: None,
                },
                dummy(),
            ),
            cat(this_property(prop)),
        ]
    };
    // $format[$p]
    let fmt_at = |idx: Expr| {
        Expr::new(
            ExprKind::ArrayAccess { array: Box::new(var("format")), index: Box::new(idx) },
            dummy(),
        )
    };
    // intval($this->f * 1000000) — whole microseconds from the fractional-second float.
    let micros = || {
        Expr::new(
            ExprKind::FunctionCall {
                name: Name::unqualified("intval"),
                args: vec![binop(this_property("f"), BinOp::Mul, int(1_000_000))],
            },
            dummy(),
        )
    };

    // The %-specifier dispatch executed once $spec has been read.
    let dispatch = Stmt::new(
        StmtKind::If {
            condition: spec_is("%"),
            then_body: vec![cat(strlit("%"))],
            elseif_clauses: vec![
                (spec_is("y"), nopad("y")),
                (spec_is("Y"), padded("y")),
                (spec_is("m"), nopad("m")),
                (spec_is("M"), padded("m")),
                (spec_is("d"), nopad("d")),
                (spec_is("D"), padded("d")),
                (spec_is("h"), nopad("h")),
                (spec_is("H"), padded("h")),
                (spec_is("i"), nopad("i")),
                (spec_is("I"), padded("i")),
                (spec_is("s"), nopad("s")),
                (spec_is("S"), padded("s")),
                // %f: whole microseconds from $this->f, no padding.
                (spec_is("f"), vec![Stmt::assign("us", micros()), cat(var("us"))]),
                // %F: whole microseconds zero-padded to six digits.
                (
                    spec_is("F"),
                    {
                        let mut stmts = vec![Stmt::assign("us", micros())];
                        // One leading zero per power of ten the value falls short of 6 digits.
                        for threshold in [100_000, 10_000, 1_000, 100, 10] {
                            stmts.push(Stmt::new(
                                StmtKind::If {
                                    condition: binop(var("us"), BinOp::Lt, int(threshold)),
                                    then_body: vec![cat(strlit("0"))],
                                    elseif_clauses: Vec::new(),
                                    else_body: None,
                                },
                                dummy(),
                            ));
                        }
                        stmts.push(cat(var("us")));
                        stmts
                    },
                ),
                // %a: total days, or "(unknown)" when `days === false` (interval not from diff()).
                (
                    spec_is("a"),
                    vec![Stmt::new(
                        StmtKind::If {
                            condition: binop(
                                this_property("days"),
                                BinOp::StrictEq,
                                Expr::new(ExprKind::BoolLiteral(false), dummy()),
                            ),
                            then_body: vec![cat(strlit("(unknown)"))],
                            elseif_clauses: Vec::new(),
                            else_body: Some(vec![cat(this_property("days"))]),
                        },
                        dummy(),
                    )],
                ),
                // %R: "-" when inverted, otherwise "+".
                (
                    spec_is("R"),
                    vec![Stmt::new(
                        StmtKind::If {
                            condition: binop(this_property("invert"), BinOp::StrictEq, int(1)),
                            then_body: vec![cat(strlit("-"))],
                            elseif_clauses: Vec::new(),
                            else_body: Some(vec![cat(strlit("+"))]),
                        },
                        dummy(),
                    )],
                ),
                // %r: "-" when inverted, otherwise nothing.
                (
                    spec_is("r"),
                    vec![Stmt::new(
                        StmtKind::If {
                            condition: binop(this_property("invert"), BinOp::StrictEq, int(1)),
                            then_body: vec![cat(strlit("-"))],
                            elseif_clauses: Vec::new(),
                            else_body: None,
                        },
                        dummy(),
                    )],
                ),
            ],
            // Unknown specifier: copy the "%" and the following character verbatim.
            else_body: Some(vec![cat(strlit("%")), cat(var("spec"))]),
        },
        dummy(),
    );

    let while_body = vec![
        Stmt::assign("c", fmt_at(var("p"))),
        Stmt::new(
            StmtKind::If {
                condition: binop(var("c"), BinOp::StrictEq, strlit("%")),
                then_body: vec![
                    p_inc(),
                    Stmt::new(
                        StmtKind::If {
                            condition: binop(var("p"), BinOp::Lt, var("len")),
                            then_body: vec![Stmt::assign("spec", fmt_at(var("p"))), dispatch, p_inc()],
                            elseif_clauses: Vec::new(),
                            else_body: None,
                        },
                        dummy(),
                    ),
                ],
                elseif_clauses: Vec::new(),
                else_body: Some(vec![cat(var("c")), p_inc()]),
            },
            dummy(),
        ),
    ];

    method(
        "format",
        vec![("format".to_string(), Some(TypeExpr::Str), None, false)],
        Some(TypeExpr::Str),
        vec![
            Stmt::assign("len", Expr::new(
                ExprKind::FunctionCall { name: Name::unqualified("strlen"), args: vec![var("format")] },
                dummy(),
            )),
            Stmt::assign("p", int(0)),
            Stmt::assign("r", strlit("")),
            Stmt::new(
                StmtKind::While {
                    condition: binop(var("p"), BinOp::Lt, var("len")),
                    body: while_body,
                },
                dummy(),
            ),
            return_expr(var("r")),
        ],
    )
}
