//! Purpose:
//! Preserves closure identities across loop backedges only after checking the
//! entire condition, update, and body before any loop instruction is emitted.
//!
//! Called from:
//! - `stmt::loops` at loop entry and `expr::callable_resolution` before direct calls.
//!
//! Key details:
//! - New for/while facts require raw scalar inputs and one closed primitive body block.
//! - Opaque syntax and indirect mutation fail closed; ordinary descriptor calls remain available.
//! - Existing do/while direct calls retain their eligibility after identity filtering.

use crate::ir::{Effects, Immediate, IrType, Op, Terminator};
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};
use crate::types::{FunctionSig, PhpType};
use std::collections::HashMap;

use super::context::{LoweringContext, StaticCallableBinding};

/// Installs only entry closure facts proved invariant over every loop path.
pub(crate) fn begin_loop_closure_facts(
    ctx: &mut LoweringContext<'_, '_>,
    condition: Option<&Expr>,
    update: Option<&Stmt>,
    body: &[Stmt],
    new_direct_path: bool,
) -> HashMap<String, StaticCallableBinding> {
    let original = ctx.static_callable_locals_snapshot();
    let mut facts = original.clone();
    facts.retain(|local, binding| {
        let StaticCallableBinding::Closure { loop_guarded, .. } = binding else {
            return false;
        };
        if !ctx.can_track_static_callable_local(local) || ctx.is_ref_bound_local(local)
            || ctx.has_eval_barrier() || ctx.eval_executed()
            || ((new_direct_path || *loop_guarded) && !scalar_nothrow_target(ctx, binding))
        {
            return false;
        }
        let stable = condition.is_none_or(|expr| stable_expr(ctx, local, expr))
            && update.is_none_or(|stmt| stable_stmt(ctx, local, stmt))
            && stable_body(ctx, local, body);
        if stable {
            if let StaticCallableBinding::Closure { loop_guarded, .. } = binding {
                *loop_guarded |= new_direct_path;
            }
        }
        stable
    });
    let mut header_facts = original;
    header_facts.retain(|_, target| !matches!(target, StaticCallableBinding::Closure { .. }));
    header_facts.extend(facts.clone());
    ctx.restore_static_callable_locals(header_facts);
    facts
}

/// Drops fresh condition bindings and retains only still-valid invariant entry closures.
pub(crate) fn retain_loop_body_facts(
    ctx: &mut LoweringContext<'_, '_>,
    entry: &HashMap<String, StaticCallableBinding>,
) {
    let mut current = ctx.static_callable_locals_snapshot();
    current.retain(|name, target| entry.get(name) == Some(target));
    ctx.clear_static_callable_locals();
    ctx.restore_static_callable_locals(current);
}

/// Applies strict call gates only to facts newly carried through for/while loops.
pub(crate) fn direct_call_is_eligible(
    ctx: &LoweringContext<'_, '_>,
    binding: &StaticCallableBinding,
    args: &[Expr],
) -> bool {
    let StaticCallableBinding::Closure { signature, loop_guarded: true, .. } = binding else {
        return true;
    };
    scalar_nothrow_target(ctx, binding)
        && args.len() == signature.params.len()
        && args.iter().zip(&signature.params).all(|(arg, (_, parameter))| {
            scalar_argument_type(ctx, arg).is_some_and(|actual| {
                *parameter == PhpType::Mixed || *parameter == actual
                    || (*parameter == PhpType::Bool && actual == PhpType::False)
            })
        })
}

