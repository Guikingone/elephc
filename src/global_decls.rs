//! Purpose:
//! Collects the PHP variable names that any function-like body in a program declares with
//! `global`, so the checker and EIR lowering answer "does program storage back this name?" from
//! ONE walk instead of two that can drift.
//!
//! Called from:
//! - `crate::types::checker::driver::check_types_impl` (once per check, before the first walk)
//! - `crate::ir_lower::function::lower_main` and the per-body lowering entry points
//!
//! Key details:
//! - `global $x;` inside a function/method/closure binds `$x` to the program-global cell the TOP
//!   LEVEL also writes through its own local slot. The checker must therefore not end a top-level
//!   binding of such a name (`unset` would leave the name unbound while another body still reaches
//!   the storage by name), and lowering must not abandon its slot.
//! - ONE walker, TWO scopes, and the SPLIT IS DELIBERATE. [`WalkScope::StatementBodies`] is what
//!   LOWERING consumes; [`WalkScope::NestedBodies`] additionally descends into closure bodies,
//!   assignment preludes and enum methods, and ONLY the checker's `unset`-kill veto consumes it.
//! - Why they must differ: the set lowering reads decides STORAGE CLASS. Moving a top-level name
//!   into the `_eir_global_*` symbol types it `Mixed`, and the array builtins have pre-existing
//!   `Mixed`-array backend gaps — so widening the lowering side made previously-correct programs
//!   fail. `$d = function () { global $a; }; $a = [3, 1, 2]; echo implode(",", $a);` printed
//!   nothing (PHP: `3,1,2`) and `array_sum`/`sort`/`usort`/`in_array`/`array_map`/`array_keys`/
//!   `array_reverse` on such a name became a hard `unsupported EIR backend feature: … for PHP type
//!   Mixed`. The checker's veto has no such coupling: it only REFUSES to end a binding, which
//!   costs a feature and never a lowering decision.
//! - What the narrow scope therefore keeps: a `global` written inside a closure or an enum method
//!   still does not reach main's storage, so the closure's write is lost. That is a PRE-EXISTING
//!   defect (it reproduces with the walk this module shipped before the scopes were split), it is
//!   tracked separately, and its real fix is blocked on the `Mixed`-array backend gaps above.
//! - Both walks are EXHAUSTIVE on their AST enum on purpose. A `global` is a statement, but a
//!   statement can sit inside an expression (a closure body, an assignment prelude), so a new
//!   variant that carries statements must fail to compile here rather than silently become a
//!   blind spot — that is exactly how the closure and enum-method holes survived.
//! - `StmtKind::PackedClassDecl` is NOT a hole: a packed class declares typed FIELDS only, with no
//!   method bodies and no expressions, so there is nothing in it to walk.

use std::collections::HashSet;

use crate::parser::ast::{CallableTarget, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind};

/// How far into a program a `global` collection walk descends. See the module preamble for why the
/// two answers must stay different.
#[derive(Clone, Copy, PartialEq, Eq)]
enum WalkScope {
    /// Statement bodies only — functions, methods, control flow, class/trait/interface methods.
    /// The set EIR LOWERING consumes to decide which top-level names live in program storage.
    StatementBodies,
    /// Everything above, PLUS closure bodies, assignment preludes and enum methods. Consumed by
    /// the checker's `unset`-kill veto alone, where a wider answer is pure conservatism.
    NestedBodies,
}

/// Collects the PHP variable names that any function-like STATEMENT body in `statements` declares
/// `global`.
///
/// The lowering-facing entry point, and the one that decides storage class. It deliberately does
/// NOT see a `global` written inside a closure body, an assignment prelude or an enum method — see
/// the module preamble.
pub(crate) fn collect_global_var_names(statements: &[Stmt]) -> HashSet<String> {
    collect_with_scope(statements, WalkScope::StatementBodies)
}

