//! Purpose:
//! Collects declared and referenced fully-qualified class-like names from the AST.
//! Gives the autoload pass the missing symbols it can try to resolve from disk.
//!
//! Called from:
//! - `crate::autoload::run()`
//!
//! Key details:
//! - Literal `class_exists(..., true)` shapes are treated as compile-time autoload demands.
//! - Dynamic autoload flags are not guessed because the checker rejects them in AOT mode.

use std::collections::{HashMap, HashSet};

use crate::parser::ast::{
    AttributeGroup, CallableTarget, CatchClause, ClassConst, ClassMethod, ClassProperty, Expr,
    ExprKind, Program, StaticReceiver, Stmt, StmtKind, TraitUse, TypeExpr,
};

/// Stable l-value shapes whose deterministic class-string assignments can seed dynamic `new`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum DynamicClassTarget {
    Variable(String),
    ArrayAccess {
        array: Box<DynamicClassTarget>,
        index: DynamicClassIndex,
    },
}

/// Literal array indexes accepted in a stable dynamic-class target.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum DynamicClassIndex {
    String(String),
    Int(i64),
}

/// Per-statement reference collection state, including prior definite class-string assignments.
struct ReferenceSet<'a> {
    names: HashSet<String>,
    dynamic_class_defaults: &'a HashMap<DynamicClassTarget, HashSet<String>>,
}

impl<'a> ReferenceSet<'a> {
    /// Creates an empty reference set backed by the current definite-assignment facts.
    fn new(dynamic_class_defaults: &'a HashMap<DynamicClassTarget, HashSet<String>>) -> Self {
        Self {
            names: HashSet::new(),
            dynamic_class_defaults,
        }
    }

    /// Inserts one canonical class-like name into this statement's autoload demands.
    fn insert(&mut self, name: String) {
        self.names.insert(name);
    }
}

/// Collect all declared fully-qualified class-like names from the program.
pub(super) fn collect_declared_fqns(program: &Program) -> HashSet<String> {
    let mut out = HashSet::new();
    for stmt in program {
        collect_declared_in_stmt(stmt, &mut out);
    }
    out
}

/// Recurse into a statement to collect declared class names.
fn collect_declared_in_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::ClassDecl { name, .. }
        | StmtKind::InterfaceDecl { name, .. }
        | StmtKind::TraitDecl { name, .. }
        | StmtKind::EnumDecl { name, .. }
        | StmtKind::PackedClassDecl { name, .. } => {
            out.insert(name.trim_start_matches('\\').to_string());
        }
        StmtKind::NamespaceBlock { body, .. } => {
            for inner in body {
                collect_declared_in_stmt(inner, out);
            }
        }
        _ => {}
    }
}

/// Collect all class reference points in the program, returning (statement index, FQN) pairs.
pub(super) fn collect_reference_points(program: &Program) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut dynamic_class_defaults = HashMap::new();
    for (stmt_idx, stmt) in program.iter().enumerate() {
        let mut refs = HashSet::new();
        collect_refs_with_definite_flow(stmt, &mut dynamic_class_defaults, &mut refs);
        let mut refs: Vec<String> = refs.into_iter().collect();
        refs.sort();
        out.extend(refs.into_iter().map(|fqn| (stmt_idx, fqn)));
    }
    out
}

/// Collects references while preserving sequential assignments through transparent AST wrappers.
fn collect_refs_with_definite_flow(
    stmt: &Stmt,
    defaults: &mut HashMap<DynamicClassTarget, HashSet<String>>,
    names: &mut HashSet<String>,
) {
    match &stmt.kind {
        StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::Synthetic(body) => {
            for inner in body {
                collect_refs_with_definite_flow(inner, defaults, names);
            }
        }
        _ => {
            let local_names = {
                let mut refs = ReferenceSet::new(defaults);
                collect_refs_stmt(stmt, &mut refs);
                refs.names
            };
            names.extend(local_names);
            update_dynamic_class_defaults(stmt, defaults);
        }
    }
}

