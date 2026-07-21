//! Purpose:
//! Minimal PRE-type-checker prune: removes ONLY statically-dead `if`/`elseif` branches (constant
//! condition after `fold_constants` + the curated pre-check extension fold) so a guarded call to a
//! never-available extension function (`if (function_exists('fastcgi_finish_request')) { ... }`)
//! never reaches the checker. Everything else is preserved verbatim.
//!
//! Called from:
//! - `crate::pipeline::compile()` (and the codegen test harnesses mirroring it) right after
//!   `crate::optimize::fold_function_existence` with `FunctionExistenceSet::for_pre_check`, before
//!   `crate::types::check_with_target`.
//!
//! Key details:
//! - This pass deliberately does NOT reuse `prune_constant_control_flow`: the full prune has
//!   several checker-observable drops that must never run before type checking — it deletes
//!   effect-free `ExprStmt`s (hiding `$undefined_var + 1;` errors), drops statements after a
//!   terminal statement (a top-level `return` inlined from an included file swallowed the whole
//!   rest of the program), and drops `while (false)`/dead-switch bodies the checker used to
//!   validate. Being purely additive on checker visibility, except for the dead guard branches
//!   themselves, is this pass's contract.
//! - Recurses through statement bodies including function declarations and class/trait/interface/
//!   enum method bodies (the Symfony guard shapes live in methods). It does not descend into
//!   closure bodies inside expressions: a curated guard inside a closure keeps its dead branch and
//!   still fails the checker loudly — a scope gap, not a silent hole.
//! - A statically-true condition inlines that branch's body IN PLACE (later branches are dead and
//!   dropped); a statically-false condition drops only that branch. Non-constant conditions keep
//!   the chain shape with all surviving bodies recursed.

use crate::parser::ast::{ClassMethod, Expr, Program, Stmt, StmtKind};

use super::fold::scalar_value;

/// Removes statically-dead `if`/`elseif`/`else` branches across the whole program, recursing into
/// nested statement bodies and method bodies. Every other statement kind is preserved unchanged
/// (same span, same attributes, same expressions).
pub fn prune_dead_static_branches(program: Program) -> Program {
    precheck_prune_block(program)
}

/// Applies `precheck_prune_stmt` to each statement of a block, splicing inlined
/// statically-true branch bodies flat into the enclosing block. Never drops trailing
/// statements: reachability is the checker's concern at this stage, not this pass's.
fn precheck_prune_block(stmts: Vec<Stmt>) -> Vec<Stmt> {
    stmts.into_iter().flat_map(precheck_prune_stmt).collect()
}

