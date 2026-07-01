//! Purpose:
//! Harvests a structural `Skeleton` from a resolved program AST in a single walk,
//! then folds trait methods onto the classes that `use` them.
//!
//! Called from:
//! - `crate::tree_shake::harvest_skeleton` (re-exported as the module entry point).
//! - Gated behind `--tree-shake` in `crate::pipeline::compile`; a no-op when the flag is off.
//!
//! Key details:
//! - Does NOT resolve type hints — it only records structure, so it never errors on an
//!   absent/optional-dependency super-type (the whole point of the skeleton).
//! - Descends every statement body that can legally contain a declaration (namespace
//!   blocks, control flow, function/method bodies) so conditionally-declared types are
//!   still captured. Over-approximating the class world is sound for later reachability.

use std::collections::{HashMap, HashSet};

use crate::names::Name;
use crate::parser::ast::{ClassMethod, Program, Stmt, StmtKind, TraitUse};

use super::skeleton::{strip_root, ClassKind, ClassSkel, FnSkel, MethodSkel, Skeleton};

/// Harvests the program's structural skeleton: every class-like declaration, its place
/// in the hierarchy, its method-name table, and every free function. Trait methods are
/// folded onto using classes in a second pass so a class reports its trait methods as
/// its own. No type hints are resolved.
pub fn harvest_skeleton(program: &Program) -> Skeleton {
    let mut skeleton = Skeleton::new();
    // Records `class FQN -> trait FQNs it uses`, gathered during the walk and consumed
    // by the trait-folding pass below (the skeleton itself stores no trait-use list).
    let mut trait_deps: HashMap<String, Vec<String>> = HashMap::new();
    harvest_stmts(program, &mut skeleton, &mut trait_deps);
    fold_traits(&mut skeleton, &trait_deps);
    skeleton
}

/// Walks a slice of statements, harvesting declarations from each.
fn harvest_stmts(
    stmts: &[Stmt],
    skeleton: &mut Skeleton,
    trait_deps: &mut HashMap<String, Vec<String>>,
) {
    for stmt in stmts {
        harvest_stmt(stmt, skeleton, trait_deps);
    }
}

/// Harvests declarations from one statement and recurses into any nested statement
/// bodies (namespace blocks, control flow, function/method bodies) that can hold
/// further declarations.
fn harvest_stmt(
    stmt: &Stmt,
    skeleton: &mut Skeleton,
    trait_deps: &mut HashMap<String, Vec<String>>,
) {
    match &stmt.kind {
        StmtKind::ClassDecl {
            name,
            extends,
            implements,
            is_abstract,
            is_final,
            trait_uses,
            methods,
            ..
        } => {
            record_class_like(
                skeleton,
                trait_deps,
                name,
                ClassKind::Class,
                extends.as_ref().map(name_to_key),
                names_to_keys(implements),
                *is_abstract,
                *is_final,
                methods,
                trait_uses,
            );
            recurse_method_bodies(methods, skeleton, trait_deps);
        }
        StmtKind::InterfaceDecl { name, extends, methods, .. } => {
            // Interfaces `extend` other interfaces (possibly several); model those parent
            // interfaces through the `interfaces` list so subtype walks traverse them.
            record_class_like(
                skeleton,
                trait_deps,
                name,
                ClassKind::Interface,
                None,
                names_to_keys(extends),
                false,
                false,
                methods,
                &[],
            );
            recurse_method_bodies(methods, skeleton, trait_deps);
        }
        StmtKind::TraitDecl { name, trait_uses, methods, .. } => {
            record_class_like(
                skeleton,
                trait_deps,
                name,
                ClassKind::Trait,
                None,
                Vec::new(),
                false,
                false,
                methods,
                trait_uses,
            );
            recurse_method_bodies(methods, skeleton, trait_deps);
        }
        StmtKind::EnumDecl { name, implements, methods, .. } => {
            // Enums cannot be `new`-ed and have no parent; they may implement interfaces.
            record_class_like(
                skeleton,
                trait_deps,
                name,
                ClassKind::Enum,
                None,
                names_to_keys(implements),
                false,
                false,
                methods,
                &[],
            );
            recurse_method_bodies(methods, skeleton, trait_deps);
        }
        StmtKind::FunctionDecl { name, body, .. } => {
            let key = strip_root(name).to_string();
            skeleton.functions.insert(key.clone(), FnSkel { fqn: key });
            harvest_stmts(body, skeleton, trait_deps);
        }
        // Bodies that can legally wrap further declarations: recurse into all of them.
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::Synthetic(body)
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::Foreach { body, .. } => harvest_stmts(body, skeleton, trait_deps),
        StmtKind::If { then_body, elseif_clauses, else_body, .. } => {
            harvest_stmts(then_body, skeleton, trait_deps);
            for (_, body) in elseif_clauses {
                harvest_stmts(body, skeleton, trait_deps);
            }
            if let Some(body) = else_body {
                harvest_stmts(body, skeleton, trait_deps);
            }
        }
        StmtKind::For { init, update, body, .. } => {
            if let Some(init) = init {
                harvest_stmt(init, skeleton, trait_deps);
            }
            if let Some(update) = update {
                harvest_stmt(update, skeleton, trait_deps);
            }
            harvest_stmts(body, skeleton, trait_deps);
        }
        StmtKind::Switch { cases, default, .. } => {
            for (_, body) in cases {
                harvest_stmts(body, skeleton, trait_deps);
            }
            if let Some(body) = default {
                harvest_stmts(body, skeleton, trait_deps);
            }
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            harvest_stmts(try_body, skeleton, trait_deps);
            for catch in catches {
                harvest_stmts(&catch.body, skeleton, trait_deps);
            }
            if let Some(body) = finally_body {
                harvest_stmts(body, skeleton, trait_deps);
            }
        }
        // Remaining statement kinds carry no nested declaration-bearing body.
        _ => {}
    }
}