/// Collects the same names PLUS the ones declared inside closure bodies, assignment preludes and
/// enum methods.
///
/// The checker's `unset`-kill veto reads this one (`Checker::top_level_binding_is_program_global`).
/// A name any of those bodies declares `global` must not have its TOP-LEVEL binding ended: the
/// storage stays reachable by name, so dropping the binding turned a program PHP runs into a false
/// `Undefined variable`. Over-approximating here only ever withholds the kill.
pub(crate) fn collect_global_var_names_in_nested_bodies(statements: &[Stmt]) -> HashSet<String> {
    collect_with_scope(statements, WalkScope::NestedBodies)
}

/// Runs one walk at the requested scope.
fn collect_with_scope(statements: &[Stmt], scope: WalkScope) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_in_body(statements, scope, &mut names);
    names
}

/// Recursively scans statement bodies for `global` declarations.
///
/// Exhaustive on `StmtKind`: a new statement that can carry a body or an expression has to be
/// classified here rather than defaulting to "declares nothing", which is how a `global` inside an
/// enum method went uncollected.
fn collect_in_body(statements: &[Stmt], scope: WalkScope, names: &mut HashSet<String>) {
    for stmt in statements {
        collect_in_stmt(stmt, scope, names);
    }
}

/// Scans one statement and everything nested inside it.
fn collect_in_stmt(stmt: &Stmt, scope: WalkScope, names: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::Global { vars } => {
            names.extend(vars.iter().cloned());
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_in_expr(condition, scope, names);
            collect_in_body(then_body, scope, names);
            for (condition, body) in elseif_clauses {
                collect_in_expr(condition, scope, names);
                collect_in_body(body, scope, names);
            }
            if let Some(body) = else_body {
                collect_in_body(body, scope, names);
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            collect_in_body(then_body, scope, names);
            if let Some(body) = else_body {
                collect_in_body(body, scope, names);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            collect_in_expr(condition, scope, names);
            collect_in_body(body, scope, names);
        }
        StmtKind::Foreach { array, body, .. } => {
            collect_in_expr(array, scope, names);
            collect_in_body(body, scope, names);
        }
        StmtKind::FunctionDecl { params, body, .. } => {
            collect_in_param_defaults(params, scope, names);
            collect_in_body(body, scope, names);
        }
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => {
            collect_in_body(body, scope, names);
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(init) = init {
                collect_in_stmt(init, scope, names);
            }
            if let Some(condition) = condition {
                collect_in_expr(condition, scope, names);
            }
            if let Some(update) = update {
                collect_in_stmt(update, scope, names);
            }
            collect_in_body(body, scope, names);
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            collect_in_expr(subject, scope, names);
            for (patterns, body) in cases {
                collect_in_exprs(patterns, scope, names);
                collect_in_body(body, scope, names);
            }
            if let Some(body) = default {
                collect_in_body(body, scope, names);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            collect_in_body(try_body, scope, names);
            for catch in catches {
                collect_in_body(&catch.body, scope, names);
            }
            if let Some(body) = finally_body {
                collect_in_body(body, scope, names);
            }
        }
        StmtKind::ClassDecl {
            properties,
            methods,
            constants,
            ..
        }
        | StmtKind::InterfaceDecl {
            properties,
            methods,
            constants,
            ..
        }
        | StmtKind::TraitDecl {
            properties,
            methods,
            constants,
            ..
        } => {
            for property in properties {
                if let Some(default) = &property.default {
                    collect_in_expr(default, scope, names);
                }
            }
            for method in methods {
                collect_in_param_defaults(&method.params, scope, names);
                collect_in_body(&method.body, scope, names);
            }
            for constant in constants {
                collect_in_expr(&constant.value, scope, names);
            }
        }
        // An enum's methods are checked and lowered exactly like a class's, so a `global` written
        // in one binds the same program storage. Skipping them made the CHECKER approve a
        // top-level `unset` of a name the enum method still writes — which is why the veto's wide
        // scope walks them. The lowering scope does NOT: moving such a name into program storage
        // is the storage-class change the module preamble refuses, and an enum method is reached
        // through the same `_eir_global_*` symbol a closure body is.
        StmtKind::EnumDecl {
            cases,
            methods,
            constants,
            ..
        } => {
            if scope == WalkScope::NestedBodies {
                for case in cases {
                    if let Some(value) = &case.value {
                        collect_in_expr(value, scope, names);
                    }
                }
                for method in methods {
                    collect_in_param_defaults(&method.params, scope, names);
                    collect_in_body(&method.body, scope, names);
                }
                for constant in constants {
                    collect_in_expr(&constant.value, scope, names);
                }
            }
        }
        StmtKind::Echo(expr) | StmtKind::Throw(expr) | StmtKind::ExprStmt(expr) => {
            collect_in_expr(expr, scope, names);
        }
        StmtKind::Assign { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::ConstDecl { value, .. }
        | StmtKind::ListUnpack { value, .. }
        | StmtKind::ArrayPush { value, .. }
        | StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => collect_in_expr(value, scope, names),
        StmtKind::RefAssign { source, .. } => collect_in_expr(source, scope, names),
        StmtKind::StaticVar { init, .. } => collect_in_expr(init, scope, names),
        StmtKind::ArrayAssign { index, value, .. }
        | StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            collect_in_expr(index, scope, names);
            collect_in_expr(value, scope, names);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            collect_in_expr(target, scope, names);
            collect_in_expr(value, scope, names);
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            collect_in_expr(object, scope, names);
            collect_in_expr(value, scope, names);
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            collect_in_expr(object, scope, names);
            collect_in_expr(index, scope, names);
            collect_in_expr(value, scope, names);
        }
        StmtKind::Return(value) => {
            if let Some(value) = value {
                collect_in_expr(value, scope, names);
            }
        }
        StmtKind::Include { path, .. } => collect_in_expr(path, scope, names),
        // A packed class declares typed FIELDS only — no defaults, no bodies, no expressions.
        StmtKind::PackedClassDecl { .. }
        // Externs are C declarations: types and names, never PHP statements.
        | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. }
        | StmtKind::ExternGlobalDecl { .. }
        // Leaves: no sub-statements and no sub-expressions.
        | StmtKind::Break(_)
        | StmtKind::Continue(_)
        | StmtKind::IncludeOnceMark { .. }
        | StmtKind::NamespaceDecl { .. }
        | StmtKind::UseDecl { .. }
        // Variant groups/marks carry function NAMES; the bodies live in their own `FunctionDecl`s.
        | StmtKind::FunctionVariantGroup { .. }
        | StmtKind::FunctionVariantMark { .. } => {}
    }
}

/// Scans one expression for the statement bodies it can carry.
///
/// Only two expression shapes hold statements — a closure literal's body and the assignment
/// prelude the parser synthesizes — but the walk has to REACH them, so every nesting arm is
/// enumerated. Exhaustive for the same reason [`collect_in_stmt`] is.
fn collect_in_expr(expr: &Expr, scope: WalkScope, names: &mut HashSet<String>) {
    // The whole expression descent belongs to the wide scope. Returning here rather than gating
    // each arm is what makes `WalkScope::StatementBodies` EXACTLY the statement-only walk the
    // lowering side has always used — every statement arm below that reaches an expression becomes
    // a no-op, so no statement arm has to be duplicated or remembered.
    if scope != WalkScope::NestedBodies {
        return;
    }
    match &expr.kind {
        // The two statement-bearing expression shapes.
        ExprKind::Closure { params, body, .. } => {
            collect_in_param_defaults(params, scope, names);
            collect_in_body(body, scope, names);
        }
        ExprKind::Assignment {
            target,
            value,
            result_target,
            prelude,
            ..
        } => {
            collect_in_expr(target, scope, names);
            collect_in_expr(value, scope, names);
            if let Some(result_target) = result_target {
                collect_in_expr(result_target, scope, names);
            }
            collect_in_body(prelude, scope, names);
        }
        ExprKind::FunctionCall { args, .. }
        | ExprKind::StaticMethodCall { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::ClosureCall { args, .. } => collect_in_exprs(args, scope, names),
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            collect_in_expr(object, scope, names);
            collect_in_exprs(args, scope, names);
        }
        ExprKind::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => {
            collect_in_expr(object, scope, names);
            collect_in_expr(method, scope, names);
            collect_in_exprs(args, scope, names);
        }
        ExprKind::ExprCall { callee, args } => {
            collect_in_expr(callee, scope, names);
            collect_in_exprs(args, scope, names);
        }
        ExprKind::NewDynamic { name_expr, args } => {
            collect_in_expr(name_expr, scope, names);
            collect_in_exprs(args, scope, names);
        }
        ExprKind::NewDynamicObject {
            class_name, args, ..
        } => {
            collect_in_expr(class_name, scope, names);
            collect_in_exprs(args, scope, names);
        }
        ExprKind::Pipe { value, callable } => {
            collect_in_expr(value, scope, names);
            collect_in_expr(callable, scope, names);
        }
        ExprKind::IncludeValue { path, .. } => collect_in_expr(path, scope, names),
        ExprKind::Yield { key, value } => {
            if let Some(key) = key {
                collect_in_expr(key, scope, names);
            }
            if let Some(value) = value {
                collect_in_expr(value, scope, names);
            }
        }
        ExprKind::BinaryOp { left, right, .. } => {
            collect_in_expr(left, scope, names);
            collect_in_expr(right, scope, names);
        }
        ExprKind::InstanceOf { value, target } => {
            collect_in_expr(value, scope, names);
            if let InstanceOfTarget::Expr(target) = target {
                collect_in_expr(target, scope, names);
            }
        }
        ExprKind::YieldFrom(inner)
        | ExprKind::Clone(inner)
        | ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::Cast { expr: inner, .. }
        | ExprKind::PtrCast { expr: inner, .. }
        | ExprKind::NamedArg { value: inner, .. }
        | ExprKind::BufferNew { len: inner, .. } => collect_in_expr(inner, scope, names),
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
            collect_in_expr(value, scope, names);
            collect_in_expr(default, scope, names);
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_in_expr(condition, scope, names);
            collect_in_expr(then_expr, scope, names);
            collect_in_expr(else_expr, scope, names);
        }
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            collect_in_expr(subject, scope, names);
            for (patterns, body) in arms {
                collect_in_exprs(patterns, scope, names);
                collect_in_expr(body, scope, names);
            }
            if let Some(default) = default {
                collect_in_expr(default, scope, names);
            }
        }
        ExprKind::ArrayLiteral(items) => collect_in_exprs(items, scope, names),
        ExprKind::ArrayLiteralAssoc(items) => {
            for (key, value) in items {
                collect_in_expr(key, scope, names);
                collect_in_expr(value, scope, names);
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            collect_in_expr(array, scope, names);
            collect_in_expr(index, scope, names);
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. }
        | ExprKind::ObjectClassName { object } => collect_in_expr(object, scope, names),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            collect_in_expr(object, scope, names);
            collect_in_expr(property, scope, names);
        }
        ExprKind::FirstClassCallable(target) => {
            if let CallableTarget::Method { object, .. } = target {
                collect_in_expr(object, scope, names);
            }
        }
        ExprKind::Variable(_)
        | ExprKind::PreIncrement(_)
        | ExprKind::PostIncrement(_)
        | ExprKind::PreDecrement(_)
        | ExprKind::PostDecrement(_)
        | ExprKind::StringLiteral(_)
        | ExprKind::IntLiteral(_)
        | ExprKind::FloatLiteral(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::Null
        | ExprKind::ConstRef(_)
        | ExprKind::This
        | ExprKind::StaticPropertyAccess { .. }
        | ExprKind::ClassConstant { .. }
        | ExprKind::ScopedConstantAccess { .. }
        | ExprKind::MagicConstant(_) => {}
    }
}

/// Scans a list of expressions.
fn collect_in_exprs(exprs: &[Expr], scope: WalkScope, names: &mut HashSet<String>) {
    for expr in exprs {
        collect_in_expr(expr, scope, names);
    }
}

/// Scans the default-value expressions of a parameter list, which can hold a closure literal.
fn collect_in_param_defaults(
    params: &[(String, Option<crate::parser::ast::TypeExpr>, Option<Expr>, bool)],
    scope: WalkScope,
    names: &mut HashSet<String>,
) {
    for (_, _, default, _) in params {
        if let Some(default) = default {
            collect_in_expr(default, scope, names);
        }
    }
}
