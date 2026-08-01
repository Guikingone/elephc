//! Purpose:
//! Decides whether a hooked property is BACKED or VIRTUAL, which is what PHP 8.4+ uses to
//! allow or reject a write to a property that declares a `get` hook but no `set` hook.
//!
//! Called from:
//! - `super::stmt_check::assignments::properties` (the property-write read-only check).
//!
//! Key details:
//! - PHP's rule is textual, not semantic: a hook makes the property BACKED when its body names
//!   the property on `$this` — read or written — anywhere outside a nested closure. Measured
//!   against `php -n` 8.5.6 via `ReflectionProperty::isVirtual()`; see the table on
//!   `hooked_property_is_backed`.
//! - The scan deliberately does not descend into `ExprKind::Closure` (arrow functions included):
//!   PHP reports a property whose only `$this->p` sits inside a closure as VIRTUAL.
//! - Under-detection is the safe direction. An unrecognized expression shape falls through to
//!   "not backed", which preserves the pre-existing rejection rather than admitting a write the
//!   lowering might not model.

use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};

use super::Checker;

/// Returns true when `property`'s `get` hook makes it a BACKED property, so a write to it must
/// be allowed and routed to the raw backing slot.
///
/// PHP 8.5.6 (`php -n`) `ReflectionProperty::isVirtual()` matrix that this mirrors:
///
/// | `get` hook body                              | `isVirtual()` | write from outside      |
/// |----------------------------------------------|---------------|-------------------------|
/// | `return $this->p ??= 'def';`                 | `false`       | allowed (backing store) |
/// | `return strtoupper($this->p);`               | `false`       | allowed                 |
/// | `$this->p = 'w'; return 'x';`                | `false`       | allowed                 |
/// | `return match (1) { 1 => $this->p, ... };`   | `false`       | allowed                 |
/// | `return strtoupper($this->other);`           | `true`        | `Error: … is read-only` |
/// | `return $o->p;` (a different object)         | `true`        | `Error: … is read-only` |
/// | `return (fn () => $this->p)();`              | `true`        | `Error: … is read-only` |
///
/// Symfony's `ViewEvent::$controllerArgumentsEvent` is the first row: its hook body is
/// `return $this->controllerArgumentsEvent ??= new ControllerArgumentsEvent(...)`.
///
/// The hook body is read from the `__propget_<property>` accessor the parser desugars hooks
/// into, looked up on the class that actually declares it so an inherited hook still resolves.
/// A hook whose declaration cannot be found is reported as not backed (the safe direction).
pub(crate) fn hooked_property_is_backed(
    checker: &Checker,
    class_name: &str,
    property: &str,
) -> bool {
    let accessor = crate::names::property_hook_get_method(property);
    let accessor_key = crate::names::php_symbol_key(&accessor);
    let Some(class_info) = checker.classes.get(class_name) else {
        return false;
    };
    // A hook declared by an ancestor lives in that ancestor's `method_decls`, not the
    // subclass's, so resolve the declaring class before looking the body up.
    let declaring_class = class_info
        .method_declaring_classes
        .get(&accessor_key)
        .map(String::as_str)
        .unwrap_or(class_name);
    let declaring_info = if declaring_class == class_name {
        class_info
    } else {
        match checker.classes.get(declaring_class) {
            Some(info) => info,
            None => class_info,
        }
    };
    declaring_info
        .method_decls
        .iter()
        .find(|method| crate::names::php_symbol_key(&method.name) == accessor_key)
        .is_some_and(|method| body_names_backing_store(&method.body, property))
}

/// Returns true when any statement in `body` names `$this-><property>` outside a closure.
fn body_names_backing_store(body: &[Stmt], property: &str) -> bool {
    body.iter().any(|stmt| stmt_names_backing_store(stmt, property))
}

/// Returns true when `object` is the literal `$this` receiver.
fn is_this(object: &Expr) -> bool {
    matches!(object.kind, ExprKind::This)
}

/// Returns true when a `$this-><name>` member write targets `property`.
///
/// PHP property names are case-sensitive (unlike method names), so this compares exactly.
fn writes_backing_store(object: &Expr, name: &str, property: &str) -> bool {
    is_this(object) && name == property
}

