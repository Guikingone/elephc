//! Purpose:
//! Prunes constant control-flow loop exit cases.
//! Rewrites statements or expressions whose compile-time condition is known while preserving required effects.
//!
//! Called from:
//! - `crate::optimize::control::prune`
//!
//! Key details:
//! - Loop exits, empty bodies, and effectful conditions must be handled before removing structural statements.

use super::super::*;

/// Returns true if the statement list contains a break or continue that reaches the loop whose
/// body it is, considering nested ifdef, if, try, switch and synthetic blocks, and the `break N` /
/// `continue N` inside nested loops that climb past them.
pub(crate) fn block_contains_loop_exit(body: &[Stmt]) -> bool {
    block_exits_past(body, 0)
}

/// Returns true when a statement of `body`, which sits inside `depth` loops nested in the loop
/// being asked about, breaks or continues out of that loop.
///
/// A `goto` lowered to a guarded segment is `$sel = i; break <n>;` inside a `Synthetic` block, and
/// from inside an inner `foreach` it is `break 2`. Missing either made a `do { } while (false)`
/// segment wrapper look exit-free, so it was inlined and its `break` left without a loop: the
/// function stopped at the first taken `goto`.
fn block_exits_past(body: &[Stmt], depth: usize) -> bool {
    body.iter().any(|stmt| stmt_exits_past(stmt, depth))
}

/// Statement form of [`block_exits_past`].
fn stmt_exits_past(stmt: &Stmt, depth: usize) -> bool {
    match &stmt.kind {
        StmtKind::Break(levels) | StmtKind::Continue(levels) => *levels > depth,
        StmtKind::Synthetic(stmts) => block_exits_past(stmts, depth),
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Foreach { body, .. } => block_exits_past(body, depth + 1),
        _ if depth > 0 => nested_blocks_exit_past(stmt, depth),
        _ => stmt_contains_loop_exit(stmt),
    }
}

/// Recurses into the non-loop blocks of `stmt` at the same loop depth.
fn nested_blocks_exit_past(stmt: &Stmt, depth: usize) -> bool {
    match &stmt.kind {
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            block_exits_past(then_body, depth)
                || elseif_clauses
                    .iter()
                    .any(|(_, body)| block_exits_past(body, depth))
                || else_body
                    .as_ref()
                    .is_some_and(|body| block_exits_past(body, depth))
        }
        StmtKind::IfDef {
            then_body, else_body, ..
        } => {
            block_exits_past(then_body, depth)
                || else_body
                    .as_ref()
                    .is_some_and(|body| block_exits_past(body, depth))
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            block_exits_past(try_body, depth)
                || catches
                    .iter()
                    .any(|catch| block_exits_past(&catch.body, depth))
                || finally_body
                    .as_ref()
                    .is_some_and(|body| block_exits_past(body, depth))
        }
        // A `switch` is a break level too; counting it as one more keeps `break` inside it local.
        StmtKind::Switch { cases, default, .. } => {
            cases.iter().any(|(_, body)| block_exits_past(body, depth + 1))
                || default
                    .as_ref()
                    .is_some_and(|body| block_exits_past(body, depth + 1))
        }
        _ => false,
    }
}

/// Returns true if the statement contains a break or continue at any nesting
/// level, recursing into ifdef, if, try, and switch statement bodies.
fn stmt_contains_loop_exit(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::Break(_) | StmtKind::Continue(_) => true,
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            block_contains_loop_exit(then_body)
                || elseif_clauses
                    .iter()
                    .any(|(_, body)| block_contains_loop_exit(body))
                || else_body
                    .as_ref()
                    .is_some_and(|body| block_contains_loop_exit(body))
        }
        StmtKind::IfDef {
            then_body, else_body, ..
        } => {
            block_contains_loop_exit(then_body)
                || else_body
                    .as_ref()
                    .is_some_and(|body| block_contains_loop_exit(body))
        }
        StmtKind::Synthetic(stmts) => block_contains_loop_exit(stmts),
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Foreach { body, .. } => block_exits_past(body, 1),
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            block_contains_loop_exit(try_body)
                || catches
                    .iter()
                    .any(|catch| block_contains_loop_exit(&catch.body))
                || finally_body
                    .as_ref()
                    .is_some_and(|body| block_contains_loop_exit(body))
        }
        StmtKind::Switch { cases, default, .. } => {
            cases
                .iter()
                .any(|(_, body)| block_contains_loop_exit(body))
                || default
                    .as_ref()
                    .is_some_and(|body| block_contains_loop_exit(body))
        }
        _ => false,
    }
}
