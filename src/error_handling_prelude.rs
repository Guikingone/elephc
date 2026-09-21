//! Purpose:
//! PHP's error- and exception-handling surface, built as AST and shared by EVERY SAPI:
//! `error_reporting()`, the `set_error_handler()` / `get_error_handler()` /
//! `restore_error_handler()` stack, the `set_exception_handler()` /
//! `restore_exception_handler()` stack, `trigger_error()`, the `error_get_last()` /
//! `error_clear_last()` pair over the request's last-diagnostic record, and the two dispatch
//! helpers (`__elephc_diag_dispatch` / `__elephc_diag_render`) that give a user handler the
//! diagnostics COMPILED CODE raises — plus `error_log()`, which is the same family and had the
//! same `--web`-only home.
//!
//! - THE LAST-ERROR RECORD IS PER-REQUEST STATE, and `web_prelude`'s `bootstrap 43c` clears it
//!   at the head of every request beside the `43a` / `43b` resets. That reset is DEFENSIVE
//!   rather than load-bearing today, and the measurement is on `decl_stmt_bootstrap_43c`:
//!   every `--web` request is served by a forked handler child, so a static local already
//!   starts fresh in all three isolation modes and a 400-request soak passes with or without
//!   it. `__elephc_error_reporting_state` deliberately has no reset at all — php keeps the
//!   reporting MASK a process-wide setting — so it is NOT the precedent for this record; the
//!   shutdown and exception registries are.
//!
//! Called from:
//! - `crate::web_prelude::build::web_declarations`, which splices [`declarations`] into the
//!   `--web` surface where these declarations used to sit inline.
//! - `crate::pipeline::compile` through [`inject_if_used`], which gives a NON-`--web` build
//!   the same surface when the program mentions it.
//!
//! Key details:
//! - ONE implementation, two injection sites. These functions were `--web`-only until a plain
//!   CLI build turned out to have none of them (`Call to undefined function
//!   error_reporting()`), and PHP's error rule is not something two copies may each hold: the
//!   bodies live here and both preludes hand out the same `Stmt`s.
//! - Built as AST (`function(...).param(...).body(vec![...])`), never parsed from PHP source,
//!   which is the standing rule for every compiler-provided prelude outside tests.
//! - Pay-for-use off `--web`: [`inject_if_used`] injects nothing unless the program names one
//!   of [`SURFACE`], so a program that never mentions them does not grow. Under `--web` the
//!   whole prelude is injected unconditionally and pruned later, which is why
//!   [`inject_if_used`] returns early for a `--web` compile — the declarations are already
//!   there and a second copy would be a redeclaration.
//! - `__elephc_diag_render` has NO PHP caller: the generated runtime's `__rt_diag_warning`
//!   reaches it by symbol. Reachability cannot see that, so both injection sites force
//!   [`DIAG_DISPATCH_GROUP`] (see `pipeline::compile`).
//! - `register_shutdown_function()` IS here now, and the constraint that used to keep it out is
//!   worth stating because it is what the shape below answers. It was: "its registry only means
//!   something if something drains it, and the only drain that exists is the `--web` request
//!   wrapper's `finally`; a CLI copy would register callbacks that never run." That was right —
//!   and a `finally` is not a drain a CLI build may rely on, because `exit()` does not run
//!   `finally` blocks. Measured on php 8.5.10 AND on elephc, on the same source
//!   (`try { exit(3); } finally { echo "FINALLY"; }` prints nothing but the body in both, rc=3):
//!   a wrapper copied from `--web` would cover the normal end and miss the exit, which is the
//!   path a console application takes.
//!
//!   So the drain is not a `finally` at all. [`decl_fn_elephc_shutdown_run`] is a nullary `void`
//!   entry, and CODEGEN calls it by symbol at the process-exit sites it owns:
//!   `codegen::lower_inst::builtins::system::lower_exit` (every `exit()` / `die()`, `--web`
//!   included) and `codegen::frame::emit_main_epilogue` (the normal end of the top level). That
//!   covers php's normal-end and `exit()` paths in one mechanism with no control-flow rewrite.
//!
//!   WHAT IT STILL DOES NOT COVER: an uncaught exception. php runs shutdown functions after
//!   printing the fatal report; elephc's report is `__rt_report_uncaught_exception`, a SHARED
//!   RUNTIME symbol in a separate object that cannot name this program's functions, entered by
//!   a tail jump with no frame. Reaching the drain from there needs the `RuntimeFeatures`
//!   gating that `diag_user_handler` already demonstrates, and that is where the fix belongs.

use std::path::Path;

use crate::parser::ast::{
    BinOp, CastType, Expr, ExprKind, MagicConstant, Program, Stmt, StmtKind, TypeExpr,
};
use crate::prelude_prune::usage;
use crate::span::Span;
use crate::synthetic_class::{
    e_array, e_array_assoc, e_binop, e_bool, e_call, e_cast, e_const, e_index, e_int, e_null,
    e_str, e_ternary, e_var, function, internal_declarations, s_array_push, s_assign, s_expr,
    s_if, s_return, s_static, s_try, s_while, t_array, t_class, t_mixed, t_nullable,
};

/// Prelude inventory group holding the engine-diagnostic dispatch pair.
///
/// `__elephc_diag_render` has NO PHP caller: the generated runtime's `__rt_diag_warning`
/// reaches it, and declaration reachability cannot see a call that lives in assembly. Recorded
/// as its own group and forced by `pipeline::compile`, so a program whose only error-handler
/// use is `set_error_handler()` still has the dispatch rule compiled in.
pub(crate) const DIAG_DISPATCH_GROUP: &str = "diag_dispatch";

/// Prelude inventory group holding the surface itself, on the non-`--web` path.
///
/// A separate id from `"web"`, because the two injection sites are mutually exclusive and a
/// shared id would make "which prelude contributed this" unanswerable.
pub(crate) const GROUP: &str = "error_handling";

/// The PHP-visible names this prelude declares — the pay-for-use gate off `--web`.
///
/// The internal `__elephc_*` helpers are deliberately absent: nothing but these functions and
/// the generated runtime reaches them, so a program that names none of these cannot want them.
pub(crate) const SURFACE: &[&str] = &[
    "error_reporting",
    "set_error_handler",
    "get_error_handler",
    "restore_error_handler",
    "trigger_error",
    "error_get_last",
    "error_clear_last",
    "set_exception_handler",
    "restore_exception_handler",
    // `error_log` is the one name here that is NOT undefined off `--web`: the registry builtin
    // answers it. It is gated all the same, because the prelude declaration SHADOWS that builtin
    // and must therefore be pay-for-use — a program that never spells `error_log` keeps the
    // cheaper direct `__rt_error_log` call it has today.
    "error_log",
    "register_shutdown_function",
];

/// Prelude inventory group holding the nullary shutdown drain, on the non-`--web` path.
///
/// `__elephc_shutdown_run` has NO PHP caller off `--web`: `lower_exit` and `emit_main_epilogue`
/// reach it by symbol, and declaration reachability cannot see a call that lives in assembly.
/// Recorded as its own group and forced by `pipeline::compile` — but only when the program
/// SPELLS `register_shutdown_function`, so a program that merely names `error_reporting()`
/// still gets the whole registry pruned and pays for no drain call at its exits.
pub(crate) const SHUTDOWN_RUN_GROUP: &str = "shutdown_run";

/// `__FILE__` — the MAGIC-CONSTANT node, not a string literal.
///
/// Both injection sites run `magic_constants::substitute_file_constants` over the built
/// program, so this resolves against the entry file exactly as the PHP form's `__FILE__` did.
/// Spelling a literal here would instead bake in whatever path the BUILDER happened to know,
/// which is the compiler's own working directory rather than the script's.
fn e_magic_file() -> Expr {
    Expr::new(ExprKind::MagicConstant(MagicConstant::File), Span::dummy())
}

