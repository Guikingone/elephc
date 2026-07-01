//! Purpose:
//! Builds a body-level index over the resolved program for the Stage-2 reachability
//! fixpoint: maps every free function and class method to the AST it must scan, plus
//! per-class typed-property tables used for lightweight receiver typing.
//!
//! Called from:
//! - `crate::tree_shake::reach::compute_reachable`, which walks these bodies to emit edges.
//!
//! Key details:
//! - The `Skeleton` records method *existence* and hierarchy but not bodies; this index holds
//!   the actual `&ClassMethod`/statement slices so the fixpoint can follow calls out of a body.
//! - Trait method bodies are folded onto every using class (mirroring the skeleton's fold) so a
//!   `(class, method)` lookup resolves the trait's body when the class pulls it in via `use`.
//! - Every reference borrows the program with lifetime `'a`; nothing is cloned out of the AST.

use std::collections::{HashMap, HashSet};

use crate::names::Name;
use crate::parser::ast::{
    ClassMethod, ClassProperty, Expr, Program, Stmt, StmtKind, TraitUse, TypeExpr,
};

use super::skeleton::strip_root;

/// One free function's scannable AST: its parameter list (for receiver typing) and body.
pub(super) struct FnEntry<'a> {
    /// The parameter tuples `(name, type, default, by_ref)` exactly as parsed.
    pub params: &'a [(String, Option<TypeExpr>, Option<Expr>, bool)],
    /// The function body statements to scan when the function becomes reachable.
    pub body: &'a [Stmt],
}

/// One class-like's scannable members: its method bodies (own + folded trait methods),
/// its typed-property table, and its parent link for property-type inheritance walks.
pub(super) struct ClassEntry<'a> {
    /// Lowercase method name → the declaring `&ClassMethod` (own method, or a folded trait method).
    pub methods: HashMap<String, &'a ClassMethod>,
    /// Property name → its declared type expression (only properties that carry a type hint).
    pub properties: HashMap<String, &'a TypeExpr>,
    /// Canonical FQN of the parent class, for walking inherited property types.
    pub parent: Option<String>,
}

/// Body-level index over a whole program, keyed by canonical function/class FQNs.
pub(super) struct AstIndex<'a> {
    /// Free functions by canonical FQN.
    pub functions: HashMap<String, FnEntry<'a>>,
    /// Class-likes by canonical FQN.
    pub classes: HashMap<String, ClassEntry<'a>>,
}

/// Builds the body-level index by walking the program once, then folds trait method bodies
/// onto every using class so a class exposes its trait methods' bodies as its own.
pub(super) fn build_index(program: &Program) -> AstIndex<'_> {
    let mut index = AstIndex { functions: HashMap::new(), classes: HashMap::new() };
    // `class FQN -> trait FQNs used`, gathered during the walk and consumed by the fold below.
    let mut trait_deps: HashMap<String, Vec<String>> = HashMap::new();
    index_stmts(program, &mut index, &mut trait_deps);
    fold_trait_bodies(&mut index, &trait_deps);
    index
}

/// Walks a statement slice, indexing each statement's declarations and recursing into any
/// nested declaration-bearing body (mirrors the skeleton harvest so nothing is missed).
fn index_stmts<'a>(
    stmts: &'a [Stmt],
    index: &mut AstIndex<'a>,
    trait_deps: &mut HashMap<String, Vec<String>>,
) {
    for stmt in stmts {
        index_stmt(stmt, index, trait_deps);
    }
}

/// Indexes one statement: records function/class members and recurses into nested bodies that
/// can legally hold further declarations (namespace blocks, control flow, function/method bodies).
fn index_stmt<'a>(
    stmt: &'a Stmt,
    index: &mut AstIndex<'a>,
    trait_deps: &mut HashMap<String, Vec<String>>,
) {
    match &stmt.kind {
        StmtKind::ClassDecl { name, extends, trait_uses, properties, methods, .. } => {
            record_class(index, trait_deps, name, extends.as_ref(), properties, methods, trait_uses);
            recurse_methods(methods, index, trait_deps);
        }
        StmtKind::TraitDecl { name, trait_uses, properties, methods, .. } => {
            record_class(index, trait_deps, name, None, properties, methods, trait_uses);
            recurse_methods(methods, index, trait_deps);
        }
        StmtKind::InterfaceDecl { name, properties, methods, .. } => {
            record_class(index, trait_deps, name, None, properties, methods, &[]);
            recurse_methods(methods, index, trait_deps);
        }
        StmtKind::EnumDecl { name, methods, .. } => {
            record_class(index, trait_deps, name, None, &[], methods, &[]);
            recurse_methods(methods, index, trait_deps);
        }
        StmtKind::FunctionDecl { name, params, body, .. } => {
            let key = strip_root(name).to_string();
            index.functions.insert(key, FnEntry { params, body });
            index_stmts(body, index, trait_deps);
        }
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::Foreach { body, .. } => index_stmts(body, index, trait_deps),
        StmtKind::If { then_body, elseif_clauses, else_body, .. } => {
            index_stmts(then_body, index, trait_deps);
            for (_, body) in elseif_clauses {
                index_stmts(body, index, trait_deps);
            }
            if let Some(body) = else_body {
                index_stmts(body, index, trait_deps);
            }
        }
        StmtKind::For { init, update, body, .. } => {
            if let Some(init) = init {
                index_stmt(init, index, trait_deps);
            }
            if let Some(update) = update {
                index_stmt(update, index, trait_deps);
            }
            index_stmts(body, index, trait_deps);
        }
        StmtKind::Switch { cases, default, .. } => {
            for (_, body) in cases {
                index_stmts(body, index, trait_deps);
            }
            if let Some(body) = default {
                index_stmts(body, index, trait_deps);
            }
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            index_stmts(try_body, index, trait_deps);
            for catch in catches {
                index_stmts(&catch.body, index, trait_deps);
            }
            if let Some(body) = finally_body {
                index_stmts(body, index, trait_deps);
            }
        }
        _ => {}
    }
}

