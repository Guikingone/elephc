//! Purpose:
//! Syntactic pre-scans for the integer-range analysis: exhaustive AST child walkers, the
//! program-wide facts the analysis needs (by-reference parameter shapes, `int` return types,
//! globals reachable from functions), and the per-body set of variables that can change
//! behind the analysis' back ("pinned" variables).
//!
//! Called from:
//! - `crate::int_range::analyzer` before interpreting a function or the top-level script.
//!
//! Key details:
//! - The walkers match every `StmtKind`/`ExprKind` variant without a wildcard arm, so a new AST
//!   form fails to compile here instead of silently escaping the analysis.
//! - Pinning is flow-insensitive and conservative: a variable that may be aliased (references,
//!   `global`, `static`, by-reference arguments, `finally` writes) is treated as unknown for the
//!   whole body, and a body using dynamic scope features (`goto`, runtime `include`, `extract`,
//!   `eval`, ...) is not analyzed at all.

use std::collections::{HashMap, HashSet};

use crate::parser::ast::{
    CallableTarget, ClassMethod, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind, TypeExpr,
};

/// Function parameter tuple as stored in the AST: `(name, type, default, by_ref)`.
pub(crate) type Param = (String, Option<TypeExpr>, Option<Expr>, bool);

/// Builtins that inspect or rewrite the caller's whole symbol table.
const SCOPE_BUILTINS: &[&str] = &["extract", "parse_str", "mb_parse_str", "get_defined_vars", "eval"];

/// Language constructs parsed as calls whose operands are never bound by reference.
const NON_BINDING_CONSTRUCTS: &[&str] = &["isset", "empty", "unset"];

/// Which argument positions of a callee may bind a reference to the caller's variable.
#[derive(Debug, Clone, Default)]
pub(crate) struct RefShape {
    positions: Vec<bool>,
    variadic_by_ref: bool,
}

impl RefShape {
    /// Builds the shape of a user-declared parameter list.
    fn from_params(params: &[Param], variadic_by_ref: bool) -> Self {
        Self {
            positions: params.iter().map(|param| param.3).collect(),
            variadic_by_ref,
        }
    }

    /// Merges two shapes (a position binds by reference when either shape does).
    fn merge(&mut self, other: &RefShape) {
        if other.positions.len() > self.positions.len() {
            self.positions.resize(other.positions.len(), false);
        }
        for (index, by_ref) in other.positions.iter().enumerate() {
            self.positions[index] |= *by_ref;
        }
        self.variadic_by_ref |= other.variadic_by_ref;
    }

    /// Returns `true` when the positional argument `index` may be bound by reference.
    fn binds_position(&self, index: usize) -> bool {
        match self.positions.get(index) {
            Some(by_ref) => *by_ref,
            None => self.variadic_by_ref,
        }
    }

    /// Returns `true` when any parameter is by reference (used for named arguments).
    fn binds_any(&self) -> bool {
        self.variadic_by_ref || self.positions.iter().any(|by_ref| *by_ref)
    }
}

/// Program-wide facts shared by every analyzed body.
#[derive(Debug, Default)]
pub(crate) struct ProgramInfo {
    functions: HashMap<String, RefShape>,
    methods: HashMap<String, RefShape>,
    classes: HashSet<String>,
    int_functions: HashMap<String, bool>,
    /// Top-level variables that a function can reach through `global` or `$GLOBALS[...]`.
    pub(crate) top_level_pinned: HashSet<String>,
}

impl ProgramInfo {
    /// Collects declarations and global aliases from the whole program.
    pub(crate) fn collect(program: &[Stmt]) -> Self {
        let mut info = ProgramInfo::default();
        let mut names = HashSet::new();
        walk_stmts(program, true, &mut |node| match node {
            Node::Stmt(stmt) => {
                info.observe_stmt(stmt);
                stmt_writes(stmt, &mut names);
            }
            Node::Expr(expr) => {
                expr_writes(expr, &mut names);
                if let ExprKind::Variable(name) = &expr.kind {
                    names.insert(name.clone());
                }
            }
        });
        for name in &names {
            if let Some(target) = global_alias_target(name) {
                info.top_level_pinned.insert(target.to_string());
            }
        }
        info
    }