/// `__LINE__`, which is an INT LITERAL and not a `MagicConstant`.
///
/// There is no `MagicConstant::Line`: the parser lowers the token to `IntLiteral(span.line)` at
/// parse time (`parser/expr/prefix.rs`), so a built node has to carry the value directly. A
/// synthetic declaration is built on `Span::dummy()`, whose line is 0, so 0 is what the parser
/// would have produced for this node — the prelude has no source line of its own to report.
fn e_magic_line() -> Expr {
    e_int(0)
}

/// `__elephc_error_reporting_state` — transcribed from the PHP form.
///
/// The request-local error mask `error_reporting()` and `trigger_error()` share. The value lives
/// in a STATIC function local, which is what makes it survive across calls within one request —
/// and, because a worker is reused, across requests too. Nothing resets it per request today:
/// unlike the shutdown and exception registries below, PHP itself keeps the mask a process-wide
/// setting, so the transcription keeps it one as well.
fn decl_fn_elephc_error_reporting_state() -> Stmt {
    function("__elephc_error_reporting_state")
        .param_default("next", t_nullable(TypeExpr::Int), e_null())
        .param_default("replace", TypeExpr::Bool, e_bool(false))
        .returns(TypeExpr::Int)
        .body(vec![
            s_static("current", e_const("E_ALL")),
            s_assign("previous", e_var("current")),
            s_if(
                e_var("replace"),
                vec![
                    s_assign("current", e_cast(CastType::Int, e_var("next"))),
                ],
                vec![],
                None,
            ),
            s_return(e_var("previous")),
        ])
        .build()
}

/// `error_reporting` — transcribed from the PHP form.
fn decl_fn_error_reporting() -> Stmt {
    function("error_reporting")
        .param_default("error_level", t_nullable(TypeExpr::Int), e_null())
        .returns(TypeExpr::Int)
        .body(vec![
            s_if(
                e_binop(e_var("error_level"), BinOp::StrictEq, e_null()),
                vec![
                    s_return(e_call("__elephc_error_reporting_state", vec![])),
                ],
                vec![],
                None,
            ),
            s_return(e_call("__elephc_error_reporting_state", vec![e_var("error_level"), e_bool(true)])),
        ])
        .build()
}

/// `__elephc_error_handler_state` — transcribed from the PHP form.
///
/// One entry point for the whole user-error-handler stack, dispatched on `$operation`: 0 reads
/// the top handler, 1 pushes one and returns the previous, 2 reads the top handler's MASK, 3
/// pops. The two parallel STATIC arrays are the stack; `restore_error_handler()` popping both
/// together is what keeps a handler and its mask at the same depth.
///
/// EVERY READ IS AN EARLY `if`, NOT A TERNARY, and that is load-bearing rather than style.
/// `return $count === 0 ? null : $handlers[$count - 1];` compiles to a value of the WRONG TYPE:
/// the string `'my_handler'` comes back as integer `0`, so `is_callable()` on it is false and
/// invoking it aborts with "mixed value is not callable". That is why `set_error_handler()` had
/// never once reached a handler. The `if`-and-return form of the same expression is correct;
/// see the repro in the report accompanying this change. Restore the ternary only after that
/// miscompile is fixed AND a test invokes a handler.
fn decl_fn_elephc_error_handler_state() -> Stmt {
    function("__elephc_error_handler_state")
        .param_default("next", t_mixed(), e_null())
        .param_default("levels", TypeExpr::Int, e_const("E_ALL"))
        .param_default("operation", TypeExpr::Int, e_int(0))
        .returns(t_mixed())
        .body(vec![
            s_static("handlers", e_array(vec![])),
            s_static("masks", e_array(vec![])),
            s_assign("count", e_call("count", vec![e_var("handlers")])),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(0)),
                vec![
                    s_if(
                        e_binop(e_var("count"), BinOp::StrictEq, e_int(0)),
                        vec![
                            s_return(e_null()),
                        ],
                        vec![],
                        None,
                    ),
                    s_return(e_index(e_var("handlers"), e_binop(e_var("count"), BinOp::Sub, e_int(1)))),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(2)),
                vec![
                    s_if(
                        e_binop(e_var("count"), BinOp::StrictEq, e_int(0)),
                        vec![
                            s_return(e_const("E_ALL")),
                        ],
                        vec![],
                        None,
                    ),
                    s_return(e_index(e_var("masks"), e_binop(e_var("count"), BinOp::Sub, e_int(1)))),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(3)),
                vec![
                    s_if(
                        e_binop(e_var("count"), BinOp::StrictNotEq, e_int(0)),
                        vec![
                            s_expr(e_call("array_pop", vec![e_var("handlers")])),
                            s_expr(e_call("array_pop", vec![e_var("masks")])),
                        ],
                        vec![],
                        None,
                    ),
                    s_return(e_bool(true)),
                ],
                vec![],
                None,
            ),
            s_assign("previous", e_null()),
            s_if(
                e_binop(e_var("count"), BinOp::StrictNotEq, e_int(0)),
                vec![
                    s_assign("previous", e_index(e_var("handlers"), e_binop(e_var("count"), BinOp::Sub, e_int(1)))),
                ],
                vec![],
                None,
            ),
            s_array_push("handlers", e_var("next")),
            s_array_push("masks", e_var("levels")),
            s_return(e_var("previous")),
        ])
        .build()
}

/// `set_error_handler` — transcribed from the PHP form.
fn decl_fn_set_error_handler() -> Stmt {
    function("set_error_handler")
        .param("callback", t_mixed())
        .param_default("error_levels", TypeExpr::Int, e_const("E_ALL"))
        .returns(t_mixed())
        .body(vec![
            s_return(e_call("__elephc_error_handler_state", vec![e_var("callback"), e_var("error_levels"), e_int(1)])),
        ])
        .build()
}

/// `get_error_handler` — transcribed from the PHP form.
fn decl_fn_get_error_handler() -> Stmt {
    function("get_error_handler")
        .returns(t_mixed())
        .body(vec![
            s_return(e_call("__elephc_error_handler_state", vec![])),
        ])
        .build()
}

/// `restore_error_handler` — transcribed from the PHP form.
fn decl_fn_restore_error_handler() -> Stmt {
    function("restore_error_handler")
        .returns(TypeExpr::Bool)
        .body(vec![
            s_return(e_cast(CastType::Bool, e_call("__elephc_error_handler_state", vec![e_null(), e_const("E_ALL"), e_int(3)]))),
        ])
        .build()
}
/// `__elephc_exception_handler_state` — transcribed from the PHP form.
///
/// The exception-handler stack behind `set_exception_handler()` / `restore_exception_handler()`,
/// dispatched on `$operation`: 0 reads the top, 1 pushes, 2 pops, 3 clears for the next request
/// (see `bootstrap 43b`).
///
/// EVERY READ IS AN EARLY `if`, NOT A TERNARY, for the reason
/// [`decl_fn_elephc_error_handler_state`] spells out: the two reads here WERE ternaries, and
/// `set_exception_handler('h2')` answered `int(0)` where php answers `string(2) "h1"` —
/// measured on a CLI binary, and the same defect was live under `--web`. The `if`-and-return
/// form of the same expression is correct. Nothing had caught it because the only coverage of
/// this stack asserted truthiness, which `0` fails and `null` fails identically.
fn decl_fn_elephc_exception_handler_state() -> Stmt {
    function("__elephc_exception_handler_state")
        .param_default("next", t_mixed(), e_null())
        .param_default("operation", TypeExpr::Int, e_int(0))
        .returns(t_mixed())
        .body(vec![
            s_static("handlers", e_array(vec![])),
            s_assign("count", e_call("count", vec![e_var("handlers")])),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(0)),
                vec![
                    s_if(
                        e_binop(e_var("count"), BinOp::StrictEq, e_int(0)),
                        vec![
                            s_return(e_null()),
                        ],
                        vec![],
                        None,
                    ),
                    s_return(e_index(e_var("handlers"), e_binop(e_var("count"), BinOp::Sub, e_int(1)))),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(2)),
                vec![
                    s_if(
                        e_binop(e_var("count"), BinOp::StrictNotEq, e_int(0)),
                        vec![
                            s_expr(e_call("array_pop", vec![e_var("handlers")])),
                        ],
                        vec![],
                        None,
                    ),
                    s_return(e_bool(true)),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(3)),
                vec![
                    s_assign("handlers", e_array(vec![])),
                    s_return(e_null()),
                ],
                vec![],
                None,
            ),
            s_assign("previous", e_null()),
            s_if(
                e_binop(e_var("count"), BinOp::StrictNotEq, e_int(0)),
                vec![
                    s_assign("previous", e_index(e_var("handlers"), e_binop(e_var("count"), BinOp::Sub, e_int(1)))),
                ],
                vec![],
                None,
            ),
            s_array_push("handlers", e_var("next")),
            s_return(e_var("previous")),
        ])
        .build()
}

