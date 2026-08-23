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
    ExprKind, Program, StaticReceiver, Stmt, StmtKind, TraitAdaptation, TraitUse, TypeExpr,
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

/// Per-statement reference collection state, including possible class-string assignments.
struct ReferenceSet {
    names: HashSet<String>,
    dynamic_class_defaults: HashMap<DynamicClassTarget, HashSet<String>>,
}

impl ReferenceSet {
    /// Creates an empty reference set seeded by the current class-string assignment facts.
    fn new(dynamic_class_defaults: &HashMap<DynamicClassTarget, HashSet<String>>) -> Self {
        Self {
            names: HashSet::new(),
            dynamic_class_defaults: dynamic_class_defaults.clone(),
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
    // Autoload dependency insertion can place a parent declaration before the subclass that
    // supplies a class-string option consumed by the parent. Seed the flow facts from the whole
    // program so discovery does not depend on declaration order; the ordered pass below still
    // applies definite top-level overwrites at their observable reference points.
    let mut seed = ReferenceSet::new(&HashMap::new());
    for stmt in program {
        collect_refs_stmt(stmt, &mut seed);
    }
    let mut dynamic_class_defaults = seed.dynamic_class_defaults;
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
            let mut refs = ReferenceSet::new(defaults);
            collect_refs_stmt(stmt, &mut refs);
            let local_names = refs.names;
            *defaults = refs.dynamic_class_defaults;
            names.extend(local_names);
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

    update_dynamic_class_target_default(target, value, defaults);
}

/// Records possible class strings assigned to one stable dynamic class target.
fn update_dynamic_class_target_default(
    target: DynamicClassTarget,
    value: &Expr,
    defaults: &mut HashMap<DynamicClassTarget, HashSet<String>>,
) {
    if let Some(value) = literal_eval_return_value(value, defaults) {
        update_dynamic_class_target_default(target, &value, defaults);
        return;
    }
    if update_dynamic_class_array_defaults(&target, value, defaults) {
        return;
    }
    let mut candidates = HashSet::new();
    collect_possible_class_strings(value, defaults, &mut candidates);
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

/// Records per-key class-string defaults when a stable target receives a literal array.
///
/// A class name commonly crosses a dynamic boundary as `$classes[0]`. Recording only the
/// outer `$classes` local would lose that fact before `new $classes[0]` or
/// `class_exists($classes[0])` reaches the autoload walk.
fn update_dynamic_class_array_defaults(
    target: &DynamicClassTarget,
    value: &Expr,
    defaults: &mut HashMap<DynamicClassTarget, HashSet<String>>,
) -> bool {
    let entries = match &value.kind {
        ExprKind::ArrayLiteral(items) => items
            .iter()
            .enumerate()
            .map(|(index, value)| (DynamicClassIndex::Int(index as i64), value))
            .collect::<Vec<_>>(),
        ExprKind::ArrayLiteralAssoc(items) => items
            .iter()
            .filter_map(|(key, value)| Some((dynamic_class_index(key)?, value)))
            .collect::<Vec<_>>(),
        _ => return false,
    };
    defaults.remove(target);
    for (index, value) in entries {
        update_dynamic_class_target_default(
            DynamicClassTarget::ArrayAccess {
                array: Box::new(target.clone()),
                index,
            },
            value,
            defaults,
        );
    }
    true
}

/// Extracts the direct return expression from a statically known `eval` source.
///
/// This intentionally recognizes only an eval fragment consisting of one value return. More
/// involved runtime code remains opaque, while this shape safely preserves the same class-string
/// flow facts as a literal RHS without executing source during compilation.
fn literal_eval_return_value(
    expr: &Expr,
    defaults: &HashMap<DynamicClassTarget, HashSet<String>>,
) -> Option<Expr> {
    let ExprKind::FunctionCall { name, args } = &expr.kind else {
        return None;
    };
    if name.as_canonical().trim_start_matches('\\') != "eval" || args.len() != 1 {
        return None;
    }
    let sources = literal_eval_source_candidates(&args[0], defaults);
    for source in sources {
        let source = format!("<?php\n{source}");
        let Ok(tokens) = crate::lexer::tokenize(&source) else {
            continue;
        };
        let Ok(program) = crate::parser::parse(&tokens) else {
            continue;
        };
        let [Stmt {
            kind: StmtKind::Return(Some(value)),
            ..
        }] = program.as_slice()
        else {
            continue;
        };
        return Some(value.clone());
    }
    None
}

/// Returns literal eval-source candidates flowing through a stable class-string local.
fn literal_eval_source_candidates(
    expr: &Expr,
    defaults: &HashMap<DynamicClassTarget, HashSet<String>>,
) -> Vec<String> {
    match &expr.kind {
        ExprKind::StringLiteral(source) => vec![source.clone()],
        ExprKind::Variable(_) | ExprKind::ArrayAccess { .. } => dynamic_class_target(expr)
            .and_then(|target| defaults.get(&target))
            .map(|sources| sources.iter().cloned().collect())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// Collects class strings that a deterministic expression can produce from literals or tracked
/// stable l-values.
fn collect_possible_class_strings(
    expr: &Expr,
    defaults: &HashMap<DynamicClassTarget, HashSet<String>>,
    out: &mut HashSet<String>,
) {
    match &expr.kind {
        ExprKind::StringLiteral(name) => {
            let name = name.trim_start_matches('\\');
            if !name.is_empty() {
                out.insert(name.to_string());
            }
        }
        ExprKind::ClassConstant {
            receiver: StaticReceiver::Named(name),
        } => {
            let canonical = name.as_canonical();
            let canonical = canonical.trim_start_matches('\\');
            if !canonical.is_empty() {
                out.insert(canonical.to_string());
            }
        }
        ExprKind::Variable(_) | ExprKind::ArrayAccess { .. } => {
            if let Some(target) = dynamic_class_target(expr) {
                if let Some(candidates) = defaults.get(&target) {
                    out.extend(candidates.iter().cloned());
                }
            }
        }
        ExprKind::NullCoalesce { value, default }
        | ExprKind::ShortTernary { value, default } => {
            collect_possible_class_strings(value, defaults, out);
            collect_possible_class_strings(default, defaults, out);
        }
        ExprKind::Ternary {
            then_expr,
            else_expr,
            ..
        } => {
            collect_possible_class_strings(then_expr, defaults, out);
            collect_possible_class_strings(else_expr, defaults, out);
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
fn collect_refs_stmt(stmt: &Stmt, out: &mut ReferenceSet) {
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
            properties,
            methods,
            constants,
            ..
        } => {
            for parent in extends {
                push_name(parent, out);
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
        StmtKind::EnumDecl {
            backing_type: _,
            cases,
            implements,
            trait_uses,
            methods,
            constants,
            ..
        } => {
            for interface in implements {
                push_name(interface, out);
            }
            for trait_use in trait_uses {
                collect_trait_use(trait_use, out);
            }
            for case in cases {
                collect_attribute_groups(&case.attributes, out);
                if let Some(value) = &case.value {
                    collect_refs_expr(value, out);
                }
            }
            for method in methods {
                collect_method(method, out);
            }
            for constant in constants {
                collect_class_const(constant, out);
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
            variadic_type,
            return_type,
            body,
            ..
        } => {
            for groups in param_attributes {
                collect_attribute_groups(groups, out);
            }
            collect_callable_signature(params, variadic_type.as_ref(), return_type.as_ref(), out);
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
        StmtKind::TypedAssign { value, .. } => collect_refs_expr(value, out),
        StmtKind::Throw(e) => collect_refs_expr(e, out),
        StmtKind::Synthetic(stmts) => {
            for s in stmts {
                collect_refs_stmt(s, out);
            }
        }
        _ => {}
    }
    update_dynamic_class_defaults(stmt, &mut out.dynamic_class_defaults);
}

/// Collect class references from a catch clause.
fn collect_refs_catch(catch: &CatchClause, out: &mut ReferenceSet) {
    for exception_type in &catch.exception_types {
        push_name(exception_type, out);
    }
    for s in &catch.body {
        collect_refs_stmt(s, out);
    }
}

/// Collect class references from a class method declaration.
fn collect_method(method: &ClassMethod, out: &mut ReferenceSet) {
    collect_attribute_groups(&method.attributes, out);
    for groups in &method.param_attributes {
        collect_attribute_groups(groups, out);
    }
    collect_callable_signature(
        &method.params,
        method.variadic_type.as_ref(),
        method.return_type.as_ref(),
        out,
    );
    for s in &method.body {
        collect_refs_stmt(s, out);
    }
}

/// Collect class references from a class property declaration.
fn collect_property(prop: &ClassProperty, out: &mut ReferenceSet) {
    collect_attribute_groups(&prop.attributes, out);
    if let Some(d) = &prop.default {
        collect_refs_expr(d, out);
    }
}

/// Collect class references from a class constant declaration.
fn collect_class_const(constant: &ClassConst, out: &mut ReferenceSet) {
    collect_attribute_groups(&constant.attributes, out);
    collect_refs_expr(&constant.value, out);
}

/// Collects parameter default expressions without treating declaration type names as autoload demands.
///
/// PHP defers resolving named types in function-like signatures until an operation needs their
/// metadata. Loading a type merely because it occurs in a callback annotation would execute
/// optional autoload sources and change the program's observable top-level behavior.
fn collect_callable_signature(
    params: &[(String, Option<TypeExpr>, Option<Expr>, bool)],
    _variadic_type: Option<&TypeExpr>,
    _return_type: Option<&TypeExpr>,
    out: &mut ReferenceSet,
) {
    for (_, _, default, _) in params {
        if let Some(default) = default {
            collect_refs_expr(default, out);
        }
    }
}

/// Collects attribute class names and references nested in their argument expressions.
fn collect_attribute_groups(groups: &[AttributeGroup], out: &mut ReferenceSet) {
    for group in groups {
        for attribute in &group.attributes {
            push_name(&attribute.name, out);
            for argument in &attribute.args {
                collect_refs_expr(argument, out);
                collect_attribute_class_constant_refs(argument, out);
            }
        }
    }
}

/// Collects class-string constants retained as attribute metadata dependencies.
///
/// Ordinary `Type::class` expressions do not autoload the named type in PHP, so the general
/// expression walker deliberately ignores them. Attribute metadata is different: reflection may
/// later instantiate or inspect the recorded class string, and AOT must have discovered an
/// existing declaration before reachability can keep it. Nested constant-expression arrays and
/// named arguments are traversed recursively.
fn collect_attribute_class_constant_refs(expr: &Expr, out: &mut ReferenceSet) {
    match &expr.kind {
        ExprKind::ClassConstant {
            receiver: StaticReceiver::Named(name),
        } => push_name(name, out),
        ExprKind::ArrayLiteral(items) => {
            for item in items {
                collect_attribute_class_constant_refs(item, out);
            }
        }
        ExprKind::ArrayLiteralAssoc(items) => {
            for (key, value) in items {
                collect_attribute_class_constant_refs(key, out);
                collect_attribute_class_constant_refs(value, out);
            }
        }
        ExprKind::NamedArg { value, .. }
        | ExprKind::Negate(value)
        | ExprKind::Not(value)
        | ExprKind::BitNot(value)
        | ExprKind::ErrorSuppress(value)
        | ExprKind::Spread(value)
        | ExprKind::Cast { expr: value, .. } => {
            collect_attribute_class_constant_refs(value, out);
        }
        ExprKind::BinaryOp { left, right, .. } => {
            collect_attribute_class_constant_refs(left, out);
            collect_attribute_class_constant_refs(right, out);
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_attribute_class_constant_refs(condition, out);
            collect_attribute_class_constant_refs(then_expr, out);
            collect_attribute_class_constant_refs(else_expr, out);
        }
        ExprKind::ShortTernary { value, default }
        | ExprKind::NullCoalesce { value, default } => {
            collect_attribute_class_constant_refs(value, out);
            collect_attribute_class_constant_refs(default, out);
        }
        _ => {}
    }
}

/// Collect class references from a trait use declaration.
fn collect_trait_use(trait_use: &TraitUse, out: &mut ReferenceSet) {
    for name in &trait_use.trait_names {
        push_name(name, out);
    }
    for adaptation in &trait_use.adaptations {
        match adaptation {
            TraitAdaptation::Alias { trait_name, .. } => {
                if let Some(name) = trait_name {
                    push_name(name, out);
                }
            }
            TraitAdaptation::InsteadOf {
                trait_name,
                instead_of,
                ..
            } => {
                if let Some(name) = trait_name {
                    push_name(name, out);
                }
                for name in instead_of {
                    push_name(name, out);
                }
            }
        }
    }
}

/// Collect class references from a type expression.
fn collect_type_expr(ty: &TypeExpr, out: &mut ReferenceSet) {
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
fn collect_refs_expr(expr: &Expr, out: &mut ReferenceSet) {
    match &expr.kind {
        ExprKind::NewObject { class_name, args } => {
            push_name(class_name, out);
            for a in args {
                collect_refs_expr(a, out);
            }
        }
        ExprKind::NewDynamic { name_expr, args } => {
            collect_dynamic_class_target_candidates(name_expr, out);
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
            if let Some(target) = dynamic_class_target(target) {
                update_dynamic_class_target_default(
                    target,
                    value,
                    &mut out.dynamic_class_defaults,
                );
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
                        if args
                            .first()
                            .is_some_and(|arg| !matches!(arg.kind, ExprKind::StringLiteral(_)))
                        {
                            collect_dynamic_class_target_candidates(
                                args.first().expect("checked above"),
                                out,
                            );
                        }
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
            if items.len() == 2 {
                collect_dynamic_class_target_candidates(&items[0], out);
            }
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
            variadic_type,
            return_type,
            body,
            ..
        } => {
            collect_callable_signature(params, variadic_type.as_ref(), return_type.as_ref(), out);
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

/// Adds known class-string candidates used as a dynamic class target in the current expression.
fn collect_dynamic_class_target_candidates(expr: &Expr, out: &mut ReferenceSet) {
    let Some(target) = dynamic_class_target(expr) else {
        return;
    };
    let Some(candidates) = out.dynamic_class_defaults.get(&target) else {
        return;
    };
    let candidates = candidates.iter().cloned().collect::<Vec<_>>();
    for candidate in candidates {
        out.insert(candidate);
    }
}

/// Collect a class reference from a static receiver (::scope).
fn collect_static_receiver(receiver: &StaticReceiver, out: &mut ReferenceSet) {
    if let StaticReceiver::Named(name) = receiver {
        push_name(name, out);
    }
}

/// Collect a class reference from a first-class callable target.
fn collect_callable_target(target: &CallableTarget, out: &mut ReferenceSet) {
    match target {
        CallableTarget::StaticMethod { receiver, .. } => collect_static_receiver(receiver, out),
        CallableTarget::Method { object, .. } => collect_refs_expr(object, out),
        CallableTarget::Function(_) => {}
    }
}

/// Normalize a name to its canonical FQN (strip leading `\`), then insert it into `out` if non-empty.
fn push_name(name: &crate::names::Name, out: &mut ReferenceSet) {
    let canonical = name.as_canonical();
    let trimmed = canonical.trim_start_matches('\\');
    if !trimmed.is_empty() {
        out.insert(trimmed.to_string());
    }
}

/// Extract a literal string FQN from an expression argument (used for `class_exists` etc.).
/// Inserts the cleaned FQN into `out` if the argument is a string literal.
fn push_literal_fqn(arg: Option<&crate::parser::ast::Expr>, out: &mut ReferenceSet) {
    let Some(arg) = arg else { return };
    let ExprKind::StringLiteral(name) = &arg.kind else {
        return;
    };
    let cleaned = name.trim_start_matches('\\').to_string();
    if !cleaned.is_empty() {
        out.insert(cleaned);
    }
}