    /// Records the declaration facts carried by one statement.
    fn observe_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::FunctionDecl {
                name,
                params,
                variadic_by_ref,
                return_type,
                ..
            } => {
                let key = symbol_key(name);
                self.functions
                    .entry(key.clone())
                    .or_default()
                    .merge(&RefShape::from_params(params, *variadic_by_ref));
                let returns_int = matches!(return_type, Some(TypeExpr::Int));
                let entry = self.int_functions.entry(key).or_insert(true);
                *entry &= returns_int;
            }
            StmtKind::ClassDecl { name, methods, .. }
            | StmtKind::EnumDecl { name, methods, .. }
            | StmtKind::InterfaceDecl { name, methods, .. }
            | StmtKind::TraitDecl { name, methods, .. } => {
                self.classes.insert(symbol_key(name));
                for method in methods {
                    self.observe_method(method);
                }
            }
            StmtKind::Global { vars } => {
                self.top_level_pinned.extend(vars.iter().cloned());
            }
            _ => {}
        }
    }

    /// Records a method's by-reference shape under its (case-insensitive) name.
    fn observe_method(&mut self, method: &ClassMethod) {
        self.methods
            .entry(symbol_key(&method.name))
            .or_default()
            .merge(&RefShape::from_params(&method.params, method.variadic_by_ref));
    }

    /// Returns `true` when the user function `name` is declared with an `int` return type.
    pub(crate) fn user_function_returns_int(&self, name: &str) -> bool {
        self.int_functions.get(&symbol_key(name)).copied().unwrap_or(false)
    }

    /// Returns `true` when `name` is a user-declared function.
    pub(crate) fn is_user_function(&self, name: &str) -> bool {
        self.functions.contains_key(&symbol_key(name))
    }

    /// By-reference shape of a direct function call, or `None` when the callee is unknown.
    fn function_shape(&self, name: &str) -> Option<RefShape> {
        let key = symbol_key(name);
        if NON_BINDING_CONSTRUCTS.contains(&key.as_str()) {
            return Some(RefShape::default());
        }
        if let Some(shape) = self.functions.get(&key) {
            return Some(shape.clone());
        }
        let sig = crate::types::builtin_call_sig(&key)?;
        Some(RefShape {
            positions: sig.ref_params.clone(),
            variadic_by_ref: false,
        })
    }

    /// By-reference shape of a method call by name, or `None` when no user method has that name.
    fn method_shape(&self, method: &str) -> Option<&RefShape> {
        self.methods.get(&symbol_key(method))
    }

    /// By-reference shape of a constructor call, or `None` for non-user classes.
    fn constructor_shape(&self, class_name: &str) -> Option<RefShape> {
        if !self.classes.contains(&symbol_key(class_name)) {
            return None;
        }
        Some(self.methods.get("__construct").cloned().unwrap_or_default())
    }
}

/// Canonical lookup key for a function, method, or class name.
pub(crate) fn symbol_key(name: &str) -> String {
    crate::names::php_symbol_key(name.trim_start_matches('\\'))
}

/// Returns the top-level variable aliased by a `$GLOBALS['name']` rewrite, if `name` is one.
pub(crate) fn global_alias_target(name: &str) -> Option<&str> {
    name.strip_prefix("GLOBALS[")?.strip_suffix(']')
}

/// Result of scanning one function (or script) body.
#[derive(Debug, Default)]
pub(crate) struct BodyScan {
    /// Variables that may change without a visible write in this body.
    pub(crate) pinned: HashSet<String>,
    /// `true` when the body uses a feature that defeats local reasoning.
    pub(crate) unanalyzable: bool,
}