/// Recurses into each method body so a function/class declared inside a method is still indexed.
fn recurse_methods<'a>(
    methods: &'a [ClassMethod],
    index: &mut AstIndex<'a>,
    trait_deps: &mut HashMap<String, Vec<String>>,
) {
    for method in methods {
        index_stmts(&method.body, index, trait_deps);
    }
}

/// Records one class-like's own method-body table and typed-property table, keyed by canonical
/// FQN, and registers its trait uses (deferred to the body-folding pass).
#[allow(clippy::too_many_arguments)]
fn record_class<'a>(
    index: &mut AstIndex<'a>,
    trait_deps: &mut HashMap<String, Vec<String>>,
    name: &str,
    extends: Option<&Name>,
    properties: &'a [ClassProperty],
    methods: &'a [ClassMethod],
    trait_uses: &'a [TraitUse],
) {
    let fqn = strip_root(name).to_string();
    let mut method_table: HashMap<String, &'a ClassMethod> = HashMap::new();
    for method in methods {
        method_table.insert(method.name.to_ascii_lowercase(), method);
    }
    let mut property_table: HashMap<String, &'a TypeExpr> = HashMap::new();
    for property in properties {
        if let Some(ty) = &property.type_expr {
            property_table.insert(property.name.clone(), ty);
        }
    }
    let deps = collect_trait_names(trait_uses);
    if !deps.is_empty() {
        trait_deps.insert(fqn.clone(), deps);
    }
    index.classes.insert(
        fqn.clone(),
        ClassEntry {
            methods: method_table,
            properties: property_table,
            parent: extends.map(|name| strip_root(name.as_str()).to_string()),
        },
    );
}

/// Flattens the trait names a class references through its `use` clauses into canonical keys.
fn collect_trait_names(trait_uses: &[TraitUse]) -> Vec<String> {
    let mut names = Vec::new();
    for trait_use in trait_uses {
        for trait_name in &trait_use.trait_names {
            names.push(strip_root(trait_name.as_str()).to_string());
        }
    }
    names
}

/// Folds trait method bodies onto every using class (transitively through nested trait `use`),
/// so a `(class, method)` lookup resolves the trait's body. Class-declared methods and more
/// direct trait methods win (`or_insert`), matching PHP precedence.
fn fold_trait_bodies(index: &mut AstIndex<'_>, trait_deps: &HashMap<String, Vec<String>>) {
    let mut additions: Vec<(String, Vec<(String, &ClassMethod)>)> = Vec::new();
    for (class_fqn, deps) in trait_deps {
        let mut collected: Vec<(String, &ClassMethod)> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        for dep in deps {
            collect_trait_methods(dep, index, trait_deps, &mut visited, &mut collected);
        }
        additions.push((class_fqn.clone(), collected));
    }
    for (class_fqn, methods) in additions {
        if let Some(class) = index.classes.get_mut(&class_fqn) {
            for (lower_name, method) in methods {
                class.methods.entry(lower_name).or_insert(method);
            }
        }
    }
}

/// Recursively collects a trait's own method bodies then those of the traits it uses, guarding
/// against cycles. Direct methods are appended before nested ones so precedence is first-seen.
fn collect_trait_methods<'a>(
    trait_fqn: &str,
    index: &AstIndex<'a>,
    trait_deps: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    out: &mut Vec<(String, &'a ClassMethod)>,
) {
    let key = strip_root(trait_fqn).to_string();
    if !visited.insert(key.clone()) {
        return;
    }
    if let Some(trait_entry) = index.classes.get(&key) {
        for (lower_name, method) in &trait_entry.methods {
            out.push((lower_name.clone(), method));
        }
    }
    if let Some(deps) = trait_deps.get(&key) {
        for dep in deps {
            collect_trait_methods(dep, index, trait_deps, visited, out);
        }
    }
}

impl<'a> AstIndex<'a> {
    /// Resolves the declared type of `property` on `class`, walking the parent chain until the
    /// property is found or the chain leaves the indexed world. Returns `None` for an untyped or
    /// absent property (the caller then treats the receiver as unknown).
    pub(super) fn property_type(&self, class: &str, property: &str) -> Option<&'a TypeExpr> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut current = Some(strip_root(class).to_string());
        while let Some(cur) = current {
            if !visited.insert(cur.clone()) {
                return None;
            }
            let entry = self.classes.get(&cur)?;
            if let Some(ty) = entry.properties.get(property) {
                return Some(ty);
            }
            current = entry.parent.clone();
        }
        None
    }
}
