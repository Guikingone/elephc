//! Purpose:
//! Implements optimizer control-flow fold logic.
//! Supports normalization, reachability, path analysis, and structural rewrites used by pruning and DCE.
//!
//! Called from:
//! - `crate::optimize::control`
//!
//! Key details:
//! - Control-flow helpers must treat terminal effects, switch fallthrough, and exception paths conservatively.

use super::*;

/// Recursively folds expressions within a statement, applying constant-folding to all
/// sub-expressions while preserving the statement structure.
///
/// Handles every `StmtKind` variant: passes through variants with no expressions unchanged,
/// and recursively processes expression fields in variants that contain them (conditions,
/// bodies, values, etc.). Used by the optimizer to propagate folded constants through
/// the AST before DCE or control-flow pruning.
///
/// - `stmt`: The statement to process.
/// Returns a new `Stmt` with all nested expressions folded via `fold_expr`.
pub(crate) fn fold_stmt(stmt: Stmt) -> Stmt {
    let span = stmt.span;
    let source_mode = stmt.source_mode;
    let strict_types = stmt.strict_types;
    let attributes = stmt.attributes.clone();
    let kind = match stmt.kind {
        StmtKind::Synthetic(stmts) => StmtKind::Synthetic(fold_block(stmts)),
        StmtKind::IncludeOnceMark { source_path } => StmtKind::IncludeOnceMark { source_path },
        StmtKind::IncludeOnceGuard { source_path, body } => StmtKind::IncludeOnceGuard {
            source_path,
            body: fold_block(body),
        },
        StmtKind::Echo(expr) => StmtKind::Echo(fold_expr(expr)),
        StmtKind::Assign { name, value } => StmtKind::Assign {
            name,
            value: fold_expr(value),
        },
        StmtKind::RefAssign { target, source } => StmtKind::RefAssign { target, source },
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => fold_if_statement(
            condition,
            then_body,
            elseif_clauses,
            else_body,
            span,
            source_mode,
            strict_types,
        ),
        StmtKind::IfDef {
            symbol,
            then_body,
            else_body,
        } => StmtKind::IfDef {
            symbol,
            then_body: fold_block(then_body),
            else_body: else_body.map(fold_block),
        },
        StmtKind::While { condition, body } => StmtKind::While {
            condition: fold_expr(condition),
            body: fold_block(body),
        },
        StmtKind::DoWhile { body, condition } => StmtKind::DoWhile {
            body: fold_block(body),
            condition: fold_expr(condition),
        },
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => StmtKind::For {
            init: init.map(|stmt| Box::new(fold_stmt(*stmt))),
            condition: condition.map(fold_expr),
            update: update.map(|stmt| Box::new(fold_stmt(*stmt))),
            body: fold_block(body),
        },
        StmtKind::ArrayAssign {
            array,
            index,
            value,
        } => StmtKind::ArrayAssign {
            array,
            index: fold_expr(index),
            value: fold_expr(value),
        },
        StmtKind::NestedArrayAssign { target, value } => StmtKind::NestedArrayAssign {
            target: fold_expr(target),
            value: fold_expr(value),
        },
        StmtKind::ArrayPush { array, value } => StmtKind::ArrayPush {
            array,
            value: fold_expr(value),
        },
        StmtKind::TypedAssign {
            type_expr,
            name,
            value,
        } => StmtKind::TypedAssign {
            type_expr,
            name,
            value: fold_expr(value),
        },
        StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body,
        } => StmtKind::Foreach {
            array: fold_expr(array),
            key_var,
            value_var,
            value_by_ref,
            body: fold_block(body),
        },
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => StmtKind::Switch {
            subject: fold_expr(subject),
            cases: cases
                .into_iter()
                .map(|(exprs, body)| {
                    (
                        exprs.into_iter().map(fold_expr).collect(),
                        fold_block(body),
                    )
                })
                .collect(),
            default: default.map(fold_block),
        },
        StmtKind::Include {
            path,
            once,
            required,
        } => StmtKind::Include {
            path,
            once,
            required,
        },
        StmtKind::Throw(expr) => StmtKind::Throw(fold_expr(expr)),
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => StmtKind::Try {
            try_body: fold_block(try_body),
            catches: catches
                .into_iter()
                .map(|catch| crate::parser::ast::CatchClause {
                    exception_types: catch.exception_types,
                    variable: catch.variable,
                    body: fold_block(catch.body),
                })
                .collect(),
            finally_body: finally_body.map(fold_block),
        },
        StmtKind::Break(levels) => StmtKind::Break(levels),
        StmtKind::Continue(levels) => StmtKind::Continue(levels),
        StmtKind::ExprStmt(expr) => StmtKind::ExprStmt(fold_expr(expr)),
        StmtKind::NamespaceDecl { name } => StmtKind::NamespaceDecl { name },
        StmtKind::NamespaceBlock { name, body } => StmtKind::NamespaceBlock {
            name,
            body: fold_block(body),
        },
        StmtKind::UseDecl { imports } => StmtKind::UseDecl { imports },
        StmtKind::FunctionDecl {
            by_ref_return,
            name,
            params,
            param_attributes,
            variadic,
            variadic_by_ref,
            variadic_type,
            return_type,
            body,
        } => StmtKind::FunctionDecl {
            by_ref_return,
            name,
            params: fold_params(params),
            param_attributes,
            variadic,
            variadic_by_ref,
            variadic_type,
            return_type,
            body: fold_block(body),
        },
        StmtKind::Return(expr) => StmtKind::Return(expr.map(fold_expr)),
        StmtKind::ConstDecl { name, value } => StmtKind::ConstDecl {
            name,
            value: fold_expr(value),
        },
        StmtKind::ListUnpack { vars, value } => StmtKind::ListUnpack {
            vars,
            value: fold_expr(value),
        },
        StmtKind::Global { vars } => StmtKind::Global { vars },
        StmtKind::StaticVar { name, init } => StmtKind::StaticVar {
            name,
            init: fold_expr(init),
        },
        StmtKind::ClassDecl {
            name,
            doc_comment,
            extends,
            implements,
            is_abstract,
            is_final,
            is_readonly_class,
            trait_uses,
            properties,
            methods,
        constants,
        } => StmtKind::ClassDecl {
            name,
            doc_comment,
            extends,
            implements,
            is_abstract,
            is_final,
            is_readonly_class,
            trait_uses,
            properties: properties.into_iter().map(fold_property).collect(),
            methods: methods.into_iter().map(fold_method).collect(),
        constants,
        },
        StmtKind::EnumDecl {
            name,
            backing_type,
            cases,
            implements,
            trait_uses,
            methods,
            constants,
        } => StmtKind::EnumDecl {
            name,
            backing_type,
            implements,
            trait_uses,
            methods: methods.into_iter().map(fold_method).collect(),
            constants,
            cases: cases.into_iter().map(fold_enum_case).collect(),
        },
        StmtKind::PackedClassDecl { name, fields } => StmtKind::PackedClassDecl { name, fields },
        StmtKind::InterfaceDecl {
            name,
            extends,
            properties,
            methods,
        constants,
        } => StmtKind::InterfaceDecl {
            name,
            extends,
            properties: properties.into_iter().map(fold_property).collect(),
            methods: methods.into_iter().map(fold_method).collect(),
        constants,
        },
        StmtKind::TraitDecl {
            name,
            trait_uses,
            properties,
            methods,
        constants,
        } => StmtKind::TraitDecl {
            name,
            trait_uses,
            properties: properties.into_iter().map(fold_property).collect(),
            methods: methods.into_iter().map(fold_method).collect(),
        constants,
        },
        StmtKind::PropertyAssign {
            object,
            property,
            value,
        } => StmtKind::PropertyAssign {
            object: Box::new(fold_expr(*object)),
            property,
            value: fold_expr(value),
        },
        StmtKind::PropertyRefAssign {
            object,
            property,
            source,
        } => StmtKind::PropertyRefAssign {
            object: Box::new(fold_expr(*object)),
            property,
            source: fold_expr(source),
        },
        StmtKind::StaticPropertyAssign {
            receiver,
            property,
            value,
        } => StmtKind::StaticPropertyAssign {
            receiver,
            property,
            value: fold_expr(value),
        },
        StmtKind::StaticPropertyArrayPush {
            receiver,
            property,
            value,
        } => StmtKind::StaticPropertyArrayPush {
            receiver,
            property,
            value: fold_expr(value),
        },
        StmtKind::StaticPropertyArrayAssign {
            receiver,
            property,
            index,
            value,
        } => StmtKind::StaticPropertyArrayAssign {
            receiver,
            property,
            index: fold_expr(index),
            value: fold_expr(value),
        },
        StmtKind::StaticPropertyElementRefAssign {
            receiver,
            property,
            index,
            source,
        } => StmtKind::StaticPropertyElementRefAssign {
            receiver,
            property,
            index: fold_expr(index),
            source: fold_expr(source),
        },
        StmtKind::DynamicStaticPropertyWrite {
            receiver,
            property,
            index,
            append,
            value,
        } => StmtKind::DynamicStaticPropertyWrite {
            receiver,
            property: Box::new(fold_expr(*property)),
            index: index.map(fold_expr),
            append,
            value: fold_expr(value),
        },
        StmtKind::PropertyArrayPush {
            object,
            property,
            value,
        } => StmtKind::PropertyArrayPush {
            object: Box::new(fold_expr(*object)),
            property,
            value: fold_expr(value),
        },
        StmtKind::PropertyArrayAssign {
            object,
            property,
            index,
            value,
        } => StmtKind::PropertyArrayAssign {
            object: Box::new(fold_expr(*object)),
            property,
            index: fold_expr(index),
            value: fold_expr(value),
        },
        StmtKind::ExternFunctionDecl {
            name,
            params,
            return_type,
            library,
        } => StmtKind::ExternFunctionDecl {
            name,
            params,
            return_type,
            library,
        },
        StmtKind::ExternClassDecl { name, fields } => StmtKind::ExternClassDecl { name, fields },
        StmtKind::ClassLikeActivate { name, kind, source_path } => StmtKind::ClassLikeActivate { name, kind, source_path },
        StmtKind::ExternGlobalDecl { name, c_type } => {
            StmtKind::ExternGlobalDecl { name, c_type }
        }
        StmtKind::FunctionVariantGroup { name, variants } => {
            StmtKind::FunctionVariantGroup { name, variants }
        }
        StmtKind::FunctionVariantMark { name, variant } => {
            StmtKind::FunctionVariantMark { name, variant }
        }
    };
    Stmt {
        kind,
        span,
        source_mode,
        strict_types,
        attributes,
    }
}

