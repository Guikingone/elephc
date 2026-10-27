//! Purpose:
//! Rewrites reads of variables that no code in their scope can ever define into a typed
//! compiler call, so the program compiles and php's run-time `Warning: Undefined variable $x`
//! plus a `null` value are reproduced instead of a compile error.
//!
//! Called from:
//! - `crate::func_args::desugar()`, after the argument-introspection rewrite, so every compile
//!   path (pipeline and test harnesses) sees the same AST.
//!
//! Key details:
//! - The decision is deliberately conservative: a name is "written" by every assignment form,
//!   destructuring, increment, element/property write, `foreach`, `catch`, `global`, `static`,
//!   reference binding, closure capture, parameter, and ANY direct call argument (a by-reference
//!   parameter is invisible before type checking). Only a name with none of these anywhere in
//!   its scope is treated as undefined, so flow-sensitive cases keep their existing diagnostics.
//! - A scope that can create locals dynamically (`extract`, `compact`, `get_defined_vars`,
//!   `parse_str`, `eval`, `include`/`require`, `$GLOBALS`) is never rewritten.
//! - Reads under `isset`/`empty`/`unset` and on the left of `??` stay untouched, and under
//!   `@` they become a silent `null`: php reports nothing there.
//! - Arrow functions are left alone; they read their enclosing scope at creation.

use std::cell::RefCell;
use std::collections::HashSet;

use crate::names::{CompilerCall, Name};
use crate::parser::ast::{
    CallableTarget, ClassMethod, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind,
};

/// Names php defines in every scope or that a program may reach without a local write.
const ALWAYS_DEFINED: &[&str] = &[
    "this", "GLOBALS", "_GET", "_POST", "_SERVER", "_COOKIE", "_FILES", "_ENV", "_REQUEST",
    "_SESSION", "http_response_header", "php_errormsg",
];

/// Builtins that create or expose locals by name, making the whole scope dynamic.
const DYNAMIC_SCOPE_CALLS: &[&str] = &[
    "extract", "compact", "get_defined_vars", "parse_str", "eval", "func_get_args",
];

