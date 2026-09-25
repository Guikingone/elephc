//! Purpose:
//! While, do-while, and for-loop CFG lowering.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Lowers a `while` loop.
pub(super) fn lower_while(
    ctx: &mut LoweringContext<'_, '_>,
    condition: &Expr,
    body: &[Stmt],
    loop_span: Span,
) {
    apply_loop_storage_contracts(ctx, loop_span, Some(condition.span));
    let header = ctx.builder.create_named_block("while.cond", Vec::new());
    let body_block = ctx.builder.create_named_block("while.body", Vec::new());
    let exit = ctx.builder.create_named_block("while.exit", Vec::new());
    branch_to(ctx, header);

    ctx.builder.position_at_end(header);
    let cond = lower_expr(ctx, condition);
    let cond = ctx.truthy_consuming(cond, Some(condition.span));
    // The condition runs before both the body and the ordinary loop exit. Keep its
    // flow-sensitive facts separate from assignments made later in the body: otherwise a
    // body-only type leaks past the false condition (for example `$key = (string) $key` in the
    // condition followed by `$key = get_object()` in the body leaves an object-typed key after
    // the loop even though every normal exit just executed the string cast).
    let condition_exit_types = ctx.local_types_snapshot();
    let condition_exit_initialized = ctx.initialized_slots_snapshot();
    ctx.builder.terminate(Terminator::CondBr {
        cond: cond.value,
        then_target: body_block,
        then_args: Vec::new(),
        else_target: exit,
        else_args: Vec::new(),
    });

    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(body_block);
    let surrounding_try_handler_stack = ctx.try_handler_stack.clone();
    ctx.loop_stack.push(LoopFrame {
        break_block: exit,
        continue_block: header,
        cleanup: None,
        source_pin: None,
    });
    lower_block(ctx, body);
    let body_exit_types = ctx.local_types_snapshot();
    ctx.loop_stack.pop();
    ctx.try_handler_stack = surrounding_try_handler_stack;
    branch_to(ctx, header);
    ctx.builder.position_at_end(exit);
    ctx.restore_local_types(condition_exit_types);
    keep_arrays_the_body_filled(ctx, &body_exit_types);
    // …but a store in the BODY that widened a slot to boxed storage is not a body-only narrowing
    // to be undone: the exit is reached from iterations where that store ran, so the restored
    // narrow fact would be a claim about the slot's content that no longer holds on every incoming
    // edge. `$b = \strlen('ab'); $k = 0; while ($k < 2) { $b = 'sb' . $k; ++$k; } echo $b;` read
    // the boxed slot as `int` and printed `0` where PHP prints `sb1`.
    ctx.reassert_widened_local_storage_types();
    ctx.restore_initialized_slots(condition_exit_initialized);
    ctx.clear_static_callable_locals();
}

/// Keeps the element type a loop body gave an array that was EMPTY when the condition ran.
///
/// The `while` exit restores the condition's types so a body-only narrowing does not leak past a
/// false condition. For an array that was `[]` before the loop that restore is wrong: `[]` is typed
/// `array<never>` -- "no element seen yet" -- and the body is exactly what adds elements. Restoring
/// it made every read after the loop use a `never` element type, so the elements the loop pushed
/// read back as null. MEASURED against php 8.5.10:
///
///     $sets = []; $pos = 0;
///     while ($pos <= 1) { $sets[] = [$pos]; $pos++; }
///     foreach ($sets as $s) { $out[] = $s; }        php [[0],[1]]    elephc [null,null]
///
/// A `for` loop was right all along: it is lowered at a representation fixed point. The body's type
/// is always sound for such an array -- an empty array has no element to contradict it -- so only
/// that bottom element type is replaced, and every other fact keeps the condition's view.
fn keep_arrays_the_body_filled(ctx: &mut LoweringContext<'_, '_>, body_exit_types: &crate::types::TypeEnv) {
    let empty_at_condition: Vec<String> = ctx
        .local_types_snapshot()
        .iter()
        .filter(|(_, ty)| matches!(ty, PhpType::Array(element) if matches!(element.as_ref(), PhpType::Never)))
        .map(|(name, _)| name.clone())
        .collect();
    for name in empty_at_condition {
        let Some(body_ty) = body_exit_types.get(&name) else {
            continue;
        };
        let filled = match body_ty {
            PhpType::Array(element) => !matches!(element.as_ref(), PhpType::Never),
            PhpType::AssocArray { .. } => true,
            _ => false,
        };
        if filled {
            ctx.set_local_logical_type(&name, body_ty.clone());
        }
    }
}

