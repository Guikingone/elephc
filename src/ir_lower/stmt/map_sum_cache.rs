//! Purpose:
//! Caches a loop-invariant `array_sum(array_map($callback, $source))` result in a hidden local
//! before the loop, then reuses it each iteration.
//!
//! Called from:
//! - `crate::ir_lower::function::lower_body_into_function` before stored-map marking.
//!
//! Key details:
//! - The loop must be a `for` with a provably positive bound, so hoisting the map+sum cannot
//!   add work PHP would have skipped on a zero-trip loop.
//! - The callback must be a capture-free single-parameter closure whose body is one of the
//!   supported numeric map shapes, the source must be a plain `array<int>` local that is not
//!   written or aliased in the loop, and the mapped local must have no other use.
//! - The cached value is a compiler-generated `Mixed` local. Normal lowering owns, loads and
//!   releases it, and the frame epilogue releases the final value, so no ownership path is
//!   bypassed.
//! - Anything unproven leaves the original loop untouched.

use std::collections::HashSet;

use crate::names::Name;
use crate::parser::ast::{BinOp, Expr, ExprKind, Stmt, StmtKind};
use crate::types::{PhpType, TypeEnv};

use super::map_read_fusion::stmt_mentions;

/// Caches every loop-invariant numeric map sum in one function body.
pub(crate) fn cache_loop_invariant_map_sums(
    body: &mut Vec<Stmt>,
    env: &TypeEnv,
    by_ref_params: &HashSet<String>,
    aliased: &HashSet<String>,
) {
    let mut counter = 0usize;
    cache_block(body, env, by_ref_params, aliased, &mut counter);
}

/// Recurse into nested statement blocks and cache each supported loop.
fn cache_block(
    body: &mut Vec<Stmt>,
    env: &TypeEnv,
    by_ref_params: &HashSet<String>,
    aliased: &HashSet<String>,
    counter: &mut usize,
) {
    let mut index = 0;
    while index < body.len() {
        let original = body[index].clone();
        if let Some((cached, mapped, map_call_for_cached)) = try_cache_loop(
            &mut body[index],
            env,
            by_ref_params,
            aliased,
            *counter,
        ) {
            let used_elsewhere = body
                .iter()
                .enumerate()
                .any(|(other, stmt)| other != index && stmt_mentions(stmt, &mapped));
            if std::env::var_os("ELEPHC_DEBUG_MAP_CACHE").is_some() {
                eprintln!("map_sum_cache: candidate mapped={} used_elsewhere={}", mapped, used_elsewhere);
            }
            if used_elsewhere {
                body[index] = original;
            } else {
                let map_stmt = Stmt::new(
                    StmtKind::Assign {
                        name: mapped.clone(),
                        value: map_call_for_cached,
                    },
                    cached.span,
                );
                body.splice(index..index, [map_stmt, cached]);
                *counter += 1;
                index += 2;
            }
        } else {
            recurse_blocks(&mut body[index], env, by_ref_params, aliased, counter);
        }
        index += 1;
    }
}