/// `set_exception_handler` — transcribed from the PHP form.
fn decl_fn_set_exception_handler() -> Stmt {
    function("set_exception_handler")
        .param("callback", t_mixed())
        .returns(t_mixed())
        .body(vec![
            s_return(e_call("__elephc_exception_handler_state", vec![e_var("callback"), e_int(1)])),
        ])
        .build()
}

/// `restore_exception_handler` — transcribed from the PHP form.
fn decl_fn_restore_exception_handler() -> Stmt {
    function("restore_exception_handler")
        .returns(TypeExpr::Bool)
        .body(vec![
            s_return(e_cast(CastType::Bool, e_call("__elephc_exception_handler_state", vec![e_null(), e_int(2)]))),
        ])
        .build()
}

/// `__elephc_last_error_state` — the record `error_get_last()` reads, in ONE place.
///
/// PHP keeps a single "most recent diagnostic" slot per request and publishes it as a
/// four-key array in EXACTLY this order — `type`, `message`, `file`, `line` — or `null` when
/// nothing has been recorded. `$operation` selects the action, the same shape
/// [`decl_fn_elephc_error_handler_state`] uses: 0 reads, 1 records, 2 clears.
///
/// THE RECORD IS FIVE SCALAR STATICS, NOT ONE STATIC HOLDING NULL-OR-ARRAY, and that is
/// deliberate rather than clumsy. A static whose value alternates between `null` and an array
/// is a `Mixed` slot, and this tree has a documented history of miscompiles on exactly that
/// shape — see the warning on [`decl_fn_elephc_error_handler_state`], where reading a `Mixed`
/// static through a ternary returned integer `0` for the string it held. Five monomorphic
/// statics plus a `bool` presence flag give the array builder concrete types at every read and
/// keep the "nothing recorded yet" state out of the payload entirely.
///
/// WHAT php RECORDS, measured on php 8.5.10 — each of these is a test in
/// `tests/error_handling_surface_tests.rs`:
///
/// ```text
/// php -r 'var_dump(error_get_last());'                        → NULL
/// php -r '@$x = $u; var_dump(error_get_last());'               → array, type 2
/// php -r 'error_reporting(0); $x = $u; var_dump(...);'         → array, type 2
/// php -r 'set_error_handler(fn(...) => true); $x = $u; ...'    → NULL
/// php -r 'set_error_handler(fn(...) => false); $x = $u; ...'   → array, type 2
/// php -r 'trigger_error("boom"); ...'                          → array, type 1024
/// php -r 'trigger_error("bang", E_USER_WARNING); ...'          → array, type 512
/// php -r 'trigger_error("x"); error_clear_last(); ...'         → NULL
/// ```
///
/// The two that decide the design are the third and the fourth. SUPPRESSION DOES NOT STOP THE
/// RECORD: `@` and `error_reporting(0)` silence the DISPLAY and nothing else, which is why the
/// write below sits on the mask-excluded path too. A HANDLER THAT RETURNS TRUE DOES STOP IT,
/// and it does not merely record `null` — it leaves the PREVIOUS record standing, measured:
///
/// ```text
/// php -r '$a = $u1; set_error_handler(fn(...) => true); $b = $u2; var_dump(error_get_last());'
/// → the $u1 record, not the $u2 one and not NULL
/// ```
///
/// That is php-src's structure showing through: the recording lives in `php_error_cb`, the
/// INTERNAL handler, which a user handler returning anything but `false` pre-empts entirely.
/// The mask is consulted inside `php_error_cb` for the display only, after the record is
/// written. [`decl_fn_elephc_diag_dispatch`] reproduces that by writing at every outcome except
/// the one where the handler took it.
fn decl_fn_elephc_last_error_state() -> Stmt {
    function("__elephc_last_error_state")
        .param_default("error_level", TypeExpr::Int, e_int(0))
        .param_default("message", TypeExpr::Str, e_str(""))
        .param_default("file", TypeExpr::Str, e_str(""))
        .param_default("line", TypeExpr::Int, e_int(0))
        .param_default("operation", TypeExpr::Int, e_int(0))
        .returns(t_mixed())
        .body(vec![
            s_static("recorded", e_bool(false)),
            s_static("last_type", e_int(0)),
            s_static("last_message", e_str("")),
            s_static("last_file", e_str("")),
            s_static("last_line", e_int(0)),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(1)),
                vec![
                    s_assign("recorded", e_bool(true)),
                    s_assign("last_type", e_var("error_level")),
                    s_assign("last_message", e_var("message")),
                    s_assign("last_file", e_var("file")),
                    s_assign("last_line", e_var("line")),
                    s_return(e_null()),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(2)),
                vec![
                    s_assign("recorded", e_bool(false)),
                    s_assign("last_type", e_int(0)),
                    s_assign("last_message", e_str("")),
                    s_assign("last_file", e_str("")),
                    s_assign("last_line", e_int(0)),
                    s_return(e_null()),
                ],
                vec![],
                None,
            ),
            // AN EARLY `if`, NEVER A TERNARY — the same constraint
            // `decl_fn_elephc_error_handler_state` documents at length. A `Mixed`-typed
            // ternary over a static is the exact shape that miscompiled there.
            s_if(
                e_binop(e_var("recorded"), BinOp::StrictEq, e_bool(false)),
                vec![
                    s_return(e_null()),
                ],
                vec![],
                None,
            ),
            s_return(e_array_assoc(vec![
                (e_str("type"), e_var("last_type")),
                (e_str("message"), e_var("last_message")),
                (e_str("file"), e_var("last_file")),
                (e_str("line"), e_var("last_line")),
            ])),
        ])
        .build()
}

/// `error_get_last` — transcribed from the PHP form.
///
/// php's signature is `error_get_last(): ?array`, and the nullable array return is what
/// Symfony's `ErrorHandler::handleFatalError` relies on: it does
/// `if ($error && $error['type'] &= E_PARSE|E_ERROR|E_CORE_ERROR|E_COMPILE_ERROR)`, so both
/// `null` and an ordinary `E_WARNING` record answer false there and only a genuine fatal
/// promotes. Returning `null` unconditionally would satisfy that ONE caller while being wrong
/// for every other — `DefaultMarshaller` reads `error_get_last()['message']` — which is why
/// this is a real record and not a stub.
fn decl_fn_error_get_last() -> Stmt {
    function("error_get_last")
        .returns(t_nullable(t_array()))
        .body(vec![
            s_return(e_call("__elephc_last_error_state", vec![])),
        ])
        .build()
}

/// `error_clear_last` — transcribed from the PHP form.
///
/// php's `error_clear_last(): void`. Nothing in the `--web` corpus calls it — the Symfony
/// vendor tree has `error_get_last` in five places and `error_clear_last` in none — but it is
/// half of a two-function surface in php and a program that clears cannot be made to work by
/// adding only the reader. It costs one declaration behind the same pay-for-use gate.
fn decl_fn_error_clear_last() -> Stmt {
    function("error_clear_last")
        .returns(TypeExpr::Void)
        .body(vec![
            s_expr(e_call(
                "__elephc_last_error_state",
                vec![e_int(0), e_str(""), e_str(""), e_int(0), e_int(2)],
            )),
        ])
        .build()
}

