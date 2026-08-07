//! Purpose:
//! DateInterval ISO-8601 constructor and component-property metadata.
//!
//! Called from:
//! - DateInterval declaration injection.
//!
//! Key details:
//! - Direct intervals keep `days` as false until a DateTime diff supplies it.

use super::*;

/// `DateInterval::__construct(string $duration)` — parses an ISO 8601 duration into components.
///
/// Scans `P[nY][nM][nW][nD][T[nH][nM][nS]]`, accumulating each number and assigning it to the
/// matching component on the unit letter; `M` before `T` is months, after `T` is minutes; `W`
/// contributes 7 days each. The leading `P` is required (a missing/lowercase `P` throws); the
/// `T` time separator is consumed as a no-op and unknown letters throw.
pub(super) fn date_interval_constructor() -> ClassMethod {
    let var = |n: &str| Expr::new(ExprKind::Variable(n.to_string()), dummy());
    let int = |n: i64| Expr::new(ExprKind::IntLiteral(n), dummy());
    let strlit = |s: &str| Expr::new(ExprKind::StringLiteral(s.to_string()), dummy());
    let binop = |l: Expr, op: BinOp, r: Expr| {
        Expr::new(ExprKind::BinaryOp { left: Box::new(l), op, right: Box::new(r) }, dummy())
    };
    let call = |name: &str, args: Vec<Expr>| {
        Expr::new(ExprKind::FunctionCall { name: Name::unqualified(name), args }, dummy())
    };
    // $p = $p + 1;
    let p_inc = || Stmt::assign("p", binop(var("p"), BinOp::Add, int(1)));
    // $num = 0;
    let reset_num = || Stmt::assign("num", int(0));
    // $c === "<letter>"
    let is_c = |ch: &str| binop(var("c"), BinOp::StrictEq, strlit(ch));

    // if ($o >= 48 && $o <= 57) { $num = $num * 10 + ($o - 48); $p = $p + 1; continue; }
    let digit_if = Stmt::new(
        StmtKind::If {
            condition: binop(
                binop(var("o"), BinOp::GtEq, int(48)),
                BinOp::And,
                binop(var("o"), BinOp::LtEq, int(57)),
            ),
            then_body: vec![
                Stmt::assign(
                    "num",
                    binop(
                        binop(var("num"), BinOp::Mul, int(10)),
                        BinOp::Add,
                        binop(var("o"), BinOp::Sub, int(48)),
                    ),
                ),
                p_inc(),
                Stmt::new(StmtKind::Continue(1), dummy()),
            ],
            elseif_clauses: Vec::new(),
            else_body: None,
        },
        dummy(),
    );


    let inc_units = || Stmt::assign("units", binop(var("units"), BinOp::Add, int(1)));
    let throw_malformed_interval = || {
        Stmt::new(
            StmtKind::Throw(Expr::new(
                ExprKind::NewObject {
                    class_name: Name::unqualified("DateMalformedIntervalStringException"),
                    args: vec![strlit("Unknown or bad format")],
                },
                dummy(),
            )),
            dummy(),
        )
    };

    // M dispatch: minutes after T, months before; counts as a recognized unit.
    let m_branch = vec![
        Stmt::new(
            StmtKind::If {
                condition: binop(var("inTime"), BinOp::StrictEq, int(1)),
                then_body: vec![assign_this_property("i", var("num")), inc_units()],
                elseif_clauses: Vec::new(),
                else_body: Some(vec![assign_this_property("m", var("num")), inc_units()]),
            },
            dummy(),
        ),
        reset_num(),
    ];

    // if ($c === "T") {...} elseif ... unit letters ... elseif "P" (leading, no-op) else throw
    let unit_if = Stmt::new(
        StmtKind::If {
            condition: is_c("T"),
            then_body: vec![Stmt::assign("inTime", int(1))],
            elseif_clauses: vec![
                (is_c("Y"), vec![assign_this_property("y", var("num")), inc_units(), reset_num()]),
                (
                    is_c("W"),
                    vec![
                        assign_this_property(
                            "d",
                            binop(this_property("d"), BinOp::Add, binop(var("num"), BinOp::Mul, int(7))),
                        ),
                        inc_units(),
                        reset_num(),
                    ],
                ),
                (
                    is_c("D"),
                    vec![
                        assign_this_property("d", binop(this_property("d"), BinOp::Add, var("num"))),
                        inc_units(),
                        reset_num(),
                    ],
                ),
                (is_c("H"), vec![assign_this_property("h", var("num")), inc_units(), reset_num()]),
                (is_c("S"), vec![assign_this_property("s", var("num")), inc_units(), reset_num()]),
                (is_c("M"), m_branch),
                (is_c("P"), vec![]),
            ],
            else_body: Some(vec![throw_malformed_interval()]),
        },
        dummy(),
    );

    let while_body = vec![
        Stmt::assign(
            "c",
            Expr::new(
                ExprKind::ArrayAccess { array: Box::new(var("duration")), index: Box::new(var("p")) },
                dummy(),
            ),
        ),
        Stmt::assign("o", call("ord", vec![var("c")])),
        digit_if,
        unit_if,
        p_inc(),
    ];

    let body = vec![
        Stmt::assign("len", call("strlen", vec![var("duration")])),
        // PHP requires the duration to start with a literal `P`; anything else
        // (e.g. "1Y", "p1y", "") is a DateMalformedIntervalStringException.
        Stmt::new(
            StmtKind::If {
                condition: binop(
                    call("substr", vec![var("duration"), int(0), int(1)]),
                    BinOp::StrictNotEq,
                    strlit("P"),
                ),
                then_body: vec![throw_malformed_interval()],
                elseif_clauses: Vec::new(),
                else_body: None,
            },
            dummy(),
        ),
        Stmt::assign("num", int(0)),
        Stmt::assign("inTime", int(0)),
        Stmt::assign("units", int(0)),
        Stmt::assign("p", int(0)),
        Stmt::new(
            StmtKind::While { condition: binop(var("p"), BinOp::Lt, var("len")), body: while_body },
            dummy(),
        ),
        Stmt::new(
            StmtKind::If {
                condition: binop(var("units"), BinOp::StrictEq, int(0)),
                then_body: vec![throw_malformed_interval()],
                elseif_clauses: Vec::new(),
                else_body: None,
            },
            dummy(),
        ),
    ];

    method(
        "__construct",
        vec![("duration".to_string(), Some(TypeExpr::Str), None, false)],
        None,
        body,
    )
}

/// Builds a `DateInterval` component property. The numeric components
/// (`y`/`m`/`d`/`h`/`i`/`s`/`invert`) are `int` defaulting to `0`. `days` is special: PHP exposes it
/// as `int|false`, holding an absolute whole-day count only for intervals produced by
/// `DateTime::diff()` and the boolean `false` for intervals constructed directly (which
/// `format("%a")` renders as `(unknown)`). The boxed `false` default relies on the EIR object_new
/// scalar-into-Mixed default support.
pub(super) fn interval_property(name: &str) -> ClassProperty {
    if name == "days" {
        return property(
            "days",
            TypeExpr::Union(vec![TypeExpr::Int, TypeExpr::Bool]),
            Expr::new(ExprKind::BoolLiteral(false), dummy()),
        );
    }
    property(name, TypeExpr::Int, Expr::new(ExprKind::IntLiteral(0), dummy()))
}
