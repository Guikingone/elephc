//! Purpose:
//! Decides whether a parsed program references PHP's `register_shutdown_function` (so the prelude is
//! injected only for programs that use it) and whether it already declares its own
//! `register_shutdown_function` function (so a user definition is never clobbered).
//!
//! Called from:
//! - `crate::shutdown_prelude::inject_if_used`.
//!
//! Key details:
//! - Runs after name resolution (the injection point in `pipeline::compile` is after
//!   `autoload::run` and the conditional-function hoist), so function `Name`s are the
//!   canonicalized forms produced by the name resolver; `name_is_register_shutdown_function` matches on
//!   the unqualified last segment, which is `register_shutdown_function` whether the call was written
//!   bare, as `\register_shutdown_function`, or as a namespaced `N\register_shutdown_function` that the prelude-global
//!   fallback rewrote to the global `register_shutdown_function`. PHP function names are case-insensitive.
//! - A `"register_shutdown_function"` string literal also counts as a reference so the
//!   `function_exists('register_shutdown_function')` and `'register_shutdown_function'` callable forms still inject
//!   the function. Over-injection (e.g. an unrelated string) only adds a small, later
//!   dead-code-eliminated function; soundness (never missing a real use) is what
//!   matters, so the `match`es are exhaustive with no wildcard arm.

use crate::names::Name;
use crate::parser::ast::{
    CallableTarget, ClassConst, ClassMethod, ClassProperty, EnumCaseDecl, Expr, ExprKind,
    InstanceOfTarget, PackedField, Stmt, StmtKind, TraitUse, TypeExpr,
};

/// Returns whether any top-level statement references `register_shutdown_function`, so the prelude
/// must be injected ahead of user code.
pub(super) fn program_references_register_shutdown_function(program: &[Stmt]) -> bool {
    program.iter().any(stmt_refs_rsf)
}

/// Returns whether the program already declares its own `register_shutdown_function` function (at top
/// level or inside a namespace/guard/synthetic block), in which case the prelude must
/// not be injected so the user definition wins and there is no redeclaration error.
pub(super) fn program_declares_register_shutdown_function(program: &[Stmt]) -> bool {
    program.iter().any(stmt_declares_register_shutdown_function)
}

/// Returns whether a function name is `register_shutdown_function`, compared case-insensitively on its
/// unqualified last segment.
fn name_is_register_shutdown_function(name: &Name) -> bool {
    name.last_segment()
        .is_some_and(|segment| segment.eq_ignore_ascii_case("register_shutdown_function"))
}

/// Returns whether a statement declares a top-level `register_shutdown_function` function, recursing
/// only into the block forms that can host a hoisted function declaration.
fn stmt_declares_register_shutdown_function(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::FunctionDecl { name, .. } => name.eq_ignore_ascii_case("register_shutdown_function"),
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => body.iter().any(stmt_declares_register_shutdown_function),
        _ => false,
    }
}

/// Returns whether a first-class-callable target references `register_shutdown_function` via a function
/// name; method/static-method targets cannot name `register_shutdown_function` but their receiver is
/// still walked for nested references.
fn callable_target_refs_rsf(target: &CallableTarget) -> bool {
    match target {
        CallableTarget::Function(name) => name_is_register_shutdown_function(name),
        CallableTarget::StaticMethod { .. } => false,
        CallableTarget::Method { object, .. } => expr_refs_rsf(object),
    }
}

/// Returns whether any parameter's default value references `register_shutdown_function` (type hints
/// cannot). Shared by function, method, and closure parameter lists.
fn params_ref_rsf(params: &[(String, Option<TypeExpr>, Option<Expr>, bool)]) -> bool {
    params
        .iter()
        .any(|(_, _, default, _)| default.as_ref().is_some_and(expr_refs_rsf))
}

/// Returns whether a `use Trait` clause references `register_shutdown_function`; trait/method names in
/// adaptations are not call sites, so this is always false.
fn trait_use_refs_rsf(_trait_use: &TraitUse) -> bool {
    false
}

/// Returns whether a class property's default value references `register_shutdown_function`.
fn class_property_refs_rsf(property: &ClassProperty) -> bool {
    property.default.as_ref().is_some_and(expr_refs_rsf)
}

/// Returns whether a method's parameter defaults or body reference `register_shutdown_function`.
fn class_method_refs_rsf(method: &ClassMethod) -> bool {
    params_ref_rsf(&method.params) || method.body.iter().any(stmt_refs_rsf)
}

/// Returns whether a class constant's initializer references `register_shutdown_function`.
fn class_const_refs_rsf(constant: &ClassConst) -> bool {
    expr_refs_rsf(&constant.value)
}

