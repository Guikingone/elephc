//! Purpose:
//! Lowers `func_num_args()`, `func_get_args()`, and `func_get_arg($i)` inside an
//! arity-hungry function/method body, and appends the matching hidden trailing
//! arity-count ABI operand at DIRECT call sites that target such a function/method.
//!
//! Called from:
//! - `crate::ir_lower::expr::lower_function_call` — dispatches the three intrinsics before
//!   the general builtin path, and appends the hidden operand for direct user-function calls.
//! - `crate::ir_lower::expr` method/static-call lowering — appends the hidden operand for
//!   direct method/static-method calls.
//!
//! Key details:
//! - The hidden trailing arity-count parameter/operand is NOT part of the
//!   checker-visible `FunctionSig`; only DIRECT calls (name/method statically known at this
//!   lowering site) can supply it. Anything reached through a dynamic/uniform invoker
//!   (`call_user_func`, `array_map` with a runtime callable, first-class callables, closures
//!   wrapping a named function, `$fn()`) must be rejected before reaching here — see the
//!   checker-level gates in `crate::types::checker::callables::first_class` and
//!   `crate::types::checker::functions::call_validation`, plus the `panic!` safety net in
//!   `crate::ir_lower::expr::mod::lower_static_callable_value_call`.
//! - `func_get_args()`/`func_get_arg()` are lowered by SYNTHESIZING a small PHP expression
//!   tree (`[$p0, $p1, ..., ...$tail]` then `array_slice(..., 0, $argc)`) and feeding it
//!   through the ordinary `lower_expr` path, instead of hand-writing new EIR ops — this
//!   reuses the existing array-literal/spread, `array_slice`, and (for `func_get_arg`)
//!   `if`/`throw` lowering wholesale, and was chosen after `throw` used as a TERNARY arm was
//!   found to mis-lower a value produced on the non-throwing side (a pre-existing, unrelated
//!   bug); real `if { throw; }` STATEMENTS have none of that risk.

use crate::ir::ValueId;
use crate::ir_lower::context::{LoweredValue, LoweringContext};
use crate::names::php_symbol_key;
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};
use crate::span::Span;
use crate::types::checker::func_args_scan::HIDDEN_ARGC_PARAM_NAME;
use crate::types::FunctionSig;

use super::{is_spread_arg, lower_expr};

/// Returns `true` for the case-insensitive, optionally-backslash-qualified names
/// `func_num_args`, `func_get_args`, and `func_get_arg`.
pub(super) fn is_func_args_intrinsic(name: &str) -> bool {
    crate::types::checker::func_args_scan::is_func_args_intrinsic_name(name)
}

/// Lowers a direct call to `func_num_args()`, `func_get_args()`, or `func_get_arg($i)`.
///
/// Requires the CURRENT function/method body to be arity-hungry (`ctx.self_is_arity_hungry()`
/// — guaranteed by the checker for every legal call site: closures and global scope are
/// rejected before EIR lowering runs). Panics with a precise message if reached with a
/// non-arity-hungry owner, which can only happen for a call surface this compiler does not
/// yet support (closures) reaching this far — a defensive, loud safety net rather than a
/// silent miscompile.
pub(super) fn lower_func_args_intrinsic(
    ctx: &mut LoweringContext<'_, '_>,
    canonical_name: &str,
    args: &[Expr],
    expr: &Expr,
) -> LoweredValue {
    if !ctx.self_is_arity_hungry() {
        panic!(
            "func_num_args()/func_get_args()/func_get_arg() reached EIR lowering inside '{}', \
             which the checker did not mark arity-hungry. This compiler only supports these \
             builtins in direct free-function and method bodies (not closures); '{}' calling \
             one of them is unsupported.",
            ctx.owner_name(),
            ctx.owner_name(),
        );
    }
    let sig = ctx
        .self_signature()
        .expect("arity-hungry owner must have a resolved FunctionSig")
        .clone();
    let regular_param_count = crate::types::call_args::regular_param_count(&sig);
    let span = expr.span;

    match php_symbol_key(canonical_name.trim_start_matches('\\')).as_str() {
        "func_num_args" => lower_argc(ctx, &sig, regular_param_count, span),
        "func_get_args" => lower_expr(ctx, &args_slice_expr(&sig, regular_param_count, span)),
        "func_get_arg" => lower_func_get_arg(ctx, &sig, regular_param_count, args, span),
        other => unreachable!("is_func_args_intrinsic gated an unexpected name: {other}"),
    }
}