/// Folds all statements in a block by mapping `fold_stmt` over each element.
///
/// - `body`: A vector of statements representing a block body.
/// Returns a new `Vec<Stmt>` with each statement folded.
pub(crate) fn fold_block(body: Vec<Stmt>) -> Vec<Stmt> {
    body.into_iter().map(fold_stmt).collect()
}

/// Folds an if-chain and removes a leading branch whose condition became a scalar literal.
///
/// This early, effect-safe pruning happens before type checking so a statically unreachable
/// branch cannot report diagnostics for expressions PHP never evaluates. Dynamic conditions
/// retain the original if-chain shape and all source-order evaluation rules.
fn fold_if_statement(
    condition: Expr,
    then_body: Vec<Stmt>,
    elseif_clauses: Vec<(Expr, Vec<Stmt>)>,
    else_body: Option<Vec<Stmt>>,
    span: crate::span::Span,
    source_mode: crate::source::SourceMode,
    strict_types: bool,
) -> StmtKind {
    let condition = fold_expr(condition);
    let Some((truthy, required_evaluation)) = known_condition_truthiness(&condition) else {
        return StmtKind::If {
            condition,
            then_body: fold_block(then_body),
            elseif_clauses: elseif_clauses
                .into_iter()
                .map(|(condition, body)| (fold_expr(condition), fold_block(body)))
                .collect(),
            else_body: else_body.map(fold_block),
        };
    };

    if truthy {
        return prefix_required_condition_evaluation(
            StmtKind::Synthetic(fold_block(then_body)),
            required_evaluation,
            span,
            source_mode,
            strict_types,
        );
    }

    let selected = fold_reachable_elseif_chain(
        elseif_clauses,
        else_body,
        span,
        source_mode,
        strict_types,
    );
    prefix_required_condition_evaluation(
        selected,
        required_evaluation,
        span,
        source_mode,
        strict_types,
    )
}

