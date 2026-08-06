//! Purpose:
//! Models PHP's "a `return` at an included file's top level returns from the INCLUDE" rule.
//! Normalizes early-return dispatch shapes and eliminates scope returns into a temp + flag.
//!
//! Called from:
//! - `crate::resolver::files::parse_file` (normalization) and `crate::resolver::engine_includes`.
//!
//! Key details:
//! - An included file's top-level scope spans its control flow but stops at function/class bodies.

use crate::parser::ast::{CatchClause, Expr, ExprKind, Stmt, StmtKind};
use crate::span::Span;

/// Outcome of rewriting an included file's top-level `return`s into assignments to the hidden
/// include temporary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum IncludeReturnRewrite {
    /// The body has no top-level `return`, so the temporary keeps PHP's `int(1)` default.
    Absent,
    /// A `return` that is a DIRECT element of the body's top-level statement list was rewritten.
    /// Every path through the body reaches it, so the temporary needs no default seed.
    Unconditional,
    /// A `return` nested inside the body's control flow was rewritten. The temporary may stay at
    /// its default, and the rewritten body reads the "already returned" flag, which the caller
    /// must seed to `false` BEFORE the body (outside any include-once guard).
    Conditional,
}

/// Rewrites `if (C) { … return; } <tail>` into `if (C) { … return; } else { <tail> }` throughout an
/// included file's top-level scope.
///
/// # Why
///
/// PHP's own control flow makes `<tail>` reachable only when `C` is false — the two regions are
/// mutually exclusive. elephc's include machinery, however, flattens a file's declarations
/// statically, and its exclusivity bookkeeping (`discovery::branches::merge_alternatives`) keys off
/// SYNTACTIC if/else alternatives. Without this normalization the dispatch idiom every Symfony
/// polyfill bootstrap uses —
///
/// ```php
/// if (\PHP_VERSION_ID >= 80000) { return require __DIR__.'/bootstrap80.php'; }
/// return require __DIR__.'/bootstrap72.php';
/// ```
///
/// — flattens BOTH targets as if they ran in sequence, and the same `mb_*` function declared by
/// each one collides ("Duplicate function declaration"). Making the exclusivity syntactic lets the
/// existing alternative-merging handle it, with no change to how exclusivity is represented.
///
/// The rewrite is semantics-preserving for any PHP program, and only applies when the `if` has no
/// `else` and EVERY one of its branches terminates: otherwise `<tail>` is genuinely reachable
/// through a branch and must stay sequential.
pub(super) fn normalize_early_returns(stmts: Vec<Stmt>) -> Vec<Stmt> {
    normalize_list(stmts)
}

/// Applies the early-return normalization to one statement list, then recurses into the control
/// flow of every statement that stays in the include's own scope.
fn normalize_list(stmts: Vec<Stmt>) -> Vec<Stmt> {
    let mut stmts: Vec<Stmt> = stmts.into_iter().map(normalize_stmt).collect();

    for i in 0..stmts.len() {
        if i + 1 >= stmts.len() || !adopts_tail_as_else(&stmts[i]) {
            continue;
        }
        let tail = stmts.split_off(i + 1);
        let StmtKind::If { else_body, .. } = &mut stmts[i].kind else {
            unreachable!("adopts_tail_as_else matched a non-If statement");
        };
        *else_body = Some(tail);
        break;
    }

    stmts
}

/// Returns whether `stmt` is an `else`-less `if` whose every branch terminates, so the statements
/// following it in the enclosing list can be adopted as its `else` body.
fn adopts_tail_as_else(stmt: &Stmt) -> bool {
    let StmtKind::If {
        then_body,
        elseif_clauses,
        else_body: None,
        ..
    } = &stmt.kind
    else {
        return false;
    };
    list_terminates(then_body) && elseif_clauses.iter().all(|(_, body)| list_terminates(body))
}