/// Rewrites one statement: resolves constant-condition `if` chains and recurses into nested
/// statement bodies; all other statement kinds pass through unchanged.
fn precheck_prune_stmt(stmt: Stmt) -> Vec<Stmt> {
    let span = stmt.span;
    let attributes = stmt.attributes;
    match stmt.kind {
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => prune_if_chain_minimal(condition, then_body, elseif_clauses, else_body, span, attributes),
        StmtKind::While { condition, body } => vec![Stmt {
            kind: StmtKind::While {
                condition,
                body: precheck_prune_block(body),
            },
            span,
            attributes,
        }],
        StmtKind::DoWhile { body, condition } => vec![Stmt {
            kind: StmtKind::DoWhile {
                body: precheck_prune_block(body),
                condition,
            },
            span,
            attributes,
        }],
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => vec![Stmt {
            kind: StmtKind::For {
                init,
                condition,
                update,
                body: precheck_prune_block(body),
            },
            span,
            attributes,
        }],
        StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body,
        } => vec![Stmt {
            kind: StmtKind::Foreach {
                array,
                key_var,
                value_var,
                value_by_ref,
                body: precheck_prune_block(body),
            },
            span,
            attributes,
        }],
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => vec![Stmt {
            kind: StmtKind::Switch {
                subject,
                cases: cases
                    .into_iter()
                    .map(|(conditions, body)| (conditions, precheck_prune_block(body)))
                    .collect(),
                default: default.map(precheck_prune_block),
            },
            span,
            attributes,
        }],
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => vec![Stmt {
            kind: StmtKind::Try {
                try_body: precheck_prune_block(try_body),
                catches: catches
                    .into_iter()
                    .map(|catch| crate::parser::ast::CatchClause {
                        exception_types: catch.exception_types,
                        variable: catch.variable,
                        body: precheck_prune_block(catch.body),
                    })
                    .collect(),
                finally_body: finally_body.map(precheck_prune_block),
            },
            span,
            attributes,
        }],
        StmtKind::NamespaceBlock { name, body } => vec![Stmt {
            kind: StmtKind::NamespaceBlock {
                name,
                body: precheck_prune_block(body),
            },
            span,
            attributes,
        }],
        StmtKind::Synthetic(body) => vec![Stmt {
            kind: StmtKind::Synthetic(precheck_prune_block(body)),
            span,
            attributes,
        }],
        StmtKind::IncludeOnceGuard { label, body } => vec![Stmt {
            kind: StmtKind::IncludeOnceGuard {
                label,
                body: precheck_prune_block(body),
            },
            span,
            attributes,
        }],
        StmtKind::IfDef {
            symbol,
            then_body,
            else_body,
        } => vec![Stmt {
            kind: StmtKind::IfDef {
                symbol,
                then_body: precheck_prune_block(then_body),
                else_body: else_body.map(precheck_prune_block),
            },
            span,
            attributes,
        }],
        StmtKind::FunctionDecl {
            name,
            params,
            variadic,
            variadic_type,
            return_type,
            by_ref_return,
            body,
            param_attributes,
            variadic_by_ref,
        } => vec![Stmt {
            kind: StmtKind::FunctionDecl {
                name,
                params,
                variadic,
                variadic_type,
                return_type,
                by_ref_return,
                body: precheck_prune_block(body),
                param_attributes,
                variadic_by_ref,
            },
            span,
            attributes,
        }],
        StmtKind::ClassDecl {
            name,
            extends,
            implements,
            is_abstract,
            is_final,
            is_readonly_class,
            trait_uses,
            properties,
            methods,
            constants,
        } => vec![Stmt {
            kind: StmtKind::ClassDecl {
                name,
                extends,
                implements,
                is_abstract,
                is_final,
                is_readonly_class,
                trait_uses,
                properties,
                methods: methods.into_iter().map(precheck_prune_method).collect(),
                constants,
            },
            span,
            attributes,
        }],
        StmtKind::TraitDecl {
            name,
            trait_uses,
            properties,
            methods,
            constants,
        } => vec![Stmt {
            kind: StmtKind::TraitDecl {
                name,
                trait_uses,
                properties,
                methods: methods.into_iter().map(precheck_prune_method).collect(),
                constants,
            },
            span,
            attributes,
        }],
        StmtKind::InterfaceDecl {
            name,
            extends,
            properties,
            methods,
            constants,
        } => vec![Stmt {
            kind: StmtKind::InterfaceDecl {
                name,
                extends,
                properties,
                methods: methods.into_iter().map(precheck_prune_method).collect(),
                constants,
            },
            span,
            attributes,
        }],
        StmtKind::EnumDecl {
            name,
            backing_type,
            cases,
            implements,
            methods,
            constants,
            trait_uses,
        } => vec![Stmt {
            kind: StmtKind::EnumDecl {
                name,
                backing_type,
                cases,
                implements,
                methods: methods.into_iter().map(precheck_prune_method).collect(),
                constants,
                trait_uses,
            },
            span,
            attributes,
        }],
        kind => vec![Stmt { kind, span, attributes }],
    }
}

/// Recurses into one class/trait/interface/enum method body.
fn precheck_prune_method(method: ClassMethod) -> ClassMethod {
    ClassMethod {
        body: precheck_prune_block(method.body),
        ..method
    }
}

/// Resolves an `if`/`elseif`/`else` chain whose conditions may have folded to constants:
/// statically-false branches are dropped, the first statically-true branch's body is inlined in
/// place of the whole remaining chain, and non-constant branches keep their position with their
/// bodies recursed. Preserves the original statement's span and attributes on the rebuilt chain.
fn prune_if_chain_minimal(
    condition: Expr,
    then_body: Vec<Stmt>,
    elseif_clauses: Vec<(Expr, Vec<Stmt>)>,
    else_body: Option<Vec<Stmt>>,
    span: crate::span::Span,
    attributes: Vec<crate::parser::ast::AttributeGroup>,
) -> Vec<Stmt> {
    let mut surviving: Vec<(Expr, Vec<Stmt>)> = Vec::new();
    let mut resolved_else: Option<Vec<Stmt>> = None;
    for (cond, body) in std::iter::once((condition, then_body)).chain(elseif_clauses) {
        match scalar_value(&cond) {
            // Statically-false branch: dead, drop it (this is the whole point of the pass).
            Some(value) if !value.truthy() => continue,
            // Statically-true branch: it becomes the chain's `else`; everything after is dead.
            Some(_) => {
                resolved_else = Some(body);
                break;
            }
            // Runtime condition: keep the branch in position.
            None => surviving.push((cond, body)),
        }
    }
    if resolved_else.is_none() {
        resolved_else = else_body;
    }
    let resolved_else = resolved_else.map(precheck_prune_block);

    let mut surviving = surviving.into_iter();
    let Some((first_condition, first_body)) = surviving.next() else {
        // No runtime branch survived: the chain reduces to the resolved else body (possibly empty),
        // spliced flat into the enclosing block.
        return resolved_else.unwrap_or_default();
    };
    vec![Stmt {
        kind: StmtKind::If {
            condition: first_condition,
            then_body: precheck_prune_block(first_body),
            elseif_clauses: surviving
                .map(|(cond, body)| (cond, precheck_prune_block(body)))
                .collect(),
            else_body: resolved_else,
        },
        span,
        attributes,
    }]
}
