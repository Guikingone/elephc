//! Purpose:
//! Resolves class-like constant references used as default values (class property defaults and
//! class method/constructor parameter defaults) into the referenced constant's literal value,
//! before class schema metadata is built.
//!
//! Called from:
//! - `crate::types::checker::driver::init` (after `class_map` is fully populated).
//!
//! Key details:
//! - Runs on the complete class and interface maps, so it is order-independent: a default in a
//!   class declared before the class-like symbol it references still resolves.
//! - Rewrites the default expression in place to the resolved scalar literal, so both syntactic
//!   type inference and codegen see a literal instead of a `ScopedConstantAccess`. For property
//!   defaults this also satisfies the codegen property-default emitter, which only accepts literal
//!   forms (`literal_default_value`).
//! - Only scalar literal results are substituted; non-literal constant values (arrays, computed
//!   expressions, enum cases, unresolvable references) are left untouched so existing behavior is
//!   preserved and codegen never receives an unsupported default form.

use std::collections::{HashMap, HashSet};

use crate::parser::ast::{Expr, ExprKind, StaticReceiver};
use crate::types::checker::builtin_types::InterfaceDeclInfo;
use crate::types::traits::FlattenedClass;

/// Maximum number of constant indirections to follow when resolving a default
/// (`A::X` where `const X = B::Y` where `const Y = 1`). Guards against cyclic constant
/// definitions producing unbounded recursion.
const MAX_CONST_RESOLUTION_DEPTH: usize = 32;

/// A class-like symbol's constant values and parent links, used for order-independent resolution.
struct ClassConsts {
    /// Constant name (case-sensitive, matching PHP) → declared value expression.
    constants: HashMap<String, Expr>,
    /// Fully-qualified parent class or parent interface names, for walking inheritance.
    parents: Vec<String>,
}

/// Rewrites every class property default and class method/constructor parameter default that is a
/// class-constant reference (`A::X`, `self::X`, `parent::X`, `static::X`) into the referenced
/// constant's scalar literal value, when that value resolves to a literal.
///
/// Operates on the complete class and interface maps so resolution is declaration-order independent.
/// Non-resolvable references and non-literal constant values are left unchanged.
pub(crate) fn resolve_const_default_references(
    class_map: &mut HashMap<String, FlattenedClass>,
    interface_map: &HashMap<String, InterfaceDeclInfo>,
) {
    let const_table = build_const_table(class_map, interface_map);
    for class in class_map.values_mut() {
        let class_name = class.name.clone();
        // Instance and static property defaults.
        for prop in &mut class.properties {
            rewrite_default(&mut prop.default, &class_name, &const_table);
        }
        // Method, static-method, and constructor (incl. promoted) parameter defaults.
        for method in &mut class.methods {
            for (_, _, default, _) in &mut method.params {
                rewrite_default(default, &class_name, &const_table);
            }
        }
    }
}

/// Rewrites a single optional default slot in place when it resolves to a scalar literal.
fn rewrite_default(
    slot: &mut Option<Expr>,
    current_class: &str,
    table: &HashMap<String, ClassConsts>,
) {
    let Some(default) = slot.as_ref() else {
        return;
    };
    if let Some(resolved) = resolve_const_default(default, current_class, table) {
        *slot = Some(resolved);
    }
}

/// Builds the order-independent constant lookup table from every class and interface declaration.
fn build_const_table(
    class_map: &HashMap<String, FlattenedClass>,
    interface_map: &HashMap<String, InterfaceDeclInfo>,
) -> HashMap<String, ClassConsts> {
    let mut table: HashMap<String, ClassConsts> = class_map
        .iter()
        .map(|(name, class)| {
            let constants = class
                .constants
                .iter()
                .map(|c| (c.name.clone(), c.value.clone()))
                .collect();
            (
                name.clone(),
                ClassConsts {
                    constants,
                    parents: class.extends.iter().cloned().collect(),
                },
            )
        })
        .collect();
    table.extend(interface_map.iter().map(|(name, interface)| {
        let constants = interface
            .constants
            .iter()
            .map(|constant| (constant.name.clone(), constant.value.clone()))
            .collect();
        (
            name.clone(),
            ClassConsts {
                constants,
                parents: interface.extends.clone(),
            },
        )
    }));
    table
}

