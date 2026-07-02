//! Purpose:
//! Hoists conditionally-declared functions (a `FunctionDecl` nested inside an `if`/loop/`try`/etc.
//! body) up to the program's top level so the type checker's top-level `collect_function_decls`
//! registers them like any other user function. This unlocks the Symfony mbstring/intl polyfills,
//! which define every wrapper as `if (!function_exists('mb_...')) { function mb_...(...) { ... } }`.
//!
//! Called from:
//! - `crate::pipeline::compile()` after `crate::autoload::run`, on the fully name-resolved,
//!   autoloaded program, before constant folding and type checking.
//!
//! Key details:
//! - It runs AFTER autoload (not inside the resolver) so `crate::autoload::polyfill_prune` has
//!   already dropped its allowlisted provided-function/optional-helper guards first; hoisting them
//!   earlier would re-expose the heavy delegate classes that prune exists to keep out.
//! - Because name resolution has already run, `FunctionDecl` names are canonical and bodies are
//!   fully qualified, so a nested declaration can be moved to the top level verbatim — no namespace
//!   or `use`-import context needs to travel with it.
//! - A guarded declaration `if (!function_exists('X')) { function X ... }` is hoisted only when `X`
//!   is not already known (a builtin per the catalog, or an already-hoisted/top-level user
//!   function). When `X` IS already known the guard is statically false, so the nested definition is
//!   dropped (the existing builtin/function wins, matching PHP). Hoisting MOVES the declaration so no
//!   duplicate top-level symbol survives later control-flow pruning of a statically-true branch.
//! - Statically-dead branches (`if (false) { ... }`) are never hoisted from. Function/method/closure
//!   bodies are never descended into: a function declared inside another function is a runtime-local
//!   declaration, not a top-level one.

use std::collections::HashSet;

use crate::names::php_symbol_key;
use crate::parser::ast::{CatchClause, Expr, ExprKind, Program, Stmt, StmtKind};

/// Lowercased-name view of a function declaration for catalog/known-set comparison.
///
/// PHP function names are case-insensitive, and the polyfill guards spell the name as a string
/// literal that must match the declared name regardless of case or a leading namespace separator.
fn function_key(name: &str) -> String {
    php_symbol_key(name.trim_start_matches('\\'))
}

/// Returns whether `name` is a supported builtin function (case-insensitive), so a guarded
/// redefinition of it must be dropped rather than hoisted (the builtin wins, matching PHP).
fn is_builtin_function(name: &str) -> bool {
    crate::types::checker::builtins::canonical_builtin_function_name(name.trim_start_matches('\\'))
        .is_some()
}

/// Carries the `!function_exists('X')` guards active on the current path and whether the walk has
/// entered a conditional/loop/`try` body (so a `FunctionDecl` reached is conditionally declared).
#[derive(Clone, Default)]
struct HoistContext {
    /// Function names (lowercased) guarded by an enclosing `!function_exists('name')` on the path.
    guard_names: HashSet<String>,
    /// `true` once the walk has descended into at least one conditional/loop/`try` body, marking any
    /// `FunctionDecl` reached as conditionally declared (and therefore a hoist candidate).
    in_container: bool,
}

/// Hoists conditionally-declared functions in `program` to the top level and returns the rewritten
/// program with those declarations appended.
///
/// Runs on the name-resolved, autoloaded program. Surviving statements keep their order; hoisted
/// declarations are appended because PHP hoists top-level function definitions regardless of textual
/// position, and appending avoids disturbing any leading statement.
pub fn hoist_conditional_function_declarations(program: Program) -> Program {
    let mut known = collect_effective_top_level_function_names(&program);
    let mut hoisted: Vec<Stmt> = Vec::new();
    let ctx = HoistContext::default();
    let mut result = process_body(program, &ctx, &mut known, &mut hoisted);
    result.append(&mut hoisted);
    result
}

/// Collects the names (lowercased) of functions already visible at the top level: those declared
/// directly at the top level or inside a top-level `Synthetic` grouping (recursively), but not
/// inside conditionals or other bodies.
///
/// These seed the "already known" set so a guarded polyfill definition of a function the program
/// already declares at the top level is dropped rather than hoisted into a duplicate.
fn collect_effective_top_level_function_names(stmts: &[Stmt]) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_effective_top_level_function_names_into(stmts, &mut names);
    names
}

