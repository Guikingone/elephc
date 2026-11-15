//! Purpose:
//! Pre-scans a loop body for nested array writes (`$a[k1][k2] = v`, `$a[k1][k2][] = v`) that
//! would store a value of another type into a concretely-typed local container, and widens such
//! roots to `mixed` elements once, before the loop.
//!
//! Called from:
//! - `crate::ir_lower::stmt::apply_loop_storage_contracts`, i.e. every loop preheader.
//!
//! Key details:
//! - Outside loops a mismatched nested write widens its root at the write itself
//!   (`lower_widened_nested_local_assign`). A loop body cannot change a local's representation
//!   mid-body: reads on the back edge were already lowered against the concrete type. Widening
//!   the root in the preheader gives the whole loop one representation, after which the writes
//!   go through the generic boxed-Mixed fetch-for-write path.
//! - Only predictably mismatched writes widen: a concrete value type that differs from the
//!   leaf's element type, or a key the concrete chain cannot address. Boxed or unknown values
//!   keep the existing store coercion, and empty-typed leaves are refined by the write, so
//!   homogeneous matrices and append loops keep their concrete fast paths.

use crate::ir_lower::context::LoweringContext;
use crate::parser::ast::{BinOp, Expr, ExprKind, Stmt, StmtKind};
use crate::types::checker::infer_expr_type_syntactic;
use crate::types::PhpType;

/// Returns the locals whose element storage must be `mixed` for the nested writes in `body`.
pub(super) fn nested_write_roots_needing_mixed_elements(
    ctx: &LoweringContext<'_, '_>,
    body: &[Stmt],
) -> Vec<String> {
    let mut roots = Vec::new();
    collect_body(ctx, body, &mut roots);
    roots
}

/// Scans one statement list, including the parser's three-statement nested-append desugar.
fn collect_body(ctx: &LoweringContext<'_, '_>, body: &[Stmt], roots: &mut Vec<String>) {
    for (index, stmt) in body.iter().enumerate() {
        if let Some(root) = nested_append_mismatch_root(ctx, &body[index..]) {
            push_root(roots, root);
        }
        collect_stmt(ctx, stmt, roots);
    }
}