/// Scans a body for pinned variables and unanalyzable features.
pub(crate) fn scan_body(body: &[Stmt], params: &[Param], info: &ProgramInfo) -> BodyScan {
    let mut scan = BodyScan::default();
    for param in params {
        if param.3 {
            scan.pinned.insert(param.0.clone());
        }
    }
    walk_stmts(body, false, &mut |node| match node {
        Node::Stmt(stmt) => scan_stmt(stmt, &mut scan),
        Node::Expr(expr) => scan_expr(expr, info, &mut scan),
    });
    scan
}

/// Applies the pinning rules of one statement.
fn scan_stmt(stmt: &Stmt, scan: &mut BodyScan) {
    match &stmt.kind {
        StmtKind::Global { vars } => scan.pinned.extend(vars.iter().cloned()),
        StmtKind::StaticVar { name, .. } => {
            scan.pinned.insert(name.clone());
        }
        StmtKind::RefAssign { target, source } => {
            scan.pinned.insert(target.clone());
            pin_root(source, scan);
        }
        StmtKind::RefAssignToTarget { target, source, .. } => {
            pin_root(target, scan);
            pin_root(source, scan);
        }
        StmtKind::Foreach {
            array,
            value_var,
            value_by_ref: true,
            ..
        } => {
            scan.pinned.insert(value_var.clone());
            pin_root(array, scan);
        }
        StmtKind::Try {
            finally_body: Some(finally_body),
            ..
        } => {
            // A `finally` runs on every exit path (break/continue/return/exception); keeping
            // its writes out of the tracked set avoids modelling those edges precisely.
            let mut writes = HashSet::new();
            collect_writes_stmts(finally_body, &mut writes);
            scan.pinned.extend(writes);
        }
        StmtKind::Goto(_) | StmtKind::Label(_) | StmtKind::Include { .. } => {
            scan.unanalyzable = true;
        }
        _ => {}
    }
}

/// Applies the pinning rules of one expression node.
fn scan_expr(expr: &Expr, info: &ProgramInfo, scan: &mut BodyScan) {
    match &expr.kind {
        ExprKind::Closure { capture_refs, .. } => scan.pinned.extend(capture_refs.iter().cloned()),
        ExprKind::IncludeValue { .. } => scan.unanalyzable = true,
        ExprKind::FunctionCall { name, args } => {
            if SCOPE_BUILTINS.contains(&symbol_key(name.as_str()).as_str()) {
                scan.unanalyzable = true;
            }
            pin_by_ref_args(args, info.function_shape(name.as_str()).as_ref(), scan);
        }
        ExprKind::MethodCall { method, args, .. }
        | ExprKind::NullsafeMethodCall { method, args, .. }
        | ExprKind::StaticMethodCall { method, args, .. } => {
            pin_by_ref_args(args, info.method_shape(method), scan);
        }
        ExprKind::NewObject { class_name, args } => {
            pin_by_ref_args(args, info.constructor_shape(class_name.as_str()).as_ref(), scan);
        }
        ExprKind::ClosureCall { args, .. }
        | ExprKind::ExprCall { args, .. }
        | ExprKind::NewDynamic { args, .. }
        | ExprKind::NewDynamicObject { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::NullsafeDynamicMethodCall { args, .. } => pin_by_ref_args(args, None, scan),
        ExprKind::Pipe { value, .. } => pin_by_ref_args(std::slice::from_ref(value), None, scan),
        _ => {}
    }
}

/// Pins every bare-variable argument that the callee may bind by reference.
///
/// An unknown callee (`None`) may bind any argument.
fn pin_by_ref_args(args: &[Expr], shape: Option<&RefShape>, scan: &mut BodyScan) {
    for (index, arg) in args.iter().enumerate() {
        let (value, binds) = match &arg.kind {
            ExprKind::NamedArg { value, .. } => (&**value, shape.is_none_or(RefShape::binds_any)),
            ExprKind::Spread(value) => (&**value, shape.is_none_or(RefShape::binds_any)),
            _ => (arg, shape.is_none_or(|shape| shape.binds_position(index))),
        };
        if binds {
            pin_root(value, scan);
        }
    }
}

/// Pins the variable at the root of an lvalue-shaped expression.
fn pin_root(expr: &Expr, scan: &mut BodyScan) {
    if let Some(name) = root_variable(expr) {
        scan.pinned.insert(name.to_string());
    }
}

/// Returns the variable at the root of `$v`, `$v[...]`, `$v->p`, ...
pub(crate) fn root_variable(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name),
        ExprKind::ArrayAccess { array, .. } => root_variable(array),
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. }
        | ExprKind::DynamicPropertyAccess { object, .. }
        | ExprKind::NullsafeDynamicPropertyAccess { object, .. } => root_variable(object),
        _ => None,
    }
}