/// The `__elephc_last_error_state(..., 1)` write, built once for the four sites that share it.
///
/// Every caller is inside [`decl_fn_elephc_diag_dispatch`] and reads its four parameters by
/// name, so the arguments are identical at each site and the expression is worth building in
/// one place rather than transcribing four times.
fn e_record_last_error() -> Expr {
    e_call(
        "__elephc_last_error_state",
        vec![
            e_var("error_level"),
            e_var("message"),
            e_var("file"),
            e_var("line"),
            e_int(1),
        ],
    )
}

/// `__elephc_diag_dispatch` — PHP's diagnostic-dispatch rule, in ONE place.
///
/// Every diagnostic — one a user raised with `trigger_error()` AND one the engine raised from
/// compiled code through `__rt_diag_warning` — has to answer the same three questions in the
/// same order, and before this function existed only `trigger_error()` did: the reporting mask,
/// then the installed handler and ITS level mask, then the default rendering. Engine-raised
/// diagnostics bypassed all three and wrote straight to fd 2, so a program that installed a
/// handler to log or swallow them saw none of them and got the message printed anyway.
///
/// The return value is a four-way OUTCOME rather than a bool because the caller has to know
/// both whether the display is owed AND what the handler said:
///
/// - `0` — nobody took it; the caller renders the default display.
/// - `1` — the reporting mask excludes this level; nothing is displayed and no handler ran.
/// - `2` — the handler ran and returned true.
/// - `3` — the handler ran and returned false.
///
/// THE MASK GATE COMES FIRST, which is what `error_reporting(0)` has to mean here; note php-src
/// calls the handler even for a masked level and leaves the decision to it. That ordering is
/// what `trigger_error()` already did before this lift and is preserved deliberately rather
/// than quietly changed under a bug fix.
///
/// THE ONE LAST-ERROR DIVERGENCE THAT ORDERING COSTS, stated so the next reader does not have
/// to find it twice. When a level is masked AND a handler is installed that would have taken
/// it, php calls the handler, the handler returns true, and NOTHING is recorded:
///
/// ```text
/// php -r 'error_reporting(0); set_error_handler(function($n,$s,$f,$l){ return true; });
///         $x = $u; var_dump(error_get_last());'
/// HANDLER CALLED n=2
/// NULL
/// ```
///
/// Here the mask gate returns first, the handler is never consulted, and the record IS
/// written. Both halves of the divergence — the uncalled handler and the extra record — are
/// the same pre-existing ordering choice, not something the last-error record introduced;
/// fixing it means swapping the gate order, which is a separate change with its own
/// `trigger_error()` blast radius. Every other combination matches php and is tested.
///
/// `$__elephc_diag_inside` is the reentrancy guard: php does not hand the handler a diagnostic
/// raised inside the handler, and the `finally` releases the guard even when the handler throws.
fn decl_fn_elephc_diag_dispatch() -> Stmt {
    function("__elephc_diag_dispatch")
        .param("error_level", TypeExpr::Int)
        .param("message", TypeExpr::Str)
        .param("file", TypeExpr::Str)
        .param("line", TypeExpr::Int)
        .returns(TypeExpr::Int)
        .body(vec![
            s_static("__elephc_diag_inside", e_bool(false)),
            s_if(
                e_binop(e_binop(e_var("error_level"), BinOp::BitAnd, e_call("error_reporting", vec![])), BinOp::StrictEq, e_int(0)),
                vec![
                    // RECORDED EVEN THOUGH NOTHING IS DISPLAYED. `@$undefined` and
                    // `error_reporting(0)` both land here, and php keeps the record for both:
                    // `php -r '@$x = $u; var_dump(error_get_last());'` prints the E_WARNING
                    // array, not NULL. Suppression is a display rule in php-src — the write
                    // happens in `php_error_cb` before the mask is consulted.
                    s_expr(e_record_last_error()),
                    s_return(e_int(1)),
                ],
                vec![],
                None,
            ),
            s_if(
                e_var("__elephc_diag_inside"),
                vec![
                    // A diagnostic raised INSIDE a handler is recorded, measured:
                    // `php -r 'set_error_handler(function($n,$s,$f,$l){ $i = $nothere2;
                    // return true; }); $x = $u; var_dump(error_get_last());'` reports
                    // `Undefined variable $nothere2`. php does not re-enter the handler, so
                    // the internal one runs and writes.
                    s_expr(e_record_last_error()),
                    s_return(e_int(0)),
                ],
                vec![],
                None,
            ),
            s_assign("__elephc_diag_handler", e_call("__elephc_error_handler_state", vec![])),
            s_if(
                e_binop(e_binop(e_var("__elephc_diag_handler"), BinOp::StrictNotEq, e_null()), BinOp::And, e_binop(e_binop(e_var("error_level"), BinOp::BitAnd, e_cast(CastType::Int, e_call("__elephc_error_handler_state", vec![e_null(), e_const("E_ALL"), e_int(2)]))), BinOp::StrictNotEq, e_int(0))),
                vec![
                    s_assign("__elephc_diag_inside", e_bool(true)),
                    s_assign("__elephc_diag_taken", e_bool(false)),
                    s_try(
                        vec![
                            // `call_user_func_array`, NOT `$handler(...)`: the value comes out of
                            // a static array, so closed-world callable analysis has no candidate
                            // set for a direct invoke and lowers it to a "mixed value is not
                            // callable" fatal. That is what `trigger_error()` did before this
                            // lift — its handler call killed the worker on the first diagnostic,
                            // which no test caught because none ever INVOKED a handler.
                            s_assign("__elephc_diag_taken", e_cast(CastType::Bool, e_call("call_user_func_array", vec![e_var("__elephc_diag_handler"), e_array(vec![e_var("error_level"), e_var("message"), e_var("file"), e_var("line")])]))),
                        ],
                        vec![],
                        Some(vec![
                            s_assign("__elephc_diag_inside", e_bool(false)),
                        ]),
                    ),
                    // THE ONE OUTCOME THAT DOES NOT RECORD. A handler returning true takes
                    // the diagnostic and php's internal handler never runs, so nothing is
                    // written — and, measured, the PREVIOUS record is left standing rather
                    // than cleared:
                    // `php -r '$a = $u1; set_error_handler(fn(...) => true); $b = $u2;
                    //          var_dump(error_get_last());'` reports the `$u1` warning.
                    // Writing here and undoing on `true` would therefore be wrong twice over;
                    // not writing at all is the whole rule.
                    s_if(
                        e_binop(e_var("__elephc_diag_taken"), BinOp::StrictEq, e_bool(false)),
                        vec![
                            s_expr(e_record_last_error()),
                        ],
                        vec![],
                        None,
                    ),
                    s_return(e_ternary(e_var("__elephc_diag_taken"), e_int(2), e_int(3))),
                ],
                vec![],
                None,
            ),
            // No handler, or one whose level mask excludes this diagnostic — php's internal
            // handler runs and records. Measured for the mask case:
            // `php -r 'set_error_handler(fn(...) => true, E_NOTICE); $x = $u;
            //          var_dump(error_get_last());'` records the E_WARNING.
            s_expr(e_record_last_error()),
            s_return(e_int(0)),
        ])
        .build()
}

