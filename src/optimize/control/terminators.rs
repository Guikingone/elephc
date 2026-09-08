//! Purpose:
//! Removes redundant trailing terminators from control shells during control-flow normalization:
//! a `continue` that ends a loop body, a `break` that ends the last `switch` body, and a bare
//! `return;` that ends a function body all transfer to exactly where falling off the block would.
//! Also answers the level-aware question the loop rotation needs: whether a body `continue`s
//! the loop around it.
//!
//! Called from:
//! - `crate::optimize::control::loops` for loop bodies.
//! - `crate::optimize::control::switch::prune_switch_stmt()` for the final `switch` body.
//! - `crate::optimize::control::prune::statements` for function and method bodies.
//!
//! Key details:
//! - Only the tail path is inspected: the last statement, and recursively the last statement of
//!   each `if` / `ifdef` / `try` branch that ends there. A terminator followed by more code is never touched.
//! - Recursion never enters loops or `switch` bodies, which retarget `break` / `continue`, nor
//!   `finally` bodies, so a stripped terminator always meant "leave this shell normally".
//! - Every rewrite moves nodes; nothing is cloned, so span-keyed checker decisions stay singular.

use super::*;

/// The terminator kind a tail-stripping walk removes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TailTerminator {
    /// `continue;` (level 1) at the end of a loop body.
    LoopContinue,
    /// `break;` (level 1) at the end of the body that runs last in a `switch`.
    SwitchBreak,
    /// `return;` without a value at the end of a function body.
    FunctionReturn,
}

/// Removes `terminator` from the tail path of `body`, re-normalizing any `if` / `ifdef` / `try`
/// shell whose branches changed so that emptied shells collapse the same way fresh input would.
pub(crate) fn strip_trailing_terminator(body: Vec<Stmt>, terminator: TailTerminator) -> Vec<Stmt> {
    strip_trailing_terminator_tracked(body, terminator).0
}

/// Returns whether the tail path of `body` carries `terminator`, without rewriting anything.
pub(crate) fn tail_carries_terminator(body: &[Stmt], terminator: TailTerminator) -> bool {
    let Some(last) = body.last() else {
        return false;
    };
    if is_terminator(last, terminator) {
        return true;
    }
    match &last.kind {
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            tail_carries_terminator(then_body, terminator)
                || elseif_clauses
                    .iter()
                    .any(|(_, body)| tail_carries_terminator(body, terminator))
                || else_body
                    .as_ref()
                    .is_some_and(|body| tail_carries_terminator(body, terminator))
        }
        StmtKind::IfDef {
            then_body, else_body, ..
        } => {
            tail_carries_terminator(then_body, terminator)
                || else_body
                    .as_ref()
                    .is_some_and(|body| tail_carries_terminator(body, terminator))
        }
        StmtKind::Try {
            try_body, catches, ..
        } => {
            tail_carries_terminator(try_body, terminator)
                || catches
                    .iter()
                    .any(|catch| tail_carries_terminator(&catch.body, terminator))
        }
        _ => false,
    }
}

/// Returns whether `stmt` is exactly the statement `terminator` names.
fn is_terminator(stmt: &Stmt, terminator: TailTerminator) -> bool {
    matches!(
        (&stmt.kind, terminator),
        (StmtKind::Continue(1), TailTerminator::LoopContinue)
            | (StmtKind::Break(1), TailTerminator::SwitchBreak)
            | (StmtKind::Return(None), TailTerminator::FunctionReturn)
    )
}

/// Strips the tail terminator from `body`, reporting whether anything was removed.
fn strip_trailing_terminator_tracked(mut body: Vec<Stmt>, terminator: TailTerminator) -> (Vec<Stmt>, bool) {
    let Some(last) = body.pop() else {
        return (body, false);
    };
    let (rewritten, changed) = strip_terminator_from_tail_stmt(last, terminator);
    body.extend(rewritten);
    (body, changed)
}

