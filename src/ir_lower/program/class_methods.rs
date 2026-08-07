//! Purpose:
//! Class-like method and referenced builtin method lowering.
//!
//! Called from:
//! - `crate::ir_lower::program`.
//!
//! Key details:
//! - Keeps program metadata deterministic and EIR lowering behavior unchanged.

use super::*;

/// Lowers concrete class/interface methods, including trait methods flattened into classes.
pub(super) fn lower_class_like_methods(
    statements: &[Stmt],
    module: &mut Module,
    check_result: &CheckResult,
    constants: &std::collections::HashMap<String, (ExprKind, PhpType)>,
    fiber_return_sigs: &std::collections::HashMap<String, crate::types::FunctionSig>,
) {
    for stmt in statements {
        match &stmt.kind {
            StmtKind::ClassDecl { name, methods, .. } => {
                let methods = check_result
                    .classes
                    .get(name)
                    .map(|class_info| class_info.method_decls.as_slice())
                    .unwrap_or(methods.as_slice());
                lower_methods_for_class_like(
                    name,
                    methods,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
            }
            StmtKind::TraitDecl { .. } => {}
            StmtKind::InterfaceDecl { name, methods, .. } => {
                lower_methods_for_class_like(
                    name,
                    methods,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
            }
            StmtKind::EnumDecl { name, methods, .. } => {
                // Enum methods are lowered like class methods on the case singleton; prefer the
                // checker's flattened declarations (with `self` types resolved to the enum).
                let methods = check_result
                    .classes
                    .get(name)
                    .map(|class_info| class_info.method_decls.as_slice())
                    .unwrap_or(methods.as_slice());
                lower_methods_for_class_like(
                    name,
                    methods,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
            }
            StmtKind::NamespaceBlock { body, .. }
            | StmtKind::Synthetic(body)
            | StmtKind::IncludeOnceGuard { body, .. } => {
                lower_class_like_methods(body, module, check_result, constants, fiber_return_sigs);
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                lower_class_like_methods(
                    then_body,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
                for (_, body) in elseif_clauses {
                    lower_class_like_methods(
                        body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
                if let Some(body) = else_body {
                    lower_class_like_methods(
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
                lower_class_like_methods(
                    then_body,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
                if let Some(body) = else_body {
                    lower_class_like_methods(
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
                lower_class_like_methods(body, module, check_result, constants, fiber_return_sigs);
            }
            StmtKind::Switch { cases, default, .. } => {
                for (_, body) in cases {
                    lower_class_like_methods(
                        body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
                if let Some(body) = default {
                    lower_class_like_methods(
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
                lower_class_like_methods(
                    try_body,
                    module,
                    check_result,
                    constants,
                    fiber_return_sigs,
                );
                for catch in catches {
                    lower_class_like_methods(
                        &catch.body,
                        module,
                        check_result,
                        constants,
                        fiber_return_sigs,
                    );
                }
                if let Some(body) = finally_body {
                    lower_class_like_methods(
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

/// Lowers all concrete methods for one class-like declaration.
pub(super) fn lower_methods_for_class_like(
    class_name: &str,
    methods: &[ClassMethod],
    module: &mut Module,
    check_result: &CheckResult,
    constants: &std::collections::HashMap<String, (ExprKind, PhpType)>,
    fiber_return_sigs: &std::collections::HashMap<String, crate::types::FunctionSig>,
) {
    for method in methods {
        if !method.has_body {
            continue;
        }
        let method_key = php_method_key(&method.name);
        if class_method_already_lowered(module, class_name, &method_key, method.is_static) {
            continue;
        }
        function::lower_class_method(
            class_name,
            &method.name,
            method.is_static,
            &method.params,
            method.return_type.as_ref(),
            &method.body,
            module,
            check_result,
            constants,
            fiber_return_sigs,
        );
    }
}