/// Attempts to resolve a single default expression to a scalar literal.
///
/// Returns `Some(literal_expr)` only when `default` is a class-constant reference that resolves
/// (possibly through further constant indirections) to a scalar literal value. Returns `None`
/// for non-constant-reference defaults and for references that cannot be resolved to a literal,
/// leaving the original default untouched.
fn resolve_const_default(
    default: &Expr,
    current_class: &str,
    table: &HashMap<String, ClassConsts>,
) -> Option<Expr> {
    let ExprKind::ScopedConstantAccess { .. } = &default.kind else {
        return None;
    };
    let mut visited = HashSet::new();
    resolve_const_chain(default, current_class, table, &mut visited, 0)
}

/// Recursively follows a constant reference to its underlying scalar literal value.
///
/// `current_class` is the lexical class against which `self`/`parent`/`static` receivers are
/// resolved. `visited` records `(class, constant)` pairs already seen to break cycles, and
/// `depth` bounds the indirection chain. Returns the resolved scalar literal expression, or
/// `None` if the reference does not resolve to a literal.
fn resolve_const_chain(
    expr: &Expr,
    current_class: &str,
    table: &HashMap<String, ClassConsts>,
    visited: &mut HashSet<(String, String)>,
    depth: usize,
) -> Option<Expr> {
    if depth > MAX_CONST_RESOLUTION_DEPTH {
        return None;
    }
    match &expr.kind {
        ExprKind::ScopedConstantAccess { receiver, name } => {
            let owning_class = receiver_class(receiver, current_class, table)?;
            if !visited.insert((owning_class.clone(), name.clone())) {
                return None;
            }
            let (value, defining_class) = lookup_constant(&owning_class, name, table)?;
            resolve_const_chain(&value, &defining_class, table, visited, depth + 1)
        }
        _ if is_scalar_literal(expr) => Some(expr.clone()),
        _ => None,
    }
}

/// Resolves a `StaticReceiver` in a constant reference to the fully-qualified class name that
/// owns the constant lookup. `self`/`static` resolve to the lexical class; `parent` resolves to
/// its parent (or `None` when there is no parent or no lexical class context).
fn receiver_class(
    receiver: &StaticReceiver,
    current_class: &str,
    table: &HashMap<String, ClassConsts>,
) -> Option<String> {
    match receiver {
        StaticReceiver::Named(name) => Some(name.as_canonical()),
        StaticReceiver::Self_ | StaticReceiver::Static => {
            (!current_class.is_empty()).then(|| current_class.to_string())
        }
        StaticReceiver::Parent => table
            .get(current_class)
            .and_then(|class| class.parents.first().cloned()),
    }
}

/// Looks up a class-like constant by name, walking class or interface parent links.
///
/// Returns the value expression with the symbol that declares it, which supplies the lexical
/// context for resolving nested `self`/`parent` references in that value.
fn lookup_constant(
    class_name: &str,
    const_name: &str,
    table: &HashMap<String, ClassConsts>,
) -> Option<(Expr, String)> {
    let mut pending = vec![class_name.to_string()];
    let mut visited = HashSet::new();
    while let Some(cn) = pending.pop() {
        if !visited.insert(cn.clone()) {
            continue;
        }
        let Some(class) = table.get(&cn) else {
            continue;
        };
        if let Some(value) = class.constants.get(const_name) {
            return Some((value.clone(), cn));
        }
        pending.extend(class.parents.iter().rev().cloned());
    }
    None
}

/// Reports whether `expr` is a scalar literal that can serve as a default for both type inference
/// and codegen: integer, float, boolean, string, null, or the negation of an integer/float
/// literal.
fn is_scalar_literal(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::IntLiteral(_)
        | ExprKind::FloatLiteral(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::StringLiteral(_)
        | ExprKind::Null => true,
        ExprKind::Negate(inner) => {
            matches!(inner.kind, ExprKind::IntLiteral(_) | ExprKind::FloatLiteral(_))
        }
        _ => false,
    }
}