/// Proves scalar inputs/results, closed local state, and the lowered body's nonthrowing effects.
fn scalar_nothrow_target(ctx: &LoweringContext<'_, '_>, binding: &StaticCallableBinding) -> bool {
    let StaticCallableBinding::Closure { name, signature, captures, .. } = binding else {
        return false;
    };
    if !scalar_signature(signature) || captures.iter().any(|capture| {
        capture.by_ref_local.is_some()
            || !scalar_type(&ctx.builder.value_php_type(capture.value))
            || !ctx.builder.function().value(capture.value).is_some_and(|value| {
                value.ir_type == if value.php_type == PhpType::Float { IrType::F64 } else { IrType::I64 }
            })
            || ctx.builder.value_defining_op(capture.value) == Some(Op::LoadRefCell)
    }) {
        return false;
    }
    let Some(function) = ctx.closure_function(name) else { return false };
    if function.flags.is_generator || function.flags.by_ref_return || function.blocks.len() != 1
        || function.return_type == IrType::Void
    {
        return false;
    }
    let fallible = Effects::MAY_THROW | Effects::MAY_FATAL | Effects::MAY_WARN;
    function.instructions.iter().all(|inst| {
        !inst.effects.intersects(fallible)
            && (scalar_type(&inst.result_php_type) || inst.result_php_type == PhpType::Mixed)
            && (inst.op == Op::ConcatReset
                || !inst.effects.intersects(Effects::READS_GLOBAL | Effects::WRITES_GLOBAL))
            && scalar_body_primitive(inst.op)
            && (inst.op != Op::MixedNumericBinop || matches!(inst.immediate,
                Some(Immediate::MixedNumericOp(crate::ir::MixedNumericOp::Add
                    | crate::ir::MixedNumericOp::Sub | crate::ir::MixedNumericOp::Mul))))
    }) && function.blocks.iter().all(|block| {
        !matches!(block.terminator, Some(Terminator::Throw { .. } | Terminator::Fatal { .. }))
    })
}

/// Fails closed for every call surface and future opcode, even after effect refinement.
fn scalar_body_primitive(op: Op) -> bool {
    use Op::*;
    matches!(op, ConstI64 | ConstF64 | ConstBool | ConstNull | LoadLocal | StoreLocal
        | ZeroLocalSlot | UnsetLocal | ReleaseLocalSlot | ICheckedAdd | ICheckedSub | ICheckedMul
        | FAdd | FSub | FMul | FNeg | IBitAnd | IBitOr | IBitXor | IBitNot
        | ICmp | FCmp | PhpRelCmp | StrictEq | StrictNotEq | LooseEq | LooseNotEq | Spaceship
        | IsNull | IsTruthy | IsEmpty | TypePredicate | IToF | MixedBox | MixedClone | MixedUnbox
        | MixedTagOf | MixedNumericBinop | Acquire | Release | ReleaseUnlessAliases
        | Move | Borrow | EnsureOwned | ConcatReset | Nop)
}

/// Allows exact positional scalar calls without coercion, defaults, references, or generators.
fn scalar_signature(signature: &FunctionSig) -> bool {
    signature.variadic.is_none() && !signature.by_ref_return && !signature.is_generator
        && signature.deprecation.is_none() && !signature.ref_params.iter().any(|by_ref| *by_ref)
        && signature.params.iter().all(|(_, ty)| scalar_type(ty) || *ty == PhpType::Mixed)
        && (scalar_type(&signature.return_type) || signature.return_type == PhpType::Mixed)
}

/// Recognizes PHP scalars that cannot own user cleanup hooks.
fn scalar_type(ty: &PhpType) -> bool {
    matches!(ty, PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::False | PhpType::Void)
}

/// Requires a literal or a physically scalar local, rather than a checker-only boxed refinement.
fn scalar_argument_type(ctx: &LoweringContext<'_, '_>, expr: &Expr) -> Option<PhpType> {
    match &expr.kind {
        ExprKind::IntLiteral(_) => Some(PhpType::Int),
        ExprKind::FloatLiteral(_) => Some(PhpType::Float),
        ExprKind::BoolLiteral(false) => Some(PhpType::False),
        ExprKind::BoolLiteral(true) => Some(PhpType::Bool),
        ExprKind::Null => Some(PhpType::Void),
        ExprKind::Variable(name) => {
            if ctx.is_ref_bound_local(name) || !ctx.can_track_static_callable_local(name) {
                return None;
            }
            let ty = ctx.local_type(name);
            let slot = *ctx.local_slots.get(name)?;
            let storage = ctx.builder.local_php_type(slot);
            if !scalar_type(&ty) || !scalar_type(&storage) { return None; }
            let ir = ctx.builder.function().locals.get(slot.as_raw() as usize)?.ir_type;
            ((ir == IrType::I64 && ty != PhpType::Float)
                || (ir == IrType::F64 && ty == PhpType::Float)).then_some(ty)
        }
        _ => None,
    }
}