/// Recursive worker for [`collect_effective_top_level_function_names`], descending only through
/// `Synthetic` groupings (which are transparent to top-level registration) and never through
/// conditional/declaration bodies.
fn collect_effective_top_level_function_names_into(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::FunctionDecl { name, .. } => {
                names.insert(function_key(name));
            }
            StmtKind::Synthetic(body) => {
                collect_effective_top_level_function_names_into(body, names);
            }
            _ => {}
        }
    }
}

/// Processes a statement list in order under `ctx`, hoisting nested conditional function
/// declarations into `hoisted` and returning the (possibly rewritten) statements that remain.
fn process_body(
    stmts: Vec<Stmt>,
    ctx: &HoistContext,
    known: &mut HashSet<String>,
    hoisted: &mut Vec<Stmt>,
) -> Vec<Stmt> {
    let mut out = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::FunctionDecl { .. } if ctx.in_container => {
                if let Some(kept) = try_hoist_function(stmt, ctx, known, hoisted) {
                    out.push(kept);
                }
            }
            _ => out.push(rewrite_stmt(stmt, ctx, known, hoisted)),
        }
    }
    out
}

/// Decides what to do with a conditionally-nested `FunctionDecl`, returning `Some(stmt)` to keep it
/// in the body, or `None` when it was hoisted out or dropped as statically dead.
///
/// A declaration guarded by `!function_exists('X')` is dropped when `X` is already known (a builtin
/// or a previously hoisted/top-level user function): the guard is statically false, the pre-existing
/// definition wins (matching PHP), and dropping the dead body avoids spurious diagnostics. Otherwise
/// the declaration is moved to the top level and its name recorded so a later guarded duplicate is
/// likewise dropped.
fn try_hoist_function(
    stmt: Stmt,
    ctx: &HoistContext,
    known: &mut HashSet<String>,
    hoisted: &mut Vec<Stmt>,
) -> Option<Stmt> {
    let StmtKind::FunctionDecl { name, .. } = &stmt.kind else {
        return Some(stmt);
    };
    let key = function_key(name);
    let guarded = ctx.guard_names.contains(&key);
    if guarded && (is_builtin_function(&key) || known.contains(&key)) {
        return None;
    }
    known.insert(key);
    hoisted.push(stmt);
    None
}

/// Rewrites a non-declaration statement, recursing into conditional/loop/`try`/grouping bodies to
/// hoist any nested function declarations. Leaf statements and declaration bodies (functions,
/// classes, closures) are returned unchanged — those never hold top-level-hoistable declarations.
fn rewrite_stmt(
    stmt: Stmt,
    ctx: &HoistContext,
    known: &mut HashSet<String>,
    hoisted: &mut Vec<Stmt>,
) -> Stmt {
    let Stmt { kind, span, attributes } = stmt;
    let kind = match kind {
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            let then_body = process_branch(then_body, &condition, ctx, known, hoisted);
            let elseif_clauses = elseif_clauses
                .into_iter()
                .map(|(cond, body)| {
                    let body = process_branch(body, &cond, ctx, known, hoisted);
                    (cond, body)
                })
                .collect();
            let else_body =
                else_body.map(|body| process_container_body(body, ctx, known, hoisted));
            StmtKind::If {
                condition,
                then_body,
                elseif_clauses,
                else_body,
            }
        }
        StmtKind::While { condition, body } => StmtKind::While {
            condition,
            body: process_container_body(body, ctx, known, hoisted),
        },
        StmtKind::DoWhile { body, condition } => StmtKind::DoWhile {
            body: process_container_body(body, ctx, known, hoisted),
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
            body: process_container_body(body, ctx, known, hoisted),
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
            body: process_container_body(body, ctx, known, hoisted),
        },
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => StmtKind::Switch {
            subject,
            cases: cases
                .into_iter()
                .map(|(values, body)| (values, process_container_body(body, ctx, known, hoisted)))
                .collect(),
            default: default.map(|body| process_container_body(body, ctx, known, hoisted)),
        },
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => StmtKind::Try {
            try_body: process_container_body(try_body, ctx, known, hoisted),
            catches: catches
                .into_iter()
                .map(|CatchClause { exception_types, variable, body }| CatchClause {
                    exception_types,
                    variable,
                    body: process_container_body(body, ctx, known, hoisted),
                })
                .collect(),
            finally_body: finally_body.map(|body| process_container_body(body, ctx, known, hoisted)),
        },
        StmtKind::Synthetic(body) => {
            StmtKind::Synthetic(process_body(body, ctx, known, hoisted))
        }
        StmtKind::IncludeOnceGuard { label, body } => StmtKind::IncludeOnceGuard {
            label,
            body: process_body(body, ctx, known, hoisted),
        },
        StmtKind::NamespaceBlock { name, body } => StmtKind::NamespaceBlock {
            name,
            body: process_body(body, ctx, known, hoisted),
        },
        other => other,
    };
    Stmt {
        kind,
        span,
        attributes,
    }
}