/// Returns whether every path through `stmts` leaves the enclosing scope, so control never falls
/// out of the list's end. Deliberately conservative: only the shapes that unambiguously diverge.
fn list_terminates(stmts: &[Stmt]) -> bool {
    let Some(last) = stmts.last() else {
        return false;
    };
    match &last.kind {
        StmtKind::Return(_) | StmtKind::Throw(_) => true,
        StmtKind::Synthetic(body) => list_terminates(body),
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body: Some(else_body),
            ..
        } => {
            list_terminates(then_body)
                && elseif_clauses.iter().all(|(_, body)| list_terminates(body))
                && list_terminates(else_body)
        }
        _ => false,
    }
}

/// Rebuilds one statement with its in-scope bodies normalized. Function, class, interface, trait
/// and enum bodies are separate `return` scopes and are left untouched.
fn normalize_stmt(stmt: Stmt) -> Stmt {
    let Stmt {
        kind,
        span,
        attributes,
    } = stmt;
    let kind = match kind {
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => StmtKind::If {
            condition,
            then_body: normalize_list(then_body),
            elseif_clauses: elseif_clauses
                .into_iter()
                .map(|(condition, body)| (condition, normalize_list(body)))
                .collect(),
            else_body: else_body.map(normalize_list),
        },
        StmtKind::IfDef {
            symbol,
            then_body,
            else_body,
        } => StmtKind::IfDef {
            symbol,
            then_body: normalize_list(then_body),
            else_body: else_body.map(normalize_list),
        },
        StmtKind::While { condition, body } => StmtKind::While {
            condition,
            body: normalize_list(body),
        },
        StmtKind::DoWhile { body, condition } => StmtKind::DoWhile {
            body: normalize_list(body),
            condition,
        },
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => StmtKind::For {
            init,
            condition,
            update,
            body: normalize_list(body),
        },
        StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body,
        } => StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body: normalize_list(body),
        },
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => StmtKind::Switch {
            subject,
            cases: cases
                .into_iter()
                .map(|(values, body)| (values, normalize_list(body)))
                .collect(),
            default: default.map(normalize_list),
        },
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => StmtKind::Try {
            try_body: normalize_list(try_body),
            catches: catches
                .into_iter()
                .map(|clause| CatchClause {
                    body: normalize_list(clause.body),
                    ..clause
                })
                .collect(),
            finally_body: finally_body.map(normalize_list),
        },
        StmtKind::Synthetic(body) => StmtKind::Synthetic(normalize_list(body)),
        StmtKind::NamespaceBlock { name, body } => StmtKind::NamespaceBlock {
            name,
            body: normalize_list(body),
        },
        StmtKind::IncludeOnceGuard { label, body } => StmtKind::IncludeOnceGuard {
            label,
            body: normalize_list(body),
        },
        other => other,
    };
    Stmt {
        kind,
        span,
        attributes,
    }
}

/// Rewrites the included file's top-level `return`s to assign the include temporary `temp`,
/// reporting whether the temporary ends up assigned on every path.
///
/// A `return` that is a direct element of the top-level statement list keeps the cheap shape:
/// `<temp> = E;` with the now-unreachable tail dropped, and no flag variable at all. As soon as a
/// `return` sits inside the file's control flow (`if (…) { return …; }`), the general elimination
/// below runs instead, because whether the file returned is then a RUNTIME fact.
pub(super) fn rewrite_scope_returns(
    body: &mut Vec<Stmt>,
    temp: &str,
    flag: &str,
) -> IncludeReturnRewrite {
    let first_direct = body
        .iter()
        .position(|stmt| matches!(stmt.kind, StmtKind::Return(_)));
    let first_nested = body.iter().position(stmt_nests_scope_return);

    match (first_direct, first_nested) {
        (None, None) => IncludeReturnRewrite::Absent,
        (Some(direct), nested) if nested.is_none_or(|nested| direct < nested) => {
            capture_direct_return(body, direct, temp);
            IncludeReturnRewrite::Unconditional
        }
        _ => {
            let eliminated = eliminate_returns_in_list(std::mem::take(body), temp, flag);
            *body = eliminated;
            IncludeReturnRewrite::Conditional
        }
    }
}