/// Records a mismatched nested write in `stmt` and recurses into nested statement bodies.
fn collect_stmt(ctx: &LoweringContext<'_, '_>, stmt: &Stmt, roots: &mut Vec<String>) {
    match &stmt.kind {
        StmtKind::NestedArrayAssign { target, value } => {
            if let Some(root) = nested_assign_mismatch_root(ctx, target, value) {
                push_root(roots, root);
            }
        }
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            collect_body(ctx, then_body, roots);
            for (_, clause_body) in elseif_clauses {
                collect_body(ctx, clause_body, roots);
            }
            if let Some(else_body) = else_body {
                collect_body(ctx, else_body, roots);
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::Synthetic(body) => collect_body(ctx, body, roots),
        StmtKind::Switch { cases, default, .. } => {
            for (_, body) in cases {
                collect_body(ctx, body, roots);
            }
            if let Some(default) = default {
                collect_body(ctx, default, roots);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            collect_body(ctx, try_body, roots);
            for catch in catches {
                collect_body(ctx, &catch.body, roots);
            }
            if let Some(finally_body) = finally_body {
                collect_body(ctx, finally_body, roots);
            }
        }
        _ => {}
    }
}

/// Adds `root` once.
fn push_root(roots: &mut Vec<String>, root: String) {
    if !roots.contains(&root) {
        roots.push(root);
    }
}

/// Returns the root of `target = value` when the write cannot keep the root's concrete layout.
fn nested_assign_mismatch_root(
    ctx: &LoweringContext<'_, '_>,
    target: &Expr,
    value: &Expr,
) -> Option<String> {
    let (name, chain) = super::nested_local_chain(target)?;
    let leaf = concrete_chain_leaf_type(ctx, name, &chain)?;
    let Some(leaf) = leaf else {
        // A key the concrete chain cannot address (gradual or wrong-shaped key).
        return Some(name.to_string());
    };
    value_mismatches_leaf(&predicted_value_type(ctx, value), &leaf).then(|| name.to_string())
}

/// Recognizes `$t = $a[..][k]; $t[] = $v; $a[..][k] = $t;` whose appended value cannot be stored
/// in the addressed list's element type, returning the root of `$a`.
fn nested_append_mismatch_root(ctx: &LoweringContext<'_, '_>, window: &[Stmt]) -> Option<String> {
    let [read, push, write_back, ..] = window else {
        return None;
    };
    let StmtKind::Assign { name: temp, value: target } = &read.kind else {
        return None;
    };
    let StmtKind::ArrayPush { array, value } = &push.kind else {
        return None;
    };
    let StmtKind::NestedArrayAssign { value: stored, .. } = &write_back.kind else {
        return None;
    };
    if array != temp || !matches!(&stored.kind, ExprKind::Variable(name) if name == temp) {
        return None;
    }
    let (name, chain) = super::nested_local_chain(target)?;
    let Some(leaf) = concrete_chain_leaf_type(ctx, name, &chain)? else {
        return Some(name.to_string());
    };
    let element = match leaf {
        PhpType::Array(element) => element.codegen_repr(),
        // Appending to a non-list leaf is a keyed write of the value into that container.
        PhpType::AssocArray { value, .. } => value.codegen_repr(),
        _ => return None,
    };
    value_mismatches_leaf(&predicted_value_type(ctx, value), &element).then(|| name.to_string())
}

/// Walks a local's concrete chain type.
///
/// Returns `None` when the root is not a plain concrete container local (nothing to widen),
/// `Some(None)` when a key cannot address the concrete chain, and `Some(Some(leaf))` with the
/// element type the write stores into otherwise.
fn concrete_chain_leaf_type(
    ctx: &LoweringContext<'_, '_>,
    name: &str,
    chain: &[&Expr],
) -> Option<Option<PhpType>> {
    if !ctx.has_local_slot(name) || ctx.is_ref_bound_local(name) {
        return None;
    }
    let mut container = ctx.local_type(name).codegen_repr();
    let root_element = match &container {
        PhpType::Array(element) => element.codegen_repr(),
        PhpType::AssocArray { value, .. } => value.codegen_repr(),
        _ => return None,
    };
    if root_element == PhpType::Mixed || super::is_empty_indexed_array_element(&root_element) {
        return None;
    }
    for key in chain {
        let key_ty = super::nested_index_static_type(ctx, key);
        container = match container {
            PhpType::Array(element) if matches!(key_ty, PhpType::Int | PhpType::Bool) => {
                element.codegen_repr()
            }
            PhpType::AssocArray { value, .. }
                if matches!(key_ty, PhpType::Int | PhpType::Bool | PhpType::Str) =>
            {
                value.codegen_repr()
            }
            PhpType::Mixed => return None,
            _ => return Some(None),
        };
    }
    Some(Some(container))
}

/// Returns the type a written value lowers to when it is statically certain.
///
/// Only locals, literals, and container literals are predicted; arithmetic and calls can lower
/// to another representation than their syntactic type (checked arithmetic boxes), so they are
/// reported as unknown (`Never`) and never trigger a widening on their own.
fn predicted_value_type(ctx: &LoweringContext<'_, '_>, value: &Expr) -> PhpType {
    match &value.kind {
        ExprKind::Variable(name) => ctx.local_type(name).codegen_repr(),
        ExprKind::IntLiteral(_)
        | ExprKind::FloatLiteral(_)
        | ExprKind::StringLiteral(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::ArrayLiteral(_)
        | ExprKind::ArrayLiteralAssoc(_) => infer_expr_type_syntactic(value).codegen_repr(),
        // Concatenation (including desugared string interpolation) always yields a string.
        ExprKind::BinaryOp { op: BinOp::Concat, .. } => PhpType::Str,
        _ => PhpType::Never,
    }
}

/// Returns true when storing a value of `value` type changes the leaf's element layout.
fn value_mismatches_leaf(value: &PhpType, leaf: &PhpType) -> bool {
    if matches!(leaf, PhpType::Mixed) || matches!(value, PhpType::Never | PhpType::Void) || value == leaf {
        return false;
    }
    // A boxed value is cast into scalar slots by the store itself; container slots have no such
    // coercion, so a boxed value written there needs `mixed` elements.
    if matches!(value, PhpType::Mixed | PhpType::Union(_)) {
        return matches!(
            leaf,
            PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Object(_)
        );
    }
    if let (PhpType::Array(leaf_element), PhpType::Array(_)) = (leaf, value) {
        if super::is_empty_indexed_array_element(leaf_element) {
            return false;
        }
    }
    if let (PhpType::Array(_), PhpType::Array(value_element)) = (leaf, value) {
        // An empty literal (`[]`) is refined by later writes, not a layout change.
        if super::is_empty_indexed_array_element(value_element) {
            return false;
        }
    }
    true
}