/// Collects the names of locals written anywhere in `stmts` (not inside nested declarations).
pub(crate) fn collect_writes_stmts(stmts: &[Stmt], out: &mut HashSet<String>) {
    walk_stmts(stmts, false, &mut |node| match node {
        Node::Stmt(stmt) => stmt_writes(stmt, out),
        Node::Expr(expr) => expr_writes(expr, out),
    });
}

/// Collects the names of locals written anywhere inside `expr`.
pub(crate) fn collect_writes_expr(expr: &Expr, out: &mut HashSet<String>) {
    walk_expr(expr, false, &mut |node| match node {
        Node::Stmt(stmt) => stmt_writes(stmt, out),
        Node::Expr(expr) => expr_writes(expr, out),
    });
}

/// Collects the names of locals read anywhere inside `expr`.
pub(crate) fn collect_reads_expr(expr: &Expr, out: &mut HashSet<String>) {
    walk_expr(expr, false, &mut |node| match node {
        Node::Expr(expr) => match &expr.kind {
            ExprKind::Variable(name) => {
                out.insert(name.clone());
            }
            ExprKind::PreIncrement(name)
            | ExprKind::PostIncrement(name)
            | ExprKind::PreDecrement(name)
            | ExprKind::PostDecrement(name)
            | ExprKind::ClosureCall { var: name, .. } => {
                out.insert(name.clone());
            }
            ExprKind::Closure { captures, capture_refs, is_arrow, body, .. } => {
                out.extend(captures.iter().cloned());
                out.extend(capture_refs.iter().cloned());
                if *is_arrow {
                    // Arrow functions capture every parent variable they mention, by value.
                    walk_stmts(body, false, &mut |inner| {
                        if let Node::Expr(Expr { kind: ExprKind::Variable(name), .. }) = inner {
                            out.insert(name.clone());
                        }
                    });
                }
            }
            _ => {}
        },
        Node::Stmt(_) => {}
    });
}

/// Records the locals written by one statement node.
fn stmt_writes(stmt: &Stmt, out: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::Assign { name, .. }
        | StmtKind::TypedAssign { name, .. }
        | StmtKind::StaticVar { name, .. }
        | StmtKind::ArrayAssign { array: name, .. }
        | StmtKind::ArrayPush { array: name, .. }
        | StmtKind::RefAssign { target: name, .. } => {
            out.insert(name.clone());
        }
        StmtKind::NestedArrayAssign { target, .. } | StmtKind::RefAssignToTarget { target, .. } => {
            if let Some(name) = root_variable(target) {
                out.insert(name.to_string());
            }
        }
        StmtKind::ListUnpack { vars, .. } | StmtKind::Global { vars } => {
            out.extend(vars.iter().cloned());
        }
        StmtKind::Foreach { key_var, value_var, .. } => {
            out.insert(value_var.clone());
            if let Some(key_var) = key_var {
                out.insert(key_var.clone());
            }
        }
        StmtKind::Try { catches, .. } => {
            for catch in catches {
                if let Some(variable) = &catch.variable {
                    out.insert(variable.clone());
                }
            }
        }
        _ => {}
    }
}

