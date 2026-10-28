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
//!   its scope is treated as undefined by the scope pass.
//! - A second, flow-sensitive pass then rewrites reads that no write on any path can precede
//!   (`var_dump($a); $a = 1;`): branches merge by union and loops count all their body's writes.
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
    /// Names `unset` cannot leave undefined in the scope being walked: in the main scope, a
    /// function's `global $x` may recreate the cell before the next read.
    static UNKILLABLE: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Calls whose variable arguments are probes rather than reads.
const PROBE_CALLS: &[&str] = &["isset", "empty", "unset"];

/// Rewrites the undefined-variable reads of every scope in the program.
pub fn rewrite(program: &mut Vec<Stmt>) {
    crate::globals_array::rewrite(program);
    let mut program_globals = HashSet::new();
    collect_program_globals(program, &mut program_globals);
    // Closure bodies bind globals too (`function () { global $a; ... }`).
    crate::undefined_constants::walk_program_exprs_mut(program, &mut |expr| {
        if let ExprKind::Closure { body, .. } = &expr.kind {
            collect_program_globals(body, &mut program_globals);
        }
    });
    bind_function_only_globals(program, &program_globals);
    let externs = program
        .iter()
        .filter_map(|stmt| match &stmt.kind {
            StmtKind::ExternGlobalDecl { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    EXTERN_GLOBALS.with(|names| *names.borrow_mut() = externs);
    let mut main_scope = ScopeFacts::new();
    main_scope.written.extend(program_globals.iter().cloned());
    main_scope.written.extend(["argv", "argc"].map(str::to_string));
    vivify_scope(program, &main_scope);
    let mut maybe = main_scope.written.clone();
    collect_stmts(program, &mut main_scope);
    rewrite_scope_stmts(program, &main_scope);
    if !main_scope.dynamic {
        UNKILLABLE.with(|names| *names.borrow_mut() = program_globals);
        rewrite_reads_before_writes(program, &mut maybe);
        UNKILLABLE.with(|names| names.borrow_mut().clear());
    }
    EXTERN_GLOBALS.with(|names| names.borrow_mut().clear());
}

/// Binds, at the top of the main scope, the globals only functions write (`global $c; $c = 5;`):
/// main then reads the shared cell instead of a local nothing assigns. A main-scope `global` is
/// a no-op in php, so this only gives the checker and lowering the cell they need.
fn bind_function_only_globals(program: &mut Vec<Stmt>, program_globals: &HashSet<String>) {
    let mut main_writes = ScopeFacts { written: HashSet::new(), dynamic: false, flow: false };
    collect_stmts(program, &mut main_writes);
    let mut names: Vec<String> = program_globals
        .iter()
        .filter(|name| {
            !main_writes.written.contains(*name)
                && !ALWAYS_DEFINED.contains(&name.as_str())
                && !matches!(name.as_str(), "argc" | "argv")
        })
        .cloned()
        .collect();
    if names.is_empty() {
        return;
    }
    names.sort();
    let span = program.first().map(|stmt| stmt.span).unwrap_or_else(crate::span::Span::dummy);
    let binding = Stmt::new(StmtKind::Global { vars: names }, span);
    // Namespaced programs keep every statement inside their blocks.
    match program.iter_mut().find(|stmt| matches!(stmt.kind, StmtKind::NamespaceBlock { .. })) {
        Some(Stmt { kind: StmtKind::NamespaceBlock { body, .. }, .. }) => body.insert(0, binding),
        _ => program.insert(0, binding),
    }
}

/// Per-scope facts gathered before rewriting.
struct ScopeFacts {
    written: HashSet<String>,
    dynamic: bool,
    /// Set by the flow-sensitive pass, which leaves nested function scopes to the scope pass.
    flow: bool,
}

impl ScopeFacts {
    /// Starts a scope with the names php always defines and the program's extern globals.
    fn new() -> Self {
        let mut written: HashSet<String> =
            ALWAYS_DEFINED.iter().map(|name| name.to_string()).collect();
        EXTERN_GLOBALS.with(|names| written.extend(names.borrow().iter().cloned()));
        Self { written, dynamic: false, flow: false }
    }

    /// Returns whether a read of `name` is certainly undefined in this scope.
    fn read_is_undefined(&self, name: &str) -> bool {
        !self.dynamic && !self.written.contains(name) && !is_compiler_temp(name)
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
    let mut maybe = facts.written.clone();
    collect_stmts(body, &mut facts);
    rewrite_scope_stmts(body, &facts);
    if !facts.dynamic {
        rewrite_reads_before_writes(body, &mut maybe);
    }
}

/// Runs array auto-vivification on one scope unless it can define names dynamically.
fn vivify_scope(body: &mut Vec<Stmt>, initial: &ScopeFacts) {
    let mut probe = ScopeFacts { written: HashSet::new(), dynamic: false, flow: false };
    collect_stmts(body, &mut probe);
    if probe.dynamic {
        return;
    }
    let mut seen = initial.written.clone();
    // A variable the scope also assigns as a whole (`$a = false;`) is left to the checker: an
    // inserted `$a = []` would type it as an array and refuse that later reassignment.
    collect_plain_assigned_names(body, &mut seen);
    crate::undefined_constants::walk_program_exprs_mut(body, &mut |expr| {
        if let ExprKind::Assignment { target, .. } = &expr.kind {
            if let ExprKind::Variable(name) = &target.kind {
                seen.insert(name.clone());
            }
        }
    });
    vivify_stmts(body, &mut seen, false);
}

/// Records the names a scope's statements assign as a whole (not element writes).
fn collect_plain_assigned_names(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::Assign { name, .. } | StmtKind::TypedAssign { name, .. } => {
                names.insert(name.clone());
            }
            StmtKind::RefAssign { target, .. } => {
                names.insert(target.clone());
            }
            _ => for_each_child_body(stmt, &mut |body| collect_plain_assigned_names(body, names)),
        }
    }
}

/// Rewrites undefined reads inside one expression of the current scope.
fn rewrite_expr(expr: &mut Expr, facts: &ScopeFacts) {
    match &mut expr.kind {
        ExprKind::Variable(name) if facts.read_is_undefined(name) => {
            let span = expr.span;
            let name = std::mem::take(name);
            let warning = Expr::new(
                ExprKind::FunctionCall {
                    name: Name::compiler_call(CompilerCall::UndefinedVariable),
                    args: vec![Expr::new(ExprKind::StringLiteral(name), span)],
                },
                span,
            );
            // The read is a boxed `mixed` null: php's null takes part in arithmetic, bitwise
            // and array operations that a `null`-typed operand would be refused in.
            *expr = Expr::new(
                ExprKind::FunctionCall {
                    name: Name::compiler_call(CompilerCall::MixedReceiver),
                    args: vec![warning],
                },
                span,
            );
        }
        // php reports nothing for probes or the left side of `??`.
        ExprKind::NullCoalesce { value, default } => {
            rewrite_probe(value, facts);
            rewrite_expr(default, facts);
        }
        // `@` suppresses the warning but the read still yields `null`.
        ExprKind::ErrorSuppress(inner) => silence_undefined_reads(inner, facts),
        ExprKind::FunctionCall { name, args } => {
            let key = crate::names::php_symbol_key(name.as_str().trim_start_matches('\\'));
            let short = key.rsplit('\\').next().unwrap_or(&key).to_string();
            if PROBE_CALLS.contains(&short.as_str()) {
                for arg in args.iter_mut() {
                    rewrite_probe(arg, facts);
                }
            } else {
                rewrite_exprs(args, facts);
            }
        }
        ExprKind::Closure { params, variadic, body, is_arrow, captures, capture_refs, .. } => {
            if !*is_arrow && !facts.flow {
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

/// Rewrites a probed lvalue (`isset($a[$k])`, `$o->p ?? d`): its base stays silent, but the
/// index and dynamic-name expressions along the spine are ordinary reads.
fn rewrite_probe(expr: &mut Expr, facts: &ScopeFacts) {
    match &mut expr.kind {
        ExprKind::Variable(_) => {}
        ExprKind::ArrayAccess { array, index } => {
            rewrite_probe(array, facts);
            rewrite_expr(index, facts);
        }
        ExprKind::PropertyAccess { object, .. } | ExprKind::NullsafePropertyAccess { object, .. } => {
            rewrite_probe(object, facts)
        }
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            rewrite_probe(object, facts);
            rewrite_expr(property, facts);
        }
        _ => rewrite_expr(expr, facts),
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

/// Returns whether some statement of `stmts` (nested blocks included, nested functions not)
/// writes an element of the variable `name`.
pub(crate) fn writes_elements_of(stmts: &[Stmt], name: &str) -> bool {
    stmts.iter().any(|stmt| {
        if element_write_root(stmt).as_deref() == Some(name) {
            return true;
        }
        // A compound element update (`$a[] += 1`) is an assignment expression whose prelude
        // holds the element writes.
        if let StmtKind::ExprStmt(Expr { kind: ExprKind::Assignment { prelude, .. }, .. }) = &stmt.kind {
            if writes_elements_of(prelude, name) {
                return true;
            }
        }
        let mut found = false;
        for_each_child_body(stmt, &mut |body| found = found || writes_elements_of(body, name));
        found
    })
}

/// Returns the variable a statement writes an element of, if it is that kind of statement.
fn element_write_root(stmt: &Stmt) -> Option<String> {
    match &stmt.kind {
        // A nested push (`$x['k'][] = v`) is desugared into a block that reads the branch into
        // a temporary, appends, and writes it back: the block's last statement names the
        // variable, and the array must exist before the block's first read of it.
        StmtKind::Synthetic(body) => body
            .last()
            .and_then(element_write_root)
            .filter(|root| !is_compiler_temp(root)),
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
    let mut facts = ScopeFacts { written: HashSet::new(), dynamic: false, flow: false };
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

/// Rewrites reads that no write can precede, statement by statement (php warns there even when
/// the scope writes the name later, e.g. `if ($abc > 5) {...} $abc = 1;`).
///
/// `maybe` holds the names some path may have written before the current point: branches merge
/// by union, a loop first adds every write of its body (a later write reaches the next
/// iteration), and `unset($x)` removes `$x` again. Inside one statement reads are taken in
/// evaluation order (`$a = isset($a) ? $a : []` reads `$a` before writing it). A statement that
/// writes through or increments a name nothing defined yet gets that name initialized first,
/// the way php's fetch-for-write meets an undefined variable.
fn rewrite_reads_before_writes(stmts: &mut Vec<Stmt>, maybe: &mut HashSet<String>) {
    let mut index = 0;
    while index < stmts.len() {
        // A loop-carried name goes on to hold whatever the body writes, so its init is a boxed
        // `mixed` null; a write-through root keeps a plain null, whose property write raises
        // php's Error.
        let mut inits: Vec<(String, bool, bool)> = loop_first_iteration_undefined(&stmts[index], maybe)
            .into_iter()
            .map(|(name, warns)| (name, warns, true))
            .collect();
        inits.extend(
            undefined_write_roots(&stmts[index], maybe)
                .into_iter()
                .map(|(name, warns)| (name, warns, false)),
        );
        for (name, warns, boxed) in inits {
            let span = stmts[index].span;
            stmts.insert(index, undefined_init(&name, warns, boxed, span));
            maybe.insert(name);
            index += 1;
        }
        rewrite_flow_stmt(&mut stmts[index], maybe);
        index += 1;
    }
}

/// Names a loop reads (or writes through) on its first iteration before the loop itself defines
/// them; the loop body's own writes only reach later iterations.
///
/// The body is rewritten on a copy with the entry state; every name of the loop's writes that
/// comes out as an undefined read or a fresh init is hoisted before the loop. The hoisted init
/// warns only when the body certainly runs (`do`/`while`, `for(;;)`), since it fires once at
/// entry rather than at the read.
fn loop_first_iteration_undefined(stmt: &Stmt, maybe: &HashSet<String>) -> Vec<(String, bool)> {
    let body_runs = match &stmt.kind {
        StmtKind::DoWhile { .. } => true,
        StmtKind::For { condition, .. } => condition.is_none(),
        StmtKind::While { .. } | StmtKind::Foreach { .. } => false,
        _ => return Vec::new(),
    };
    let mut own = ScopeFacts { written: HashSet::new(), dynamic: false, flow: false };
    collect_stmt(stmt, &mut own);
    let mut entry = maybe.clone();
    let mut probe = stmt.clone();
    match &mut probe.kind {
        StmtKind::While { condition, .. } => rewrite_ordered(condition, &mut entry),
        StmtKind::For { init, condition, .. } => {
            if let Some(init) = init {
                let mut writes = ScopeFacts { written: HashSet::new(), dynamic: false, flow: true };
                collect_stmt(init, &mut writes);
                entry.extend(writes.written);
            }
            if let Some(condition) = condition {
                rewrite_ordered(condition, &mut entry);
            }
        }
        StmtKind::Foreach { key_var, value_var, .. } => {
            entry.extend(key_var.iter().cloned());
            entry.insert(value_var.clone());
        }
        _ => {}
    }
    let original_null_inits = null_inits(stmt);
    let mut hoisted: Vec<(String, bool)> = Vec::new();
    let mut record = |name: &str, warns: bool| {
        if own.written.contains(name) && !maybe.contains(name) && !is_compiler_temp(name) && !hoisted.iter().any(|(seen, _)| seen == name) {
            hoisted.push((name.to_string(), warns && body_runs));
        }
    };
    for_each_child_body_mut(&mut probe, &mut |body| {
        let mut state = entry.clone();
        rewrite_reads_before_writes(body, &mut state);
    });
    if let StmtKind::While { condition, .. } | StmtKind::For { condition: Some(condition), .. } = &probe.kind {
        visit_undefined_reads(condition, &mut |name| record(name, true));
    }
    let mut visit_stmts = |body: &mut Vec<Stmt>| {
        crate::undefined_constants::walk_program_exprs_mut(body, &mut |expr| {
            if let Some(name) = undefined_read_name(expr) {
                record(&name, true);
            }
        });
    };
    for_each_child_body_mut(&mut probe, &mut visit_stmts);
    for (name, count) in null_inits(&probe) {
        if count > original_null_inits.get(&name).copied().unwrap_or(0) {
            record(&name, false);
        }
    }
    hoisted
}

/// Returns the name an `UndefinedVariable` warning call reports, when `expr` is one.
fn undefined_read_name(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::FunctionCall { name, args }
            if name.compiler_runtime() == Some(CompilerCall::UndefinedVariable) =>
        {
            match args.first().map(|arg| &arg.kind) {
                Some(ExprKind::StringLiteral(name)) => Some(name.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Calls `report` for every `UndefinedVariable` warning call inside `expr`.
fn visit_undefined_reads(expr: &Expr, report: &mut dyn FnMut(&str)) {
    if let Some(name) = undefined_read_name(expr) {
        report(&name);
        return;
    }
    for_each_child_expr(expr, &mut |child| visit_undefined_reads(child, report));
}

/// Counts the silent `$x = null;` statements per name in a statement's bodies.
fn null_inits(stmt: &Stmt) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    fn walk(stmts: &[Stmt], counts: &mut std::collections::HashMap<String, usize>) {
        for stmt in stmts {
            if let StmtKind::Assign { name, value } = &stmt.kind {
                if matches!(value.kind, ExprKind::Null) {
                    *counts.entry(name.clone()).or_insert(0) += 1;
                }
            }
            for_each_child_body(stmt, &mut |body| walk(body, counts));
        }
    }
    for_each_child_body(stmt, &mut |body| walk(body, &mut counts));
    counts
}

/// Applies the flow-sensitive rewrite to one statement and records its writes in `maybe`.
fn rewrite_flow_stmt(stmt: &mut Stmt, maybe: &mut HashSet<String>) {
    let mut own = ScopeFacts { written: HashSet::new(), dynamic: false, flow: false };
    collect_stmt(stmt, &mut own);
    match &mut stmt.kind {
        StmtKind::If { condition, then_body, elseif_clauses, else_body } => {
            // A condition's writes (`if ($m = f())`, `preg_match(.., $m)`) count for the
            // conditions themselves and precede every branch body.
            rewrite_ordered(condition, maybe);
            let mut merged = maybe.clone();
            let mut branch = maybe.clone();
            rewrite_reads_before_writes(then_body, &mut branch);
            merged.extend(branch);
            for (condition, body) in elseif_clauses.iter_mut() {
                rewrite_ordered(condition, maybe);
                merged.extend(maybe.iter().cloned());
                let mut branch = maybe.clone();
                rewrite_reads_before_writes(body, &mut branch);
                merged.extend(branch);
            }
            if let Some(body) = else_body {
                let mut branch = maybe.clone();
                rewrite_reads_before_writes(body, &mut branch);
                merged.extend(branch);
            }
            *maybe = merged;
        }
        StmtKind::Foreach { array, .. } => {
            // The subject is evaluated once, before the loop writes its key and value.
            rewrite_ordered(array, maybe);
            maybe.extend(own.written.iter().cloned());
            for_each_child_body_mut(stmt, &mut |body| rewrite_reads_before_writes(body, maybe));
        }
        StmtKind::Switch { subject, .. } => {
            rewrite_ordered(subject, maybe);
            maybe.extend(own.written.iter().cloned());
            for_each_child_body_mut(stmt, &mut |body| rewrite_reads_before_writes(body, maybe));
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            // The try body runs in order; a catch or finally may follow any point of it.
            rewrite_reads_before_writes(try_body, maybe);
            maybe.extend(own.written.iter().cloned());
            for catch in catches.iter_mut() {
                rewrite_reads_before_writes(&mut catch.body, maybe);
            }
            if let Some(body) = finally_body {
                rewrite_reads_before_writes(body, maybe);
            }
        }
        StmtKind::While { .. } | StmtKind::DoWhile { .. } | StmtKind::For { .. } => {
            // Every write inside may reach any read inside (loops repeat): add them first, then
            // walk the bodies.
            maybe.extend(own.written.iter().cloned());
            for_each_child_body_mut(stmt, &mut |body| rewrite_reads_before_writes(body, maybe));
            if let StmtKind::While { condition, .. } | StmtKind::DoWhile { condition, .. } = &mut stmt.kind {
                rewrite_expr(condition, &facts_from(maybe));
            }
        }
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::IncludeOnceGuard { body, .. } => rewrite_reads_before_writes(body, maybe),
        StmtKind::FunctionDecl { .. }
        | StmtKind::ClassDecl { .. }
        | StmtKind::TraitDecl { .. }
        | StmtKind::InterfaceDecl { .. }
        | StmtKind::EnumDecl { .. } => {}
        StmtKind::Assign { name, value } | StmtKind::TypedAssign { name, value, .. } => {
            rewrite_ordered(value, maybe);
            maybe.insert(name.clone());
        }
        StmtKind::ArrayPush { value, .. }
        | StmtKind::Echo(value)
        | StmtKind::Throw(value)
        | StmtKind::Return(Some(value)) => rewrite_ordered(value, maybe),
        StmtKind::ExprStmt(value) => {
            if let Some(names) = unset_variables(value) {
                for name in names {
                    if !UNKILLABLE.with(|kept| kept.borrow().contains(&name)) {
                        maybe.remove(&name);
                    }
                }
                return;
            }
            rewrite_ordered(value, maybe);
        }
        StmtKind::ArrayAssign { index, value, .. } => {
            rewrite_ordered(index, maybe);
            rewrite_ordered(value, maybe);
        }
        // The value of a property write is read before the property is stored.
        StmtKind::PropertyAssign { value, .. }
        | StmtKind::PropertyArrayPush { value, .. }
        | StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => rewrite_ordered(value, maybe),
        StmtKind::PropertyArrayAssign { index, value, .. }
        | StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            rewrite_ordered(index, maybe);
            rewrite_ordered(value, maybe);
        }
        _ => {}
    }
    maybe.extend(own.written);
}

/// Returns the plain variables a statement-level `unset(...)` destroys, when it is one.
fn unset_variables(expr: &Expr) -> Option<Vec<String>> {
    let ExprKind::FunctionCall { name, args } = &expr.kind else {
        return None;
    };
    let key = crate::names::php_symbol_key(name.as_str().trim_start_matches('\\'));
    if key != "unset" {
        return None;
    }
    Some(
        args.iter()
            .filter_map(|arg| match &arg.kind {
                ExprKind::Variable(name) => Some(name.clone()),
                _ => None,
            })
            .collect(),
    )
}

/// Rewrites undefined reads of `expr` in evaluation order, adding its writes to `defined` as
/// they happen: an assignment's value is read before its target is written.
fn rewrite_ordered(expr: &mut Expr, defined: &mut HashSet<String>) {
    let mut writes = ScopeFacts { written: HashSet::new(), dynamic: false, flow: true };
    collect_expr(expr, &mut writes);
    if writes.written.iter().all(|name| defined.contains(name)) {
        rewrite_expr(expr, &facts_from(defined));
        return;
    }
    match &mut expr.kind {
        ExprKind::Assignment { target, value, .. } if matches!(target.kind, ExprKind::Variable(_)) => {
            rewrite_ordered(value, defined);
            defined.extend(writes.written);
        }
        ExprKind::BinaryOp { .. }
        | ExprKind::Ternary { .. }
        | ExprKind::ShortTernary { .. }
        | ExprKind::Not(_)
        | ExprKind::Negate(_)
        | ExprKind::BitNot(_)
        | ExprKind::Cast { .. }
        | ExprKind::ArrayLiteral(_)
        | ExprKind::ArrayLiteralAssoc(_) => {
            for_each_child_expr_mut(expr, &mut |child| rewrite_ordered(child, defined));
            defined.extend(writes.written);
        }
        // Calls (whose variable arguments may be by-reference), increments and other writing
        // forms keep the conservative statement view: their writes count before their reads.
        _ => {
            defined.extend(writes.written);
            rewrite_expr(expr, &facts_from(defined));
        }
    }
}

/// Names a statement writes through (a property, a compound element write, an increment, a
/// reference source) while no path has defined them, with whether php warns for the read.
fn undefined_write_roots(stmt: &Stmt, maybe: &HashSet<String>) -> Vec<(String, bool)> {
    let mut roots: Vec<(String, bool)> = Vec::new();
    let add = |name: &str, warns: bool, roots: &mut Vec<(String, bool)>| {
        if maybe.contains(name) || ALWAYS_DEFINED.contains(&name) || name == "this" || is_compiler_temp(name) {
            return;
        }
        if let Some(seen) = roots.iter_mut().find(|(seen, _)| seen == name) {
            seen.1 |= warns;
        } else {
            roots.push((name.to_string(), warns));
        }
    };
    let mut reads = HashSet::new();
    match &stmt.kind {
        StmtKind::RefAssign { source, .. } => {
            if let Some(root) = spine_root(source) {
                add(root, false, &mut roots);
            }
            return roots;
        }
        // A reference stored into an element or property (`$o->p =& $x`, parsed as
        // `<target> = $x; $x =& <target>;`) creates an undefined `$x` silently, as php does.
        StmtKind::Synthetic(body) => {
            // Only the two-statement shape the parser builds: a copy of `$x`, then `$x =& ...`.
            if let [store, Stmt { kind: StmtKind::RefAssign { target, .. }, .. }] = body.as_slice() {
                let copies_target = match &store.kind {
                    StmtKind::ArrayAssign { value, .. }
                    | StmtKind::ArrayPush { value, .. }
                    | StmtKind::PropertyAssign { value, .. }
                    | StmtKind::NestedArrayAssign { value, .. } => {
                        matches!(&value.kind, ExprKind::Variable(name) if name == target)
                    }
                    _ => false,
                };
                if copies_target {
                    add(target, false, &mut roots);
                }
            }
            return roots;
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            collect_value_reads(value, &mut reads);
            if let Some(root) = spine_root(object) {
                let warns = reads.contains(root);
                add(root, warns, &mut roots);
            }
            return roots;
        }
        // A compound element write (`$a[] &= 1`) reads the undefined array first; a plain one
        // auto-vivifies silently and is left to the vivify pass.
        StmtKind::ArrayPush { array, value } | StmtKind::ArrayAssign { array, value, .. } => {
            collect_value_reads(value, &mut reads);
            if reads.contains(array) {
                add(array, true, &mut roots);
            }
            return roots;
        }
        StmtKind::NestedArrayAssign { target, value } => {
            collect_value_reads(value, &mut reads);
            if let Some(root) = spine_root(target).filter(|root| reads.contains(*root)) {
                add(root, true, &mut roots);
            }
            return roots;
        }
        StmtKind::ExprStmt(expr) | StmtKind::Echo(expr) | StmtKind::Return(Some(expr)) => {
            collect_value_reads(expr, &mut reads);
            visit_write_roots(expr, &reads, &mut |name, warns| add(name, warns, &mut roots));
        }
        StmtKind::Assign { value, .. } | StmtKind::TypedAssign { value, .. } => {
            collect_value_reads(value, &mut reads);
            visit_write_roots(value, &reads, &mut |name, warns| add(name, warns, &mut roots));
        }
        _ => {}
    }
    roots
}

/// Reports the increment targets and the roots of property or compound element writes in `expr`.
fn visit_write_roots(expr: &Expr, reads: &HashSet<String>, report: &mut dyn FnMut(&str, bool)) {
    match &expr.kind {
        ExprKind::PreIncrement(name)
        | ExprKind::PostIncrement(name)
        | ExprKind::PreDecrement(name)
        | ExprKind::PostDecrement(name) => report(name, true),
        ExprKind::Assignment { target, value, .. } => {
            match &target.kind {
                ExprKind::PropertyAccess { object, .. }
                | ExprKind::DynamicPropertyAccess { object, .. } => {
                    if let Some(root) = spine_root(object) {
                        report(root, reads.contains(root));
                    }
                }
                // A plain element write auto-vivifies silently (the vivify pass seeds it); only
                // a compound one (`$a[] &= 1`) reads the undefined variable first.
                ExprKind::ArrayAccess { .. } => {
                    if let Some(root) = spine_root(target) {
                        if reads.contains(root) {
                            report(root, true);
                        }
                    }
                }
                _ => {}
            }
            visit_write_roots(value, reads, report);
        }
        ExprKind::Closure { .. } => {}
        _ => for_each_child_expr(expr, &mut |child| visit_write_roots(child, reads, report)),
    }
}

/// Collects the variables an expression reads as values (not the targets it writes).
fn collect_value_reads(expr: &Expr, reads: &mut HashSet<String>) {
    match &expr.kind {
        ExprKind::Variable(name) => {
            reads.insert(name.clone());
        }
        ExprKind::Assignment { value, .. } => collect_value_reads(value, reads),
        ExprKind::Closure { .. } => {}
        _ => for_each_child_expr(expr, &mut |child| collect_value_reads(child, reads)),
    }
}

/// Returns the variable at the base of a write target (`$a`, `$a[1]->p`, `$o->p[2]`).
fn spine_root(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name),
        ExprKind::ArrayAccess { array, .. } => spine_root(array),
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. }
        | ExprKind::DynamicPropertyAccess { object, .. }
        | ExprKind::NullsafeDynamicPropertyAccess { object, .. } => spine_root(object),
        _ => None,
    }
}

/// Builds `$name = null;`, through the warning call when php reports the undefined read.
fn undefined_init(name: &str, warns: bool, boxed: bool, span: crate::span::Span) -> Stmt {
    let value = if warns {
        Expr::new(
            ExprKind::FunctionCall {
                name: Name::compiler_call(CompilerCall::UndefinedVariable),
                args: vec![Expr::new(ExprKind::StringLiteral(name.to_string()), span)],
            },
            span,
        )
    } else {
        Expr::new(ExprKind::Null, span)
    };
    if !boxed {
        return Stmt::new(StmtKind::Assign { name: name.to_string(), value }, span);
    }
    // Boxed as `mixed`: the variable goes on to take whatever the loop writes, and a
    // `null`-typed local would refuse it (`$a &= 15`, `$a[$k][] = v`).
    let value = Expr::new(
        ExprKind::FunctionCall {
            name: Name::compiler_call(CompilerCall::MixedReceiver),
            args: vec![value],
        },
        span,
    );
    Stmt::new(StmtKind::Assign { name: name.to_string(), value }, span)
}

/// Returns whether `name` is a compiler temporary (`…#gen`): the desugaring that introduces one
/// always defines it before reading it, often in an assignment's prelude this pass reads around.
fn is_compiler_temp(name: &str) -> bool {
    name.contains('#')
}

/// Builds rewrite facts whose defined names are `names`.
fn facts_from(names: &HashSet<String>) -> ScopeFacts {
    ScopeFacts { written: names.clone(), dynamic: false, flow: true }
}

/// Mutable twin of `for_each_child_body`, for other AST passes.
pub(crate) fn for_each_child_body_mut_pub(stmt: &mut Stmt, visit: &mut dyn FnMut(&mut Vec<Stmt>)) {
    for_each_child_body_mut(stmt, visit);
}

/// Returns whether evaluating `expr` may write the variable `name` (assignment, increment,
/// any call argument, which may be by reference, or a closure capture).
pub(crate) fn expr_may_write_name(expr: &Expr, name: &str) -> bool {
    let mut facts = ScopeFacts { written: HashSet::new(), dynamic: false, flow: false };
    collect_expr(expr, &mut facts);
    facts.written.contains(name)
}

/// Adds the names one scope's statements write (assignments, `global`, `foreach`, ...).
pub(crate) fn collect_scope_written_names(stmts: &[Stmt], names: &mut HashSet<String>) {
    let mut facts = ScopeFacts { written: HashSet::new(), dynamic: false, flow: false };
    collect_stmts(stmts, &mut facts);
    names.extend(facts.written);
}

/// Mutable twin of `for_each_child_body`.
fn for_each_child_body_mut(stmt: &mut Stmt, visit: &mut dyn FnMut(&mut Vec<Stmt>)) {
    match &mut stmt.kind {
        StmtKind::If { then_body, elseif_clauses, else_body, .. } => {
            visit(then_body);
            for (_, body) in elseif_clauses.iter_mut() {
                visit(body);
            }
            if let Some(body) = else_body {
                visit(body);
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::For { body, .. } => visit(body),
        StmtKind::Switch { cases, default, .. } => {
            for (_, body) in cases.iter_mut() {
                visit(body);
            }
            if let Some(body) = default {
                visit(body);
            }
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            visit(try_body);
            for catch in catches.iter_mut() {
                visit(&mut catch.body);
            }
            if let Some(body) = finally_body {
                visit(body);
            }
        }
        _ => {}
    }
}
