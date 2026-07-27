//! Purpose:
//! Decides, for a given OPcache prelude function name (`opcache_get_configuration` or
//! `opcache_reset`), whether a parsed program references it (so the corresponding
//! prelude function is injected only when used) and whether the program already
//! declares its own function of that name (so a user definition is never clobbered).
//!
//! Called from:
//! - `crate::opcache_prelude::inject_if_used`.
//!
//! Key details:
//! - The walk is generic over the target name so a single exhaustive traversal backs
//!   every OPcache prelude function; each function is detected and injected
//!   independently (pay-for-use per function, with its own redeclaration guard).
//! - Runs before name resolution, so function `Name`s are raw source text; matched
//!   case-insensitively on the unqualified last segment (PHP function names are
//!   case-insensitive and may be written `\opcache_reset`).
//! - A matching string literal also counts as a reference so the
//!   `function_exists('opcache_reset')` and callable forms still inject the function.
//!   Over-injection (e.g. an unrelated string) only adds a small, later
//!   dead-code-eliminated function; soundness (never missing a real use) is what
//!   matters, so the `match`es are exhaustive with no wildcard arm.

use crate::names::Name;
use crate::parser::ast::{
    CallableTarget, ClassConst, ClassMethod, ClassProperty, EnumCaseDecl, Expr, ExprKind,
    InstanceOfTarget, PackedField, Stmt, StmtKind, TraitUse, TypeExpr,
};

/// Returns whether any top-level statement references `target` (an OPcache prelude
/// function name, already lowercase), so that function must be injected ahead of user
/// code.
pub(crate) fn program_references(program: &[Stmt], target: &str) -> bool {
    program.iter().any(|stmt| stmt_refs(stmt, target))
}

/// Returns whether the program already declares its own `target` function (at top level
/// or inside a namespace/guard/synthetic block), in which case that prelude function
/// must not be injected so the user definition wins and there is no redeclaration error.
pub(crate) fn program_declares(program: &[Stmt], target: &str) -> bool {
    program.iter().any(|stmt| stmt_declares(stmt, target))
}

/// Returns whether a function name equals `target`, compared case-insensitively on its
/// unqualified last segment.
fn name_is(name: &Name, target: &str) -> bool {
    name.last_segment()
        .is_some_and(|segment| segment.eq_ignore_ascii_case(target))
}

/// Returns whether a statement declares a top-level `target` function, recursing only
/// into block forms that can host a hoisted declaration.
fn stmt_declares(stmt: &Stmt, target: &str) -> bool {
    match &stmt.kind {
        StmtKind::FunctionDecl { name, .. } => name.eq_ignore_ascii_case(target),
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => body.iter().any(|stmt| stmt_declares(stmt, target)),
        _ => false,
    }
}

/// Returns whether a first-class-callable target references `target` via a function
/// name; method/static-method targets cannot name it but their receiver is still walked
/// for nested references.
fn callable_target_refs(target_ref: &CallableTarget, target: &str) -> bool {
    match target_ref {
        CallableTarget::Function(name) => name_is(name, target),
        CallableTarget::StaticMethod { .. } => false,
        CallableTarget::Method { object, .. } => expr_refs(object, target),
    }
}

/// Returns whether any parameter's default value references the function (type hints
/// cannot). Shared by function, method, and closure parameter lists.
fn params_ref(params: &[(String, Option<TypeExpr>, Option<Expr>, bool)], target: &str) -> bool {
    params
        .iter()
        .any(|(_, _, default, _)| default.as_ref().is_some_and(|expr| expr_refs(expr, target)))
}

/// Returns whether a `use Trait` clause references the function; trait/method names in
/// adaptations are not call sites, so this is always false.
fn trait_use_refs(_trait_use: &TraitUse, _target: &str) -> bool {
    false
}

/// Returns whether a class property's default value references the function.
fn class_property_refs(property: &ClassProperty, target: &str) -> bool {
    property.default.as_ref().is_some_and(|expr| expr_refs(expr, target))
}

/// Returns whether a method's parameter defaults or body reference the function.
fn class_method_refs(method: &ClassMethod, target: &str) -> bool {
    params_ref(&method.params, target) || method.body.iter().any(|stmt| stmt_refs(stmt, target))
}

/// Returns whether a class constant's initializer references the function.
fn class_const_refs(constant: &ClassConst, target: &str) -> bool {
    expr_refs(&constant.value, target)
}

/// Returns whether an enum case's backing-value expression references the function.
fn enum_case_refs(case: &EnumCaseDecl, target: &str) -> bool {
    case.value.as_ref().is_some_and(|expr| expr_refs(expr, target))
}

/// Returns whether a `packed class` field references the function; packed fields carry
/// only types, never call sites.
fn packed_field_refs(_field: &PackedField, _target: &str) -> bool {
    false
}