/// Returns whether an enum case's backing-value expression references `register_shutdown_function`.
fn enum_case_refs_rsf(case: &EnumCaseDecl) -> bool {
    case.value.as_ref().is_some_and(expr_refs_rsf)
}

/// Returns whether a `packed class` field references `register_shutdown_function`; packed fields carry
/// only types, never call sites.
fn packed_field_refs_rsf(_field: &PackedField) -> bool {
    false
}

/// Returns whether an `instanceof` target's runtime-expression operand references
/// `register_shutdown_function` (name targets are class positions, never call sites).
fn instanceof_target_refs_rsf(target: &InstanceOfTarget) -> bool {
    match target {
        InstanceOfTarget::Name(_) => false,
        InstanceOfTarget::Expr(expr) => expr_refs_rsf(expr),
    }
}

/// Returns whether an expression references `register_shutdown_function` at any call position or as a
/// `"register_shutdown_function"` string literal, recursing into every child. The `match` is exhaustive
/// so a new `ExprKind` cannot silently bypass detection.
fn expr_refs_rsf(expr: &Expr) -> bool {
    match &expr.kind {
        // Main-side dynamic call/class-name forms: recurse into evaluated children.
        ExprKind::NullsafeDynamicMethodCall { object, method, args } => {
            expr_refs_rsf(object) || expr_refs_rsf(method) || args.iter().any(expr_refs_rsf)
        }
        ExprKind::ObjectClassName { object } => expr_refs_rsf(object),
        // `require`/`include` in expression position: recurse into the path expression. This is a
        // transient parser node expanded by the resolver before later passes, but the match must
        // stay exhaustive so a new `ExprKind` cannot silently bypass detection.
        ExprKind::IncludeValue { path, .. } => expr_refs_rsf(path),
        // A "register_shutdown_function" string literal counts (function_exists/callable forms).
        ExprKind::StringLiteral(value) => value.eq_ignore_ascii_case("register_shutdown_function"),

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
            name_is_register_shutdown_function(name) || args.iter().any(expr_refs_rsf)
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            expr_refs_rsf(object) || args.iter().any(expr_refs_rsf)
        }
        ExprKind::StaticMethodCall { args, .. } => args.iter().any(expr_refs_rsf),
        ExprKind::FirstClassCallable(target) => callable_target_refs_rsf(target),

        ExprKind::BinaryOp { left, right, .. } => expr_refs_rsf(left) || expr_refs_rsf(right),
        ExprKind::InstanceOf { value, target } => {
            expr_refs_rsf(value) || instanceof_target_refs_rsf(target)
        }
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::Clone(inner)
        | ExprKind::YieldFrom(inner) => expr_refs_rsf(inner),
        ExprKind::NullCoalesce { value, default }
        | ExprKind::ShortTernary { value, default } => {
            expr_refs_rsf(value) || expr_refs_rsf(default)
        }
        ExprKind::Pipe { value, callable } => expr_refs_rsf(value) || expr_refs_rsf(callable),
        ExprKind::ListUnpack { value, .. } => expr_refs_rsf(value),
        ExprKind::Assignment {
            target,
            value,
            result_target,
            prelude,
            ..
        } => {
            expr_refs_rsf(target)
                || expr_refs_rsf(value)
                || result_target.as_deref().is_some_and(expr_refs_rsf)
                || prelude.iter().any(stmt_refs_rsf)
        }
        ExprKind::ClosureCall { args, .. } => args.iter().any(expr_refs_rsf),
        ExprKind::ArrayLiteral(items) => items.iter().any(expr_refs_rsf),
        ExprKind::ArrayLiteralAssoc(pairs) => pairs
            .iter()
            .any(|(key, value)| expr_refs_rsf(key) || expr_refs_rsf(value)),
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            expr_refs_rsf(subject)
                || arms.iter().any(|(conditions, body)| {
                    conditions.iter().any(expr_refs_rsf) || expr_refs_rsf(body)
                })
                || default.as_deref().is_some_and(expr_refs_rsf)
        }
        ExprKind::ArrayAccess { array, index } => expr_refs_rsf(array) || expr_refs_rsf(index),
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => expr_refs_rsf(condition) || expr_refs_rsf(then_expr) || expr_refs_rsf(else_expr),
        ExprKind::Cast { expr, .. } | ExprKind::PtrCast { expr, .. } => expr_refs_rsf(expr),
        ExprKind::Closure { params, body, .. } => {
            params_ref_rsf(params) || body.iter().any(stmt_refs_rsf)
        }
        ExprKind::NamedArg { value, .. } => expr_refs_rsf(value),
        ExprKind::ExprCall { callee, args } => {
            expr_refs_rsf(callee) || args.iter().any(expr_refs_rsf)
        }
        ExprKind::NewObject { args, .. } => args.iter().any(expr_refs_rsf),
        ExprKind::NewDynamic { name_expr, args } => {
            expr_refs_rsf(name_expr) || args.iter().any(expr_refs_rsf)
        }
        ExprKind::NewDynamicObject {
            class_name, args, ..
        } => expr_refs_rsf(class_name) || args.iter().any(expr_refs_rsf),
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. } => expr_refs_rsf(object),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            expr_refs_rsf(object) || expr_refs_rsf(property)
        }
        ExprKind::StaticPropertyAccess { .. } => false,
        ExprKind::BufferNew { len, .. } => expr_refs_rsf(len),
        ExprKind::ClassConstant { .. } | ExprKind::ScopedConstantAccess { .. } => false,
        // `$obj::CONST` — recurse into the evaluated object expression.
        ExprKind::DynamicClassConstantAccess { object, .. } => expr_refs_rsf(object),
        ExprKind::DynamicStaticPropertyAccess { property, .. } => expr_refs_rsf(property),
        ExprKind::NewScopedObject { args, .. } => args.iter().any(expr_refs_rsf),
        ExprKind::Yield { key, value } => {
            key.as_deref().is_some_and(expr_refs_rsf)
                || value.as_deref().is_some_and(expr_refs_rsf)
        }
    }
}