/// Recursively checks a statement for a mention of `$this-><property>`.
///
/// Nested function, class, trait and interface declarations are skipped: they introduce a new
/// `$this` binding (or none at all), so a `$this->p` inside them says nothing about this hook.
/// Unrecognized statement kinds fall through to `false` — the safe, under-detecting direction.
fn stmt_names_backing_store(stmt: &Stmt, property: &str) -> bool {
    let names_expr = |expr: &Expr| expr_names_backing_store(expr, property);
    let names_body = |body: &[Stmt]| body_names_backing_store(body, property);
    match &stmt.kind {
        StmtKind::FunctionDecl { .. }
        | StmtKind::ClassDecl { .. }
        | StmtKind::TraitDecl { .. }
        | StmtKind::InterfaceDecl { .. } => false,
        // A write to `$this->p` inside the hook backs the property just as a read does
        // (`php -n`: `get { $this->p = 'w'; return 'x'; }` reports `isVirtual() === false`).
        StmtKind::PropertyAssign {
            object,
            property: name,
            value,
        } => writes_backing_store(object, name, property) || names_expr(object) || names_expr(value),
        StmtKind::PropertyArrayPush {
            object,
            property: name,
            value,
        } => writes_backing_store(object, name, property) || names_expr(object) || names_expr(value),
        StmtKind::PropertyArrayAssign {
            object,
            property: name,
            index,
            value,
        } => {
            writes_backing_store(object, name, property)
                || names_expr(object)
                || names_expr(index)
                || names_expr(value)
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            names_body(try_body)
                || catches.iter().any(|c| names_body(&c.body))
                || finally_body.as_ref().is_some_and(|f| names_body(f))
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            names_expr(condition)
                || names_body(then_body)
                || elseif_clauses
                    .iter()
                    .any(|(c, b)| names_expr(c) || names_body(b))
                || else_body.as_ref().is_some_and(|b| names_body(b))
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            names_body(then_body) || else_body.as_ref().is_some_and(|b| names_body(b))
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            names_expr(condition) || names_body(body)
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            init.as_deref()
                .is_some_and(|s| stmt_names_backing_store(s, property))
                || condition.as_ref().is_some_and(names_expr)
                || update
                    .as_deref()
                    .is_some_and(|s| stmt_names_backing_store(s, property))
                || names_body(body)
        }
        StmtKind::Foreach { array, body, .. } => names_expr(array) || names_body(body),
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            names_expr(subject)
                || cases
                    .iter()
                    .any(|(vals, body)| vals.iter().any(names_expr) || names_body(body))
                || default.as_ref().is_some_and(|d| names_body(d))
        }
        StmtKind::Synthetic(stmts) | StmtKind::NamespaceBlock { body: stmts, .. } => {
            names_body(stmts)
        }
        StmtKind::Echo(e) | StmtKind::ExprStmt(e) | StmtKind::Throw(e) => names_expr(e),
        StmtKind::Assign { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::ConstDecl { value, .. }
        | StmtKind::ListUnpack { value, .. }
        | StmtKind::StaticVar { init: value, .. } => names_expr(value),
        StmtKind::ArrayAssign { index, value, .. } => names_expr(index) || names_expr(value),
        StmtKind::NestedArrayAssign { target, value } => names_expr(target) || names_expr(value),
        StmtKind::ArrayPush { value, .. } => names_expr(value),
        StmtKind::Return(opt) => opt.as_ref().is_some_and(names_expr),
        StmtKind::Include { path, .. } => names_expr(path),
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => names_expr(value),
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            names_expr(index) || names_expr(value)
        }
        StmtKind::DynamicStaticPropertyWrite {
            property: name_expr,
            index,
            value,
            ..
        } => {
            names_expr(name_expr) || index.as_ref().is_some_and(names_expr) || names_expr(value)
        }
        _ => false,
    }
}

/// Recursively checks an expression for a mention of `$this-><property>`.
///
/// `ExprKind::Closure` (arrow functions included) is deliberately not descended into: PHP treats
/// a property whose only `$this->p` sits inside a closure as VIRTUAL (`php -n`:
/// `get => (fn () => $this->p)()` reports `isVirtual() === true`). Unrecognized expression kinds
/// fall through to `false` — the safe, under-detecting direction.
fn expr_names_backing_store(expr: &Expr, property: &str) -> bool {
    let names = |inner: &Expr| expr_names_backing_store(inner, property);
    match &expr.kind {
        // `$this->p` in any read or compound-assignment position — the backing-store mention.
        ExprKind::PropertyAccess {
            object,
            property: name,
        }
        | ExprKind::NullsafePropertyAccess {
            object,
            property: name,
        } => writes_backing_store(object, name, property) || names(object),
        // A closure body is a different scope for this rule; do not peek inside.
        ExprKind::Closure { .. } => false,
        ExprKind::BinaryOp { left, right, .. } => names(left) || names(right),
        ExprKind::InstanceOf { value, .. } => names(value),
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Clone(inner)
        | ExprKind::Spread(inner)
        | ExprKind::YieldFrom(inner)
        | ExprKind::Cast { expr: inner, .. }
        | ExprKind::PtrCast { expr: inner, .. } => names(inner),
        ExprKind::NullCoalesce { value, default } => names(value) || names(default),
        ExprKind::Pipe { value, callable } => names(value) || names(callable),
        ExprKind::Assignment {
            target,
            value,
            result_target,
            prelude,
            ..
        } => {
            names(target)
                || names(value)
                || result_target.as_deref().is_some_and(names)
                || body_names_backing_store(prelude, property)
        }
        ExprKind::ListUnpack { value, .. } => names(value),
        ExprKind::FunctionCall { args, .. }
        | ExprKind::ClosureCall { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::StaticMethodCall { args, .. } => args.iter().any(names),
        ExprKind::NewDynamic { args, .. } => args.iter().any(names),
        ExprKind::NewDynamicObject {
            class_name, args, ..
        } => names(class_name) || args.iter().any(names),
        ExprKind::ExprCall { callee, args } => names(callee) || args.iter().any(names),
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            names(object) || args.iter().any(names)
        }
        ExprKind::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => names(object) || names(method) || args.iter().any(names),
        ExprKind::ArrayLiteral(items) => items.iter().any(names),
        ExprKind::ArrayLiteralAssoc(pairs) => pairs.iter().any(|(k, v)| names(k) || names(v)),
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            names(subject)
                || arms
                    .iter()
                    .any(|(patterns, value)| patterns.iter().any(names) || names(value))
                || default.as_deref().is_some_and(names)
        }
        ExprKind::ArrayAccess { array, index } => names(array) || names(index),
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => names(condition) || names(then_expr) || names(else_expr),
        ExprKind::ShortTernary { value, default } => names(value) || names(default),
        ExprKind::DynamicPropertyAccess { object, property: p }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property: p } => {
            names(object) || names(p)
        }
        ExprKind::NamedArg { value, .. } => names(value),
        ExprKind::BufferNew { len, .. } => names(len),
        ExprKind::Yield { key, value } => {
            key.as_deref().is_some_and(names) || value.as_deref().is_some_and(names)
        }
        _ => false,
    }
}