/// Records top-level definite class-string assignments for later dynamic object construction.
fn update_dynamic_class_defaults(
    stmt: &Stmt,
    defaults: &mut HashMap<DynamicClassTarget, HashSet<String>>,
) {
    let (target, value) = match &stmt.kind {
        StmtKind::Assign { name, value } => (DynamicClassTarget::Variable(name.clone()), value),
        StmtKind::ArrayAssign {
            array,
            index,
            value,
        } => {
            let Some(index) = dynamic_class_index(index) else {
                return;
            };
            (
                DynamicClassTarget::ArrayAccess {
                    array: Box::new(DynamicClassTarget::Variable(array.clone())),
                    index,
                },
                value,
            )
        }
        StmtKind::NestedArrayAssign { target, value } => {
            let Some(target) = dynamic_class_target(target) else {
                return;
            };
            (target, value)
        }
        _ => return,
    };

    let mut candidates = HashSet::new();
    collect_possible_class_strings(value, &mut candidates);
    let preserves_previous = matches!(
        &value.kind,
        ExprKind::NullCoalesce { value: current, .. }
            if dynamic_class_target(current).as_ref() == Some(&target)
    );
    if preserves_previous {
        defaults.entry(target).or_default().extend(candidates);
    } else if candidates.is_empty() {
        defaults.remove(&target);
    } else {
        defaults.insert(target, candidates);
    }
}

/// Collects literal class strings that a deterministic assignment expression can produce.
fn collect_possible_class_strings(expr: &Expr, out: &mut HashSet<String>) {
    match &expr.kind {
        ExprKind::StringLiteral(name) => {
            let name = name.trim_start_matches('\\');
            if !name.is_empty() {
                out.insert(name.to_string());
            }
        }
        ExprKind::NullCoalesce { value, default }
        | ExprKind::ShortTernary { value, default } => {
            collect_possible_class_strings(value, out);
            collect_possible_class_strings(default, out);
        }
        ExprKind::Ternary {
            then_expr,
            else_expr,
            ..
        } => {
            collect_possible_class_strings(then_expr, out);
            collect_possible_class_strings(else_expr, out);
        }
        _ => {}
    }
}

/// Converts a stable variable/literal-array l-value into a hashable flow key.
fn dynamic_class_target(expr: &Expr) -> Option<DynamicClassTarget> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(DynamicClassTarget::Variable(name.clone())),
        ExprKind::ArrayAccess { array, index } => Some(DynamicClassTarget::ArrayAccess {
            array: Box::new(dynamic_class_target(array)?),
            index: dynamic_class_index(index)?,
        }),
        _ => None,
    }
}

/// Converts a literal string or integer array index into a stable flow key.
fn dynamic_class_index(expr: &Expr) -> Option<DynamicClassIndex> {
    match &expr.kind {
        ExprKind::StringLiteral(value) => Some(DynamicClassIndex::String(value.clone())),
        ExprKind::IntLiteral(value) => Some(DynamicClassIndex::Int(*value)),
        _ => None,
    }
}

