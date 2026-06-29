//! Purpose:
//! Resolves class-constant references used as default values (class property defaults and class
//! method/constructor parameter defaults) into the referenced constant's literal value, before
//! class schema metadata is built.
//!
//! Called from:
//! - `crate::types::checker::driver::init` (after `class_map` is fully populated).
//!
//! Key details:
//! - Runs on the complete `class_map`, so it is order-independent: a default in a class declared
//!   before the class it references (`B` before `A`) still resolves.
//! - Rewrites the default expression in place to the resolved scalar literal, so both syntactic
//!   type inference and codegen see a literal instead of a `ScopedConstantAccess`. For property
//!   defaults this also satisfies the codegen property-default emitter, which only accepts literal
//!   forms (`literal_default_value`).
//! - Only scalar literal results are substituted; non-literal constant values (arrays, computed
//!   expressions, enum cases, unresolvable references) are left untouched so existing behavior is
//!   preserved and codegen never receives an unsupported default form.

use std::collections::{HashMap, HashSet};

use crate::parser::ast::{Expr, ExprKind, StaticReceiver};
use crate::types::traits::FlattenedClass;

/// Maximum number of constant indirections to follow when resolving a default
/// (`A::X` where `const X = B::Y` where `const Y = 1`). Guards against cyclic constant
/// definitions producing unbounded recursion.
const MAX_CONST_RESOLUTION_DEPTH: usize = 32;

/// A class's constant value expressions plus its parent link, used for order-independent
/// constant resolution while defaults are rewritten.
struct ClassConsts {
    /// Constant name (case-sensitive, matching PHP) → declared value expression.
    constants: HashMap<String, Expr>,
    /// Fully-qualified parent class name, for walking the inheritance chain.
    parent: Option<String>,
}

/// Rewrites every class property default and class method/constructor parameter default that is a
/// class-constant reference (`A::X`, `self::X`, `parent::X`, `static::X`) into the referenced
/// constant's scalar literal value, when that value resolves to a literal.
///
/// Operates on the fully-populated `class_map` so resolution does not depend on declaration order.
/// Non-resolvable references and non-literal constant values are left unchanged.
pub(crate) fn resolve_const_default_references(class_map: &mut HashMap<String, FlattenedClass>) {
    let const_table = build_const_table(class_map);
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

/// Builds the order-independent constant lookup table from every class in `class_map`.
fn build_const_table(class_map: &HashMap<String, FlattenedClass>) -> HashMap<String, ClassConsts> {
    class_map
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
                    parent: class.extends.clone(),
                },
            )
        })
        .collect()
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
        StaticReceiver::Parent => table.get(current_class).and_then(|c| c.parent.clone()),
    }
}

/// Looks up a class constant by name, walking the parent chain. Returns the constant's value
/// expression together with the class that actually declares it (used as the lexical context for
/// resolving any nested `self`/`parent` references in that value).
fn lookup_constant(
    class_name: &str,
    const_name: &str,
    table: &HashMap<String, ClassConsts>,
) -> Option<(Expr, String)> {
    let mut current = Some(class_name.to_string());
    while let Some(cn) = current {
        let class = table.get(&cn)?;
        if let Some(value) = class.constants.get(const_name) {
            return Some((value.clone(), cn));
        }
        current = class.parent.clone();
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
