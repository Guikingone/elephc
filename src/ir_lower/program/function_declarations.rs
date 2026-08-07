//! Purpose:
//! User function declaration discovery and lowering.
//!
//! Called from:
//! - `crate::ir_lower::program`.
//!
//! Key details:
//! - Keeps program metadata deterministic and EIR lowering behavior unchanged.

use super::*;

/// Lowers every function declaration reachable in the statement tree.
pub(super) fn lower_function_declarations(
    statements: &[Stmt],
    module: &mut Module,
    check_result: &CheckResult,
    constants: &std::collections::HashMap<String, (ExprKind, PhpType)>,
    fiber_return_sigs: &std::collections::HashMap<String, crate::types::FunctionSig>,
) {
    for stmt in statements {
        match &stmt.kind {
            StmtKind::FunctionDecl {
                by_ref_return: _,
                name,
                params,
                variadic: _,
                variadic_by_ref: _,
                variadic_type: _,
                return_type,
                body,
                ..
            } => function::lower_user_function(
                name,
                params,
                return_type.as_ref(),
                &stmt.attributes,
                body,
                module,
                check_result,
                constants,
                fiber_return_sigs,
            ),
            StmtKind::NamespaceBlock { body, .. }
            | StmtKind::Synthetic(body)
            | StmtKind::IncludeOnceGuard { body, .. } => {
                lower_function_declarations(
                    body,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                lower_function_declarations(
                    then_body,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
                for (_, body) in elseif_clauses {
                    lower_function_declarations(
                        body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
                if let Some(body) = else_body {
                    lower_function_declarations(
                        body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
            }
            StmtKind::IfDef {
                then_body,
                else_body,
                ..
            } => {
                lower_function_declarations(
                    then_body,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
                if let Some(body) = else_body {
                    lower_function_declarations(
                        body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Foreach { body, .. } => {
                lower_function_declarations(
                    body,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
            }
            StmtKind::Switch { cases, default, .. } => {
                for (_, body) in cases {
                    lower_function_declarations(
                        body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
                if let Some(body) = default {
                    lower_function_declarations(
                        body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
            }
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                lower_function_declarations(
                    try_body,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
                for catch in catches {
                    lower_function_declarations(
                        &catch.body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
                if let Some(body) = finally_body {
                    lower_function_declarations(
                        body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
            }
            _ => {}
        }
    }
}