/// Replaces `body[index]` — a top-level `return E;` — with `<temp> = E;` (or an empty sequence for
/// a bare `return;`, leaving the temporary at its default) and truncates the unreachable tail.
fn capture_direct_return(body: &mut Vec<Stmt>, index: usize, temp: &str) {
    let span = body[index].span;
    let placeholder = Stmt::new(StmtKind::Return(None), span);
    let original = std::mem::replace(&mut body[index], placeholder);
    body[index] = match original.kind {
        StmtKind::Return(Some(value)) => assign_temp(temp, value, span),
        _ => Stmt::new(StmtKind::Synthetic(Vec::new()), span),
    };
    body.truncate(index + 1);
}

/// Eliminates every in-scope `return` from a statement list.
///
/// Each `return E;` becomes `<temp> = E; <flag> = true;`, and the statements that follow the first
/// statement that MAY have returned are wrapped in `if (!<flag>) { … }` so they are skipped once
/// the file has returned. Statements after an unconditional `return` are dropped outright.
fn eliminate_returns_in_list(mut stmts: Vec<Stmt>, temp: &str, flag: &str) -> Vec<Stmt> {
    let Some(index) = stmts
        .iter()
        .position(|stmt| matches!(stmt.kind, StmtKind::Return(_)) || stmt_nests_scope_return(stmt))
    else {
        return stmts;
    };

    let tail = stmts.split_off(index + 1);
    let stmt = stmts.pop().expect("split index is within the list");
    let span = stmt.span;

    if let StmtKind::Return(value) = stmt.kind {
        if let Some(value) = value {
            stmts.push(assign_temp(temp, value, span));
        }
        stmts.push(assign_flag(flag, true, span));
        // `tail` is unreachable: the file has returned on every path reaching here.
        return stmts;
    }

    stmts.push(eliminate_returns_in_stmt(stmt, temp, flag));
    if !tail.is_empty() {
        stmts.push(Stmt::new(
            StmtKind::If {
                condition: flag_is_clear(flag, span),
                then_body: eliminate_returns_in_list(tail, temp, flag),
                elseif_clauses: Vec::new(),
                else_body: None,
            },
            span,
        ));
    }
    stmts
}

/// Rebuilds a compound statement with every in-scope body return-eliminated.
fn eliminate_returns_in_stmt(stmt: Stmt, temp: &str, flag: &str) -> Stmt {
    let Stmt {
        kind,
        span,
        attributes,
    } = stmt;
    let kind = match kind {
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => StmtKind::If {
            condition,
            then_body: eliminate_returns_in_list(then_body, temp, flag),
            elseif_clauses: elseif_clauses
                .into_iter()
                .map(|(condition, body)| (condition, eliminate_returns_in_list(body, temp, flag)))
                .collect(),
            else_body: else_body.map(|body| eliminate_returns_in_list(body, temp, flag)),
        },
        StmtKind::IfDef {
            symbol,
            then_body,
            else_body,
        } => StmtKind::IfDef {
            symbol,
            then_body: eliminate_returns_in_list(then_body, temp, flag),
            else_body: else_body.map(|body| eliminate_returns_in_list(body, temp, flag)),
        },
        StmtKind::While { condition, body } => StmtKind::While {
            condition,
            body: eliminate_returns_in_loop_body(body, temp, flag),
        },
        StmtKind::DoWhile { body, condition } => StmtKind::DoWhile {
            body: eliminate_returns_in_loop_body(body, temp, flag),
            condition,
        },
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => StmtKind::For {
            init,
            condition,
            update,
            body: eliminate_returns_in_loop_body(body, temp, flag),
        },
        StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body,
        } => StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body: eliminate_returns_in_loop_body(body, temp, flag),
        },
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => StmtKind::Switch {
            subject,
            cases: cases
                .into_iter()
                .map(|(values, body)| {
                    (values, eliminate_returns_in_loop_body(body, temp, flag))
                })
                .collect(),
            default: default.map(|body| eliminate_returns_in_loop_body(body, temp, flag)),
        },
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => StmtKind::Try {
            try_body: eliminate_returns_in_list(try_body, temp, flag),
            catches: catches
                .into_iter()
                .map(|clause| CatchClause {
                    body: eliminate_returns_in_list(clause.body, temp, flag),
                    ..clause
                })
                .collect(),
            finally_body: finally_body.map(|body| eliminate_returns_in_list(body, temp, flag)),
        },
        StmtKind::Synthetic(body) => {
            StmtKind::Synthetic(eliminate_returns_in_list(body, temp, flag))
        }
        StmtKind::NamespaceBlock { name, body } => StmtKind::NamespaceBlock {
            name,
            body: eliminate_returns_in_list(body, temp, flag),
        },
        StmtKind::IncludeOnceGuard { label, body } => StmtKind::IncludeOnceGuard {
            label,
            body: eliminate_returns_in_list(body, temp, flag),
        },
        other => other,
    };
    Stmt {
        kind,
        span,
        attributes,
    }
}