/// Returns whether an `instanceof` target's runtime-expression operand references the
/// function (name targets are class positions, never call sites).
fn instanceof_target_refs(target_ref: &InstanceOfTarget, target: &str) -> bool {
    match target_ref {
        InstanceOfTarget::Name(_) => false,
        InstanceOfTarget::Expr(expr) => expr_refs(expr, target),
    }
}

/// Returns whether an expression references `target` at any call position or as a
/// matching string literal, recursing into every child. The `match` is exhaustive so a
/// new `ExprKind` cannot silently bypass detection.
fn expr_refs(expr: &Expr, target: &str) -> bool {
    match &expr.kind {
        // `require`/`include` in expression position: recurse into the path expression.
        ExprKind::IncludeValue { path, .. } => expr_refs(path, target),
        // A matching string literal counts (function_exists/callable).
        ExprKind::StringLiteral(value) => value.eq_ignore_ascii_case(target),

        // Leaves and identifier-only forms carry no call site.
        ExprKind::IntLiteral(_)
        | ExprKind::FloatLiteral(_)
        | ExprKind::Variable(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::Null
        | ExprKind::This
        | ExprKind::PreIncrement(_)
        | ExprKind::PostIncrement(_)
        | ExprKind::PreDecrement(_)
        | ExprKind::PostDecrement(_)
        | ExprKind::ConstRef(_)
        | ExprKind::MagicConstant(_) => false,

        ExprKind::FunctionCall { name, args } => {
            name_is(name, target) || args.iter().any(|arg| expr_refs(arg, target))
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            expr_refs(object, target) || args.iter().any(|arg| expr_refs(arg, target))
        }
        ExprKind::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => {
            expr_refs(object, target)
                || expr_refs(method, target)
                || args.iter().any(|arg| expr_refs(arg, target))
        }
        ExprKind::StaticMethodCall { args, .. } => args.iter().any(|arg| expr_refs(arg, target)),
        ExprKind::FirstClassCallable(callable) => callable_target_refs(callable, target),

        ExprKind::BinaryOp { left, right, .. } => {
            expr_refs(left, target) || expr_refs(right, target)
        }
        ExprKind::InstanceOf { value, target: iof } => {
            expr_refs(value, target) || instanceof_target_refs(iof, target)
        }
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::Clone(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::YieldFrom(inner) => expr_refs(inner, target),
        ExprKind::NullCoalesce { value, default }
        | ExprKind::ShortTernary { value, default } => {
            expr_refs(value, target) || expr_refs(default, target)
        }
        ExprKind::Pipe { value, callable } => {
            expr_refs(value, target) || expr_refs(callable, target)
        }
        ExprKind::Assignment {
            target: assign_target,
            value,
            result_target,
            prelude,
            ..
        } => {
            expr_refs(assign_target, target)
                || expr_refs(value, target)
                || result_target.as_deref().is_some_and(|expr| expr_refs(expr, target))
                || prelude.iter().any(|stmt| stmt_refs(stmt, target))
        }
        ExprKind::ClosureCall { args, .. } => args.iter().any(|arg| expr_refs(arg, target)),
        ExprKind::ArrayLiteral(items) => items.iter().any(|item| expr_refs(item, target)),
        ExprKind::ArrayLiteralAssoc(pairs) => pairs
            .iter()
            .any(|(key, value)| expr_refs(key, target) || expr_refs(value, target)),
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            expr_refs(subject, target)
                || arms.iter().any(|(conditions, body)| {
                    conditions.iter().any(|cond| expr_refs(cond, target))
                        || expr_refs(body, target)
                })
                || default.as_deref().is_some_and(|expr| expr_refs(expr, target))
        }
        ExprKind::ArrayAccess { array, index } => {
            expr_refs(array, target) || expr_refs(index, target)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_refs(condition, target)
                || expr_refs(then_expr, target)
                || expr_refs(else_expr, target)
        }
        ExprKind::Cast { expr, .. } | ExprKind::PtrCast { expr, .. } => expr_refs(expr, target),
        ExprKind::Closure { params, body, .. } => {
            params_ref(params, target) || body.iter().any(|stmt| stmt_refs(stmt, target))
        }
        ExprKind::NamedArg { value, .. } => expr_refs(value, target),
        ExprKind::ExprCall { callee, args } => {
            expr_refs(callee, target) || args.iter().any(|arg| expr_refs(arg, target))
        }
        ExprKind::NewObject { args, .. } => args.iter().any(|arg| expr_refs(arg, target)),
        ExprKind::NewDynamic { name_expr, args } => {
            expr_refs(name_expr, target) || args.iter().any(|arg| expr_refs(arg, target))
        }
        ExprKind::NewDynamicObject {
            class_name, args, ..
        } => expr_refs(class_name, target) || args.iter().any(|arg| expr_refs(arg, target)),
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. } => expr_refs(object, target),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            expr_refs(object, target) || expr_refs(property, target)
        }
        ExprKind::StaticPropertyAccess { .. } => false,
        ExprKind::BufferNew { len, .. } => expr_refs(len, target),
        ExprKind::ClassConstant { .. } | ExprKind::ScopedConstantAccess { .. } => false,
        ExprKind::ObjectClassName { object } => expr_refs(object, target),
        ExprKind::NewScopedObject { args, .. } => args.iter().any(|arg| expr_refs(arg, target)),
        ExprKind::Yield { key, value } => {
            key.as_deref().is_some_and(|expr| expr_refs(expr, target))
                || value.as_deref().is_some_and(|expr| expr_refs(expr, target))
        }
    }
}