/// `__elephc_diag_render` — the runtime's entry into [`decl_fn_elephc_diag_dispatch`].
///
/// `__rt_diag_warning` accumulates one engine diagnostic as RENDERED BYTES: the severity word,
/// `": "`, the message, and a trailing newline, because that is all the raise sites ever had.
/// A user handler is owed the message WITHOUT the severity word and WITHOUT the newline, plus
/// the numeric level, so this undoes the rendering before dispatching and reports back whether
/// the display is still owed:
///
/// - returns `0` — the caller must write the raw bytes it already holds (unchanged output).
/// - returns `1` — nothing more to write: either a handler took the diagnostic, the reporting
///   mask excluded it, or this function rendered php's full ` in FILE on line N` form itself.
///
/// The enriched rendering happens HERE and not in the runtime because the runtime holds bytes,
/// not a formatter; it is used only when the raise site supplied a location, so a diagnostic
/// raised without one still produces byte-identical output to before.
fn decl_fn_elephc_diag_render() -> Stmt {
    function("__elephc_diag_render")
        .param("raw", TypeExpr::Str)
        .param("file", TypeExpr::Str)
        .param("line", TypeExpr::Int)
        .returns(TypeExpr::Int)
        .body(vec![
            s_assign("__elephc_diag_text", e_var("raw")),
            s_if(
                e_call("str_ends_with", vec![e_var("__elephc_diag_text"), e_str("\n")]),
                vec![
                    s_assign("__elephc_diag_text", e_call("substr", vec![e_var("__elephc_diag_text"), e_int(0), e_binop(e_call("strlen", vec![e_var("__elephc_diag_text")]), BinOp::Sub, e_int(1))])),
                ],
                vec![],
                None,
            ),
            s_assign("__elephc_diag_level", e_const("E_WARNING")),
            s_assign("__elephc_diag_label", e_str("Warning")),
            s_if(
                e_call("str_starts_with", vec![e_var("__elephc_diag_text"), e_str("Warning: ")]),
                vec![
                    s_assign("__elephc_diag_text", e_call("substr", vec![e_var("__elephc_diag_text"), e_int(9)])),
                ],
                vec![
                (e_call("str_starts_with", vec![e_var("__elephc_diag_text"), e_str("Notice: ")]), vec![
                    s_assign("__elephc_diag_level", e_const("E_NOTICE")),
                    s_assign("__elephc_diag_label", e_str("Notice")),
                    s_assign("__elephc_diag_text", e_call("substr", vec![e_var("__elephc_diag_text"), e_int(8)])),
                ]),
                (e_call("str_starts_with", vec![e_var("__elephc_diag_text"), e_str("Deprecated: ")]), vec![
                    s_assign("__elephc_diag_level", e_const("E_DEPRECATED")),
                    s_assign("__elephc_diag_label", e_str("Deprecated")),
                    s_assign("__elephc_diag_text", e_call("substr", vec![e_var("__elephc_diag_text"), e_int(12)])),
                ]),
            ],
                None,
            ),
            s_assign("__elephc_diag_outcome", e_call("__elephc_diag_dispatch", vec![e_var("__elephc_diag_level"), e_var("__elephc_diag_text"), e_var("file"), e_var("line")])),
            s_if(
                e_binop(e_binop(e_var("__elephc_diag_outcome"), BinOp::StrictEq, e_int(1)), BinOp::Or, e_binop(e_var("__elephc_diag_outcome"), BinOp::StrictEq, e_int(2))),
                vec![
                    s_return(e_int(1)),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_binop(e_var("line"), BinOp::StrictEq, e_int(0)), BinOp::Or, e_binop(e_var("file"), BinOp::StrictEq, e_str(""))),
                vec![
                    s_return(e_int(0)),
                ],
                vec![],
                None,
            ),
            s_expr(e_call("fwrite", vec![e_const("STDERR"), e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_binop(e_var("__elephc_diag_label"), BinOp::Concat, e_str(": ")), BinOp::Concat, e_var("__elephc_diag_text")), BinOp::Concat, e_str(" in ")), BinOp::Concat, e_var("file")), BinOp::Concat, e_str(" on line ")), BinOp::Concat, e_cast(CastType::String, e_var("line"))), BinOp::Concat, e_str("\n"))])),
            s_return(e_int(1)),
        ])
        .build()
}

/// `trigger_error` — transcribed from the PHP form.
///
/// Web-SAPI user-error dispatch. The rule itself now lives in
/// [`decl_fn_elephc_diag_dispatch`], which engine-raised diagnostics reach too; this function
/// keeps only its own default rendering and its own return value, both unchanged: the handler's
/// boolean is what `trigger_error()` returns when a handler took the error, and a level the mask
/// excludes still returns true with nothing rendered.
fn decl_fn_trigger_error() -> Stmt {
    function("trigger_error")
        .param("message", TypeExpr::Str)
        .param_default("error_level", TypeExpr::Int, e_const("E_USER_NOTICE"))
        .returns(TypeExpr::Bool)
        .body(vec![
            s_assign("__elephc_te_outcome", e_call("__elephc_diag_dispatch", vec![e_var("error_level"), e_var("message"), e_magic_file(), e_magic_line()])),
            s_if(
                e_binop(e_binop(e_var("__elephc_te_outcome"), BinOp::StrictEq, e_int(1)), BinOp::Or, e_binop(e_var("__elephc_te_outcome"), BinOp::StrictEq, e_int(2))),
                vec![
                    s_return(e_bool(true)),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("__elephc_te_outcome"), BinOp::StrictEq, e_int(3)),
                vec![
                    s_return(e_bool(false)),
                ],
                vec![],
                None,
            ),
            s_assign("__elephc_te_prefix", e_str("Notice")),
            s_if(
                e_binop(e_var("error_level"), BinOp::StrictEq, e_const("E_USER_ERROR")),
                vec![
                    s_assign("__elephc_te_prefix", e_str("Fatal error")),
                ],
                vec![
                (e_binop(e_binop(e_var("error_level"), BinOp::StrictEq, e_const("E_USER_WARNING")), BinOp::Or, e_binop(e_var("error_level"), BinOp::StrictEq, e_const("E_WARNING"))), vec![
                    s_assign("__elephc_te_prefix", e_str("Warning")),
                ]),
                (e_binop(e_binop(e_var("error_level"), BinOp::StrictEq, e_const("E_USER_DEPRECATED")), BinOp::Or, e_binop(e_var("error_level"), BinOp::StrictEq, e_const("E_DEPRECATED"))), vec![
                    s_assign("__elephc_te_prefix", e_str("Deprecated")),
                ]),
            ],
                None,
            ),
            s_expr(e_call("fwrite", vec![e_const("STDERR"), e_binop(e_binop(e_binop(e_var("__elephc_te_prefix"), BinOp::Concat, e_str(": ")), BinOp::Concat, e_var("message")), BinOp::Concat, e_str("\n"))])),
            s_return(e_bool(true)),
        ])
        .build()
}