/// Records the locals written by one expression node.
fn expr_writes(expr: &Expr, out: &mut HashSet<String>) {
    match &expr.kind {
        ExprKind::Assignment {
            target,
            conditional_value_temp,
            ..
        } => {
            if let Some(name) = root_variable(target) {
                out.insert(name.to_string());
            }
            if let Some(temp) = conditional_value_temp {
                out.insert(temp.clone());
            }
        }
        ExprKind::ListUnpack { vars, .. } => out.extend(vars.iter().cloned()),
        ExprKind::PreIncrement(name)
        | ExprKind::PostIncrement(name)
        | ExprKind::PreDecrement(name)
        | ExprKind::PostDecrement(name) => {
            out.insert(name.clone());
        }
        ExprKind::FunctionCall { name, args } if symbol_key(name.as_str()) == "unset" => {
            for arg in args {
                if let Some(name) = root_variable(arg) {
                    out.insert(name.to_string());
                }
            }
        }
        _ => {}
    }
}

/// One node handed to a walker callback.
pub(crate) enum Node<'a> {
    Stmt(&'a Stmt),
    Expr(&'a Expr),
}

/// Walks statements in pre-order. With `deep == false` the walk stays inside the current body:
/// nested function, class, and closure bodies are not entered (closure nodes themselves are
/// still reported so their capture lists can be inspected).
pub(crate) fn walk_stmts<'a>(stmts: &'a [Stmt], deep: bool, f: &mut dyn FnMut(Node<'a>)) {
    for stmt in stmts {
        walk_stmt(stmt, deep, f);
    }
}

/// Walks one statement and everything executable below it.
pub(crate) fn walk_stmt<'a>(stmt: &'a Stmt, deep: bool, f: &mut dyn FnMut(Node<'a>)) {
    f(Node::Stmt(stmt));
    match &stmt.kind {
        StmtKind::Echo(value)
        | StmtKind::Assign { value, .. }
        | StmtKind::ArrayPush { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::Throw(value)
        | StmtKind::ExprStmt(value)
        | StmtKind::ConstDecl { value, .. }
        | StmtKind::ListUnpack { value, .. }
        | StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => walk_expr(value, deep, f),
        StmtKind::StaticVar { init, .. } => walk_expr(init, deep, f),
        StmtKind::RefAssign { source, .. } => walk_expr(source, deep, f),
        StmtKind::RefAssignToTarget { target, source, .. } => {
            walk_expr(target, deep, f);
            walk_expr(source, deep, f);
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            walk_expr(condition, deep, f);
            walk_stmts(then_body, deep, f);
            for (condition, body) in elseif_clauses {
                walk_expr(condition, deep, f);
                walk_stmts(body, deep, f);
            }
            if let Some(body) = else_body {
                walk_stmts(body, deep, f);
            }
        }
        StmtKind::IfDef { then_body, else_body, .. } => {
            walk_stmts(then_body, deep, f);
            if let Some(body) = else_body {
                walk_stmts(body, deep, f);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            walk_expr(condition, deep, f);
            walk_stmts(body, deep, f);
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(init) = init {
                walk_stmt(init, deep, f);
            }
            if let Some(condition) = condition {
                walk_expr(condition, deep, f);
            }
            if let Some(update) = update {
                walk_stmt(update, deep, f);
            }
            walk_stmts(body, deep, f);
        }
        StmtKind::ArrayAssign { index, value, .. } => {
            walk_expr(index, deep, f);
            walk_expr(value, deep, f);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            walk_expr(target, deep, f);
            walk_expr(value, deep, f);
        }
        StmtKind::Foreach { array, body, .. } => {
            walk_expr(array, deep, f);
            walk_stmts(body, deep, f);
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            walk_expr(subject, deep, f);
            for (patterns, body) in cases {
                for pattern in patterns {
                    walk_expr(pattern, deep, f);
                }
                walk_stmts(body, deep, f);
            }
            if let Some(body) = default {
                walk_stmts(body, deep, f);
            }
        }
        StmtKind::Include { path, .. } => walk_expr(path, deep, f),
        StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. } => walk_stmts(body, deep, f),
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            walk_stmts(try_body, deep, f);
            for catch in catches {
                walk_stmts(&catch.body, deep, f);
            }
            if let Some(body) = finally_body {
                walk_stmts(body, deep, f);
            }
        }
        StmtKind::Return(value) => {
            if let Some(value) = value {
                walk_expr(value, deep, f);
            }
        }
        StmtKind::FunctionDecl { params, body, .. } => {
            if deep {
                walk_params(params, f);
                walk_stmts(body, deep, f);
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
            if deep {
                for property in properties {
                    if let Some(default) = &property.default {
                        walk_expr(default, deep, f);
                    }
                }
                walk_class_members(methods, constants, f);
            }
        }
        StmtKind::EnumDecl {
            cases,
            methods,
            constants,
            ..
        } => {
            if deep {
                for case in cases {
                    if let Some(value) = &case.value {
                        walk_expr(value, deep, f);
                    }
                }
                walk_class_members(methods, constants, f);
            }
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            walk_expr(object, deep, f);
            walk_expr(value, deep, f);
        }
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            walk_expr(index, deep, f);
            walk_expr(value, deep, f);
        }
        StmtKind::DynamicStaticPropertyWrite {
            property,
            index,
            value,
            ..
        } => {
            walk_expr(property, deep, f);
            if let Some(index) = index {
                walk_expr(index, deep, f);
            }
            walk_expr(value, deep, f);
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            walk_expr(object, deep, f);
            walk_expr(index, deep, f);
            walk_expr(value, deep, f);
        }
        StmtKind::IncludeOnceMark { .. }
        | StmtKind::Break(_)
        | StmtKind::Continue(_)
        | StmtKind::Goto(_)
        | StmtKind::Label(_)
        | StmtKind::NamespaceDecl { .. }
        | StmtKind::UseDecl { .. }
        | StmtKind::FunctionVariantGroup { .. }
        | StmtKind::FunctionVariantMark { .. }
        | StmtKind::Global { .. }
        | StmtKind::PackedClassDecl { .. }
        | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. }
        | StmtKind::ExternGlobalDecl { .. } => {}
    }
}