/// Recurses into the blocks a statement owns without rewriting the statement itself.
fn recurse_blocks(
    stmt: &mut Stmt,
    env: &TypeEnv,
    by_ref_params: &HashSet<String>,
    aliased: &HashSet<String>,
    counter: &mut usize,
) {
    match &mut stmt.kind {
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            cache_block(then_body, env, by_ref_params, aliased, counter);
            for (_, body) in elseif_clauses {
                cache_block(body, env, by_ref_params, aliased, counter);
            }
            if let Some(body) = else_body {
                cache_block(body, env, by_ref_params, aliased, counter);
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            cache_block(then_body, env, by_ref_params, aliased, counter);
            if let Some(body) = else_body {
                cache_block(body, env, by_ref_params, aliased, counter);
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::Synthetic(body) => {
            cache_block(body, env, by_ref_params, aliased, counter);
        }
        StmtKind::Switch { cases, default, .. } => {
            for (_, body) in cases {
                cache_block(body, env, by_ref_params, aliased, counter);
            }
            if let Some(body) = default {
                cache_block(body, env, by_ref_params, aliased, counter);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            cache_block(try_body, env, by_ref_params, aliased, counter);
            for catch in catches {
                cache_block(&mut catch.body, env, by_ref_params, aliased, counter);
            }
            if let Some(body) = finally_body {
                cache_block(body, env, by_ref_params, aliased, counter);
            }
        }
        _ => {}
    }
}

/// Rewrites one `for` loop and returns the cached assignment to insert before it.
fn try_cache_loop(
    stmt: &mut Stmt,
    env: &TypeEnv,
    by_ref_params: &HashSet<String>,
    aliased: &HashSet<String>,
    counter: usize,
) -> Option<(Stmt, String, Expr)> {
    let StmtKind::For {
        init,
        condition,
        body,
        ..
    } = &mut stmt.kind
    else {
        return None;
    };
    if !positive_bound(init.as_deref(), condition.as_ref()?) {
        if std::env::var_os("ELEPHC_DEBUG_MAP_CACHE").is_some() {
            eprintln!("map_sum_cache: bound rejected init={:?}", init.as_ref().map(|s| std::mem::discriminant(&s.kind)));
        }
        return None;
    }
    let (mapped, map_call, sum_expr) = find_map_sum_pair(body, env, by_ref_params, aliased)?;
    // The mapped local may not be observed anywhere else in the function body. The caller's
    // whole block is the function body for the top-level loop, and `stmt_mentions` is
    // deliberately conservative for statement shapes it does not understand.
    let hidden = crate::names::generated_local_name(&format!("__elephc_cached_map_sum_{counter}"));
    let rewritten = replace_sum(sum_expr, &mapped, &hidden);
    if mentions_array_sum_of(&rewritten, &mapped) {
        return None;
    }
    let sum_stmt = body.get_mut(1)?;
    if let StmtKind::Assign { value, .. } = &mut sum_stmt.kind {
        *value = rewritten;
    } else {
        return None;
    }
    // The map assignment (index 0) is about to be removed; every remaining statement must be
    // free of the mapped local after the sum expression was rewritten.
    if body.iter().skip(1).any(|stmt| stmt_mentions(stmt, &mapped)) {
        return None;
    }
    body.remove(0);
    let cached = Stmt::new(
        StmtKind::Assign {
            name: hidden,
            value: Expr::new(
                ExprKind::FunctionCall {
                    name: Name::unqualified("array_sum"),
                    args: vec![Expr::new(
                        ExprKind::Variable(mapped.clone()),
                        stmt.span,
                    )],
                },
                stmt.span,
            ),
        },
        stmt.span,
    );
    Some((cached, mapped, map_call))
}

/// Finds the adjacent `$mapped = array_map(...)` and `$sum = ... array_sum($mapped)` pair.
fn find_map_sum_pair(
    body: &[Stmt],
    env: &TypeEnv,
    by_ref_params: &HashSet<String>,
    aliased: &HashSet<String>,
) -> Option<(String, Expr, Expr)> {
    if body.len() != 2 {
        return None;
    }
    let StmtKind::Assign {
        name: mapped,
        value: map_call,
    } = &body[0].kind
    else {
        return None;
    };
    let ExprKind::FunctionCall { name: map_name, args } = &map_call.kind else {
        return None;
    };
    if map_name.as_str().trim_start_matches('\\') != "array_map" || args.len() != 2 {
        return None;
    }
    let ExprKind::Variable(source) = &args[1].kind else {
        return None;
    };
    if aliased.contains(source) || by_ref_params.contains(source) {
        return None;
    }
    if env.get(source).map(PhpType::codegen_repr) != Some(PhpType::Array(Box::new(PhpType::Int))) {
        return None;
    }
    if !pure_numeric_callback(&args[0]) {
        return None;
    }
    if body.iter().any(|stmt| stmt_writes_local(stmt, source)) {
        return None;
    }
    let StmtKind::Assign { value: sum_expr, .. } = &body[1].kind else {
        return None;
    };
    if !mentions_array_sum_of(sum_expr, mapped) {
        return None;
    }
    Some((mapped.clone(), map_call.clone(), sum_expr.clone()))
}

/// Returns whether `expr` is a supported capture-free numeric map callback.
fn pure_numeric_callback(expr: &Expr) -> bool {
    let ExprKind::Closure {
        params,
        variadic,
        body,
        captures,
        capture_refs,
        ..
    } = &expr.kind
    else {
        return false;
    };
    if params.len() != 1
        || variadic.is_some()
        || !captures.is_empty()
        || !capture_refs.is_empty()
    {
        return false;
    }
    let [Stmt {
        kind: StmtKind::Return(Some(result)),
        ..
    }] = body.as_slice()
    else {
        return false;
    };
    super::stored_numeric_map::parse_stored_numeric_map(result, &params[0].0).is_some()
}

/// Returns whether `expr` contains `array_sum($mapped)` in one of the supported positions.
fn mentions_array_sum_of(expr: &Expr, mapped: &str) -> bool {
    match &expr.kind {
        ExprKind::FunctionCall { name, args } => {
            name.as_str().trim_start_matches('\\') == "array_sum"
                && matches!(args.as_slice(), [Expr { kind: ExprKind::Variable(source), .. }] if source == mapped)
        }
        ExprKind::BinaryOp { left, right, .. } => {
            mentions_array_sum_of(left, mapped) || mentions_array_sum_of(right, mapped)
        }
        _ => false,
    }
}

/// Replaces every `array_sum($mapped)` call in the supported expression positions.
fn replace_sum(expr: Expr, mapped: &str, hidden: &str) -> Expr {
    match expr.kind {
        ExprKind::FunctionCall { name, args }
            if name.as_str().trim_start_matches('\\') == "array_sum"
                && matches!(args.as_slice(), [Expr { kind: ExprKind::Variable(source), .. }] if source == mapped) =>
        {
            Expr::new(ExprKind::Variable(hidden.to_string()), expr.span)
        }
        ExprKind::BinaryOp { left, op, right } => Expr::new(
            ExprKind::BinaryOp {
                left: Box::new(replace_sum(*left, mapped, hidden)),
                op,
                right: Box::new(replace_sum(*right, mapped, hidden)),
            },
            expr.span,
        ),
        _ => expr,
    }
}

/// Returns whether a `for` loop starts at a non-negative counter and has a positive bound.
fn positive_bound(init: Option<&Stmt>, condition: &Expr) -> bool {
    let Some(Stmt {
        kind: StmtKind::Assign { name: counter, value },
        ..
    }) = init
    else {
        return false;
    };
    if !matches!(value.kind, ExprKind::IntLiteral(seed) if seed >= 0) {
        return false;
    }
    let ExprKind::BinaryOp {
        left,
        op: BinOp::Lt | BinOp::LtEq,
        right,
    } = &condition.kind
    else {
        return false;
    };
    if !matches!(&left.kind, ExprKind::Variable(name) if name == counter) {
        return false;
    }
    positive_expr(right)
}

/// Returns whether an expression is a positive runtime bound independent of loop progress.
fn positive_expr(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::IntLiteral(value) => *value > 0,
        ExprKind::BinaryOp {
            left,
            op: BinOp::Add,
            right,
        } => match (&left.kind, &right.kind) {
            (ExprKind::IntLiteral(value), ExprKind::Variable(name))
            | (ExprKind::Variable(name), ExprKind::IntLiteral(value))
                if name == "argc" =>
            {
                *value >= 0
            }
            _ => false,
        },
        _ => false,
    }
}

/// Writes to one local in a statement, conservatively rejecting unknown statement shapes.
fn stmt_writes_local(stmt: &Stmt, name: &str) -> bool {
    match &stmt.kind {
        StmtKind::Assign { name: target, .. } | StmtKind::TypedAssign { name: target, .. } => {
            target == name
        }
        StmtKind::ArrayAssign { array, .. } | StmtKind::ArrayPush { array, .. } => array == name,
        StmtKind::RefAssign { target, .. } => target == name,
        StmtKind::Foreach {
            value_var,
            value_by_ref,
            ..
        } => *value_by_ref && value_var == name,
        StmtKind::Global { vars } => vars.iter().any(|var| var == name),
        StmtKind::StaticVar { name: static_name, .. } => static_name == name,
        StmtKind::Synthetic(body)
        | StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::NamespaceBlock { body, .. } => body.iter().any(|stmt| stmt_writes_local(stmt, name)),
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            then_body.iter().any(|stmt| stmt_writes_local(stmt, name))
                || elseif_clauses
                    .iter()
                    .any(|(_, body)| body.iter().any(|stmt| stmt_writes_local(stmt, name)))
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(|stmt| stmt_writes_local(stmt, name)))
        }
        StmtKind::Switch { cases, default, .. } => {
            cases
                .iter()
                .any(|(_, body)| body.iter().any(|stmt| stmt_writes_local(stmt, name)))
                || default
                    .as_ref()
                    .is_some_and(|body| body.iter().any(|stmt| stmt_writes_local(stmt, name)))
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            try_body.iter().any(|stmt| stmt_writes_local(stmt, name))
                || catches
                    .iter()
                    .any(|catch| catch.body.iter().any(|stmt| stmt_writes_local(stmt, name)))
                || finally_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(|stmt| stmt_writes_local(stmt, name)))
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            then_body.iter().any(|stmt| stmt_writes_local(stmt, name))
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(|stmt| stmt_writes_local(stmt, name)))
        }
        StmtKind::NestedArrayAssign { .. } => true,
        StmtKind::IncludeOnceMark { .. } => false,
        StmtKind::Echo(_)
        | StmtKind::Return(_)
        | StmtKind::Break(_)
        | StmtKind::Continue(_)
        | StmtKind::Include { .. }
        | StmtKind::Throw(_)
        | StmtKind::ExprStmt(_)
        | StmtKind::ConstDecl { .. }
        | StmtKind::ListUnpack { .. }
        | StmtKind::NamespaceDecl { .. }
        | StmtKind::UseDecl { .. }
        | StmtKind::FunctionDecl { .. }
        | StmtKind::FunctionVariantGroup { .. }
        | StmtKind::FunctionVariantMark { .. }
        | StmtKind::ClassDecl { .. }
        | StmtKind::EnumDecl { .. }
        | StmtKind::PackedClassDecl { .. }
        | StmtKind::InterfaceDecl { .. }
        | StmtKind::TraitDecl { .. }
        | StmtKind::PropertyAssign { .. }
        | StmtKind::StaticPropertyAssign { .. }
        | StmtKind::StaticPropertyArrayPush { .. }
        | StmtKind::StaticPropertyArrayAssign { .. }
        | StmtKind::PropertyArrayPush { .. }
        | StmtKind::PropertyArrayAssign { .. }
        | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. }
        | StmtKind::ExternGlobalDecl { .. } => true,
    }
}