/// Selects the first statically true elseif branch or preserves the suffix beginning at the
/// first dynamic condition; when every elseif is false, it selects the optional else body.
fn fold_reachable_elseif_chain(
    mut elseif_clauses: Vec<(Expr, Vec<Stmt>)>,
    else_body: Option<Vec<Stmt>>,
    span: crate::span::Span,
    source_mode: crate::source::SourceMode,
    strict_types: bool,
) -> StmtKind {
    while !elseif_clauses.is_empty() {
        let (condition, body) = elseif_clauses.remove(0);
        let condition = fold_expr(condition);
        match known_condition_truthiness(&condition) {
            Some((true, required_evaluation)) => {
                return prefix_required_condition_evaluation(
                    StmtKind::Synthetic(fold_block(body)),
                    required_evaluation,
                    span,
                    source_mode,
                    strict_types,
                );
            }
            Some((false, required_evaluation)) => {
                if let Some(evaluation) = required_evaluation {
                    let selected = fold_reachable_elseif_chain(
                        elseif_clauses,
                        else_body,
                        span,
                        source_mode,
                        strict_types,
                    );
                    return prefix_required_condition_evaluation(
                        selected,
                        Some(evaluation),
                        span,
                        source_mode,
                        strict_types,
                    );
                }
            }
            None => {
                return StmtKind::If {
                    condition,
                    then_body: fold_block(body),
                    elseif_clauses: elseif_clauses
                        .into_iter()
                        .map(|(condition, body)| (fold_expr(condition), fold_block(body)))
                        .collect(),
                    else_body: else_body.map(fold_block),
                };
            }
        }
    }

    StmtKind::Synthetic(else_body.map(fold_block).unwrap_or_default())
}