/// `error_log` — MOVED VERBATIM out of `web_prelude::build`, with one node changed (below).
///
/// PHP declares `error_log()` in every SAPI and elephc did too — but off `--web` the answer was
/// the REGISTRY BUILTIN (`src/builtins/system/error_log.rs` + `__rt_error_log`), which writes the
/// message to stderr and ignores `$message_type` and `$destination` entirely. So
/// `error_log($m, 3, $file)` on a CLI binary logged to the console and returned `true`, and
/// nothing said so. This declaration is `SourceMode::Internal`, which is what lets it shadow the
/// registry builtin (see `types::checker::driver::functions::collect_function_decls`) — the same
/// shadowing `--web` already relied on, now available to both SAPIs.
///
/// THE ONE CHANGED NODE is the `message_type === 0` newline. The `--web` form appended `"\n"`
/// only when the message did not already end in one, which is NOT what php does: php appends
/// unconditionally (`php -r 'error_log("b\n");'` writes `b\n\n` — verified on 8.5.10, and
/// `__rt_error_log` matches it). Transcribing that branch as it stood would have made the moved
/// declaration a REGRESSION on CLI, losing a newline the registry builtin gets right, so the
/// condition is gone and the concat is unconditional. This also fixes the same divergence under
/// `--web`.
///
/// The `message_type === 1` (mail) message lost its "under --web" wording for the same reason:
/// the branch is now reachable from a CLI binary, where that sentence would be a lie. Mail
/// delivery is still unimplemented in both SAPIs.
fn decl_fn_error_log() -> Stmt {
    function("error_log")
        .param("message", TypeExpr::Str)
        .param_default("message_type", TypeExpr::Int, e_int(0))
        .param_default("destination", t_nullable(TypeExpr::Str), e_null())
        .param_default("additional_headers", t_nullable(TypeExpr::Str), e_null())
        .returns(TypeExpr::Bool)
        .body(vec![
            s_if(
                e_binop(e_var("message_type"), BinOp::StrictEq, e_int(3)),
                vec![
                    s_if(
                        e_binop(e_var("destination"), BinOp::StrictEq, e_null()),
                        vec![
                            s_return(e_bool(false)),
                        ],
                        vec![],
                        None,
                    ),
                    s_assign("__elephc_el_fh", e_call("fopen", vec![e_cast(CastType::String, e_var("destination")), e_str("a")])),
                    s_if(
                        e_binop(e_var("__elephc_el_fh"), BinOp::StrictEq, e_bool(false)),
                        vec![
                            s_return(e_bool(false)),
                        ],
                        vec![],
                        None,
                    ),
                    s_expr(e_call("fwrite", vec![e_var("__elephc_el_fh"), e_var("message")])),
                    s_expr(e_call("fclose", vec![e_var("__elephc_el_fh")])),
                    s_return(e_bool(true)),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("message_type"), BinOp::StrictEq, e_int(0)),
                vec![
                    s_expr(e_call("fwrite", vec![e_const("STDERR"), e_binop(e_var("message"), BinOp::Concat, e_str("\n"))])),
                    s_return(e_bool(true)),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("message_type"), BinOp::StrictEq, e_int(1)),
                vec![
                    s_expr(e_call("fwrite", vec![e_const("STDERR"), e_binop(e_binop(e_binop(e_binop(e_binop(e_str("error_log(): mail delivery (type 1) is not supported by elephc"), BinOp::Concat, e_str(" [to=")), BinOp::Concat, e_cast(CastType::String, e_var("destination"))), BinOp::Concat, e_str(", headers=")), BinOp::Concat, e_cast(CastType::String, e_var("additional_headers"))), BinOp::Concat, e_str("]\n"))])),
                ],
                vec![],
                None,
            ),
            s_return(e_bool(false)),
        ])
        .build()
}

/// `__elephc_shutdown_function_state` — transcribed from the PHP form.
///
/// The shutdown callback registry, dispatched on `$operation`: 1 registers, 2 drains
/// ([`decl_fn_elephc_shutdown_run`] is the only caller of mode 2 now), 3 clears. The reset is
/// not optional housekeeping — the registry is a STATIC function local, so without
/// `bootstrap 43a` a reused `--web` worker would run the previous request's callbacks again.
///
/// The drain is an ordinary `array_shift()`-until-empty loop. It was written with a CURSOR for a
/// while, because `array_shift()` on a `static` array returned `NULL` for anything an earlier
/// call had pushed — and the note left here described that as specific to a static array whose
/// ELEMENTS ARE ARRAYS, with flat scalars reported as correct. THAT WAS WRONG: the element type
/// never mattered. `array_shift()` on a `static` array of plain strings returned `NULL` just the
/// same, and once the array outgrew its first allocation the static lost its contents entirely,
/// appends included. Both causes are fixed (an empty-array `static` is typed `array<mixed>`
/// rather than `array<never>`, and `ReceiverPlace` now knows `Op::LoadStaticLocal`), and
/// `tests/static_local_array_tests.rs` holds the repro with its controls.
///
/// This loop keeps php's semantics exactly. Registration order is the array's own order.
/// `register_shutdown_function()` called from INSIDE a shutdown callback appends to the same
/// registry, and the loop re-reads `count()` each turn, so it reaches it — which is what php
/// does: `php -r 'register_shutdown_function(function(){ echo "outer\n";
/// register_shutdown_function(function(){ echo "inner\n"; }); });'` prints both. And a second
/// drain is a no-op because the registry is empty, which is what the exit path relies on.
///
/// `$draining` is the RE-ENTRANCY LATCH, and it is the one thing the `--web` form did not need.
/// A shutdown callback may call `exit()`, and php then terminates at once WITHOUT running the
/// callbacks still queued behind it — measured:
///
/// ```text
/// php -r 'register_shutdown_function(function(){ echo "one\n"; exit(7); });
///         register_shutdown_function(function(){ echo "two\n"; }); echo "body\n"; exit(3);'
/// body
/// one
/// rc=7
/// ```
///
/// `two` never prints and the callback's status wins. That `exit()` re-enters this helper
/// through `lower_exit`'s drain call — in BOTH SAPIs, since that arm is shared — and without the
/// latch the loop would resume and print `two`. The latch is cleared by operation 3 as well as
/// by a completed drain, because a reused `--web` worker resets the registry between requests
/// and must reset this with it.
fn decl_fn_elephc_shutdown_function_state() -> Stmt {
    function("__elephc_shutdown_function_state")
        .param_default("callback", t_mixed(), e_null())
        .param_default("args", t_mixed(), e_array(vec![]))
        .param_default("operation", TypeExpr::Int, e_int(0))
        .returns(t_mixed())
        .body(vec![
            s_static("callbacks", e_array(vec![])),
            s_static("draining", e_bool(false)),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(1)),
                vec![
                    s_array_push("callbacks", e_array(vec![e_var("callback"), e_var("args")])),
                    s_return(e_null()),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(2)),
                vec![
                    s_if(
                        e_var("draining"),
                        vec![
                            s_return(e_null()),
                        ],
                        vec![],
                        None,
                    ),
                    s_assign("draining", e_bool(true)),
                    s_while(e_binop(e_call("count", vec![e_var("callbacks")]), BinOp::Gt, e_int(0)), vec![
                        s_assign("__elephc_shutdown_entry", e_cast(CastType::Array, e_call("array_shift", vec![e_var("callbacks")]))),
                        s_expr(e_call("call_user_func_array", vec![e_index(e_var("__elephc_shutdown_entry"), e_int(0)), e_index(e_var("__elephc_shutdown_entry"), e_int(1))])),
                    ]),
                    s_assign("draining", e_bool(false)),
                    s_return(e_null()),
                ],
                vec![],
                None,
            ),
            s_if(
                e_binop(e_var("operation"), BinOp::StrictEq, e_int(3)),
                vec![
                    s_assign("callbacks", e_array(vec![])),
                    s_assign("draining", e_bool(false)),
                ],
                vec![],
                None,
            ),
            s_return(e_null()),
        ])
        .build()
}

/// `__elephc_shutdown_run` — the zero-argument drain entry, and the ONLY one codegen calls.
///
/// It exists purely so the drain has a symbol that can be reached from emitted assembly with no
/// argument marshalling. `__elephc_shutdown_function_state(null, [], 2)` would mean building a
/// Mixed null and an empty PHP array at every process-exit site; a nullary `void` function is a
/// bare `bl` / `call`, which is what `lower_exit` and `emit_main_epilogue` emit.
///
/// It has NO PHP caller off `--web`, so declaration reachability cannot see it and would delete
/// it — the same shape as `__elephc_diag_render`, and the reason [`SHUTDOWN_RUN_GROUP`] is
/// forced by `pipeline::compile` whenever the program asks for this surface. Under `--web` the
/// request wrapper's `finally` calls it, so it is reachable there by ordinary means.
fn decl_fn_elephc_shutdown_run() -> Stmt {
    function(crate::names::SHUTDOWN_RUN_FUNCTION)
        .returns(TypeExpr::Void)
        .body(vec![s_expr(e_call(
            "__elephc_shutdown_function_state",
            vec![e_null(), e_array(vec![]), e_int(2)],
        ))])
        .build()
}

