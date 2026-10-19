//! Purpose:
//! Builds `Elephc\Testing\Runner`: runs one test at a time, in a forked child by default,
//! prints one line per outcome, keeps the totals, writes the optional JUnit report, and
//! turns the totals into the process exit code.
//!
//! Called from:
//! - `crate::testing_prelude::build::testing_declarations`.
//!
//! Key details:
//! - TRANSCRIBED from the PHP form in `crate::testing_prelude::fragments`; the oracle in
//!   `testing_prelude::oracle_tests` keeps the two in step.
//! - ISOLATION is plain PHP over the compiled pcntl builtins: `stream_socket_pair` for the
//!   outcome record, `pcntl_fork`, `pcntl_alarm` for the timeout (signal 14 is reported as
//!   the timeout), `pcntl_waitpid` and the `wif*` decoders. A child that dies before
//!   reporting (a native fatal, a signal) is an ERROR naming the signal or exit code, and
//!   the run continues. `--no-isolation` runs the test in the runner's own process.
//! - The outcome record crossing the pipe is `status\nassertions\nmessage`.
//! - An escaping `AssertionFailedError` is a FAILURE, `SkippedTestError` a SKIP,
//!   `IncompleteTestError` an INCOMPLETE, any other `Throwable` an ERROR; each keeps the
//!   assertions the test made before it stopped. Skips and incompletes do not fail the run.
//! - The report lines (`ok   `, `FAIL `, `ERROR `, `SKIP `, `INCOMPLETE `, the `OK (...)`,
//!   `OK, but there were issues!` and `Tests: ...` totals) are a stable surface: Ivory's
//!   `make test` and the codegen tests read them.

use crate::parser::ast::{BinOp, CastType, Stmt, TypeExpr};
use crate::synthetic_class::{
    class, e_array, e_binop, e_bool, e_call, e_cast, e_const, e_index, e_int, e_method_call,
    e_neg, e_not, e_str, e_ternary, e_this, e_this_prop, e_var, method, s_assign, s_echo, s_expr,
    s_foreach, s_if, s_prop_array_push, s_prop_assign, s_return, s_return_void, s_try, t_array,
    t_class, t_mixed,
};