/// Returns whether a statement references `target` at any call position or string
/// literal, recursing into nested statements, expressions, and class members. The
/// `match` is exhaustive so a new `StmtKind` cannot silently bypass detection.
fn stmt_refs(stmt: &Stmt, target: &str) -> bool {
    match &stmt.kind {
        // Statements with no call position and no child expr/stmt.
        StmtKind::RefAssign { .. }
        | StmtKind::IncludeOnceMark { .. }
        | StmtKind::Break(_)
        | StmtKind::Continue(_)
        | StmtKind::NamespaceDecl { .. }
        | StmtKind::FunctionVariantGroup { .. }
        | StmtKind::FunctionVariantMark { .. }
        | StmtKind::Global { .. }
        | StmtKind::UseDecl { .. }
        | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. }
        | StmtKind::ExternGlobalDecl { .. } => false,

        StmtKind::Echo(expr) | StmtKind::Throw(expr) | StmtKind::ExprStmt(expr) => {
            expr_refs(expr, target)
        }
        StmtKind::Assign { value, .. } => expr_refs(value, target),
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            expr_refs(condition, target)
                || then_body.iter().any(|stmt| stmt_refs(stmt, target))
                || elseif_clauses.iter().any(|(cond, body)| {
                    expr_refs(cond, target) || body.iter().any(|stmt| stmt_refs(stmt, target))
                })
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(|stmt| stmt_refs(stmt, target)))
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            then_body.iter().any(|stmt| stmt_refs(stmt, target))
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(|stmt| stmt_refs(stmt, target)))
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            expr_refs(condition, target) || body.iter().any(|stmt| stmt_refs(stmt, target))
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            init.as_deref().is_some_and(|stmt| stmt_refs(stmt, target))
                || condition.as_ref().is_some_and(|expr| expr_refs(expr, target))
                || update.as_deref().is_some_and(|stmt| stmt_refs(stmt, target))
                || body.iter().any(|stmt| stmt_refs(stmt, target))
        }
        StmtKind::ArrayAssign { index, value, .. } => {
            expr_refs(index, target) || expr_refs(value, target)
        }
        StmtKind::NestedArrayAssign { target: t, value } => {
            expr_refs(t, target) || expr_refs(value, target)
        }
        StmtKind::ArrayPush { value, .. } => expr_refs(value, target),
        StmtKind::TypedAssign { value, .. } => expr_refs(value, target),
        StmtKind::Foreach { array, body, .. } => {
            expr_refs(array, target) || body.iter().any(|stmt| stmt_refs(stmt, target))
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            expr_refs(subject, target)
                || cases.iter().any(|(conditions, body)| {
                    conditions.iter().any(|cond| expr_refs(cond, target))
                        || body.iter().any(|stmt| stmt_refs(stmt, target))
                })
                || default
                    .as_ref()
                    .is_some_and(|body| body.iter().any(|stmt| stmt_refs(stmt, target)))
        }
        StmtKind::Include { path, .. } => expr_refs(path, target),
        StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. } => body.iter().any(|stmt| stmt_refs(stmt, target)),
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            try_body.iter().any(|stmt| stmt_refs(stmt, target))
                || catches
                    .iter()
                    .any(|catch| catch.body.iter().any(|stmt| stmt_refs(stmt, target)))
                || finally_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(|stmt| stmt_refs(stmt, target)))
        }
        StmtKind::FunctionDecl { params, body, .. } => {
            params_ref(params, target) || body.iter().any(|stmt| stmt_refs(stmt, target))
        }
        StmtKind::Return(value) => value.as_ref().is_some_and(|expr| expr_refs(expr, target)),
        StmtKind::ConstDecl { value, .. } => expr_refs(value, target),
        StmtKind::ListUnpack { value, .. } => expr_refs(value, target),
        StmtKind::StaticVar { init, .. } => expr_refs(init, target),
        StmtKind::ClassDecl {
            trait_uses,
            properties,
            methods,
            constants,
            ..
        } => {
            trait_uses.iter().any(|tu| trait_use_refs(tu, target))
                || properties.iter().any(|p| class_property_refs(p, target))
                || methods.iter().any(|m| class_method_refs(m, target))
                || constants.iter().any(|c| class_const_refs(c, target))
        }
        StmtKind::EnumDecl { cases, .. } => cases.iter().any(|case| enum_case_refs(case, target)),
        StmtKind::PackedClassDecl { fields, .. } => {
            fields.iter().any(|f| packed_field_refs(f, target))
        }
        StmtKind::InterfaceDecl {
            properties,
            methods,
            constants,
            ..
        } => {
            properties.iter().any(|p| class_property_refs(p, target))
                || methods.iter().any(|m| class_method_refs(m, target))
                || constants.iter().any(|c| class_const_refs(c, target))
        }
        StmtKind::TraitDecl {
            trait_uses,
            properties,
            methods,
            constants,
            ..
        } => {
            trait_uses.iter().any(|tu| trait_use_refs(tu, target))
                || properties.iter().any(|p| class_property_refs(p, target))
                || methods.iter().any(|m| class_method_refs(m, target))
                || constants.iter().any(|c| class_const_refs(c, target))
        }
        StmtKind::PropertyAssign { object, value, .. } => {
            expr_refs(object, target) || expr_refs(value, target)
        }
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => expr_refs(value, target),
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            expr_refs(index, target) || expr_refs(value, target)
        }
        StmtKind::PropertyArrayPush { object, value, .. } => {
            expr_refs(object, target) || expr_refs(value, target)
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            expr_refs(object, target)
                || expr_refs(index, target)
                || expr_refs(value, target)
        }
    }
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for the generic OPcache-usage AST walk over both prelude function
    //! names: a procedural call, a string reference (function_exists/callable), and a
    //! nested reference are detected; an unrelated program is not; and a user
    //! declaration is recognized. Exercised for `opcache_get_configuration` and
    //! `opcache_reset`.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.

    use super::*;

    /// The two OPcache prelude function names, exercised by every reference test.
    const GET_CONFIGURATION: &str = "opcache_get_configuration";
    const RESET: &str = "opcache_reset";

    /// Parses source the way `inject_if_used` sees it: tokenize then parse.
    fn parse(source: &str) -> Vec<Stmt> {
        let tokens = crate::lexer::tokenize(source).expect("test source must tokenize");
        crate::parser::parse(&tokens).expect("test source must parse")
    }

    /// A procedural `opcache_get_configuration()` call is detected.
    #[test]
    fn detects_procedural_call() {
        assert!(program_references(
            &parse(r#"<?php $c = opcache_get_configuration();"#),
            GET_CONFIGURATION
        ));
    }

    /// A procedural `opcache_reset()` call is detected.
    #[test]
    fn detects_reset_call() {
        assert!(program_references(
            &parse(r#"<?php var_dump(opcache_reset());"#),
            RESET
        ));
    }

    /// An `"opcache_reset"` string (function_exists/callable) is detected.
    #[test]
    fn detects_reset_string_reference() {
        assert!(program_references(
            &parse(r#"<?php if (function_exists("opcache_reset")) { echo "y"; }"#),
            RESET
        ));
    }

    /// A nested reference inside a function body is detected.
    #[test]
    fn detects_nested_reference() {
        assert!(program_references(
            &parse(r#"<?php function f() { return opcache_reset(); }"#),
            RESET
        ));
    }

    /// Case-insensitive matching, as PHP function names are.
    #[test]
    fn detects_case_insensitive() {
        assert!(program_references(&parse(r#"<?php OPCACHE_RESET();"#), RESET));
    }

    /// Detection is per-name: a program using only `opcache_reset` does not count as
    /// referencing `opcache_get_configuration`.
    #[test]
    fn reference_detection_is_per_name() {
        let program = parse(r#"<?php opcache_reset();"#);
        assert!(program_references(&program, RESET));
        assert!(!program_references(&program, GET_CONFIGURATION));
    }

    /// A program with no reference is not detected.
    #[test]
    fn ignores_unrelated_program() {
        let program = parse(r#"<?php $a = [1, 2]; echo count($a);"#);
        assert!(!program_references(&program, GET_CONFIGURATION));
        assert!(!program_references(&program, RESET));
    }

    /// A user-declared function is recognized so the prelude is skipped, per name.
    #[test]
    fn detects_user_declaration() {
        let program = parse(r#"<?php function opcache_reset(): bool { return true; }"#);
        assert!(program_declares(&program, RESET));
        assert!(!program_declares(&program, GET_CONFIGURATION));
    }

    /// A program that only calls it does not count as declaring it.
    #[test]
    fn call_is_not_a_declaration() {
        assert!(!program_declares(
            &parse(r#"<?php opcache_reset();"#),
            RESET
        ));
    }
}