/// Checks every statement, including dead-looking backedge writers, before lowering.
fn stable_body(ctx: &LoweringContext<'_, '_>, local: &str, body: &[Stmt]) -> bool {
    body.iter().all(|stmt| stable_stmt(ctx, local, stmt))
}

/// Admits a closed scalar statement subset; new syntax defaults to descriptor fallback.
fn stable_stmt(ctx: &LoweringContext<'_, '_>, local: &str, stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::Assign { name, value } | StmtKind::TypedAssign { name, value, .. } => {
            name != local && (!ctx.local_slots.contains_key(name)
                || harmless_local(ctx, name)) && stable_expr(ctx, local, value)
        }
        StmtKind::Echo(value) | StmtKind::ExprStmt(value) => stable_expr(ctx, local, value),
        StmtKind::If { condition, then_body, elseif_clauses, else_body } => {
            stable_expr(ctx, local, condition) && stable_body(ctx, local, then_body)
                && elseif_clauses.iter().all(|(condition, body)| {
                    stable_expr(ctx, local, condition) && stable_body(ctx, local, body)
                }) && else_body.as_ref().is_none_or(|body| stable_body(ctx, local, body))
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { condition, body } => {
            stable_expr(ctx, local, condition) && stable_body(ctx, local, body)
        }
        StmtKind::For { init, condition, update, body } => {
            init.as_ref().is_none_or(|stmt| stable_stmt(ctx, local, stmt))
                && condition.as_ref().is_none_or(|expr| stable_expr(ctx, local, expr))
                && update.as_ref().is_none_or(|stmt| stable_stmt(ctx, local, stmt))
                && stable_body(ctx, local, body)
        }
        StmtKind::Synthetic(body) => stable_body(ctx, local, body),
        StmtKind::Return(value) => value.as_ref().is_none_or(|value| stable_expr(ctx, local, value)),
        StmtKind::Break(_) | StmtKind::Continue(_) => true,
        _ => false,
    }
}

/// Rejects reference, global, and destructor-bearing locals as possible implicit callbacks.
fn harmless_local(ctx: &LoweringContext<'_, '_>, name: &str) -> bool {
    !ctx.is_ref_bound_local(name) && ctx.can_track_static_callable_local(name)
        && (scalar_type(&ctx.local_type(name)) || ctx.local_type(name) == PhpType::Str)
}

/// Rejects opaque calls, aliases, declarations, and indirect writes anywhere in the loop.
fn stable_expr(ctx: &LoweringContext<'_, '_>, local: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::IntLiteral(_) | ExprKind::FloatLiteral(_) | ExprKind::BoolLiteral(_)
        | ExprKind::Null | ExprKind::StringLiteral(_) | ExprKind::ConstRef(_) => true,
        ExprKind::Variable(name) => name != local && harmless_local(ctx, name),
        ExprKind::BinaryOp { left, right, .. } => {
            stable_expr(ctx, local, left) && stable_expr(ctx, local, right)
        }
        ExprKind::Negate(value) | ExprKind::Not(value) | ExprKind::BitNot(value)
        | ExprKind::Cast { expr: value, .. } => stable_expr(ctx, local, value),
        ExprKind::PreIncrement(name) | ExprKind::PostIncrement(name)
        | ExprKind::PreDecrement(name) | ExprKind::PostDecrement(name) => {
            name != local && harmless_local(ctx, name)
        }
        ExprKind::ClosureCall { var, args } => {
            var == local && args.iter().all(|arg| stable_expr(ctx, local, arg))
        }
        ExprKind::Assignment { target, value, result_target, prelude, .. } => {
            matches!(&target.kind, ExprKind::Variable(name) if name != local
                && (!ctx.local_slots.contains_key(name) || harmless_local(ctx, name)))
                && stable_body(ctx, local, prelude) && stable_expr(ctx, local, value)
                && result_target.as_ref().is_none_or(|target| stable_expr(ctx, local, target))
        }
        ExprKind::Ternary { condition, then_expr, else_expr } => {
            stable_expr(ctx, local, condition) && stable_expr(ctx, local, then_expr)
                && stable_expr(ctx, local, else_expr)
        }
        _ => false,
    }
}