/// Returns whether a statement references `register_shutdown_function` at any call position or string
/// literal, recursing into nested statements, expressions, and class members. The
/// `match` is exhaustive so a new `StmtKind` cannot silently bypass detection.
fn stmt_refs_rsf(stmt: &Stmt) -> bool {
    match &stmt.kind {
        // Statements with no call position and no child expr/stmt.
        StmtKind::RefAssign { .. }
        | StmtKind::RefAssignToTarget { .. }
        | StmtKind::IncludeOnceMark { .. }
        | StmtKind::Break(_)
        | StmtKind::Continue(_)
        | StmtKind::Goto(_)
        | StmtKind::Label(_)
        | StmtKind::NamespaceDecl { .. }
        | StmtKind::FunctionVariantGroup { .. }
        | StmtKind::FunctionVariantMark { .. }
        | StmtKind::Global { .. }
        | StmtKind::UseDecl { .. }
        | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. }
        | StmtKind::ExternGlobalDecl { .. } => false,

        StmtKind::Echo(expr) | StmtKind::Throw(expr) | StmtKind::ExprStmt(expr) => {
            expr_refs_rsf(expr)
        }
        StmtKind::Assign { value, .. } => expr_refs_rsf(value),
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            expr_refs_rsf(condition)
                || then_body.iter().any(stmt_refs_rsf)
                || elseif_clauses
                    .iter()
                    .any(|(cond, body)| expr_refs_rsf(cond) || body.iter().any(stmt_refs_rsf))
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(stmt_refs_rsf))
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            then_body.iter().any(stmt_refs_rsf)
                || else_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(stmt_refs_rsf))
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            expr_refs_rsf(condition) || body.iter().any(stmt_refs_rsf)
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            init.as_deref().is_some_and(stmt_refs_rsf)
                || condition.as_ref().is_some_and(expr_refs_rsf)
                || update.as_deref().is_some_and(stmt_refs_rsf)
                || body.iter().any(stmt_refs_rsf)
        }
        StmtKind::ArrayAssign { index, value, .. } => expr_refs_rsf(index) || expr_refs_rsf(value),
        StmtKind::NestedArrayAssign { target, value } => {
            expr_refs_rsf(target) || expr_refs_rsf(value)
        }
        StmtKind::ArrayPush { value, .. } => expr_refs_rsf(value),
        StmtKind::TypedAssign { value, .. } => expr_refs_rsf(value),
        StmtKind::Foreach { array, body, .. } => {
            expr_refs_rsf(array) || body.iter().any(stmt_refs_rsf)
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            expr_refs_rsf(subject)
                || cases.iter().any(|(conditions, body)| {
                    conditions.iter().any(expr_refs_rsf) || body.iter().any(stmt_refs_rsf)
                })
                || default
                    .as_ref()
                    .is_some_and(|body| body.iter().any(stmt_refs_rsf))
        }
        StmtKind::Include { path, .. } => expr_refs_rsf(path),
        StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. } => body.iter().any(stmt_refs_rsf),
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            try_body.iter().any(stmt_refs_rsf)
                || catches.iter().any(|catch| catch.body.iter().any(stmt_refs_rsf))
                || finally_body
                    .as_ref()
                    .is_some_and(|body| body.iter().any(stmt_refs_rsf))
        }
        StmtKind::FunctionDecl { params, body, .. } => {
            params_ref_rsf(params) || body.iter().any(stmt_refs_rsf)
        }
        StmtKind::Return(value) => value.as_ref().is_some_and(expr_refs_rsf),
        StmtKind::ConstDecl { value, .. } => expr_refs_rsf(value),
        StmtKind::ListUnpack { value, .. } => expr_refs_rsf(value),
        StmtKind::StaticVar { init, .. } => expr_refs_rsf(init),
        StmtKind::ClassDecl {
            trait_uses,
            properties,
            methods,
            constants,
            ..
        } => {
            trait_uses.iter().any(trait_use_refs_rsf)
                || properties.iter().any(class_property_refs_rsf)
                || methods.iter().any(class_method_refs_rsf)
                || constants.iter().any(class_const_refs_rsf)
        }
        StmtKind::EnumDecl { cases, .. } => cases.iter().any(enum_case_refs_rsf),
        StmtKind::PackedClassDecl { fields, .. } => fields.iter().any(packed_field_refs_rsf),
        StmtKind::InterfaceDecl {
            properties,
            methods,
            constants,
            ..
        } => {
            properties.iter().any(class_property_refs_rsf)
                || methods.iter().any(class_method_refs_rsf)
                || constants.iter().any(class_const_refs_rsf)
        }
        StmtKind::TraitDecl {
            trait_uses,
            properties,
            methods,
            constants,
            ..
        } => {
            trait_uses.iter().any(trait_use_refs_rsf)
                || properties.iter().any(class_property_refs_rsf)
                || methods.iter().any(class_method_refs_rsf)
                || constants.iter().any(class_const_refs_rsf)
        }
        StmtKind::PropertyAssign { object, value, .. } => {
            expr_refs_rsf(object) || expr_refs_rsf(value)
        }
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => expr_refs_rsf(value),
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            expr_refs_rsf(index) || expr_refs_rsf(value)
        }
        StmtKind::DynamicStaticPropertyWrite { property, index, value, .. } => {
            expr_refs_rsf(property)
                || index.as_ref().is_some_and(expr_refs_rsf)
                || expr_refs_rsf(value)
        }
        StmtKind::PropertyArrayPush { object, value, .. } => {
            expr_refs_rsf(object) || expr_refs_rsf(value)
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => expr_refs_rsf(object) || expr_refs_rsf(index) || expr_refs_rsf(value),
    }
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for the `register_shutdown_function`-usage AST walk: a procedural call, a string
    //! reference (function_exists/callable), and a nested reference are detected, an
    //! unrelated program is not, and a user-declared `register_shutdown_function` is recognized.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Tests parse raw source (pre name-resolution), matching the stage at which
    //!   detection runs inside `inject_if_used`.

    use super::*;

    /// Parses source the way `inject_if_used` sees it: tokenize then parse.
    fn parse(source: &str) -> Vec<Stmt> {
        let tokens = crate::lexer::tokenize(source).expect("test source must tokenize");
        crate::parser::parse(&tokens).expect("test source must parse")
    }

    /// A procedural `register_shutdown_function(...)` call is detected.
    #[test]
    fn detects_procedural_call() {
        assert!(program_references_register_shutdown_function(&parse(
            r#"<?php register_shutdown_function([1, 2]);"#
        )));
    }

    /// A `"register_shutdown_function"` string (function_exists/callable form) is detected.
    #[test]
    fn detects_string_reference() {
        assert!(program_references_register_shutdown_function(&parse(
            r#"<?php if (function_exists("register_shutdown_function")) { echo "y"; }"#
        )));
    }

    /// A nested reference inside a function body is detected.
    #[test]
    fn detects_nested_reference() {
        assert!(program_references_register_shutdown_function(&parse(
            r#"<?php function f($x) { return register_shutdown_function($x, true); }"#
        )));
    }

    /// Case-insensitive matching, as PHP function names are.
    #[test]
    fn detects_case_insensitive() {
        assert!(program_references_register_shutdown_function(&parse(
            r#"<?php REGISTER_SHUTDOWN_FUNCTION($x);"#
        )));
    }

    /// A program with no `register_shutdown_function` use is not detected.
    #[test]
    fn ignores_unrelated_program() {
        assert!(!program_references_register_shutdown_function(&parse(
            r#"<?php $a = [1, 2]; echo count($a);"#
        )));
    }

    /// A user-declared `register_shutdown_function` function is recognized so the prelude is skipped.
    #[test]
    fn detects_user_declaration() {
        assert!(program_declares_register_shutdown_function(&parse(
            r#"<?php function register_shutdown_function($v, $r = false) { return ""; }"#
        )));
    }

    /// A program that only calls `register_shutdown_function` does not count as declaring it.
    #[test]
    fn call_is_not_a_declaration() {
        assert!(!program_declares_register_shutdown_function(&parse(r#"<?php register_shutdown_function($x);"#)));
    }
}
