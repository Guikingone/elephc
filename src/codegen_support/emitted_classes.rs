//! Purpose:
//! Selects every class descriptor required by generated code and dynamic runtime factories.
//!
//! Called from:
//! - "crate::codegen_support::runtime_features" while deriving runtime requirements.
//!
//! Key details:
//! - Expands inheritance, implementation, method-body, reflection, and internal factory dependencies to a fixed point.

use super::program_usage::{collect_required_class_names, collect_required_class_names_in_stmts};
use super::{dynamic_new, reflection};
use crate::parser::ast::{Expr, ExprKind, Program, Stmt, StmtKind};
use crate::types::ClassInfo;
use std::collections::{HashMap, HashSet};

/// Returns the set of class names that should be emitted in the
/// user-asm section. Starts from required classes, unconditionally includes
/// the throwable hierarchy (needed by runtime JSON helpers), reflection
/// classes, and attribute factories, then expands to cover the full
/// inheritance and implementation dependency chain.
pub(super) fn collect_emitted_class_names(
    program: &Program,
    classes: &HashMap<String, ClassInfo>,
) -> HashSet<String> {
    let mut names = collect_required_class_names(program);
    if names.contains("Fiber") {
        names.insert("FiberError".to_string());
    }
    // Seed the throwable hierarchy unconditionally: json_encode /
    // json_decode / json_validate can throw JsonException at runtime
    // through JSON_THROW_ON_ERROR even when user code only catches a
    // wider type (e.g. `catch (Exception $e)`). Without these
    // descriptors in the user-asm tables, the catch-time inheritance
    // walk in __rt_exception_matches sees a -1 parent for the thrown
    // class and reports no match.
    // The Error branch is seeded whole for the same reason as the JSON note above:
    // `intdiv($a, 0)` throws a DivisionByZeroError from a codegen guard that has no
    // EIR class reference, so its descriptor (and every ancestor's) must be present
    // or __rt_exception_matches walks into a -1 parent and reports no match.
    for builtin in [
        "Throwable",
        "Error",
        "TypeError",
        "ArgumentCountError",
        "ValueError",
        "ArithmeticError",
        "DivisionByZeroError",
        "AssertionError",
        "UnhandledMatchError",
        "Exception",
        "LogicException",
        "RuntimeException",
        "JsonException",
        "InvalidArgumentException",
        "OutOfBoundsException",
        "OutOfRangeException",
    ] {
        names.insert(builtin.to_string());
    }
    for builtin in [
        "ReflectionAttribute",
        "ReflectionClass",
        "ReflectionMethod",
        "ReflectionProperty",
    ] {
        names.insert(builtin.to_string());
    }
    for factory in reflection::collect_attribute_factories(classes) {
        // Only resolvable attribute classes are emitted; non-class attributes
        // are registered solely so `getArguments()` can return their arguments.
        if factory.resolvable {
            names.insert(factory.class_name);
        }
    }
    collect_dynamic_object_factory_classes(program, classes, &mut names);
    expand_emitted_class_dependencies(&mut names, classes);
    names
}

/// Repeatedly expands `names` by adding parent classes and all
/// method-implementation classes (both instance and static) until a
/// fixed point is reached, ensuring emitted vtables and interface
/// tables are complete.
fn expand_emitted_class_dependencies(
    names: &mut HashSet<String>,
    classes: &HashMap<String, ClassInfo>,
) {
    loop {
        let mut changed = false;
        let snapshot: Vec<String> = names.iter().cloned().collect();
        for class_name in snapshot {
            let Some(class_info) = classes.get(&class_name) else {
                continue;
            };
            if let Some(parent) = &class_info.parent {
                changed |= names.insert(parent.clone());
            }
            for impl_class in class_info
                .method_impl_classes
                .values()
                .chain(class_info.static_method_impl_classes.values())
            {
                changed |= names.insert(impl_class.clone());
            }
            let previous_len = names.len();
            for method in &class_info.method_decls {
                collect_dynamic_object_factory_classes(&method.body, classes, names);
                collect_required_class_names_in_stmts(&method.body, names);
            }
            changed |= names.len() != previous_len;
        }
        if !changed {
            break;
        }
    }
}