/// Determines scalar truthiness and the expression that still must be evaluated before a
/// short-circuit identity can select a branch (`$value || true`, `$value && false`).
fn known_condition_truthiness(condition: &Expr) -> Option<(bool, Option<Expr>)> {
    if let Some(value) = scalar_value(condition) {
        return Some((value.truthy(), None));
    }
    let ExprKind::BinaryOp { left, op, right } = &condition.kind else {
        return None;
    };
    let right = scalar_value(right)?;
    match op {
        BinOp::Or if right.truthy() => Some((true, Some((**left).clone()))),
        BinOp::And if !right.truthy() => Some((false, Some((**left).clone()))),
        _ => None,
    }
}

/// Prefixes a selected branch with the expression PHP must still evaluate for a short-circuit
/// identity, retaining the original statement's source and strict-types profile.
fn prefix_required_condition_evaluation(
    selected: StmtKind,
    required_evaluation: Option<Expr>,
    span: crate::span::Span,
    source_mode: crate::source::SourceMode,
    strict_types: bool,
) -> StmtKind {
    let Some(required_evaluation) = required_evaluation else {
        return selected;
    };
    let mut body = vec![Stmt {
        kind: StmtKind::ExprStmt(required_evaluation),
        span,
        source_mode,
        strict_types,
        attributes: Vec::new(),
    }];
    match selected {
        StmtKind::Synthetic(mut selected_body) => body.append(&mut selected_body),
        selected => body.push(Stmt {
            kind: selected,
            span,
            source_mode,
            strict_types,
            attributes: Vec::new(),
        }),
    }
    StmtKind::Synthetic(body)
}