/// Lowers `func_num_args()` for this body.
///
/// When the count is self-derivable (`argc_is_self_derivable`: every declared parameter is
/// required, so it was necessarily passed) the body carries NO hidden argc parameter, and the
/// count is `<declared parameter count> + array_len($<variadic tail>)` — read straight off the
/// frame. Otherwise it reads the hidden `$__fga_argc` operand its callers supplied.
///
/// This is the same predicate `crate::types::checker::func_args_scan` uses to decide whether
/// to emit the hidden parameter, and `maybe_append_hidden_argc_operand` uses to decide whether
/// to pass it — one shared rule rather than three independent ones.
///
/// Emitted as EIR rather than synthesized as a `count(...)` PHP expression on purpose: routing
/// it through the builtin path made `count` infer a `Heap(Mixed)` result in some bodies and an
/// `I64` in others, and the `release` the Mixed shape drags along then ran on a raw integer
/// (measured: a clean segfault in `f(string $s) { return func_get_arg(0); }`). `array_len` is
/// unconditionally `I64`, and allocates nothing.
fn lower_argc(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    regular_param_count: usize,
    span: Span,
) -> LoweredValue {
    if !crate::types::checker::func_args_scan::argc_is_self_derivable(sig) {
        return lower_expr(
            ctx,
            &Expr::new(ExprKind::Variable(HIDDEN_ARGC_PARAM_NAME.to_string()), span),
        );
    }
    let declared = lower_expr(
        ctx,
        &Expr::new(ExprKind::IntLiteral(regular_param_count as i64), span),
    );
    // `ensure_variadic_for_func_args` guarantees a tail on every marked signature; a signature
    // without one can only mean zero extra arguments are representable, so the declared count
    // already IS the answer.
    let Some(variadic_name) = sig.variadic.as_ref() else {
        return declared;
    };
    let tail = lower_expr(
        ctx,
        &Expr::new(ExprKind::Variable(variadic_name.clone()), span),
    );
    let tail_len = ctx.emit_value(
        crate::ir::Op::ArrayLen,
        vec![tail.value],
        None,
        crate::types::PhpType::Int,
        crate::ir::Op::ArrayLen.default_effects(),
        Some(span),
    );
    ctx.emit_value(
        crate::ir::Op::IAdd,
        vec![declared.value, tail_len.value],
        None,
        crate::types::PhpType::Int,
        crate::ir::Op::IAdd.default_effects(),
        Some(span),
    )
}

/// Builds `[$p0, $p1, ..., ...$tail]` — every declared regular parameter followed by a
/// spread of the (real or synthesized) variadic tail array, in call order. This is exactly
/// the full set of values PHP would report from `func_get_args()` if every declared
/// parameter AND every extra argument had actually been passed; `args_slice_expr` trims it
/// down to the real `func_num_args()` count.
fn full_args_array_expr(sig: &FunctionSig, regular_param_count: usize, span: Span) -> Expr {
    let mut items: Vec<Expr> = sig
        .params
        .iter()
        .take(regular_param_count)
        .map(|(name, _)| Expr::new(ExprKind::Variable(name.clone()), span))
        .collect();
    if let Some(variadic_name) = &sig.variadic {
        items.push(Expr::new(
            ExprKind::Spread(Box::new(Expr::new(ExprKind::Variable(variadic_name.clone()), span))),
            span,
        ));
    }
    Expr::new(ExprKind::ArrayLiteral(items), span)
}

/// Builds `array_slice(<full_args_array_expr>, 0, $__fga_argc)` — PHP's `func_get_args()`
/// semantics: every parameter/variadic-tail value up to (not including) the number of
/// arguments actually passed, trailing unpassed defaulted parameters excluded.
/// `array_slice` already returns a fresh array (mutation isolation, matching PHP's
/// documented "func_get_args() returns copies" behavior) and tolerates a length greater
/// than the source array (php-verified: `array_slice($a, 0, $n)` with `$n > count($a)`
/// simply returns the whole array), so no extra bounds handling is needed here.
///
/// When the count is self-derivable there is nothing to trim — every declared parameter was
/// necessarily passed, so the full array IS the answer — and the freshly built array literal
/// already satisfies the copy semantics `array_slice` was providing.
fn args_slice_expr(sig: &FunctionSig, regular_param_count: usize, span: Span) -> Expr {
    let full = full_args_array_expr(sig, regular_param_count, span);
    if crate::types::checker::func_args_scan::argc_is_self_derivable(sig) {
        return full;
    }
    Expr::new(
        ExprKind::FunctionCall {
            name: crate::names::Name::unqualified("array_slice"),
            args: vec![
                full,
                Expr::new(ExprKind::IntLiteral(0), span),
                Expr::new(ExprKind::Variable(HIDDEN_ARGC_PARAM_NAME.to_string()), span),
            ],
        },
        span,
    )
}