/// Return-eliminates a loop or `switch`-case body and appends `if (<flag>) { break; }`.
///
/// The trailing break is what stops the construct once the file has returned: the enclosing list's
/// `if (!<flag>)` guard only skips the statements AFTER the loop, so without it the loop would keep
/// iterating (or the `switch` would fall through) with the return already taken.
fn eliminate_returns_in_loop_body(body: Vec<Stmt>, temp: &str, flag: &str) -> Vec<Stmt> {
    let span = body.first().map_or_else(Span::dummy, |stmt| stmt.span);
    let mut body = eliminate_returns_in_list(body, temp, flag);
    body.push(Stmt::new(
        StmtKind::If {
            condition: Expr::new(ExprKind::Variable(flag.to_string()), span),
            then_body: vec![Stmt::new(StmtKind::Break(1), span)],
            elseif_clauses: Vec::new(),
            else_body: None,
        },
        span,
    ));
    body
}

/// Returns whether `stmt` nests an in-scope `return` inside its own control flow. A statement that
/// IS a `return` reports `false`: its caller handles the direct case, which needs no flag.
fn stmt_nests_scope_return(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            list_has_scope_return(then_body)
                || elseif_clauses
                    .iter()
                    .any(|(_, body)| list_has_scope_return(body))
                || else_body.as_deref().is_some_and(list_has_scope_return)
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            list_has_scope_return(then_body)
                || else_body.as_deref().is_some_and(list_has_scope_return)
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. } => list_has_scope_return(body),
        StmtKind::Switch { cases, default, .. } => {
            cases.iter().any(|(_, body)| list_has_scope_return(body))
                || default.as_deref().is_some_and(list_has_scope_return)
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            list_has_scope_return(try_body)
                || catches
                    .iter()
                    .any(|clause| list_has_scope_return(&clause.body))
                || finally_body.as_deref().is_some_and(list_has_scope_return)
        }
        _ => false,
    }
}

/// Returns whether `stmts` contains a `return` belonging to the include's own scope.
fn list_has_scope_return(stmts: &[Stmt]) -> bool {
    stmts
        .iter()
        .any(|stmt| matches!(stmt.kind, StmtKind::Return(_)) || stmt_nests_scope_return(stmt))
}

/// Builds `<temp> = <value>;` for the hidden include temporary.
fn assign_temp(temp: &str, value: Expr, span: Span) -> Stmt {
    Stmt::new(
        StmtKind::Assign {
            name: temp.to_string(),
            value,
        },
        span,
    )
}

/// Builds `<flag> = <value>;` for the hidden "the included file has returned" flag.
pub(super) fn assign_flag(flag: &str, value: bool, span: Span) -> Stmt {
    Stmt::new(
        StmtKind::Assign {
            name: flag.to_string(),
            value: Expr::new(ExprKind::BoolLiteral(value), span),
        },
        span,
    )
}

/// Builds `!<flag>` — the guard under which statements after a possible return still run.
fn flag_is_clear(flag: &str, span: Span) -> Expr {
    Expr::new(
        ExprKind::Not(Box::new(Expr::new(
            ExprKind::Variable(flag.to_string()),
            span,
        ))),
        span,
    )
}
