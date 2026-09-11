//! Purpose:
//! Include and include-once statement lowering.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Lowers a resolver-preserved runtime include/require through the Magician bridge.
pub(super) fn lower_include(
    ctx: &mut LoweringContext<'_, '_>,
    path: &Expr,
    once: bool,
    required: bool,
    span: Span,
) {
    let path = lower_expr(ctx, path);
    let call = ctx.emit_value(
        Op::RuntimeCall,
        vec![path.value],
        Some(Immediate::RuntimeCall(RuntimeCallTarget::DynamicInclude {
            once,
            required,
            strict_php: crate::strict_php::is_enabled(),
        })),
        PhpType::Mixed,
        effects_lookup::runtime_effects(),
        Some(span),
    );
    release_expr_statement_result(ctx, call, span);
    ctx.mark_eval_executed();
    ctx.apply_eval_barrier();
    ctx.clear_static_callable_locals();
}

/// Lowers an include-once marker.
pub(super) fn lower_include_once_mark(ctx: &mut LoweringContext<'_, '_>, source_path: &std::path::Path, span: Span) {
    let source = source_immediate(ctx, source_path);
    ctx.emit_void(
        Op::IncludeOnceMark,
        Vec::new(),
        Some(source),
        Op::IncludeOnceMark.default_effects(),
        Some(span),
    );
}

/// Lowers an include-once guarded body.
pub(super) fn lower_include_once_guard(
    ctx: &mut LoweringContext<'_, '_>,
    source_path: &std::path::Path,
    body: &[Stmt],
    span: Span,
) {
    let source = source_immediate(ctx, source_path);
    let should_run = ctx
        .builder
        .emit_with_effects(
            Op::IncludeOnceGuard,
            Vec::new(),
            Some(source),
            IrType::I64,
            PhpType::Bool,
            Ownership::NonHeap,
            Op::IncludeOnceGuard.default_effects(),
            Some(span),
        )
        .expect("include_once_guard produces a branch condition");
    let body_block = ctx
        .builder
        .create_named_block("include_once_body", Vec::new());
    let after_block = ctx
        .builder
        .create_named_block("include_once_after", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: should_run,
        then_target: body_block,
        then_args: Vec::new(),
        else_target: after_block,
        else_args: Vec::new(),
    });
    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(body_block);
    let surrounding_try_handler_stack = ctx.try_handler_stack.clone();
    lower_block(ctx, body);
    branch_to(ctx, after_block);
    ctx.builder.position_at_end(after_block);
    ctx.try_handler_stack = surrounding_try_handler_stack;
    ctx.clear_static_callable_locals();
}

/// Resolves against frozen input metadata; a missing input is a compile diagnostic.
fn source_immediate(ctx: &LoweringContext<'_, '_>, path: &std::path::Path) -> Immediate {
    match ctx.source_catalog.as_ref().and_then(|catalog| catalog.id_for_path(path)) {
        Some(id) => Immediate::Source(id),
        None => Immediate::UnresolvedSource(path.to_path_buf()),
    }
}