/// Lowers `func_get_arg($position)`: validates `0 <= $position < $__fga_argc` with the same
/// two static `ValueError` messages PHP 8 raises (php-verified), then reads
/// `<full_args_array_expr>[$position]`.
///
/// Implemented with real EIR blocks/branches (mirroring `lower_conditional_non_local_null_coalesce_assignment`)
/// and a genuine `StmtKind::Throw` STATEMENT lowered via `crate::ir_lower::stmt::lower_stmt` —
/// not a `Throw` expression inside a `Ternary` arm, which is a separate, pre-existing bug
/// (verified empirically: it drops the non-throwing arm's value in some cases).
fn lower_func_get_arg(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    regular_param_count: usize,
    args: &[Expr],
    span: Span,
) -> LoweredValue {
    let position_expr = args.first().cloned().unwrap_or(Expr::new(ExprKind::IntLiteral(0), span));
    let position = lower_expr(ctx, &position_expr);
    let zero = lower_expr(ctx, &Expr::new(ExprKind::IntLiteral(0), span));
    let is_negative = ctx.emit_value(
        crate::ir::Op::ICmp,
        vec![position.value, zero.value],
        Some(crate::ir::Immediate::CmpPredicate(crate::ir::CmpPredicate::Slt)),
        crate::types::PhpType::Bool,
        crate::ir::Op::ICmp.default_effects(),
        Some(span),
    );

    let negative_block = ctx.builder.create_named_block("func_get_arg.negative", Vec::new());
    let range_check_block = ctx.builder.create_named_block("func_get_arg.range_check", Vec::new());
    ctx.builder.terminate(crate::ir::Terminator::CondBr {
        cond: is_negative.value,
        then_target: negative_block,
        then_args: Vec::new(),
        else_target: range_check_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(negative_block);
    throw_value_error(
        ctx,
        "func_get_arg(): Argument #1 ($position) must be greater than or equal to 0",
        span,
    );

    ctx.builder.position_at_end(range_check_block);
    let argc = lower_argc(ctx, sig, regular_param_count, span);
    let out_of_range = ctx.emit_value(
        crate::ir::Op::ICmp,
        vec![position.value, argc.value],
        Some(crate::ir::Immediate::CmpPredicate(crate::ir::CmpPredicate::Sge)),
        crate::types::PhpType::Bool,
        crate::ir::Op::ICmp.default_effects(),
        Some(span),
    );
    let out_of_range_block = ctx.builder.create_named_block("func_get_arg.out_of_range", Vec::new());
    let value_block = ctx.builder.create_named_block("func_get_arg.value", Vec::new());
    ctx.builder.terminate(crate::ir::Terminator::CondBr {
        cond: out_of_range.value,
        then_target: out_of_range_block,
        then_args: Vec::new(),
        else_target: value_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(out_of_range_block);
    throw_value_error(
        ctx,
        "func_get_arg(): Argument #1 ($position) must be less than the number of the arguments passed to the currently executed function",
        span,
    );

    ctx.builder.position_at_end(value_block);
    // `position` was already lowered above from the caller's own expression; reuse it as the
    // index into the same synthesized full-args array `func_get_args()` reads from. Bounds are
    // already proven (0 <= position < argc <= full array length), so a plain indexed read is
    // safe.
    let index_expr = Expr::new(ExprKind::Variable(stash_position_local(ctx, position.value, span)), span);
    let selected = Expr::new(
        ExprKind::ArrayAccess {
            array: Box::new(full_args_array_expr(sig, regular_param_count, span)),
            index: Box::new(index_expr),
        },
        span,
    );
    lower_expr(ctx, &selected)
}

/// Stores an already-lowered integer `ValueId` into a fresh hidden local and returns its
/// name, so it can be re-read through the ordinary `Variable` expression path (used to
/// build the synthesized `func_get_arg` index-access expression without re-evaluating the
/// caller's `$position` expression a second time — it may have side effects).
fn stash_position_local(ctx: &mut LoweringContext<'_, '_>, value: ValueId, span: Span) -> String {
    let temp_name = ctx.declare_hidden_temp(crate::types::PhpType::Int);
    let lowered = LoweredValue {
        value,
        ir_type: crate::ir::IrType::I64,
    };
    super::store_value_into_temp(ctx, &temp_name, crate::types::PhpType::Int, lowered, span);
    temp_name
}

/// Computes the number of PHP-visible arguments a DIRECT call site passes to an
/// arity-hungry callee, as a compile-time-known literal count. PHP named-argument keys
/// are always static text, so the resolved slot count (`plan.regular_args.len() +
/// plan.variadic_args.len()`, matching `func_num_args()`'s "highest filled index + 1"
/// semantics php-verified against `f($a, $b = 2)`/`m($a, $b = 10, $c = 20)` probes) is
/// static too — no runtime counting is needed for any SUPPORTED call shape.
///
/// A STATICALLY-sized spread (`f(...[7, 8])`, or any spread source
/// `super::has_static_call_spread_args`/`super::expand_static_call_spread_args` can flatten
/// — the same static-spread expansion `lower_args_with_signature` itself applies before
/// materializing operands) is expanded first, so its element count is folded in exactly like
/// ordinary positional arguments. Only a spread whose length survives expansion (genuinely
/// runtime-determined, e.g. `f(...$dynamicallySizedArray)`) is unsupported; the checker
/// rejects that case at compile time (`func_args_scan::validate_dynamic_spread_calls`)
/// before this ever runs, so the `panic!` below is unreachable defense-in-depth, not the
/// primary gate.
fn compute_static_passed_count(sig: &FunctionSig, args: &[Expr], callee_desc: &str) -> i64 {
    let expanded_args;
    let args: &[Expr] = if super::has_static_call_spread_args(args) {
        expanded_args = super::expand_static_call_spread_args(args);
        &expanded_args
    } else {
        args
    };
    if args.iter().any(is_spread_arg) {
        panic!(
            "compiler defense-in-depth: '{}' calls func_num_args()/func_get_args()/func_get_arg() \
             and was reached with a non-static spread (`...`) argument — the checker's \
             func_args_scan::validate_dynamic_spread_calls gate should have rejected this at \
             compile time already",
            callee_desc
        );
    }
    if crate::types::call_args::has_named_args(args) {
        match crate::types::call_args::plan_call_args(sig, args, Span::dummy(), true, true) {
            Ok(plan) => (plan.regular_args.len() + plan.variadic_args.len()) as i64,
            // The checker already validated this exact call against `sig`; a planner error
            // here would mean the checker accepted something the planner rejects, which is
            // a checker/planner divergence bug, not a legitimate runtime case. Fall back to
            // the raw source arg count rather than panicking, since it is still a syntactic
            // upper bound.
            Err(_) => args.len() as i64,
        }
    } else {
        args.len() as i64
    }
}

/// Appends the hidden trailing arity-count operand to `operands` when `callee_key` (a
/// free-function name or `"Class::method"`) is arity-hungry. No-op otherwise — untouched
/// (non-arity-hungry) call sites pay zero extra cost, satisfying the zero-cost requirement.
///
/// This is the SINGLE choke point through which every direct call shape (free function,
/// statically-named callback, constructor, instance method, static method) appends the
/// operand, so gating `argc_is_self_derivable` here — with the same predicate the two callee
/// prologues in `crate::ir_lower::function` use — is what keeps caller and callee in lockstep
/// no matter which callee-key resolver got here.
pub(super) fn maybe_append_hidden_argc_operand(
    ctx: &mut LoweringContext<'_, '_>,
    callee_key: &str,
    sig: &FunctionSig,
    args: &[Expr],
    span: Span,
    operands: &mut Vec<ValueId>,
) {
    if !ctx.is_arity_hungry_callee(callee_key)
        || crate::types::checker::func_args_scan::argc_is_self_derivable(sig)
    {
        return;
    }
    let count = compute_static_passed_count(sig, args, callee_key);
    let value = lower_expr(ctx, &Expr::new(ExprKind::IntLiteral(count), span));
    operands.push(value.value);
}

/// Lowers `throw new \ValueError($message);` as a real statement (terminates the current
/// block via `crate::ir_lower::stmt::lower_throw`'s `Terminator::Throw`), reusing the
/// existing, well-tested exception-construction and throw machinery instead of hand-writing
/// a target-specific `ValueError` allocation sequence.
fn throw_value_error(ctx: &mut LoweringContext<'_, '_>, message: &str, span: Span) {
    let throw_stmt = Stmt::new(
        StmtKind::Throw(Expr::new(
            ExprKind::NewObject {
                class_name: crate::names::Name::unqualified("ValueError"),
                args: vec![Expr::new(ExprKind::StringLiteral(message.to_string()), span)],
            },
            span,
        )),
        span,
    );
    crate::ir_lower::stmt::lower_stmt(ctx, &throw_stmt);
}