/// Recurses into each method's body so a class/trait/enum declared inside a method is
/// still harvested into the global class world.
fn recurse_method_bodies(
    methods: &[ClassMethod],
    skeleton: &mut Skeleton,
    trait_deps: &mut HashMap<String, Vec<String>>,
) {
    for method in methods {
        harvest_stmts(&method.body, skeleton, trait_deps);
    }
}

/// Inserts one class-like record into the skeleton, building its lowercase-keyed method
/// table and registering its trait uses for the later folding pass.
#[allow(clippy::too_many_arguments)]
fn record_class_like(
    skeleton: &mut Skeleton,
    trait_deps: &mut HashMap<String, Vec<String>>,
    name: &str,
    kind: ClassKind,
    parent: Option<String>,
    interfaces: Vec<String>,
    is_abstract: bool,
    is_final: bool,
    methods: &[ClassMethod],
    trait_uses: &[TraitUse],
) {
    let fqn = strip_root(name).to_string();
    let mut method_table: HashMap<String, MethodSkel> = HashMap::new();
    for method in methods {
        method_table.insert(method.name.to_ascii_lowercase(), MethodSkel::from_ast(method));
    }
    let deps = collect_trait_names(trait_uses);
    if !deps.is_empty() {
        trait_deps.insert(fqn.clone(), deps);
    }
    skeleton.classes.insert(
        fqn.clone(),
        ClassSkel {
            fqn,
            parent,
            interfaces,
            is_abstract,
            is_final,
            kind,
            methods: method_table,
        },
    );
}

/// Flattens the trait names referenced by a class's `use` clauses into canonical keys.
/// Trait adaptations (aliases/`insteadof`) are ignored in Stage 1: folding every trait
/// method name is a sound over-approximation for reachability.
fn collect_trait_names(trait_uses: &[TraitUse]) -> Vec<String> {
    let mut names = Vec::new();
    for trait_use in trait_uses {
        for trait_name in &trait_use.trait_names {
            names.push(name_to_key(trait_name));
        }
    }
    names
}

/// Folds trait methods onto every class-like that uses a trait, transitively through
/// traits that themselves use traits. Class-declared methods and earlier (more direct)
/// trait methods win, matching PHP precedence; adaptations are not applied.
fn fold_traits(skeleton: &mut Skeleton, trait_deps: &HashMap<String, Vec<String>>) {
    // Collect additions first (read-only over the skeleton) to avoid aliasing a mutable
    // borrow with the recursive reads.
    let mut additions: Vec<(String, Vec<(String, MethodSkel)>)> = Vec::new();
    for (class_fqn, deps) in trait_deps {
        let mut collected: Vec<(String, MethodSkel)> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        for dep in deps {
            collect_trait_methods(dep, skeleton, trait_deps, &mut visited, &mut collected);
        }
        additions.push((class_fqn.clone(), collected));
    }
    for (class_fqn, methods) in additions {
        if let Some(class) = skeleton.classes.get_mut(&class_fqn) {
            for (lower_name, method) in methods {
                // `or_insert` keeps a class-declared method or an earlier-listed (more
                // direct) trait method over a later/nested one.
                class.methods.entry(lower_name).or_insert(method);
            }
        }
    }
}

/// Recursively collects a trait's own methods followed by the methods of the traits it
/// uses, guarding against cycles with `visited`. Direct methods are appended before
/// nested ones so the folding pass can resolve precedence by first-seen order.
fn collect_trait_methods(
    trait_fqn: &str,
    skeleton: &Skeleton,
    trait_deps: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    out: &mut Vec<(String, MethodSkel)>,
) {
    let key = strip_root(trait_fqn).to_string();
    if !visited.insert(key.clone()) {
        return;
    }
    if let Some(trait_skel) = skeleton.classes.get(&key) {
        for (lower_name, method) in &trait_skel.methods {
            out.push((lower_name.clone(), method.clone()));
        }
    }
    if let Some(deps) = trait_deps.get(&key) {
        for dep in deps {
            collect_trait_methods(dep, skeleton, trait_deps, visited, out);
        }
    }
}

/// Converts a resolved `Name` into a canonical skeleton key (no leading `\`).
fn name_to_key(name: &Name) -> String {
    strip_root(name.as_str()).to_string()
}

/// Converts a slice of resolved `Name`s into canonical skeleton keys.
fn names_to_keys(names: &[Name]) -> Vec<String> {
    names.iter().map(name_to_key).collect()
}
