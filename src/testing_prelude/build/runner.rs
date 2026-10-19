//! Purpose:
//! Builds `Elephc\Testing\Runner`: runs one test at a time through
//! `TestCase::__elephc_run_test`, prints one line per outcome, and turns the totals into
//! the process exit code.
//!
//! Called from:
//! - `crate::testing_prelude::build::testing_declarations`.
//!
//! Key details:
//! - TRANSCRIBED from the PHP form in `crate::testing_prelude::fragments`; the oracle in
//!   `testing_prelude::oracle_tests` keeps the two in step.
//! - An escaping `AssertionFailedError` is a FAILURE, any other `Throwable` is an ERROR;
//!   both keep the assertions the test made before it stopped.
//! - The report lines (`ok   `, `FAIL `, `ERROR `, the `OK (...)` and `Tests: ...` totals)
//!   are a stable surface: Ivory's `make test` and the codegen tests read them.

use crate::parser::ast::{BinOp, Stmt, TypeExpr};
use crate::synthetic_class::{
    class, e_binop, e_call, e_int, e_method_call, e_str, e_ternary, e_this, e_this_prop, e_var,
    method, s_assign, s_echo, s_expr, s_foreach, s_if, s_prop_assign, s_return, s_try, t_class,
};

/// Builds the final `Runner` class.
pub(super) fn decl_runner() -> Stmt {
    class("Runner")
        .final_()
        .private_prop("tests", TypeExpr::Int, Some(e_int(0)))
        .private_prop("assertions", TypeExpr::Int, Some(e_int(0)))
        .private_prop("failures", TypeExpr::Int, Some(e_int(0)))
        .private_prop("errors", TypeExpr::Int, Some(e_int(0)))
        .method(
            method("run")
                .param("case", t_class("TestCase"))
                .param("name", TypeExpr::Str)
                .param("test", t_class("callable"))
                .returns(TypeExpr::Void)
                .body(vec![
                    s_prop_assign(e_this(), "tests", e_binop(e_this_prop("tests"), BinOp::Add, e_int(1))),
                    s_assign("label", e_binop(e_binop(e_call("get_class", vec![e_var("case")]), BinOp::Concat, e_str("::")), BinOp::Concat, e_var("name"))),
                    s_try(vec![
                        s_expr(e_method_call(e_var("case"), "__elephc_run_test", vec![e_var("test")])),
                        s_assign("count", e_method_call(e_var("case"), "__elephc_assertion_count", vec![])),
                        s_prop_assign(e_this(), "assertions", e_binop(e_this_prop("assertions"), BinOp::Add, e_var("count"))),
                        s_echo(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("ok   "), BinOp::Concat, e_var("label")), BinOp::Concat, e_str(" (")), BinOp::Concat, e_var("count")), BinOp::Concat, e_ternary(e_binop(e_var("count"), BinOp::StrictEq, e_int(1)), e_str(" assertion)"), e_str(" assertions)"))), BinOp::Concat, e_str("\n"))),
                    ], vec![
                        (vec!["AssertionFailedError"], Some("failure"), vec![
                            s_prop_assign(e_this(), "assertions", e_binop(e_this_prop("assertions"), BinOp::Add, e_method_call(e_var("case"), "__elephc_assertion_count", vec![]))),
                            s_prop_assign(e_this(), "failures", e_binop(e_this_prop("failures"), BinOp::Add, e_int(1))),
                            s_echo(e_binop(e_binop(e_str("FAIL "), BinOp::Concat, e_var("label")), BinOp::Concat, e_str("\n"))),
                            s_expr(e_method_call(e_this(), "echoIndented", vec![e_method_call(e_var("failure"), "getMessage", vec![])])),
                        ]),
                        (vec!["\\Throwable"], Some("error"), vec![
                            s_prop_assign(e_this(), "assertions", e_binop(e_this_prop("assertions"), BinOp::Add, e_method_call(e_var("case"), "__elephc_assertion_count", vec![]))),
                            s_prop_assign(e_this(), "errors", e_binop(e_this_prop("errors"), BinOp::Add, e_int(1))),
                            s_echo(e_binop(e_binop(e_str("ERROR "), BinOp::Concat, e_var("label")), BinOp::Concat, e_str("\n"))),
                            s_expr(e_method_call(e_this(), "echoIndented", vec![e_binop(e_binop(e_call("get_class", vec![e_var("error")]), BinOp::Concat, e_str(": ")), BinOp::Concat, e_method_call(e_var("error"), "getMessage", vec![]))])),
                        ]),
                    ], None),
                ]),
        )
        .method(
            method("echoIndented")
                .private()
                .param("text", TypeExpr::Str)
                .returns(TypeExpr::Void)
                .body(vec![
                    s_assign("lines", e_call("explode", vec![e_str("\n"), e_var("text")])),
                    s_foreach(e_var("lines"), None, "line", vec![
                        s_echo(e_binop(e_binop(e_str("     "), BinOp::Concat, e_var("line")), BinOp::Concat, e_str("\n"))),
                    ]),
                ]),
        )
        .method(
            method("finish")
                .returns(TypeExpr::Int)
                .body(vec![
                    s_echo(e_str("\n")),
                    s_if(
                        e_binop(e_this_prop("tests"), BinOp::StrictEq, e_int(0)),
                        vec![
                            s_echo(e_str("No tests executed!\n")),
                            s_return(e_int(1)),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_binop(e_binop(e_this_prop("failures"), BinOp::StrictEq, e_int(0)), BinOp::And, e_binop(e_this_prop("errors"), BinOp::StrictEq, e_int(0))),
                        vec![
                            s_echo(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("OK ("), BinOp::Concat, e_this_prop("tests")), BinOp::Concat, e_ternary(e_binop(e_this_prop("tests"), BinOp::StrictEq, e_int(1)), e_str(" test, "), e_str(" tests, "))), BinOp::Concat, e_this_prop("assertions")), BinOp::Concat, e_ternary(e_binop(e_this_prop("assertions"), BinOp::StrictEq, e_int(1)), e_str(" assertion)"), e_str(" assertions)"))), BinOp::Concat, e_str("\n"))),
                            s_return(e_int(0)),
                        ],
                        vec![],
                        None,
                    ),
                    s_echo(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("Tests: "), BinOp::Concat, e_this_prop("tests")), BinOp::Concat, e_str(", Assertions: ")), BinOp::Concat, e_this_prop("assertions")), BinOp::Concat, e_str(", Failures: ")), BinOp::Concat, e_this_prop("failures")), BinOp::Concat, e_str(", Errors: ")), BinOp::Concat, e_this_prop("errors")), BinOp::Concat, e_str(".")), BinOp::Concat, e_str("\n"))),
                    s_return(e_int(1)),
                ]),
        )
        .build()
}