/// `register_shutdown_function` — transcribed from the PHP form.
fn decl_fn_register_shutdown_function() -> Stmt {
    function("register_shutdown_function")
        .param("callback", t_class("callable"))
        .variadic("args", Some(t_mixed()))
        .returns(TypeExpr::Void)
        .body(vec![
            s_expr(e_call("__elephc_shutdown_function_state", vec![e_var("callback"), e_var("args"), e_int(1)])),
        ])
        .build()
}

/// The whole error/exception surface, in the order the `--web` prelude's PHP source had.
///
/// Declarations are hoisted, so the order is documentation rather than semantics — but it is
/// the order `web_declarations` shipped, and keeping it makes the two surfaces diffable.
pub(crate) fn declarations() -> Program {
    internal_declarations(|| {
        vec![
            decl_fn_elephc_error_reporting_state(),
            decl_fn_error_reporting(),
            decl_fn_elephc_error_handler_state(),
            decl_fn_set_error_handler(),
            decl_fn_get_error_handler(),
            decl_fn_restore_error_handler(),
            decl_fn_elephc_exception_handler_state(),
            decl_fn_set_exception_handler(),
            decl_fn_restore_exception_handler(),
            decl_fn_elephc_last_error_state(),
            decl_fn_error_get_last(),
            decl_fn_error_clear_last(),
            decl_fn_elephc_diag_dispatch(),
            decl_fn_elephc_diag_render(),
            decl_fn_trigger_error(),
            decl_fn_error_log(),
            decl_fn_elephc_shutdown_function_state(),
            decl_fn_elephc_shutdown_run(),
            decl_fn_register_shutdown_function(),
        ]
    })
}

/// The function names [`declarations`] contributes, normalized.
///
/// Derived from the declarations themselves rather than listed a second time, so a helper
/// added above cannot fall out of the shadowing filter by being forgotten here.
fn declared_function_names() -> std::collections::HashSet<String> {
    declarations()
        .iter()
        .filter_map(|stmt| match &stmt.kind {
            StmtKind::FunctionDecl { name, .. } => Some(crate::names::php_symbol_key(name)),
            _ => None,
        })
        .collect()
}

/// Removes from the PROGRAM every function declaration whose name this prelude owns, so the
/// built-in wins — which is what php does and what elephc already does for a registry builtin.
///
/// THE SHAPE THIS EXISTS FOR is the polyfill guard, which is ordinary library code:
///
/// ```php
/// if (!function_exists('trigger_error')) { function trigger_error(…) { … } }
/// ```
///
/// php never takes that branch, because `trigger_error` is built in. elephc emits the guarded
/// body as a symbol anyway — the registration is conditional, the EMISSION is not — so a
/// prelude that declares the same name collides at assembly time:
/// `error: symbol '_fn_trigger_u_error' is already defined`, with no source position. The same
/// source failed a `--web` LINK the same way before this filter existed, so the defect is the
/// prelude's, not the CLI injection's; fixing it here fixes both.
///
/// DROPPING THE PRELUDE'S COPY INSTEAD DOES NOT WORK, and the reason is worth recording. Left
/// to the program's own declaration, the guard evaluates `function_exists('trigger_error')`,
/// which is TRUE (the name is in the closed world), so the branch never runs, so the
/// declaration is never registered, and the call one line later dies with `Call to undefined
/// function trigger_error()`. Measured. The built-in has to win, and for it to win the
/// shadowing declaration has to go.
///
/// A program that declares one of these names UNCONDITIONALLY is rejected outright by php
/// (`Cannot redeclare trigger_error()`), so nothing correct is lost by treating it the same
/// way. Function BODIES are not walked: a declaration nested inside one is a different
/// declaration and is left alone.
pub(crate) fn without_shadowing_declarations(program: Program) -> Program {
    without_shadowing_names(program, &declared_function_names())
}

/// The same filter against an EXPLICIT owned set, for a caller that injects only part of
/// [`declarations`].
///
/// The two have to agree or the program loses a declaration nothing replaces: off `--web` the
/// shutdown trio is dropped for a program that never spells `register_shutdown_function`, and a
/// filter keyed on the FULL set would still strip that program's own `function
/// register_shutdown_function()` — leaving an undefined function where there had been a working
/// one. Passing the set that was actually injected is what keeps the two sides one decision.
fn without_shadowing_names(
    program: Program,
    owned: &std::collections::HashSet<String>,
) -> Program {
    fn filter(body: Vec<Stmt>, owned: &std::collections::HashSet<String>) -> Vec<Stmt> {
        body.into_iter()
            .filter(|stmt| match &stmt.kind {
                StmtKind::FunctionDecl { name, .. } => {
                    !owned.contains(&crate::names::php_symbol_key(name))
                }
                _ => true,
            })
            .map(|mut stmt| {
                stmt.kind = match stmt.kind {
                    StmtKind::NamespaceBlock { name, body } => StmtKind::NamespaceBlock {
                        name,
                        body: filter(body, owned),
                    },
                    StmtKind::Synthetic(body) => StmtKind::Synthetic(filter(body, owned)),
                    StmtKind::IncludeOnceGuard { source_path, body } => {
                        StmtKind::IncludeOnceGuard {
                            source_path,
                            body: filter(body, owned),
                        }
                    }
                    StmtKind::If {
                        condition,
                        then_body,
                        elseif_clauses,
                        else_body,
                    } => StmtKind::If {
                        condition,
                        then_body: filter(then_body, owned),
                        elseif_clauses: elseif_clauses
                            .into_iter()
                            .map(|(guard, arm)| (guard, filter(arm, owned)))
                            .collect(),
                        else_body: else_body.map(|arm| filter(arm, owned)),
                    },
                    other => other,
                };
                stmt
            })
            .collect()
    }
    filter(program, owned)
}

/// Returns whether the program names any function this prelude declares.
///
/// A literal counts as well as a call: `function_exists('set_error_handler')` and
/// `array_map('trigger_error', …)` already land in `usage.functions`, and the `literals` pool
/// catches the remaining string forms. What this cannot see is a name the program COMPUTES
/// (`$f = 'trigger' . '_error'; $f();`), which stays an undefined function — the same
/// limitation `superglobals::seed_cli_populated_superglobals` has, and for the same reason: a
/// pay-for-use gate reads what the source SPELLS.
fn program_names_the_surface(program: &[Stmt]) -> bool {
    let used = usage::collect(program);
    SURFACE.iter().any(|name| program_names(&used, name))
}

/// Returns true for the three declarations that make up the shutdown registry.
///
/// Derived by NAME rather than by position so that reordering `declarations()` cannot silently
/// change what the gate drops.
fn is_shutdown_declaration(kind: &StmtKind) -> bool {
    let StmtKind::FunctionDecl { name, .. } = kind else {
        return false;
    };
    matches!(
        crate::names::php_symbol_key(name).as_str(),
        "register_shutdown_function"
            | "__elephc_shutdown_function_state"
            | "__elephc_shutdown_run"
    )
}

/// Returns whether the program names `register_shutdown_function` specifically.
///
/// Separate from [`program_names_the_surface`] because the two questions have different
/// answers and different costs: the surface gate decides whether PHP's error rule is compiled
/// in at all, while THIS one decides whether every `exit()` in the binary grows a drain call.
/// A program that only calls `error_reporting()` must answer false here.
fn program_names_shutdown_registration(program: &[Stmt]) -> bool {
    program_names(&usage::collect(program), "register_shutdown_function")
}

/// Returns whether one name is spelled by the program, as a call or as a string literal.
fn program_names(used: &usage::Usage, name: &str) -> bool {
    used.references(name) || used.literals.contains(&crate::names::php_symbol_key(name))
}