/// Processes an `if`/`elseif` branch body, adding the branch condition's `!function_exists('X')`
/// guards to the context and marking `in_container`. A statically-false condition means the branch
/// is dead, so its body is returned unchanged (never hoisted from), matching PHP and later DCE.
fn process_branch(
    body: Vec<Stmt>,
    condition: &Expr,
    ctx: &HoistContext,
    known: &mut HashSet<String>,
    hoisted: &mut Vec<Stmt>,
) -> Vec<Stmt> {
    if constant_truthiness(condition) == Some(false) {
        return body;
    }
    let mut branch_ctx = ctx.clone();
    branch_ctx.in_container = true;
    for name in collect_function_exists_guards(condition) {
        branch_ctx.guard_names.insert(name);
    }
    process_body(body, &branch_ctx, known, hoisted)
}

/// Processes a loop/`try`/`else` body: marks `in_container` (so nested functions are hoist
/// candidates) but adds no new `function_exists` guards.
fn process_container_body(
    body: Vec<Stmt>,
    ctx: &HoistContext,
    known: &mut HashSet<String>,
    hoisted: &mut Vec<Stmt>,
) -> Vec<Stmt> {
    let mut inner = ctx.clone();
    inner.in_container = true;
    process_body(body, &inner, known, hoisted)
}

/// Collects the function names (lowercased) guarded by `!function_exists('name')` within a
/// condition, descending through `&&` chains so `!function_exists('a') && !function_exists('b')`
/// contributes both. Only the negated form guards a redefinition; other shapes contribute nothing.
fn collect_function_exists_guards(condition: &Expr) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_function_exists_guards_into(condition, &mut names);
    names
}

/// Recursive worker for [`collect_function_exists_guards`].
fn collect_function_exists_guards_into(condition: &Expr, names: &mut HashSet<String>) {
    match &condition.kind {
        ExprKind::BinaryOp {
            left,
            op: crate::parser::ast::BinOp::And,
            right,
        } => {
            collect_function_exists_guards_into(left, names);
            collect_function_exists_guards_into(right, names);
        }
        ExprKind::Not(inner) => {
            if let Some(name) = function_exists_argument(inner) {
                names.insert(function_key(&name));
            }
        }
        _ => {}
    }
}

/// Extracts the single string-literal argument of a `function_exists('name')` call, matching the
/// callee case-insensitively and tolerating a leading namespace separator (PHP resolves it to the
/// global builtin). Returns `None` for any other expression.
fn function_exists_argument(expr: &Expr) -> Option<String> {
    let ExprKind::FunctionCall { name, args } = &expr.kind else {
        return None;
    };
    if !name
        .as_str()
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("function_exists")
    {
        return None;
    }
    let [arg] = args.as_slice() else {
        return None;
    };
    match &arg.kind {
        ExprKind::StringLiteral(value) => Some(value.clone()),
        _ => None,
    }
}

/// Evaluates a condition's truthiness when it is a literal, used only to skip hoisting from a
/// statically-dead `if (false)` branch. Returns `None` for anything non-literal (including the
/// polyfill's `!function_exists(...)`), which is treated as potentially live.
fn constant_truthiness(expr: &Expr) -> Option<bool> {
    match &expr.kind {
        ExprKind::BoolLiteral(value) => Some(*value),
        ExprKind::Null => Some(false),
        ExprKind::IntLiteral(value) => Some(*value != 0),
        ExprKind::FloatLiteral(value) => Some(*value != 0.0),
        ExprKind::StringLiteral(value) => Some(!(value.is_empty() || value == "0")),
        _ => None,
    }
}