/// Adds every concrete class that an internal dynamic object factory can instantiate.
fn collect_dynamic_object_factory_classes(
    stmts: &[Stmt],
    classes: &HashMap<String, ClassInfo>,
    names: &mut HashSet<String>,
) {
    for stmt in stmts {
        collect_dynamic_object_factory_classes_in_stmt(stmt, classes, names);
    }
}

/// Adds dynamic factory class dependencies found in a statement.
fn collect_dynamic_object_factory_classes_in_stmt(
    stmt: &Stmt,
    classes: &HashMap<String, ClassInfo>,
    names: &mut HashSet<String>,
) {
    match &stmt.kind {
        StmtKind::ClassDecl { methods, .. }
        | StmtKind::TraitDecl { methods, .. }
        | StmtKind::InterfaceDecl { methods, .. } => methods.iter().for_each(|method| {
            collect_dynamic_object_factory_classes(&method.body, classes, names)
        }),
        StmtKind::FunctionDecl { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. } => {
            collect_dynamic_object_factory_classes(body, classes, names);
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            collect_dynamic_object_factory_classes(try_body, classes, names);
            for catch in catches {
                collect_dynamic_object_factory_classes(&catch.body, classes, names);
            }
            if let Some(finally_body) = finally_body {
                collect_dynamic_object_factory_classes(finally_body, classes, names);
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            collect_dynamic_object_factory_classes(then_body, classes, names);
            if let Some(else_body) = else_body {
                collect_dynamic_object_factory_classes(else_body, classes, names);
            }
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_dynamic_object_factory_classes_in_expr(condition, classes, names);
            collect_dynamic_object_factory_classes(then_body, classes, names);
            for (condition, body) in elseif_clauses {
                collect_dynamic_object_factory_classes_in_expr(condition, classes, names);
                collect_dynamic_object_factory_classes(body, classes, names);
            }
            if let Some(else_body) = else_body {
                collect_dynamic_object_factory_classes(else_body, classes, names);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { condition, body } => {
            collect_dynamic_object_factory_classes_in_expr(condition, classes, names);
            collect_dynamic_object_factory_classes(body, classes, names);
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(init) = init {
                collect_dynamic_object_factory_classes_in_stmt(init, classes, names);
            }
            if let Some(condition) = condition {
                collect_dynamic_object_factory_classes_in_expr(condition, classes, names);
            }
            if let Some(update) = update {
                collect_dynamic_object_factory_classes_in_stmt(update, classes, names);
            }
            collect_dynamic_object_factory_classes(body, classes, names);
        }
        StmtKind::Foreach { array, body, .. } => {
            collect_dynamic_object_factory_classes_in_expr(array, classes, names);
            collect_dynamic_object_factory_classes(body, classes, names);
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            collect_dynamic_object_factory_classes_in_expr(subject, classes, names);
            for (patterns, body) in cases {
                for pattern in patterns {
                    collect_dynamic_object_factory_classes_in_expr(pattern, classes, names);
                }
                collect_dynamic_object_factory_classes(body, classes, names);
            }
            if let Some(default) = default {
                collect_dynamic_object_factory_classes(default, classes, names);
            }
        }
        StmtKind::Echo(expr)
        | StmtKind::Throw(expr)
        | StmtKind::ExprStmt(expr)
        | StmtKind::ConstDecl { value: expr, .. }
        | StmtKind::Assign { value: expr, .. }
        | StmtKind::TypedAssign { value: expr, .. }
        | StmtKind::StaticVar { init: expr, .. }
        | StmtKind::ListUnpack { value: expr, .. }
        | StmtKind::Return(Some(expr))
        | StmtKind::ArrayPush { value: expr, .. }
        | StmtKind::PropertyAssign { value: expr, .. }
        | StmtKind::PropertyArrayPush { value: expr, .. }
        | StmtKind::StaticPropertyAssign { value: expr, .. }
        | StmtKind::StaticPropertyArrayPush { value: expr, .. } => {
            collect_dynamic_object_factory_classes_in_expr(expr, classes, names);
        }
        StmtKind::ArrayAssign { index, value, .. }
        | StmtKind::PropertyArrayAssign { index, value, .. }
        | StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            collect_dynamic_object_factory_classes_in_expr(index, classes, names);
            collect_dynamic_object_factory_classes_in_expr(value, classes, names);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            collect_dynamic_object_factory_classes_in_expr(target, classes, names);
            collect_dynamic_object_factory_classes_in_expr(value, classes, names);
        }
        _ => {}
    }
}

/// Adds dynamic factory class dependencies found in an expression.
fn collect_dynamic_object_factory_classes_in_expr(
    expr: &Expr,
    classes: &HashMap<String, ClassInfo>,
    names: &mut HashSet<String>,
) {
    match &expr.kind {
        ExprKind::NewDynamicObject {
            class_name,
            required_parent,
            args,
            ..
        } => {
            collect_dynamic_factory_descendants(required_parent.as_str(), classes, names);
            collect_dynamic_object_factory_classes_in_expr(class_name, classes, names);
            for arg in args {
                collect_dynamic_object_factory_classes_in_expr(arg, classes, names);
            }
        }
        ExprKind::BinaryOp { left, right, .. } => {
            collect_dynamic_object_factory_classes_in_expr(left, classes, names);
            collect_dynamic_object_factory_classes_in_expr(right, classes, names);
        }
        ExprKind::InstanceOf { value, target } => {
            collect_dynamic_object_factory_classes_in_expr(value, classes, names);
            if let crate::parser::ast::InstanceOfTarget::Expr(expr) = target {
                collect_dynamic_object_factory_classes_in_expr(expr, classes, names);
            }
        }
        ExprKind::Negate(expr)
        | ExprKind::Not(expr)
        | ExprKind::BitNot(expr)
        | ExprKind::Throw(expr)
        | ExprKind::ErrorSuppress(expr)
        | ExprKind::Print(expr)
        | ExprKind::Spread(expr)
        | ExprKind::Cast { expr, .. }
        | ExprKind::PtrCast { expr, .. } => {
            collect_dynamic_object_factory_classes_in_expr(expr, classes, names);
        }
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
            collect_dynamic_object_factory_classes_in_expr(value, classes, names);
            collect_dynamic_object_factory_classes_in_expr(default, classes, names);
        }
        ExprKind::Pipe { value, callable } => {
            collect_dynamic_object_factory_classes_in_expr(value, classes, names);
            collect_dynamic_object_factory_classes_in_expr(callable, classes, names);
        }
        ExprKind::Assignment {
            target,
            value,
            result_target,
            prelude,
            ..
        } => {
            collect_dynamic_object_factory_classes(prelude, classes, names);
            collect_dynamic_object_factory_classes_in_expr(target, classes, names);
            collect_dynamic_object_factory_classes_in_expr(value, classes, names);
            if let Some(result_target) = result_target {
                collect_dynamic_object_factory_classes_in_expr(result_target, classes, names);
            }
        }
        ExprKind::FunctionCall { args, .. }
        | ExprKind::ClosureCall { args, .. }
        | ExprKind::StaticMethodCall { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewScopedObject { args, .. } => {
            for arg in args {
                collect_dynamic_object_factory_classes_in_expr(arg, classes, names);
            }
        }
        ExprKind::NewDynamic { name_expr, args } => {
            for class_name in dynamic_new::supported_dynamic_new_builtin_class_names() {
                if classes.contains_key(*class_name) {
                    names.insert((*class_name).to_string());
                }
            }
            collect_dynamic_object_factory_classes_in_expr(name_expr, classes, names);
            for arg in args {
                collect_dynamic_object_factory_classes_in_expr(arg, classes, names);
            }
        }
        ExprKind::ExprCall { callee, args } => {
            collect_dynamic_object_factory_classes_in_expr(callee, classes, names);
            for arg in args {
                collect_dynamic_object_factory_classes_in_expr(arg, classes, names);
            }
        }
        ExprKind::ArrayLiteral(items) => {
            for item in items {
                collect_dynamic_object_factory_classes_in_expr(item, classes, names);
            }
        }
        ExprKind::ArrayLiteralAssoc(items) => {
            for (key, value) in items {
                collect_dynamic_object_factory_classes_in_expr(key, classes, names);
                collect_dynamic_object_factory_classes_in_expr(value, classes, names);
            }
        }
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            collect_dynamic_object_factory_classes_in_expr(subject, classes, names);
            for (patterns, value) in arms {
                for pattern in patterns {
                    collect_dynamic_object_factory_classes_in_expr(pattern, classes, names);
                }
                collect_dynamic_object_factory_classes_in_expr(value, classes, names);
            }
            if let Some(default) = default {
                collect_dynamic_object_factory_classes_in_expr(default, classes, names);
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            collect_dynamic_object_factory_classes_in_expr(array, classes, names);
            collect_dynamic_object_factory_classes_in_expr(index, classes, names);
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_dynamic_object_factory_classes_in_expr(condition, classes, names);
            collect_dynamic_object_factory_classes_in_expr(then_expr, classes, names);
            collect_dynamic_object_factory_classes_in_expr(else_expr, classes, names);
        }
        ExprKind::Closure { body, .. } => {
            collect_dynamic_object_factory_classes(body, classes, names)
        }
        ExprKind::NamedArg { value, .. } => {
            collect_dynamic_object_factory_classes_in_expr(value, classes, names);
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. } => {
            collect_dynamic_object_factory_classes_in_expr(object, classes, names);
        }
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            collect_dynamic_object_factory_classes_in_expr(object, classes, names);
            collect_dynamic_object_factory_classes_in_expr(property, classes, names);
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            collect_dynamic_object_factory_classes_in_expr(object, classes, names);
            for arg in args {
                collect_dynamic_object_factory_classes_in_expr(arg, classes, names);
            }
        }
        ExprKind::FirstClassCallable(crate::parser::ast::CallableTarget::Method {
            object, ..
        }) => collect_dynamic_object_factory_classes_in_expr(object, classes, names),
        ExprKind::BufferNew { len, .. } => {
            collect_dynamic_object_factory_classes_in_expr(len, classes, names);
        }
        ExprKind::Yield { key, value } => {
            if let Some(key) = key {
                collect_dynamic_object_factory_classes_in_expr(key, classes, names);
            }
            if let Some(value) = value {
                collect_dynamic_object_factory_classes_in_expr(value, classes, names);
            }
        }
        ExprKind::YieldFrom(inner) => {
            collect_dynamic_object_factory_classes_in_expr(inner, classes, names);
        }
        _ => {}
    }
}

/// Adds every known class that can satisfy an internal dynamic factory parent constraint.
fn collect_dynamic_factory_descendants(
    required_parent: &str,
    classes: &HashMap<String, ClassInfo>,
    names: &mut HashSet<String>,
) {
    for class_name in classes.keys() {
        if emitted_class_descends_from(class_name, required_parent, classes) {
            names.insert(class_name.clone());
        }
    }
}

/// Returns true if `class_name` is the required class or extends it.
fn emitted_class_descends_from(
    class_name: &str,
    required_parent: &str,
    classes: &HashMap<String, ClassInfo>,
) -> bool {
    let mut current = Some(class_name);
    while let Some(name) = current {
        if crate::names::php_symbol_key(name.trim_start_matches('\\'))
            == crate::names::php_symbol_key(required_parent.trim_start_matches('\\'))
        {
            return true;
        }
        current = classes.get(name).and_then(|info| info.parent.as_deref());
    }
    false
}