/// Gives a NON-`--web` build the error-handling surface PHP's every SAPI has, when the program
/// mentions it.
///
/// Returns the program untouched under `--web`: `web_prelude::inject_if_web` has already
/// injected the same declarations (through [`declarations`]), and a second copy would be a
/// redeclaration.
///
/// `entry_path` resolves the `__FILE__` in `trigger_error()`'s handler call. The magic-constant
/// pass has already run per file by the time a prelude is injected, so nothing later would
/// resolve it — exactly the reason `web_prelude::inject_if_web` substitutes there too.
pub fn inject_if_used(
    program: Program,
    web: bool,
    entry_path: &Path,
    inventory: &mut crate::optimize::reachability::PreludeInventory,
) -> Program {
    if web || !program_names_the_surface(&program) {
        return program;
    }
    // PAY-FOR-USE WITHIN THE SURFACE. The shutdown trio rides in the same `declarations()` list
    // as the error functions, and leaving it there for every caller is not free: reachability
    // keeps the WHOLE injected prelude once any of it is in (the dispatch helper's
    // `call_user_func` is a dynamic call, which the pass treats conservatively), so a program
    // whose only mention is `error_reporting()` would compile the registry AND grow a drain call
    // at every one of its `exit()` sites. Dropping the declarations here is the only gate fine
    // enough to stop that, and it is the same POLICY drop `web_prelude::inject_if_web` performs
    // for the callable session handler — before `record_program`, so the pruner's literal
    // harvest cannot re-root what policy removed.
    let mut declarations = declarations();
    if !program_names_shutdown_registration(&program) {
        declarations.retain(|stmt| !is_shutdown_declaration(&stmt.kind));
    }
    // The shadowing filter is keyed on what was ACTUALLY injected, not on the full surface: a
    // program that keeps its own `register_shutdown_function()` polyfill must keep it when the
    // prelude declined to declare one.
    let owned: std::collections::HashSet<String> = declarations
        .iter()
        .filter_map(|stmt| match &stmt.kind {
            StmtKind::FunctionDecl { name, .. } => Some(crate::names::php_symbol_key(name)),
            _ => None,
        })
        .collect();
    let program = without_shadowing_names(program, &owned);
    let mut combined =
        crate::magic_constants::substitute_file_constants(declarations, entry_path);
    inventory.record_program(GROUP, &combined);
    // See `DIAG_DISPATCH_GROUP`: these two are reached from the generated runtime, not from PHP.
    let diag_group = inventory.group_mut(DIAG_DISPATCH_GROUP);
    diag_group
        .functions
        .insert(crate::names::php_symbol_key("__elephc_diag_dispatch"));
    diag_group
        .functions
        .insert(crate::names::php_symbol_key(crate::names::DIAG_RENDER_FUNCTION));
    // See `SHUTDOWN_RUN_GROUP`: the drain entry is reached from EMITTED CODE, not from PHP, so
    // reachability would delete it and `lower_exit` would call a symbol nobody defined. Forced
    // only for a program that spells `register_shutdown_function` — one that merely names
    // `error_reporting()` keeps the registry pruned and pays nothing at its exit sites.
    if program_names_shutdown_registration(&program) {
        inventory
            .group_mut(SHUTDOWN_RUN_GROUP)
            .functions
            .insert(crate::names::php_symbol_key(
                crate::names::SHUTDOWN_RUN_FUNCTION,
            ));
    }
    combined.extend(program);
    combined
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for the shared error-handling prelude: that the gate and the declared set
    //! agree, that a program naming nothing is untouched, that a `--web` compile is left to
    //! the web prelude, and that one mention forces the runtime's dispatch pair.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Tests parse raw source, which is the stage `inject_if_used` runs at.

    use super::*;
    use crate::parser::ast::StmtKind;

    /// Parses PHP the way `inject_if_used` sees it: tokenize then parse.
    fn parse(source: &str) -> Program {
        let tokens = crate::lexer::tokenize(source).expect("test source must tokenize");
        crate::parser::parse(&tokens).expect("test source must parse")
    }

    /// The names the prelude declares, in order.
    fn declared_names() -> Vec<String> {
        declarations()
            .iter()
            .filter_map(|stmt| match &stmt.kind {
                StmtKind::FunctionDecl { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    /// Every PHP-visible name in [`SURFACE`] must actually be declared, or the gate admits a
    /// program to a surface that does not contain what it asked for.
    #[test]
    fn every_gated_name_is_declared() {
        let declared = declared_names();
        for name in SURFACE {
            assert!(
                declared.iter().any(|candidate| candidate == name),
                "{name} is gated on but not declared"
            );
        }
    }

    /// Pay-for-use: a program that names none of the surface must be handed back unchanged.
    #[test]
    fn a_program_that_names_nothing_is_left_alone() {
        let mut inventory = crate::optimize::reachability::PreludeInventory::new();
        let program = parse("<?php echo 1;");
        let before = program.len();
        let after = inject_if_used(program, false, Path::new("entry.php"), &mut inventory);
        assert_eq!(after.len(), before);
        assert!(inventory.groups.is_empty());
    }

    /// A `--web` compile must be left alone: the web prelude already injected these.
    #[test]
    fn a_web_compile_is_left_alone() {
        let mut inventory = crate::optimize::reachability::PreludeInventory::new();
        let program = parse("<?php set_error_handler('h');");
        let before = program.len();
        let after = inject_if_used(program, true, Path::new("entry.php"), &mut inventory);
        assert_eq!(after.len(), before);
        assert!(inventory.groups.is_empty());
    }

    /// One mention is enough, and the dispatch pair is forced so the runtime's call into
    /// `__elephc_diag_render` still has a target after reachability pruning.
    #[test]
    fn one_mention_injects_the_surface_and_forces_the_dispatch_group() {
        let mut inventory = crate::optimize::reachability::PreludeInventory::new();
        let injected = inject_if_used(
            parse("<?php error_reporting(E_ALL);"),
            false,
            Path::new("entry.php"),
            &mut inventory,
        );
        assert!(injected.len() > 1);
        assert!(inventory.groups.contains_key(GROUP));
        let diag = inventory
            .groups
            .get(DIAG_DISPATCH_GROUP)
            .expect("the dispatch group is forced");
        assert!(diag
            .functions
            .contains(&crate::names::php_symbol_key(crate::names::DIAG_RENDER_FUNCTION)));
    }

    /// A `function_exists('trigger_error')` guard is a mention: its subject must exist, or the
    /// guard silently takes its else branch.
    #[test]
    fn a_function_exists_probe_is_a_mention() {
        assert!(program_names_the_surface(&parse(
            "<?php if (function_exists('trigger_error')) { echo 1; }"
        )));
    }

    /// The polyfill guard: a declaration nested inside the `if` must go, because elephc emits
    /// its body as a symbol whether or not the branch runs, and the prelude declares the same
    /// name. The end-to-end proof is `a_function_exists_guarded_polyfill_still_builds` in
    /// `tests/error_handling_surface_tests.rs`; this pins the walk that finds it.
    #[test]
    fn a_guarded_polyfill_declaration_is_dropped() {
        let program = without_shadowing_declarations(parse(
            "<?php if (!function_exists('trigger_error')) { function trigger_error($m) { return true; } } echo 1;",
        ));
        let StmtKind::If { then_body, .. } = &program[0].kind else {
            panic!("expected the guard to survive, only its declaration to go");
        };
        assert!(then_body.is_empty(), "the shadowing declaration must be dropped");
        assert_eq!(program.len(), 2, "nothing else may be removed");
    }

    /// A declaration this prelude does not own is left where it is, however it is nested.
    #[test]
    fn an_unrelated_declaration_is_left_alone() {
        let program = without_shadowing_declarations(parse(
            "<?php if (!function_exists('str_contains')) { function str_contains($h, $n) { return false; } }",
        ));
        let StmtKind::If { then_body, .. } = &program[0].kind else {
            panic!("expected an if");
        };
        assert_eq!(then_body.len(), 1, "an unrelated polyfill must survive");
    }
}