/// Walks parameter default expressions.
fn walk_params<'a>(params: &'a [Param], f: &mut dyn FnMut(Node<'a>)) {
    for param in params {
        if let Some(default) = &param.2 {
            walk_expr(default, true, f);
        }
    }
}

/// Walks method bodies and constant initializers of a class-like declaration.
fn walk_class_members<'a>(
    methods: &'a [ClassMethod],
    constants: &'a [crate::parser::ast::ClassConst],
    f: &mut dyn FnMut(Node<'a>),
) {
    for method in methods {
        walk_params(&method.params, f);
        walk_stmts(&method.body, true, f);
    }
    for constant in constants {
        walk_expr(&constant.value, true, f);
    }
}

/// Walks one expression in pre-order (see [`walk_stmts`] for the meaning of `deep`).
pub(crate) fn walk_expr<'a>(expr: &'a Expr, deep: bool, f: &mut dyn FnMut(Node<'a>)) {
    f(Node::Expr(expr));
    if let ExprKind::Assignment { prelude, .. } = &expr.kind {
        walk_stmts(prelude, deep, f);
    }
    for_each_child(expr, &mut |child| match child {
        Child::Expr(child) => walk_expr(child, deep, f),
        Child::Body(body, params) => {
            if deep {
                walk_params(params, f);
                walk_stmts(body, deep, f);
            }
        }
    });
}

/// A direct child of an expression.
pub(crate) enum Child<'a> {
    /// A child expression, in evaluation order.
    Expr(&'a Expr),
    /// A closure body with its parameters (callers decide whether it belongs to the scope).
    Body(&'a [Stmt], &'a [Param]),
}