/// Strips the tail terminator from the last statement of a block. A matching terminator
/// disappears; an `if` / `ifdef` / `try` shell is stripped branch by branch and, when a branch
/// changed, re-pruned so the shell takes its canonical shape; anything else is left alone.
fn strip_terminator_from_tail_stmt(stmt: Stmt, terminator: TailTerminator) -> (Vec<Stmt>, bool) {
    if is_terminator(&stmt, terminator) {
        return (Vec::new(), true);
    }
    let span = stmt.span;
    let source_mode = stmt.source_mode;
    let strict_types = stmt.strict_types;
    let attributes = stmt.attributes;
    let rebuild = |kind: StmtKind| Stmt {
        kind,
        span,
        source_mode,
        strict_types,
        attributes,
    };
    match stmt.kind {
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            let (then_body, mut changed) = strip_trailing_terminator_tracked(then_body, terminator);
            let elseif_clauses: Vec<_> = elseif_clauses
                .into_iter()
                .map(|(condition, body)| {
                    let (body, body_changed) = strip_trailing_terminator_tracked(body, terminator);
                    changed |= body_changed;
                    (condition, body)
                })
                .collect();
            let else_body = else_body.map(|body| {
                let (body, body_changed) = strip_trailing_terminator_tracked(body, terminator);
                changed |= body_changed;
                body
            });
            let stmt = rebuild(StmtKind::If {
                condition,
                then_body,
                elseif_clauses,
                else_body,
            });
            if changed {
                (prune_stmt(stmt), true)
            } else {
                (vec![stmt], false)
            }
        }
        StmtKind::IfDef {
            symbol,
            then_body,
            else_body,
        } => {
            let (then_body, mut changed) = strip_trailing_terminator_tracked(then_body, terminator);
            let else_body = else_body.map(|body| {
                let (body, body_changed) = strip_trailing_terminator_tracked(body, terminator);
                changed |= body_changed;
                body
            });
            let stmt = rebuild(StmtKind::IfDef {
                symbol,
                then_body,
                else_body,
            });
            if changed {
                (prune_stmt(stmt), true)
            } else {
                (vec![stmt], false)
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            let (try_body, mut changed) = strip_trailing_terminator_tracked(try_body, terminator);
            let catches: Vec<_> = catches
                .into_iter()
                .map(|catch| {
                    let (body, body_changed) = strip_trailing_terminator_tracked(catch.body, terminator);
                    changed |= body_changed;
                    crate::parser::ast::CatchClause { body, ..catch }
                })
                .collect();
            let stmt = rebuild(StmtKind::Try {
                try_body,
                catches,
                finally_body,
            });
            if changed {
                (prune_stmt(stmt), true)
            } else {
                (vec![stmt], false)
            }
        }
        kind => (vec![rebuild(kind)], false),
    }
}

/// Returns whether `body` contains a `continue` that targets the loop `depth` constructs above
/// it: `depth` is the number of loop / `switch` shells between `body` and that loop, so a direct
/// loop body asks with `0`. Deeper `continue` levels leave that loop and are reported as `false`.
pub(crate) fn block_continues_enclosing_loop(body: &[Stmt], depth: usize) -> bool {
    body.iter()
        .any(|stmt| stmt_continues_enclosing_loop(stmt, depth))
}

/// Statement-level walker for `block_continues_enclosing_loop`. Loops and `switch` bodies raise
/// the depth because PHP counts both as `continue` targets; declarations and expressions are
/// opaque because a `continue` inside a closure body belongs to that closure.
fn stmt_continues_enclosing_loop(stmt: &Stmt, depth: usize) -> bool {
    match &stmt.kind {
        StmtKind::Continue(levels) => *levels == depth + 1,
        StmtKind::Synthetic(stmts) | StmtKind::IncludeOnceGuard { body: stmts, .. } => {
            block_continues_enclosing_loop(stmts, depth)
        }
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            block_continues_enclosing_loop(then_body, depth)
                || elseif_clauses
                    .iter()
                    .any(|(_, body)| block_continues_enclosing_loop(body, depth))
                || else_body
                    .as_ref()
                    .is_some_and(|body| block_continues_enclosing_loop(body, depth))
        }
        StmtKind::IfDef {
            then_body, else_body, ..
        } => {
            block_continues_enclosing_loop(then_body, depth)
                || else_body
                    .as_ref()
                    .is_some_and(|body| block_continues_enclosing_loop(body, depth))
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            block_continues_enclosing_loop(try_body, depth)
                || catches
                    .iter()
                    .any(|catch| block_continues_enclosing_loop(&catch.body, depth))
                || finally_body
                    .as_ref()
                    .is_some_and(|body| block_continues_enclosing_loop(body, depth))
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Foreach { body, .. } => block_continues_enclosing_loop(body, depth + 1),
        StmtKind::Switch { cases, default, .. } => {
            cases
                .iter()
                .any(|(_, body)| block_continues_enclosing_loop(body, depth + 1))
                || default
                    .as_ref()
                    .is_some_and(|body| block_continues_enclosing_loop(body, depth + 1))
        }
        _ => false,
    }
}
