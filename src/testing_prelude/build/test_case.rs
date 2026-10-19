//! Purpose:
//! Builds `Elephc\Testing\TestCase`: the lifecycle hooks, the runner-facing entry points,
//! the private failure and value-rendering helpers, and every assertion of the supported
//! subset, one builder helper per method.
//!
//! Called from:
//! - `crate::testing_prelude::build::testing_declarations`.
//!
//! Key details:
//! - TRANSCRIBED from the PHP form in `crate::testing_prelude::fragments` with
//!   `synthetic_class::transcribe`; `testing_prelude::oracle_tests` compares the built class
//!   against that parse node by node, so a change here must keep the PHP form in step.
//! - Method ORDER is the PHP source order: the oracle zips the two method lists.
//! - Every assertion counts itself through `__elephc_pass` / `__elephc_fail`, and a failure
//!   is an `AssertionFailedError` the runner catches; a user message precedes the assertion
//!   text on its own line, as PHPUnit prints it.
//! - Values in failure text go through the private `__elephc_export`, not `var_export()`:
//!   the var_export prelude is detected before this one is injected, so a call to it from
//!   here would not be seen.

use crate::parser::ast::{BinOp, CastType, Stmt, TypeExpr};
use crate::synthetic_class::{
    class, e_binop, e_call, e_cast, e_closure_call, e_int, e_method_call, e_new, e_null, e_str,
    e_ternary, e_this, e_this_prop, e_var, method, s_expr, s_if, s_prop_assign, s_return,
    s_throw, s_try, t_class, t_mixed, ClassBuilder, MethodBuilder,
};

mod assertions;

/// Builds the abstract `TestCase` class with its methods in PHP source order.
pub(super) fn decl_test_case() -> Stmt {
    let mut methods: Vec<MethodBuilder> = vec![
        method_set_up(),
        method_tear_down(),
        method_elephc_run_test(),
        method_elephc_assertion_count(),
        method_fail(),
        method_elephc_pass(),
        method_elephc_fail(),
        method_elephc_export(),
    ];
    methods.extend(assertions::assertion_methods());
    methods
        .into_iter()
        .fold(
            class("TestCase")
                .abstract_()
                .private_prop("__elephc_assertions", TypeExpr::Int, Some(e_int(0))),
            ClassBuilder::method,
        )
        .build()
}

/// `TestCase::setUp()`, transcribed from the PHP form.
fn method_set_up() -> MethodBuilder {
    method("setUp")
        .protected()
        .returns(TypeExpr::Void)
}

/// `TestCase::tearDown()`, transcribed from the PHP form.
fn method_tear_down() -> MethodBuilder {
    method("tearDown")
        .protected()
        .returns(TypeExpr::Void)
}

/// `TestCase::__elephc_run_test()`, transcribed from the PHP form.
fn method_elephc_run_test() -> MethodBuilder {
    method("__elephc_run_test")
        .param("test", t_class("callable"))
        .keep_unread_params()
        .returns(TypeExpr::Void)
        .body(vec![
            s_prop_assign(e_this(), "__elephc_assertions", e_int(0)),
            s_expr(e_method_call(e_this(), "setUp", vec![])),
            s_try(vec![
                s_expr(e_closure_call("test", vec![])),
            ], vec![
            ], Some(vec![
                s_expr(e_method_call(e_this(), "tearDown", vec![])),
            ])),
        ])
}

/// `TestCase::__elephc_assertion_count()`, transcribed from the PHP form.
fn method_elephc_assertion_count() -> MethodBuilder {
    method("__elephc_assertion_count")
        .returns(TypeExpr::Int)
        .body(vec![
            s_return(e_this_prop("__elephc_assertions")),
        ])
}

/// `TestCase::fail()`, transcribed from the PHP form.
fn method_fail() -> MethodBuilder {
    method("fail")
        .protected()
        .param_default("message", TypeExpr::Str, e_str(""))
        .returns(TypeExpr::Void)
        .body(vec![
            s_throw(e_new("AssertionFailedError", vec![e_var("message")])),
        ])
}

/// `TestCase::__elephc_pass()`, transcribed from the PHP form.
fn method_elephc_pass() -> MethodBuilder {
    method("__elephc_pass")
        .private()
        .returns(TypeExpr::Void)
        .body(vec![
            s_prop_assign(e_this(), "__elephc_assertions", e_binop(e_this_prop("__elephc_assertions"), BinOp::Add, e_int(1))),
        ])
}

/// `TestCase::__elephc_fail()`, transcribed from the PHP form.
fn method_elephc_fail() -> MethodBuilder {
    method("__elephc_fail")
        .private()
        .param("message", TypeExpr::Str)
        .param("description", TypeExpr::Str)
        .returns(TypeExpr::Void)
        .body(vec![
            s_prop_assign(e_this(), "__elephc_assertions", e_binop(e_this_prop("__elephc_assertions"), BinOp::Add, e_int(1))),
            s_if(
                e_binop(e_var("message"), BinOp::StrictNotEq, e_str("")),
                vec![
                    s_throw(e_new("AssertionFailedError", vec![e_binop(e_binop(e_var("message"), BinOp::Concat, e_str("\n")), BinOp::Concat, e_var("description"))])),
                ],
                vec![],
                None,
            ),
            s_throw(e_new("AssertionFailedError", vec![e_var("description")])),
        ])
}

/// `TestCase::__elephc_export()`, transcribed from the PHP form.
fn method_elephc_export() -> MethodBuilder {
    method("__elephc_export")
        .private()
        .param("value", t_mixed())
        .returns(TypeExpr::Str)
        .body(vec![
            s_if(
                e_binop(e_var("value"), BinOp::StrictEq, e_null()),
                vec![
                    s_return(e_str("null")),
                ],
                vec![],
                None,
            ),
            s_if(
                e_call("is_bool", vec![e_var("value")]),
                vec![
                    s_return(e_ternary(e_var("value"), e_str("true"), e_str("false"))),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_call("is_int", vec![e_var("value")]), BinOp::Or, e_call("is_float", vec![e_var("value")])),
                vec![
                    s_return(e_cast(CastType::String, e_var("value"))),
                ],
                vec![],
                None,
            ),
            s_if(
                e_call("is_string", vec![e_var("value")]),
                vec![
                    s_return(e_binop(e_binop(e_str("'"), BinOp::Concat, e_var("value")), BinOp::Concat, e_str("'"))),
                ],
                vec![],
                None,
            ),
            s_if(
                e_call("is_array", vec![e_var("value")]),
                vec![
                    s_return(e_binop(e_binop(e_str("Array ("), BinOp::Concat, e_call("count", vec![e_var("value")])), BinOp::Concat, e_str(")"))),
                ],
                vec![],
                None,
            ),
            s_if(
                e_call("is_object", vec![e_var("value")]),
                vec![
                    s_return(e_binop(e_call("get_class", vec![e_var("value")]), BinOp::Concat, e_str(" Object"))),
                ],
                vec![],
                None,
            ),
            s_return(e_call("gettype", vec![e_var("value")])),
        ])
}
