//! Purpose:
//! Synthetic AST implementation of `DateTimeInterface::diff`.
//!
//! Called from:
//! - Shared DateTime method assembly.
//!
//! Key details:
//! - Exact elapsed days and calendar components preserve sign and microsecond carry behavior.

use super::*;

/// `DateTimeInterface::diff(DateTimeInterface $target): DateInterval` — exact elapsed difference.
///
/// Populates a fresh `DateInterval` with the total `days` and the `h`/`i`/`s` remainder computed
/// from the timestamp difference, plus `invert` (1 when `$target` precedes `$this`), and the
/// calendar `y`/`m`/`d` breakdown counted by advancing whole years/months/days through `mktime()`.
/// `days` is the exact whole-day count.
pub(super) fn datetime_diff_method() -> ClassMethod {
    let target_ts = Expr::new(
        ExprKind::MethodCall {
            object: Box::new(Expr::new(ExprKind::Variable("targetObject".to_string()), dummy())),
            method: "getTimestamp".to_string(),
            args: Vec::new(),
        },
        dummy(),
    );
    // $target->getMicrosecond() — read the target's sub-second component (PHP 8.4
    // promoted it onto DateTimeInterface).
    let target_micro = Expr::new(
        ExprKind::MethodCall {
            object: Box::new(Expr::new(ExprKind::Variable("targetObject".to_string()), dummy())),
            method: "getMicrosecond".to_string(),
            args: Vec::new(),
        },
        dummy(),
    );
    let secs_var = || Expr::new(ExprKind::Variable("secs".to_string()), dummy());
    let rem_var = || Expr::new(ExprKind::Variable("rem".to_string()), dummy());
    let iv_var = || Expr::new(ExprKind::Variable("iv".to_string()), dummy());
    let int_lit = |n: i64| Expr::new(ExprKind::IntLiteral(n), dummy());
    let binop = |l: Expr, op: BinOp, r: Expr| {
        Expr::new(ExprKind::BinaryOp { left: Box::new(l), op, right: Box::new(r) }, dummy())
    };
    // Integer division via the PHP intdiv() builtin. (It now unboxes Mixed/Union operands, so it is
    // safe here even though $secs/$rem are Mixed locals derived from an interface method call.)
    let intdiv = |a: Expr, b: Expr| {
        Expr::new(
            ExprKind::FunctionCall { name: Name::unqualified("intdiv"), args: vec![a, b] },
            dummy(),
        )
    };
    let set_iv = |prop: &str, value: Expr| {
        Stmt::new(
            StmtKind::PropertyAssign {
                object: Box::new(iv_var()),
                property: prop.to_string(),
                value,
            },
            dummy(),
        )
    };
    let var = |n: &str| Expr::new(ExprKind::Variable(n.to_string()), dummy());
    // (int)date(fmt, $ts_var): decompose a timestamp local into one calendar component.
    let date_of = |fmt: &str, ts: &str| {
        Expr::new(
            ExprKind::Cast {
                target: crate::parser::ast::CastType::Int,
                expr: Box::new(Expr::new(
                    ExprKind::FunctionCall {
                        name: Name::unqualified("date"),
                        args: vec![
                            Expr::new(ExprKind::StringLiteral(fmt.to_string()), dummy()),
                            Expr::new(ExprKind::Variable(ts.to_string()), dummy()),
                        ],
                    },
                    dummy(),
                )),
            },
            dummy(),
        )
    };
    let mktime6 = |h: Expr, mi: Expr, s: Expr, mo: Expr, d: Expr, y: Expr| {
        Expr::new(
            ExprKind::FunctionCall { name: Name::unqualified("__elephc_mktime_raw"), args: vec![h, mi, s, mo, d, y] },
            dummy(),
        )
    };
    // while (<candidate> <= $later) { $ctr = $ctr + 1; }: count whole calendar units.
    let advance_while = |ctr: &str, candidate: Expr| {
        Stmt::new(
            StmtKind::While {
                condition: binop(candidate, BinOp::LtEq, var("later")),
                body: vec![Stmt::assign(ctr, binop(var(ctr), BinOp::Add, int_lit(1)))],
            },
            dummy(),
        )
    };
    method(
        "diff",
        vec![
            (
                "targetObject".to_string(),
                Some(TypeExpr::Named(Name::unqualified("DateTimeInterface"))),
                None,
                false,
            ),
            (
                "absolute".to_string(),
                Some(TypeExpr::Bool),
                Some(Expr::new(ExprKind::BoolLiteral(false), dummy())),
                false,
            ),
        ],
        Some(TypeExpr::Named(Name::unqualified("DateInterval"))),
        vec![
            // Cache $this->timestamp BEFORE the method call: evaluating $target->getTimestamp()
            // first would otherwise clobber the $this receiver before the property read.
            Stmt::assign("base", this_property("timestamp")),
            // Read $this->microsecond before the target method calls clobber the receiver.
            Stmt::assign("mus", this_property("microsecond")),
            // $tts = $target->getTimestamp();
            Stmt::assign("tts", target_ts),
            // $mut = $target->getMicrosecond();
            Stmt::assign("mut", target_micro),
            // $secs = $tts - $base;
            Stmt::assign("secs", binop(var("tts"), BinOp::Sub, var("base"))),
            // $iv = new DateInterval("P0D");
            Stmt::assign(
                "iv",
                Expr::new(
                    ExprKind::NewObject {
                        class_name: Name::unqualified("DateInterval"),
                        args: vec![Expr::new(ExprKind::StringLiteral("P0D".to_string()), dummy())],
                    },
                    dummy(),
                ),
            ),
            // Order by the full instant (seconds, then microseconds): invert when $target is
            // earlier — including the same-second case where its microseconds are smaller.
            // earlier/later carry the second component; mearlier/mlater the microseconds.
            Stmt::new(
                StmtKind::If {
                    condition: binop(
                        binop(secs_var(), BinOp::Lt, int_lit(0)),
                        BinOp::Or,
                        binop(
                            binop(secs_var(), BinOp::Eq, int_lit(0)),
                            BinOp::And,
                            binop(var("mut"), BinOp::Lt, var("mus")),
                        ),
                    ),
                    then_body: vec![
                        set_iv("invert", int_lit(1)),
                        Stmt::assign("secs", binop(int_lit(0), BinOp::Sub, secs_var())),
                        Stmt::assign("earlier", var("tts")),
                        Stmt::assign("mearlier", var("mut")),
                        Stmt::assign("later", var("base")),
                        Stmt::assign("mlater", var("mus")),
                    ],
                    elseif_clauses: Vec::new(),
                    else_body: Some(vec![
                        Stmt::assign("earlier", var("base")),
                        Stmt::assign("mearlier", var("mus")),
                        Stmt::assign("later", var("tts")),
                        Stmt::assign("mlater", var("mut")),
                    ]),
                },
                dummy(),
            ),
            // Fractional-second difference with a one-second borrow: when the later
            // microseconds are smaller, borrow a whole second into the fraction. This keeps
            // $secs and $later consistent for the breakdown and calendar walk below.
            Stmt::assign("frac", binop(var("mlater"), BinOp::Sub, var("mearlier"))),
            Stmt::new(
                StmtKind::If {
                    condition: binop(var("frac"), BinOp::Lt, int_lit(0)),
                    then_body: vec![
                        Stmt::assign("frac", binop(var("frac"), BinOp::Add, int_lit(1_000_000))),
                        Stmt::assign("later", binop(var("later"), BinOp::Sub, int_lit(1))),
                        Stmt::assign("secs", binop(secs_var(), BinOp::Sub, int_lit(1))),
                    ],
                    elseif_clauses: Vec::new(),
                    else_body: None,
                },
                dummy(),
            ),
            // $iv->f = $frac / 1000000.0;
            set_iv(
                "f",
                binop(
                    var("frac"),
                    BinOp::Div,
                    Expr::new(ExprKind::FloatLiteral(1_000_000.0), dummy()),
                ),
            ),
            // $iv->days = intdiv($secs, 86400);
            set_iv("days", intdiv(secs_var(), int_lit(86400))),
            // $rem = $secs % 86400;
            Stmt::assign("rem", binop(secs_var(), BinOp::Mod, int_lit(86400))),
            // $iv->h = intdiv($rem, 3600);
            set_iv("h", intdiv(rem_var(), int_lit(3600))),
            // $iv->i = intdiv($rem % 3600, 60);
            set_iv("i", intdiv(binop(rem_var(), BinOp::Mod, int_lit(3600)), int_lit(60))),
            // $iv->s = $rem % 60;
            set_iv("s", binop(rem_var(), BinOp::Mod, int_lit(60))),
            // -- calendar components: decompose the earlier date, then count whole years, months,
            //    and days by advancing through mktime() (which normalizes month/day overflow)
            //    until the next unit would pass $later. Matches PHP's calendar y/m/d breakdown.
            Stmt::assign("ey", date_of("Y", "earlier")),
            Stmt::assign("emo", date_of("n", "earlier")),
            Stmt::assign("ed", date_of("j", "earlier")),
            Stmt::assign("eh", date_of("G", "earlier")),
            Stmt::assign("ei", date_of("i", "earlier")),
            Stmt::assign("es", date_of("s", "earlier")),
            // years: while mktime(eh,ei,es, emo, ed, ey + y + 1) <= later { y++ }
            Stmt::assign("y", int_lit(0)),
            advance_while(
                "y",
                mktime6(
                    var("eh"),
                    var("ei"),
                    var("es"),
                    var("emo"),
                    var("ed"),
                    binop(binop(var("ey"), BinOp::Add, var("y")), BinOp::Add, int_lit(1)),
                ),
            ),
            // months: while mktime(eh,ei,es, emo + m + 1, ed, ey + y) <= later { m++ }
            Stmt::assign("m", int_lit(0)),
            advance_while(
                "m",
                mktime6(
                    var("eh"),
                    var("ei"),
                    var("es"),
                    binop(binop(var("emo"), BinOp::Add, var("m")), BinOp::Add, int_lit(1)),
                    var("ed"),
                    binop(var("ey"), BinOp::Add, var("y")),
                ),
            ),
            // days: while mktime(eh,ei,es, emo + m, ed + d + 1, ey + y) <= later { d++ }
            Stmt::assign("d", int_lit(0)),
            advance_while(
                "d",
                mktime6(
                    var("eh"),
                    var("ei"),
                    var("es"),
                    binop(var("emo"), BinOp::Add, var("m")),
                    binop(binop(var("ed"), BinOp::Add, var("d")), BinOp::Add, int_lit(1)),
                    binop(var("ey"), BinOp::Add, var("y")),
                ),
            ),
            set_iv("y", var("y")),
            set_iv("m", var("m")),
            set_iv("d", var("d")),
            // PHP's `$absolute` flag forces a positive interval: drop the invert flag set above so
            // the returned DateInterval never reads as negative regardless of argument order.
            Stmt::new(
                StmtKind::If {
                    condition: var("absolute"),
                    then_body: vec![set_iv("invert", int_lit(0))],
                    elseif_clauses: Vec::new(),
                    else_body: None,
                },
                dummy(),
            ),
            return_expr(iv_var()),
        ],
    )
}