/// Lowers a `do while` loop.
pub(super) fn lower_do_while(
    ctx: &mut LoweringContext<'_, '_>,
    body: &[Stmt],
    condition: &Expr,
    loop_span: Span,
) {
    apply_loop_storage_contracts(ctx, loop_span, Some(condition.span));
    let body_block = ctx.builder.create_named_block("do.body", Vec::new());
    let cond_block = ctx.builder.create_named_block("do.cond", Vec::new());
    let exit = ctx.builder.create_named_block("do.exit", Vec::new());
    branch_to(ctx, body_block);

    ctx.builder.position_at_end(body_block);
    let surrounding_try_handler_stack = ctx.try_handler_stack.clone();
    ctx.loop_stack.push(LoopFrame {
        break_block: exit,
        continue_block: cond_block,
        cleanup: None,
        source_pin: None,
    });
    lower_block(ctx, body);
    ctx.loop_stack.pop();
    ctx.try_handler_stack = surrounding_try_handler_stack;
    branch_to(ctx, cond_block);

    ctx.builder.position_at_end(cond_block);
    let cond = lower_expr(ctx, condition);
    let cond = ctx.truthy_consuming(cond, Some(condition.span));
    ctx.builder.terminate(Terminator::CondBr {
        cond: cond.value,
        then_target: body_block,
        then_args: Vec::new(),
        else_target: exit,
        else_args: Vec::new(),
    });
    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(exit);
    // A `break` leaves the loop without passing through any `if` join, so a store that widened
    // a slot to boxed storage just before it (`$n = count($m); foreach ($m as $v) { if ($v) {
    // $n = $m; break; } }`) never reached the flow facts this exit resumes from, and the read
    // below the loop loaded the boxed slot as a raw `int` -- a pointer where php has an array.
    // Symfony's DeepClone polyfill returns exactly that `$n`; every container rebuild then asked
    // `array_fill()` for a pointer-sized count. Same rule as the `while` exit.
    ctx.reassert_widened_local_storage_types();
    ctx.clear_static_callable_locals();
}

/// Lowers a `for` loop after establishing its loop-carried storage representation.
///
/// The fixed-point region starts below the initializer because an array created by the initializer
/// does not exist at the statement entry and therefore cannot be discovered by the outer
/// statement-level representation scan.
pub(super) fn lower_for(
    ctx: &mut LoweringContext<'_, '_>,
    init: Option<&Stmt>,
    condition: Option<&Expr>,
    update: Option<&Stmt>,
    body: &[Stmt],
    loop_span: Span,
) {
    if let Some(init) = init {
        lower_stmt(ctx, init);
    }
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let contract_span = condition
        .map(|c| c.span)
        .or_else(|| body.first().map(|s| s.span));
    apply_loop_storage_contracts(ctx, loop_span, contract_span);

    repr_fixpoint::lower_for_body_at_type_fixpoint(
        ctx,
        loop_span,
        condition,
        update,
        body,
        |ctx| lower_for_once(ctx, condition, update, body),
    );
}

/// Emits the control-flow graph, body, and update of a `for` loop exactly once.
fn lower_for_once(
    ctx: &mut LoweringContext<'_, '_>,
    condition: Option<&Expr>,
    update: Option<&Stmt>,
    body: &[Stmt],
) {
    let header = ctx.builder.create_named_block("for.cond", Vec::new());
    let body_block = ctx.builder.create_named_block("for.body", Vec::new());
    let update_block = ctx.builder.create_named_block("for.update", Vec::new());
    let exit = ctx.builder.create_named_block("for.exit", Vec::new());
    branch_to(ctx, header);

    ctx.builder.position_at_end(header);
    let cond = if let Some(condition) = condition {
        let cond = lower_expr(ctx, condition);
        ctx.truthy_consuming(cond, Some(condition.span))
    } else {
        emit_const_bool(ctx, true, None)
    };
    ctx.builder.terminate(Terminator::CondBr {
        cond: cond.value,
        then_target: body_block,
        then_args: Vec::new(),
        else_target: exit,
        else_args: Vec::new(),
    });

    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(body_block);
    let surrounding_try_handler_stack = ctx.try_handler_stack.clone();
    ctx.loop_stack.push(LoopFrame {
        break_block: exit,
        continue_block: update_block,
        cleanup: None,
        source_pin: None,
    });
    lower_block(ctx, body);
    ctx.loop_stack.pop();
    ctx.try_handler_stack = surrounding_try_handler_stack;
    branch_to(ctx, update_block);

    ctx.builder.position_at_end(update_block);
    if let Some(update) = update {
        lower_stmt(ctx, update);
    }
    branch_to(ctx, header);
    ctx.builder.position_at_end(exit);
    ctx.reassert_widened_local_storage_types();
    ctx.clear_static_callable_locals();
}