/// Visits the direct executable children of `expr` in evaluation order.
pub(crate) fn for_each_child<'a>(expr: &'a Expr, visit: &mut dyn FnMut(Child<'a>)) {
    let mut on_expr = |child: &'a Expr| visit(Child::Expr(child));
    match &expr.kind {
        ExprKind::StringLiteral(_)
        | ExprKind::IntLiteral(_)
        | ExprKind::FloatLiteral(_)
        | ExprKind::Variable(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::Null
        | ExprKind::PreIncrement(_)
        | ExprKind::PostIncrement(_)
        | ExprKind::PreDecrement(_)
        | ExprKind::PostDecrement(_)
        | ExprKind::ConstRef(_)
        | ExprKind::StaticPropertyAccess { .. }
        | ExprKind::This
        | ExprKind::ClassConstant { .. }
        | ExprKind::ScopedConstantAccess { .. }
        | ExprKind::MagicConstant(_) => {}
        ExprKind::BinaryOp { left, right, .. }
        | ExprKind::NullCoalesce {
            value: left,
            default: right,
        }
        | ExprKind::ShortTernary {
            value: left,
            default: right,
        }
        | ExprKind::Pipe {
            value: left,
            callable: right,
        }
        | ExprKind::ArrayAccess {
            array: left,
            index: right,
        }
        | ExprKind::DynamicPropertyAccess {
            object: left,
            property: right,
        }
        | ExprKind::NullsafeDynamicPropertyAccess {
            object: left,
            property: right,
        } => {
            on_expr(left);
            on_expr(right);
        }
        ExprKind::InstanceOf { value, target } => {
            on_expr(value);
            if let InstanceOfTarget::Expr(target) = target {
                on_expr(target);
            }
        }
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
        | ExprKind::NamedArg { value: inner, .. }
        | ExprKind::IncludeValue { path: inner, .. }
        | ExprKind::PtrCast { expr: inner, .. }
        | ExprKind::BufferNew { len: inner, .. }
        | ExprKind::PropertyAccess { object: inner, .. }
        | ExprKind::NullsafePropertyAccess { object: inner, .. }
        | ExprKind::ObjectClassName { object: inner }
        | ExprKind::DynamicClassConstantAccess { object: inner, .. }
        | ExprKind::DynamicStaticPropertyAccess { property: inner, .. } => on_expr(inner),
        ExprKind::Assignment {
            target,
            value,
            result_target,
            prelude,
            ..
        } => {
            // Prelude statements are walked by `walk_expr` and executed by the analyzer.
            let _ = prelude;
            on_expr(value);
            on_expr(target);
            if let Some(result_target) = result_target {
                on_expr(result_target);
            }
        }
        ExprKind::ListUnpack { value, .. } => on_expr(value),
        ExprKind::FunctionCall { args, .. }
        | ExprKind::ClosureCall { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::StaticMethodCall { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::ArrayLiteral(args) => {
            for arg in args {
                on_expr(arg);
            }
        }
        ExprKind::ArrayLiteralAssoc(entries) => {
            for (key, value) in entries {
                on_expr(key);
                on_expr(value);
            }
        }
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            on_expr(subject);
            for (conditions, value) in arms {
                for condition in conditions {
                    on_expr(condition);
                }
                on_expr(value);
            }
            if let Some(default) = default {
                on_expr(default);
            }
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            on_expr(condition);
            on_expr(then_expr);
            on_expr(else_expr);
        }
        ExprKind::Closure { params, body, .. } => visit(Child::Body(body, params)),
        ExprKind::ExprCall { callee: head, args }
        | ExprKind::NewDynamic {
            name_expr: head,
            args,
        }
        | ExprKind::NewDynamicObject {
            class_name: head,
            args,
            ..
        }
        | ExprKind::MethodCall {
            object: head, args, ..
        }
        | ExprKind::NullsafeMethodCall {
            object: head, args, ..
        } => {
            on_expr(head);
            for arg in args {
                on_expr(arg);
            }
        }
        ExprKind::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => {
            on_expr(object);
            on_expr(method);
            for arg in args {
                on_expr(arg);
            }
        }
        ExprKind::FirstClassCallable(target) => {
            if let CallableTarget::Method { object, .. } = target {
                on_expr(object);
            }
        }
        ExprKind::Yield { key, value } => {
            if let Some(key) = key {
                on_expr(key);
            }
            if let Some(value) = value {
                on_expr(value);
            }
        }
    }
}
