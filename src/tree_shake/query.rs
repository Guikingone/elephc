//! Purpose:
//! Answers the structural questions later tree-shaking stages ask of the `Skeleton`:
//! subtype/implementer enumeration, instantiability, and inherited-method resolution.
//!
//! Called from:
//! - Stage 2 reachability (virtual-dispatch fan-out, `new`/interface handling).
//!
//! Key details:
//! - All walks operate over the CLOSED class world: a super-type absent from the map is
//!   external/unknown and stops the walk (no error, no panic).
//! - Every traversal carries a `visited` guard so malformed cyclic hierarchies terminate.

use std::collections::HashSet;

use super::skeleton::{strip_root, ClassKind, MethodSkel, Skeleton};

impl Skeleton {
    /// Returns `fqn` itself plus every class/interface/trait/enum that transitively
    /// extends or implements it, over the closed hierarchy. The returned vector always
    /// contains the queried type (even if it is external/undeclared). Robust to cycles
    /// and to super-types that are not present in the map.
    pub fn subtypes(&self, fqn: &str) -> Vec<String> {
        let target = strip_root(fqn);
        let mut result = vec![target.to_string()];
        for name in self.classes.keys() {
            if name.as_str() == target {
                continue;
            }
            if self.has_ancestor(name, target) {
                result.push(name.clone());
            }
        }
        result
    }

    /// Returns the concrete class-likes (classes and enums) that implement `iface`,
    /// directly or through any chain of ancestry (super-classes, extended interfaces).
    /// Interfaces and traits are excluded — they declare, but do not implement, behavior.
    pub fn implementers(&self, iface: &str) -> Vec<String> {
        let target = strip_root(iface);
        let mut result = Vec::new();
        for (name, class) in &self.classes {
            if !matches!(class.kind, ClassKind::Class | ClassKind::Enum) {
                continue;
            }
            if self.has_ancestor(name, target) {
                result.push(name.clone());
            }
        }
        result
    }

    /// Returns `true` when `fqn` names a concrete class present in the map — that is, a
    /// `class` (not interface/trait/enum) that is not `abstract`. Absent types are not
    /// instantiable.
    pub fn is_instantiable(&self, fqn: &str) -> bool {
        match self.classes.get(strip_root(fqn)) {
            Some(class) => matches!(class.kind, ClassKind::Class) && !class.is_abstract,
            None => false,
        }
    }

    /// Resolves `lower_name` (a lowercase method key) by walking `class` and then its
    /// parent chain, returning the declaring-class FQN and the method. Trait methods are
    /// already folded onto each class, so they resolve on the using class directly.
    /// Returns `None` when the method is not found before the chain leaves the closed
    /// world (an external parent) or a cycle is detected.
    pub fn resolve_method(&self, class: &str, lower_name: &str) -> Option<(String, &MethodSkel)> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut current = Some(strip_root(class).to_string());
        while let Some(cur) = current {
            if !visited.insert(cur.clone()) {
                return None;
            }
            let class_skel = self.classes.get(&cur)?;
            if let Some(method) = class_skel.methods.get(lower_name) {
                return Some((cur, method));
            }
            current = class_skel.parent.as_ref().map(|parent| strip_root(parent).to_string());
        }
        None
    }

    /// Returns `true` when `target` is a transitive super-type (parent class, implemented
    /// interface, or extended interface) of `start`. Follows both the parent link and the
    /// interface list, guarding against cycles and stopping at external boundaries.
    fn has_ancestor(&self, start: &str, target: &str) -> bool {
        let mut visited: HashSet<String> = HashSet::new();
        let mut stack: Vec<String> = Vec::new();
        if let Some(class) = self.classes.get(strip_root(start)) {
            push_supers(class, &mut stack);
        }
        while let Some(current) = stack.pop() {
            let current = strip_root(&current).to_string();
            if current == target {
                return true;
            }
            if !visited.insert(current.clone()) {
                continue;
            }
            // An absent super-type is external; `get` yields `None` and the branch ends.
            if let Some(class) = self.classes.get(&current) {
                push_supers(class, &mut stack);
            }
        }
        false
    }
}

/// Pushes a class-like's direct super-types (parent class, then interfaces) onto the walk
/// stack. Shared by the ancestor search so parent and interface edges are followed uniformly.
fn push_supers(class: &super::skeleton::ClassSkel, stack: &mut Vec<String>) {
    if let Some(parent) = &class.parent {
        stack.push(parent.clone());
    }
    for interface in &class.interfaces {
        stack.push(interface.clone());
    }
}