thread_local! {
    /// `extern global` names of the program being rewritten, visible in every scope.
    static EXTERN_GLOBALS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Calls whose variable arguments are probes rather than reads.
const PROBE_CALLS: &[&str] = &["isset", "empty", "unset"];

/// Rewrites the undefined-variable reads of every scope in the program.
pub fn rewrite(program: &mut Vec<Stmt>) {
    let mut program_globals = HashSet::new();
    collect_program_globals(program, &mut program_globals);
    let externs = program
        .iter()
        .filter_map(|stmt| match &stmt.kind {
            StmtKind::ExternGlobalDecl { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    EXTERN_GLOBALS.with(|names| *names.borrow_mut() = externs);
    let mut main_scope = ScopeFacts::new();
    main_scope.written.extend(program_globals);
    main_scope.written.extend(["argv", "argc"].map(str::to_string));
    vivify_scope(program, &main_scope);
    collect_stmts(program, &mut main_scope);
    rewrite_scope_stmts(program, &main_scope);
    EXTERN_GLOBALS.with(|names| names.borrow_mut().clear());
}

/// Per-scope facts gathered before rewriting.
struct ScopeFacts {
    written: HashSet<String>,
    dynamic: bool,
}

impl ScopeFacts {
    /// Starts a scope with the names php always defines and the program's extern globals.
    fn new() -> Self {
        let mut written: HashSet<String> =
            ALWAYS_DEFINED.iter().map(|name| name.to_string()).collect();
        EXTERN_GLOBALS.with(|names| written.extend(names.borrow().iter().cloned()));
        Self { written, dynamic: false }
    }

    /// Returns whether a read of `name` is certainly undefined in this scope.
    fn read_is_undefined(&self, name: &str) -> bool {
        !self.dynamic && !self.written.contains(name)
    }
}

/// Records every name a `global` statement anywhere in the program binds.
fn collect_program_globals(stmts: &[Stmt], globals: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::Global { vars } => globals.extend(vars.iter().cloned()),
            StmtKind::FunctionDecl { body, .. } => collect_program_globals(body, globals),
            StmtKind::ClassDecl { methods, .. }
            | StmtKind::TraitDecl { methods, .. }
            | StmtKind::InterfaceDecl { methods, .. }
            | StmtKind::EnumDecl { methods, .. } => {
                for method in methods {
                    collect_program_globals(&method.body, globals);
                }
            }
            _ => for_each_child_body(stmt, &mut |body| collect_program_globals(body, globals)),
        }
    }
}

/// Visits the statement lists nested directly in a control-flow statement.
fn for_each_child_body(stmt: &Stmt, visit: &mut dyn FnMut(&[Stmt])) {
    match &stmt.kind {
        StmtKind::If { then_body, elseif_clauses, else_body, .. } => {
            visit(then_body);
            for (_, body) in elseif_clauses {
                visit(body);
            }
            if let Some(body) = else_body {
                visit(body);
            }
        }
        StmtKind::IfDef { then_body, else_body, .. } => {
            visit(then_body);
            if let Some(body) = else_body {
                visit(body);
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. } => visit(body),
        StmtKind::For { init, update, body, .. } => {
            if let Some(init) = init {
                visit(std::slice::from_ref(init.as_ref()));
            }
            if let Some(update) = update {
                visit(std::slice::from_ref(update.as_ref()));
            }
            visit(body);
        }
        StmtKind::Switch { cases, default, .. } => {
            for (_, body) in cases {
                visit(body);
            }
            if let Some(body) = default {
                visit(body);
            }
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            visit(try_body);
            for catch in catches {
                visit(&catch.body);
            }
            if let Some(body) = finally_body {
                visit(body);
            }
        }
        _ => {}
    }
}

/// Collects the written names and dynamic markers of one scope's statements.
fn collect_stmts(stmts: &[Stmt], facts: &mut ScopeFacts) {
    for stmt in stmts {
        collect_stmt(stmt, facts);
    }
}

/// Collects one statement without entering nested function, method or class scopes.
fn collect_stmt(stmt: &Stmt, facts: &mut ScopeFacts) {
    match &stmt.kind {
        StmtKind::Assign { name, value } | StmtKind::TypedAssign { name, value, .. } => {
            facts.written.insert(name.clone());
            collect_expr(value, facts);
        }
        StmtKind::RefAssign { target, source } => {
            facts.written.insert(target.clone());
            mark_spine_written(source, facts);
            collect_expr(source, facts);
        }
        StmtKind::ArrayAssign { array, index, value } => {
            facts.written.insert(array.clone());
            collect_expr(index, facts);
            collect_expr(value, facts);
        }
        StmtKind::ArrayPush { array, value } => {
            facts.written.insert(array.clone());
            collect_expr(value, facts);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            mark_spine_written(target, facts);
            collect_expr(target, facts);
            collect_expr(value, facts);
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            mark_spine_written(object, facts);
            collect_expr(object, facts);
            collect_expr(value, facts);
        }
        StmtKind::PropertyArrayAssign { object, index, value, .. } => {
            mark_spine_written(object, facts);
            collect_expr(object, facts);
            collect_expr(index, facts);
            collect_expr(value, facts);
        }
        StmtKind::ListUnpack { vars, value } => {
            facts.written.extend(vars.iter().cloned());
            collect_expr(value, facts);
        }
        StmtKind::Foreach { array, key_var, value_var, .. } => {
            if let Some(key) = key_var {
                facts.written.insert(key.clone());
            }
            facts.written.insert(value_var.clone());
            collect_expr(array, facts);
        }
        StmtKind::Global { vars } => facts.written.extend(vars.iter().cloned()),
        StmtKind::StaticVar { name, init } => {
            facts.written.insert(name.clone());
            collect_expr(init, facts);
        }
        StmtKind::Include { path, .. } => {
            facts.dynamic = true;
            collect_expr(path, facts);
        }
        StmtKind::Try { catches, .. } => {
            for catch in catches {
                if let Some(variable) = &catch.variable {
                    facts.written.insert(variable.clone());
                }
            }
        }
        StmtKind::Echo(expr)
        | StmtKind::Throw(expr)
        | StmtKind::ExprStmt(expr)
        | StmtKind::ConstDecl { value: expr, .. } => collect_expr(expr, facts),
        StmtKind::Return(Some(expr)) => collect_expr(expr, facts),
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => collect_expr(value, facts),
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            collect_expr(index, facts);
            collect_expr(value, facts);
        }
        StmtKind::If { condition, elseif_clauses, .. } => {
            collect_expr(condition, facts);
            for (condition, _) in elseif_clauses {
                collect_expr(condition, facts);
            }
        }
        StmtKind::While { condition, .. } | StmtKind::DoWhile { condition, .. } => {
            collect_expr(condition, facts)
        }
        StmtKind::For { condition: Some(condition), .. } => collect_expr(condition, facts),
        StmtKind::Switch { subject, cases, .. } => {
            collect_expr(subject, facts);
            for (conditions, _) in cases {
                for condition in conditions {
                    collect_expr(condition, facts);
                }
            }
        }
        _ => {}
    }
    for_each_child_body(stmt, &mut |body| collect_stmts(body, facts));
}

/// Marks the base variable of a write target (`$a['k']['j']`, `$o->p[0]`) as written.
fn mark_spine_written(target: &Expr, facts: &mut ScopeFacts) {
    match &target.kind {
        ExprKind::Variable(name) => {
            facts.written.insert(name.clone());
        }
        ExprKind::ArrayAccess { array, .. } => mark_spine_written(array, facts),
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. }
        | ExprKind::DynamicPropertyAccess { object, .. }
        | ExprKind::NullsafeDynamicPropertyAccess { object, .. } => mark_spine_written(object, facts),
        _ => {}
    }
}

/// Collects writes and dynamic markers inside an expression of the current scope.
fn collect_expr(expr: &Expr, facts: &mut ScopeFacts) {
    match &expr.kind {
        ExprKind::Variable(name) if name == "GLOBALS" => facts.dynamic = true,
        ExprKind::PreIncrement(name)
        | ExprKind::PostIncrement(name)
        | ExprKind::PreDecrement(name)
        | ExprKind::PostDecrement(name) => {
            facts.written.insert(name.clone());
        }
        ExprKind::Assignment { target, value, result_target, prelude, .. } => {
            mark_spine_written(target, facts);
            if let Some(result_target) = result_target {
                mark_spine_written(result_target, facts);
            }
            collect_stmts(prelude, facts);
            collect_expr(target, facts);
            collect_expr(value, facts);
        }
        ExprKind::FunctionCall { name, args } => {
            let key = crate::names::php_symbol_key(name.as_str().trim_start_matches('\\'));
            let short = key.rsplit('\\').next().unwrap_or(&key).to_string();
            if DYNAMIC_SCOPE_CALLS.contains(&short.as_str()) {
                facts.dynamic = true;
            }
            if !PROBE_CALLS.contains(&short.as_str()) && !is_by_value_builtin(name) {
                mark_call_args_written(args, facts);
            }
            collect_exprs(args, facts);
        }
        ExprKind::MethodCall { object, args, .. } | ExprKind::NullsafeMethodCall { object, args, .. } => {
            mark_call_args_written(args, facts);
            collect_expr(object, facts);
            collect_exprs(args, facts);
        }
        ExprKind::NullsafeDynamicMethodCall { object, method, args } => {
            mark_call_args_written(args, facts);
            collect_expr(object, facts);
            collect_expr(method, facts);
            collect_exprs(args, facts);
        }
        ExprKind::StaticMethodCall { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewGeneric { args, .. }
        | ExprKind::ClosureCall { args, .. } => {
            mark_call_args_written(args, facts);
            collect_exprs(args, facts);
        }
        ExprKind::NewDynamic { name_expr, args } => {
            mark_call_args_written(args, facts);
            collect_expr(name_expr, facts);
            collect_exprs(args, facts);
        }
        ExprKind::NewDynamicObject { class_name, args, .. } => {
            mark_call_args_written(args, facts);
            collect_expr(class_name, facts);
            collect_exprs(args, facts);
        }
        ExprKind::ExprCall { callee, args } => {
            mark_call_args_written(args, facts);
            collect_expr(callee, facts);
            collect_exprs(args, facts);
        }
        ExprKind::Closure { captures, capture_refs, is_arrow, .. } => {
            facts.written.extend(captures.iter().cloned());
            facts.written.extend(capture_refs.iter().cloned());
            if *is_arrow {
                // An arrow function reads its enclosing scope implicitly; treat it as opaque.
                facts.dynamic = true;
            }
        }
        ExprKind::IncludeValue { path, .. } => {
            facts.dynamic = true;
            collect_expr(path, facts);
        }
        ExprKind::FirstClassCallable(CallableTarget::Method { object, .. }) => collect_expr(object, facts),
        _ => for_each_child_expr(expr, &mut |child| collect_expr(child, facts)),
    }
}

/// Collects a list of expressions.
fn collect_exprs(exprs: &[Expr], facts: &mut ScopeFacts) {
    for expr in exprs {
        collect_expr(expr, facts);
    }
}

/// Returns whether a call names a global builtin none of whose parameters is by reference.
///
/// Its arguments are plain reads, so an undefined one warns like any other read. A namespaced
/// name may be a user function and keeps the conservative treatment.
fn is_by_value_builtin(name: &Name) -> bool {
    let text = name.as_str().trim_start_matches('\\');
    !text.contains('\\')
        && crate::builtins::registry::lookup(text)
            .is_some_and(|def| !def.ref_params.iter().any(|by_ref| *by_ref))
}

/// Treats every argument spine as written: a by-reference parameter may define it.
fn mark_call_args_written(args: &[Expr], facts: &mut ScopeFacts) {
    for arg in args {
        match &arg.kind {
            ExprKind::NamedArg { value, .. } | ExprKind::Spread(value) => mark_spine_written(value, facts),
            _ => mark_spine_written(arg, facts),
        }
    }
}

/// Visits the direct child expressions of the expression kinds without scope effects.
fn for_each_child_expr(expr: &Expr, visit: &mut dyn FnMut(&Expr)) {
    match &expr.kind {
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::Clone(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::YieldFrom(inner)
        | ExprKind::Cast { expr: inner, .. }
        | ExprKind::PtrCast { expr: inner, .. }
        | ExprKind::ObjectClassName { object: inner }
        | ExprKind::PropertyAccess { object: inner, .. }
        | ExprKind::NullsafePropertyAccess { object: inner, .. }
        | ExprKind::NamedArg { value: inner, .. }
        | ExprKind::BufferNew { len: inner, .. } => visit(inner),
        ExprKind::BinaryOp { left, right, .. } => {
            visit(left);
            visit(right);
        }
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
            visit(value);
            visit(default);
        }
        ExprKind::Pipe { value, callable } => {
            visit(value);
            visit(callable);
        }
        ExprKind::InstanceOf { value, target } => {
            visit(value);
            if let InstanceOfTarget::Expr(target) = target {
                visit(target);
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            visit(array);
            visit(index);
        }
        ExprKind::Ternary { condition, then_expr, else_expr } => {
            visit(condition);
            visit(then_expr);
            visit(else_expr);
        }
        ExprKind::ArrayLiteral(items) => items.iter().for_each(&mut *visit),
        ExprKind::ArrayLiteralAssoc(pairs) => {
            for (key, value) in pairs {
                visit(key);
                visit(value);
            }
        }
        ExprKind::ArrayLiteralMixed(entries) => {
            for entry in entries {
                for expr in entry.exprs() {
                    visit(expr);
                }
            }
        }
        ExprKind::Match { subject, arms, default } => {
            visit(subject);
            for (conditions, body) in arms {
                conditions.iter().for_each(&mut *visit);
                visit(body);
            }
            if let Some(default) = default {
                visit(default);
            }
        }
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            visit(object);
            visit(property);
        }
        ExprKind::Yield { key, value } => {
            if let Some(key) = key {
                visit(key);
            }
            if let Some(value) = value {
                visit(value);
            }
        }
        _ => {}
    }
}

/// Rewrites one scope's undefined reads and recurses into nested scopes with their own facts.
fn rewrite_scope_stmts(stmts: &mut [Stmt], facts: &ScopeFacts) {
    for stmt in stmts.iter_mut() {
        rewrite_stmt(stmt, facts);
    }
}

/// Rewrites one statement of the current scope.
fn rewrite_stmt(stmt: &mut Stmt, facts: &ScopeFacts) {
    match &mut stmt.kind {
        StmtKind::FunctionDecl { params, variadic, body, .. } => {
            rewrite_function_scope(params.iter().map(|(name, ..)| name.clone()), variadic.clone(), body);
        }
        StmtKind::ClassDecl { methods, .. }
        | StmtKind::TraitDecl { methods, .. }
        | StmtKind::InterfaceDecl { methods, .. }
        | StmtKind::EnumDecl { methods, .. } => rewrite_methods(methods),
        StmtKind::Assign { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::ArrayPush { value, .. }
        | StmtKind::ListUnpack { value, .. }
        | StmtKind::StaticVar { init: value, .. }
        | StmtKind::ConstDecl { value, .. }
        | StmtKind::Echo(value)
        | StmtKind::Throw(value)
        | StmtKind::ExprStmt(value)
        | StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => rewrite_expr(value, facts),
        StmtKind::Return(Some(value)) => rewrite_expr(value, facts),
        StmtKind::ArrayAssign { index, value, .. } | StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            rewrite_expr(index, facts);
            rewrite_expr(value, facts);
        }
        StmtKind::PropertyAssign { value, .. } | StmtKind::PropertyArrayPush { value, .. } => {
            rewrite_expr(value, facts);
        }
        StmtKind::PropertyArrayAssign { index, value, .. } => {
            rewrite_expr(index, facts);
            rewrite_expr(value, facts);
        }
        StmtKind::NestedArrayAssign { value, .. } => rewrite_expr(value, facts),
        StmtKind::Foreach { array, body, .. } => {
            rewrite_expr(array, facts);
            rewrite_scope_stmts(body, facts);
        }
        StmtKind::If { condition, then_body, elseif_clauses, else_body } => {
            rewrite_expr(condition, facts);
            rewrite_scope_stmts(then_body, facts);
            for (condition, body) in elseif_clauses.iter_mut() {
                rewrite_expr(condition, facts);
                rewrite_scope_stmts(body, facts);
            }
            if let Some(body) = else_body {
                rewrite_scope_stmts(body, facts);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            rewrite_expr(condition, facts);
            rewrite_scope_stmts(body, facts);
        }
        StmtKind::For { init, condition, update, body } => {
            if let Some(init) = init {
                rewrite_stmt(init, facts);
            }
            if let Some(condition) = condition {
                rewrite_expr(condition, facts);
            }
            if let Some(update) = update {
                rewrite_stmt(update, facts);
            }
            rewrite_scope_stmts(body, facts);
        }
        StmtKind::Switch { subject, cases, default } => {
            rewrite_expr(subject, facts);
            for (conditions, body) in cases.iter_mut() {
                for condition in conditions.iter_mut() {
                    rewrite_expr(condition, facts);
                }
                rewrite_scope_stmts(body, facts);
            }
            if let Some(body) = default {
                rewrite_scope_stmts(body, facts);
            }
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            rewrite_scope_stmts(try_body, facts);
            for catch in catches.iter_mut() {
                rewrite_scope_stmts(&mut catch.body, facts);
            }
            if let Some(body) = finally_body {
                rewrite_scope_stmts(body, facts);
            }
        }
        StmtKind::IfDef { then_body, else_body, .. } => {
            rewrite_scope_stmts(then_body, facts);
            if let Some(body) = else_body {
                rewrite_scope_stmts(body, facts);
            }
        }
        StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. } => rewrite_scope_stmts(body, facts),
        _ => {}
    }
}

/// Rewrites every method body in its own scope.
fn rewrite_methods(methods: &mut [ClassMethod]) {
    for method in methods.iter_mut() {
        let params = method.params.iter().map(|(name, ..)| name.clone()).collect::<Vec<_>>();
        rewrite_function_scope(params.into_iter(), method.variadic.clone(), &mut method.body);
    }
}

/// Collects and rewrites one function-like scope.
fn rewrite_function_scope(
    params: impl Iterator<Item = String>,
    variadic: Option<String>,
    body: &mut Vec<Stmt>,
) {
    let mut facts = ScopeFacts::new();
    facts.written.extend(params);
    facts.written.extend(variadic);
    vivify_scope(body, &facts);
    collect_stmts(body, &mut facts);
    rewrite_scope_stmts(body, &facts);
}

/// Runs array auto-vivification on one scope unless it can define names dynamically.
fn vivify_scope(body: &mut Vec<Stmt>, initial: &ScopeFacts) {
    let mut probe = ScopeFacts { written: HashSet::new(), dynamic: false };
    collect_stmts(body, &mut probe);
    if probe.dynamic {
        return;
    }
    let mut seen = initial.written.clone();
    vivify_stmts(body, &mut seen, false);
}

/// Rewrites undefined reads inside one expression of the current scope.
fn rewrite_expr(expr: &mut Expr, facts: &ScopeFacts) {
    match &mut expr.kind {
        ExprKind::Variable(name) if facts.read_is_undefined(name) => {
            let span = expr.span;
            let name = std::mem::take(name);
            *expr = Expr::new(
                ExprKind::FunctionCall {
                    name: Name::compiler_call(CompilerCall::UndefinedVariable),
                    args: vec![Expr::new(ExprKind::StringLiteral(name), span)],
                },
                span,
            );
        }
        // php reports nothing for probes or the left side of `??`.
        ExprKind::NullCoalesce { default, .. } => rewrite_expr(default, facts),
        // `@` suppresses the warning but the read still yields `null`.
        ExprKind::ErrorSuppress(inner) => silence_undefined_reads(inner, facts),
        ExprKind::FunctionCall { name, args } => {
            let key = crate::names::php_symbol_key(name.as_str().trim_start_matches('\\'));
            let short = key.rsplit('\\').next().unwrap_or(&key).to_string();
            if !PROBE_CALLS.contains(&short.as_str()) {
                rewrite_exprs(args, facts);
            }
        }
        ExprKind::Closure { params, variadic, body, is_arrow, captures, capture_refs, .. } => {
            if !*is_arrow {
                // Captured names are defined inside the closure like parameters.
                let names = params.iter().map(|(name, ..)| name.clone())
                    .chain(captures.iter().cloned())
                    .chain(capture_refs.iter().cloned())
                    .collect::<Vec<_>>();
                rewrite_function_scope(names.into_iter(), variadic.clone(), body);
            }
        }
        ExprKind::Assignment { value, .. } => rewrite_expr(value, facts),
        ExprKind::MethodCall { object, args, .. } | ExprKind::NullsafeMethodCall { object, args, .. } => {
            rewrite_expr(object, facts);
            rewrite_exprs(args, facts);
        }
        ExprKind::StaticMethodCall { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewGeneric { args, .. }
        | ExprKind::ClosureCall { args, .. } => rewrite_exprs(args, facts),
        ExprKind::ExprCall { callee, args } => {
            rewrite_expr(callee, facts);
            rewrite_exprs(args, facts);
        }
        _ => for_each_child_expr_mut(expr, &mut |child| rewrite_expr(child, facts)),
    }
}

/// Replaces undefined reads under `@` with a plain `null`, since php shows no warning there.
fn silence_undefined_reads(expr: &mut Expr, facts: &ScopeFacts) {
    match &mut expr.kind {
        ExprKind::Variable(name) if facts.read_is_undefined(name) => {
            *expr = Expr::new(ExprKind::Null, expr.span);
        }
        ExprKind::Closure { .. } | ExprKind::FunctionCall { .. } => rewrite_expr(expr, facts),
        _ => for_each_child_expr_mut(expr, &mut |child| silence_undefined_reads(child, facts)),
    }
}

/// Rewrites a list of expressions.
fn rewrite_exprs(exprs: &mut [Expr], facts: &ScopeFacts) {
    for expr in exprs.iter_mut() {
        rewrite_expr(expr, facts);
    }
}

/// Mutable twin of `for_each_child_expr` for the rewrite pass.
pub(crate) fn for_each_child_expr_mut(expr: &mut Expr, visit: &mut dyn FnMut(&mut Expr)) {
    match &mut expr.kind {
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::Clone(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::YieldFrom(inner)
        | ExprKind::Cast { expr: inner, .. }
        | ExprKind::PtrCast { expr: inner, .. }
        | ExprKind::ObjectClassName { object: inner }
        | ExprKind::PropertyAccess { object: inner, .. }
        | ExprKind::NullsafePropertyAccess { object: inner, .. }
        | ExprKind::NamedArg { value: inner, .. }
        | ExprKind::BufferNew { len: inner, .. } => visit(inner),
        ExprKind::BinaryOp { left, right, .. } => {
            visit(left);
            visit(right);
        }
        ExprKind::ShortTernary { value, default } => {
            visit(value);
            visit(default);
        }
        ExprKind::Pipe { value, callable } => {
            visit(value);
            visit(callable);
        }
        ExprKind::InstanceOf { value, target } => {
            visit(value);
            if let InstanceOfTarget::Expr(target) = target {
                visit(target);
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            visit(array);
            visit(index);
        }
        ExprKind::Ternary { condition, then_expr, else_expr } => {
            visit(condition);
            visit(then_expr);
            visit(else_expr);
        }
        ExprKind::ArrayLiteral(items) => items.iter_mut().for_each(&mut *visit),
        ExprKind::ArrayLiteralAssoc(pairs) => {
            for (key, value) in pairs.iter_mut() {
                visit(key);
                visit(value);
            }
        }
        ExprKind::ArrayLiteralMixed(entries) => {
            for entry in entries.iter_mut() {
                for expr in entry.exprs_mut() {
                    visit(expr);
                }
            }
        }
        ExprKind::Match { subject, arms, default } => {
            visit(subject);
            for (conditions, body) in arms.iter_mut() {
                conditions.iter_mut().for_each(&mut *visit);
                visit(body);
            }
            if let Some(default) = default {
                visit(default);
            }
        }
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            visit(object);
            visit(property);
        }
        ExprKind::NullsafeDynamicMethodCall { object, method, args } => {
            visit(object);
            visit(method);
            args.iter_mut().for_each(&mut *visit);
        }
        ExprKind::NewDynamic { name_expr, args } => {
            visit(name_expr);
            args.iter_mut().for_each(&mut *visit);
        }
        ExprKind::NewDynamicObject { class_name, args, .. } => {
            visit(class_name);
            args.iter_mut().for_each(&mut *visit);
        }
        ExprKind::Yield { key, value } => {
            if let Some(key) = key {
                visit(key);
            }
            if let Some(value) = value {
                visit(value);
            }
        }
        _ => {}
    }
}

/// Inserts php's array auto-vivification as an explicit `$v = [];` before the element write
/// that first mentions `$v` in its scope (`$v[] = x`, `$v['k'] = x`, `$v[1][2] = x`).
///
/// php turns an undefined variable into an array on its first element write; elephc refused it
/// at top level and miscompiled it inside functions. A write inside a loop initializes before
/// the outermost loop, so later iterations keep appending; a write inside a branch initializes
/// in that branch, so the other path still leaves the variable undefined. A variable already
/// mentioned (read, written, bound by `global`/`static`/`foreach`/`list`/`catch`) is left alone.
fn vivify_stmts(stmts: &mut Vec<Stmt>, seen: &mut HashSet<String>, in_loop: bool) -> Vec<String> {
    let mut hoisted = Vec::new();
    let mut i = 0;
    while i < stmts.len() {
        if let Some(root) = element_write_root(&stmts[i]) {
            if !seen.contains(&root) {
                seen.insert(root.clone());
                if in_loop {
                    hoisted.push(root);
                } else {
                    let span = stmts[i].span;
                    stmts.insert(i, empty_array_assign(&root, span));
                    i += 1;
                }
            }
        }
        let inner = vivify_children(&mut stmts[i], seen);
        if in_loop {
            hoisted.extend(inner);
        } else {
            for name in inner {
                let span = stmts[i].span;
                stmts.insert(i, empty_array_assign(&name, span));
                i += 1;
            }
        }
        mark_mentioned_names(&mut stmts[i], seen);
        i += 1;
    }
    hoisted
}

/// Vivifies the bodies of a compound statement; returns names a loop body hoisted to it.
fn vivify_children(stmt: &mut Stmt, seen: &mut HashSet<String>) -> Vec<String> {
    match &mut stmt.kind {
        StmtKind::If { condition, then_body, elseif_clauses, else_body } => {
            mark_expr_names(condition, seen);
            let mut merged = seen.clone();
            let mut hoisted = Vec::new();
            let mut branch = seen.clone();
            hoisted.extend(vivify_stmts(then_body, &mut branch, false));
            merged.extend(branch);
            for (condition, body) in elseif_clauses.iter_mut() {
                mark_expr_names(condition, seen);
                let mut branch = seen.clone();
                hoisted.extend(vivify_stmts(body, &mut branch, false));
                merged.extend(branch);
            }
            if let Some(body) = else_body {
                let mut branch = seen.clone();
                hoisted.extend(vivify_stmts(body, &mut branch, false));
                merged.extend(branch);
            }
            *seen = merged;
            hoisted
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            mark_expr_names(condition, seen);
            vivify_stmts(body, seen, true)
        }
        StmtKind::For { init, condition, update, body } => {
            for part in [init.as_deref_mut(), update.as_deref_mut()].into_iter().flatten() {
                mark_mentioned_names(part, seen);
            }
            if let Some(condition) = condition {
                mark_expr_names(condition, seen);
            }
            vivify_stmts(body, seen, true)
        }
        StmtKind::Foreach { array, key_var, value_var, body, .. } => {
            mark_expr_names(array, seen);
            seen.extend(key_var.iter().cloned());
            seen.insert(value_var.clone());
            vivify_stmts(body, seen, true)
        }
        StmtKind::Switch { subject, cases, default } => {
            mark_expr_names(subject, seen);
            let mut hoisted = Vec::new();
            for (_, body) in cases.iter_mut() {
                hoisted.extend(vivify_stmts(body, seen, false));
            }
            if let Some(body) = default {
                hoisted.extend(vivify_stmts(body, seen, false));
            }
            hoisted
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            let mut hoisted = vivify_stmts(try_body, seen, false);
            for catch in catches.iter_mut() {
                if let Some(var) = &catch.variable {
                    seen.insert(var.clone());
                }
                hoisted.extend(vivify_stmts(&mut catch.body, seen, false));
            }
            if let Some(body) = finally_body {
                hoisted.extend(vivify_stmts(body, seen, false));
            }
            hoisted
        }
        StmtKind::Synthetic(body) | StmtKind::IncludeOnceGuard { body, .. } => vivify_stmts(body, seen, false),
        _ => Vec::new(),
    }
}

/// Returns the variable a statement writes an element of, if it is that kind of statement.
fn element_write_root(stmt: &Stmt) -> Option<String> {
    match &stmt.kind {
        StmtKind::ArrayAssign { array, .. } | StmtKind::ArrayPush { array, .. } => Some(array.clone()),
        StmtKind::NestedArrayAssign { target, .. } => {
            let mut current = target;
            while let ExprKind::ArrayAccess { array, .. } = &current.kind {
                current = array;
            }
            match &current.kind {
                ExprKind::Variable(name) => Some(name.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Builds `$name = [];` at `span`.
fn empty_array_assign(name: &str, span: crate::span::Span) -> Stmt {
    Stmt::new(
        StmtKind::Assign {
            name: name.to_string(),
            value: Expr::new(ExprKind::ArrayLiteral(Vec::new()), span),
        },
        span,
    )
}

/// Records every variable a statement mentions, read or written.
fn mark_mentioned_names(stmt: &mut Stmt, seen: &mut HashSet<String>) {
    let mut facts = ScopeFacts { written: HashSet::new(), dynamic: false };
    collect_stmt(stmt, &mut facts);
    seen.extend(facts.written);
    crate::undefined_constants::walk_program_exprs_mut(std::slice::from_mut(stmt), &mut |expr| {
        if let ExprKind::Variable(name) = &expr.kind {
            seen.insert(name.clone());
        }
    });
}

/// Records every variable an expression mentions.
fn mark_expr_names(expr: &mut Expr, seen: &mut HashSet<String>) {
    let mut stmt = Stmt::new(StmtKind::ExprStmt(std::mem::replace(expr, Expr::new(ExprKind::Null, expr.span))), expr.span);
    mark_mentioned_names(&mut stmt, seen);
    if let StmtKind::ExprStmt(original) = stmt.kind {
        *expr = original;
    }
}