/// Builds the final `Runner` class.
pub(super) fn decl_runner() -> Stmt {
    class("Runner")
        .final_()
        .private_prop("tests", TypeExpr::Int, Some(e_int(0)))
        .private_prop("assertions", TypeExpr::Int, Some(e_int(0)))
        .private_prop("failures", TypeExpr::Int, Some(e_int(0)))
        .private_prop("errors", TypeExpr::Int, Some(e_int(0)))
        .private_prop("skipped", TypeExpr::Int, Some(e_int(0)))
        .private_prop("incomplete", TypeExpr::Int, Some(e_int(0)))
        .private_prop("isolate", TypeExpr::Bool, None)
        .private_prop("timeout", TypeExpr::Int, None)
        .private_prop("filter", TypeExpr::Str, None)
        .private_prop("junit", TypeExpr::Str, None)
        .private_prop("results", t_array(), Some(e_array(vec![])))
        .private_prop("started", TypeExpr::Float, None)
        .method(
            method("__construct")
                .param_default("isolate", TypeExpr::Bool, e_bool(true))
                .param_default("timeout", TypeExpr::Int, e_int(60))
                .param_default("filter", TypeExpr::Str, e_str(""))
                .param_default("junit", TypeExpr::Str, e_str(""))
                .body(vec![
                    s_prop_assign(e_this(), "isolate", e_var("isolate")),
                    s_prop_assign(e_this(), "timeout", e_var("timeout")),
                    s_prop_assign(e_this(), "filter", e_var("filter")),
                    s_prop_assign(e_this(), "junit", e_var("junit")),
                    s_prop_assign(e_this(), "started", e_call("microtime", vec![e_bool(true)])),
                ]),
        )
        .method(
            method("run")
                .param("case", t_class("TestCase"))
                .param("name", TypeExpr::Str)
                .param("test", t_class("callable"))
                .returns(TypeExpr::Void)
                .body(vec![
                    s_expr(e_method_call(e_this(), "runLabelled", vec![e_var("case"), e_var("name"), e_var("test")])),
                ]),
        )
        .method(
            method("runWithData")
                .param("case", t_class("TestCase"))
                .param("name", TypeExpr::Str)
                .param("test", t_class("callable"))
                .param("dataSet", t_mixed())
                .returns(TypeExpr::Void)
                .body(vec![
                    s_if(
                        e_call("is_int", vec![e_var("dataSet")]),
                        vec![
                            s_expr(e_method_call(e_this(), "runLabelled", vec![e_var("case"), e_binop(e_binop(e_var("name"), BinOp::Concat, e_str(" with data set #")), BinOp::Concat, e_var("dataSet")), e_var("test")])),
                            s_return_void(),
                        ],
                        vec![],
                        None,
                    ),
                    s_expr(e_method_call(e_this(), "runLabelled", vec![e_var("case"), e_binop(e_binop(e_binop(e_var("name"), BinOp::Concat, e_str(" with data set \"")), BinOp::Concat, e_var("dataSet")), BinOp::Concat, e_str("\"")), e_var("test")])),
                ]),
        )
        .method(
            method("runLabelled")
                .private()
                .param("case", t_class("TestCase"))
                .param("name", TypeExpr::Str)
                .param("test", t_class("callable"))
                .returns(TypeExpr::Void)
                .body(vec![
                    s_assign("class", e_call("get_class", vec![e_var("case")])),
                    s_assign("label", e_binop(e_binop(e_var("class"), BinOp::Concat, e_str("::")), BinOp::Concat, e_var("name"))),
                    s_if(
                        e_binop(e_binop(e_this_prop("filter"), BinOp::StrictNotEq, e_str("")), BinOp::And, e_not(e_call("str_contains", vec![e_var("label"), e_this_prop("filter")]))),
                        vec![
                            s_return_void(),
                        ],
                        vec![],
                        None,
                    ),
                    s_prop_assign(e_this(), "tests", e_binop(e_this_prop("tests"), BinOp::Add, e_int(1))),
                    s_assign("began", e_call("microtime", vec![e_bool(true)])),
                    s_if(
                        e_this_prop("isolate"),
                        vec![
                            s_assign("record", e_method_call(e_this(), "runIsolated", vec![e_var("case"), e_var("test")])),
                        ],
                        vec![],
                        Some(vec![
                        s_assign("record", e_method_call(e_this(), "execute", vec![e_var("case"), e_var("test")])),
                    ]),
                    ),
                    s_expr(e_method_call(e_this(), "record", vec![e_var("class"), e_var("name"), e_var("label"), e_var("record"), e_binop(e_call("microtime", vec![e_bool(true)]), BinOp::Sub, e_var("began"))])),
                ]),
        )
        .method(
            method("execute")
                .private()
                .param("case", t_class("TestCase"))
                .param("test", t_class("callable"))
                .returns(TypeExpr::Str)
                .body(vec![
                    s_try(vec![
                        s_expr(e_method_call(e_var("case"), "__elephc_run_test", vec![e_var("test")])),
                        s_return(e_binop(e_binop(e_str("passed\n"), BinOp::Concat, e_method_call(e_var("case"), "__elephc_assertion_count", vec![])), BinOp::Concat, e_str("\n"))),
                    ], vec![
                        (vec!["AssertionFailedError"], Some("failure"), vec![
                            s_return(e_binop(e_binop(e_binop(e_str("failed\n"), BinOp::Concat, e_method_call(e_var("case"), "__elephc_assertion_count", vec![])), BinOp::Concat, e_str("\n")), BinOp::Concat, e_method_call(e_var("failure"), "getMessage", vec![]))),
                        ]),
                        (vec!["SkippedTestError"], Some("skipped"), vec![
                            s_return(e_binop(e_binop(e_binop(e_str("skipped\n"), BinOp::Concat, e_method_call(e_var("case"), "__elephc_assertion_count", vec![])), BinOp::Concat, e_str("\n")), BinOp::Concat, e_method_call(e_var("skipped"), "getMessage", vec![]))),
                        ]),
                        (vec!["IncompleteTestError"], Some("incomplete"), vec![
                            s_return(e_binop(e_binop(e_binop(e_str("incomplete\n"), BinOp::Concat, e_method_call(e_var("case"), "__elephc_assertion_count", vec![])), BinOp::Concat, e_str("\n")), BinOp::Concat, e_method_call(e_var("incomplete"), "getMessage", vec![]))),
                        ]),
                        (vec!["\\Throwable"], Some("error"), vec![
                            s_return(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("error\n"), BinOp::Concat, e_method_call(e_var("case"), "__elephc_assertion_count", vec![])), BinOp::Concat, e_str("\n")), BinOp::Concat, e_call("get_class", vec![e_var("error")])), BinOp::Concat, e_str(": ")), BinOp::Concat, e_method_call(e_var("error"), "getMessage", vec![]))),
                        ]),
                    ], None),
                ]),
        )
        .method(
            method("runIsolated")
                .private()
                .param("case", t_class("TestCase"))
                .param("test", t_class("callable"))
                .returns(TypeExpr::Str)
                .body(vec![
                    s_assign("pair", e_call("stream_socket_pair", vec![e_const("STREAM_PF_UNIX"), e_const("STREAM_SOCK_STREAM"), e_const("STREAM_IPPROTO_IP")])),
                    s_assign("pid", e_call("pcntl_fork", vec![])),
                    s_if(
                        e_binop(e_var("pid"), BinOp::StrictEq, e_neg(e_int(1))),
                        vec![
                            s_expr(e_call("fclose", vec![e_index(e_var("pair"), e_int(0))])),
                            s_expr(e_call("fclose", vec![e_index(e_var("pair"), e_int(1))])),
                            s_return(e_method_call(e_this(), "execute", vec![e_var("case"), e_var("test")])),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_binop(e_var("pid"), BinOp::StrictEq, e_int(0)),
                        vec![
                            s_expr(e_call("fclose", vec![e_index(e_var("pair"), e_int(0))])),
                            s_if(
                                e_binop(e_this_prop("timeout"), BinOp::Gt, e_int(0)),
                                vec![
                                    s_expr(e_call("pcntl_alarm", vec![e_this_prop("timeout")])),
                                ],
                                vec![],
                                None,
                            ),
                            s_assign("childRecord", e_method_call(e_this(), "execute", vec![e_var("case"), e_var("test")])),
                            s_expr(e_call("fwrite", vec![e_index(e_var("pair"), e_int(1)), e_var("childRecord")])),
                            s_expr(e_call("fclose", vec![e_index(e_var("pair"), e_int(1))])),
                            s_expr(e_call("exit", vec![e_int(0)])),
                        ],
                        vec![],
                        None,
                    ),
                    s_expr(e_call("fclose", vec![e_index(e_var("pair"), e_int(1))])),
                    s_assign("payload", e_call("stream_get_contents", vec![e_index(e_var("pair"), e_int(0))])),
                    s_expr(e_call("fclose", vec![e_index(e_var("pair"), e_int(0))])),
                    s_assign("record", e_ternary(e_call("is_string", vec![e_var("payload")]), e_var("payload"), e_str(""))),
                    s_assign("status", e_int(0)),
                    s_expr(e_call("pcntl_waitpid", vec![e_var("pid"), e_var("status")])),
                    s_if(
                        e_binop(e_binop(e_binop(e_var("record"), BinOp::StrictNotEq, e_str("")), BinOp::And, e_call("pcntl_wifexited", vec![e_var("status")])), BinOp::And, e_binop(e_call("pcntl_wexitstatus", vec![e_var("status")]), BinOp::StrictEq, e_int(0))),
                        vec![
                            s_return(e_var("record")),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_call("pcntl_wifsignaled", vec![e_var("status")]),
                        vec![
                            s_assign("signal", e_call("pcntl_wtermsig", vec![e_var("status")])),
                            s_if(
                                e_binop(e_var("signal"), BinOp::StrictEq, e_int(14)),
                                vec![
                                    s_return(e_binop(e_binop(e_str("error\n0\nTest exceeded the timeout of "), BinOp::Concat, e_this_prop("timeout")), BinOp::Concat, e_ternary(e_binop(e_this_prop("timeout"), BinOp::StrictEq, e_int(1)), e_str(" second"), e_str(" seconds")))),
                                ],
                                vec![],
                                None,
                            ),
                            s_return(e_binop(e_binop(e_str("error\n0\nTest process killed by signal "), BinOp::Concat, e_var("signal")), BinOp::Concat, e_method_call(e_this(), "signalName", vec![e_var("signal")]))),
                        ],
                        vec![],
                        None,
                    ),
                    s_return(e_binop(e_binop(e_str("error\n0\nTest process exited with code "), BinOp::Concat, e_call("pcntl_wexitstatus", vec![e_var("status")])), BinOp::Concat, e_str(" before reporting a result"))),
                ]),
        )
        .method(
            method("signalName")
                .private()
                .param("signal", TypeExpr::Int)
                .returns(TypeExpr::Str)
                .body(vec![
                    s_if(
                        e_binop(e_var("signal"), BinOp::StrictEq, e_int(6)),
                        vec![
                            s_return(e_str(" (SIGABRT)")),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_binop(e_var("signal"), BinOp::StrictEq, e_int(7)),
                        vec![
                            s_return(e_str(" (SIGBUS)")),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_binop(e_var("signal"), BinOp::StrictEq, e_int(8)),
                        vec![
                            s_return(e_str(" (SIGFPE)")),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_binop(e_var("signal"), BinOp::StrictEq, e_int(9)),
                        vec![
                            s_return(e_str(" (SIGKILL)")),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_binop(e_var("signal"), BinOp::StrictEq, e_int(11)),
                        vec![
                            s_return(e_str(" (SIGSEGV)")),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_binop(e_var("signal"), BinOp::StrictEq, e_int(15)),
                        vec![
                            s_return(e_str(" (SIGTERM)")),
                        ],
                        vec![],
                        None,
                    ),
                    s_return(e_str("")),
                ]),
        )
        .method(
            method("record")
                .private()
                .param("class", TypeExpr::Str)
                .param("name", TypeExpr::Str)
                .param("label", TypeExpr::Str)
                .param("record", TypeExpr::Str)
                .param("seconds", TypeExpr::Float)
                .returns(TypeExpr::Void)
                .body(vec![
                    s_assign("parts", e_call("explode", vec![e_str("\n"), e_var("record"), e_int(3)])),
                    s_assign("status", e_index(e_var("parts"), e_int(0))),
                    s_assign("count", e_ternary(e_binop(e_call("count", vec![e_var("parts")]), BinOp::Gt, e_int(1)), e_cast(CastType::Int, e_index(e_var("parts"), e_int(1))), e_int(0))),
                    s_assign("message", e_ternary(e_binop(e_call("count", vec![e_var("parts")]), BinOp::Gt, e_int(2)), e_index(e_var("parts"), e_int(2)), e_str(""))),
                    s_prop_assign(e_this(), "assertions", e_binop(e_this_prop("assertions"), BinOp::Add, e_var("count"))),
                    s_if(
                        e_binop(e_var("status"), BinOp::StrictEq, e_str("passed")),
                        vec![
                            s_echo(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("ok   "), BinOp::Concat, e_var("label")), BinOp::Concat, e_str(" (")), BinOp::Concat, e_var("count")), BinOp::Concat, e_ternary(e_binop(e_var("count"), BinOp::StrictEq, e_int(1)), e_str(" assertion)"), e_str(" assertions)"))), BinOp::Concat, e_str("\n"))),
                        ],
                        vec![
                        (e_binop(e_var("status"), BinOp::StrictEq, e_str("failed")), vec![
                            s_prop_assign(e_this(), "failures", e_binop(e_this_prop("failures"), BinOp::Add, e_int(1))),
                            s_echo(e_binop(e_binop(e_str("FAIL "), BinOp::Concat, e_var("label")), BinOp::Concat, e_str("\n"))),
                            s_expr(e_method_call(e_this(), "echoIndented", vec![e_var("message")])),
                        ]),
                        (e_binop(e_var("status"), BinOp::StrictEq, e_str("skipped")), vec![
                            s_prop_assign(e_this(), "skipped", e_binop(e_this_prop("skipped"), BinOp::Add, e_int(1))),
                            s_echo(e_binop(e_binop(e_str("SKIP "), BinOp::Concat, e_var("label")), BinOp::Concat, e_str("\n"))),
                            s_expr(e_method_call(e_this(), "echoIndented", vec![e_var("message")])),
                        ]),
                        (e_binop(e_var("status"), BinOp::StrictEq, e_str("incomplete")), vec![
                            s_prop_assign(e_this(), "incomplete", e_binop(e_this_prop("incomplete"), BinOp::Add, e_int(1))),
                            s_echo(e_binop(e_binop(e_str("INCOMPLETE "), BinOp::Concat, e_var("label")), BinOp::Concat, e_str("\n"))),
                            s_expr(e_method_call(e_this(), "echoIndented", vec![e_var("message")])),
                        ]),
                    ],
                        Some(vec![
                        s_prop_assign(e_this(), "errors", e_binop(e_this_prop("errors"), BinOp::Add, e_int(1))),
                        s_echo(e_binop(e_binop(e_str("ERROR "), BinOp::Concat, e_var("label")), BinOp::Concat, e_str("\n"))),
                        s_expr(e_method_call(e_this(), "echoIndented", vec![e_var("message")])),
                    ]),
                    ),
                    s_prop_array_push(e_this(), "results", e_array(vec![e_var("class"), e_var("name"), e_var("status"), e_var("message"), e_call("number_format", vec![e_var("seconds"), e_int(6), e_str("."), e_str("")])])),
                ]),
        )
        .method(
            method("echoIndented")
                .private()
                .param("text", TypeExpr::Str)
                .returns(TypeExpr::Void)
                .body(vec![
                    s_if(
                        e_binop(e_var("text"), BinOp::StrictEq, e_str("")),
                        vec![
                            s_return_void(),
                        ],
                        vec![],
                        None,
                    ),
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
                        e_binop(e_this_prop("junit"), BinOp::StrictNotEq, e_str("")),
                        vec![
                            s_expr(e_method_call(e_this(), "writeJunit", vec![])),
                        ],
                        vec![],
                        None,
                    ),
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
                            s_if(
                                e_binop(e_binop(e_this_prop("skipped"), BinOp::StrictEq, e_int(0)), BinOp::And, e_binop(e_this_prop("incomplete"), BinOp::StrictEq, e_int(0))),
                                vec![
                                    s_echo(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("OK ("), BinOp::Concat, e_this_prop("tests")), BinOp::Concat, e_ternary(e_binop(e_this_prop("tests"), BinOp::StrictEq, e_int(1)), e_str(" test, "), e_str(" tests, "))), BinOp::Concat, e_this_prop("assertions")), BinOp::Concat, e_ternary(e_binop(e_this_prop("assertions"), BinOp::StrictEq, e_int(1)), e_str(" assertion)"), e_str(" assertions)"))), BinOp::Concat, e_str("\n"))),
                                    s_return(e_int(0)),
                                ],
                                vec![],
                                None,
                            ),
                            s_echo(e_str("OK, but there were issues!\n")),
                            s_echo(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("Tests: "), BinOp::Concat, e_this_prop("tests")), BinOp::Concat, e_str(", Assertions: ")), BinOp::Concat, e_this_prop("assertions")), BinOp::Concat, e_method_call(e_this(), "issueCounts", vec![])), BinOp::Concat, e_str(".")), BinOp::Concat, e_str("\n"))),
                            s_return(e_int(0)),
                        ],
                        vec![],
                        None,
                    ),
                    s_echo(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("Tests: "), BinOp::Concat, e_this_prop("tests")), BinOp::Concat, e_str(", Assertions: ")), BinOp::Concat, e_this_prop("assertions")), BinOp::Concat, e_str(", Failures: ")), BinOp::Concat, e_this_prop("failures")), BinOp::Concat, e_str(", Errors: ")), BinOp::Concat, e_this_prop("errors")), BinOp::Concat, e_method_call(e_this(), "issueCounts", vec![])), BinOp::Concat, e_str(".")), BinOp::Concat, e_str("\n"))),
                    s_return(e_int(1)),
                ]),
        )
        .method(
            method("issueCounts")
                .private()
                .returns(TypeExpr::Str)
                .body(vec![
                    s_assign("text", e_str("")),
                    s_if(
                        e_binop(e_this_prop("skipped"), BinOp::Gt, e_int(0)),
                        vec![
                            s_assign("text", e_binop(e_var("text"), BinOp::Concat, e_binop(e_str(", Skipped: "), BinOp::Concat, e_this_prop("skipped")))),
                        ],
                        vec![],
                        None,
                    ),
                    s_if(
                        e_binop(e_this_prop("incomplete"), BinOp::Gt, e_int(0)),
                        vec![
                            s_assign("text", e_binop(e_var("text"), BinOp::Concat, e_binop(e_str(", Incomplete: "), BinOp::Concat, e_this_prop("incomplete")))),
                        ],
                        vec![],
                        None,
                    ),
                    s_return(e_var("text")),
                ]),
        )
        .method(
            method("writeJunit")
                .private()
                .returns(TypeExpr::Void)
                .body(vec![
                    s_assign("xml", e_binop(e_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"), BinOp::Concat, e_str("\n"))),
                    s_assign("xml", e_binop(e_var("xml"), BinOp::Concat, e_binop(e_str("<testsuites>"), BinOp::Concat, e_str("\n")))),
                    s_assign("xml", e_binop(e_var("xml"), BinOp::Concat, e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("  <testsuite name=\"elephc test\" tests=\""), BinOp::Concat, e_this_prop("tests")), BinOp::Concat, e_str("\" assertions=\"")), BinOp::Concat, e_this_prop("assertions")), BinOp::Concat, e_str("\" failures=\"")), BinOp::Concat, e_this_prop("failures")), BinOp::Concat, e_str("\" errors=\"")), BinOp::Concat, e_this_prop("errors")), BinOp::Concat, e_str("\" skipped=\"")), BinOp::Concat, e_binop(e_this_prop("skipped"), BinOp::Add, e_this_prop("incomplete"))), BinOp::Concat, e_str("\" time=\"")), BinOp::Concat, e_call("number_format", vec![e_binop(e_call("microtime", vec![e_bool(true)]), BinOp::Sub, e_this_prop("started")), e_int(6), e_str("."), e_str("")])), BinOp::Concat, e_str("\">")), BinOp::Concat, e_str("\n")))),
                    s_foreach(e_this_prop("results"), None, "result", vec![
                        s_assign("status", e_index(e_var("result"), e_int(2))),
                        s_assign("xml", e_binop(e_var("xml"), BinOp::Concat, e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_str("    <testcase classname=\""), BinOp::Concat, e_method_call(e_this(), "xmlEscape", vec![e_index(e_var("result"), e_int(0))])), BinOp::Concat, e_str("\" name=\"")), BinOp::Concat, e_method_call(e_this(), "xmlEscape", vec![e_index(e_var("result"), e_int(1))])), BinOp::Concat, e_str("\" time=\"")), BinOp::Concat, e_index(e_var("result"), e_int(4))), BinOp::Concat, e_str("\"")))),
                        s_if(
                            e_binop(e_var("status"), BinOp::StrictEq, e_str("passed")),
                            vec![
                                s_assign("xml", e_binop(e_var("xml"), BinOp::Concat, e_binop(e_str(" />"), BinOp::Concat, e_str("\n")))),
                            ],
                            vec![
                            (e_binop(e_var("status"), BinOp::StrictEq, e_str("failed")), vec![
                                s_assign("xml", e_binop(e_var("xml"), BinOp::Concat, e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_str(">"), BinOp::Concat, e_str("\n")), BinOp::Concat, e_str("      <failure type=\"failure\">")), BinOp::Concat, e_method_call(e_this(), "xmlEscape", vec![e_index(e_var("result"), e_int(3))])), BinOp::Concat, e_str("</failure>")), BinOp::Concat, e_str("\n")), BinOp::Concat, e_str("    </testcase>")), BinOp::Concat, e_str("\n")))),
                            ]),
                            (e_binop(e_binop(e_var("status"), BinOp::StrictEq, e_str("skipped")), BinOp::Or, e_binop(e_var("status"), BinOp::StrictEq, e_str("incomplete"))), vec![
                                s_assign("xml", e_binop(e_var("xml"), BinOp::Concat, e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_str(">"), BinOp::Concat, e_str("\n")), BinOp::Concat, e_str("      <skipped message=\"")), BinOp::Concat, e_method_call(e_this(), "xmlEscape", vec![e_index(e_var("result"), e_int(3))])), BinOp::Concat, e_str("\" />")), BinOp::Concat, e_str("\n")), BinOp::Concat, e_str("    </testcase>")), BinOp::Concat, e_str("\n")))),
                            ]),
                        ],
                            Some(vec![
                            s_assign("xml", e_binop(e_var("xml"), BinOp::Concat, e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_str(">"), BinOp::Concat, e_str("\n")), BinOp::Concat, e_str("      <error type=\"error\">")), BinOp::Concat, e_method_call(e_this(), "xmlEscape", vec![e_index(e_var("result"), e_int(3))])), BinOp::Concat, e_str("</error>")), BinOp::Concat, e_str("\n")), BinOp::Concat, e_str("    </testcase>")), BinOp::Concat, e_str("\n")))),
                        ]),
                        ),
                    ]),
                    s_assign("xml", e_binop(e_var("xml"), BinOp::Concat, e_binop(e_str("  </testsuite>"), BinOp::Concat, e_str("\n")))),
                    s_assign("xml", e_binop(e_var("xml"), BinOp::Concat, e_binop(e_str("</testsuites>"), BinOp::Concat, e_str("\n")))),
                    s_expr(e_call("file_put_contents", vec![e_this_prop("junit"), e_var("xml")])),
                ]),
        )
        .method(
            method("xmlEscape")
                .private()
                .param("text", TypeExpr::Str)
                .returns(TypeExpr::Str)
                .body(vec![
                    s_assign("text", e_call("str_replace", vec![e_str("&"), e_str("&amp;"), e_var("text")])),
                    s_assign("text", e_call("str_replace", vec![e_str("<"), e_str("&lt;"), e_var("text")])),
                    s_assign("text", e_call("str_replace", vec![e_str(">"), e_str("&gt;"), e_var("text")])),
                    s_assign("text", e_call("str_replace", vec![e_str("\""), e_str("&quot;"), e_var("text")])),
                    s_return(e_call("str_replace", vec![e_str("'"), e_str("&apos;"), e_var("text")])),
                ]),
        )
        .build()
}