/// Recurse into a statement to collect class references.
fn collect_refs_stmt(stmt: &Stmt, out: &mut ReferenceSet<'_>) {
    collect_attribute_groups(&stmt.attributes, out);
    match &stmt.kind {
        StmtKind::ClassDecl {
            extends,
            implements,
            trait_uses,
            properties,
            methods,
            constants,
            ..
        } => {
            if let Some(parent) = extends {
                push_name(parent, out);
            }
            for iface in implements {
                push_name(iface, out);
            }
            for tu in trait_uses {
                collect_trait_use(tu, out);
            }
            for prop in properties {
                collect_property(prop, out);
            }
            for method in methods {
                collect_method(method, out);
            }
            for constant in constants {
                collect_class_const(constant, out);
            }
        }
        StmtKind::InterfaceDecl {
            extends,
            methods,
            constants,
            ..
        } => {
            for parent in extends {
                push_name(parent, out);
            }
            for method in methods {
                collect_method(method, out);
            }
            for constant in constants {
                collect_class_const(constant, out);
            }
        }
        StmtKind::TraitDecl {
            trait_uses,
            properties,
            methods,
            constants,
            ..
        } => {
            for tu in trait_uses {
                collect_trait_use(tu, out);
            }
            for prop in properties {
                collect_property(prop, out);
            }
            for method in methods {
                collect_method(method, out);
            }
            for constant in constants {
                collect_class_const(constant, out);
            }
        }
        StmtKind::EnumDecl { cases, .. } => {
            for case in cases {
                collect_attribute_groups(&case.attributes, out);
                if let Some(value) = &case.value {
                    collect_refs_expr(value, out);
                }
            }
        }
        StmtKind::PackedClassDecl { fields, .. } => {
            for field in fields {
                collect_type_expr(&field.type_expr, out);
            }
        }
        StmtKind::FunctionDecl {
            params,
            param_attributes,
            body,
            ..
        } => {
            for groups in param_attributes {
                collect_attribute_groups(groups, out);
            }
            for (_, _, default, _) in params {
                if let Some(d) = default {
                    collect_refs_expr(d, out);
                }
            }
            for s in body {
                collect_refs_stmt(s, out);
            }
        }
        StmtKind::NamespaceBlock { body, .. } => {
            for inner in body {
                collect_refs_stmt(inner, out);
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            for s in then_body {
                collect_refs_stmt(s, out);
            }
            if let Some(body) = else_body {
                for s in body {
                    collect_refs_stmt(s, out);
                }
            }
        }
        StmtKind::Assign { value, .. } => collect_refs_expr(value, out),
        StmtKind::RefAssign { .. } => {}
        StmtKind::ExprStmt(e) => collect_refs_expr(e, out),
        StmtKind::Return(Some(e)) => collect_refs_expr(e, out),
        StmtKind::Echo(e) => collect_refs_expr(e, out),
        StmtKind::Include { path, .. } => collect_refs_expr(path, out),
        StmtKind::IncludeOnceGuard { body, .. } => {
            for s in body {
                collect_refs_stmt(s, out);
            }
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_refs_expr(condition, out);
            for s in then_body {
                collect_refs_stmt(s, out);
            }
            for (cond, body) in elseif_clauses {
                collect_refs_expr(cond, out);
                for s in body {
                    collect_refs_stmt(s, out);
                }
            }
            if let Some(body) = else_body {
                for s in body {
                    collect_refs_stmt(s, out);
                }
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            collect_refs_expr(condition, out);
            for s in body {
                collect_refs_stmt(s, out);
            }
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(s) = init {
                collect_refs_stmt(s, out);
            }
            if let Some(c) = condition {
                collect_refs_expr(c, out);
            }
            if let Some(s) = update {
                collect_refs_stmt(s, out);
            }
            for s in body {
                collect_refs_stmt(s, out);
            }
        }
        StmtKind::Foreach {
            array, body, ..
        } => {
            collect_refs_expr(array, out);
            for s in body {
                collect_refs_stmt(s, out);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            for s in try_body {
                collect_refs_stmt(s, out);
            }
            for catch in catches {
                collect_refs_catch(catch, out);
            }
            if let Some(f) = finally_body {
                for s in f {
                    collect_refs_stmt(s, out);
                }
            }
        }
        StmtKind::Switch { subject, cases, default } => {
            collect_refs_expr(subject, out);
            for (values, body) in cases {
                for v in values {
                    collect_refs_expr(v, out);
                }
                for s in body {
                    collect_refs_stmt(s, out);
                }
            }
            if let Some(default_body) = default {
                for s in default_body {
                    collect_refs_stmt(s, out);
                }
            }
        }
        StmtKind::ConstDecl { value, .. } | StmtKind::StaticVar { init: value, .. } => {
            collect_refs_expr(value, out);
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyRefAssign { object, source: value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            collect_refs_expr(object, out);
            collect_refs_expr(value, out);
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            collect_refs_expr(object, out);
            collect_refs_expr(index, out);
            collect_refs_expr(value, out);
        }
        StmtKind::StaticPropertyAssign {
            receiver, value, ..
        }
        | StmtKind::StaticPropertyArrayPush {
            receiver, value, ..
        } => {
            collect_static_receiver(receiver, out);
            collect_refs_expr(value, out);
        }
        StmtKind::StaticPropertyArrayAssign {
            receiver,
            index,
            value,
            ..
        } => {
            collect_static_receiver(receiver, out);
            collect_refs_expr(index, out);
            collect_refs_expr(value, out);
        }
        StmtKind::StaticPropertyElementRefAssign {
            receiver,
            index,
            source,
            ..
        } => {
            collect_static_receiver(receiver, out);
            collect_refs_expr(index, out);
            collect_refs_expr(source, out);
        }
        StmtKind::ArrayAssign { value, index, .. } => {
            collect_refs_expr(value, out);
            collect_refs_expr(index, out);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            collect_refs_expr(target, out);
            collect_refs_expr(value, out);
        }
        StmtKind::ArrayPush { value, .. } => collect_refs_expr(value, out),
        StmtKind::ListUnpack { value, .. } => collect_refs_expr(value, out),
        StmtKind::TypedAssign { type_expr, value, .. } => {
            collect_type_expr(type_expr, out);
            collect_refs_expr(value, out);
        }
        StmtKind::Throw(e) => collect_refs_expr(e, out),
        StmtKind::Synthetic(stmts) => {
            for s in stmts {
                collect_refs_stmt(s, out);
            }
        }
        _ => {}
    }
}

/// Collect class references from a catch clause.
fn collect_refs_catch(catch: &CatchClause, out: &mut ReferenceSet<'_>) {
    for s in &catch.body {
        collect_refs_stmt(s, out);
    }
}

/// Collect class references from a class method declaration.
fn collect_method(method: &ClassMethod, out: &mut ReferenceSet<'_>) {
    collect_attribute_groups(&method.attributes, out);
    for groups in &method.param_attributes {
        collect_attribute_groups(groups, out);
    }
    for (_, _, default, _) in &method.params {
        if let Some(d) = default {
            collect_refs_expr(d, out);
        }
    }
    for s in &method.body {
        collect_refs_stmt(s, out);
    }
}

/// Collect class references from a class property declaration.
fn collect_property(prop: &ClassProperty, out: &mut ReferenceSet<'_>) {
    collect_attribute_groups(&prop.attributes, out);
    if let Some(d) = &prop.default {
        collect_refs_expr(d, out);
    }
}

/// Collect class references from a class constant declaration.
fn collect_class_const(constant: &ClassConst, out: &mut ReferenceSet<'_>) {
    collect_attribute_groups(&constant.attributes, out);
    collect_refs_expr(&constant.value, out);
}

/// Collects attribute class names and references nested in their argument expressions.
fn collect_attribute_groups(groups: &[AttributeGroup], out: &mut ReferenceSet<'_>) {
    for group in groups {
        for attribute in &group.attributes {
            push_name(&attribute.name, out);
            for argument in &attribute.args {
                collect_refs_expr(argument, out);
            }
        }
    }
}

/// Collect class references from a trait use declaration.
fn collect_trait_use(trait_use: &TraitUse, out: &mut ReferenceSet<'_>) {
    for name in &trait_use.trait_names {
        push_name(name, out);
    }
}

/// Collect class references from a type expression.
fn collect_type_expr(ty: &TypeExpr, out: &mut ReferenceSet<'_>) {
    match ty {
        TypeExpr::Named(name) => push_name(name, out),
        TypeExpr::Array(inner) => collect_type_expr(inner, out),
        TypeExpr::Nullable(inner) => collect_type_expr(inner, out),
        TypeExpr::Union(parts) | TypeExpr::Intersection(parts) => {
            for p in parts {
                collect_type_expr(p, out);
            }
        }
        TypeExpr::Buffer(inner) => collect_type_expr(inner, out),
        TypeExpr::Ptr(Some(name)) => push_name(name, out),
        _ => {}
    }
}

/// Recurse into an expression to collect class references, including compile-time
/// autoload demands from `class_exists`/`interface_exists`/etc. with literal arguments.
fn collect_refs_expr(expr: &Expr, out: &mut ReferenceSet<'_>) {
    match &expr.kind {
        ExprKind::NewObject { class_name, args } => {
            push_name(class_name, out);
            for a in args {
                collect_refs_expr(a, out);
            }
        }
        ExprKind::NewDynamic { name_expr, args } => {
            if let Some(target) = dynamic_class_target(name_expr) {
                if let Some(candidates) = out.dynamic_class_defaults.get(&target) {
                    let candidates = candidates.iter().cloned().collect::<Vec<_>>();
                    for candidate in candidates {
                        out.insert(candidate);
                    }
                }
            }
            collect_refs_expr(name_expr, out);
            for arg in args {
                collect_refs_expr(arg, out);
            }
        }
        ExprKind::NewDynamicObject {
            class_name,
            fallback_class,
            required_parent,
            args,
        } => {
            collect_refs_expr(class_name, out);
            push_name(fallback_class, out);
            push_name(required_parent, out);
            for arg in args {
                collect_refs_expr(arg, out);
            }
        }
        ExprKind::InstanceOf { value, target } => {
            collect_refs_expr(value, out);
            if let crate::parser::ast::InstanceOfTarget::Expr(inner) = target {
                collect_refs_expr(inner, out);
            }
        }
        ExprKind::ClassConstant { .. } => {}
        ExprKind::ScopedConstantAccess { receiver, .. }
        | ExprKind::StaticPropertyAccess { receiver, .. } => {
            collect_static_receiver(receiver, out);
        }
        ExprKind::DynamicScopedConstantAccess { receiver, .. } => {
            collect_refs_expr(receiver, out);
        }
        ExprKind::StaticMethodCall {
            receiver, args, ..
        } => {
            collect_static_receiver(receiver, out);
            for a in args {
                collect_refs_expr(a, out);
            }
        }
        ExprKind::Cast { expr: inner, .. } => collect_refs_expr(inner, out),
        ExprKind::PtrCast { expr: inner, .. } => collect_refs_expr(inner, out),
        ExprKind::BinaryOp { left, right, .. } => {
            collect_refs_expr(left, out);
            collect_refs_expr(right, out);
        }
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner) => collect_refs_expr(inner, out),
        ExprKind::NullCoalesce { value, default }
        | ExprKind::ShortTernary { value, default } => {
            collect_refs_expr(value, out);
            collect_refs_expr(default, out);
        }
        ExprKind::Pipe { value, callable } => {
            collect_refs_expr(value, out);
            collect_refs_expr(callable, out);
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_refs_expr(condition, out);
            collect_refs_expr(then_expr, out);
            collect_refs_expr(else_expr, out);
        }
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            collect_refs_expr(subject, out);
            for (patterns, value) in arms {
                for p in patterns {
                    collect_refs_expr(p, out);
                }
                collect_refs_expr(value, out);
            }
            if let Some(d) = default {
                collect_refs_expr(d, out);
            }
        }
        ExprKind::Assignment { target, value, prelude, .. } => {
            collect_refs_expr(target, out);
            collect_refs_expr(value, out);
            for s in prelude {
                collect_refs_stmt(s, out);
            }
        }
        ExprKind::FunctionCall { name, args } => {
            // Detect compile-time demands for a literal class name. The
            // autoload pass picks these up like any other class reference.
            let canonical = name.as_canonical();
            let trimmed = canonical.trim_start_matches('\\');
            match trimmed {
                "spl_autoload_call" | "spl_autoload" => {
                    push_literal_fqn(args.first(), out);
                }
                "class_exists" | "interface_exists" | "trait_exists" | "enum_exists" => {
                    // The autoload-controlling second arg only triggers when
                    // omitted or when it is a literal truthy value. A dynamic
                    // expression must not be guessed as true at AOT time.
                    let triggers_autoload = match args.get(1).map(|arg| &arg.kind) {
                        None => true,
                        Some(ExprKind::BoolLiteral(b)) => *b,
                        Some(ExprKind::IntLiteral(n)) => *n != 0,
                        Some(_) => false,
                    };
                    if triggers_autoload {
                        push_literal_fqn(args.first(), out);
                    }
                }
                _ => {}
            }
            for a in args {
                collect_refs_expr(a, out);
            }
        }
        ExprKind::ClosureCall { args, .. } => {
            for a in args {
                collect_refs_expr(a, out);
            }
        }
        ExprKind::ExprCall { callee, args } => {
            collect_refs_expr(callee, out);
            for a in args {
                collect_refs_expr(a, out);
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            collect_refs_expr(array, out);
            collect_refs_expr(index, out);
        }
        ExprKind::ArrayLiteral(items) => {
            for i in items {
                collect_refs_expr(i, out);
            }
        }
        ExprKind::ArrayLiteralAssoc(pairs) => {
            for (k, v) in pairs {
                collect_refs_expr(k, out);
                collect_refs_expr(v, out);
            }
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. }
        | ExprKind::ObjectClassName { object } => collect_refs_expr(object, out),
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            collect_refs_expr(object, out);
            for a in args {
                collect_refs_expr(a, out);
            }
        }
        ExprKind::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => {
            collect_refs_expr(object, out);
            collect_refs_expr(method, out);
            for a in args {
                collect_refs_expr(a, out);
            }
        }
        ExprKind::NewScopedObject { receiver, args } => {
            collect_static_receiver(receiver, out);
            for a in args {
                collect_refs_expr(a, out);
            }
        }
        ExprKind::NamedArg { value, .. } => collect_refs_expr(value, out),
        ExprKind::Closure {
            params,
            body,
            ..
        } => {
            for (_, _, default, _) in params {
                if let Some(d) = default {
                    collect_refs_expr(d, out);
                }
            }
            for s in body {
                collect_refs_stmt(s, out);
            }
        }
        ExprKind::FirstClassCallable(target) => collect_callable_target(target, out),
        ExprKind::BufferNew { element_type, len } => {
            collect_type_expr(element_type, out);
            collect_refs_expr(len, out);
        }
        ExprKind::Yield { key, value } => {
            if let Some(k) = key {
                collect_refs_expr(k, out);
            }
            if let Some(v) = value {
                collect_refs_expr(v, out);
            }
        }
        ExprKind::YieldFrom(inner) => collect_refs_expr(inner, out),
        _ => {}
    }
}

/// Collect a class reference from a static receiver (::scope).
fn collect_static_receiver(receiver: &StaticReceiver, out: &mut ReferenceSet<'_>) {
    if let StaticReceiver::Named(name) = receiver {
        push_name(name, out);
    }
}

/// Collect a class reference from a first-class callable target.
fn collect_callable_target(target: &CallableTarget, out: &mut ReferenceSet<'_>) {
    match target {
        CallableTarget::StaticMethod { receiver, .. } => collect_static_receiver(receiver, out),
        CallableTarget::Method { object, .. } => collect_refs_expr(object, out),
        CallableTarget::Function(_) => {}
    }
}

/// Normalize a name to its canonical FQN (strip leading `\`), then insert it into `out` if non-empty.
fn push_name(name: &crate::names::Name, out: &mut ReferenceSet<'_>) {
    let canonical = name.as_canonical();
    let trimmed = canonical.trim_start_matches('\\');
    if !trimmed.is_empty() {
        out.insert(trimmed.to_string());
    }
}

/// Extract a literal string FQN from an expression argument (used for `class_exists` etc.).
/// Inserts the cleaned FQN into `out` if the argument is a string literal.
fn push_literal_fqn(arg: Option<&crate::parser::ast::Expr>, out: &mut ReferenceSet<'_>) {
    let Some(arg) = arg else { return };
    let ExprKind::StringLiteral(name) = &arg.kind else {
        return;
    };
    let cleaned = name.trim_start_matches('\\').to_string();
    if !cleaned.is_empty() {
        out.insert(cleaned);
    }
}
